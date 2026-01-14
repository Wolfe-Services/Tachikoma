# Spec 343: Database Migration System

## Overview
Implement a robust database migration system for SQLite with version tracking, rollback support, and migration file management.

## Rust Implementation

### Migration Types
```rust
// src/database/migration/types.rs

use chrono::{DateTime, Utc};
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Migration failed: {0}")]
    ExecutionFailed(String),

    #[error("Migration {0} not found")]
    NotFound(String),

    #[error("Invalid migration format: {0}")]
    InvalidFormat(String),

    #[error("Migration checksum mismatch for {0}")]
    ChecksumMismatch(String),

    #[error("Cannot rollback: {0}")]
    RollbackFailed(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration version (timestamp-based)
    pub version: i64,
    /// Migration name
    pub name: String,
    /// SQL to apply migration
    pub up_sql: String,
    /// SQL to rollback migration
    pub down_sql: Option<String>,
    /// SHA256 checksum of up_sql
    pub checksum: String,
}

impl Migration {
    pub fn new(version: i64, name: impl Into<String>, up_sql: impl Into<String>) -> Self {
        let up = up_sql.into();
        let checksum = Self::compute_checksum(&up);
        Self {
            version,
            name: name.into(),
            up_sql: up,
            down_sql: None,
            checksum,
        }
    }

    pub fn with_down(mut self, down_sql: impl Into<String>) -> Self {
        self.down_sql = Some(down_sql.into());
        self
    }

    pub fn compute_checksum(sql: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(sql.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_checksum(&self) -> bool {
        Self::compute_checksum(&self.up_sql) == self.checksum
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AppliedMigration {
    pub version: i64,
    pub name: String,
    pub checksum: String,
    pub applied_at: DateTime<Utc>,
    pub execution_time_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct MigrationPlan {
    pub direction: MigrationDirection,
    pub migrations: Vec<Migration>,
}

impl MigrationPlan {
    pub fn is_empty(&self) -> bool {
        self.migrations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.migrations.len()
    }
}
```

### Migration Runner
```rust
// src/database/migration/runner.rs

use super::types::*;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::time::Instant;
use tracing::{info, warn, error, instrument};

pub struct MigrationRunner {
    pool: SqlitePool,
    migrations: Vec<Migration>,
}

impl MigrationRunner {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            migrations: Vec::new(),
        }
    }

    /// Register a migration
    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.version);
    }

    /// Register multiple migrations
    pub fn add_migrations(&mut self, migrations: Vec<Migration>) {
        for m in migrations {
            self.add_migration(m);
        }
    }

    /// Initialize migration tracking table
    #[instrument(skip(self))]
    pub async fn init(&self) -> Result<(), MigrationError> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now')),
                execution_time_ms INTEGER NOT NULL DEFAULT 0
            )
        "#)
        .execute(&self.pool)
        .await?;

        info!("Migration tracking table initialized");
        Ok(())
    }

    /// Get all applied migrations
    pub async fn get_applied(&self) -> Result<Vec<AppliedMigration>, MigrationError> {
        let rows = sqlx::query_as::<_, AppliedMigration>(
            "SELECT version, name, checksum, applied_at, execution_time_ms FROM _migrations ORDER BY version"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get current database version
    pub async fn current_version(&self) -> Result<Option<i64>, MigrationError> {
        let row = sqlx::query("SELECT MAX(version) as version FROM _migrations")
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.get::<Option<i64>, _>("version")))
    }

    /// Get pending migrations
    pub async fn pending(&self) -> Result<Vec<&Migration>, MigrationError> {
        let applied = self.get_applied().await?;
        let applied_versions: std::collections::HashSet<_> =
            applied.iter().map(|m| m.version).collect();

        let pending: Vec<_> = self.migrations
            .iter()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect();

        Ok(pending)
    }

    /// Create migration plan to reach target version
    pub async fn plan(&self, target: Option<i64>) -> Result<MigrationPlan, MigrationError> {
        let current = self.current_version().await?;
        let applied = self.get_applied().await?;

        match (current, target) {
            // No migrations applied, apply all up to target
            (None, target) => {
                let migrations: Vec<_> = self.migrations
                    .iter()
                    .filter(|m| target.map_or(true, |t| m.version <= t))
                    .cloned()
                    .collect();

                Ok(MigrationPlan {
                    direction: MigrationDirection::Up,
                    migrations,
                })
            }
            // Rolling forward
            (Some(cur), Some(tgt)) if tgt > cur => {
                let migrations: Vec<_> = self.migrations
                    .iter()
                    .filter(|m| m.version > cur && m.version <= tgt)
                    .cloned()
                    .collect();

                Ok(MigrationPlan {
                    direction: MigrationDirection::Up,
                    migrations,
                })
            }
            // Rolling back
            (Some(cur), Some(tgt)) if tgt < cur => {
                let migrations: Vec<_> = self.migrations
                    .iter()
                    .filter(|m| m.version > tgt && m.version <= cur)
                    .rev()
                    .cloned()
                    .collect();

                Ok(MigrationPlan {
                    direction: MigrationDirection::Down,
                    migrations,
                })
            }
            // Apply all pending (no target specified)
            (Some(_), None) => {
                let applied_versions: std::collections::HashSet<_> =
                    applied.iter().map(|m| m.version).collect();

                let migrations: Vec<_> = self.migrations
                    .iter()
                    .filter(|m| !applied_versions.contains(&m.version))
                    .cloned()
                    .collect();

                Ok(MigrationPlan {
                    direction: MigrationDirection::Up,
                    migrations,
                })
            }
            // Already at target
            _ => Ok(MigrationPlan {
                direction: MigrationDirection::Up,
                migrations: Vec::new(),
            }),
        }
    }

    /// Run all pending migrations
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<Vec<AppliedMigration>, MigrationError> {
        self.init().await?;

        let plan = self.plan(None).await?;
        self.execute_plan(&plan).await
    }

    /// Run migrations up to specific version
    #[instrument(skip(self))]
    pub async fn run_to(&self, version: i64) -> Result<Vec<AppliedMigration>, MigrationError> {
        self.init().await?;

        let plan = self.plan(Some(version)).await?;
        self.execute_plan(&plan).await
    }

    /// Execute a migration plan
    async fn execute_plan(&self, plan: &MigrationPlan) -> Result<Vec<AppliedMigration>, MigrationError> {
        let mut results = Vec::new();

        for migration in &plan.migrations {
            let result = match plan.direction {
                MigrationDirection::Up => self.apply(migration).await?,
                MigrationDirection::Down => self.rollback(migration).await?,
            };
            results.push(result);
        }

        Ok(results)
    }

    /// Apply a single migration
    #[instrument(skip(self, migration), fields(version = migration.version, name = %migration.name))]
    async fn apply(&self, migration: &Migration) -> Result<AppliedMigration, MigrationError> {
        info!("Applying migration {} - {}", migration.version, migration.name);

        let start = Instant::now();

        // Execute migration in transaction
        let mut tx = self.pool.begin().await?;

        // Execute migration SQL (may contain multiple statements)
        for statement in migration.up_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement)
                .execute(&mut *tx)
                .await
                .map_err(|e| MigrationError::ExecutionFailed(format!(
                    "Migration {} failed: {}",
                    migration.version,
                    e
                )))?;
        }

        let execution_time_ms = start.elapsed().as_millis() as i64;

        // Record migration
        sqlx::query(r#"
            INSERT INTO _migrations (version, name, checksum, execution_time_ms)
            VALUES (?, ?, ?, ?)
        "#)
        .bind(migration.version)
        .bind(&migration.name)
        .bind(&migration.checksum)
        .bind(execution_time_ms)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(
            "Migration {} applied in {}ms",
            migration.version,
            execution_time_ms
        );

        Ok(AppliedMigration {
            version: migration.version,
            name: migration.name.clone(),
            checksum: migration.checksum.clone(),
            applied_at: Utc::now(),
            execution_time_ms,
        })
    }

    /// Rollback a single migration
    #[instrument(skip(self, migration), fields(version = migration.version))]
    async fn rollback(&self, migration: &Migration) -> Result<AppliedMigration, MigrationError> {
        let down_sql = migration.down_sql.as_ref().ok_or_else(|| {
            MigrationError::RollbackFailed(format!(
                "Migration {} has no rollback SQL",
                migration.version
            ))
        })?;

        info!("Rolling back migration {} - {}", migration.version, migration.name);

        let start = Instant::now();

        let mut tx = self.pool.begin().await?;

        for statement in down_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement)
                .execute(&mut *tx)
                .await
                .map_err(|e| MigrationError::RollbackFailed(format!(
                    "Rollback {} failed: {}",
                    migration.version,
                    e
                )))?;
        }

        // Remove migration record
        sqlx::query("DELETE FROM _migrations WHERE version = ?")
            .bind(migration.version)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        let execution_time_ms = start.elapsed().as_millis() as i64;
        info!(
            "Migration {} rolled back in {}ms",
            migration.version,
            execution_time_ms
        );

        Ok(AppliedMigration {
            version: migration.version,
            name: migration.name.clone(),
            checksum: migration.checksum.clone(),
            applied_at: Utc::now(),
            execution_time_ms,
        })
    }

    /// Verify all applied migrations match their checksums
    pub async fn verify(&self) -> Result<Vec<String>, MigrationError> {
        let applied = self.get_applied().await?;
        let mut mismatches = Vec::new();

        for applied_migration in applied {
            if let Some(migration) = self.migrations.iter().find(|m| m.version == applied_migration.version) {
                if migration.checksum != applied_migration.checksum {
                    mismatches.push(format!(
                        "Migration {} checksum mismatch: expected {}, got {}",
                        migration.version,
                        migration.checksum,
                        applied_migration.checksum
                    ));
                }
            }
        }

        Ok(mismatches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_migration_runner() {
        let pool = test_pool().await;
        let mut runner = MigrationRunner::new(pool);

        runner.add_migration(Migration::new(
            1,
            "create_users",
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)"
        ));

        runner.add_migration(Migration::new(
            2,
            "add_email",
            "ALTER TABLE users ADD COLUMN email TEXT"
        ));

        let results = runner.run().await.unwrap();
        assert_eq!(results.len(), 2);

        let version = runner.current_version().await.unwrap();
        assert_eq!(version, Some(2));
    }

    #[tokio::test]
    async fn test_pending_migrations() {
        let pool = test_pool().await;
        let mut runner = MigrationRunner::new(pool);

        runner.add_migration(Migration::new(1, "m1", "SELECT 1"));
        runner.add_migration(Migration::new(2, "m2", "SELECT 2"));

        runner.init().await.unwrap();

        let pending = runner.pending().await.unwrap();
        assert_eq!(pending.len(), 2);

        runner.run_to(1).await.unwrap();

        let pending = runner.pending().await.unwrap();
        assert_eq!(pending.len(), 1);
    }
}
```

## Migration File Format

```sql
-- migrations/20240101120000_create_users.sql
-- Migration: create_users
-- Version: 20240101120000

-- UP
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_users_email ON users(email);

-- DOWN
DROP INDEX IF EXISTS idx_users_email;
DROP TABLE IF EXISTS users;
```

## Testing Requirements

- Unit tests for migration types and error handling
- Integration tests for migration runner functionality
- Tests for migration plan generation
- Tests for rollback operations
- Tests for checksum verification
- Tests for concurrent migration handling

## Acceptance Criteria

- [x] Migration types defined (Migration, AppliedMigration, MigrationError, MigrationPlan)
- [x] Migration runner with pool integration
- [x] Version tracking with timestamps
- [x] Checksum validation for migration integrity
- [x] Forward and rollback migration support
- [x] Migration plan generation for target versions
- [x] Transaction safety for migration operations
- [x] Comprehensive error handling and logging
- [x] Migration tracking table initialization
- [x] Pending migration detection
- [x] Unit tests for all components
- [x] Integration tests for runner functionality

## Files to Create
- `src/database/migration/types.rs` - Migration types
- `src/database/migration/runner.rs` - Migration runner
- `src/database/migration/mod.rs` - Module exports
