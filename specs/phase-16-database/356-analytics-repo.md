# Spec 356: Analytics Repository

## Overview
Implement the repository pattern for analytics data with event tracking, metric recording, and aggregation queries.

## Rust Implementation

### Analytics Repository
```rust
// src/database/repository/analytics.rs

use crate::database::schema::analytics::*;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum AnalyticsRepoError {
    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Invalid date range")]
    InvalidDateRange,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Query filters for events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub event_type: Option<String>,
    pub event_name: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

/// Query filters for metrics
#[derive(Debug, Clone, Default)]
pub struct MetricFilter {
    pub name: Option<String>,
    pub name_prefix: Option<String>,
    pub tags: Option<HashMap<String, String>>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

pub struct AnalyticsRepository {
    pool: SqlitePool,
}

impl AnalyticsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ==================== Event Methods ====================

    /// Track an analytics event
    #[instrument(skip(self, event), fields(event_name = %event.event_name))]
    pub async fn track_event(&self, event: AnalyticsEvent) -> Result<AnalyticsEvent, AnalyticsRepoError> {
        sqlx::query(r#"
            INSERT INTO analytics_events (
                id, event_type, event_name, timestamp,
                session_id, user_id, properties, context,
                page_url, referrer
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&event.id)
        .bind(&event.event_type)
        .bind(&event.event_name)
        .bind(event.timestamp)
        .bind(&event.session_id)
        .bind(&event.user_id)
        .bind(&event.properties)
        .bind(&event.context)
        .bind(&event.page_url)
        .bind(&event.referrer)
        .execute(&self.pool)
        .await?;

        // Update user activity if user_id is present
        if let Some(user_id) = &event.user_id {
            self.update_user_activity(user_id, &event.event_name).await?;
        }

        // Update feature usage
        self.update_feature_usage(&event.event_name).await?;

        debug!("Tracked event: {}", event.event_name);
        Ok(event)
    }

    /// Track multiple events in batch
    pub async fn track_events(&self, events: Vec<AnalyticsEvent>) -> Result<usize, AnalyticsRepoError> {
        let mut count = 0;
        for event in events {
            self.track_event(event).await?;
            count += 1;
        }
        Ok(count)
    }

    /// Query events with filters
    pub async fn query_events(
        &self,
        filter: EventFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AnalyticsEvent>, AnalyticsRepoError> {
        let mut sql = String::from("SELECT * FROM analytics_events WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(event_type) = &filter.event_type {
            sql.push_str(" AND event_type = ?");
            bindings.push(event_type.clone());
        }

        if let Some(event_name) = &filter.event_name {
            sql.push_str(" AND event_name = ?");
            bindings.push(event_name.clone());
        }

        if let Some(user_id) = &filter.user_id {
            sql.push_str(" AND user_id = ?");
            bindings.push(user_id.clone());
        }

        if let Some(session_id) = &filter.session_id {
            sql.push_str(" AND session_id = ?");
            bindings.push(session_id.clone());
        }

        if let Some(from) = filter.from_date {
            sql.push_str(" AND timestamp >= ?");
            bindings.push(from.to_rfc3339());
        }

        if let Some(to) = filter.to_date {
            sql.push_str(" AND timestamp <= ?");
            bindings.push(to.to_rfc3339());
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, AnalyticsEvent>(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(limit).bind(offset);

        let events = query.fetch_all(&self.pool).await?;
        Ok(events)
    }

    /// Count events by name in time range
    pub async fn count_events(
        &self,
        event_name: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AnalyticsRepoError> {
        let (count,): (i64,) = sqlx::query_as(r#"
            SELECT COUNT(*) FROM analytics_events
            WHERE event_name = ? AND timestamp >= ? AND timestamp <= ?
        "#)
        .bind(event_name)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Get event counts grouped by day
    pub async fn event_counts_by_day(
        &self,
        event_name: &str,
        days: i64,
    ) -> Result<Vec<(String, i64)>, AnalyticsRepoError> {
        let since = Utc::now() - Duration::days(days);

        let rows: Vec<(String, i64)> = sqlx::query_as(r#"
            SELECT date(timestamp) as day, COUNT(*) as count
            FROM analytics_events
            WHERE event_name = ? AND timestamp >= ?
            GROUP BY date(timestamp)
            ORDER BY day
        "#)
        .bind(event_name)
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    // ==================== Metric Methods ====================

    /// Record a metric
    #[instrument(skip(self, metric), fields(metric_name = %metric.name))]
    pub async fn record_metric(&self, metric: Metric) -> Result<Metric, AnalyticsRepoError> {
        sqlx::query(r#"
            INSERT INTO metrics (id, name, metric_type, value, timestamp, tags, unit)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&metric.id)
        .bind(&metric.name)
        .bind(metric.metric_type)
        .bind(metric.value)
        .bind(metric.timestamp)
        .bind(&metric.tags)
        .bind(&metric.unit)
        .execute(&self.pool)
        .await?;

        Ok(metric)
    }

    /// Record multiple metrics in batch
    pub async fn record_metrics(&self, metrics: Vec<Metric>) -> Result<usize, AnalyticsRepoError> {
        let mut count = 0;
        for metric in metrics {
            self.record_metric(metric).await?;
            count += 1;
        }
        Ok(count)
    }

    /// Query metrics
    pub async fn query_metrics(
        &self,
        filter: MetricFilter,
        limit: i64,
    ) -> Result<Vec<Metric>, AnalyticsRepoError> {
        let mut sql = String::from("SELECT * FROM metrics WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(name) = &filter.name {
            sql.push_str(" AND name = ?");
            bindings.push(name.clone());
        }

        if let Some(prefix) = &filter.name_prefix {
            sql.push_str(" AND name LIKE ?");
            bindings.push(format!("{}%", prefix));
        }

        if let Some(from) = filter.from_date {
            sql.push_str(" AND timestamp >= ?");
            bindings.push(from.to_rfc3339());
        }

        if let Some(to) = filter.to_date {
            sql.push_str(" AND timestamp <= ?");
            bindings.push(to.to_rfc3339());
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");

        let mut query = sqlx::query_as::<_, Metric>(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(limit);

        let metrics = query.fetch_all(&self.pool).await?;
        Ok(metrics)
    }

    /// Get metric statistics
    pub async fn metric_stats(
        &self,
        name: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<MetricStats, AnalyticsRepoError> {
        let row = sqlx::query_as::<_, MetricStats>(r#"
            SELECT
                COUNT(*) as count,
                COALESCE(SUM(value), 0) as sum,
                COALESCE(MIN(value), 0) as min,
                COALESCE(MAX(value), 0) as max,
                COALESCE(AVG(value), 0) as avg
            FROM metrics
            WHERE name = ? AND timestamp >= ? AND timestamp <= ?
        "#)
        .bind(name)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    // ==================== Aggregation Methods ====================

    /// Aggregate metrics for a time period
    #[instrument(skip(self))]
    pub async fn aggregate_metrics(
        &self,
        period_type: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<MetricAggregate>, AnalyticsRepoError> {
        let date_format = match period_type {
            "hourly" => "%Y-%m-%d %H:00:00",
            "daily" => "%Y-%m-%d",
            "weekly" => "%Y-W%W",
            "monthly" => "%Y-%m",
            _ => return Err(AnalyticsRepoError::InvalidDateRange),
        };

        let rows = sqlx::query_as::<_, MetricAggregateRow>(r#"
            SELECT
                name,
                strftime(?, timestamp) as period_start,
                COUNT(*) as count,
                SUM(value) as sum,
                MIN(value) as min,
                MAX(value) as max,
                AVG(value) as avg,
                tags
            FROM metrics
            WHERE timestamp >= ? AND timestamp <= ?
            GROUP BY name, strftime(?, timestamp), tags
        "#)
        .bind(date_format)
        .bind(from)
        .bind(to)
        .bind(date_format)
        .fetch_all(&self.pool)
        .await?;

        let aggregates: Vec<MetricAggregate> = rows.into_iter().map(|r| MetricAggregate {
            id: Uuid::new_v4().to_string(),
            name: r.name,
            period_start: from, // Would need proper parsing
            period_type: period_type.to_string(),
            count: r.count,
            sum: r.sum,
            min: r.min,
            max: r.max,
            avg: r.avg,
            p50: None,
            p90: None,
            p99: None,
            tags: r.tags,
        }).collect();

        // Store aggregates
        for agg in &aggregates {
            self.store_aggregate(agg).await?;
        }

        Ok(aggregates)
    }

    async fn store_aggregate(&self, agg: &MetricAggregate) -> Result<(), AnalyticsRepoError> {
        sqlx::query(r#"
            INSERT OR REPLACE INTO metric_aggregates (
                id, name, period_start, period_type, count, sum, min, max, avg, p50, p90, p99, tags
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&agg.id)
        .bind(&agg.name)
        .bind(agg.period_start)
        .bind(&agg.period_type)
        .bind(agg.count)
        .bind(agg.sum)
        .bind(agg.min)
        .bind(agg.max)
        .bind(agg.avg)
        .bind(agg.p50)
        .bind(agg.p90)
        .bind(agg.p99)
        .bind(&agg.tags)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ==================== User Activity Methods ====================

    async fn update_user_activity(&self, user_id: &str, event_name: &str) -> Result<(), AnalyticsRepoError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let now = Utc::now();

        // Try to update existing record
        let result = sqlx::query(r#"
            UPDATE user_activity
            SET event_count = event_count + 1,
                last_seen_at = ?
            WHERE user_id = ? AND date = ?
        "#)
        .bind(now)
        .bind(user_id)
        .bind(&today)
        .execute(&self.pool)
        .await?;

        // If no rows updated, insert new record
        if result.rows_affected() == 0 {
            let id = Uuid::new_v4().to_string();
            sqlx::query(r#"
                INSERT INTO user_activity (id, user_id, date, event_count, first_seen_at, last_seen_at)
                VALUES (?, ?, ?, 1, ?, ?)
            "#)
            .bind(&id)
            .bind(user_id)
            .bind(&today)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get user activity for date range
    pub async fn get_user_activity(
        &self,
        user_id: &str,
        days: i64,
    ) -> Result<Vec<UserActivity>, AnalyticsRepoError> {
        let since = (Utc::now() - Duration::days(days)).format("%Y-%m-%d").to_string();

        let activity = sqlx::query_as::<_, UserActivity>(r#"
            SELECT * FROM user_activity
            WHERE user_id = ? AND date >= ?
            ORDER BY date DESC
        "#)
        .bind(user_id)
        .bind(&since)
        .fetch_all(&self.pool)
        .await?;

        Ok(activity)
    }

    /// Get daily active users
    pub async fn daily_active_users(&self, days: i64) -> Result<Vec<(String, i64)>, AnalyticsRepoError> {
        let since = (Utc::now() - Duration::days(days)).format("%Y-%m-%d").to_string();

        let rows: Vec<(String, i64)> = sqlx::query_as(r#"
            SELECT date, COUNT(DISTINCT user_id) as count
            FROM user_activity
            WHERE date >= ?
            GROUP BY date
            ORDER BY date
        "#)
        .bind(&since)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    // ==================== Feature Usage Methods ====================

    async fn update_feature_usage(&self, feature_name: &str) -> Result<(), AnalyticsRepoError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let result = sqlx::query(r#"
            UPDATE feature_usage
            SET usage_count = usage_count + 1
            WHERE feature_name = ? AND date = ?
        "#)
        .bind(feature_name)
        .bind(&today)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            let id = Uuid::new_v4().to_string();
            sqlx::query(r#"
                INSERT INTO feature_usage (id, feature_name, date, usage_count, unique_users)
                VALUES (?, ?, ?, 1, 1)
            "#)
            .bind(&id)
            .bind(feature_name)
            .bind(&today)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get feature usage statistics
    pub async fn get_feature_usage(
        &self,
        feature_name: &str,
        days: i64,
    ) -> Result<Vec<FeatureUsage>, AnalyticsRepoError> {
        let since = (Utc::now() - Duration::days(days)).format("%Y-%m-%d").to_string();

        let usage = sqlx::query_as::<_, FeatureUsage>(r#"
            SELECT * FROM feature_usage
            WHERE feature_name = ? AND date >= ?
            ORDER BY date DESC
        "#)
        .bind(feature_name)
        .bind(&since)
        .fetch_all(&self.pool)
        .await?;

        Ok(usage)
    }

    /// Get top features by usage
    pub async fn top_features(&self, days: i64, limit: i64) -> Result<Vec<(String, i64)>, AnalyticsRepoError> {
        let since = (Utc::now() - Duration::days(days)).format("%Y-%m-%d").to_string();

        let rows: Vec<(String, i64)> = sqlx::query_as(r#"
            SELECT feature_name, SUM(usage_count) as total
            FROM feature_usage
            WHERE date >= ?
            GROUP BY feature_name
            ORDER BY total DESC
            LIMIT ?
        "#)
        .bind(&since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    // ==================== Dashboard Methods ====================

    /// Update dashboard metric
    pub async fn update_dashboard_metric(
        &self,
        widget_id: &str,
        metric_key: &str,
        value: f64,
        previous_value: Option<f64>,
    ) -> Result<(), AnalyticsRepoError> {
        let id = Uuid::new_v4().to_string();
        let change_percent = previous_value.map(|prev| {
            if prev == 0.0 { 0.0 } else { ((value - prev) / prev) * 100.0 }
        });

        sqlx::query(r#"
            INSERT OR REPLACE INTO dashboard_metrics (
                id, widget_id, metric_key, value, previous_value, change_percent, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
        "#)
        .bind(&id)
        .bind(widget_id)
        .bind(metric_key)
        .bind(value)
        .bind(previous_value)
        .bind(change_percent)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get dashboard metrics for widget
    pub async fn get_dashboard_metrics(&self, widget_id: &str) -> Result<Vec<DashboardMetric>, AnalyticsRepoError> {
        let metrics = sqlx::query_as::<_, DashboardMetric>(
            "SELECT * FROM dashboard_metrics WHERE widget_id = ?"
        )
        .bind(widget_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    // ==================== Cleanup Methods ====================

    /// Delete old analytics data
    pub async fn cleanup(&self, retention_days: i64) -> Result<i64, AnalyticsRepoError> {
        let cutoff = (Utc::now() - Duration::days(retention_days)).to_rfc3339();
        let mut total = 0i64;

        // Clean events
        let result = sqlx::query("DELETE FROM analytics_events WHERE timestamp < ?")
            .bind(&cutoff)
            .execute(&self.pool)
            .await?;
        total += result.rows_affected() as i64;

        // Clean metrics
        let result = sqlx::query("DELETE FROM metrics WHERE timestamp < ?")
            .bind(&cutoff)
            .execute(&self.pool)
            .await?;
        total += result.rows_affected() as i64;

        debug!("Cleaned up {} analytics records", total);
        Ok(total)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MetricStats {
    pub count: i64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct MetricAggregateRow {
    name: String,
    period_start: String,
    count: i64,
    sum: f64,
    min: f64,
    max: f64,
    avg: f64,
    tags: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/database/repository/analytics.rs` - Analytics repository
