//! Audit log archival.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Archive metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    /// Archive identifier.
    pub id: String,
    /// Archive creation time.
    pub created_at: DateTime<Utc>,
    /// Period start (earliest event).
    pub period_start: DateTime<Utc>,
    /// Period end (latest event).
    pub period_end: DateTime<Utc>,
    /// Number of events.
    pub event_count: u64,
    /// Original size in bytes.
    pub original_size: u64,
    /// Compressed size in bytes.
    pub compressed_size: u64,
    /// Compression algorithm used.
    pub compression: CompressionType,
    /// Checksum of the archive.
    pub checksum: String,
    /// Archive format version.
    pub format_version: u32,
    /// Index included.
    pub has_index: bool,
}

/// Compression algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Lz4,
}

impl CompressionType {
    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Gzip => ".gz",
            Self::Zstd => ".zst",
            Self::Lz4 => ".lz4",
        }
    }
}

/// Archive index entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveIndexEntry {
    /// Event ID.
    pub event_id: String,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
    /// Event category.
    pub category: String,
    /// Event action.
    pub action: String,
    /// Byte offset in archive.
    pub offset: u64,
    /// Event length in bytes.
    pub length: u32,
}

/// Archive index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveIndex {
    /// Archive ID this index belongs to.
    pub archive_id: String,
    /// Index entries.
    pub entries: Vec<ArchiveIndexEntry>,
    /// Index creation time.
    pub created_at: DateTime<Utc>,
}

impl ArchiveIndex {
    /// Search index by time range.
    pub fn search_by_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&ArchiveIndexEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Search index by category.
    pub fn search_by_category(&self, category: &str) -> Vec<&ArchiveIndexEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }
}

/// Archive storage location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArchiveLocation {
    /// Local file system.
    Local { path: PathBuf },
    /// S3-compatible storage.
    S3 {
        bucket: String,
        key: String,
        region: Option<String>,
    },
    /// Azure Blob Storage.
    AzureBlob {
        container: String,
        blob: String,
    },
    /// Google Cloud Storage.
    Gcs {
        bucket: String,
        object: String,
    },
}