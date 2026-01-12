# Spec 355: Analytics Database Schema

## Overview
Define the SQLite schema for analytics data, including metrics, events, and aggregated statistics.

## Rust Implementation

### Schema Models
```rust
// src/database/schema/analytics.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// Analytics event
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: String,
    pub event_type: String,
    pub event_name: String,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub properties: Option<String>,  // JSON
    pub context: Option<String>,     // JSON (device, browser, etc.)
    pub page_url: Option<String>,
    pub referrer: Option<String>,
}

/// Time-series metric
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Metric {
    pub id: String,
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: Option<String>,  // JSON key-value pairs
    pub unit: Option<String>,
}

/// Aggregated metric (hourly/daily)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MetricAggregate {
    pub id: String,
    pub name: String,
    pub period_start: DateTime<Utc>,
    pub period_type: String,  // hourly, daily, weekly
    pub count: i64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: Option<f64>,
    pub p90: Option<f64>,
    pub p99: Option<f64>,
    pub tags: Option<String>,
}

/// User activity summary
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserActivity {
    pub id: String,
    pub user_id: String,
    pub date: String,  // YYYY-MM-DD
    pub session_count: i32,
    pub event_count: i32,
    pub total_duration_ms: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub events_breakdown: Option<String>,  // JSON
}

/// Feature usage tracking
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FeatureUsage {
    pub id: String,
    pub feature_name: String,
    pub date: String,
    pub usage_count: i64,
    pub unique_users: i64,
    pub avg_duration_ms: Option<f64>,
    pub success_rate: Option<f64>,
}

/// Dashboard widget data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DashboardMetric {
    pub id: String,
    pub widget_id: String,
    pub metric_key: String,
    pub value: f64,
    pub previous_value: Option<f64>,
    pub change_percent: Option<f64>,
    pub updated_at: DateTime<Utc>,
}
```

### Migration SQL
```rust
// src/database/migrations/006_create_analytics.rs

use crate::database::migration::Migration;

pub fn migration() -> Migration {
    Migration::new(
        20240101000006,
        "create_analytics",
        r#"
-- Analytics events
CREATE TABLE IF NOT EXISTS analytics_events (
    id TEXT PRIMARY KEY NOT NULL,
    event_type TEXT NOT NULL,
    event_name TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    session_id TEXT,
    user_id TEXT,
    properties TEXT,
    context TEXT,
    page_url TEXT,
    referrer TEXT
);

CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_name ON analytics_events(event_name);
CREATE INDEX IF NOT EXISTS idx_analytics_events_timestamp ON analytics_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_analytics_events_user ON analytics_events(user_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_session ON analytics_events(session_id);

-- Time-series metrics
CREATE TABLE IF NOT EXISTS metrics (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    metric_type TEXT NOT NULL DEFAULT 'gauge'
        CHECK (metric_type IN ('counter', 'gauge', 'histogram', 'timer')),
    value REAL NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    tags TEXT,
    unit TEXT
);

CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics(name);
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_metrics_name_time ON metrics(name, timestamp);

-- Aggregated metrics
CREATE TABLE IF NOT EXISTS metric_aggregates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_type TEXT NOT NULL CHECK (period_type IN ('hourly', 'daily', 'weekly', 'monthly')),
    count INTEGER NOT NULL DEFAULT 0,
    sum REAL NOT NULL DEFAULT 0,
    min REAL NOT NULL DEFAULT 0,
    max REAL NOT NULL DEFAULT 0,
    avg REAL NOT NULL DEFAULT 0,
    p50 REAL,
    p90 REAL,
    p99 REAL,
    tags TEXT,
    UNIQUE(name, period_start, period_type, tags)
);

CREATE INDEX IF NOT EXISTS idx_metric_agg_name ON metric_aggregates(name);
CREATE INDEX IF NOT EXISTS idx_metric_agg_period ON metric_aggregates(period_start, period_type);

-- User activity summary
CREATE TABLE IF NOT EXISTS user_activity (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    date TEXT NOT NULL,  -- YYYY-MM-DD
    session_count INTEGER NOT NULL DEFAULT 0,
    event_count INTEGER NOT NULL DEFAULT 0,
    total_duration_ms INTEGER NOT NULL DEFAULT 0,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    events_breakdown TEXT,
    UNIQUE(user_id, date)
);

CREATE INDEX IF NOT EXISTS idx_user_activity_user ON user_activity(user_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_date ON user_activity(date);

-- Feature usage tracking
CREATE TABLE IF NOT EXISTS feature_usage (
    id TEXT PRIMARY KEY NOT NULL,
    feature_name TEXT NOT NULL,
    date TEXT NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    unique_users INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms REAL,
    success_rate REAL,
    UNIQUE(feature_name, date)
);

CREATE INDEX IF NOT EXISTS idx_feature_usage_name ON feature_usage(feature_name);
CREATE INDEX IF NOT EXISTS idx_feature_usage_date ON feature_usage(date);

-- Dashboard metrics (pre-computed for fast loading)
CREATE TABLE IF NOT EXISTS dashboard_metrics (
    id TEXT PRIMARY KEY NOT NULL,
    widget_id TEXT NOT NULL,
    metric_key TEXT NOT NULL,
    value REAL NOT NULL,
    previous_value REAL,
    change_percent REAL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(widget_id, metric_key)
);

CREATE INDEX IF NOT EXISTS idx_dashboard_metrics_widget ON dashboard_metrics(widget_id);

-- Session tracking
CREATE TABLE IF NOT EXISTS analytics_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    duration_ms INTEGER,
    event_count INTEGER NOT NULL DEFAULT 0,
    device_type TEXT,
    browser TEXT,
    os TEXT,
    country TEXT,
    city TEXT,
    referrer TEXT,
    landing_page TEXT,
    exit_page TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON analytics_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_started ON analytics_sessions(started_at);

-- Funnel tracking
CREATE TABLE IF NOT EXISTS funnels (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    steps TEXT NOT NULL,  -- JSON array of step definitions
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS funnel_conversions (
    id TEXT PRIMARY KEY NOT NULL,
    funnel_id TEXT NOT NULL REFERENCES funnels(id) ON DELETE CASCADE,
    user_id TEXT,
    session_id TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    current_step INTEGER NOT NULL DEFAULT 0,
    completed INTEGER NOT NULL DEFAULT 0,
    step_timestamps TEXT  -- JSON array of timestamps for each step
);

CREATE INDEX IF NOT EXISTS idx_funnel_conv_funnel ON funnel_conversions(funnel_id);
CREATE INDEX IF NOT EXISTS idx_funnel_conv_user ON funnel_conversions(user_id);
"#
    ).with_down(r#"
DROP TABLE IF EXISTS funnel_conversions;
DROP TABLE IF EXISTS funnels;
DROP TABLE IF EXISTS analytics_sessions;
DROP TABLE IF EXISTS dashboard_metrics;
DROP TABLE IF EXISTS feature_usage;
DROP TABLE IF EXISTS user_activity;
DROP TABLE IF EXISTS metric_aggregates;
DROP TABLE IF EXISTS metrics;
DROP TABLE IF EXISTS analytics_events;
"#)
}
```

### Analytics Event Builder
```rust
// src/database/schema/analytics_builder.rs

use super::analytics::*;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

/// Builder for analytics events
pub struct EventBuilder {
    event: AnalyticsEvent,
}

impl EventBuilder {
    pub fn new(event_type: impl Into<String>, event_name: impl Into<String>) -> Self {
        Self {
            event: AnalyticsEvent {
                id: Uuid::new_v4().to_string(),
                event_type: event_type.into(),
                event_name: event_name.into(),
                timestamp: Utc::now(),
                session_id: None,
                user_id: None,
                properties: None,
                context: None,
                page_url: None,
                referrer: None,
            },
        }
    }

    pub fn track(name: impl Into<String>) -> Self {
        Self::new("track", name)
    }

    pub fn page(url: impl Into<String>) -> Self {
        let url = url.into();
        Self::new("page", "page_view").page_url(&url)
    }

    pub fn identify(user_id: impl Into<String>) -> Self {
        Self::new("identify", "user_identified").user_id(&user_id.into())
    }

    pub fn user_id(mut self, id: &str) -> Self {
        self.event.user_id = Some(id.to_string());
        self
    }

    pub fn session_id(mut self, id: &str) -> Self {
        self.event.session_id = Some(id.to_string());
        self
    }

    pub fn properties(mut self, props: Value) -> Self {
        self.event.properties = Some(props.to_string());
        self
    }

    pub fn property(mut self, key: &str, value: impl Into<Value>) -> Self {
        let mut props: Value = self.event.properties
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(ref mut map) = props {
            map.insert(key.to_string(), value.into());
        }

        self.event.properties = Some(props.to_string());
        self
    }

    pub fn context(mut self, ctx: Value) -> Self {
        self.event.context = Some(ctx.to_string());
        self
    }

    pub fn page_url(mut self, url: &str) -> Self {
        self.event.page_url = Some(url.to_string());
        self
    }

    pub fn referrer(mut self, referrer: &str) -> Self {
        self.event.referrer = Some(referrer.to_string());
        self
    }

    pub fn build(self) -> AnalyticsEvent {
        self.event
    }
}

/// Builder for metrics
pub struct MetricBuilder {
    metric: Metric,
}

impl MetricBuilder {
    pub fn counter(name: impl Into<String>, value: f64) -> Self {
        Self::new(name, MetricType::Counter, value)
    }

    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self::new(name, MetricType::Gauge, value)
    }

    pub fn timer(name: impl Into<String>, duration_ms: f64) -> Self {
        Self::new(name, MetricType::Timer, duration_ms).unit("ms")
    }

    pub fn histogram(name: impl Into<String>, value: f64) -> Self {
        Self::new(name, MetricType::Histogram, value)
    }

    fn new(name: impl Into<String>, metric_type: MetricType, value: f64) -> Self {
        Self {
            metric: Metric {
                id: Uuid::new_v4().to_string(),
                name: name.into(),
                metric_type,
                value,
                timestamp: Utc::now(),
                tags: None,
                unit: None,
            },
        }
    }

    pub fn tag(mut self, key: &str, value: &str) -> Self {
        let mut tags: Value = self.metric.tags
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(ref mut map) = tags {
            map.insert(key.to_string(), Value::String(value.to_string()));
        }

        self.metric.tags = Some(tags.to_string());
        self
    }

    pub fn unit(mut self, unit: &str) -> Self {
        self.metric.unit = Some(unit.to_string());
        self
    }

    pub fn build(self) -> Metric {
        self.metric
    }
}

/// Predefined event types
pub mod events {
    use super::*;

    pub fn mission_created(mission_id: &str) -> EventBuilder {
        EventBuilder::track("mission_created")
            .property("mission_id", mission_id)
    }

    pub fn mission_completed(mission_id: &str, duration_ms: i64) -> EventBuilder {
        EventBuilder::track("mission_completed")
            .property("mission_id", mission_id)
            .property("duration_ms", duration_ms)
    }

    pub fn spec_created(spec_id: &str, mission_id: &str) -> EventBuilder {
        EventBuilder::track("spec_created")
            .property("spec_id", spec_id)
            .property("mission_id", mission_id)
    }

    pub fn forge_item_created(item_id: &str, item_type: &str) -> EventBuilder {
        EventBuilder::track("forge_item_created")
            .property("item_id", item_id)
            .property("item_type", item_type)
    }

    pub fn search_performed(query: &str, result_count: i32) -> EventBuilder {
        EventBuilder::track("search_performed")
            .property("query", query)
            .property("result_count", result_count)
    }
}

/// Predefined metrics
pub mod metrics {
    use super::*;

    pub fn api_request(endpoint: &str, duration_ms: f64, status: i32) -> MetricBuilder {
        MetricBuilder::timer("api.request.duration", duration_ms)
            .tag("endpoint", endpoint)
            .tag("status", &status.to_string())
    }

    pub fn active_users(count: f64) -> MetricBuilder {
        MetricBuilder::gauge("users.active", count)
    }

    pub fn missions_created() -> MetricBuilder {
        MetricBuilder::counter("missions.created", 1.0)
    }

    pub fn database_query(query_type: &str, duration_ms: f64) -> MetricBuilder {
        MetricBuilder::timer("database.query.duration", duration_ms)
            .tag("query_type", query_type)
    }
}
```

## Schema Design Decisions

1. **Event-based**: Track user interactions as events
2. **Time-series**: Efficient storage for metrics over time
3. **Pre-aggregation**: Aggregate tables for fast dashboard queries
4. **Funnel Support**: Track conversion funnels
5. **Session Tracking**: Link events to sessions

## Files to Create
- `src/database/schema/analytics.rs` - Analytics models
- `src/database/schema/analytics_builder.rs` - Builder helpers
- `src/database/migrations/006_create_analytics.rs` - Migration
