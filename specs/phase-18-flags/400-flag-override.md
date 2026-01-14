# 400 - Feature Flag Override

## Overview

Override mechanisms for feature flags allowing development testing, QA, and special user handling.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/override.rs

use crate::types::*;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Override manager for feature flags
pub struct OverrideManager {
    /// User-level overrides
    user_overrides: RwLock<HashMap<String, HashMap<FlagId, Override>>>,
    /// Session-level overrides (temporary)
    session_overrides: RwLock<HashMap<String, HashMap<FlagId, Override>>>,
    /// Global overrides (affects all users)
    global_overrides: RwLock<HashMap<FlagId, Override>>,
    /// Override audit log
    audit_log: RwLock<Vec<OverrideAuditEntry>>,
}

/// An override for a feature flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Override {
    /// The flag being overridden
    pub flag_id: FlagId,
    /// The override value
    pub value: FlagValue,
    /// Override type
    pub override_type: OverrideType,
    /// When the override expires (None = never)
    pub expires_at: Option<DateTime<Utc>>,
    /// Who created the override
    pub created_by: String,
    /// When the override was created
    pub created_at: DateTime<Utc>,
    /// Reason for the override
    pub reason: Option<String>,
    /// Override priority (higher = checked first)
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverrideType {
    /// User-specific override
    User,
    /// Session-specific override
    Session,
    /// Global override (all users)
    Global,
    /// Testing override
    Testing,
    /// Emergency override
    Emergency,
}

/// Audit entry for override changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideAuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: OverrideAction,
    pub flag_id: FlagId,
    pub override_type: OverrideType,
    pub target_id: Option<String>,
    pub value: Option<FlagValue>,
    pub actor: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverrideAction {
    Created,
    Updated,
    Removed,
    Expired,
}

impl OverrideManager {
    pub fn new() -> Self {
        Self {
            user_overrides: RwLock::new(HashMap::new()),
            session_overrides: RwLock::new(HashMap::new()),
            global_overrides: RwLock::new(HashMap::new()),
            audit_log: RwLock::new(Vec::new()),
        }
    }

    /// Set a user-level override
    pub async fn set_user_override(
        &self,
        user_id: &str,
        flag_id: FlagId,
        value: FlagValue,
        actor: &str,
        reason: Option<&str>,
        expires_in: Option<Duration>,
    ) {
        let override_entry = Override {
            flag_id: flag_id.clone(),
            value: value.clone(),
            override_type: OverrideType::User,
            expires_at: expires_in.map(|d| Utc::now() + d),
            created_by: actor.to_string(),
            created_at: Utc::now(),
            reason: reason.map(|s| s.to_string()),
            priority: 100,
        };

        let mut overrides = self.user_overrides.write().await;
        let user_overrides = overrides
            .entry(user_id.to_string())
            .or_insert_with(HashMap::new);
        user_overrides.insert(flag_id.clone(), override_entry);

        self.log_audit(
            OverrideAction::Created,
            flag_id,
            OverrideType::User,
            Some(user_id.to_string()),
            Some(value),
            actor,
            reason,
        ).await;
    }

    /// Set a session-level override (temporary)
    pub async fn set_session_override(
        &self,
        session_id: &str,
        flag_id: FlagId,
        value: FlagValue,
    ) {
        let override_entry = Override {
            flag_id: flag_id.clone(),
            value,
            override_type: OverrideType::Session,
            expires_at: Some(Utc::now() + Duration::hours(24)),
            created_by: "session".to_string(),
            created_at: Utc::now(),
            reason: None,
            priority: 200, // Session overrides have higher priority
        };

        let mut overrides = self.session_overrides.write().await;
        let session_overrides = overrides
            .entry(session_id.to_string())
            .or_insert_with(HashMap::new);
        session_overrides.insert(flag_id, override_entry);
    }

    /// Set a global override (affects all users)
    pub async fn set_global_override(
        &self,
        flag_id: FlagId,
        value: FlagValue,
        actor: &str,
        reason: Option<&str>,
        expires_in: Option<Duration>,
    ) {
        let override_entry = Override {
            flag_id: flag_id.clone(),
            value: value.clone(),
            override_type: OverrideType::Global,
            expires_at: expires_in.map(|d| Utc::now() + d),
            created_by: actor.to_string(),
            created_at: Utc::now(),
            reason: reason.map(|s| s.to_string()),
            priority: 50, // Global overrides have lower priority than user/session
        };

        let mut overrides = self.global_overrides.write().await;
        overrides.insert(flag_id.clone(), override_entry);

        self.log_audit(
            OverrideAction::Created,
            flag_id,
            OverrideType::Global,
            None,
            Some(value),
            actor,
            reason,
        ).await;
    }

    /// Set an emergency override (highest priority)
    pub async fn set_emergency_override(
        &self,
        flag_id: FlagId,
        value: FlagValue,
        actor: &str,
        reason: &str,
    ) {
        let override_entry = Override {
            flag_id: flag_id.clone(),
            value: value.clone(),
            override_type: OverrideType::Emergency,
            expires_at: None, // Emergency overrides don't expire automatically
            created_by: actor.to_string(),
            created_at: Utc::now(),
            reason: Some(reason.to_string()),
            priority: 1000, // Highest priority
        };

        let mut overrides = self.global_overrides.write().await;
        overrides.insert(flag_id.clone(), override_entry);

        self.log_audit(
            OverrideAction::Created,
            flag_id,
            OverrideType::Emergency,
            None,
            Some(value),
            actor,
            Some(reason),
        ).await;
    }

    /// Get override for a specific context
    pub async fn get_override(
        &self,
        flag_id: &FlagId,
        user_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Option<Override> {
        let now = Utc::now();
        let mut candidates: Vec<Override> = Vec::new();

        // Check session overrides
        if let Some(sid) = session_id {
            let session_overrides = self.session_overrides.read().await;
            if let Some(user_map) = session_overrides.get(sid) {
                if let Some(ovr) = user_map.get(flag_id) {
                    if ovr.expires_at.map(|e| e > now).unwrap_or(true) {
                        candidates.push(ovr.clone());
                    }
                }
            }
        }

        // Check user overrides
        if let Some(uid) = user_id {
            let user_overrides = self.user_overrides.read().await;
            if let Some(user_map) = user_overrides.get(uid) {
                if let Some(ovr) = user_map.get(flag_id) {
                    if ovr.expires_at.map(|e| e > now).unwrap_or(true) {
                        candidates.push(ovr.clone());
                    }
                }
            }
        }

        // Check global overrides
        let global_overrides = self.global_overrides.read().await;
        if let Some(ovr) = global_overrides.get(flag_id) {
            if ovr.expires_at.map(|e| e > now).unwrap_or(true) {
                candidates.push(ovr.clone());
            }
        }

        // Return highest priority override
        candidates.sort_by(|a, b| b.priority.cmp(&a.priority));
        candidates.into_iter().next()
    }

    /// Remove a user override
    pub async fn remove_user_override(
        &self,
        user_id: &str,
        flag_id: &FlagId,
        actor: &str,
    ) {
        let mut overrides = self.user_overrides.write().await;
        if let Some(user_map) = overrides.get_mut(user_id) {
            user_map.remove(flag_id);
        }

        self.log_audit(
            OverrideAction::Removed,
            flag_id.clone(),
            OverrideType::User,
            Some(user_id.to_string()),
            None,
            actor,
            None,
        ).await;
    }

    /// Remove a global override
    pub async fn remove_global_override(
        &self,
        flag_id: &FlagId,
        actor: &str,
    ) {
        let mut overrides = self.global_overrides.write().await;
        overrides.remove(flag_id);

        self.log_audit(
            OverrideAction::Removed,
            flag_id.clone(),
            OverrideType::Global,
            None,
            None,
            actor,
            None,
        ).await;
    }

    /// Clean up expired overrides
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();

        // Clean session overrides
        {
            let mut session_overrides = self.session_overrides.write().await;
            for user_map in session_overrides.values_mut() {
                user_map.retain(|_, ovr| {
                    ovr.expires_at.map(|e| e > now).unwrap_or(true)
                });
            }
            session_overrides.retain(|_, v| !v.is_empty());
        }

        // Clean user overrides
        {
            let mut user_overrides = self.user_overrides.write().await;
            for user_map in user_overrides.values_mut() {
                user_map.retain(|_, ovr| {
                    ovr.expires_at.map(|e| e > now).unwrap_or(true)
                });
            }
            user_overrides.retain(|_, v| !v.is_empty());
        }

        // Clean global overrides (except emergency)
        {
            let mut global_overrides = self.global_overrides.write().await;
            global_overrides.retain(|_, ovr| {
                ovr.override_type == OverrideType::Emergency ||
                ovr.expires_at.map(|e| e > now).unwrap_or(true)
            });
        }
    }

    /// Get all overrides for a user
    pub async fn get_user_overrides(&self, user_id: &str) -> Vec<Override> {
        let overrides = self.user_overrides.read().await;
        overrides.get(user_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all global overrides
    pub async fn get_global_overrides(&self) -> Vec<Override> {
        let overrides = self.global_overrides.read().await;
        overrides.values().cloned().collect()
    }

    /// Get audit log entries
    pub async fn get_audit_log(&self, limit: usize) -> Vec<OverrideAuditEntry> {
        let log = self.audit_log.read().await;
        log.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    async fn log_audit(
        &self,
        action: OverrideAction,
        flag_id: FlagId,
        override_type: OverrideType,
        target_id: Option<String>,
        value: Option<FlagValue>,
        actor: &str,
        reason: Option<&str>,
    ) {
        let entry = OverrideAuditEntry {
            timestamp: Utc::now(),
            action,
            flag_id,
            override_type,
            target_id,
            value,
            actor: actor.to_string(),
            reason: reason.map(|s| s.to_string()),
        };

        let mut log = self.audit_log.write().await;
        log.push(entry);

        // Keep only last 10000 entries
        if log.len() > 10000 {
            log.drain(0..1000);
        }
    }
}

impl Default for OverrideManager {
    fn default() -> Self {
        Self::new()
    }
}

/// URL-based override handler for development/testing
pub struct UrlOverrideHandler {
    /// Query parameter prefix for flag overrides
    pub param_prefix: String,
    /// Allowed environments for URL overrides
    pub allowed_environments: Vec<String>,
}

impl UrlOverrideHandler {
    pub fn new(prefix: &str) -> Self {
        Self {
            param_prefix: prefix.to_string(),
            allowed_environments: vec!["development".to_string(), "staging".to_string()],
        }
    }

    /// Parse flag overrides from URL query string
    pub fn parse_overrides(&self, query_string: &str) -> HashMap<FlagId, FlagValue> {
        let mut overrides = HashMap::new();

        for param in query_string.split('&') {
            let parts: Vec<&str> = param.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0];
                let value = parts[1];

                if key.starts_with(&self.param_prefix) {
                    let flag_key = key.trim_start_matches(&self.param_prefix);
                    let flag_id = FlagId::new(flag_key);

                    let flag_value = match value.to_lowercase().as_str() {
                        "true" | "1" | "on" => FlagValue::Boolean(true),
                        "false" | "0" | "off" => FlagValue::Boolean(false),
                        _ => FlagValue::String(value.to_string()),
                    };

                    overrides.insert(flag_id, flag_value);
                }
            }
        }

        overrides
    }

    /// Check if URL overrides are allowed in the current environment
    pub fn is_allowed(&self, environment: &str) -> bool {
        self.allowed_environments.contains(&environment.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_override() {
        let manager = OverrideManager::new();

        manager.set_user_override(
            "user-123",
            FlagId::new("test-flag"),
            FlagValue::Boolean(true),
            "admin",
            Some("Testing"),
            None,
        ).await;

        let ovr = manager.get_override(
            &FlagId::new("test-flag"),
            Some("user-123"),
            None,
        ).await;

        assert!(ovr.is_some());
        assert_eq!(ovr.unwrap().value.as_bool(), Some(true));
    }

    #[tokio::test]
    async fn test_override_priority() {
        let manager = OverrideManager::new();

        // Set global override
        manager.set_global_override(
            FlagId::new("test-flag"),
            FlagValue::Boolean(false),
            "admin",
            None,
            None,
        ).await;

        // Set user override (should take priority)
        manager.set_user_override(
            "user-123",
            FlagId::new("test-flag"),
            FlagValue::Boolean(true),
            "admin",
            None,
            None,
        ).await;

        let ovr = manager.get_override(
            &FlagId::new("test-flag"),
            Some("user-123"),
            None,
        ).await;

        assert_eq!(ovr.unwrap().value.as_bool(), Some(true));
    }

    #[tokio::test]
    async fn test_emergency_override() {
        let manager = OverrideManager::new();

        manager.set_emergency_override(
            FlagId::new("broken-feature"),
            FlagValue::Boolean(false),
            "oncall",
            "Feature causing outage",
        ).await;

        let ovr = manager.get_override(
            &FlagId::new("broken-feature"),
            None,
            None,
        ).await;

        assert!(ovr.is_some());
        let override_entry = ovr.unwrap();
        assert_eq!(override_entry.override_type, OverrideType::Emergency);
        assert_eq!(override_entry.priority, 1000);
    }

    #[test]
    fn test_url_override_parsing() {
        let handler = UrlOverrideHandler::new("ff_");

        let overrides = handler.parse_overrides(
            "ff_new-feature=true&ff_experiment=variant-b&other=value"
        );

        assert_eq!(overrides.len(), 2);
        assert_eq!(
            overrides.get(&FlagId::new("new-feature")).unwrap().as_bool(),
            Some(true)
        );
        assert_eq!(
            overrides.get(&FlagId::new("experiment")).unwrap().as_string(),
            Some("variant-b")
        );
    }
}
```

## Override Priorities

1. **Emergency** (1000) - Highest priority, for incidents
2. **Session** (200) - Dev/QA testing via URL params
3. **User** (100) - User-specific overrides
4. **Global** (50) - System-wide overrides

## Related Specs

- 394-flag-evaluation.md - Evaluation engine
- 407-flag-audit.md - Audit logging
- 401-flag-admin-ui.md - Admin interface
