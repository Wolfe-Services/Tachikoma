# 440 - Audit Timeline

**Phase:** 20 - Audit System
**Spec ID:** 440
**Status:** Planned
**Dependencies:** 435-audit-query
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement timeline visualization support for audit events, enabling chronological views and event correlation across time.

---

## Acceptance Criteria

- [ ] Timeline data aggregation
- [ ] Event grouping by time intervals
- [ ] Related event clustering
- [ ] Activity heatmaps
- [ ] Timeline navigation helpers

---

## Implementation Details

### 1. Timeline Types (src/timeline.rs)

```rust
//! Audit timeline data structures.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time interval granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeGranularity {
    Minute,
    FiveMinutes,
    FifteenMinutes,
    Hour,
    Day,
    Week,
    Month,
}

impl TimeGranularity {
    /// Get the duration of this granularity.
    pub fn duration(&self) -> Duration {
        match self {
            Self::Minute => Duration::minutes(1),
            Self::FiveMinutes => Duration::minutes(5),
            Self::FifteenMinutes => Duration::minutes(15),
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::days(1),
            Self::Week => Duration::weeks(1),
            Self::Month => Duration::days(30),
        }
    }

    /// Truncate a timestamp to this granularity.
    pub fn truncate(&self, ts: DateTime<Utc>) -> DateTime<Utc> {
        use chrono::Timelike;
        match self {
            Self::Minute => ts.with_second(0).unwrap().with_nanosecond(0).unwrap(),
            Self::FiveMinutes => {
                let mins = (ts.minute() / 5) * 5;
                ts.with_minute(mins).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
            }
            Self::FifteenMinutes => {
                let mins = (ts.minute() / 15) * 15;
                ts.with_minute(mins).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
            }
            Self::Hour => ts.with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap(),
            Self::Day => ts.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            Self::Week => {
                use chrono::Datelike;
                let days_since_monday = ts.weekday().num_days_from_monday();
                (ts - Duration::days(days_since_monday as i64))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            }
            Self::Month => {
                use chrono::Datelike;
                ts.date_naive()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            }
        }
    }

    /// Suggest granularity based on time range.
    pub fn suggest_for_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let duration = end - start;
        let hours = duration.num_hours();

        if hours <= 1 {
            Self::Minute
        } else if hours <= 6 {
            Self::FiveMinutes
        } else if hours <= 24 {
            Self::FifteenMinutes
        } else if hours <= 24 * 7 {
            Self::Hour
        } else if hours <= 24 * 30 {
            Self::Day
        } else if hours <= 24 * 90 {
            Self::Week
        } else {
            Self::Month
        }
    }
}

/// A single bucket in the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineBucket {
    /// Start of this bucket.
    pub start: DateTime<Utc>,
    /// End of this bucket.
    pub end: DateTime<Utc>,
    /// Total event count.
    pub count: u64,
    /// Event counts by category.
    pub by_category: HashMap<String, u64>,
    /// Event counts by severity.
    pub by_severity: HashMap<String, u64>,
    /// Notable event IDs in this bucket.
    pub notable_events: Vec<String>,
}

impl TimelineBucket {
    /// Create a new bucket.
    pub fn new(start: DateTime<Utc>, granularity: TimeGranularity) -> Self {
        Self {
            start,
            end: start + granularity.duration(),
            count: 0,
            by_category: HashMap::new(),
            by_severity: HashMap::new(),
            notable_events: Vec::new(),
        }
    }

    /// Add an event to this bucket.
    pub fn add_event(&mut self, category: &str, severity: &str, event_id: &str, is_notable: bool) {
        self.count += 1;
        *self.by_category.entry(category.to_string()).or_insert(0) += 1;
        *self.by_severity.entry(severity.to_string()).or_insert(0) += 1;

        if is_notable && self.notable_events.len() < 10 {
            self.notable_events.push(event_id.to_string());
        }
    }
}

/// Timeline data for a time range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Start of the timeline.
    pub start: DateTime<Utc>,
    /// End of the timeline.
    pub end: DateTime<Utc>,
    /// Granularity used.
    pub granularity: TimeGranularity,
    /// Time buckets.
    pub buckets: Vec<TimelineBucket>,
    /// Total event count.
    pub total_events: u64,
    /// Peak bucket (highest count).
    pub peak_bucket_index: Option<usize>,
}

impl Timeline {
    /// Create a new timeline.
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>, granularity: TimeGranularity) -> Self {
        let mut buckets = Vec::new();
        let mut current = granularity.truncate(start);

        while current < end {
            buckets.push(TimelineBucket::new(current, granularity));
            current = current + granularity.duration();
        }

        Self {
            start,
            end,
            granularity,
            buckets,
            total_events: 0,
            peak_bucket_index: None,
        }
    }

    /// Find the bucket for a timestamp.
    pub fn bucket_for(&mut self, ts: DateTime<Utc>) -> Option<&mut TimelineBucket> {
        let truncated = self.granularity.truncate(ts);
        self.buckets.iter_mut().find(|b| b.start == truncated)
    }

    /// Finalize the timeline after adding all events.
    pub fn finalize(&mut self) {
        self.total_events = self.buckets.iter().map(|b| b.count).sum();
        self.peak_bucket_index = self.buckets
            .iter()
            .enumerate()
            .max_by_key(|(_, b)| b.count)
            .map(|(i, _)| i);
    }
}
```

### 2. Timeline Builder (src/timeline_builder.rs)

```rust
//! Timeline construction from audit events.

use crate::timeline::{TimeGranularity, Timeline, TimelineBucket};
use crate::AuditSeverity;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

/// Configuration for timeline generation.
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    /// Time range start.
    pub start: DateTime<Utc>,
    /// Time range end.
    pub end: DateTime<Utc>,
    /// Granularity (auto-selected if None).
    pub granularity: Option<TimeGranularity>,
    /// Category filter.
    pub categories: Option<Vec<String>>,
    /// Minimum severity for notable events.
    pub notable_min_severity: AuditSeverity,
}

impl TimelineConfig {
    /// Create for the last N hours.
    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        Self {
            start,
            end,
            granularity: None,
            categories: None,
            notable_min_severity: AuditSeverity::High,
        }
    }

    /// Create for the last N days.
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        Self {
            start,
            end,
            granularity: None,
            categories: None,
            notable_min_severity: AuditSeverity::High,
        }
    }
}

/// Timeline builder.
pub struct TimelineBuilder {
    conn: Arc<Mutex<Connection>>,
}

impl TimelineBuilder {
    /// Create a new timeline builder.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Build a timeline from the database.
    pub fn build(&self, config: &TimelineConfig) -> Result<Timeline, TimelineError> {
        let granularity = config.granularity
            .unwrap_or_else(|| TimeGranularity::suggest_for_range(config.start, config.end));

        let mut timeline = Timeline::new(config.start, config.end, granularity);
        let conn = self.conn.lock();

        let mut sql = String::from(
            "SELECT id, timestamp, category, severity FROM audit_events
             WHERE timestamp >= ? AND timestamp < ?"
        );

        if let Some(ref cats) = config.categories {
            let placeholders = cats.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND category IN ({})", placeholders));
        }

        sql.push_str(" ORDER BY timestamp");

        let mut stmt = conn.prepare(&sql)?;

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(config.start.to_rfc3339()),
            Box::new(config.end.to_rfc3339()),
        ];

        if let Some(ref cats) = config.categories {
            for cat in cats {
                params.push(Box::new(cat.clone()));
            }
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(EventRow {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                category: row.get(2)?,
                severity: row.get(3)?,
            })
        })?;

        for row in rows {
            let row = row?;
            let ts = DateTime::parse_from_rfc3339(&row.timestamp)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| config.start);

            let is_notable = is_severity_notable(&row.severity, config.notable_min_severity);

            if let Some(bucket) = timeline.bucket_for(ts) {
                bucket.add_event(&row.category, &row.severity, &row.id, is_notable);
            }
        }

        timeline.finalize();
        Ok(timeline)
    }

    /// Build an activity heatmap.
    pub fn build_heatmap(&self, config: &TimelineConfig) -> Result<ActivityHeatmap, TimelineError> {
        let timeline = self.build(config)?;
        Ok(ActivityHeatmap::from_timeline(&timeline))
    }
}

struct EventRow {
    id: String,
    timestamp: String,
    category: String,
    severity: String,
}

fn is_severity_notable(severity: &str, min: AuditSeverity) -> bool {
    let sev = match severity.to_lowercase().as_str() {
        "critical" => AuditSeverity::Critical,
        "high" => AuditSeverity::High,
        "medium" => AuditSeverity::Medium,
        "low" => AuditSeverity::Low,
        _ => AuditSeverity::Info,
    };
    sev >= min
}

/// Activity heatmap data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeatmap {
    /// Heatmap data by hour of day (0-23) and day of week (0-6).
    pub data: [[u64; 7]; 24],
    /// Maximum value in the heatmap.
    pub max_value: u64,
}

use serde::{Deserialize, Serialize};

impl ActivityHeatmap {
    /// Create from a timeline.
    pub fn from_timeline(timeline: &Timeline) -> Self {
        let mut data = [[0u64; 7]; 24];
        let mut max_value = 0u64;

        for bucket in &timeline.buckets {
            use chrono::{Datelike, Timelike};
            let hour = bucket.start.hour() as usize;
            let dow = bucket.start.weekday().num_days_from_monday() as usize;

            data[hour][dow] += bucket.count;
            max_value = max_value.max(data[hour][dow]);
        }

        Self { data, max_value }
    }
}

/// Timeline error.
#[derive(Debug, thiserror::Error)]
pub enum TimelineError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}
```

---

## Testing Requirements

1. Granularity truncation is correct
2. Auto-granularity selection is reasonable
3. Buckets cover the full time range
4. Notable events are captured
5. Heatmap generation works

---

## Related Specs

- Depends on: [435-audit-query.md](435-audit-query.md)
- Next: [441-audit-user-activity.md](441-audit-user-activity.md)
