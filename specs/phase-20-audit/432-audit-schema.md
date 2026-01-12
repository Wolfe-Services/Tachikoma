# 432 - Audit Schema

**Phase:** 20 - Audit System
**Spec ID:** 432
**Status:** Planned
**Dependencies:** 431-audit-event-types
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Define the storage schema for audit events, supporting both SQLite persistence and append-only log files for immutability guarantees.

---

## Acceptance Criteria

- [ ] SQLite schema for audit events
- [ ] Append-only log file format
- [ ] Schema versioning support
- [ ] Index definitions for common queries
- [ ] Migration support

---

## Implementation Details

### 1. Schema Module (src/schema.rs)

```rust
//! Audit storage schema definitions.

use serde::{Deserialize, Serialize};

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Schema for the audit_events table.
pub const AUDIT_EVENTS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY NOT NULL,
    timestamp TEXT NOT NULL,
    category TEXT NOT NULL,
    action TEXT NOT NULL,
    severity TEXT NOT NULL,
    actor_type TEXT NOT NULL,
    actor_id TEXT,
    actor_name TEXT,
    target_type TEXT,
    target_id TEXT,
    target_name TEXT,
    outcome TEXT NOT NULL,
    outcome_reason TEXT,
    metadata TEXT,
    correlation_id TEXT,
    ip_address TEXT,
    user_agent TEXT,
    checksum TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_category ON audit_events(category);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_events(action);
CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_events(severity);
CREATE INDEX IF NOT EXISTS idx_audit_actor_id ON audit_events(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_target_id ON audit_events(target_id);
CREATE INDEX IF NOT EXISTS idx_audit_correlation ON audit_events(correlation_id);
CREATE INDEX IF NOT EXISTS idx_audit_category_timestamp ON audit_events(category, timestamp);
"#;

/// Schema for the schema_migrations table.
pub const MIGRATIONS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT
);
"#;

/// Schema for audit log sequence tracking.
pub const SEQUENCE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS audit_sequence (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_sequence_number INTEGER NOT NULL DEFAULT 0,
    last_event_id TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO audit_sequence (id, last_sequence_number) VALUES (1, 0);
"#;

/// Append-only log entry format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Sequence number (monotonically increasing).
    pub sequence: u64,
    /// Event ID.
    pub event_id: String,
    /// Timestamp in ISO 8601 format.
    pub timestamp: String,
    /// JSON-serialized event data.
    pub event_data: String,
    /// SHA-256 checksum of event_data.
    pub checksum: String,
    /// Previous entry's checksum (for chain integrity).
    pub prev_checksum: Option<String>,
}

impl AuditLogEntry {
    /// Create a new log entry.
    pub fn new(
        sequence: u64,
        event_id: String,
        timestamp: String,
        event_data: String,
        prev_checksum: Option<String>,
    ) -> Self {
        let checksum = Self::compute_checksum(&event_data, prev_checksum.as_deref());
        Self {
            sequence,
            event_id,
            timestamp,
            event_data,
            checksum,
            prev_checksum,
        }
    }

    /// Compute checksum for the entry.
    pub fn compute_checksum(event_data: &str, prev_checksum: Option<&str>) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(event_data.as_bytes());
        if let Some(prev) = prev_checksum {
            hasher.update(prev.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Verify this entry's checksum.
    pub fn verify(&self) -> bool {
        let computed = Self::compute_checksum(&self.event_data, self.prev_checksum.as_deref());
        computed == self.checksum
    }

    /// Format as append-only log line.
    pub fn to_log_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parse from log line.
    pub fn from_log_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

/// Database row representation.
#[derive(Debug, Clone)]
pub struct AuditEventRow {
    pub id: String,
    pub timestamp: String,
    pub category: String,
    pub action: String,
    pub severity: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,
    pub outcome: String,
    pub outcome_reason: Option<String>,
    pub metadata: Option<String>,
    pub correlation_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub checksum: String,
}

/// Convert AuditEvent to database row.
pub fn event_to_row(event: &super::AuditEvent) -> AuditEventRow {
    use super::{AuditActor, AuditOutcome};

    let (actor_type, actor_id, actor_name) = match &event.actor {
        AuditActor::User { user_id, username, .. } => {
            ("user".to_string(), Some(user_id.to_string()), username.clone())
        }
        AuditActor::System { component, .. } => {
            ("system".to_string(), Some(component.clone()), None)
        }
        AuditActor::ApiClient { client_id, client_name, .. } => {
            ("api_client".to_string(), Some(client_id.clone()), client_name.clone())
        }
        AuditActor::Backend { backend_name, .. } => {
            ("backend".to_string(), Some(backend_name.clone()), None)
        }
        AuditActor::Unknown => ("unknown".to_string(), None, None),
    };

    let (outcome, outcome_reason) = match &event.outcome {
        AuditOutcome::Success => ("success".to_string(), None),
        AuditOutcome::Failure { reason } => ("failure".to_string(), Some(reason.clone())),
        AuditOutcome::Denied { reason } => ("denied".to_string(), Some(reason.clone())),
        AuditOutcome::Pending => ("pending".to_string(), None),
        AuditOutcome::Unknown => ("unknown".to_string(), None),
    };

    let metadata = if event.metadata.is_empty() {
        None
    } else {
        serde_json::to_string(&event.metadata).ok()
    };

    let event_json = serde_json::to_string(event).unwrap_or_default();
    let checksum = AuditLogEntry::compute_checksum(&event_json, None);

    AuditEventRow {
        id: event.id.to_string(),
        timestamp: event.timestamp.to_rfc3339(),
        category: event.category.to_string(),
        action: event.action.to_string(),
        severity: format!("{:?}", event.severity).to_lowercase(),
        actor_type,
        actor_id,
        actor_name,
        target_type: event.target.as_ref().map(|t| t.resource_type.clone()),
        target_id: event.target.as_ref().map(|t| t.resource_id.clone()),
        target_name: event.target.as_ref().and_then(|t| t.resource_name.clone()),
        outcome,
        outcome_reason,
        metadata,
        correlation_id: event.correlation_id.clone(),
        ip_address: event.ip_address.clone(),
        user_agent: event.user_agent.clone(),
        checksum,
    }
}
```

### 2. Migration Support (src/migration.rs)

```rust
//! Schema migration support.

use rusqlite::Connection;
use thiserror::Error;

/// Migration error.
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("migration {version} failed: {reason}")]
    MigrationFailed { version: u32, reason: String },
}

/// A schema migration.
pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub up: &'static str,
    pub down: Option<&'static str>,
}

/// All migrations in order.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial audit schema",
        up: super::schema::AUDIT_EVENTS_SCHEMA,
        down: Some("DROP TABLE IF EXISTS audit_events;"),
    },
];

/// Run all pending migrations.
pub fn run_migrations(conn: &Connection) -> Result<u32, MigrationError> {
    // Ensure migrations table exists
    conn.execute(super::schema::MIGRATIONS_SCHEMA, [])?;
    conn.execute(super::schema::SEQUENCE_SCHEMA, [])?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut applied = 0;
    for migration in MIGRATIONS.iter().filter(|m| m.version > current_version) {
        conn.execute_batch(migration.up)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, description) VALUES (?1, ?2)",
            rusqlite::params![migration.version, migration.description],
        )?;
        applied += 1;
    }

    Ok(applied)
}

/// Get current schema version.
pub fn current_version(conn: &Connection) -> Result<u32, MigrationError> {
    Ok(conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0))
}
```

---

## Testing Requirements

1. Schema creates valid SQLite tables
2. Migrations run idempotently
3. Checksum computation is deterministic
4. Log entries serialize/deserialize correctly
5. Chain integrity verification works

---

## Related Specs

- Depends on: [431-audit-event-types.md](431-audit-event-types.md)
- Next: [433-audit-capture.md](433-audit-capture.md)
