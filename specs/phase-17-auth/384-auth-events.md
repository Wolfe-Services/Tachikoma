# Spec 384: Authentication Events

## Phase
17 - Authentication/Authorization

## Spec ID
384

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits

## Estimated Context
~8%

---

## Objective

Define and implement the authentication event system for notifying interested parties about authentication-related occurrences. This includes login events, password changes, MFA events, session events, and security alerts. The event system should support multiple subscribers and async event handling.

---

## Acceptance Criteria

- [ ] Define comprehensive `AuthEvent` enum
- [ ] Implement `AuthEventEmitter` trait
- [ ] Support multiple event subscribers
- [ ] Provide async event emission
- [ ] Support event filtering by type
- [ ] Implement in-memory event bus
- [ ] Support event persistence (optional)
- [ ] Enable real-time event streaming

---

## Implementation Details

### Authentication Events System

```rust
// src/auth/events.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, instrument};

use crate::auth::types::*;

/// Authentication events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthEvent {
    // Login events
    LoginSuccess {
        user_id: UserId,
        auth_method: AuthMethod,
        ip_address: Option<String>,
        user_agent: Option<String>,
        timestamp: DateTime<Utc>,
    },
    LoginFailure {
        user_id: Option<UserId>,
        username: String,
        reason: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        timestamp: DateTime<Utc>,
    },
    Logout {
        user_id: UserId,
        session_id: Option<SessionId>,
        timestamp: DateTime<Utc>,
    },

    // Registration events
    UserRegistered {
        user_id: UserId,
        username: String,
        timestamp: DateTime<Utc>,
    },

    // Password events
    PasswordChanged {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
    PasswordResetRequested {
        user_id: UserId,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },
    PasswordReset {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },

    // Session events
    SessionCreated {
        session_id: SessionId,
        user_id: UserId,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },
    SessionDestroyed {
        session_id: SessionId,
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
    AllSessionsDestroyed {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },

    // Token events
    TokenRefreshed {
        user_id: UserId,
        old_token_id: String,
        new_token_id: String,
        timestamp: DateTime<Utc>,
    },
    RefreshTokenReuse {
        user_id: UserId,
        family_id: String,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },
    RefreshTokenRevoked {
        user_id: UserId,
        token_id: String,
        timestamp: DateTime<Utc>,
    },
    AllRefreshTokensRevoked {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },

    // MFA events
    MfaEnabled {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
    MfaDisabled {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
    MfaVerified {
        user_id: UserId,
        method: crate::auth::mfa::MfaCodeType,
        timestamp: DateTime<Utc>,
    },
    MfaFailed {
        user_id: UserId,
        method: crate::auth::mfa::MfaCodeType,
        timestamp: DateTime<Utc>,
    },

    // Account events
    AccountEnabled {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
    AccountDisabled {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
    AccountLocked {
        user_id: UserId,
        reason: String,
        locked_until: Option<DateTime<Utc>>,
        timestamp: DateTime<Utc>,
    },
    AccountUnlocked {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },
    AccountDeleted {
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },

    // API key events
    ApiKeyCreated {
        user_id: UserId,
        key_id: String,
        key_name: String,
        timestamp: DateTime<Utc>,
    },
    ApiKeyRevoked {
        user_id: UserId,
        key_id: String,
        timestamp: DateTime<Utc>,
    },

    // OAuth events
    OAuth2Login {
        user_id: UserId,
        provider: String,
        is_new_user: bool,
        timestamp: DateTime<Utc>,
    },
    OAuth2Unlinked {
        user_id: UserId,
        provider: String,
        timestamp: DateTime<Utc>,
    },

    // Role/Permission events
    RoleCreated {
        role_id: String,
        timestamp: DateTime<Utc>,
    },
    RoleUpdated {
        role_id: String,
        timestamp: DateTime<Utc>,
    },
    RoleDeleted {
        role_id: String,
        timestamp: DateTime<Utc>,
    },
    RolesAssigned {
        user_id: UserId,
        role_ids: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    RolesRemoved {
        user_id: UserId,
        role_ids: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    PermissionGranted {
        user_id: UserId,
        permission: String,
        granted_by: Option<UserId>,
        timestamp: DateTime<Utc>,
    },
    PermissionRevoked {
        user_id: UserId,
        permission: String,
        timestamp: DateTime<Utc>,
    },

    // Security events
    RateLimitExceeded {
        key: String,
        timestamp: DateTime<Utc>,
    },
    SuspiciousActivity {
        user_id: Option<UserId>,
        description: String,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl AuthEvent {
    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            AuthEvent::LoginSuccess { .. } => "login_success",
            AuthEvent::LoginFailure { .. } => "login_failure",
            AuthEvent::Logout { .. } => "logout",
            AuthEvent::UserRegistered { .. } => "user_registered",
            AuthEvent::PasswordChanged { .. } => "password_changed",
            AuthEvent::PasswordResetRequested { .. } => "password_reset_requested",
            AuthEvent::PasswordReset { .. } => "password_reset",
            AuthEvent::SessionCreated { .. } => "session_created",
            AuthEvent::SessionDestroyed { .. } => "session_destroyed",
            AuthEvent::AllSessionsDestroyed { .. } => "all_sessions_destroyed",
            AuthEvent::TokenRefreshed { .. } => "token_refreshed",
            AuthEvent::RefreshTokenReuse { .. } => "refresh_token_reuse",
            AuthEvent::RefreshTokenRevoked { .. } => "refresh_token_revoked",
            AuthEvent::AllRefreshTokensRevoked { .. } => "all_refresh_tokens_revoked",
            AuthEvent::MfaEnabled { .. } => "mfa_enabled",
            AuthEvent::MfaDisabled { .. } => "mfa_disabled",
            AuthEvent::MfaVerified { .. } => "mfa_verified",
            AuthEvent::MfaFailed { .. } => "mfa_failed",
            AuthEvent::AccountEnabled { .. } => "account_enabled",
            AuthEvent::AccountDisabled { .. } => "account_disabled",
            AuthEvent::AccountLocked { .. } => "account_locked",
            AuthEvent::AccountUnlocked { .. } => "account_unlocked",
            AuthEvent::AccountDeleted { .. } => "account_deleted",
            AuthEvent::ApiKeyCreated { .. } => "api_key_created",
            AuthEvent::ApiKeyRevoked { .. } => "api_key_revoked",
            AuthEvent::OAuth2Login { .. } => "oauth2_login",
            AuthEvent::OAuth2Unlinked { .. } => "oauth2_unlinked",
            AuthEvent::RoleCreated { .. } => "role_created",
            AuthEvent::RoleUpdated { .. } => "role_updated",
            AuthEvent::RoleDeleted { .. } => "role_deleted",
            AuthEvent::RolesAssigned { .. } => "roles_assigned",
            AuthEvent::RolesRemoved { .. } => "roles_removed",
            AuthEvent::PermissionGranted { .. } => "permission_granted",
            AuthEvent::PermissionRevoked { .. } => "permission_revoked",
            AuthEvent::RateLimitExceeded { .. } => "rate_limit_exceeded",
            AuthEvent::SuspiciousActivity { .. } => "suspicious_activity",
        }
    }

    /// Get the user ID if present in the event
    pub fn user_id(&self) -> Option<UserId> {
        match self {
            AuthEvent::LoginSuccess { user_id, .. } => Some(*user_id),
            AuthEvent::LoginFailure { user_id, .. } => *user_id,
            AuthEvent::Logout { user_id, .. } => Some(*user_id),
            AuthEvent::UserRegistered { user_id, .. } => Some(*user_id),
            AuthEvent::PasswordChanged { user_id, .. } => Some(*user_id),
            AuthEvent::PasswordResetRequested { user_id, .. } => Some(*user_id),
            AuthEvent::PasswordReset { user_id, .. } => Some(*user_id),
            AuthEvent::SessionCreated { user_id, .. } => Some(*user_id),
            AuthEvent::SessionDestroyed { user_id, .. } => Some(*user_id),
            AuthEvent::AllSessionsDestroyed { user_id, .. } => Some(*user_id),
            AuthEvent::TokenRefreshed { user_id, .. } => Some(*user_id),
            AuthEvent::RefreshTokenReuse { user_id, .. } => Some(*user_id),
            AuthEvent::RefreshTokenRevoked { user_id, .. } => Some(*user_id),
            AuthEvent::AllRefreshTokensRevoked { user_id, .. } => Some(*user_id),
            AuthEvent::MfaEnabled { user_id, .. } => Some(*user_id),
            AuthEvent::MfaDisabled { user_id, .. } => Some(*user_id),
            AuthEvent::MfaVerified { user_id, .. } => Some(*user_id),
            AuthEvent::MfaFailed { user_id, .. } => Some(*user_id),
            AuthEvent::AccountEnabled { user_id, .. } => Some(*user_id),
            AuthEvent::AccountDisabled { user_id, .. } => Some(*user_id),
            AuthEvent::AccountLocked { user_id, .. } => Some(*user_id),
            AuthEvent::AccountUnlocked { user_id, .. } => Some(*user_id),
            AuthEvent::AccountDeleted { user_id, .. } => Some(*user_id),
            AuthEvent::ApiKeyCreated { user_id, .. } => Some(*user_id),
            AuthEvent::ApiKeyRevoked { user_id, .. } => Some(*user_id),
            AuthEvent::OAuth2Login { user_id, .. } => Some(*user_id),
            AuthEvent::OAuth2Unlinked { user_id, .. } => Some(*user_id),
            AuthEvent::RolesAssigned { user_id, .. } => Some(*user_id),
            AuthEvent::RolesRemoved { user_id, .. } => Some(*user_id),
            AuthEvent::PermissionGranted { user_id, .. } => Some(*user_id),
            AuthEvent::PermissionRevoked { user_id, .. } => Some(*user_id),
            AuthEvent::SuspiciousActivity { user_id, .. } => *user_id,
            _ => None,
        }
    }

    /// Get the timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            AuthEvent::LoginSuccess { timestamp, .. } => *timestamp,
            AuthEvent::LoginFailure { timestamp, .. } => *timestamp,
            AuthEvent::Logout { timestamp, .. } => *timestamp,
            AuthEvent::UserRegistered { timestamp, .. } => *timestamp,
            AuthEvent::PasswordChanged { timestamp, .. } => *timestamp,
            AuthEvent::PasswordResetRequested { timestamp, .. } => *timestamp,
            AuthEvent::PasswordReset { timestamp, .. } => *timestamp,
            AuthEvent::SessionCreated { timestamp, .. } => *timestamp,
            AuthEvent::SessionDestroyed { timestamp, .. } => *timestamp,
            AuthEvent::AllSessionsDestroyed { timestamp, .. } => *timestamp,
            AuthEvent::TokenRefreshed { timestamp, .. } => *timestamp,
            AuthEvent::RefreshTokenReuse { timestamp, .. } => *timestamp,
            AuthEvent::RefreshTokenRevoked { timestamp, .. } => *timestamp,
            AuthEvent::AllRefreshTokensRevoked { timestamp, .. } => *timestamp,
            AuthEvent::MfaEnabled { timestamp, .. } => *timestamp,
            AuthEvent::MfaDisabled { timestamp, .. } => *timestamp,
            AuthEvent::MfaVerified { timestamp, .. } => *timestamp,
            AuthEvent::MfaFailed { timestamp, .. } => *timestamp,
            AuthEvent::AccountEnabled { timestamp, .. } => *timestamp,
            AuthEvent::AccountDisabled { timestamp, .. } => *timestamp,
            AuthEvent::AccountLocked { timestamp, .. } => *timestamp,
            AuthEvent::AccountUnlocked { timestamp, .. } => *timestamp,
            AuthEvent::AccountDeleted { timestamp, .. } => *timestamp,
            AuthEvent::ApiKeyCreated { timestamp, .. } => *timestamp,
            AuthEvent::ApiKeyRevoked { timestamp, .. } => *timestamp,
            AuthEvent::OAuth2Login { timestamp, .. } => *timestamp,
            AuthEvent::OAuth2Unlinked { timestamp, .. } => *timestamp,
            AuthEvent::RoleCreated { timestamp, .. } => *timestamp,
            AuthEvent::RoleUpdated { timestamp, .. } => *timestamp,
            AuthEvent::RoleDeleted { timestamp, .. } => *timestamp,
            AuthEvent::RolesAssigned { timestamp, .. } => *timestamp,
            AuthEvent::RolesRemoved { timestamp, .. } => *timestamp,
            AuthEvent::PermissionGranted { timestamp, .. } => *timestamp,
            AuthEvent::PermissionRevoked { timestamp, .. } => *timestamp,
            AuthEvent::RateLimitExceeded { timestamp, .. } => *timestamp,
            AuthEvent::SuspiciousActivity { timestamp, .. } => *timestamp,
        }
    }

    /// Check if this is a security-related event
    pub fn is_security_event(&self) -> bool {
        matches!(
            self,
            AuthEvent::LoginFailure { .. }
                | AuthEvent::RefreshTokenReuse { .. }
                | AuthEvent::MfaFailed { .. }
                | AuthEvent::AccountLocked { .. }
                | AuthEvent::RateLimitExceeded { .. }
                | AuthEvent::SuspiciousActivity { .. }
        )
    }
}

/// Event emitter trait
#[async_trait]
pub trait AuthEventEmitter: Send + Sync {
    /// Emit an event
    async fn emit(&self, event: AuthEvent);
}

/// Event handler trait for subscribers
#[async_trait]
pub trait AuthEventHandler: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: &AuthEvent);

    /// Filter - return true if handler should receive this event type
    fn accepts(&self, event: &AuthEvent) -> bool {
        true // Accept all by default
    }
}

/// Event bus for distributing events
pub struct AuthEventBus {
    handlers: RwLock<Vec<Arc<dyn AuthEventHandler>>>,
    broadcast_tx: broadcast::Sender<AuthEvent>,
    history: RwLock<Vec<AuthEvent>>,
    history_limit: usize,
}

impl AuthEventBus {
    pub fn new(history_limit: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        Self {
            handlers: RwLock::new(Vec::new()),
            broadcast_tx,
            history: RwLock::new(Vec::new()),
            history_limit,
        }
    }

    /// Register a handler
    pub async fn register(&self, handler: Arc<dyn AuthEventHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }

    /// Subscribe to events (returns a receiver)
    pub fn subscribe(&self) -> broadcast::Receiver<AuthEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Get recent events
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<AuthEvent> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(self.history_limit);
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get events for a specific user
    pub async fn get_user_events(&self, user_id: UserId, limit: Option<usize>) -> Vec<AuthEvent> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(50);
        history
            .iter()
            .rev()
            .filter(|e| e.user_id() == Some(user_id))
            .take(limit)
            .cloned()
            .collect()
    }
}

#[async_trait]
impl AuthEventEmitter for AuthEventBus {
    #[instrument(skip(self), fields(event_type = %event.event_type()))]
    async fn emit(&self, event: AuthEvent) {
        debug!(event_type = %event.event_type(), "Emitting auth event");

        // Add to history
        {
            let mut history = self.history.write().await;
            history.push(event.clone());

            // Trim history if needed
            if history.len() > self.history_limit {
                let drain_count = history.len() - self.history_limit;
                history.drain(0..drain_count);
            }
        }

        // Broadcast to subscribers
        let _ = self.broadcast_tx.send(event.clone());

        // Notify handlers
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            if handler.accepts(&event) {
                handler.handle(&event).await;
            }
        }
    }
}

/// Logging event handler
pub struct LoggingEventHandler;

#[async_trait]
impl AuthEventHandler for LoggingEventHandler {
    async fn handle(&self, event: &AuthEvent) {
        match event {
            AuthEvent::LoginSuccess { user_id, auth_method, .. } => {
                tracing::info!(user_id = %user_id, method = ?auth_method, "User logged in");
            }
            AuthEvent::LoginFailure { username, reason, .. } => {
                tracing::warn!(username = %username, reason = %reason, "Login failed");
            }
            AuthEvent::AccountLocked { user_id, reason, .. } => {
                tracing::warn!(user_id = %user_id, reason = %reason, "Account locked");
            }
            AuthEvent::SuspiciousActivity { description, ip_address, .. } => {
                tracing::warn!(
                    description = %description,
                    ip = ?ip_address,
                    "Suspicious activity detected"
                );
            }
            _ => {
                tracing::debug!(event_type = %event.event_type(), "Auth event");
            }
        }
    }
}

/// Security alert handler for critical events
pub struct SecurityAlertHandler {
    alert_callback: Box<dyn Fn(&AuthEvent) + Send + Sync>,
}

impl SecurityAlertHandler {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(&AuthEvent) + Send + Sync + 'static,
    {
        Self {
            alert_callback: Box::new(callback),
        }
    }
}

#[async_trait]
impl AuthEventHandler for SecurityAlertHandler {
    async fn handle(&self, event: &AuthEvent) {
        (self.alert_callback)(event);
    }

    fn accepts(&self, event: &AuthEvent) -> bool {
        event.is_security_event()
    }
}

/// Event persistence handler
pub struct PersistentEventHandler {
    storage: Arc<dyn EventStorage>,
}

impl PersistentEventHandler {
    pub fn new(storage: Arc<dyn EventStorage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl AuthEventHandler for PersistentEventHandler {
    async fn handle(&self, event: &AuthEvent) {
        if let Err(e) = self.storage.store(event).await {
            tracing::error!(error = %e, "Failed to persist auth event");
        }
    }
}

/// Event storage trait
#[async_trait]
pub trait EventStorage: Send + Sync {
    async fn store(&self, event: &AuthEvent) -> AuthResult<()>;
    async fn query(&self, filter: EventFilter) -> AuthResult<Vec<AuthEvent>>;
}

/// Event filter for querying
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub user_id: Option<UserId>,
    pub event_types: Option<Vec<String>>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// In-memory event storage
pub struct InMemoryEventStorage {
    events: RwLock<Vec<AuthEvent>>,
}

impl InMemoryEventStorage {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl EventStorage for InMemoryEventStorage {
    async fn store(&self, event: &AuthEvent) -> AuthResult<()> {
        let mut events = self.events.write().await;
        events.push(event.clone());
        Ok(())
    }

    async fn query(&self, filter: EventFilter) -> AuthResult<Vec<AuthEvent>> {
        let events = self.events.read().await;

        let results: Vec<_> = events
            .iter()
            .filter(|e| {
                if let Some(user_id) = filter.user_id {
                    if e.user_id() != Some(user_id) {
                        return false;
                    }
                }
                if let Some(ref types) = filter.event_types {
                    if !types.contains(&e.event_type().to_string()) {
                        return false;
                    }
                }
                if let Some(from) = filter.from_time {
                    if e.timestamp() < from {
                        return false;
                    }
                }
                if let Some(to) = filter.to_time {
                    if e.timestamp() > to {
                        return false;
                    }
                }
                true
            })
            .rev()
            .take(filter.limit.unwrap_or(100))
            .cloned()
            .collect();

        Ok(results)
    }
}

/// No-op emitter for when events are not needed
pub struct NoOpEmitter;

#[async_trait]
impl AuthEventEmitter for NoOpEmitter {
    async fn emit(&self, _event: AuthEvent) {}
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_type() {
        let event = AuthEvent::LoginSuccess {
            user_id: UserId::new(),
            auth_method: AuthMethod::Password,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };
        assert_eq!(event.event_type(), "login_success");
    }

    #[test]
    fn test_event_user_id() {
        let user_id = UserId::new();
        let event = AuthEvent::LoginSuccess {
            user_id,
            auth_method: AuthMethod::Password,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };
        assert_eq!(event.user_id(), Some(user_id));
    }

    #[test]
    fn test_is_security_event() {
        let login_failure = AuthEvent::LoginFailure {
            user_id: None,
            username: "test".to_string(),
            reason: "wrong password".to_string(),
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };
        assert!(login_failure.is_security_event());

        let login_success = AuthEvent::LoginSuccess {
            user_id: UserId::new(),
            auth_method: AuthMethod::Password,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };
        assert!(!login_success.is_security_event());
    }

    #[tokio::test]
    async fn test_event_bus_emit() {
        let bus = AuthEventBus::new(100);
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        let handler = Arc::new(CountingHandler { counter: counter_clone });
        bus.register(handler).await;

        let event = AuthEvent::LoginSuccess {
            user_id: UserId::new(),
            auth_method: AuthMethod::Password,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };

        bus.emit(event).await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_bus_subscribe() {
        let bus = Arc::new(AuthEventBus::new(100));
        let mut receiver = bus.subscribe();

        let bus_clone = bus.clone();
        let event = AuthEvent::LoginSuccess {
            user_id: UserId::new(),
            auth_method: AuthMethod::Password,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };

        tokio::spawn(async move {
            bus_clone.emit(event).await;
        });

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.event_type(), "login_success");
    }

    #[tokio::test]
    async fn test_event_bus_history() {
        let bus = AuthEventBus::new(10);

        for i in 0..15 {
            bus.emit(AuthEvent::LoginSuccess {
                user_id: UserId::new(),
                auth_method: AuthMethod::Password,
                ip_address: None,
                user_agent: None,
                timestamp: Utc::now(),
            }).await;
        }

        let history = bus.get_history(None).await;
        assert_eq!(history.len(), 10); // Limited to history_limit
    }

    #[tokio::test]
    async fn test_event_storage() {
        let storage = InMemoryEventStorage::new();
        let user_id = UserId::new();

        storage.store(&AuthEvent::LoginSuccess {
            user_id,
            auth_method: AuthMethod::Password,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        }).await.unwrap();

        storage.store(&AuthEvent::PasswordChanged {
            user_id,
            timestamp: Utc::now(),
        }).await.unwrap();

        let results = storage.query(EventFilter {
            user_id: Some(user_id),
            ..Default::default()
        }).await.unwrap();

        assert_eq!(results.len(), 2);
    }

    struct CountingHandler {
        counter: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl AuthEventHandler for CountingHandler {
        async fn handle(&self, _event: &AuthEvent) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses UserId, SessionId, AuthMethod
- **Spec 381**: Audit Logging - Converts events to audit entries
- **Spec 368**: Local Auth - Emits login events
- **Spec 369**: Session Management - Emits session events
- **Spec 378**: MFA - Emits MFA events
- **Spec 383**: Account Lockout - Emits lockout events
