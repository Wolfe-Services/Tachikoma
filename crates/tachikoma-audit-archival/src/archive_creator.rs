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