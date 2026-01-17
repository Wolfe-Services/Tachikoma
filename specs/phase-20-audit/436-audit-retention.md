# 436 - Audit Retention

**Phase:** 20 - Audit System
**Spec ID:** 436
**Status:** Planned
**Dependencies:** 434-audit-persistence
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement configurable retention policies for audit events, enabling automatic cleanup of old events while preserving compliance-critical records.

---

## Acceptance Criteria

- [x] Configurable retention periods per category
- [x] Automatic cleanup scheduler
- [x] Retention policy enforcement
- [x] Pre-deletion archival option
- [x] Compliance holds support

---

## Implementation Details

### 1. Retention Policy (src/retention.rs)

```rust
//! Audit retention policy management.

use crate::AuditCategory;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Retention duration specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionDuration {
    /// Keep for specified days.
    Days(u32),
    /// Keep for specified months.
    Months(u32),
    /// Keep for specified years.
    Years(u32),
    /// Keep indefinitely.
    Indefinite,
}

impl RetentionDuration {
    /// Convert to a cutoff datetime.
    pub fn to_cutoff(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        match self {
            Self::Days(d) => Some(now - Duration::days(*d as i64)),
            Self::Months(m) => Some(now - Duration::days(*m as i64 * 30)),
            Self::Years(y) => Some(now - Duration::days(*y as i64 * 365)),
            Self::Indefinite => None,
        }
    }

    /// Check if a timestamp is within retention.
    pub fn is_retained(&self, timestamp: DateTime<Utc>) -> bool {
        match self.to_cutoff() {
            Some(cutoff) => timestamp >= cutoff,
            None => true, // Indefinite retention
        }
    }
}

impl Default for RetentionDuration {
    fn default() -> Self {
        Self::Years(1)
    }
}

/// Retention policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Default retention for all categories.
    #[serde(default)]
    pub default_retention: RetentionDuration,
    /// Category-specific retention overrides.
    #[serde(default)]
    pub category_retention: HashMap<AuditCategory, RetentionDuration>,
    /// Severity-based retention extensions.
    #[serde(default)]
    pub severity_extensions: SeverityRetentionExtensions,
    /// Archive before deletion.
    #[serde(default)]
    pub archive_before_delete: bool,
    /// Enable compliance holds.
    #[serde(default)]
    pub enable_holds: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        let mut category_retention = HashMap::new();
        // Security events kept longer by default
        category_retention.insert(AuditCategory::Security, RetentionDuration::Years(7));
        category_retention.insert(AuditCategory::Authentication, RetentionDuration::Years(3));
        category_retention.insert(AuditCategory::Authorization, RetentionDuration::Years(3));

        Self {
            default_retention: RetentionDuration::Years(1),
            category_retention,
            severity_extensions: SeverityRetentionExtensions::default(),
            archive_before_delete: true,
            enable_holds: true,
        }
    }
}

impl RetentionPolicy {
    /// Get retention duration for a specific category.
    pub fn retention_for(&self, category: AuditCategory) -> RetentionDuration {
        self.category_retention
            .get(&category)
            .copied()
            .unwrap_or(self.default_retention)
    }

    /// Check if an event should be retained.
    pub fn should_retain(
        &self,
        category: AuditCategory,
        severity: crate::AuditSeverity,
        timestamp: DateTime<Utc>,
    ) -> bool {
        let base_retention = self.retention_for(category);
        let extended = self.severity_extensions.extend(base_retention, severity);
        extended.is_retained(timestamp)
    }
}

/// Severity-based retention extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityRetentionExtensions {
    /// Multiplier for high severity events.
    pub high_multiplier: f32,
    /// Multiplier for critical severity events.
    pub critical_multiplier: f32,
}

impl Default for SeverityRetentionExtensions {
    fn default() -> Self {
        Self {
            high_multiplier: 2.0,
            critical_multiplier: 5.0,
        }
    }
}

impl SeverityRetentionExtensions {
    /// Extend retention based on severity.
    pub fn extend(
        &self,
        base: RetentionDuration,
        severity: crate::AuditSeverity,
    ) -> RetentionDuration {
        use crate::AuditSeverity;

        let multiplier = match severity {
            AuditSeverity::Critical => self.critical_multiplier,
            AuditSeverity::High => self.high_multiplier,
            _ => 1.0,
        };

        if multiplier <= 1.0 {
            return base;
        }

        match base {
            RetentionDuration::Days(d) => {
                RetentionDuration::Days((d as f32 * multiplier) as u32)
            }
            RetentionDuration::Months(m) => {
                RetentionDuration::Months((m as f32 * multiplier) as u32)
            }
            RetentionDuration::Years(y) => {
                RetentionDuration::Years((y as f32 * multiplier) as u32)
            }
            RetentionDuration::Indefinite => RetentionDuration::Indefinite,
        }
    }
}

/// Legal or compliance hold on audit data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionHold {
    /// Unique hold identifier.
    pub id: String,
    /// Hold name/description.
    pub name: String,
    /// When the hold was created.
    pub created_at: DateTime<Utc>,
    /// Who created the hold.
    pub created_by: String,
    /// Categories affected by the hold.
    pub categories: Vec<AuditCategory>,
    /// Time range affected (None = all time).
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// When the hold expires (None = indefinite).
    pub expires_at: Option<DateTime<Utc>>,
    /// Reason for the hold.
    pub reason: String,
}

impl RetentionHold {
    /// Check if this hold is active.
    pub fn is_active(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() < expires,
            None => true,
        }
    }

    /// Check if this hold applies to an event.
    pub fn applies_to(
        &self,
        category: AuditCategory,
        timestamp: DateTime<Utc>,
    ) -> bool {
        if !self.is_active() {
            return false;
        }

        if !self.categories.is_empty() && !self.categories.contains(&category) {
            return false;
        }

        if let Some((start, end)) = self.time_range {
            if timestamp < start || timestamp > end {
                return false;
            }
        }

        true
    }
}
```

### 2. Retention Enforcer (src/enforcer.rs)

```rust
//! Retention policy enforcement.

use crate::{
    retention::{RetentionHold, RetentionPolicy},
    AuditCategory, AuditPersistence,
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

/// Retention enforcement result.
#[derive(Debug, Clone)]
pub struct EnforcementResult {
    pub events_deleted: u64,
    pub events_archived: u64,
    pub events_held: u64,
    pub duration_ms: u64,
}

/// Retention enforcer configuration.
#[derive(Debug, Clone)]
pub struct EnforcerConfig {
    /// How often to run enforcement.
    pub enforcement_interval: Duration,
    /// Maximum events to process per run.
    pub batch_size: u32,
    /// Dry run mode (no actual deletions).
    pub dry_run: bool,
}

impl Default for EnforcerConfig {
    fn default() -> Self {
        Self {
            enforcement_interval: Duration::from_secs(3600), // 1 hour
            batch_size: 10_000,
            dry_run: false,
        }
    }
}

/// Retention policy enforcer.
pub struct RetentionEnforcer {
    policy: Arc<RwLock<RetentionPolicy>>,
    holds: Arc<RwLock<Vec<RetentionHold>>>,
    conn: Arc<parking_lot::Mutex<Connection>>,
    config: EnforcerConfig,
}

impl RetentionEnforcer {
    /// Create a new enforcer.
    pub fn new(
        policy: RetentionPolicy,
        conn: Arc<parking_lot::Mutex<Connection>>,
        config: EnforcerConfig,
    ) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
            holds: Arc::new(RwLock::new(Vec::new())),
            conn,
            config,
        }
    }

    /// Update the retention policy.
    pub fn update_policy(&self, policy: RetentionPolicy) {
        *self.policy.write() = policy;
    }

    /// Add a retention hold.
    pub fn add_hold(&self, hold: RetentionHold) {
        self.holds.write().push(hold);
    }

    /// Remove a retention hold.
    pub fn remove_hold(&self, hold_id: &str) -> bool {
        let mut holds = self.holds.write();
        if let Some(pos) = holds.iter().position(|h| h.id == hold_id) {
            holds.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get active holds.
    pub fn active_holds(&self) -> Vec<RetentionHold> {
        self.holds.read().iter().filter(|h| h.is_active()).cloned().collect()
    }

    /// Run enforcement once.
    pub async fn enforce(&self) -> Result<EnforcementResult, EnforcementError> {
        let start = std::time::Instant::now();
        let policy = self.policy.read().clone();
        let holds = self.active_holds();

        let mut deleted = 0u64;
        let mut archived = 0u64;
        let mut held = 0u64;

        let conn = self.conn.lock();

        // Get events eligible for deletion
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, category, severity FROM audit_events
             ORDER BY timestamp ASC LIMIT ?"
        )?;

        let candidates: Vec<DeletionCandidate> = stmt
            .query_map([self.config.batch_size], |row| {
                Ok(DeletionCandidate {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    category: row.get(2)?,
                    severity: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut to_delete: Vec<String> = Vec::new();

        for candidate in candidates {
            let timestamp = DateTime::parse_from_rfc3339(&candidate.timestamp)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let category: AuditCategory = candidate.category.parse()
                .unwrap_or(AuditCategory::System);
            let severity = parse_severity(&candidate.severity);

            // Check if any hold applies
            let is_held = holds.iter().any(|h| h.applies_to(category, timestamp));
            if is_held {
                held += 1;
                continue;
            }

            // Check retention policy
            if !policy.should_retain(category, severity, timestamp) {
                to_delete.push(candidate.id);
            }
        }

        // Archive if configured
        if policy.archive_before_delete && !to_delete.is_empty() {
            archived = self.archive_events(&conn, &to_delete)?;
        }

        // Delete events
        if !self.config.dry_run && !to_delete.is_empty() {
            deleted = self.delete_events(&conn, &to_delete)?;
        } else if self.config.dry_run {
            debug!("Dry run: would delete {} events", to_delete.len());
            deleted = to_delete.len() as u64;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(EnforcementResult {
            events_deleted: deleted,
            events_archived: archived,
            events_held: held,
            duration_ms,
        })
    }

    fn archive_events(&self, conn: &Connection, ids: &[String]) -> Result<u64, EnforcementError> {
        // Archive to a separate table or file before deletion
        let mut count = 0u64;
        for id in ids {
            conn.execute(
                "INSERT INTO audit_archive SELECT * FROM audit_events WHERE id = ?",
                [id],
            )?;
            count += 1;
        }
        Ok(count)
    }

    fn delete_events(&self, conn: &Connection, ids: &[String]) -> Result<u64, EnforcementError> {
        let mut count = 0u64;
        for chunk in ids.chunks(100) {
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!("DELETE FROM audit_events WHERE id IN ({})", placeholders);
            let params: Vec<&dyn rusqlite::ToSql> = chunk.iter()
                .map(|s| s as &dyn rusqlite::ToSql)
                .collect();
            count += conn.execute(&sql, params.as_slice())? as u64;
        }
        Ok(count)
    }

    /// Start the background enforcement loop.
    pub async fn start_background_enforcement(self: Arc<Self>) {
        let mut ticker = interval(self.config.enforcement_interval);

        loop {
            ticker.tick().await;
            match self.enforce().await {
                Ok(result) => {
                    info!(
                        "Retention enforcement: deleted={}, archived={}, held={}, duration={}ms",
                        result.events_deleted,
                        result.events_archived,
                        result.events_held,
                        result.duration_ms
                    );
                }
                Err(e) => {
                    warn!("Retention enforcement failed: {}", e);
                }
            }
        }
    }
}

#[derive(Debug)]
struct DeletionCandidate {
    id: String,
    timestamp: String,
    category: String,
    severity: String,
}

fn parse_severity(s: &str) -> crate::AuditSeverity {
    match s.to_lowercase().as_str() {
        "critical" => crate::AuditSeverity::Critical,
        "high" => crate::AuditSeverity::High,
        "medium" => crate::AuditSeverity::Medium,
        "low" => crate::AuditSeverity::Low,
        _ => crate::AuditSeverity::Info,
    }
}

/// Enforcement error.
#[derive(Debug, thiserror::Error)]
pub enum EnforcementError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("policy error: {0}")]
    Policy(String),
}
```

---

## Testing Requirements

1. Retention durations calculate correct cutoffs
2. Category-specific retention works
3. Severity extensions apply correctly
4. Holds prevent deletion
5. Archive-before-delete works

---

## Related Specs

- Depends on: [434-audit-persistence.md](434-audit-persistence.md)
- Next: [437-audit-export.md](437-audit-export.md)
