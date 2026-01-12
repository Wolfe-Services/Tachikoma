# Spec 409: Analytics Local Storage

## Phase
19 - Analytics/Telemetry

## Spec ID
409

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 408: Analytics Collector (event collection)

## Estimated Context
~12%

---

## Objective

Implement persistent local storage for analytics data using SQLite, enabling historical analysis, offline operation, and efficient querying of analytics events.

---

## Acceptance Criteria

- [ ] Implement SQLite-based analytics storage
- [ ] Create efficient schema for event storage
- [ ] Implement event batch insertion
- [ ] Support time-range queries
- [ ] Create indexes for common query patterns
- [ ] Implement storage size management
- [ ] Support data compression
- [ ] Create backup and recovery mechanisms

---

## Implementation Details

### Analytics Storage

```rust
// src/analytics/storage.rs

use crate::analytics::config::StorageConfig;
use crate::analytics::types::{
    AnalyticsEvent, EventBatch, EventCategory, EventData, EventId, EventType,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Analytics storage interface
#[async_trait]
pub trait AnalyticsStorage: Send + Sync {
    /// Store a batch of events
    async fn store_batch(&self, batch: &EventBatch) -> Result<(), StorageError>;

    /// Query events by time range
    async fn query_by_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<AnalyticsEvent>, StorageError>;

    /// Query events by category
    async fn query_by_category(
        &self,
        category: EventCategory,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AnalyticsEvent>, StorageError>;

    /// Query events by type
    async fn query_by_type(
        &self,
        event_type: &EventType,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AnalyticsEvent>, StorageError>;

    /// Get event count by category
    async fn count_by_category(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<(EventCategory, u64)>, StorageError>;

    /// Delete events older than a given date
    async fn delete_before(&self, before: DateTime<Utc>) -> Result<u64, StorageError>;

    /// Get storage statistics
    async fn stats(&self) -> Result<StorageStats, StorageError>;

    /// Compact the database
    async fn compact(&self) -> Result<(), StorageError>;
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_events: u64,
    pub database_size_bytes: u64,
    pub oldest_event: Option<DateTime<Utc>>,
    pub newest_event: Option<DateTime<Utc>>,
    pub events_by_category: Vec<(EventCategory, u64)>,
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Storage full")]
    StorageFull,
}

/// SQLite-based analytics storage
pub struct SqliteAnalyticsStorage {
    conn: Arc<Mutex<Connection>>,
    config: StorageConfig,
    path: PathBuf,
}

impl SqliteAnalyticsStorage {
    /// Create a new SQLite storage
    pub fn new(path: PathBuf, config: StorageConfig) -> Result<Self, StorageError> {
        let conn = Connection::open(&path)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
            path,
        };

        // Initialize schema
        tokio::runtime::Handle::current().block_on(storage.init_schema())?;

        Ok(storage)
    }

    /// Create an in-memory storage (for testing)
    pub fn in_memory(config: StorageConfig) -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
            path: PathBuf::from(":memory:"),
        };

        tokio::runtime::Handle::current().block_on(storage.init_schema())?;

        Ok(storage)
    }

    /// Initialize the database schema
    async fn init_schema(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;

        conn.execute_batch(
            r#"
            -- Main events table
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                event_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                session_id TEXT,
                priority INTEGER NOT NULL,
                data TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_events_timestamp
                ON events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_category
                ON events(category);
            CREATE INDEX IF NOT EXISTS idx_events_type
                ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_session
                ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_category_timestamp
                ON events(category, timestamp);

            -- Batch tracking table
            CREATE TABLE IF NOT EXISTS batches (
                id TEXT PRIMARY KEY,
                sequence INTEGER NOT NULL,
                event_count INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                processed_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Aggregated daily statistics
            CREATE TABLE IF NOT EXISTS daily_stats (
                date TEXT NOT NULL,
                category TEXT NOT NULL,
                event_type TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (date, category, event_type)
            );

            -- Storage metadata
            CREATE TABLE IF NOT EXISTS storage_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        // Set pragmas for performance
        if self.config.wal_mode {
            conn.execute_batch("PRAGMA journal_mode = WAL;")
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        let sync_pragma = match self.config.sync_mode {
            crate::analytics::config::SyncMode::Full => "PRAGMA synchronous = FULL;",
            crate::analytics::config::SyncMode::Normal => "PRAGMA synchronous = NORMAL;",
            crate::analytics::config::SyncMode::Off => "PRAGMA synchronous = OFF;",
        };
        conn.execute_batch(sync_pragma)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Check storage size and enforce limits
    async fn check_storage_limits(&self) -> Result<(), StorageError> {
        let stats = self.stats().await?;
        let max_bytes = self.config.max_size_mb * 1024 * 1024;

        if stats.database_size_bytes > max_bytes {
            // Delete oldest 10% of events
            let delete_count = stats.total_events / 10;
            let conn = self.conn.lock().await;

            conn.execute(
                r#"
                DELETE FROM events
                WHERE id IN (
                    SELECT id FROM events
                    ORDER BY timestamp ASC
                    LIMIT ?
                )
                "#,
                params![delete_count],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Update daily statistics
    async fn update_daily_stats(&self, event: &AnalyticsEvent) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;
        let date = event.timestamp.format("%Y-%m-%d").to_string();
        let category = format!("{:?}", event.category);
        let event_type = format!("{:?}", event.event_type);

        conn.execute(
            r#"
            INSERT INTO daily_stats (date, category, event_type, count)
            VALUES (?, ?, ?, 1)
            ON CONFLICT(date, category, event_type)
            DO UPDATE SET count = count + 1
            "#,
            params![date, category, event_type],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    fn serialize_event(event: &AnalyticsEvent) -> Result<(String, String), StorageError> {
        let data = serde_json::to_string(&event.data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        let metadata = serde_json::to_string(&event.metadata)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        Ok((data, metadata))
    }

    fn deserialize_event(
        id: String,
        category: String,
        event_type: String,
        timestamp: String,
        session_id: Option<String>,
        priority: i32,
        data: String,
        metadata: String,
    ) -> Result<AnalyticsEvent, StorageError> {
        use std::str::FromStr;

        let id = Uuid::parse_str(&id)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let category: EventCategory = serde_json::from_str(&format!("\"{}\"", category.to_lowercase()))
            .unwrap_or(EventCategory::Custom);

        let event_type: EventType = if event_type.starts_with("Custom(") {
            let custom_name = event_type
                .trim_start_matches("Custom(")
                .trim_end_matches(')');
            EventType::Custom(custom_name.to_string())
        } else {
            serde_json::from_str(&format!("\"{}\"", event_type.to_lowercase()))
                .unwrap_or(EventType::Custom(event_type))
        };

        let timestamp = DateTime::parse_from_rfc3339(&timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let session_id = session_id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let priority = match priority {
            0 => crate::analytics::types::EventPriority::Low,
            1 => crate::analytics::types::EventPriority::Normal,
            2 => crate::analytics::types::EventPriority::High,
            _ => crate::analytics::types::EventPriority::Critical,
        };

        let data: EventData = serde_json::from_str(&data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let metadata = serde_json::from_str(&metadata)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        Ok(AnalyticsEvent {
            id: EventId::from_uuid(id),
            category,
            event_type,
            timestamp,
            session_id,
            priority,
            data,
            metadata,
        })
    }
}

#[async_trait]
impl AnalyticsStorage for SqliteAnalyticsStorage {
    async fn store_batch(&self, batch: &EventBatch) -> Result<(), StorageError> {
        self.check_storage_limits().await?;

        let conn = self.conn.lock().await;

        // Begin transaction
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| StorageError::Database(e.to_string()))?;

        // Store batch metadata
        conn.execute(
            "INSERT INTO batches (id, sequence, event_count, created_at) VALUES (?, ?, ?, ?)",
            params![
                batch.id.to_string(),
                batch.sequence as i64,
                batch.events.len() as i64,
                batch.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        // Store events
        let mut stmt = conn
            .prepare(
                r#"
                INSERT INTO events (id, category, event_type, timestamp, session_id, priority, data, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

        for event in &batch.events {
            let (data, metadata) = Self::serialize_event(event)?;

            stmt.execute(params![
                format!("{:?}", event.id),
                format!("{:?}", event.category),
                format!("{:?}", event.event_type),
                event.timestamp.to_rfc3339(),
                event.session_id.map(|id| id.to_string()),
                event.priority as i32,
                data,
                metadata,
            ])
            .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        // Commit transaction
        conn.execute("COMMIT", [])
            .map_err(|e| StorageError::Database(e.to_string()))?;

        // Update daily stats (outside transaction for performance)
        drop(conn);
        for event in &batch.events {
            self.update_daily_stats(event).await?;
        }

        Ok(())
    }

    async fn query_by_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<AnalyticsEvent>, StorageError> {
        let conn = self.conn.lock().await;

        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let query = format!(
            r#"
            SELECT id, category, event_type, timestamp, session_id, priority, data, metadata
            FROM events
            WHERE timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            {}
            "#,
            limit_clause
        );

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![start.to_rfc3339(), end.to_rfc3339()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let (id, category, event_type, timestamp, session_id, priority, data, metadata) =
                row.map_err(|e| StorageError::Database(e.to_string()))?;

            events.push(Self::deserialize_event(
                id, category, event_type, timestamp, session_id, priority, data, metadata,
            )?);
        }

        Ok(events)
    }

    async fn query_by_category(
        &self,
        category: EventCategory,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AnalyticsEvent>, StorageError> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, category, event_type, timestamp, session_id, priority, data, metadata
                FROM events
                WHERE category = ? AND timestamp >= ? AND timestamp <= ?
                ORDER BY timestamp DESC
                "#,
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let category_str = format!("{:?}", category);
        let rows = stmt
            .query_map(
                params![category_str, start.to_rfc3339(), end.to_rfc3339()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, i32>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let (id, category, event_type, timestamp, session_id, priority, data, metadata) =
                row.map_err(|e| StorageError::Database(e.to_string()))?;

            events.push(Self::deserialize_event(
                id, category, event_type, timestamp, session_id, priority, data, metadata,
            )?);
        }

        Ok(events)
    }

    async fn query_by_type(
        &self,
        event_type: &EventType,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AnalyticsEvent>, StorageError> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, category, event_type, timestamp, session_id, priority, data, metadata
                FROM events
                WHERE event_type = ? AND timestamp >= ? AND timestamp <= ?
                ORDER BY timestamp DESC
                "#,
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let type_str = format!("{:?}", event_type);
        let rows = stmt
            .query_map(params![type_str, start.to_rfc3339(), end.to_rfc3339()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let (id, category, event_type, timestamp, session_id, priority, data, metadata) =
                row.map_err(|e| StorageError::Database(e.to_string()))?;

            events.push(Self::deserialize_event(
                id, category, event_type, timestamp, session_id, priority, data, metadata,
            )?);
        }

        Ok(events)
    }

    async fn count_by_category(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<(EventCategory, u64)>, StorageError> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT category, COUNT(*) as count
                FROM events
                WHERE timestamp >= ? AND timestamp <= ?
                GROUP BY category
                "#,
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![start.to_rfc3339(), end.to_rfc3339()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let (category_str, count) = row.map_err(|e| StorageError::Database(e.to_string()))?;

            let category: EventCategory = serde_json::from_str(&format!("\"{}\"", category_str.to_lowercase()))
                .unwrap_or(EventCategory::Custom);

            results.push((category, count as u64));
        }

        Ok(results)
    }

    async fn delete_before(&self, before: DateTime<Utc>) -> Result<u64, StorageError> {
        let conn = self.conn.lock().await;

        let deleted = conn
            .execute(
                "DELETE FROM events WHERE timestamp < ?",
                params![before.to_rfc3339()],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(deleted as u64)
    }

    async fn stats(&self) -> Result<StorageStats, StorageError> {
        let conn = self.conn.lock().await;

        let total_events: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let oldest_event: Option<String> = conn
            .query_row(
                "SELECT MIN(timestamp) FROM events",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| StorageError::Database(e.to_string()))?
            .flatten();

        let newest_event: Option<String> = conn
            .query_row(
                "SELECT MAX(timestamp) FROM events",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| StorageError::Database(e.to_string()))?
            .flatten();

        let database_size_bytes = if self.path.to_str() == Some(":memory:") {
            0
        } else {
            std::fs::metadata(&self.path)
                .map(|m| m.len())
                .unwrap_or(0)
        };

        let oldest = oldest_event.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        let newest = newest_event.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        drop(conn);
        let events_by_category = self
            .count_by_category(
                oldest.unwrap_or_else(Utc::now),
                newest.unwrap_or_else(Utc::now),
            )
            .await?;

        Ok(StorageStats {
            total_events: total_events as u64,
            database_size_bytes,
            oldest_event: oldest,
            newest_event: newest,
            events_by_category,
        })
    }

    async fn compact(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;

        conn.execute_batch("VACUUM;")
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }
}

/// Storage sink adapter for the collector
pub struct StorageSink {
    storage: Arc<dyn AnalyticsStorage>,
}

impl StorageSink {
    pub fn new(storage: Arc<dyn AnalyticsStorage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl crate::analytics::collector::EventSink for StorageSink {
    async fn process(&self, batch: EventBatch) -> Result<(), crate::analytics::collector::SinkError> {
        self.storage
            .store_batch(&batch)
            .await
            .map_err(|e| crate::analytics::collector::SinkError::WriteFailed(e.to_string()))
    }

    async fn flush(&self) -> Result<(), crate::analytics::collector::SinkError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "sqlite_storage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::types::{EventBuilder, EventData, UsageEventData};

    fn create_test_storage() -> SqliteAnalyticsStorage {
        SqliteAnalyticsStorage::in_memory(StorageConfig::default()).unwrap()
    }

    #[tokio::test]
    async fn test_store_and_query() {
        let storage = create_test_storage();

        let event = EventBuilder::new(EventType::MissionCreated)
            .usage_data("mission", "create", true)
            .build();

        let batch = EventBatch::new(vec![event.clone()], 1);
        storage.store_batch(&batch).await.unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);

        let events = storage.query_by_time(start, end, None).await.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_query_by_category() {
        let storage = create_test_storage();

        let usage_event = EventBuilder::new(EventType::FeatureUsed)
            .usage_data("feature", "use", true)
            .build();

        let error_event = EventBuilder::new(EventType::ErrorOccurred)
            .error_data("ERR001", "Test error", crate::analytics::types::ErrorSeverity::Error, "test")
            .build();

        let batch = EventBatch::new(vec![usage_event, error_event], 1);
        storage.store_batch(&batch).await.unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);

        let usage_events = storage
            .query_by_category(EventCategory::Usage, start, end)
            .await
            .unwrap();
        assert_eq!(usage_events.len(), 1);

        let error_events = storage
            .query_by_category(EventCategory::Error, start, end)
            .await
            .unwrap();
        assert_eq!(error_events.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_before() {
        let storage = create_test_storage();

        let event = EventBuilder::new(EventType::SessionStarted).build();
        let batch = EventBatch::new(vec![event], 1);
        storage.store_batch(&batch).await.unwrap();

        // Delete events from before now (should delete nothing since event was just created)
        let deleted = storage
            .delete_before(Utc::now() - chrono::Duration::hours(1))
            .await
            .unwrap();
        assert_eq!(deleted, 0);

        // Delete events from after now (should delete the event)
        let deleted = storage
            .delete_before(Utc::now() + chrono::Duration::hours(1))
            .await
            .unwrap();
        assert_eq!(deleted, 1);
    }

    #[tokio::test]
    async fn test_stats() {
        let storage = create_test_storage();

        let events: Vec<AnalyticsEvent> = (0..10)
            .map(|_| EventBuilder::new(EventType::FeatureUsed).build())
            .collect();

        let batch = EventBatch::new(events, 1);
        storage.store_batch(&batch).await.unwrap();

        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.total_events, 10);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Schema initialization
   - Event storage and retrieval
   - Query by various criteria
   - Storage limit enforcement

2. **Integration Tests**
   - Large batch processing
   - Concurrent access
   - Recovery after crash

3. **Performance Tests**
   - Query performance with indexes
   - Bulk insert performance
   - Storage compaction efficiency

---

## Related Specs

- Spec 406: Analytics Types
- Spec 408: Analytics Collector
- Spec 410: Analytics Aggregation
- Spec 424: Data Retention
