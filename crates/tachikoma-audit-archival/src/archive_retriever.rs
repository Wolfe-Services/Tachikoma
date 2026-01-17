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