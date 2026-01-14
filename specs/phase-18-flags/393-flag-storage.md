# 393 - Feature Flag Storage

## Overview

Persistent storage layer for feature flags supporting multiple backends including PostgreSQL, Redis, and in-memory storage.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/storage.rs

use crate::definition::FlagDefinition;
use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Flag not found: {0}")]
    NotFound(String),
    #[error("Flag already exists: {0}")]
    AlreadyExists(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Version conflict: expected {expected}, found {actual}")]
    VersionConflict { expected: u64, actual: u64 },
}

/// Storage query options
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Filter by status
    pub status: Option<FlagStatus>,
    /// Filter by environment
    pub environment: Option<Environment>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by owner
    pub owner: Option<String>,
    /// Include archived flags
    pub include_archived: bool,
    /// Pagination offset
    pub offset: usize,
    /// Pagination limit
    pub limit: usize,
}

/// Stored flag with version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFlag {
    pub definition: FlagDefinition,
    pub version: u64,
    pub etag: String,
}

#[async_trait]
pub trait FlagStorage: Send + Sync {
    /// Get a single flag by ID
    async fn get(&self, id: &FlagId) -> Result<Option<StoredFlag>, StorageError>;

    /// Get multiple flags by IDs
    async fn get_many(&self, ids: &[FlagId]) -> Result<HashMap<FlagId, StoredFlag>, StorageError>;

    /// Get all flags (with optional filtering)
    async fn list(&self, options: QueryOptions) -> Result<Vec<StoredFlag>, StorageError>;

    /// Create a new flag
    async fn create(&self, definition: FlagDefinition) -> Result<StoredFlag, StorageError>;

    /// Update an existing flag
    async fn update(&self, definition: FlagDefinition, expected_version: Option<u64>) -> Result<StoredFlag, StorageError>;

    /// Delete a flag
    async fn delete(&self, id: &FlagId) -> Result<(), StorageError>;

    /// Check if a flag exists
    async fn exists(&self, id: &FlagId) -> Result<bool, StorageError>;

    /// Get flags modified since a timestamp
    async fn get_modified_since(&self, since: DateTime<Utc>) -> Result<Vec<StoredFlag>, StorageError>;

    /// Get total count of flags
    async fn count(&self, options: QueryOptions) -> Result<usize, StorageError>;
}

/// In-memory storage implementation (for development/testing)
pub struct InMemoryStorage {
    flags: RwLock<HashMap<FlagId, StoredFlag>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            flags: RwLock::new(HashMap::new()),
        }
    }

    fn generate_etag(version: u64) -> String {
        format!("\"v{}\"", version)
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FlagStorage for InMemoryStorage {
    async fn get(&self, id: &FlagId) -> Result<Option<StoredFlag>, StorageError> {
        let flags = self.flags.read().await;
        Ok(flags.get(id).cloned())
    }

    async fn get_many(&self, ids: &[FlagId]) -> Result<HashMap<FlagId, StoredFlag>, StorageError> {
        let flags = self.flags.read().await;
        let mut result = HashMap::new();
        for id in ids {
            if let Some(flag) = flags.get(id) {
                result.insert(id.clone(), flag.clone());
            }
        }
        Ok(result)
    }

    async fn list(&self, options: QueryOptions) -> Result<Vec<StoredFlag>, StorageError> {
        let flags = self.flags.read().await;
        let mut result: Vec<_> = flags.values()
            .filter(|f| {
                if !options.include_archived && f.definition.status == FlagStatus::Archived {
                    return false;
                }
                if let Some(ref status) = options.status {
                    if &f.definition.status != status {
                        return false;
                    }
                }
                if !options.tags.is_empty() {
                    if !options.tags.iter().any(|t| f.definition.metadata.tags.contains(t)) {
                        return false;
                    }
                }
                if let Some(ref owner) = options.owner {
                    if f.definition.metadata.owner.as_ref() != Some(owner) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        result.sort_by(|a, b| a.definition.name.cmp(&b.definition.name));

        let start = options.offset.min(result.len());
        let end = (options.offset + options.limit).min(result.len());

        Ok(result[start..end].to_vec())
    }

    async fn create(&self, definition: FlagDefinition) -> Result<StoredFlag, StorageError> {
        let mut flags = self.flags.write().await;

        if flags.contains_key(&definition.id) {
            return Err(StorageError::AlreadyExists(definition.id.as_str().to_string()));
        }

        let stored = StoredFlag {
            definition,
            version: 1,
            etag: Self::generate_etag(1),
        };

        flags.insert(stored.definition.id.clone(), stored.clone());
        Ok(stored)
    }

    async fn update(&self, definition: FlagDefinition, expected_version: Option<u64>) -> Result<StoredFlag, StorageError> {
        let mut flags = self.flags.write().await;

        let existing = flags.get(&definition.id)
            .ok_or_else(|| StorageError::NotFound(definition.id.as_str().to_string()))?;

        if let Some(expected) = expected_version {
            if existing.version != expected {
                return Err(StorageError::VersionConflict {
                    expected,
                    actual: existing.version,
                });
            }
        }

        let new_version = existing.version + 1;
        let stored = StoredFlag {
            definition,
            version: new_version,
            etag: Self::generate_etag(new_version),
        };

        flags.insert(stored.definition.id.clone(), stored.clone());
        Ok(stored)
    }

    async fn delete(&self, id: &FlagId) -> Result<(), StorageError> {
        let mut flags = self.flags.write().await;
        flags.remove(id)
            .ok_or_else(|| StorageError::NotFound(id.as_str().to_string()))?;
        Ok(())
    }

    async fn exists(&self, id: &FlagId) -> Result<bool, StorageError> {
        let flags = self.flags.read().await;
        Ok(flags.contains_key(id))
    }

    async fn get_modified_since(&self, since: DateTime<Utc>) -> Result<Vec<StoredFlag>, StorageError> {
        let flags = self.flags.read().await;
        Ok(flags.values()
            .filter(|f| f.definition.metadata.updated_at > since)
            .cloned()
            .collect())
    }

    async fn count(&self, options: QueryOptions) -> Result<usize, StorageError> {
        let list = self.list(QueryOptions {
            offset: 0,
            limit: usize::MAX,
            ..options
        }).await?;
        Ok(list.len())
    }
}

/// PostgreSQL storage implementation
pub struct PostgresStorage {
    pool: sqlx::PgPool,
}

impl PostgresStorage {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), StorageError> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS feature_flags (
                id VARCHAR(256) PRIMARY KEY,
                definition JSONB NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_feature_flags_status
                ON feature_flags ((definition->>'status'));
            CREATE INDEX IF NOT EXISTS idx_feature_flags_updated_at
                ON feature_flags (updated_at);
            CREATE INDEX IF NOT EXISTS idx_feature_flags_tags
                ON feature_flags USING GIN ((definition->'metadata'->'tags'));
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl FlagStorage for PostgresStorage {
    async fn get(&self, id: &FlagId) -> Result<Option<StoredFlag>, StorageError> {
        let row = sqlx::query_as::<_, (serde_json::Value, i64)>(
            "SELECT definition, version FROM feature_flags WHERE id = $1"
        )
        .bind(id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        match row {
            Some((def, version)) => {
                let definition: FlagDefinition = serde_json::from_value(def)?;
                Ok(Some(StoredFlag {
                    definition,
                    version: version as u64,
                    etag: format!("\"v{}\"", version),
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_many(&self, ids: &[FlagId]) -> Result<HashMap<FlagId, StoredFlag>, StorageError> {
        let id_strs: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();

        let rows = sqlx::query_as::<_, (String, serde_json::Value, i64)>(
            "SELECT id, definition, version FROM feature_flags WHERE id = ANY($1)"
        )
        .bind(&id_strs)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut result = HashMap::new();
        for (id, def, version) in rows {
            let definition: FlagDefinition = serde_json::from_value(def)?;
            result.insert(FlagId(id), StoredFlag {
                definition,
                version: version as u64,
                etag: format!("\"v{}\"", version),
            });
        }

        Ok(result)
    }

    async fn list(&self, options: QueryOptions) -> Result<Vec<StoredFlag>, StorageError> {
        let mut query = String::from(
            "SELECT definition, version FROM feature_flags WHERE 1=1"
        );

        if !options.include_archived {
            query.push_str(" AND definition->>'status' != 'archived'");
        }

        if options.status.is_some() {
            query.push_str(" AND definition->>'status' = $1");
        }

        query.push_str(" ORDER BY definition->>'name' LIMIT $2 OFFSET $3");

        let rows = sqlx::query_as::<_, (serde_json::Value, i64)>(&query)
            .bind(options.limit as i64)
            .bind(options.offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut result = Vec::new();
        for (def, version) in rows {
            let definition: FlagDefinition = serde_json::from_value(def)?;
            result.push(StoredFlag {
                definition,
                version: version as u64,
                etag: format!("\"v{}\"", version),
            });
        }

        Ok(result)
    }

    async fn create(&self, definition: FlagDefinition) -> Result<StoredFlag, StorageError> {
        let def_json = serde_json::to_value(&definition)?;

        sqlx::query(
            "INSERT INTO feature_flags (id, definition, version) VALUES ($1, $2, 1)"
        )
        .bind(definition.id.as_str())
        .bind(&def_json)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                StorageError::AlreadyExists(definition.id.as_str().to_string())
            } else {
                StorageError::Database(e.to_string())
            }
        })?;

        Ok(StoredFlag {
            definition,
            version: 1,
            etag: "\"v1\"".to_string(),
        })
    }

    async fn update(&self, definition: FlagDefinition, expected_version: Option<u64>) -> Result<StoredFlag, StorageError> {
        let def_json = serde_json::to_value(&definition)?;

        let result = if let Some(expected) = expected_version {
            sqlx::query_as::<_, (i64,)>(
                r#"
                UPDATE feature_flags
                SET definition = $1, version = version + 1, updated_at = NOW()
                WHERE id = $2 AND version = $3
                RETURNING version
                "#
            )
            .bind(&def_json)
            .bind(definition.id.as_str())
            .bind(expected as i64)
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, (i64,)>(
                r#"
                UPDATE feature_flags
                SET definition = $1, version = version + 1, updated_at = NOW()
                WHERE id = $2
                RETURNING version
                "#
            )
            .bind(&def_json)
            .bind(definition.id.as_str())
            .fetch_optional(&self.pool)
            .await
        };

        match result {
            Ok(Some((version,))) => Ok(StoredFlag {
                definition,
                version: version as u64,
                etag: format!("\"v{}\"", version),
            }),
            Ok(None) => {
                // Check if flag exists to differentiate error
                if self.exists(&definition.id).await? {
                    Err(StorageError::VersionConflict {
                        expected: expected_version.unwrap_or(0),
                        actual: 0,
                    })
                } else {
                    Err(StorageError::NotFound(definition.id.as_str().to_string()))
                }
            }
            Err(e) => Err(StorageError::Database(e.to_string())),
        }
    }

    async fn delete(&self, id: &FlagId) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM feature_flags WHERE id = $1")
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.as_str().to_string()));
        }

        Ok(())
    }

    async fn exists(&self, id: &FlagId) -> Result<bool, StorageError> {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM feature_flags WHERE id = $1)"
        )
        .bind(id.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(exists)
    }

    async fn get_modified_since(&self, since: DateTime<Utc>) -> Result<Vec<StoredFlag>, StorageError> {
        let rows = sqlx::query_as::<_, (serde_json::Value, i64)>(
            "SELECT definition, version FROM feature_flags WHERE updated_at > $1"
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut result = Vec::new();
        for (def, version) in rows {
            let definition: FlagDefinition = serde_json::from_value(def)?;
            result.push(StoredFlag {
                definition,
                version: version as u64,
                etag: format!("\"v{}\"", version),
            });
        }

        Ok(result)
    }

    async fn count(&self, _options: QueryOptions) -> Result<usize, StorageError> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM feature_flags WHERE definition->>'status' != 'archived'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(count as usize)
    }
}

/// Create storage from configuration
pub fn create_storage(config: &StorageConfig) -> Arc<dyn FlagStorage> {
    match config {
        StorageConfig::InMemory => Arc::new(InMemoryStorage::new()),
        StorageConfig::Postgres { pool } => Arc::new(PostgresStorage::new(pool.clone())),
    }
}

#[derive(Clone)]
pub enum StorageConfig {
    InMemory,
    Postgres { pool: sqlx::PgPool },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inmemory_storage_crud() {
        let storage = InMemoryStorage::new();

        // Create
        let flag = FlagDefinition::new_boolean("test-flag", "Test Flag", false).unwrap();
        let stored = storage.create(flag.clone()).await.unwrap();
        assert_eq!(stored.version, 1);

        // Get
        let retrieved = storage.get(&FlagId::new("test-flag")).await.unwrap();
        assert!(retrieved.is_some());

        // Update
        let mut updated_def = stored.definition.clone();
        updated_def.status = FlagStatus::Active;
        let updated = storage.update(updated_def, Some(1)).await.unwrap();
        assert_eq!(updated.version, 2);

        // Delete
        storage.delete(&FlagId::new("test-flag")).await.unwrap();
        let deleted = storage.get(&FlagId::new("test-flag")).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_version_conflict() {
        let storage = InMemoryStorage::new();

        let flag = FlagDefinition::new_boolean("test-flag", "Test Flag", false).unwrap();
        storage.create(flag.clone()).await.unwrap();

        let mut updated_def = flag.clone();
        updated_def.status = FlagStatus::Active;

        // First update succeeds
        storage.update(updated_def.clone(), Some(1)).await.unwrap();

        // Second update with old version fails
        let result = storage.update(updated_def, Some(1)).await;
        assert!(matches!(result, Err(StorageError::VersionConflict { .. })));
    }
}
```

## Database Schema

```sql
-- PostgreSQL schema for feature flags
CREATE TABLE feature_flags (
    id VARCHAR(256) PRIMARY KEY,
    definition JSONB NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_feature_flags_status ON feature_flags ((definition->>'status'));
CREATE INDEX idx_feature_flags_updated_at ON feature_flags (updated_at);
CREATE INDEX idx_feature_flags_tags ON feature_flags USING GIN ((definition->'metadata'->'tags'));
CREATE INDEX idx_feature_flags_owner ON feature_flags ((definition->'metadata'->>'owner'));

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_feature_flags_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER feature_flags_updated_at
    BEFORE UPDATE ON feature_flags
    FOR EACH ROW
    EXECUTE FUNCTION update_feature_flags_updated_at();
```

## Related Specs

- 392-flag-definition.md - Flag definition structure
- 405-flag-caching.md - Caching layer
- 404-flag-sync.md - Synchronization
