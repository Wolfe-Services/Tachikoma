# 416 - Event Aggregation

## Overview

Pre-aggregation of analytics events for efficient querying, supporting time-series data, funnels, and retention analysis.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/aggregation.rs

use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Time granularity for aggregations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeGranularity {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

impl TimeGranularity {
    pub fn truncate(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            TimeGranularity::Minute => timestamp
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap(),
            TimeGranularity::Hour => timestamp
                .with_minute(0).unwrap()
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap(),
            TimeGranularity::Day => timestamp
                .date_naive()
                .and_hms_opt(0, 0, 0).unwrap()
                .and_utc(),
            TimeGranularity::Week => {
                let days_from_monday = timestamp.weekday().num_days_from_monday();
                (timestamp - Duration::days(days_from_monday as i64))
                    .date_naive()
                    .and_hms_opt(0, 0, 0).unwrap()
                    .and_utc()
            }
            TimeGranularity::Month => timestamp
                .with_day(1).unwrap()
                .date_naive()
                .and_hms_opt(0, 0, 0).unwrap()
                .and_utc(),
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            TimeGranularity::Minute => Duration::minutes(1),
            TimeGranularity::Hour => Duration::hours(1),
            TimeGranularity::Day => Duration::days(1),
            TimeGranularity::Week => Duration::weeks(1),
            TimeGranularity::Month => Duration::days(30), // Approximate
        }
    }
}

/// Aggregated event counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAggregate {
    /// Event name
    pub event: String,
    /// Time bucket
    pub timestamp: DateTime<Utc>,
    /// Granularity
    pub granularity: TimeGranularity,
    /// Environment
    pub environment: String,
    /// Total event count
    pub count: u64,
    /// Unique users
    pub unique_users: u64,
    /// Breakdown by property values
    pub breakdowns: HashMap<String, HashMap<String, u64>>,
}

/// Aggregation query
#[derive(Debug, Clone)]
pub struct AggregationQuery {
    /// Events to aggregate
    pub events: Vec<String>,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
    /// Time granularity
    pub granularity: TimeGranularity,
    /// Environment filter
    pub environment: Option<String>,
    /// Properties to break down by
    pub breakdown_by: Vec<String>,
    /// Filter by distinct_id
    pub distinct_id: Option<String>,
}

/// Aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// Time series data points
    pub series: Vec<TimeSeriesPoint>,
    /// Total count
    pub total: u64,
    /// Unique users
    pub unique_users: u64,
    /// Breakdown totals
    pub breakdown_totals: HashMap<String, HashMap<String, u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub count: u64,
    pub unique_users: u64,
    pub breakdown: Option<HashMap<String, u64>>,
}

/// Aggregation storage trait
#[async_trait]
pub trait AggregationStorage: Send + Sync {
    /// Store aggregated data
    async fn store(&self, aggregate: EventAggregate) -> Result<(), AggregationError>;

    /// Store batch of aggregations
    async fn store_batch(&self, aggregates: Vec<EventAggregate>) -> Result<(), AggregationError>;

    /// Query aggregated data
    async fn query(&self, query: AggregationQuery) -> Result<AggregationResult, AggregationError>;

    /// Refresh aggregations for time period
    async fn refresh(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<(), AggregationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum AggregationError {
    #[error("Query error: {0}")]
    Query(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

/// Real-time aggregation engine
pub struct AggregationEngine {
    /// In-memory aggregation buffer
    buffer: tokio::sync::RwLock<HashMap<AggregationKey, AggregationBuffer>>,
    /// Flush interval
    flush_interval: std::time::Duration,
    /// Storage backend
    storage: std::sync::Arc<dyn AggregationStorage>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct AggregationKey {
    event: String,
    timestamp: i64,
    granularity: String,
    environment: String,
}

#[derive(Debug, Clone)]
struct AggregationBuffer {
    count: u64,
    users: std::collections::HashSet<String>,
    breakdowns: HashMap<String, HashMap<String, u64>>,
}

impl AggregationEngine {
    pub fn new(
        storage: std::sync::Arc<dyn AggregationStorage>,
        flush_interval: std::time::Duration,
    ) -> Self {
        Self {
            buffer: tokio::sync::RwLock::new(HashMap::new()),
            flush_interval,
            storage,
        }
    }

    /// Start the aggregation engine
    pub async fn start(self: std::sync::Arc<Self>) {
        let engine = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(engine.flush_interval);

            loop {
                interval.tick().await;
                if let Err(e) = engine.flush().await {
                    tracing::error!("Failed to flush aggregations: {}", e);
                }
            }
        });
    }

    /// Increment aggregation counters
    pub async fn increment(
        &self,
        event: &str,
        distinct_id: &str,
        timestamp: DateTime<Utc>,
        environment: &str,
        granularity: TimeGranularity,
        properties: &HashMap<String, serde_json::Value>,
        breakdown_keys: &[String],
    ) {
        let bucket_time = granularity.truncate(timestamp);

        let key = AggregationKey {
            event: event.to_string(),
            timestamp: bucket_time.timestamp(),
            granularity: format!("{:?}", granularity),
            environment: environment.to_string(),
        };

        let mut buffer = self.buffer.write().await;
        let entry = buffer.entry(key).or_insert_with(|| AggregationBuffer {
            count: 0,
            users: std::collections::HashSet::new(),
            breakdowns: HashMap::new(),
        });

        entry.count += 1;
        entry.users.insert(distinct_id.to_string());

        // Track breakdowns
        for breakdown_key in breakdown_keys {
            if let Some(value) = properties.get(breakdown_key) {
                let value_str = value.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| value.to_string());

                *entry.breakdowns
                    .entry(breakdown_key.clone())
                    .or_insert_with(HashMap::new)
                    .entry(value_str)
                    .or_insert(0) += 1;
            }
        }
    }

    /// Flush buffered aggregations to storage
    pub async fn flush(&self) -> Result<(), AggregationError> {
        let aggregates = {
            let mut buffer = self.buffer.write().await;
            let entries: Vec<_> = buffer.drain().collect();
            entries
        };

        if aggregates.is_empty() {
            return Ok(());
        }

        let event_aggregates: Vec<EventAggregate> = aggregates.into_iter()
            .map(|(key, buf)| EventAggregate {
                event: key.event,
                timestamp: DateTime::from_timestamp(key.timestamp, 0).unwrap(),
                granularity: match key.granularity.as_str() {
                    "Minute" => TimeGranularity::Minute,
                    "Hour" => TimeGranularity::Hour,
                    "Day" => TimeGranularity::Day,
                    "Week" => TimeGranularity::Week,
                    "Month" => TimeGranularity::Month,
                    _ => TimeGranularity::Day,
                },
                environment: key.environment,
                count: buf.count,
                unique_users: buf.users.len() as u64,
                breakdowns: buf.breakdowns,
            })
            .collect();

        self.storage.store_batch(event_aggregates).await
    }
}

/// Funnel analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelDefinition {
    /// Funnel name
    pub name: String,
    /// Ordered steps
    pub steps: Vec<FunnelStep>,
    /// Conversion window (seconds)
    pub conversion_window_seconds: u64,
    /// Whether steps must be in exact order
    pub strict_order: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    /// Step name
    pub name: String,
    /// Event name
    pub event: String,
    /// Property filters for this step
    pub filters: Vec<PropertyFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyFilter {
    pub property: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelResult {
    /// Results per step
    pub steps: Vec<FunnelStepResult>,
    /// Overall conversion rate
    pub overall_conversion_rate: f64,
    /// Average time to convert
    pub avg_time_to_convert_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStepResult {
    /// Step name
    pub name: String,
    /// Users who reached this step
    pub users: u64,
    /// Conversion rate from previous step
    pub conversion_rate: f64,
    /// Drop-off rate from previous step
    pub drop_off_rate: f64,
    /// Average time to reach from previous step
    pub avg_time_from_previous_seconds: Option<f64>,
}

/// Retention analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionDefinition {
    /// Start event (cohort entry)
    pub start_event: String,
    /// Return event (retention signal)
    pub return_event: String,
    /// Time period for cohorts
    pub cohort_period: TimeGranularity,
    /// Number of periods to analyze
    pub periods: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionResult {
    /// Cohorts and their retention
    pub cohorts: Vec<CohortRetention>,
    /// Overall retention curve
    pub overall_retention: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortRetention {
    /// Cohort start date
    pub cohort_date: DateTime<Utc>,
    /// Initial cohort size
    pub cohort_size: u64,
    /// Retention percentages by period
    pub retention: Vec<f64>,
}

/// Pre-computed materialized views for common queries
pub mod materialized {
    use super::*;

    /// Daily active users table
    pub const DAU_TABLE: &str = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS dau_mv
        ENGINE = SummingMergeTree()
        PARTITION BY toYYYYMM(date)
        ORDER BY (environment, date)
        AS SELECT
            toDate(timestamp) AS date,
            environment,
            uniqState(distinct_id) AS users
        FROM events
        GROUP BY date, environment
    "#;

    /// Event counts by hour
    pub const HOURLY_EVENTS: &str = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS hourly_events_mv
        ENGINE = SummingMergeTree()
        PARTITION BY toYYYYMM(hour)
        ORDER BY (environment, event, hour)
        AS SELECT
            toStartOfHour(timestamp) AS hour,
            environment,
            event,
            count() AS count,
            uniqState(distinct_id) AS unique_users
        FROM events
        GROUP BY hour, environment, event
    "#;

    /// Session aggregations
    pub const SESSION_STATS: &str = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS session_stats_mv
        ENGINE = SummingMergeTree()
        PARTITION BY toYYYYMM(date)
        ORDER BY (environment, date)
        AS SELECT
            toDate(timestamp) AS date,
            environment,
            count() AS total_events,
            uniqState(session_id) AS sessions,
            uniqState(distinct_id) AS users
        FROM events
        WHERE session_id IS NOT NULL
        GROUP BY date, environment
    "#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_truncation() {
        let ts = DateTime::parse_from_rfc3339("2024-01-15T14:32:45Z")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(
            TimeGranularity::Hour.truncate(ts),
            DateTime::parse_from_rfc3339("2024-01-15T14:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );

        assert_eq!(
            TimeGranularity::Day.truncate(ts),
            DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
    }
}
```

## Aggregation Tables

```sql
-- Hourly event counts
CREATE TABLE event_counts_hourly (
    hour DateTime,
    environment String,
    event String,
    count UInt64,
    unique_users UInt64
)
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(hour)
ORDER BY (environment, event, hour);

-- Daily active users
CREATE TABLE daily_active_users (
    date Date,
    environment String,
    user_count UInt64
)
ENGINE = SummingMergeTree()
ORDER BY (environment, date);
```

## Related Specs

- 415-event-persistence.md - Raw event storage
- 423-analytics-query.md - Query interface
- 427-dashboard-data.md - Dashboard queries
