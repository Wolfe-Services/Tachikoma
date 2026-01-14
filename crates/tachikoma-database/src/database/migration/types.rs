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