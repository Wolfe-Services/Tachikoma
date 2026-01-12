# Spec 353: Audit Log Database Schema

## Overview
Define the SQLite schema for comprehensive audit logging, tracking all significant actions within the system.

## Rust Implementation

### Schema Models
```rust
// src/database/schema/audit.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;

/// Audit event category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum AuditCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    Configuration,
    System,
    Security,
    UserAction,
}

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for AuditSeverity {
    fn default() -> Self {
        Self::Info
    }
}

/// Audit event outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
    Partial,
    Unknown,
}

/// Audit log entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub category: AuditCategory,
    pub action: String,
    pub severity: AuditSeverity,
    pub outcome: AuditOutcome,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub description: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub metadata: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

/// Audit context for building log entries
#[derive(Debug, Clone, Default)]
pub struct AuditContext {
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
}

impl AuditContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_actor(mut self, actor_id: &str, actor_type: &str) -> Self {
        self.actor_id = Some(actor_id.to_string());
        self.actor_type = Some(actor_type.to_string());
        self
    }

    pub fn with_request(mut self, ip: Option<&str>, user_agent: Option<&str>) -> Self {
        self.ip_address = ip.map(String::from);
        self.user_agent = user_agent.map(String::from);
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    LoginSuccess { user_id: String, method: String },
    LoginFailure { username: String, reason: String },
    LogoutSuccess { user_id: String },
    PasswordChanged { user_id: String },
    PasswordResetRequested { email: String },
    MfaEnabled { user_id: String },
    MfaDisabled { user_id: String },
    TokenRefreshed { user_id: String },
    TokenRevoked { user_id: String, reason: String },
    SuspiciousActivity { user_id: String, activity: String },
    PermissionDenied { user_id: String, resource: String, action: String },
    RateLimitExceeded { identifier: String, limit: i32 },
}

impl SecurityEvent {
    pub fn to_audit_action(&self) -> &'static str {
        match self {
            Self::LoginSuccess { .. } => "auth.login.success",
            Self::LoginFailure { .. } => "auth.login.failure",
            Self::LogoutSuccess { .. } => "auth.logout",
            Self::PasswordChanged { .. } => "auth.password.changed",
            Self::PasswordResetRequested { .. } => "auth.password.reset_requested",
            Self::MfaEnabled { .. } => "auth.mfa.enabled",
            Self::MfaDisabled { .. } => "auth.mfa.disabled",
            Self::TokenRefreshed { .. } => "auth.token.refreshed",
            Self::TokenRevoked { .. } => "auth.token.revoked",
            Self::SuspiciousActivity { .. } => "security.suspicious_activity",
            Self::PermissionDenied { .. } => "auth.permission.denied",
            Self::RateLimitExceeded { .. } => "security.rate_limit",
        }
    }

    pub fn severity(&self) -> AuditSeverity {
        match self {
            Self::LoginSuccess { .. } => AuditSeverity::Info,
            Self::LoginFailure { .. } => AuditSeverity::Warning,
            Self::LogoutSuccess { .. } => AuditSeverity::Info,
            Self::PasswordChanged { .. } => AuditSeverity::Info,
            Self::PasswordResetRequested { .. } => AuditSeverity::Info,
            Self::MfaEnabled { .. } => AuditSeverity::Info,
            Self::MfaDisabled { .. } => AuditSeverity::Warning,
            Self::TokenRefreshed { .. } => AuditSeverity::Debug,
            Self::TokenRevoked { .. } => AuditSeverity::Warning,
            Self::SuspiciousActivity { .. } => AuditSeverity::Critical,
            Self::PermissionDenied { .. } => AuditSeverity::Warning,
            Self::RateLimitExceeded { .. } => AuditSeverity::Warning,
        }
    }
}
```

### Migration SQL
```rust
// src/database/migrations/005_create_audit.rs

use crate::database::migration::Migration;

pub fn migration() -> Migration {
    Migration::new(
        20240101000005,
        "create_audit",
        r#"
-- Audit log table
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    category TEXT NOT NULL
        CHECK (category IN ('authentication', 'authorization', 'data_access', 'data_modification', 'configuration', 'system', 'security', 'user_action')),
    action TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info'
        CHECK (severity IN ('debug', 'info', 'warning', 'error', 'critical')),
    outcome TEXT NOT NULL DEFAULT 'success'
        CHECK (outcome IN ('success', 'failure', 'partial', 'unknown')),
    actor_id TEXT,
    actor_type TEXT,
    target_type TEXT,
    target_id TEXT,
    resource_type TEXT,
    resource_id TEXT,
    ip_address TEXT,
    user_agent TEXT,
    session_id TEXT,
    request_id TEXT,
    description TEXT,
    old_value TEXT,
    new_value TEXT,
    metadata TEXT,
    error_message TEXT,
    duration_ms INTEGER
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_category ON audit_logs(category);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_logs(severity);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_target ON audit_logs(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_session ON audit_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_audit_request ON audit_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_audit_ip ON audit_logs(ip_address);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_audit_actor_time ON audit_logs(actor_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_category_time ON audit_logs(category, timestamp);

-- Audit retention policy tracking
CREATE TABLE IF NOT EXISTS audit_retention_policies (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    category TEXT,
    severity TEXT,
    retention_days INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Default retention policies
INSERT OR IGNORE INTO audit_retention_policies (id, name, retention_days) VALUES
    ('policy-default', 'default', 90);

INSERT OR IGNORE INTO audit_retention_policies (id, name, category, retention_days) VALUES
    ('policy-security', 'security', 'security', 365),
    ('policy-auth', 'authentication', 'authentication', 180);

INSERT OR IGNORE INTO audit_retention_policies (id, name, severity, retention_days) VALUES
    ('policy-critical', 'critical', 'critical', 730);

-- Audit summary for analytics (aggregated data)
CREATE TABLE IF NOT EXISTS audit_summaries (
    id TEXT PRIMARY KEY NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    period_type TEXT NOT NULL CHECK (period_type IN ('hourly', 'daily', 'weekly', 'monthly')),
    category TEXT NOT NULL,
    action TEXT NOT NULL,
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    unique_actors INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(period_start, period_type, category, action)
);

CREATE INDEX IF NOT EXISTS idx_audit_summary_period ON audit_summaries(period_start, period_type);
"#
    ).with_down(r#"
DROP TABLE IF EXISTS audit_summaries;
DROP TABLE IF EXISTS audit_retention_policies;
DROP TABLE IF EXISTS audit_logs;
"#)
}
```

### Audit Log Builder
```rust
// src/database/schema/audit_builder.rs

use super::audit::*;
use chrono::Utc;
use uuid::Uuid;

/// Builder for creating audit log entries
pub struct AuditLogBuilder {
    entry: AuditLog,
}

impl AuditLogBuilder {
    pub fn new(category: AuditCategory, action: impl Into<String>) -> Self {
        Self {
            entry: AuditLog {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                category,
                action: action.into(),
                severity: AuditSeverity::Info,
                outcome: AuditOutcome::Success,
                actor_id: None,
                actor_type: None,
                target_type: None,
                target_id: None,
                resource_type: None,
                resource_id: None,
                ip_address: None,
                user_agent: None,
                session_id: None,
                request_id: None,
                description: None,
                old_value: None,
                new_value: None,
                metadata: None,
                error_message: None,
                duration_ms: None,
            },
        }
    }

    pub fn with_context(mut self, ctx: &AuditContext) -> Self {
        self.entry.actor_id = ctx.actor_id.clone();
        self.entry.actor_type = ctx.actor_type.clone();
        self.entry.ip_address = ctx.ip_address.clone();
        self.entry.user_agent = ctx.user_agent.clone();
        self.entry.session_id = ctx.session_id.clone();
        self.entry.request_id = ctx.request_id.clone();
        self
    }

    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.entry.severity = severity;
        self
    }

    pub fn outcome(mut self, outcome: AuditOutcome) -> Self {
        self.entry.outcome = outcome;
        self
    }

    pub fn success(self) -> Self {
        self.outcome(AuditOutcome::Success)
    }

    pub fn failure(self) -> Self {
        self.outcome(AuditOutcome::Failure)
    }

    pub fn actor(mut self, id: &str, actor_type: &str) -> Self {
        self.entry.actor_id = Some(id.to_string());
        self.entry.actor_type = Some(actor_type.to_string());
        self
    }

    pub fn target(mut self, target_type: &str, target_id: &str) -> Self {
        self.entry.target_type = Some(target_type.to_string());
        self.entry.target_id = Some(target_id.to_string());
        self
    }

    pub fn resource(mut self, resource_type: &str, resource_id: &str) -> Self {
        self.entry.resource_type = Some(resource_type.to_string());
        self.entry.resource_id = Some(resource_id.to_string());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.entry.description = Some(desc.into());
        self
    }

    pub fn changes(mut self, old_value: Option<&str>, new_value: Option<&str>) -> Self {
        self.entry.old_value = old_value.map(String::from);
        self.entry.new_value = new_value.map(String::from);
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.entry.metadata = Some(metadata.to_string());
        self
    }

    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.entry.error_message = Some(message.into());
        self.entry.outcome = AuditOutcome::Failure;
        self
    }

    pub fn duration_ms(mut self, ms: i64) -> Self {
        self.entry.duration_ms = Some(ms);
        self
    }

    pub fn build(self) -> AuditLog {
        self.entry
    }
}

/// Macro for quick audit logging
#[macro_export]
macro_rules! audit {
    ($category:expr, $action:expr) => {
        AuditLogBuilder::new($category, $action)
    };
    ($category:expr, $action:expr, $ctx:expr) => {
        AuditLogBuilder::new($category, $action).with_context($ctx)
    };
}
```

## Schema Design Decisions

1. **Comprehensive Indexing**: Multiple indexes for common query patterns
2. **Flexible Metadata**: JSON field for extensible data
3. **Retention Policies**: Configurable cleanup rules
4. **Aggregation**: Summary tables for analytics
5. **Context Tracking**: Full request context capture

## Files to Create
- `src/database/schema/audit.rs` - Audit models
- `src/database/schema/audit_builder.rs` - Builder pattern
- `src/database/migrations/005_create_audit.rs` - Migration
