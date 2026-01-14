# Spec 362: Database Restore

## Overview
Implement database restore functionality with point-in-time recovery, backup verification, and safe restoration procedures.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Restore Manager
```rust
// src/database/restore.rs

use crate::database::backup::{BackupMetadata, BackupError};
use sqlx::sqlite::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::fs;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::{info, warn, error, instrument};
use flate2::read::GzDecoder;
use std::io::Read;

#[derive(Debug, Error)]
pub enum RestoreError {
    #[error("Restore failed: {0}")]
    Failed(String),

    #[error("Backup not found: {0}")]
    BackupNotFound(String),

    #[error("Backup verification failed")]
    VerificationFailed,

    #[error("Backup is corrupted: {0}")]
    Corrupted(String),

    #[error("Cannot restore to active database")]
    ActiveDatabase,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Restore options
#[derive(Debug, Clone)]
pub struct RestoreOptions {
    /// Verify backup before restore
    pub verify_before_restore: bool,
    /// Create backup of current database before restore
    pub backup_current: bool,
    /// Path for pre-restore backup
    pub pre_restore_backup_path: Option<PathBuf>,
    /// Restore WAL file if present
    pub restore_wal: bool,
    /// Run integrity check after restore
    pub verify_after_restore: bool,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            verify_before_restore: true,
            backup_current: true,
            pre_restore_backup_path: None,
            restore_wal: true,
            verify_after_restore: true,
        }
    }
}

/// Restore result
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub backup_id: String,
    pub restored_at: DateTime<Utc>,
    pub target_path: PathBuf,
    pub pre_restore_backup: Option<PathBuf>,
    pub verification_passed: bool,
    pub restored_size_bytes: u64,
}

/// Restore manager
pub struct RestoreManager {
    backup_dir: PathBuf,
}

impl RestoreManager {
    pub fn new(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }

    /// Restore from a specific backup
    #[instrument(skip(self, options))]
    pub async fn restore(
        &self,
        backup_id: &str,
        target_path: &Path,
        options: RestoreOptions,
    ) -> Result<RestoreResult, RestoreError> {
        // Find backup metadata
        let metadata = self.find_backup(backup_id).await?;

        info!("Starting restore from backup: {}", backup_id);

        // Verify backup if requested
        if options.verify_before_restore {
            info!("Verifying backup integrity...");
            if !self.verify_backup(&metadata).await? {
                return Err(RestoreError::VerificationFailed);
            }
        }

        // Backup current database if requested
        let pre_restore_backup = if options.backup_current && target_path.exists() {
            let backup_path = options.pre_restore_backup_path
                .clone()
                .unwrap_or_else(|| {
                    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                    target_path.with_extension(format!("pre_restore_{}.db", timestamp))
                });

            info!("Creating pre-restore backup at: {}", backup_path.display());
            fs::copy(target_path, &backup_path).await?;

            // Also backup WAL if exists
            let wal_path = format!("{}-wal", target_path.display());
            if Path::new(&wal_path).exists() {
                let wal_backup = format!("{}-wal", backup_path.display());
                fs::copy(&wal_path, &wal_backup).await.ok();
            }

            Some(backup_path)
        } else {
            None
        };

        // Read and decompress backup
        let backup_path = Path::new(&metadata.backup_path);
        let backup_data = fs::read(backup_path).await?;

        let restored_data = if metadata.compressed {
            self.decompress_data(&backup_data)?
        } else {
            backup_data
        };

        let restored_size = restored_data.len() as u64;

        // Verify checksum
        let checksum = self.calculate_checksum(&restored_data);
        if checksum != metadata.checksum {
            error!("Checksum mismatch during restore");
            return Err(RestoreError::Corrupted("Checksum mismatch".to_string()));
        }

        // Remove existing database and WAL files
        if target_path.exists() {
            fs::remove_file(target_path).await?;
        }
        let wal_path = format!("{}-wal", target_path.display());
        if Path::new(&wal_path).exists() {
            fs::remove_file(&wal_path).await?;
        }
        let shm_path = format!("{}-shm", target_path.display());
        if Path::new(&shm_path).exists() {
            fs::remove_file(&shm_path).await?;
        }

        // Write restored database
        fs::write(target_path, &restored_data).await?;

        // Restore WAL if present and requested
        if options.restore_wal {
            let backup_wal = format!("{}-wal", metadata.backup_path);
            if Path::new(&backup_wal).exists() {
                let target_wal = format!("{}-wal", target_path.display());
                fs::copy(&backup_wal, &target_wal).await?;
            }
        }

        // Verify after restore
        let verification_passed = if options.verify_after_restore {
            self.verify_restored_database(target_path).await?
        } else {
            true
        };

        let result = RestoreResult {
            backup_id: backup_id.to_string(),
            restored_at: Utc::now(),
            target_path: target_path.to_path_buf(),
            pre_restore_backup,
            verification_passed,
            restored_size_bytes: restored_size,
        };

        if verification_passed {
            info!("Restore completed successfully from backup: {}", backup_id);
        } else {
            warn!("Restore completed but verification failed");
        }

        Ok(result)
    }

    /// Restore from latest backup
    pub async fn restore_latest(
        &self,
        target_path: &Path,
        options: RestoreOptions,
    ) -> Result<RestoreResult, RestoreError> {
        let backups = self.list_backups().await?;

        let latest = backups
            .into_iter()
            .max_by_key(|b| b.created_at)
            .ok_or_else(|| RestoreError::BackupNotFound("No backups found".to_string()))?;

        self.restore(&latest.id, target_path, options).await
    }

    /// Restore to a point in time (using backup closest to but not after the time)
    pub async fn restore_point_in_time(
        &self,
        target_time: DateTime<Utc>,
        target_path: &Path,
        options: RestoreOptions,
    ) -> Result<RestoreResult, RestoreError> {
        let backups = self.list_backups().await?;

        let backup = backups
            .into_iter()
            .filter(|b| b.created_at <= target_time)
            .max_by_key(|b| b.created_at)
            .ok_or_else(|| RestoreError::BackupNotFound(
                format!("No backup found before {}", target_time)
            ))?;

        info!("Found backup from {} for point-in-time restore to {}",
              backup.created_at, target_time);

        self.restore(&backup.id, target_path, options).await
    }

    /// Find backup by ID
    async fn find_backup(&self, backup_id: &str) -> Result<BackupMetadata, RestoreError> {
        let backups = self.list_backups().await?;

        backups
            .into_iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| RestoreError::BackupNotFound(backup_id.to_string()))
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>, RestoreError> {
        let metadata_path = self.backup_dir.join("backup_manifest.json");

        if !metadata_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&metadata_path).await?;
        let backups: Vec<BackupMetadata> = serde_json::from_str(&content)
            .map_err(|e| RestoreError::Failed(e.to_string()))?;

        Ok(backups)
    }

    /// Verify backup integrity
    async fn verify_backup(&self, metadata: &BackupMetadata) -> Result<bool, RestoreError> {
        let backup_path = Path::new(&metadata.backup_path);

        if !backup_path.exists() {
            return Err(RestoreError::BackupNotFound(metadata.backup_path.clone()));
        }

        let backup_data = fs::read(backup_path).await?;

        let data = if metadata.compressed {
            self.decompress_data(&backup_data)?
        } else {
            backup_data
        };

        let checksum = self.calculate_checksum(&data);
        Ok(checksum == metadata.checksum)
    }

    /// Verify restored database
    async fn verify_restored_database(&self, db_path: &Path) -> Result<bool, RestoreError> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:{}", db_path.display()))
            .await?;

        let (result,): (String,) = sqlx::query_as("PRAGMA integrity_check")
            .fetch_one(&pool)
            .await?;

        pool.close().await;

        Ok(result == "ok")
    }

    /// Decompress gzip data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, RestoreError> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Calculate SHA256 checksum
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Get backup info without restoring
    pub async fn get_backup_info(&self, backup_id: &str) -> Result<BackupInfo, RestoreError> {
        let metadata = self.find_backup(backup_id).await?;
        let backup_path = Path::new(&metadata.backup_path);

        let exists = backup_path.exists();
        let actual_size = if exists {
            fs::metadata(backup_path).await?.len()
        } else {
            0
        };

        let is_valid = if exists {
            self.verify_backup(&metadata).await.unwrap_or(false)
        } else {
            false
        };

        Ok(BackupInfo {
            metadata,
            exists,
            actual_size,
            is_valid,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub metadata: BackupMetadata,
    pub exists: bool,
    pub actual_size: u64,
    pub is_valid: bool,
}

/// Utility for safe database replacement
pub struct SafeRestore {
    target_path: PathBuf,
    backup_path: Option<PathBuf>,
    completed: bool,
}

impl SafeRestore {
    /// Start a safe restore operation
    pub async fn begin(target_path: PathBuf) -> Result<Self, RestoreError> {
        let backup_path = if target_path.exists() {
            let backup = target_path.with_extension("db.restore_backup");
            fs::rename(&target_path, &backup).await?;
            Some(backup)
        } else {
            None
        };

        Ok(Self {
            target_path,
            backup_path,
            completed: false,
        })
    }

    /// Get the target path for writing the restored database
    pub fn target_path(&self) -> &Path {
        &self.target_path
    }

    /// Commit the restore (delete the backup)
    pub async fn commit(mut self) -> Result<(), RestoreError> {
        if let Some(backup) = &self.backup_path {
            if backup.exists() {
                fs::remove_file(backup).await?;
            }
        }
        self.completed = true;
        Ok(())
    }

    /// Rollback the restore (restore the backup)
    pub async fn rollback(mut self) -> Result<(), RestoreError> {
        if let Some(backup) = &self.backup_path {
            if backup.exists() {
                if self.target_path.exists() {
                    fs::remove_file(&self.target_path).await?;
                }
                fs::rename(backup, &self.target_path).await?;
            }
        }
        self.completed = true;
        Ok(())
    }
}

impl Drop for SafeRestore {
    fn drop(&mut self) {
        if !self.completed {
            // Try to restore backup on unexpected drop
            if let Some(backup) = &self.backup_path {
                if backup.exists() {
                    warn!("SafeRestore dropped without commit/rollback, attempting rollback");
                    let _ = std::fs::rename(backup, &self.target_path);
                }
            }
        }
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
- `src/database/restore.rs` - Restore manager implementation
