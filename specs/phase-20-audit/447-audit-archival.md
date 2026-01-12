# 447 - Audit Archival

**Phase:** 20 - Audit System
**Spec ID:** 447
**Status:** Planned
**Dependencies:** 434-audit-persistence, 436-audit-retention
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement audit log archival for long-term storage, enabling compressed, searchable archives with efficient retrieval.

---

## Acceptance Criteria

- [ ] Compressed archive creation
- [ ] Archive index for searching
- [ ] Archive retrieval/restore
- [ ] Cloud storage support
- [ ] Archive verification

---

## Implementation Details

### 1. Archive Types (src/archive.rs)

```rust
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
```

### 2. Archive Creator (src/archive_creator.rs)

```rust
//! Archive creation.

use crate::archive::*;
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use parking_lot::Mutex;
use rusqlite::Connection;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

/// Archive creation configuration.
#[derive(Debug, Clone)]
pub struct ArchiveConfig {
    /// Compression type.
    pub compression: CompressionType,
    /// Include searchable index.
    pub include_index: bool,
    /// Chunk size for streaming.
    pub chunk_size: usize,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            compression: CompressionType::Gzip,
            include_index: true,
            chunk_size: 1000,
        }
    }
}

/// Archive creator.
pub struct ArchiveCreator {
    conn: Arc<Mutex<Connection>>,
    config: ArchiveConfig,
}

impl ArchiveCreator {
    /// Create a new archive creator.
    pub fn new(conn: Arc<Mutex<Connection>>, config: ArchiveConfig) -> Self {
        Self { conn, config }
    }

    /// Create an archive for a time range.
    pub fn create_archive(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        output_path: &Path,
    ) -> Result<ArchiveMetadata, ArchiveError> {
        let archive_id = uuid::Uuid::new_v4().to_string();
        let conn = self.conn.lock();

        // Query events
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, category, action, severity, actor_type, actor_id,
                    target_type, target_id, outcome, metadata, correlation_id
             FROM audit_events
             WHERE timestamp >= ? AND timestamp < ?
             ORDER BY timestamp"
        )?;

        let archive_path = output_path.join(format!(
            "audit_{}_{}{}.tar{}",
            period_start.format("%Y%m%d"),
            period_end.format("%Y%m%d"),
            if self.config.include_index { "_indexed" } else { "" },
            self.config.compression.extension()
        ));

        let file = File::create(&archive_path)?;
        let mut writer: Box<dyn Write> = match self.config.compression {
            CompressionType::None => Box::new(BufWriter::new(file)),
            CompressionType::Gzip => {
                Box::new(GzEncoder::new(BufWriter::new(file), Compression::default()))
            }
            CompressionType::Zstd => {
                Box::new(zstd::Encoder::new(BufWriter::new(file), 3)?)
            }
            CompressionType::Lz4 => {
                Box::new(lz4::EncoderBuilder::new().build(BufWriter::new(file))?)
            }
        };

        let mut hasher = Sha256::new();
        let mut event_count = 0u64;
        let mut original_size = 0u64;
        let mut index_entries = Vec::new();
        let mut current_offset = 0u64;

        let rows = stmt.query_map(
            rusqlite::params![period_start.to_rfc3339(), period_end.to_rfc3339()],
            |row| {
                Ok(EventRow {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    category: row.get(2)?,
                    action: row.get(3)?,
                    severity: row.get(4)?,
                    actor_type: row.get(5)?,
                    actor_id: row.get(6)?,
                    target_type: row.get(7)?,
                    target_id: row.get(8)?,
                    outcome: row.get(9)?,
                    metadata: row.get(10)?,
                    correlation_id: row.get(11)?,
                })
            },
        )?;

        for row in rows {
            let row = row?;
            let json = serde_json::to_string(&row)?;
            let line = format!("{}\n", json);
            let line_bytes = line.as_bytes();

            writer.write_all(line_bytes)?;
            hasher.update(line_bytes);

            if self.config.include_index {
                index_entries.push(ArchiveIndexEntry {
                    event_id: row.id.clone(),
                    timestamp: DateTime::parse_from_rfc3339(&row.timestamp)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| period_start),
                    category: row.category.clone(),
                    action: row.action.clone(),
                    offset: current_offset,
                    length: line_bytes.len() as u32,
                });
            }

            current_offset += line_bytes.len() as u64;
            original_size += line_bytes.len() as u64;
            event_count += 1;
        }

        writer.flush()?;
        drop(writer);

        let compressed_size = std::fs::metadata(&archive_path)?.len();
        let checksum = format!("{:x}", hasher.finalize());

        // Save index if configured
        if self.config.include_index && !index_entries.is_empty() {
            let index = ArchiveIndex {
                archive_id: archive_id.clone(),
                entries: index_entries,
                created_at: Utc::now(),
            };

            let index_path = archive_path.with_extension("index.json");
            let index_file = File::create(&index_path)?;
            serde_json::to_writer(BufWriter::new(index_file), &index)?;
        }

        Ok(ArchiveMetadata {
            id: archive_id,
            created_at: Utc::now(),
            period_start,
            period_end,
            event_count,
            original_size,
            compressed_size,
            compression: self.config.compression,
            checksum,
            format_version: 1,
            has_index: self.config.include_index,
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EventRow {
    id: String,
    timestamp: String,
    category: String,
    action: String,
    severity: String,
    actor_type: String,
    actor_id: Option<String>,
    target_type: Option<String>,
    target_id: Option<String>,
    outcome: String,
    metadata: Option<String>,
    correlation_id: Option<String>,
}

/// Archive creation error.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

### 3. Archive Retriever (src/archive_retriever.rs)

```rust
//! Archive retrieval and restore.

use crate::archive::*;
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Archive retriever.
pub struct ArchiveRetriever;

impl ArchiveRetriever {
    /// Load archive index.
    pub fn load_index(index_path: &Path) -> Result<ArchiveIndex, RetrievalError> {
        let file = File::open(index_path)?;
        let index = serde_json::from_reader(BufReader::new(file))?;
        Ok(index)
    }

    /// Read events from archive.
    pub fn read_events(
        archive_path: &Path,
        compression: CompressionType,
    ) -> Result<impl Iterator<Item = Result<serde_json::Value, RetrievalError>>, RetrievalError> {
        let file = File::open(archive_path)?;

        let reader: Box<dyn Read> = match compression {
            CompressionType::None => Box::new(file),
            CompressionType::Gzip => Box::new(GzDecoder::new(file)),
            CompressionType::Zstd => Box::new(zstd::Decoder::new(file)?),
            CompressionType::Lz4 => Box::new(lz4::Decoder::new(file)?),
        };

        let buf_reader = BufReader::new(reader);

        Ok(buf_reader.lines().map(|line| {
            let line = line.map_err(RetrievalError::Io)?;
            serde_json::from_str(&line).map_err(RetrievalError::Serialization)
        }))
    }

    /// Read specific events by index entries.
    pub fn read_events_by_index(
        archive_path: &Path,
        compression: CompressionType,
        entries: &[&ArchiveIndexEntry],
    ) -> Result<Vec<serde_json::Value>, RetrievalError> {
        // For compressed archives, we need to decompress sequentially
        // This is a simplified implementation
        let events: Vec<_> = Self::read_events(archive_path, compression)?
            .filter_map(|r| r.ok())
            .collect();

        let entry_ids: std::collections::HashSet<_> = entries.iter().map(|e| &e.event_id).collect();

        Ok(events
            .into_iter()
            .filter(|e| {
                e.get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| entry_ids.contains(&id.to_string()))
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Verify archive integrity.
    pub fn verify(archive_path: &Path, expected_checksum: &str) -> Result<bool, RetrievalError> {
        use sha2::{Sha256, Digest};

        let file = File::open(archive_path)?;
        let mut hasher = Sha256::new();
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let computed = format!("{:x}", hasher.finalize());
        Ok(computed == expected_checksum)
    }

    /// Restore events from archive to database.
    pub fn restore_to_db(
        archive_path: &Path,
        compression: CompressionType,
        conn: &rusqlite::Connection,
    ) -> Result<u64, RetrievalError> {
        let mut count = 0u64;

        for event_result in Self::read_events(archive_path, compression)? {
            let event = event_result?;

            // Insert into database (simplified)
            conn.execute(
                "INSERT OR IGNORE INTO audit_events (id, timestamp, category, action, severity)
                 VALUES (?, ?, ?, ?, ?)",
                rusqlite::params![
                    event["id"].as_str(),
                    event["timestamp"].as_str(),
                    event["category"].as_str(),
                    event["action"].as_str(),
                    event["severity"].as_str(),
                ],
            )?;

            count += 1;
        }

        Ok(count)
    }
}

/// Archive retrieval error.
#[derive(Debug, thiserror::Error)]
pub enum RetrievalError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}
```

---

## Testing Requirements

1. Archives compress correctly
2. Index enables efficient lookup
3. Decompression works for all formats
4. Checksum verification is accurate
5. Restore to database works

---

## Related Specs

- Depends on: [434-audit-persistence.md](434-audit-persistence.md), [436-audit-retention.md](436-audit-retention.md)
- Next: [448-audit-gdpr.md](448-audit-gdpr.md)
