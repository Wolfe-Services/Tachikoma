# 407 - Feature Flag Audit Logging

## Overview

Comprehensive audit logging for feature flag changes, providing compliance, debugging, and operational visibility.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/audit.rs

use crate::definition::FlagDefinition;
use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Types of auditable actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Flag lifecycle
    FlagCreated,
    FlagUpdated,
    FlagDeleted,
    FlagArchived,

    // Status changes
    FlagEnabled,
    FlagDisabled,
    FlagDeprecated,

    // Targeting changes
    RuleAdded,
    RuleUpdated,
    RuleRemoved,
    RolloutChanged,
    ExperimentStarted,
    ExperimentStopped,

    // Override changes
    OverrideAdded,
    OverrideRemoved,
    EmergencyOverride,

    // Environment changes
    EnvironmentEnabled,
    EnvironmentDisabled,

    // Access
    FlagViewed,
    FlagExported,
}

/// An audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Action that was performed
    pub action: AuditAction,
    /// Flag that was affected
    pub flag_id: String,
    /// User who performed the action
    pub actor: Actor,
    /// Previous state (for updates)
    pub previous_state: Option<serde_json::Value>,
    /// New state (for creates/updates)
    pub new_state: Option<serde_json::Value>,
    /// Human-readable description of the change
    pub description: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Request context (IP, user agent, etc.)
    pub request_context: RequestContext,
    /// Environment where the change was made
    pub environment: String,
}

/// Actor who performed an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Actor type
    pub actor_type: ActorType,
    /// Actor identifier
    pub id: String,
    /// Display name
    pub name: Option<String>,
    /// Email (for human actors)
    pub email: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    User,
    ServiceAccount,
    System,
    Api,
}

/// Request context for audit entries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestContext {
    /// Client IP address
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// API key ID (if API access)
    pub api_key_id: Option<String>,
}

/// Query options for audit log
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    /// Filter by flag ID
    pub flag_id: Option<String>,
    /// Filter by action types
    pub actions: Vec<AuditAction>,
    /// Filter by actor ID
    pub actor_id: Option<String>,
    /// Filter by actor type
    pub actor_type: Option<ActorType>,
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Environment filter
    pub environment: Option<String>,
    /// Pagination offset
    pub offset: usize,
    /// Pagination limit
    pub limit: usize,
}

/// Audit log storage trait
#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// Record an audit entry
    async fn record(&self, entry: AuditEntry) -> Result<(), AuditError>;

    /// Query audit entries
    async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEntry>, AuditError>;

    /// Get a specific entry by ID
    async fn get(&self, id: &str) -> Result<Option<AuditEntry>, AuditError>;

    /// Get entries for a specific flag
    async fn get_flag_history(&self, flag_id: &str, limit: usize) -> Result<Vec<AuditEntry>, AuditError>;

    /// Count entries matching query
    async fn count(&self, query: AuditQuery) -> Result<usize, AuditError>;
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Entry not found: {0}")]
    NotFound(String),
}

/// Audit logger service
pub struct AuditLogger {
    storage: Arc<dyn AuditStorage>,
    default_environment: String,
}

impl AuditLogger {
    pub fn new(storage: Arc<dyn AuditStorage>, environment: &str) -> Self {
        Self {
            storage,
            default_environment: environment.to_string(),
        }
    }

    /// Log flag creation
    pub async fn log_flag_created(
        &self,
        flag: &FlagDefinition,
        actor: Actor,
        context: RequestContext,
    ) -> Result<(), AuditError> {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action: AuditAction::FlagCreated,
            flag_id: flag.id.as_str().to_string(),
            actor,
            previous_state: None,
            new_state: Some(serde_json::to_value(flag)?),
            description: format!("Flag '{}' was created", flag.name),
            metadata: HashMap::new(),
            request_context: context,
            environment: self.default_environment.clone(),
        };

        self.storage.record(entry).await
    }

    /// Log flag update
    pub async fn log_flag_updated(
        &self,
        old_flag: &FlagDefinition,
        new_flag: &FlagDefinition,
        actor: Actor,
        context: RequestContext,
    ) -> Result<(), AuditError> {
        let changes = self.compute_changes(old_flag, new_flag);

        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action: AuditAction::FlagUpdated,
            flag_id: new_flag.id.as_str().to_string(),
            actor,
            previous_state: Some(serde_json::to_value(old_flag)?),
            new_state: Some(serde_json::to_value(new_flag)?),
            description: format!("Flag '{}' was updated: {}", new_flag.name, changes.join(", ")),
            metadata: HashMap::new(),
            request_context: context,
            environment: self.default_environment.clone(),
        };

        self.storage.record(entry).await
    }

    /// Log flag deletion
    pub async fn log_flag_deleted(
        &self,
        flag: &FlagDefinition,
        actor: Actor,
        context: RequestContext,
    ) -> Result<(), AuditError> {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action: AuditAction::FlagDeleted,
            flag_id: flag.id.as_str().to_string(),
            actor,
            previous_state: Some(serde_json::to_value(flag)?),
            new_state: None,
            description: format!("Flag '{}' was deleted", flag.name),
            metadata: HashMap::new(),
            request_context: context,
            environment: self.default_environment.clone(),
        };

        self.storage.record(entry).await
    }

    /// Log status change
    pub async fn log_status_change(
        &self,
        flag_id: &FlagId,
        old_status: &str,
        new_status: &str,
        actor: Actor,
        context: RequestContext,
    ) -> Result<(), AuditError> {
        let action = match new_status {
            "active" => AuditAction::FlagEnabled,
            "disabled" => AuditAction::FlagDisabled,
            "deprecated" => AuditAction::FlagDeprecated,
            "archived" => AuditAction::FlagArchived,
            _ => AuditAction::FlagUpdated,
        };

        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            flag_id: flag_id.as_str().to_string(),
            actor,
            previous_state: Some(serde_json::json!({ "status": old_status })),
            new_state: Some(serde_json::json!({ "status": new_status })),
            description: format!("Flag status changed from '{}' to '{}'", old_status, new_status),
            metadata: HashMap::new(),
            request_context: context,
            environment: self.default_environment.clone(),
        };

        self.storage.record(entry).await
    }

    /// Log override change
    pub async fn log_override(
        &self,
        flag_id: &FlagId,
        override_type: &str,
        target_id: &str,
        value: &serde_json::Value,
        actor: Actor,
        context: RequestContext,
        is_emergency: bool,
    ) -> Result<(), AuditError> {
        let action = if is_emergency {
            AuditAction::EmergencyOverride
        } else {
            AuditAction::OverrideAdded
        };

        let mut metadata = HashMap::new();
        metadata.insert("override_type".to_string(), serde_json::json!(override_type));
        metadata.insert("target_id".to_string(), serde_json::json!(target_id));

        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            flag_id: flag_id.as_str().to_string(),
            actor,
            previous_state: None,
            new_state: Some(value.clone()),
            description: format!(
                "{} override added for {} '{}'",
                if is_emergency { "Emergency" } else { "User" },
                override_type,
                target_id
            ),
            metadata,
            request_context: context,
            environment: self.default_environment.clone(),
        };

        self.storage.record(entry).await
    }

    /// Compute human-readable changes between flag versions
    fn compute_changes(&self, old: &FlagDefinition, new: &FlagDefinition) -> Vec<String> {
        let mut changes = Vec::new();

        if old.name != new.name {
            changes.push(format!("name changed from '{}' to '{}'", old.name, new.name));
        }

        if old.status != new.status {
            changes.push(format!("status changed from '{:?}' to '{:?}'", old.status, new.status));
        }

        if old.default_value != new.default_value {
            changes.push("default value changed".to_string());
        }

        if old.rules.len() != new.rules.len() {
            changes.push(format!(
                "rules count changed from {} to {}",
                old.rules.len(),
                new.rules.len()
            ));
        }

        if old.rollout != new.rollout {
            changes.push("rollout configuration changed".to_string());
        }

        if changes.is_empty() {
            changes.push("configuration updated".to_string());
        }

        changes
    }

    /// Query audit log
    pub async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEntry>, AuditError> {
        self.storage.query(query).await
    }

    /// Get flag history
    pub async fn get_flag_history(&self, flag_id: &str, limit: usize) -> Result<Vec<AuditEntry>, AuditError> {
        self.storage.get_flag_history(flag_id, limit).await
    }
}

/// In-memory audit storage (for development/testing)
pub struct InMemoryAuditStorage {
    entries: RwLock<Vec<AuditEntry>>,
    max_entries: usize,
}

impl InMemoryAuditStorage {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            max_entries,
        }
    }
}

#[async_trait]
impl AuditStorage for InMemoryAuditStorage {
    async fn record(&self, entry: AuditEntry) -> Result<(), AuditError> {
        let mut entries = self.entries.write().await;

        if entries.len() >= self.max_entries {
            entries.remove(0);
        }

        entries.push(entry);
        Ok(())
    }

    async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEntry>, AuditError> {
        let entries = self.entries.read().await;

        let filtered: Vec<_> = entries.iter()
            .filter(|e| {
                if let Some(ref flag_id) = query.flag_id {
                    if &e.flag_id != flag_id {
                        return false;
                    }
                }

                if !query.actions.is_empty() && !query.actions.contains(&e.action) {
                    return false;
                }

                if let Some(ref actor_id) = query.actor_id {
                    if &e.actor.id != actor_id {
                        return false;
                    }
                }

                if let Some(start) = query.start_time {
                    if e.timestamp < start {
                        return false;
                    }
                }

                if let Some(end) = query.end_time {
                    if e.timestamp > end {
                        return false;
                    }
                }

                true
            })
            .skip(query.offset)
            .take(query.limit)
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn get(&self, id: &str) -> Result<Option<AuditEntry>, AuditError> {
        let entries = self.entries.read().await;
        Ok(entries.iter().find(|e| e.id == id).cloned())
    }

    async fn get_flag_history(&self, flag_id: &str, limit: usize) -> Result<Vec<AuditEntry>, AuditError> {
        let entries = self.entries.read().await;

        let history: Vec<_> = entries.iter()
            .filter(|e| e.flag_id == flag_id)
            .rev()
            .take(limit)
            .cloned()
            .collect();

        Ok(history)
    }

    async fn count(&self, query: AuditQuery) -> Result<usize, AuditError> {
        let result = self.query(AuditQuery { limit: usize::MAX, ..query }).await?;
        Ok(result.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor() -> Actor {
        Actor {
            actor_type: ActorType::User,
            id: "user-123".to_string(),
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
        }
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let storage = Arc::new(InMemoryAuditStorage::new(1000));
        let logger = AuditLogger::new(storage.clone(), "production");

        let flag = FlagDefinition::new_boolean("test-flag", "Test Flag", false).unwrap();

        logger.log_flag_created(
            &flag,
            test_actor(),
            RequestContext::default(),
        ).await.unwrap();

        let entries = storage.query(AuditQuery {
            flag_id: Some("test-flag".to_string()),
            limit: 100,
            ..Default::default()
        }).await.unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, AuditAction::FlagCreated);
    }
}
```

## Database Schema

```sql
CREATE TABLE flag_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    action VARCHAR(50) NOT NULL,
    flag_id VARCHAR(256) NOT NULL,
    actor_type VARCHAR(50) NOT NULL,
    actor_id VARCHAR(256) NOT NULL,
    actor_name VARCHAR(256),
    actor_email VARCHAR(256),
    previous_state JSONB,
    new_state JSONB,
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    request_id VARCHAR(256),
    environment VARCHAR(50) NOT NULL
);

CREATE INDEX idx_audit_flag_id ON flag_audit_log(flag_id);
CREATE INDEX idx_audit_timestamp ON flag_audit_log(timestamp DESC);
CREATE INDEX idx_audit_actor ON flag_audit_log(actor_id);
CREATE INDEX idx_audit_action ON flag_audit_log(action);
```

## Related Specs

- 400-flag-override.md - Override audit
- 392-flag-definition.md - Flag changes
- 409-flag-api.md - API audit integration
