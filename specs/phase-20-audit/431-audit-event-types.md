# 431 - Audit Event Types

**Phase:** 20 - Audit System
**Spec ID:** 431
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Define the core event types for Tachikoma's audit logging system, providing strongly-typed events for all auditable actions across the application.

---

## Acceptance Criteria

- [ ] `tachikoma-audit-types` crate created
- [ ] Core event type enum with all audit categories
- [ ] Event metadata types (actor, target, context)
- [ ] Event severity levels
- [ ] Serialization support for all types
- [ ] Event builder pattern for construction

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-audit-types/Cargo.toml)

```toml
[package]
name = "tachikoma-audit-types"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Audit event types for Tachikoma"

[dependencies]
tachikoma-common-core.workspace = true
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
strum = { version = "0.26", features = ["derive"] }

[dev-dependencies]
proptest.workspace = true
```

### 2. Event ID Type (src/id.rs)

```rust
//! Audit event identifiers.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for an audit event.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditEventId(Uuid);

impl AuditEventId {
    /// Create a new random event ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for AuditEventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AuditEventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "aud_{}", self.0)
    }
}

impl fmt::Debug for AuditEventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AuditEventId({})", self)
    }
}
```

### 3. Event Category (src/category.rs)

```rust
//! Audit event categories.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

/// High-level category for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Display, EnumIter, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AuditCategory {
    /// Authentication events (login, logout, token refresh).
    Authentication,
    /// Authorization events (permission checks, access denied).
    Authorization,
    /// User management events (create, update, delete users).
    UserManagement,
    /// Mission lifecycle events.
    Mission,
    /// Forge session events.
    Forge,
    /// Configuration changes.
    Configuration,
    /// File system operations.
    FileSystem,
    /// API interactions with LLM backends.
    ApiCall,
    /// System events (startup, shutdown, errors).
    System,
    /// Security events (suspicious activity, violations).
    Security,
    /// Data export/import events.
    DataTransfer,
}

impl AuditCategory {
    /// Get all categories.
    pub fn all() -> impl Iterator<Item = Self> {
        use strum::IntoEnumIterator;
        Self::iter()
    }

    /// Check if this category requires elevated retention.
    pub fn requires_extended_retention(&self) -> bool {
        matches!(
            self,
            Self::Authentication
                | Self::Authorization
                | Self::Security
                | Self::UserManagement
        )
    }
}
```

### 4. Event Severity (src/severity.rs)

```rust
//! Audit event severity levels.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Severity level for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Informational events (normal operations).
    Info,
    /// Low-impact events that may warrant review.
    Low,
    /// Medium-impact events requiring attention.
    Medium,
    /// High-impact events requiring immediate review.
    High,
    /// Critical security events.
    Critical,
}

impl AuditSeverity {
    /// Numeric value for comparison (higher = more severe).
    pub fn level(&self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    /// Check if this severity meets a minimum threshold.
    pub fn meets_threshold(&self, threshold: Self) -> bool {
        self.level() >= threshold.level()
    }
}

impl PartialOrd for AuditSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AuditSeverity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level().cmp(&other.level())
    }
}

impl Default for AuditSeverity {
    fn default() -> Self {
        Self::Info
    }
}
```

### 5. Actor Types (src/actor.rs)

```rust
//! Audit event actors.

use serde::{Deserialize, Serialize};
use tachikoma_common_core::UserId;

/// The entity that initiated an audit event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditActor {
    /// A human user.
    User {
        user_id: UserId,
        username: Option<String>,
        session_id: Option<String>,
    },
    /// The system itself (automated processes).
    System {
        component: String,
        process_id: Option<u32>,
    },
    /// An API client.
    ApiClient {
        client_id: String,
        client_name: Option<String>,
    },
    /// An LLM backend.
    Backend {
        backend_name: String,
        model: Option<String>,
    },
    /// Unknown actor (for legacy events).
    Unknown,
}

impl AuditActor {
    /// Create a user actor.
    pub fn user(user_id: UserId) -> Self {
        Self::User {
            user_id,
            username: None,
            session_id: None,
        }
    }

    /// Create a system actor.
    pub fn system(component: impl Into<String>) -> Self {
        Self::System {
            component: component.into(),
            process_id: std::process::id().into(),
        }
    }

    /// Create an API client actor.
    pub fn api_client(client_id: impl Into<String>) -> Self {
        Self::ApiClient {
            client_id: client_id.into(),
            client_name: None,
        }
    }

    /// Get a display identifier for this actor.
    pub fn identifier(&self) -> String {
        match self {
            Self::User { user_id, username, .. } => {
                username.clone().unwrap_or_else(|| user_id.to_string())
            }
            Self::System { component, .. } => format!("system:{}", component),
            Self::ApiClient { client_id, client_name, .. } => {
                client_name.clone().unwrap_or_else(|| client_id.clone())
            }
            Self::Backend { backend_name, .. } => format!("backend:{}", backend_name),
            Self::Unknown => "unknown".to_string(),
        }
    }
}
```

### 6. Event Actions (src/action.rs)

```rust
//! Audit event actions.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Specific actions that can be audited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AuditAction {
    // Authentication
    Login,
    Logout,
    LoginFailed,
    TokenRefresh,
    TokenRevoked,
    SessionExpired,

    // Authorization
    AccessGranted,
    AccessDenied,
    PermissionChanged,
    RoleAssigned,
    RoleRevoked,

    // User Management
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserEnabled,
    UserDisabled,
    PasswordChanged,
    PasswordReset,

    // Mission
    MissionCreated,
    MissionStarted,
    MissionPaused,
    MissionResumed,
    MissionCompleted,
    MissionFailed,
    MissionAborted,
    MissionRebooted,

    // Forge
    ForgeSessionCreated,
    ForgeSessionCompleted,
    ForgeDraftGenerated,
    ForgeCritiqueReceived,
    ForgeSynthesized,

    // Configuration
    ConfigCreated,
    ConfigUpdated,
    ConfigDeleted,
    ConfigExported,
    ConfigImported,

    // File System
    FileCreated,
    FileRead,
    FileUpdated,
    FileDeleted,
    FileMoved,
    FilePermissionChanged,

    // API Calls
    ApiRequestSent,
    ApiResponseReceived,
    ApiRateLimited,
    ApiError,

    // System
    SystemStartup,
    SystemShutdown,
    SystemError,
    BackupCreated,
    BackupRestored,

    // Security
    SuspiciousActivity,
    SecurityViolation,
    IntrusionDetected,
    DataBreach,

    // Data Transfer
    DataExported,
    DataImported,
    DataDeleted,
    DataArchived,

    // Custom action
    Custom(String),
}

impl AuditAction {
    /// Get the default severity for this action.
    pub fn default_severity(&self) -> super::AuditSeverity {
        use super::AuditSeverity;
        match self {
            // Critical
            Self::DataBreach | Self::IntrusionDetected | Self::SecurityViolation => {
                AuditSeverity::Critical
            }

            // High
            Self::LoginFailed
            | Self::AccessDenied
            | Self::SuspiciousActivity
            | Self::UserDeleted
            | Self::MissionFailed
            | Self::SystemError => AuditSeverity::High,

            // Medium
            Self::PasswordChanged
            | Self::PasswordReset
            | Self::PermissionChanged
            | Self::RoleAssigned
            | Self::RoleRevoked
            | Self::ConfigUpdated
            | Self::ConfigDeleted
            | Self::UserUpdated => AuditSeverity::Medium,

            // Low
            Self::Login
            | Self::Logout
            | Self::TokenRefresh
            | Self::UserCreated
            | Self::MissionCreated
            | Self::ConfigCreated => AuditSeverity::Low,

            // Info (default)
            _ => AuditSeverity::Info,
        }
    }
}
```

### 7. Core Event Type (src/event.rs)

```rust
//! Core audit event type.

use crate::{
    AuditAction, AuditActor, AuditCategory, AuditEventId, AuditSeverity,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier.
    pub id: AuditEventId,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Event category.
    pub category: AuditCategory,
    /// Specific action.
    pub action: AuditAction,
    /// Event severity.
    pub severity: AuditSeverity,
    /// Who initiated the event.
    pub actor: AuditActor,
    /// Optional target resource identifier.
    pub target: Option<AuditTarget>,
    /// Event outcome.
    pub outcome: AuditOutcome,
    /// Additional context data.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Correlation ID for related events.
    pub correlation_id: Option<String>,
    /// IP address if applicable.
    pub ip_address: Option<String>,
    /// User agent if applicable.
    pub user_agent: Option<String>,
}

/// Target of an audit event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTarget {
    /// Type of the target resource.
    pub resource_type: String,
    /// Resource identifier.
    pub resource_id: String,
    /// Optional resource name.
    pub resource_name: Option<String>,
}

impl AuditTarget {
    /// Create a new target.
    pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            resource_name: None,
        }
    }

    /// Add a resource name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.resource_name = Some(name.into());
        self
    }
}

/// Outcome of an audited action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Action succeeded.
    Success,
    /// Action failed.
    Failure { reason: String },
    /// Action was denied.
    Denied { reason: String },
    /// Action is pending.
    Pending,
    /// Unknown outcome.
    Unknown,
}

impl AuditOutcome {
    /// Check if the outcome is successful.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if the outcome is a failure.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure { .. } | Self::Denied { .. })
    }
}

impl Default for AuditOutcome {
    fn default() -> Self {
        Self::Success
    }
}

impl AuditEvent {
    /// Create a new event builder.
    pub fn builder(category: AuditCategory, action: AuditAction) -> AuditEventBuilder {
        AuditEventBuilder::new(category, action)
    }
}

/// Builder for constructing audit events.
#[derive(Debug)]
pub struct AuditEventBuilder {
    category: AuditCategory,
    action: AuditAction,
    severity: Option<AuditSeverity>,
    actor: Option<AuditActor>,
    target: Option<AuditTarget>,
    outcome: AuditOutcome,
    metadata: HashMap<String, serde_json::Value>,
    correlation_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl AuditEventBuilder {
    /// Create a new builder.
    pub fn new(category: AuditCategory, action: AuditAction) -> Self {
        Self {
            category,
            action,
            severity: None,
            actor: None,
            target: None,
            outcome: AuditOutcome::Success,
            metadata: HashMap::new(),
            correlation_id: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Set the severity (defaults to action's default severity).
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set the actor.
    pub fn actor(mut self, actor: AuditActor) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Set the target.
    pub fn target(mut self, target: AuditTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the outcome.
    pub fn outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Add metadata.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json);
        }
        self
    }

    /// Set correlation ID.
    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set IP address.
    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent.
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Build the event.
    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: AuditEventId::new(),
            timestamp: Utc::now(),
            category: self.category,
            severity: self.severity.unwrap_or_else(|| self.action.default_severity()),
            action: self.action,
            actor: self.actor.unwrap_or(AuditActor::Unknown),
            target: self.target,
            outcome: self.outcome,
            metadata: self.metadata,
            correlation_id: self.correlation_id,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
        }
    }
}
```

### 8. Library Root (src/lib.rs)

```rust
//! Audit event types for Tachikoma.
//!
//! This crate provides the core types for the audit logging system.

#![warn(missing_docs)]

pub mod action;
pub mod actor;
pub mod category;
pub mod event;
pub mod id;
pub mod severity;

pub use action::AuditAction;
pub use actor::AuditActor;
pub use category::AuditCategory;
pub use event::{AuditEvent, AuditEventBuilder, AuditOutcome, AuditTarget};
pub use id::AuditEventId;
pub use severity::AuditSeverity;
```

---

## Testing Requirements

1. All event types serialize/deserialize correctly
2. Event builder produces valid events
3. Severity ordering works correctly
4. Action default severities are appropriate
5. Actor identifiers are generated correctly

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [432-audit-schema.md](432-audit-schema.md)
- Used by: All audit system components
