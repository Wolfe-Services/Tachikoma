# Spec 364: PostgreSQL Migration Path

## Overview
Define the migration path from SQLite to PostgreSQL for production deployments requiring higher scalability.

## Rust Implementation

### Database Abstraction Layer
```rust
// src/database/abstraction.rs

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Query failed: {0}")]
    Query(String),

    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Migration failed: {0}")]
    Migration(String),

    #[error("Database error: {0}")]
    Other(String),
}

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self {
        DbError::Other(e.to_string())
    }
}

/// Database backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseBackend {
    Sqlite,
    Postgres,
}

/// Connection configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub backend: DatabaseBackend,
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl DbConfig {
    pub fn sqlite(path: &str) -> Self {
        Self {
            backend: DatabaseBackend::Sqlite,
            url: format!("sqlite:{}", path),
            max_connections: 10,
            min_connections: 1,
        }
    }

    pub fn postgres(host: &str, port: u16, database: &str, user: &str, password: &str) -> Self {
        Self {
            backend: DatabaseBackend::Postgres,
            url: format!(
                "postgres://{}:{}@{}:{}/{}",
                user, password, host, port, database
            ),
            max_connections: 20,
            min_connections: 5,
        }
    }
}

/// Abstract database connection
pub enum DbPool {
    Sqlite(sqlx::sqlite::SqlitePool),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::postgres::PgPool),
}

impl DbPool {
    pub async fn connect(config: &DbConfig) -> Result<Self, DbError> {
        match config.backend {
            DatabaseBackend::Sqlite => {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .connect(&config.url)
                    .await?;
                Ok(DbPool::Sqlite(pool))
            }
            #[cfg(feature = "postgres")]
            DatabaseBackend::Postgres => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .connect(&config.url)
                    .await?;
                Ok(DbPool::Postgres(pool))
            }
            #[cfg(not(feature = "postgres"))]
            DatabaseBackend::Postgres => {
                Err(DbError::Other("PostgreSQL support not compiled".to_string()))
            }
        }
    }

    pub fn backend(&self) -> DatabaseBackend {
        match self {
            DbPool::Sqlite(_) => DatabaseBackend::Sqlite,
            #[cfg(feature = "postgres")]
            DbPool::Postgres(_) => DatabaseBackend::Postgres,
        }
    }

    pub async fn close(&self) {
        match self {
            DbPool::Sqlite(pool) => pool.close().await,
            #[cfg(feature = "postgres")]
            DbPool::Postgres(pool) => pool.close().await,
        }
    }
}
```

### SQL Dialect Differences
```rust
// src/database/dialect.rs

use super::abstraction::DatabaseBackend;

/// SQL dialect helper
pub struct SqlDialect {
    backend: DatabaseBackend,
}

impl SqlDialect {
    pub fn new(backend: DatabaseBackend) -> Self {
        Self { backend }
    }

    /// Get current timestamp expression
    pub fn current_timestamp(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "datetime('now')",
            DatabaseBackend::Postgres => "NOW()",
        }
    }

    /// Get UUID generation expression
    pub fn uuid_generate(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "lower(hex(randomblob(16)))",
            DatabaseBackend::Postgres => "gen_random_uuid()",
        }
    }

    /// Get boolean true value
    pub fn bool_true(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "1",
            DatabaseBackend::Postgres => "TRUE",
        }
    }

    /// Get boolean false value
    pub fn bool_false(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "0",
            DatabaseBackend::Postgres => "FALSE",
        }
    }

    /// Get text concatenation operator
    pub fn concat(&self, parts: &[&str]) -> String {
        match self.backend {
            DatabaseBackend::Sqlite => parts.join(" || "),
            DatabaseBackend::Postgres => format!("CONCAT({})", parts.join(", ")),
        }
    }

    /// Get LIKE case-insensitive
    pub fn ilike(&self, column: &str, pattern: &str) -> String {
        match self.backend {
            DatabaseBackend::Sqlite => format!("{} LIKE {} COLLATE NOCASE", column, pattern),
            DatabaseBackend::Postgres => format!("{} ILIKE {}", column, pattern),
        }
    }

    /// Get auto-increment syntax
    pub fn auto_increment(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "INTEGER PRIMARY KEY AUTOINCREMENT",
            DatabaseBackend::Postgres => "SERIAL PRIMARY KEY",
        }
    }

    /// Get text type
    pub fn text_type(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "TEXT",
            DatabaseBackend::Postgres => "TEXT",
        }
    }

    /// Get JSON type
    pub fn json_type(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "TEXT",  // JSON stored as text
            DatabaseBackend::Postgres => "JSONB",
        }
    }

    /// Get timestamp type
    pub fn timestamp_type(&self) -> &'static str {
        match self.backend {
            DatabaseBackend::Sqlite => "TEXT",  // ISO8601 stored as text
            DatabaseBackend::Postgres => "TIMESTAMPTZ",
        }
    }

    /// Upsert syntax (INSERT ... ON CONFLICT)
    pub fn upsert_syntax(&self) -> UpsertSyntax {
        match self.backend {
            DatabaseBackend::Sqlite => UpsertSyntax::SqliteOnConflict,
            DatabaseBackend::Postgres => UpsertSyntax::PostgresOnConflict,
        }
    }
}

pub enum UpsertSyntax {
    SqliteOnConflict,
    PostgresOnConflict,
}
```

### Schema Converter
```rust
// src/database/postgres_migrate.rs

use super::abstraction::{DbPool, DbConfig, DatabaseBackend, DbError};
use tracing::{info, warn, debug};
use std::collections::HashMap;

/// Converts SQLite schema to PostgreSQL
pub struct SchemaConverter {
    type_mappings: HashMap<String, String>,
}

impl SchemaConverter {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Type mappings from SQLite to PostgreSQL
        mappings.insert("INTEGER PRIMARY KEY AUTOINCREMENT".to_string(), "SERIAL PRIMARY KEY".to_string());
        mappings.insert("INTEGER PRIMARY KEY".to_string(), "INTEGER PRIMARY KEY".to_string());
        mappings.insert("INTEGER".to_string(), "INTEGER".to_string());
        mappings.insert("REAL".to_string(), "DOUBLE PRECISION".to_string());
        mappings.insert("TEXT".to_string(), "TEXT".to_string());
        mappings.insert("BLOB".to_string(), "BYTEA".to_string());

        Self { type_mappings: mappings }
    }

    /// Convert SQLite CREATE TABLE to PostgreSQL
    pub fn convert_create_table(&self, sqlite_sql: &str) -> String {
        let mut pg_sql = sqlite_sql.to_string();

        // Replace datetime('now') with NOW()
        pg_sql = pg_sql.replace("datetime('now')", "NOW()");

        // Replace AUTOINCREMENT with SERIAL
        pg_sql = pg_sql.replace("INTEGER PRIMARY KEY AUTOINCREMENT", "SERIAL PRIMARY KEY");

        // Replace boolean integers with booleans
        pg_sql = pg_sql.replace("INTEGER NOT NULL DEFAULT 0", "BOOLEAN NOT NULL DEFAULT FALSE");
        pg_sql = pg_sql.replace("INTEGER NOT NULL DEFAULT 1", "BOOLEAN NOT NULL DEFAULT TRUE");

        // Handle TEXT for JSON
        // Note: This is a simple conversion; complex cases may need manual review

        pg_sql
    }

    /// Convert SQLite index to PostgreSQL
    pub fn convert_index(&self, sqlite_sql: &str) -> String {
        // Most indexes are compatible
        sqlite_sql.to_string()
    }

    /// Convert SQLite trigger (requires manual review)
    pub fn convert_trigger(&self, _sqlite_sql: &str) -> Option<String> {
        warn!("Triggers require manual conversion to PostgreSQL");
        None
    }
}

/// Data migration from SQLite to PostgreSQL
pub struct DataMigrator {
    source: DbPool,
    target: DbPool,
    batch_size: usize,
}

impl DataMigrator {
    pub fn new(source: DbPool, target: DbPool, batch_size: usize) -> Self {
        Self {
            source,
            target,
            batch_size,
        }
    }

    /// Migrate all data from source to target
    pub async fn migrate_all(&self) -> Result<MigrationReport, DbError> {
        let mut report = MigrationReport::new();

        // Get list of tables
        let tables = self.get_source_tables().await?;

        for table in tables {
            info!("Migrating table: {}", table);

            let count = self.migrate_table(&table).await?;
            report.add_table(&table, count);

            info!("Migrated {} rows from {}", count, table);
        }

        Ok(report)
    }

    async fn get_source_tables(&self) -> Result<Vec<String>, DbError> {
        match &self.source {
            DbPool::Sqlite(pool) => {
                let rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_%'"
                )
                .fetch_all(pool)
                .await?;

                Ok(rows.into_iter().map(|r| r.0).collect())
            }
            #[cfg(feature = "postgres")]
            _ => Err(DbError::Other("Source must be SQLite".to_string())),
        }
    }

    async fn migrate_table(&self, table: &str) -> Result<u64, DbError> {
        // This is a simplified example - real implementation would:
        // 1. Read schema from source
        // 2. Transform data types
        // 3. Handle foreign keys (disable, migrate, re-enable)
        // 4. Batch insert into target

        let mut total = 0u64;
        let mut offset = 0i64;

        loop {
            let query = format!(
                "SELECT * FROM {} LIMIT {} OFFSET {}",
                table, self.batch_size, offset
            );

            // Read batch from source
            // Transform and insert into target
            // This would need actual implementation

            offset += self.batch_size as i64;

            // Break when no more rows
            // In real implementation, check row count
            if offset > 0 {
                break;  // Placeholder
            }
        }

        Ok(total)
    }
}

#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub tables: Vec<TableMigrationResult>,
    pub total_rows: u64,
    pub errors: Vec<String>,
}

impl MigrationReport {
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            total_rows: 0,
            errors: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: &str, rows: u64) {
        self.tables.push(TableMigrationResult {
            table: table.to_string(),
            rows_migrated: rows,
        });
        self.total_rows += rows;
    }
}

#[derive(Debug, Clone)]
pub struct TableMigrationResult {
    pub table: String,
    pub rows_migrated: u64,
}

/// PostgreSQL-specific schema additions
pub fn postgres_extensions_sql() -> &'static str {
    r#"
-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For fuzzy search

-- Create enum types for better type safety
DO $$ BEGIN
    CREATE TYPE mission_status AS ENUM ('draft', 'active', 'paused', 'completed', 'archived', 'failed');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE mission_priority AS ENUM ('low', 'medium', 'high', 'critical');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
"#
}
```

### Migration Guide Document
```rust
// This would be documentation, included here for completeness

/// PostgreSQL Migration Guide
///
/// # Prerequisites
/// 1. PostgreSQL 14+ installed
/// 2. Database created with appropriate permissions
/// 3. Backup of SQLite database
///
/// # Steps
/// 1. Export SQLite schema
/// 2. Convert schema using SchemaConverter
/// 3. Review and adjust converted schema
/// 4. Create PostgreSQL schema
/// 5. Migrate data using DataMigrator
/// 6. Verify data integrity
/// 7. Update application configuration
/// 8. Test application with PostgreSQL
///
/// # Known Differences
/// - AUTOINCREMENT -> SERIAL
/// - datetime('now') -> NOW()
/// - INTEGER booleans -> BOOLEAN
/// - TEXT JSON -> JSONB
/// - No VIRTUAL tables (FTS5 -> pg_trgm or tsvector)
///
/// # Performance Considerations
/// - Add appropriate indexes
/// - Configure connection pooling
/// - Set up vacuum/analyze schedules
/// - Consider partitioning for large tables
pub const MIGRATION_GUIDE: &str = include_str!("../../docs/postgres_migration.md");
```

## Files to Create
- `src/database/abstraction.rs` - Database abstraction layer
- `src/database/dialect.rs` - SQL dialect differences
- `src/database/postgres_migrate.rs` - Migration utilities
- `docs/postgres_migration.md` - Migration guide
