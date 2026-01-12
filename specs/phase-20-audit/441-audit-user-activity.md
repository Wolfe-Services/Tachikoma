# 441 - Audit User Activity

**Phase:** 20 - Audit System
**Spec ID:** 441
**Status:** Planned
**Dependencies:** 435-audit-query, 440-audit-timeline
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement user activity tracking and reporting, providing per-user audit trails and activity summaries.

---

## Acceptance Criteria

- [ ] Per-user activity queries
- [ ] User activity summaries
- [ ] Session tracking
- [ ] Activity anomaly detection
- [ ] User activity reports

---

## Implementation Details

### 1. User Activity Types (src/user_activity.rs)

```rust
//! User activity tracking.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User activity summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    /// User identifier.
    pub user_id: String,
    /// User name (if available).
    pub username: Option<String>,
    /// Time period start.
    pub period_start: DateTime<Utc>,
    /// Time period end.
    pub period_end: DateTime<Utc>,
    /// Total actions.
    pub total_actions: u64,
    /// Actions by category.
    pub by_category: HashMap<String, u64>,
    /// Actions by action type.
    pub by_action: HashMap<String, u64>,
    /// Failed actions count.
    pub failed_actions: u64,
    /// Unique sessions.
    pub session_count: u64,
    /// Unique IP addresses.
    pub ip_addresses: Vec<String>,
    /// First activity in period.
    pub first_activity: Option<DateTime<Utc>>,
    /// Last activity in period.
    pub last_activity: Option<DateTime<Utc>>,
}

impl UserActivitySummary {
    /// Create an empty summary.
    pub fn new(user_id: impl Into<String>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            user_id: user_id.into(),
            username: None,
            period_start: start,
            period_end: end,
            total_actions: 0,
            by_category: HashMap::new(),
            by_action: HashMap::new(),
            failed_actions: 0,
            session_count: 0,
            ip_addresses: Vec::new(),
            first_activity: None,
            last_activity: None,
        }
    }

    /// Success rate as percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total_actions == 0 {
            100.0
        } else {
            let successful = self.total_actions - self.failed_actions;
            (successful as f64 / self.total_actions as f64) * 100.0
        }
    }

    /// Average actions per session.
    pub fn actions_per_session(&self) -> f64 {
        if self.session_count == 0 {
            0.0
        } else {
            self.total_actions as f64 / self.session_count as f64
        }
    }
}

/// User session information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session identifier.
    pub session_id: String,
    /// User identifier.
    pub user_id: String,
    /// Session start time.
    pub started_at: DateTime<Utc>,
    /// Session end time (if ended).
    pub ended_at: Option<DateTime<Utc>>,
    /// IP address.
    pub ip_address: Option<String>,
    /// User agent.
    pub user_agent: Option<String>,
    /// Action count in this session.
    pub action_count: u64,
    /// Last activity in session.
    pub last_activity: DateTime<Utc>,
}

impl UserSession {
    /// Session duration.
    pub fn duration(&self) -> Duration {
        let end = self.ended_at.unwrap_or(self.last_activity);
        end - self.started_at
    }

    /// Check if session appears to be active.
    pub fn is_active(&self, timeout: Duration) -> bool {
        self.ended_at.is_none() && (Utc::now() - self.last_activity) < timeout
    }
}

/// Activity anomaly detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityAnomaly {
    /// User identifier.
    pub user_id: String,
    /// Anomaly type.
    pub anomaly_type: AnomalyType,
    /// Severity score (0-100).
    pub severity: u32,
    /// Description.
    pub description: String,
    /// When detected.
    pub detected_at: DateTime<Utc>,
    /// Related event IDs.
    pub related_events: Vec<String>,
}

/// Types of activity anomalies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    /// Unusual login time.
    UnusualLoginTime,
    /// Unusual location/IP.
    UnusualLocation,
    /// Excessive failed attempts.
    ExcessiveFailures,
    /// Unusual activity volume.
    UnusualVolume,
    /// Privilege escalation attempt.
    PrivilegeEscalation,
    /// Access to unusual resources.
    UnusualAccess,
    /// Concurrent sessions from different locations.
    ConcurrentSessions,
}
```

### 2. Activity Tracker (src/activity_tracker.rs)

```rust
//! User activity tracking and analysis.

use crate::user_activity::*;
use chrono::{DateTime, Duration, Utc};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Configuration for activity tracking.
#[derive(Debug, Clone)]
pub struct ActivityConfig {
    /// Session timeout duration.
    pub session_timeout: Duration,
    /// Threshold for excessive failures.
    pub failure_threshold: u32,
    /// Threshold for unusual volume (actions per hour).
    pub volume_threshold: u32,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            session_timeout: Duration::minutes(30),
            failure_threshold: 5,
            volume_threshold: 100,
        }
    }
}

/// User activity tracker.
pub struct ActivityTracker {
    conn: Arc<Mutex<Connection>>,
    config: ActivityConfig,
}

impl ActivityTracker {
    /// Create a new activity tracker.
    pub fn new(conn: Arc<Mutex<Connection>>, config: ActivityConfig) -> Self {
        Self { conn, config }
    }

    /// Get activity summary for a user.
    pub fn user_summary(
        &self,
        user_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<UserActivitySummary, ActivityError> {
        let conn = self.conn.lock();
        let mut summary = UserActivitySummary::new(user_id, start, end);

        // Get activity counts
        let mut stmt = conn.prepare(
            "SELECT category, action, outcome, timestamp, ip_address
             FROM audit_events
             WHERE actor_id = ? AND timestamp >= ? AND timestamp < ?
             ORDER BY timestamp"
        )?;

        let mut sessions = HashSet::new();
        let mut ips = HashSet::new();

        let rows = stmt.query_map(
            rusqlite::params![user_id, start.to_rfc3339(), end.to_rfc3339()],
            |row| {
                Ok(ActivityRow {
                    category: row.get(0)?,
                    action: row.get(1)?,
                    outcome: row.get(2)?,
                    timestamp: row.get(3)?,
                    ip_address: row.get(4)?,
                })
            },
        )?;

        for row in rows {
            let row = row?;
            summary.total_actions += 1;

            *summary.by_category.entry(row.category.clone()).or_insert(0) += 1;
            *summary.by_action.entry(row.action.clone()).or_insert(0) += 1;

            if row.outcome != "success" {
                summary.failed_actions += 1;
            }

            if let Ok(ts) = DateTime::parse_from_rfc3339(&row.timestamp) {
                let ts = ts.with_timezone(&Utc);
                if summary.first_activity.is_none() {
                    summary.first_activity = Some(ts);
                }
                summary.last_activity = Some(ts);
            }

            if let Some(ip) = row.ip_address {
                ips.insert(ip);
            }

            // Simple session heuristic: group by hour
            if let Ok(ts) = DateTime::parse_from_rfc3339(&row.timestamp) {
                let session_key = ts.format("%Y-%m-%d-%H").to_string();
                sessions.insert(session_key);
            }
        }

        summary.session_count = sessions.len() as u64;
        summary.ip_addresses = ips.into_iter().collect();

        // Get username
        let username: Option<String> = conn.query_row(
            "SELECT actor_name FROM audit_events WHERE actor_id = ? LIMIT 1",
            [user_id],
            |row| row.get(0),
        ).ok();
        summary.username = username;

        Ok(summary)
    }

    /// Get recent sessions for a user.
    pub fn user_sessions(
        &self,
        user_id: &str,
        limit: u32,
    ) -> Result<Vec<UserSession>, ActivityError> {
        let conn = self.conn.lock();

        // This is a simplified implementation
        // A real implementation would track sessions more precisely
        let mut stmt = conn.prepare(
            "SELECT MIN(timestamp), MAX(timestamp), ip_address, user_agent, COUNT(*)
             FROM audit_events
             WHERE actor_id = ?
             GROUP BY date(timestamp), ip_address
             ORDER BY MIN(timestamp) DESC
             LIMIT ?"
        )?;

        let sessions: Vec<UserSession> = stmt
            .query_map(rusqlite::params![user_id, limit], |row| {
                let start: String = row.get(0)?;
                let end: String = row.get(1)?;
                Ok(UserSession {
                    session_id: format!("sess_{}", uuid::Uuid::new_v4()),
                    user_id: user_id.to_string(),
                    started_at: DateTime::parse_from_rfc3339(&start)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    ended_at: Some(DateTime::parse_from_rfc3339(&end)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now())),
                    ip_address: row.get(2)?,
                    user_agent: row.get(3)?,
                    action_count: row.get::<_, i64>(4)? as u64,
                    last_activity: DateTime::parse_from_rfc3339(&end)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(sessions)
    }

    /// Detect anomalies for a user.
    pub fn detect_anomalies(&self, user_id: &str) -> Result<Vec<ActivityAnomaly>, ActivityError> {
        let mut anomalies = Vec::new();
        let now = Utc::now();
        let last_hour = now - Duration::hours(1);

        let conn = self.conn.lock();

        // Check for excessive failures
        let failures: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events
             WHERE actor_id = ? AND timestamp >= ? AND outcome != 'success'",
            rusqlite::params![user_id, last_hour.to_rfc3339()],
            |row| row.get(0),
        )?;

        if failures > self.config.failure_threshold as i64 {
            anomalies.push(ActivityAnomaly {
                user_id: user_id.to_string(),
                anomaly_type: AnomalyType::ExcessiveFailures,
                severity: ((failures as f64 / self.config.failure_threshold as f64) * 50.0).min(100.0) as u32,
                description: format!("{} failed actions in the last hour", failures),
                detected_at: now,
                related_events: Vec::new(),
            });
        }

        // Check for unusual volume
        let volume: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events
             WHERE actor_id = ? AND timestamp >= ?",
            rusqlite::params![user_id, last_hour.to_rfc3339()],
            |row| row.get(0),
        )?;

        if volume > self.config.volume_threshold as i64 {
            anomalies.push(ActivityAnomaly {
                user_id: user_id.to_string(),
                anomaly_type: AnomalyType::UnusualVolume,
                severity: ((volume as f64 / self.config.volume_threshold as f64) * 50.0).min(100.0) as u32,
                description: format!("{} actions in the last hour", volume),
                detected_at: now,
                related_events: Vec::new(),
            });
        }

        Ok(anomalies)
    }
}

struct ActivityRow {
    category: String,
    action: String,
    outcome: String,
    timestamp: String,
    ip_address: Option<String>,
}

/// Activity tracking error.
#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}
```

---

## Testing Requirements

1. User summaries aggregate correctly
2. Session tracking groups events properly
3. Anomaly detection triggers appropriately
4. Success rate calculation is accurate
5. Time range filtering works

---

## Related Specs

- Depends on: [435-audit-query.md](435-audit-query.md), [440-audit-timeline.md](440-audit-timeline.md)
- Next: [442-audit-system-events.md](442-audit-system-events.md)
