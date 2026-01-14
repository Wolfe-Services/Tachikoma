// src/database/migration/runner.rs

use super::types::*;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use tracing::{debug, info, warn};

pub struct MigrationRunner {
    pool: SqlitePool,
    migrations: HashMap<i64, Migration>,
}

impl MigrationRunner {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            migrations: HashMap::new(),
        }
    }

    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.insert(migration.version, migration);
    }

    pub fn add_migrations(&mut self, migrations: Vec<Migration>) {
        for migration in migrations {
            self.add_migration(migration);
        }
    }

    /// Initialize the migration tracking table
    pub async fn init(&self) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _tachikoma_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at DATETIME NOT NULL,
                execution_time_ms INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get current database version (highest applied migration)
    pub async fn current_version(&self) -> Result<Option<i64>, MigrationError> {
        self.init().await?;

        let row = sqlx::query("SELECT MAX(version) as version FROM _tachikoma_migrations")
            .fetch_one(&self.pool)
            .await?;

        let version: Option<i64> = row.try_get("version").unwrap_or(None);
        Ok(version)
    }

    /// Get list of applied migrations
    pub async fn get_applied(&self) -> Result<Vec<AppliedMigration>, MigrationError> {
        self.init().await?;

        let applied = sqlx::query_as::<_, AppliedMigration>(
            "SELECT version, name, checksum, applied_at, execution_time_ms 
             FROM _tachikoma_migrations 
             ORDER BY version"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(applied)
    }

    /// Get list of pending migrations
    pub async fn pending(&self) -> Result<Vec<Migration>, MigrationError> {
        let current = self.current_version().await?.unwrap_or(0);
        
        let mut pending: Vec<_> = self.migrations
            .values()
            .filter(|m| m.version > current)
            .cloned()
            .collect();
        
        pending.sort_by_key(|m| m.version);
        Ok(pending)
    }

    /// Create migration plan
    pub async fn plan(&self, target_version: Option<i64>) -> Result<Vec<Migration>, MigrationError> {
        let current = self.current_version().await?.unwrap_or(0);
        
        let target = match target_version {
            Some(v) => v,
            None => {
                // Find highest version
                self.migrations.keys().max().copied().unwrap_or(0)
            }
        };

        let mut plan = Vec::new();

        if target > current {
            // Moving up
            for version in (current + 1)..=target {
                if let Some(migration) = self.migrations.get(&version) {
                    plan.push(migration.clone());
                }
            }
        } else if target < current {
            // Moving down - need to rollback in reverse order
            for version in (target + 1)..=current {
                if let Some(migration) = self.migrations.get(&version) {
                    plan.push(migration.clone());
                }
            }
            plan.reverse();
        }

        Ok(plan)
    }

    /// Run all pending migrations
    pub async fn run(&self) -> Result<Vec<MigrationResult>, MigrationError> {
        let pending = self.pending().await?;
        self.execute_migrations(pending, MigrationDirection::Up).await
    }

    /// Run migrations up to specific version
    pub async fn run_to(&self, target_version: i64) -> Result<Vec<MigrationResult>, MigrationError> {
        let plan = self.plan(Some(target_version)).await?;
        let current = self.current_version().await?.unwrap_or(0);
        
        let direction = if target_version > current {
            MigrationDirection::Up
        } else {
            MigrationDirection::Down
        };
        
        self.execute_migrations(plan, direction).await
    }

    /// Verify migration checksums
    pub async fn verify(&self) -> Result<Vec<String>, MigrationError> {
        let applied = self.get_applied().await?;
        let mut mismatches = Vec::new();

        for applied_migration in applied {
            if let Some(file_migration) = self.migrations.get(&applied_migration.version) {
                if applied_migration.checksum != file_migration.checksum {
                    mismatches.push(format!(
                        "Migration {} checksum mismatch: expected {}, found {}",
                        applied_migration.version,
                        file_migration.checksum,
                        applied_migration.checksum
                    ));
                }
            } else {
                mismatches.push(format!(
                    "Migration {} is applied but no longer exists in files",
                    applied_migration.version
                ));
            }
        }

        Ok(mismatches)
    }

    async fn execute_migrations(
        &self,
        migrations: Vec<Migration>,
        direction: MigrationDirection,
    ) -> Result<Vec<MigrationResult>, MigrationError> {
        if migrations.is_empty() {
            return Ok(Vec::new());
        }

        self.init().await?;
        let mut results = Vec::new();

        for migration in migrations {
            let start = std::time::Instant::now();

            match direction {
                MigrationDirection::Up => {
                    info!("Applying migration: {} - {}", migration.version, migration.name);
                    
                    // Execute migration
                    if let Err(e) = sqlx::query(&migration.up_sql)
                        .execute(&self.pool)
                        .await
                    {
                        return Err(MigrationError::ExecutionFailed(format!(
                            "Failed to apply migration {}: {}",
                            migration.version, e
                        )));
                    }

                    // Record in migrations table
                    sqlx::query(
                        "INSERT INTO _tachikoma_migrations 
                         (version, name, checksum, applied_at, execution_time_ms) 
                         VALUES (?, ?, ?, ?, ?)"
                    )
                    .bind(migration.version)
                    .bind(&migration.name)
                    .bind(&migration.checksum)
                    .bind(Utc::now())
                    .bind(start.elapsed().as_millis() as i64)
                    .execute(&self.pool)
                    .await?;

                    results.push(MigrationResult {
                        version: migration.version,
                        name: migration.name,
                        execution_time_ms: start.elapsed().as_millis() as i64,
                    });
                }
                MigrationDirection::Down => {
                    info!("Rolling back migration: {} - {}", migration.version, migration.name);
                    
                    // Execute rollback if available
                    match &migration.down_sql {
                        Some(down_sql) => {
                            if let Err(e) = sqlx::query(down_sql)
                                .execute(&self.pool)
                                .await
                            {
                                return Err(MigrationError::RollbackFailed(format!(
                                    "Failed to rollback migration {}: {}",
                                    migration.version, e
                                )));
                            }
                        }
                        None => {
                            warn!("No rollback SQL for migration {}, skipping rollback", migration.version);
                        }
                    }

                    // Remove from migrations table
                    sqlx::query("DELETE FROM _tachikoma_migrations WHERE version = ?")
                        .bind(migration.version)
                        .execute(&self.pool)
                        .await?;

                    results.push(MigrationResult {
                        version: migration.version,
                        name: migration.name,
                        execution_time_ms: start.elapsed().as_millis() as i64,
                    });
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub version: i64,
    pub name: String,
    pub execution_time_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_migration_runner_init() {
        let pool = setup_pool().await;
        let runner = MigrationRunner::new(pool);
        
        runner.init().await.unwrap();
        
        // Verify table was created
        let current = runner.current_version().await.unwrap();
        assert_eq!(current, None);
    }

    #[tokio::test]
    async fn test_migration_execution() {
        let pool = setup_pool().await;
        let mut runner = MigrationRunner::new(pool);
        
        let migration = Migration::new(
            20240101000000,
            "test_migration",
            "CREATE TABLE test_table (id INTEGER PRIMARY KEY)"
        );
        
        runner.add_migration(migration);
        
        let results = runner.run().await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].version, 20240101000000);
        
        let current = runner.current_version().await.unwrap();
        assert_eq!(current, Some(20240101000000));
    }
}