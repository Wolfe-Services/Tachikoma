# 415 - Event Persistence

## Overview

Storage layer for analytics events supporting multiple backends including ClickHouse, PostgreSQL, and blob storage.

## Rust Implementation

```rust
// crates/analytics/src/persistence.rs

use crate::event_types::AnalyticsEvent;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Query error: {0}")]
    Query(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Not found")]
    NotFound,
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Event storage trait
#[async_trait]
pub trait EventStorage: Send + Sync {
    /// Insert a single event
    async fn insert(&self, event: AnalyticsEvent) -> Result<(), PersistenceError>;

    /// Insert multiple events
    async fn insert_batch(&self, events: Vec<AnalyticsEvent>) -> Result<(), PersistenceError>;

    /// Query events
    async fn query(&self, query: EventQuery) -> Result<Vec<AnalyticsEvent>, PersistenceError>;

    /// Count events matching query
    async fn count(&self, query: EventQuery) -> Result<u64, PersistenceError>;

    /// Delete events (for retention policies)
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64, PersistenceError>;
}

/// Query parameters for events
#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    /// Filter by event names
    pub events: Vec<String>,
    /// Filter by distinct_id
    pub distinct_id: Option<String>,
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Property filters
    pub property_filters: Vec<PropertyFilter>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Order by
    pub order_by: Option<OrderBy>,
}

#[derive(Debug, Clone)]
pub struct PropertyFilter {
    pub property: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    IsSet,
    IsNotSet,
}

#[derive(Debug, Clone)]
pub struct OrderBy {
    pub field: String,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// ClickHouse storage implementation
pub struct ClickHouseStorage {
    client: clickhouse::Client,
    database: String,
    table: String,
}

impl ClickHouseStorage {
    pub async fn new(url: &str, database: &str) -> Result<Self, PersistenceError> {
        let client = clickhouse::Client::default()
            .with_url(url)
            .with_database(database);

        let storage = Self {
            client,
            database: database.to_string(),
            table: "events".to_string(),
        };

        storage.ensure_schema().await?;

        Ok(storage)
    }

    async fn ensure_schema(&self) -> Result<(), PersistenceError> {
        let create_table = format!(r#"
            CREATE TABLE IF NOT EXISTS {}.{} (
                id UUID,
                event String,
                category LowCardinality(String),
                distinct_id String,
                timestamp DateTime64(3),
                received_at DateTime64(3),
                properties String,
                user_properties Nullable(String),
                session_id Nullable(String),
                environment LowCardinality(String),
                sdk String,
                sdk_version String,
                platform LowCardinality(String),
                date Date DEFAULT toDate(timestamp),

                INDEX idx_event event TYPE bloom_filter GRANULARITY 1,
                INDEX idx_distinct_id distinct_id TYPE bloom_filter GRANULARITY 1
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(date)
            ORDER BY (environment, distinct_id, timestamp, id)
            TTL date + INTERVAL 90 DAY
            SETTINGS index_granularity = 8192
        "#, self.database, self.table);

        self.client.query(&create_table).execute().await
            .map_err(|e| PersistenceError::Query(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
struct ClickHouseEvent {
    id: String,
    event: String,
    category: String,
    distinct_id: String,
    timestamp: i64,
    received_at: i64,
    properties: String,
    user_properties: Option<String>,
    session_id: Option<String>,
    environment: String,
    sdk: String,
    sdk_version: String,
    platform: String,
}

impl From<AnalyticsEvent> for ClickHouseEvent {
    fn from(event: AnalyticsEvent) -> Self {
        Self {
            id: event.id.0.to_string(),
            event: event.event,
            category: format!("{:?}", event.category).to_lowercase(),
            distinct_id: event.distinct_id,
            timestamp: event.timestamp.timestamp_millis(),
            received_at: event.received_at.timestamp_millis(),
            properties: serde_json::to_string(&event.properties).unwrap_or_default(),
            user_properties: event.user_properties.map(|p| serde_json::to_string(&p).unwrap_or_default()),
            session_id: event.session_id,
            environment: event.environment,
            sdk: event.source.sdk,
            sdk_version: event.source.sdk_version,
            platform: format!("{:?}", event.source.platform).to_lowercase(),
        }
    }
}

#[async_trait]
impl EventStorage for ClickHouseStorage {
    async fn insert(&self, event: AnalyticsEvent) -> Result<(), PersistenceError> {
        self.insert_batch(vec![event]).await
    }

    async fn insert_batch(&self, events: Vec<AnalyticsEvent>) -> Result<(), PersistenceError> {
        let ch_events: Vec<ClickHouseEvent> = events.into_iter()
            .map(ClickHouseEvent::from)
            .collect();

        let mut insert = self.client.insert(&self.table)
            .map_err(|e| PersistenceError::Query(e.to_string()))?;

        for event in ch_events {
            insert.write(&event).await
                .map_err(|e| PersistenceError::Query(e.to_string()))?;
        }

        insert.end().await
            .map_err(|e| PersistenceError::Query(e.to_string()))?;

        Ok(())
    }

    async fn query(&self, query: EventQuery) -> Result<Vec<AnalyticsEvent>, PersistenceError> {
        let sql = self.build_query_sql(&query);

        let rows: Vec<ClickHouseEvent> = self.client.query(&sql)
            .fetch_all()
            .await
            .map_err(|e| PersistenceError::Query(e.to_string()))?;

        // Convert back to AnalyticsEvent
        let events = rows.into_iter()
            .filter_map(|row| self.row_to_event(row).ok())
            .collect();

        Ok(events)
    }

    async fn count(&self, query: EventQuery) -> Result<u64, PersistenceError> {
        let mut sql = String::from("SELECT COUNT(*) FROM ");
        sql.push_str(&self.table);
        sql.push_str(&self.build_where_clause(&query));

        let count: u64 = self.client.query(&sql)
            .fetch_one()
            .await
            .map_err(|e| PersistenceError::Query(e.to_string()))?;

        Ok(count)
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64, PersistenceError> {
        let sql = format!(
            "ALTER TABLE {} DELETE WHERE timestamp < toDateTime64({}, 3)",
            self.table,
            timestamp.timestamp_millis()
        );

        self.client.query(&sql).execute().await
            .map_err(|e| PersistenceError::Query(e.to_string()))?;

        Ok(0) // ClickHouse doesn't return count for ALTER DELETE
    }
}

impl ClickHouseStorage {
    fn build_query_sql(&self, query: &EventQuery) -> String {
        let mut sql = String::from("SELECT * FROM ");
        sql.push_str(&self.table);
        sql.push_str(&self.build_where_clause(query));

        if let Some(ref order) = query.order_by {
            sql.push_str(&format!(
                " ORDER BY {} {}",
                order.field,
                match order.direction {
                    OrderDirection::Asc => "ASC",
                    OrderDirection::Desc => "DESC",
                }
            ));
        } else {
            sql.push_str(" ORDER BY timestamp DESC");
        }

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    fn build_where_clause(&self, query: &EventQuery) -> String {
        let mut conditions = Vec::new();

        if !query.events.is_empty() {
            let events_list: Vec<String> = query.events.iter()
                .map(|e| format!("'{}'", e))
                .collect();
            conditions.push(format!("event IN ({})", events_list.join(", ")));
        }

        if let Some(ref distinct_id) = query.distinct_id {
            conditions.push(format!("distinct_id = '{}'", distinct_id));
        }

        if let Some(start) = query.start_time {
            conditions.push(format!("timestamp >= toDateTime64({}, 3)", start.timestamp_millis()));
        }

        if let Some(end) = query.end_time {
            conditions.push(format!("timestamp <= toDateTime64({}, 3)", end.timestamp_millis()));
        }

        if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        }
    }

    fn row_to_event(&self, row: ClickHouseEvent) -> Result<AnalyticsEvent, PersistenceError> {
        use crate::event_types::{EventCategory, EventId, EventSource, Platform};
        use chrono::TimeZone;

        let properties: HashMap<String, serde_json::Value> =
            serde_json::from_str(&row.properties)?;

        let user_properties = row.user_properties
            .map(|s| serde_json::from_str(&s))
            .transpose()?;

        Ok(AnalyticsEvent {
            id: EventId(uuid::Uuid::parse_str(&row.id).unwrap_or_default()),
            event: row.event,
            category: match row.category.as_str() {
                "pageview" => EventCategory::Pageview,
                "action" => EventCategory::Action,
                "identify" => EventCategory::Identify,
                "group" => EventCategory::Group,
                "revenue" => EventCategory::Revenue,
                "session" => EventCategory::Session,
                "featureflag" => EventCategory::FeatureFlag,
                "system" => EventCategory::System,
                _ => EventCategory::Custom,
            },
            distinct_id: row.distinct_id,
            timestamp: Utc.timestamp_millis_opt(row.timestamp).unwrap(),
            properties,
            user_properties,
            session_id: row.session_id,
            environment: row.environment,
            source: EventSource {
                sdk: row.sdk,
                sdk_version: row.sdk_version,
                platform: match row.platform.as_str() {
                    "web" => Platform::Web,
                    "ios" => Platform::Ios,
                    "android" => Platform::Android,
                    "server" => Platform::Server,
                    _ => Platform::Unknown,
                },
                library: None,
            },
            received_at: Utc.timestamp_millis_opt(row.received_at).unwrap(),
        })
    }
}

/// PostgreSQL storage implementation (for smaller deployments)
pub struct PostgresStorage {
    pool: sqlx::PgPool,
}

impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self, PersistenceError> {
        let pool = sqlx::PgPool::connect(database_url).await
            .map_err(|e| PersistenceError::Connection(e.to_string()))?;

        let storage = Self { pool };
        storage.ensure_schema().await?;

        Ok(storage)
    }

    async fn ensure_schema(&self) -> Result<(), PersistenceError> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY,
                event VARCHAR(256) NOT NULL,
                category VARCHAR(50) NOT NULL,
                distinct_id VARCHAR(256) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                received_at TIMESTAMPTZ NOT NULL,
                properties JSONB NOT NULL DEFAULT '{}',
                user_properties JSONB,
                session_id VARCHAR(256),
                environment VARCHAR(50) NOT NULL,
                sdk VARCHAR(100) NOT NULL,
                sdk_version VARCHAR(50) NOT NULL,
                platform VARCHAR(50) NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_events_distinct_id_timestamp
                ON events (distinct_id, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_events_event_timestamp
                ON events (event, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp
                ON events (timestamp DESC);
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::Query(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl EventStorage for PostgresStorage {
    async fn insert(&self, event: AnalyticsEvent) -> Result<(), PersistenceError> {
        sqlx::query(r#"
            INSERT INTO events (
                id, event, category, distinct_id, timestamp, received_at,
                properties, user_properties, session_id, environment,
                sdk, sdk_version, platform
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#)
        .bind(event.id.0)
        .bind(&event.event)
        .bind(format!("{:?}", event.category).to_lowercase())
        .bind(&event.distinct_id)
        .bind(event.timestamp)
        .bind(event.received_at)
        .bind(serde_json::to_value(&event.properties)?)
        .bind(event.user_properties.map(|p| serde_json::to_value(&p)).transpose()?)
        .bind(&event.session_id)
        .bind(&event.environment)
        .bind(&event.source.sdk)
        .bind(&event.source.sdk_version)
        .bind(format!("{:?}", event.source.platform).to_lowercase())
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::Query(e.to_string()))?;

        Ok(())
    }

    async fn insert_batch(&self, events: Vec<AnalyticsEvent>) -> Result<(), PersistenceError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| PersistenceError::Connection(e.to_string()))?;

        for event in events {
            sqlx::query(r#"
                INSERT INTO events (
                    id, event, category, distinct_id, timestamp, received_at,
                    properties, user_properties, session_id, environment,
                    sdk, sdk_version, platform
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#)
            .bind(event.id.0)
            .bind(&event.event)
            .bind(format!("{:?}", event.category).to_lowercase())
            .bind(&event.distinct_id)
            .bind(event.timestamp)
            .bind(event.received_at)
            .bind(serde_json::to_value(&event.properties)?)
            .bind(event.user_properties.map(|p| serde_json::to_value(&p)).transpose()?)
            .bind(&event.session_id)
            .bind(&event.environment)
            .bind(&event.source.sdk)
            .bind(&event.source.sdk_version)
            .bind(format!("{:?}", event.source.platform).to_lowercase())
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::Query(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| PersistenceError::Connection(e.to_string()))?;

        Ok(())
    }

    async fn query(&self, _query: EventQuery) -> Result<Vec<AnalyticsEvent>, PersistenceError> {
        // Implementation similar to ClickHouse
        todo!()
    }

    async fn count(&self, _query: EventQuery) -> Result<u64, PersistenceError> {
        todo!()
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<u64, PersistenceError> {
        let result = sqlx::query("DELETE FROM events WHERE timestamp < $1")
            .bind(timestamp)
            .execute(&self.pool)
            .await
            .map_err(|e| PersistenceError::Query(e.to_string()))?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require actual database connections
}
```

## ClickHouse Schema

```sql
CREATE TABLE events (
    id UUID,
    event String,
    category LowCardinality(String),
    distinct_id String,
    timestamp DateTime64(3),
    received_at DateTime64(3),
    properties String,
    user_properties Nullable(String),
    session_id Nullable(String),
    environment LowCardinality(String),
    sdk String,
    sdk_version String,
    platform LowCardinality(String),
    date Date DEFAULT toDate(timestamp)
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (environment, distinct_id, timestamp, id)
TTL date + INTERVAL 90 DAY;
```

## Related Specs

- 414-event-batching.md - Batch processing
- 416-event-aggregation.md - Aggregation
- 426-data-retention.md - Retention policies
