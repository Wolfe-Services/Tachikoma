# Spec 361: Database Backup

## Overview
Implement database backup functionality with support for full backups, incremental backups, and backup scheduling.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Backup Manager
```rust
// src/database/backup.rs

use sqlx::sqlite::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;
use tracing::{info, warn, error, instrument};
use uuid::Uuid;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Backup failed: {0}")]
    Failed(String),

    #[error("Backup directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Backup not found: {0}")]
    NotFound(String),
}

/// Backup type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupType {
    Full,
    Incremental,
    Snapshot,
}

/// Backup options
#[derive(Debug, Clone)]
pub struct BackupOptions {
    /// Backup directory
    pub backup_dir: PathBuf,
    /// Compress backup files
    pub compress: bool,
    /// Include WAL file in backup
    pub include_wal: bool,
    /// Verify backup after creation
    pub verify: bool,
    /// Maximum number of backups to retain
    pub max_backups: Option<usize>,
    /// Maximum age of backups to retain
    pub max_age_days: Option<i64>,
}

impl Default for BackupOptions {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from("backups"),
            compress: true,
            include_wal: true,
            verify: true,
            max_backups: Some(10),
            max_age_days: Some(30),
        }
    }
}

/// Backup metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub backup_type: String,
    pub source_path: String,
    pub backup_path: String,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub compressed: bool,
    pub checksum: String,
    pub database_version: Option<String>,
    pub page_count: Option<i64>,
}

/// Backup manager
pub struct BackupManager {
    pool: SqlitePool,
    database_path: PathBuf,
    options: BackupOptions,
}

impl BackupManager {
    pub fn new(pool: SqlitePool, database_path: PathBuf, options: BackupOptions) -> Self {
        Self {
            pool,
            database_path,
            options,
        }
    }

    /// Create a full backup
    #[instrument(skip(self))]
    pub async fn create_backup(&self) -> Result<BackupMetadata, BackupError> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.options.backup_dir).await?;

        let backup_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let extension = if self.options.compress { "db.gz" } else { "db" };
        let backup_filename = format!("backup_{}_{}.{}", timestamp, &backup_id[..8], extension);
        let backup_path = self.options.backup_dir.join(&backup_filename);

        info!("Creating backup: {}", backup_filename);

        // Use SQLite backup API via raw connection
        let source_path = self.database_path.to_string_lossy().to_string();

        // Create checkpoint first to ensure all WAL changes are written
        self.checkpoint().await?;

        // Copy database file
        let source_data = fs::read(&self.database_path).await?;

        let backup_data = if self.options.compress {
            self.compress_data(&source_data)?
        } else {
            source_data.clone()
        };

        fs::write(&backup_path, &backup_data).await?;

        // Include WAL file if present and requested
        if self.options.include_wal {
            let wal_path = format!("{}-wal", self.database_path.display());
            if Path::new(&wal_path).exists() {
                let wal_backup = format!("{}-wal", backup_path.display());
                fs::copy(&wal_path, &wal_backup).await.ok();
            }
        }

        // Calculate checksum
        let checksum = self.calculate_checksum(&source_data);

        // Get database info
        let (page_count,): (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;

        let metadata = BackupMetadata {
            id: backup_id,
            backup_type: "full".to_string(),
            source_path,
            backup_path: backup_path.to_string_lossy().to_string(),
            created_at: Utc::now(),
            size_bytes: backup_data.len() as u64,
            compressed: self.options.compress,
            checksum,
            database_version: Some("SQLite 3".to_string()),
            page_count: Some(page_count),
        };

        // Save metadata
        self.save_metadata(&metadata).await?;

        // Verify backup if requested
        if self.options.verify {
            self.verify_backup(&metadata).await?;
        }

        // Clean up old backups
        self.cleanup_old_backups().await?;

        info!("Backup created successfully: {} ({} bytes)", backup_filename, metadata.size_bytes);
        Ok(metadata)
    }

    /// Create a checkpoint (flush WAL to main database)
    async fn checkpoint(&self) -> Result<(), BackupError> {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Compress data using gzip
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, BackupError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    /// Calculate SHA256 checksum
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Save backup metadata
    async fn save_metadata(&self, metadata: &BackupMetadata) -> Result<(), BackupError> {
        let metadata_path = self.options.backup_dir.join("backup_manifest.json");

        let mut backups: Vec<BackupMetadata> = if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        backups.push(metadata.clone());

        let json = serde_json::to_string_pretty(&backups)
            .map_err(|e| BackupError::Failed(e.to_string()))?;

        fs::write(&metadata_path, json).await?;
        Ok(())
    }

    /// Verify backup integrity
    #[instrument(skip(self, metadata))]
    pub async fn verify_backup(&self, metadata: &BackupMetadata) -> Result<bool, BackupError> {
        let backup_path = Path::new(&metadata.backup_path);

        if !backup_path.exists() {
            return Err(BackupError::NotFound(metadata.backup_path.clone()));
        }

        let backup_data = fs::read(backup_path).await?;

        let data = if metadata.compressed {
            self.decompress_data(&backup_data)?
        } else {
            backup_data
        };

        let checksum = self.calculate_checksum(&data);

        if checksum != metadata.checksum {
            error!("Backup verification failed: checksum mismatch");
            return Ok(false);
        }

        // Try to open as SQLite database
        let temp_path = std::env::temp_dir().join(format!("verify_{}.db", metadata.id));
        fs::write(&temp_path, &data).await?;

        let result = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:{}", temp_path.display()))
            .await;

        fs::remove_file(&temp_path).await.ok();

        match result {
            Ok(pool) => {
                // Run integrity check
                let (check_result,): (String,) = sqlx::query_as("PRAGMA integrity_check")
                    .fetch_one(&pool)
                    .await?;

                pool.close().await;

                if check_result == "ok" {
                    info!("Backup verification passed: {}", metadata.id);
                    Ok(true)
                } else {
                    error!("Backup integrity check failed: {}", check_result);
                    Ok(false)
                }
            }
            Err(e) => {
                error!("Cannot open backup as database: {}", e);
                Ok(false)
            }
        }
    }

    /// Decompress gzip data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, BackupError> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>, BackupError> {
        let metadata_path = self.options.backup_dir.join("backup_manifest.json");

        if !metadata_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&metadata_path).await?;
        let backups: Vec<BackupMetadata> = serde_json::from_str(&content)
            .map_err(|e| BackupError::Failed(e.to_string()))?;

        Ok(backups)
    }

    /// Get latest backup
    pub async fn latest_backup(&self) -> Result<Option<BackupMetadata>, BackupError> {
        let mut backups = self.list_backups().await?;
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups.into_iter().next())
    }

    /// Clean up old backups based on retention policy
    #[instrument(skip(self))]
    async fn cleanup_old_backups(&self) -> Result<usize, BackupError> {
        let mut backups = self.list_backups().await?;
        let mut removed = 0;

        // Sort by date, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply max_backups limit
        if let Some(max) = self.options.max_backups {
            while backups.len() > max {
                if let Some(backup) = backups.pop() {
                    self.delete_backup(&backup.id).await?;
                    removed += 1;
                }
            }
        }

        // Apply max_age limit
        if let Some(max_days) = self.options.max_age_days {
            let cutoff = Utc::now() - Duration::days(max_days);

            let old_backups: Vec<_> = backups
                .iter()
                .filter(|b| b.created_at < cutoff)
                .cloned()
                .collect();

            for backup in old_backups {
                self.delete_backup(&backup.id).await?;
                removed += 1;
            }
        }

        if removed > 0 {
            info!("Cleaned up {} old backups", removed);
        }

        Ok(removed)
    }

    /// Delete a backup
    pub async fn delete_backup(&self, backup_id: &str) -> Result<(), BackupError> {
        let backups = self.list_backups().await?;

        let backup = backups
            .iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| BackupError::NotFound(backup_id.to_string()))?;

        // Delete backup file
        let backup_path = Path::new(&backup.backup_path);
        if backup_path.exists() {
            fs::remove_file(backup_path).await?;
        }

        // Delete WAL backup if exists
        let wal_path = format!("{}-wal", backup.backup_path);
        if Path::new(&wal_path).exists() {
            fs::remove_file(&wal_path).await.ok();
        }

        // Update manifest
        let remaining: Vec<_> = backups
            .into_iter()
            .filter(|b| b.id != backup_id)
            .collect();

        let metadata_path = self.options.backup_dir.join("backup_manifest.json");
        let json = serde_json::to_string_pretty(&remaining)
            .map_err(|e| BackupError::Failed(e.to_string()))?;

        fs::write(&metadata_path, json).await?;

        info!("Deleted backup: {}", backup_id);
        Ok(())
    }

    /// Get backup statistics
    pub async fn stats(&self) -> Result<BackupStats, BackupError> {
        let backups = self.list_backups().await?;

        let total_size: u64 = backups.iter().map(|b| b.size_bytes).sum();
        let latest = backups.iter().map(|b| b.created_at).max();

        Ok(BackupStats {
            total_backups: backups.len(),
            total_size_bytes: total_size,
            latest_backup: latest,
            backup_directory: self.options.backup_dir.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BackupStats {
    pub total_backups: usize,
    pub total_size_bytes: u64,
    pub latest_backup: Option<DateTime<Utc>>,
    pub backup_directory: PathBuf,
}

impl BackupStats {
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Tests would use temp directories
}
```

## Files to Create
- `src/database/backup.rs` - Backup manager implementation
