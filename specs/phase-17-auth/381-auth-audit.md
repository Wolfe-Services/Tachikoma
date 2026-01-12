# Spec 381: Authentication Audit Logging

## Phase
17 - Authentication/Authorization

## Spec ID
381

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration
- Spec 384: Auth Events

## Estimated Context
~10%

---

## Objective

Implement comprehensive audit logging for all authentication and authorization events. This provides a security audit trail for compliance, incident investigation, and monitoring. The audit log should capture relevant context while protecting sensitive information.

---

## Acceptance Criteria

- [ ] Define `AuditEntry` structure with all relevant fields
- [ ] Implement `AuditLogger` for recording events
- [ ] Support multiple storage backends
- [ ] Mask sensitive data in logs
- [ ] Implement audit log queries and filtering
- [ ] Support log retention policies
- [ ] Enable real-time audit alerting (hooks)
- [ ] Provide audit report generation

---

## Implementation Details

### Audit Logging System

```rust
// src/auth/audit.rs

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::auth::{
    config::AuditConfig,
    events::AuthEvent,
    types::*,
};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: String,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event_type: AuditEventType,

    /// User ID (if applicable)
    pub user_id: Option<UserId>,

    /// Username (if applicable)
    pub username: Option<String>,

    /// IP address of the request
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Resource being accessed
    pub resource: Option<String>,

    /// Action performed
    pub action: String,

    /// Whether the action was successful
    pub success: bool,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,

    /// Request ID for correlation
    pub request_id: Option<String>,

    /// Session ID (if applicable)
    pub session_id: Option<String>,

    /// Geographic location (if resolved)
    pub geo_location: Option<GeoInfo>,
}

/// Geographic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoInfo {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
}

/// Audit event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication events
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    Logout,
    SessionCreated,
    SessionDestroyed,
    SessionExpired,

    // Token events
    TokenIssued,
    TokenRefreshed,
    TokenRevoked,
    TokenValidationFailed,

    // Password events
    PasswordChanged,
    PasswordResetRequested,
    PasswordReset,

    // MFA events
    MfaEnabled,
    MfaDisabled,
    MfaVerified,
    MfaFailed,
    BackupCodeUsed,

    // Account events
    AccountCreated,
    AccountUpdated,
    AccountDisabled,
    AccountEnabled,
    AccountLocked,
    AccountUnlocked,
    AccountDeleted,

    // API key events
    ApiKeyCreated,
    ApiKeyUsed,
    ApiKeyRevoked,

    // OAuth events
    OAuthLogin,
    OAuthLinked,
    OAuthUnlinked,

    // Authorization events
    PermissionGranted,
    PermissionRevoked,
    RoleAssigned,
    RoleRemoved,
    AccessDenied,

    // Security events
    SuspiciousActivity,
    BruteForceDetected,
    RateLimitExceeded,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(event_type: AuditEventType, action: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            user_id: None,
            username: None,
            ip_address: None,
            user_agent: None,
            resource: None,
            action: action.into(),
            success: true,
            error: None,
            context: HashMap::new(),
            request_id: None,
            session_id: None,
            geo_location: None,
        }
    }

    /// Set user information
    pub fn with_user(mut self, user_id: UserId, username: Option<String>) -> Self {
        self.user_id = Some(user_id);
        self.username = username;
        self
    }

    /// Set request metadata
    pub fn with_metadata(mut self, metadata: &AuthMetadata) -> Self {
        self.ip_address = metadata.ip_address.clone();
        self.user_agent = metadata.user_agent.clone();
        self.request_id = metadata.request_id.clone();
        if let Some(geo) = &metadata.geo_location {
            self.geo_location = Some(GeoInfo {
                country: geo.country.clone(),
                region: geo.region.clone(),
                city: geo.city.clone(),
            });
        }
        self
    }

    /// Set success status
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    /// Set error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self.success = false;
        self
    }

    /// Add context data
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json) = serde_json::to_value(value) {
            self.context.insert(key.into(), json);
        }
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }
}

/// Audit logger
pub struct AuditLogger {
    storage: Arc<dyn AuditStorage>,
    config: AuditConfig,
    alert_handlers: RwLock<Vec<Arc<dyn AuditAlertHandler>>>,
}

impl AuditLogger {
    pub fn new(storage: Arc<dyn AuditStorage>, config: AuditConfig) -> Self {
        Self {
            storage,
            config,
            alert_handlers: RwLock::new(Vec::new()),
        }
    }

    /// Register an alert handler
    pub async fn register_alert_handler(&self, handler: Arc<dyn AuditAlertHandler>) {
        let mut handlers = self.alert_handlers.write().await;
        handlers.push(handler);
    }

    /// Log an audit entry
    #[instrument(skip(self, entry), fields(event_type = ?entry.event_type))]
    pub async fn log(&self, entry: AuditEntry) -> AuthResult<()> {
        // Check if this event type should be logged
        if !self.should_log(&entry.event_type) {
            return Ok(());
        }

        // Mask sensitive data if configured
        let entry = if self.config.mask_sensitive_data {
            self.mask_sensitive(&entry)
        } else {
            entry
        };

        // Store the entry
        self.storage.store(&entry).await?;

        // Check for alert conditions
        self.check_alerts(&entry).await;

        info!(
            event_type = ?entry.event_type,
            user_id = ?entry.user_id,
            success = %entry.success,
            "Audit event logged"
        );

        Ok(())
    }

    /// Log from an auth event
    pub async fn log_event(&self, event: &AuthEvent, metadata: &AuthMetadata) {
        let entry = self.event_to_entry(event, metadata);
        if let Some(entry) = entry {
            let _ = self.log(entry).await;
        }
    }

    /// Convert auth event to audit entry
    fn event_to_entry(&self, event: &AuthEvent, metadata: &AuthMetadata) -> Option<AuditEntry> {
        match event {
            AuthEvent::LoginSuccess { user_id, auth_method, .. } => {
                Some(AuditEntry::new(AuditEventType::LoginSuccess, "User logged in")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("auth_method", auth_method))
            }
            AuthEvent::LoginFailure { user_id, username, reason, .. } => {
                let mut entry = AuditEntry::new(AuditEventType::LoginFailure, "Login failed")
                    .with_metadata(metadata)
                    .with_error(reason);
                if let Some(uid) = user_id {
                    entry = entry.with_user(*uid, Some(username.clone()));
                } else {
                    entry.username = Some(username.clone());
                }
                Some(entry)
            }
            AuthEvent::Logout { user_id, session_id, .. } => {
                let mut entry = AuditEntry::new(AuditEventType::Logout, "User logged out")
                    .with_user(*user_id, None)
                    .with_metadata(metadata);
                if let Some(sid) = session_id {
                    entry = entry.with_session(*sid);
                }
                Some(entry)
            }
            AuthEvent::PasswordChanged { user_id, .. } => {
                Some(AuditEntry::new(AuditEventType::PasswordChanged, "Password changed")
                    .with_user(*user_id, None)
                    .with_metadata(metadata))
            }
            AuthEvent::PasswordResetRequested { user_id, .. } => {
                Some(AuditEntry::new(AuditEventType::PasswordResetRequested, "Password reset requested")
                    .with_user(*user_id, None)
                    .with_metadata(metadata))
            }
            AuthEvent::PasswordReset { user_id, .. } => {
                Some(AuditEntry::new(AuditEventType::PasswordReset, "Password reset completed")
                    .with_user(*user_id, None)
                    .with_metadata(metadata))
            }
            AuthEvent::MfaEnabled { user_id, .. } => {
                Some(AuditEntry::new(AuditEventType::MfaEnabled, "MFA enabled")
                    .with_user(*user_id, None)
                    .with_metadata(metadata))
            }
            AuthEvent::MfaDisabled { user_id, .. } => {
                Some(AuditEntry::new(AuditEventType::MfaDisabled, "MFA disabled")
                    .with_user(*user_id, None)
                    .with_metadata(metadata))
            }
            AuthEvent::AccountLocked { user_id, reason, .. } => {
                Some(AuditEntry::new(AuditEventType::AccountLocked, "Account locked")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("reason", reason))
            }
            AuthEvent::AccountUnlocked { user_id, .. } => {
                Some(AuditEntry::new(AuditEventType::AccountUnlocked, "Account unlocked")
                    .with_user(*user_id, None)
                    .with_metadata(metadata))
            }
            AuthEvent::ApiKeyCreated { user_id, key_name, .. } => {
                Some(AuditEntry::new(AuditEventType::ApiKeyCreated, "API key created")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("key_name", key_name))
            }
            AuthEvent::ApiKeyRevoked { user_id, key_id, .. } => {
                Some(AuditEntry::new(AuditEventType::ApiKeyRevoked, "API key revoked")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("key_id", key_id))
            }
            AuthEvent::PermissionGranted { user_id, permission, .. } => {
                Some(AuditEntry::new(AuditEventType::PermissionGranted, "Permission granted")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("permission", permission))
            }
            AuthEvent::PermissionRevoked { user_id, permission, .. } => {
                Some(AuditEntry::new(AuditEventType::PermissionRevoked, "Permission revoked")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("permission", permission))
            }
            AuthEvent::RolesAssigned { user_id, role_ids, .. } => {
                Some(AuditEntry::new(AuditEventType::RoleAssigned, "Roles assigned")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("roles", role_ids))
            }
            AuthEvent::RolesRemoved { user_id, role_ids, .. } => {
                Some(AuditEntry::new(AuditEventType::RoleRemoved, "Roles removed")
                    .with_user(*user_id, None)
                    .with_metadata(metadata)
                    .with_context("roles", role_ids))
            }
            _ => None, // Other events not audited
        }
    }

    /// Check if event type should be logged
    fn should_log(&self, event_type: &AuditEventType) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Convert to config event type and check
        let config_event = match event_type {
            AuditEventType::LoginSuccess => crate::auth::config::AuditEventType::LoginSuccess,
            AuditEventType::LoginFailure => crate::auth::config::AuditEventType::LoginFailure,
            AuditEventType::Logout => crate::auth::config::AuditEventType::Logout,
            AuditEventType::PasswordChanged => crate::auth::config::AuditEventType::PasswordChange,
            AuditEventType::PasswordReset => crate::auth::config::AuditEventType::PasswordReset,
            AuditEventType::MfaEnabled => crate::auth::config::AuditEventType::MfaEnabled,
            AuditEventType::MfaDisabled => crate::auth::config::AuditEventType::MfaDisabled,
            AuditEventType::AccountLocked => crate::auth::config::AuditEventType::AccountLocked,
            AuditEventType::AccountUnlocked => crate::auth::config::AuditEventType::AccountUnlocked,
            AuditEventType::TokenRefreshed => crate::auth::config::AuditEventType::TokenRefresh,
            AuditEventType::ApiKeyCreated => crate::auth::config::AuditEventType::ApiKeyCreated,
            AuditEventType::ApiKeyRevoked => crate::auth::config::AuditEventType::ApiKeyRevoked,
            AuditEventType::SessionCreated => crate::auth::config::AuditEventType::SessionCreated,
            AuditEventType::SessionDestroyed => crate::auth::config::AuditEventType::SessionDestroyed,
            _ => return true, // Log by default
        };

        self.config.log_events.contains(&config_event)
    }

    /// Mask sensitive data in entry
    fn mask_sensitive(&self, entry: &AuditEntry) -> AuditEntry {
        let mut masked = entry.clone();

        // Mask IP address (keep first octet)
        if let Some(ip) = &masked.ip_address {
            masked.ip_address = Some(mask_ip(ip));
        }

        // Mask user agent (keep browser name only)
        if let Some(ua) = &masked.user_agent {
            masked.user_agent = Some(mask_user_agent(ua));
        }

        masked
    }

    /// Check for alert conditions
    async fn check_alerts(&self, entry: &AuditEntry) {
        let handlers = self.alert_handlers.read().await;

        for handler in handlers.iter() {
            if handler.should_alert(entry).await {
                let _ = handler.send_alert(entry).await;
            }
        }
    }

    /// Query audit logs
    pub async fn query(&self, query: AuditQuery) -> AuthResult<Vec<AuditEntry>> {
        self.storage.query(query).await
    }

    /// Get recent entries for a user
    pub async fn get_user_history(
        &self,
        user_id: UserId,
        limit: usize,
    ) -> AuthResult<Vec<AuditEntry>> {
        self.storage.query(AuditQuery {
            user_id: Some(user_id),
            limit: Some(limit),
            ..Default::default()
        }).await
    }

    /// Apply retention policy
    pub async fn apply_retention(&self) -> AuthResult<usize> {
        let cutoff = Utc::now() - Duration::days(self.config.retention_days as i64);
        self.storage.delete_before(cutoff).await
    }
}

/// Mask IP address
fn mask_ip(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.xxx.xxx.xxx", parts[0])
    } else {
        "xxx.xxx.xxx.xxx".to_string()
    }
}

/// Mask user agent
fn mask_user_agent(ua: &str) -> String {
    // Extract browser name
    let browsers = ["Chrome", "Firefox", "Safari", "Edge", "Opera"];
    for browser in browsers {
        if ua.contains(browser) {
            return browser.to_string();
        }
    }
    "Unknown".to_string()
}

/// Audit query parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQuery {
    pub user_id: Option<UserId>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub success_only: Option<bool>,
    pub ip_address: Option<String>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Audit storage trait
#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// Store an audit entry
    async fn store(&self, entry: &AuditEntry) -> AuthResult<()>;

    /// Query audit entries
    async fn query(&self, query: AuditQuery) -> AuthResult<Vec<AuditEntry>>;

    /// Delete entries before a timestamp
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> AuthResult<usize>;
}

/// Audit alert handler trait
#[async_trait]
pub trait AuditAlertHandler: Send + Sync {
    /// Check if entry should trigger an alert
    async fn should_alert(&self, entry: &AuditEntry) -> bool;

    /// Send the alert
    async fn send_alert(&self, entry: &AuditEntry) -> AuthResult<()>;
}

/// In-memory audit storage
pub struct InMemoryAuditStorage {
    entries: RwLock<Vec<AuditEntry>>,
}

impl InMemoryAuditStorage {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryAuditStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditStorage for InMemoryAuditStorage {
    async fn store(&self, entry: &AuditEntry) -> AuthResult<()> {
        let mut entries = self.entries.write().await;
        entries.push(entry.clone());
        Ok(())
    }

    async fn query(&self, query: AuditQuery) -> AuthResult<Vec<AuditEntry>> {
        let entries = self.entries.read().await;

        let mut results: Vec<_> = entries
            .iter()
            .filter(|e| {
                if let Some(user_id) = query.user_id {
                    if e.user_id != Some(user_id) {
                        return false;
                    }
                }
                if let Some(ref types) = query.event_types {
                    if !types.contains(&e.event_type) {
                        return false;
                    }
                }
                if let Some(success) = query.success_only {
                    if e.success != success {
                        return false;
                    }
                }
                if let Some(ref from) = query.from_time {
                    if e.timestamp < *from {
                        return false;
                    }
                }
                if let Some(ref to) = query.to_time {
                    if e.timestamp > *to {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by timestamp descending
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100);
        results = results.into_iter().skip(offset).take(limit).collect();

        Ok(results)
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> AuthResult<usize> {
        let mut entries = self.entries.write().await;
        let before = entries.len();
        entries.retain(|e| e.timestamp >= timestamp);
        Ok(before - entries.len())
    }
}

/// Security alert handler for suspicious activity
pub struct SecurityAlertHandler {
    failed_login_threshold: u32,
    failed_login_window: RwLock<HashMap<String, Vec<DateTime<Utc>>>>,
}

impl SecurityAlertHandler {
    pub fn new(threshold: u32) -> Self {
        Self {
            failed_login_threshold: threshold,
            failed_login_window: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl AuditAlertHandler for SecurityAlertHandler {
    async fn should_alert(&self, entry: &AuditEntry) -> bool {
        match entry.event_type {
            AuditEventType::LoginFailure => {
                // Check for multiple failed logins from same IP
                if let Some(ip) = &entry.ip_address {
                    let mut window = self.failed_login_window.write().await;
                    let attempts = window.entry(ip.clone()).or_insert_with(Vec::new);

                    // Remove old attempts (older than 1 hour)
                    let cutoff = Utc::now() - Duration::hours(1);
                    attempts.retain(|t| *t > cutoff);

                    attempts.push(entry.timestamp);

                    attempts.len() >= self.failed_login_threshold as usize
                } else {
                    false
                }
            }
            AuditEventType::AccountLocked => true,
            AuditEventType::SuspiciousActivity => true,
            AuditEventType::BruteForceDetected => true,
            _ => false,
        }
    }

    async fn send_alert(&self, entry: &AuditEntry) -> AuthResult<()> {
        // Log alert (in production, would send email/slack/etc)
        tracing::warn!(
            event_type = ?entry.event_type,
            ip_address = ?entry.ip_address,
            user_id = ?entry.user_id,
            "Security alert triggered"
        );
        Ok(())
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::config::AuditConfig;

    #[tokio::test]
    async fn test_audit_logging() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let config = AuditConfig::default();
        let logger = AuditLogger::new(storage.clone(), config);

        let entry = AuditEntry::new(AuditEventType::LoginSuccess, "User logged in")
            .with_user(UserId::new(), Some("testuser".to_string()))
            .with_success(true);

        logger.log(entry).await.unwrap();

        let results = logger.query(AuditQuery::default()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, AuditEventType::LoginSuccess);
    }

    #[tokio::test]
    async fn test_audit_query_filtering() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let config = AuditConfig::default();
        let logger = AuditLogger::new(storage.clone(), config);

        let user_id = UserId::new();

        // Log multiple entries
        logger.log(AuditEntry::new(AuditEventType::LoginSuccess, "Login")
            .with_user(user_id, None)).await.unwrap();
        logger.log(AuditEntry::new(AuditEventType::LoginFailure, "Login failed")
            .with_user(UserId::new(), None)).await.unwrap();
        logger.log(AuditEntry::new(AuditEventType::PasswordChanged, "Password changed")
            .with_user(user_id, None)).await.unwrap();

        // Query by user
        let results = logger.query(AuditQuery {
            user_id: Some(user_id),
            ..Default::default()
        }).await.unwrap();
        assert_eq!(results.len(), 2);

        // Query by event type
        let results = logger.query(AuditQuery {
            event_types: Some(vec![AuditEventType::LoginSuccess]),
            ..Default::default()
        }).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_retention() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let mut config = AuditConfig::default();
        config.retention_days = 7;
        let logger = AuditLogger::new(storage.clone(), config);

        // Log entry
        logger.log(AuditEntry::new(AuditEventType::LoginSuccess, "Login")).await.unwrap();

        // Apply retention (should keep entry)
        let deleted = logger.apply_retention().await.unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_ip_masking() {
        assert_eq!(mask_ip("192.168.1.100"), "192.xxx.xxx.xxx");
        assert_eq!(mask_ip("10.0.0.1"), "10.xxx.xxx.xxx");
    }

    #[test]
    fn test_user_agent_masking() {
        assert_eq!(
            mask_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0.4472.124"),
            "Chrome"
        );
        assert_eq!(
            mask_user_agent("Mozilla/5.0 Firefox/89.0"),
            "Firefox"
        );
        assert_eq!(mask_user_agent("Custom Bot"), "Unknown");
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntry::new(AuditEventType::LoginSuccess, "Login")
            .with_user(UserId::new(), Some("testuser".to_string()))
            .with_metadata(&AuthMetadata {
                ip_address: Some("192.168.1.1".to_string()),
                user_agent: Some("Chrome".to_string()),
                request_id: Some("req-123".to_string()),
                geo_location: None,
            })
            .with_context("extra", "data")
            .with_success(true);

        assert_eq!(entry.event_type, AuditEventType::LoginSuccess);
        assert!(entry.user_id.is_some());
        assert_eq!(entry.ip_address, Some("192.168.1.1".to_string()));
        assert!(entry.context.contains_key("extra"));
    }

    #[tokio::test]
    async fn test_security_alert_handler() {
        let handler = SecurityAlertHandler::new(3);

        // First two failures should not alert
        for i in 0..2 {
            let entry = AuditEntry::new(AuditEventType::LoginFailure, "Failed")
                .with_metadata(&AuthMetadata {
                    ip_address: Some("192.168.1.1".to_string()),
                    ..Default::default()
                });
            assert!(!handler.should_alert(&entry).await, "Attempt {} should not alert", i);
        }

        // Third failure should alert
        let entry = AuditEntry::new(AuditEventType::LoginFailure, "Failed")
            .with_metadata(&AuthMetadata {
                ip_address: Some("192.168.1.1".to_string()),
                ..Default::default()
            });
        assert!(handler.should_alert(&entry).await);
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses UserId and AuthMetadata
- **Spec 367**: Auth Configuration - Uses AuditConfig
- **Spec 384**: Auth Events - Converts events to audit entries
- **Spec 382**: Rate Limiting - Triggers audit on rate limit
- **Spec 383**: Account Lockout - Triggers audit on lockout
