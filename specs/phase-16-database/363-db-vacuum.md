# Spec 363: Database Vacuum and Maintenance

## Overview
Implement database maintenance operations including vacuum, analyze, integrity checks, and optimization.

## Rust Implementation

### Maintenance Manager
```rust
// src/database/maintenance.rs

use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;
use tracing::{info, warn, debug, instrument};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Error)]
pub enum MaintenanceError {
    #[error("Maintenance operation failed: {0}")]
    Failed(String),

    #[error("Database is locked")]
    DatabaseLocked,

    #[error("Integrity check failed: {0}")]
    IntegrityFailed(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Maintenance operation result
#[derive(Debug, Clone)]
pub struct MaintenanceResult {
    pub operation: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub success: bool,
    pub details: Option<String>,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub page_size: i64,
    pub page_count: i64,
    pub freelist_count: i64,
    pub total_bytes: i64,
    pub used_bytes: i64,
    pub free_bytes: i64,
    pub fragmentation_ratio: f64,
    pub table_count: i64,
    pub index_count: i64,
    pub trigger_count: i64,
    pub wal_size_bytes: Option<u64>,
}

/// Table statistics
#[derive(Debug, Clone)]
pub struct TableStats {
    pub name: String,
    pub row_count: i64,
    pub page_count: Option<i64>,
    pub index_count: i64,
}

/// Maintenance manager
pub struct MaintenanceManager {
    pool: SqlitePool,
    database_path: String,
}

impl MaintenanceManager {
    pub fn new(pool: SqlitePool, database_path: impl Into<String>) -> Self {
        Self {
            pool,
            database_path: database_path.into(),
        }
    }

    /// Run full vacuum to reclaim space and defragment
    #[instrument(skip(self))]
    pub async fn vacuum(&self) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();
        info!("Starting VACUUM operation");

        // Get size before
        let before_stats = self.get_stats().await?;

        // Run vacuum
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await?;

        // Get size after
        let after_stats = self.get_stats().await?;

        let completed = Utc::now();
        let space_reclaimed = before_stats.total_bytes - after_stats.total_bytes;

        let details = format!(
            "Space reclaimed: {} bytes (before: {}, after: {})",
            space_reclaimed,
            before_stats.total_bytes,
            after_stats.total_bytes
        );

        info!("VACUUM completed. {}", details);

        Ok(MaintenanceResult {
            operation: "vacuum".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success: true,
            details: Some(details),
        })
    }

    /// Run incremental vacuum (requires auto_vacuum=incremental)
    #[instrument(skip(self))]
    pub async fn incremental_vacuum(&self, pages: i64) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();
        info!("Starting incremental vacuum of {} pages", pages);

        sqlx::query(&format!("PRAGMA incremental_vacuum({})", pages))
            .execute(&self.pool)
            .await?;

        let completed = Utc::now();

        Ok(MaintenanceResult {
            operation: "incremental_vacuum".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success: true,
            details: Some(format!("Vacuumed {} pages", pages)),
        })
    }

    /// Run ANALYZE to update query planner statistics
    #[instrument(skip(self))]
    pub async fn analyze(&self, table: Option<&str>) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();
        let operation = match table {
            Some(t) => {
                info!("Running ANALYZE on table: {}", t);
                format!("ANALYZE {}", t)
            }
            None => {
                info!("Running ANALYZE on entire database");
                "ANALYZE".to_string()
            }
        };

        sqlx::query(&operation)
            .execute(&self.pool)
            .await?;

        let completed = Utc::now();

        Ok(MaintenanceResult {
            operation: "analyze".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success: true,
            details: table.map(|t| format!("Analyzed table: {}", t)),
        })
    }

    /// Run integrity check
    #[instrument(skip(self))]
    pub async fn integrity_check(&self, max_errors: Option<i32>) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();
        info!("Running integrity check");

        let query = match max_errors {
            Some(n) => format!("PRAGMA integrity_check({})", n),
            None => "PRAGMA integrity_check".to_string(),
        };

        let rows: Vec<(String,)> = sqlx::query_as(&query)
            .fetch_all(&self.pool)
            .await?;

        let completed = Utc::now();
        let results: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();

        let success = results.len() == 1 && results[0] == "ok";
        let details = if success {
            "Database integrity OK".to_string()
        } else {
            format!("Integrity issues found: {:?}", results)
        };

        if !success {
            warn!("Integrity check failed: {}", details);
        } else {
            info!("Integrity check passed");
        }

        Ok(MaintenanceResult {
            operation: "integrity_check".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success,
            details: Some(details),
        })
    }

    /// Run quick check (faster than full integrity check)
    pub async fn quick_check(&self) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();

        let (result,): (String,) = sqlx::query_as("PRAGMA quick_check")
            .fetch_one(&self.pool)
            .await?;

        let completed = Utc::now();
        let success = result == "ok";

        Ok(MaintenanceResult {
            operation: "quick_check".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success,
            details: Some(result),
        })
    }

    /// Run foreign key check
    pub async fn foreign_key_check(&self) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();

        let rows: Vec<(String, i64, String, i64)> = sqlx::query_as(
            "PRAGMA foreign_key_check"
        )
        .fetch_all(&self.pool)
        .await?;

        let completed = Utc::now();
        let success = rows.is_empty();

        let details = if success {
            "No foreign key violations".to_string()
        } else {
            format!("Found {} foreign key violations", rows.len())
        };

        Ok(MaintenanceResult {
            operation: "foreign_key_check".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success,
            details: Some(details),
        })
    }

    /// Optimize database (vacuum + analyze + reindex)
    #[instrument(skip(self))]
    pub async fn optimize(&self) -> Result<Vec<MaintenanceResult>, MaintenanceError> {
        info!("Starting full database optimization");
        let mut results = Vec::new();

        // 1. Analyze first
        results.push(self.analyze(None).await?);

        // 2. Reindex
        results.push(self.reindex(None).await?);

        // 3. Vacuum
        results.push(self.vacuum().await?);

        // 4. WAL checkpoint
        results.push(self.checkpoint().await?);

        info!("Database optimization completed");
        Ok(results)
    }

    /// Reindex tables
    pub async fn reindex(&self, table: Option<&str>) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();

        let query = match table {
            Some(t) => format!("REINDEX {}", t),
            None => "REINDEX".to_string(),
        };

        sqlx::query(&query)
            .execute(&self.pool)
            .await?;

        let completed = Utc::now();

        Ok(MaintenanceResult {
            operation: "reindex".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success: true,
            details: table.map(|t| format!("Reindexed: {}", t)),
        })
    }

    /// Checkpoint WAL to main database
    pub async fn checkpoint(&self) -> Result<MaintenanceResult, MaintenanceError> {
        let start = Utc::now();

        let (busy, log, checkpointed): (i32, i32, i32) = sqlx::query_as(
            "PRAGMA wal_checkpoint(TRUNCATE)"
        )
        .fetch_one(&self.pool)
        .await?;

        let completed = Utc::now();
        let success = busy == 0;

        let details = format!(
            "Log frames: {}, Checkpointed: {}, Busy: {}",
            log, checkpointed, busy
        );

        Ok(MaintenanceResult {
            operation: "checkpoint".to_string(),
            started_at: start,
            completed_at: completed,
            duration_ms: (completed - start).num_milliseconds(),
            success,
            details: Some(details),
        })
    }

    /// Get database statistics
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> Result<DatabaseStats, MaintenanceError> {
        let (page_size,): (i64,) = sqlx::query_as("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await?;

        let (page_count,): (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;

        let (freelist_count,): (i64,) = sqlx::query_as("PRAGMA freelist_count")
            .fetch_one(&self.pool)
            .await?;

        let total_bytes = page_size * page_count;
        let free_bytes = page_size * freelist_count;
        let used_bytes = total_bytes - free_bytes;

        let (table_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
        .fetch_one(&self.pool)
        .await?;

        let (index_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index'"
        )
        .fetch_one(&self.pool)
        .await?;

        let (trigger_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger'"
        )
        .fetch_one(&self.pool)
        .await?;

        // Get WAL size if exists
        let wal_path = format!("{}-wal", self.database_path);
        let wal_size = if Path::new(&wal_path).exists() {
            fs::metadata(&wal_path).await.ok().map(|m| m.len())
        } else {
            None
        };

        Ok(DatabaseStats {
            page_size,
            page_count,
            freelist_count,
            total_bytes,
            used_bytes,
            free_bytes,
            fragmentation_ratio: if total_bytes > 0 {
                free_bytes as f64 / total_bytes as f64
            } else {
                0.0
            },
            table_count,
            index_count,
            trigger_count,
            wal_size_bytes: wal_size,
        })
    }

    /// Get per-table statistics
    pub async fn get_table_stats(&self) -> Result<Vec<TableStats>, MaintenanceError> {
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();

        for (table_name,) in tables {
            // Get row count
            let (row_count,): (i64,) = sqlx::query_as(
                &format!("SELECT COUNT(*) FROM {}", table_name)
            )
            .fetch_one(&self.pool)
            .await?;

            // Get index count
            let (index_count,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name=?"
            )
            .bind(&table_name)
            .fetch_one(&self.pool)
            .await?;

            stats.push(TableStats {
                name: table_name,
                row_count,
                page_count: None,  // Would need dbstat virtual table
                index_count,
            });
        }

        Ok(stats)
    }

    /// Check if maintenance is recommended
    pub async fn needs_maintenance(&self) -> Result<MaintenanceRecommendation, MaintenanceError> {
        let stats = self.get_stats().await?;

        let mut recommendations = Vec::new();

        // Check fragmentation
        if stats.fragmentation_ratio > 0.25 {
            recommendations.push("High fragmentation - vacuum recommended".to_string());
        }

        // Check freelist
        if stats.freelist_count > 1000 {
            recommendations.push("Large freelist - vacuum recommended".to_string());
        }

        // Check WAL size
        if let Some(wal_size) = stats.wal_size_bytes {
            if wal_size > 100 * 1024 * 1024 {  // 100MB
                recommendations.push("Large WAL file - checkpoint recommended".to_string());
            }
        }

        Ok(MaintenanceRecommendation {
            needs_maintenance: !recommendations.is_empty(),
            recommendations,
            stats,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MaintenanceRecommendation {
    pub needs_maintenance: bool,
    pub recommendations: Vec<String>,
    pub stats: DatabaseStats,
}

impl DatabaseStats {
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn free_mb(&self) -> f64 {
        self.free_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    // Tests
}
```

## Files to Create
- `src/database/maintenance.rs` - Maintenance manager
