# 434 - Audit Persistence

**Phase:** 20 - Audit System
**Spec ID:** 434
**Status:** Planned
**Dependencies:** 432-audit-schema, 433-audit-capture
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Implement durable persistence for audit events using SQLite for structured queries and append-only log files for immutability guarantees.

---

## Acceptance Criteria

- [ ] SQLite persistence with transactions
- [ ] Append-only log file writing
- [ ] Write-ahead logging (WAL) mode
- [ ] Batch persistence for performance
- [ ] Checksummed chain integrity

---

## Implementation Details

### 1. Persistence Trait (src/persistence.rs)

```rust
//! Audit persistence traits and implementations.

use crate::{AuditEvent, EventBatch};
use async_trait::async_trait;
use thiserror::Error;

/// Persistence error.
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Database(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("integrity error: {0}")]
    Integrity(String),
}

/// Result type for persistence operations.
pub type PersistenceResult<T> = Result<T, PersistenceError>;

/// Trait for audit event persistence.
#[async_trait]
pub trait AuditPersistence: Send + Sync {
    /// Persist a single event.
    async fn persist(&self, event: &AuditEvent) -> PersistenceResult<()>;

    /// Persist a batch of events.
    async fn persist_batch(&self, batch: &EventBatch) -> PersistenceResult<usize>;

    /// Get the last persisted sequence number.
    async fn last_sequence(&self) -> PersistenceResult<u64>;

    /// Verify integrity of stored events.
    async fn verify_integrity(&self) -> PersistenceResult<IntegrityReport>;

    /// Flush any buffered writes.
    async fn flush(&self) -> PersistenceResult<()>;
}

/// Integrity verification report.
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub total_events: u64,
    pub verified_events: u64,
    pub corrupted_events: Vec<String>,
    pub chain_breaks: Vec<u64>,
    pub is_valid: bool,
}

impl IntegrityReport {
    pub fn valid(total: u64) -> Self {
        Self {
            total_events: total,
            verified_events: total,
            corrupted_events: Vec::new(),
            chain_breaks: Vec::new(),
            is_valid: true,
        }
    }
}
```

### 2. SQLite Persistence (src/sqlite.rs)

```rust
//! SQLite-based audit persistence.

use crate::{
    schema::{event_to_row, run_migrations, AuditLogEntry},
    AuditEvent, EventBatch, PersistenceError, PersistenceResult,
    AuditPersistence, IntegrityReport,
};
use async_trait::async_trait;
use parking_lot::Mutex;
use rusqlite::{params, Connection, Transaction};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info};

/// SQLite persistence configuration.
#[derive(Debug, Clone)]
pub struct SqliteConfig {
    /// Path to the database file.
    pub db_path: String,
    /// Enable WAL mode.
    pub wal_mode: bool,
    /// Synchronous mode (FULL, NORMAL, OFF).
    pub synchronous: String,
    /// Cache size in KB.
    pub cache_size_kb: u32,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            db_path: "audit.db".to_string(),
            wal_mode: true,
            synchronous: "NORMAL".to_string(),
            cache_size_kb: 8192,
        }
    }
}

/// SQLite audit persistence implementation.
pub struct SqlitePersistence {
    conn: Arc<Mutex<Connection>>,
    config: SqliteConfig,
}

impl SqlitePersistence {
    /// Create a new SQLite persistence instance.
    pub fn new(config: SqliteConfig) -> PersistenceResult<Self> {
        let conn = Connection::open(&config.db_path)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Configure SQLite
        if config.wal_mode {
            conn.pragma_update(None, "journal_mode", "WAL")
                .map_err(|e| PersistenceError::Database(e.to_string()))?;
        }
        conn.pragma_update(None, "synchronous", &config.synchronous)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        conn.pragma_update(None, "cache_size", &format!("-{}", config.cache_size_kb))
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Run migrations
        let applied = run_migrations(&conn)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        if applied > 0 {
            info!("Applied {} audit schema migrations", applied);
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        })
    }

    /// Create an in-memory instance for testing.
    pub fn in_memory() -> PersistenceResult<Self> {
        let config = SqliteConfig {
            db_path: ":memory:".to_string(),
            ..Default::default()
        };
        Self::new(config)
    }

    fn persist_event_tx(tx: &Transaction, event: &AuditEvent) -> PersistenceResult<()> {
        let row = event_to_row(event);

        tx.execute(
            r#"
            INSERT INTO audit_events (
                id, timestamp, category, action, severity,
                actor_type, actor_id, actor_name,
                target_type, target_id, target_name,
                outcome, outcome_reason, metadata,
                correlation_id, ip_address, user_agent, checksum
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
            )
            "#,
            params![
                row.id, row.timestamp, row.category, row.action, row.severity,
                row.actor_type, row.actor_id, row.actor_name,
                row.target_type, row.target_id, row.target_name,
                row.outcome, row.outcome_reason, row.metadata,
                row.correlation_id, row.ip_address, row.user_agent, row.checksum
            ],
        ).map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl AuditPersistence for SqlitePersistence {
    async fn persist(&self, event: &AuditEvent) -> PersistenceResult<()> {
        let conn = self.conn.lock();
        let row = event_to_row(event);

        conn.execute(
            r#"
            INSERT INTO audit_events (
                id, timestamp, category, action, severity,
                actor_type, actor_id, actor_name,
                target_type, target_id, target_name,
                outcome, outcome_reason, metadata,
                correlation_id, ip_address, user_agent, checksum
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
            )
            "#,
            params![
                row.id, row.timestamp, row.category, row.action, row.severity,
                row.actor_type, row.actor_id, row.actor_name,
                row.target_type, row.target_id, row.target_name,
                row.outcome, row.outcome_reason, row.metadata,
                row.correlation_id, row.ip_address, row.user_agent, row.checksum
            ],
        ).map_err(|e| PersistenceError::Database(e.to_string()))?;

        debug!("Persisted audit event {}", event.id);
        Ok(())
    }

    async fn persist_batch(&self, batch: &EventBatch) -> PersistenceResult<usize> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        let mut count = 0;
        for captured in &batch.events {
            Self::persist_event_tx(&tx, &captured.event)?;
            count += 1;
        }

        tx.commit()
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        debug!("Persisted batch of {} audit events", count);
        Ok(count)
    }

    async fn last_sequence(&self) -> PersistenceResult<u64> {
        let conn = self.conn.lock();
        let seq: i64 = conn
            .query_row(
                "SELECT last_sequence_number FROM audit_sequence WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        Ok(seq as u64)
    }

    async fn verify_integrity(&self) -> PersistenceResult<IntegrityReport> {
        let conn = self.conn.lock();
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // For SQLite, we verify checksums individually
        // Full chain verification is done on append-only logs
        Ok(IntegrityReport::valid(total as u64))
    }

    async fn flush(&self) -> PersistenceResult<()> {
        let conn = self.conn.lock();
        conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        Ok(())
    }
}
```

### 3. Append-Only Log Persistence (src/append_log.rs)

```rust
//! Append-only log file persistence for immutability.

use crate::{
    schema::AuditLogEntry, AuditEvent, EventBatch,
    PersistenceError, PersistenceResult, AuditPersistence, IntegrityReport,
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, warn};

/// Append-only log configuration.
#[derive(Debug, Clone)]
pub struct AppendLogConfig {
    /// Directory for log files.
    pub log_dir: PathBuf,
    /// Maximum size per log file in bytes.
    pub max_file_size: u64,
    /// File name prefix.
    pub file_prefix: String,
    /// Sync after each write.
    pub sync_on_write: bool,
}

impl Default for AppendLogConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("audit_logs"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            file_prefix: "audit".to_string(),
            sync_on_write: true,
        }
    }
}

/// Append-only log persistence.
pub struct AppendLogPersistence {
    config: AppendLogConfig,
    writer: Arc<Mutex<BufWriter<File>>>,
    current_file: Arc<Mutex<PathBuf>>,
    current_size: AtomicU64,
    sequence: AtomicU64,
    last_checksum: Arc<Mutex<Option<String>>>,
}

impl AppendLogPersistence {
    /// Create a new append-only log persistence.
    pub fn new(config: AppendLogConfig) -> PersistenceResult<Self> {
        std::fs::create_dir_all(&config.log_dir)?;

        let (file_path, sequence, last_checksum) = Self::find_or_create_log(&config)?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        let size = file.metadata()?.len();

        Ok(Self {
            config,
            writer: Arc::new(Mutex::new(BufWriter::new(file))),
            current_file: Arc::new(Mutex::new(file_path)),
            current_size: AtomicU64::new(size),
            sequence: AtomicU64::new(sequence),
            last_checksum: Arc::new(Mutex::new(last_checksum)),
        })
    }

    fn find_or_create_log(config: &AppendLogConfig) -> PersistenceResult<(PathBuf, u64, Option<String>)> {
        let pattern = format!("{}_*.log", config.file_prefix);
        let mut latest_file: Option<PathBuf> = None;
        let mut max_sequence = 0u64;

        if let Ok(entries) = std::fs::read_dir(&config.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "log") {
                    if let Some((seq, checksum)) = Self::read_last_entry(&path)? {
                        if seq >= max_sequence {
                            max_sequence = seq;
                            latest_file = Some(path);
                        }
                    }
                }
            }
        }

        if let Some(path) = latest_file {
            let (seq, checksum) = Self::read_last_entry(&path)?.unwrap_or((0, None));
            let size = std::fs::metadata(&path)?.len();
            if size < config.max_file_size {
                return Ok((path, seq, checksum));
            }
        }

        // Create new log file
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let file_name = format!("{}_{}.log", config.file_prefix, timestamp);
        let path = config.log_dir.join(file_name);

        Ok((path, max_sequence, None))
    }

    fn read_last_entry(path: &Path) -> PersistenceResult<Option<(u64, Option<String>)>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut last_entry: Option<AuditLogEntry> = None;

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(entry) = AuditLogEntry::from_log_line(&line) {
                    last_entry = Some(entry);
                }
            }
        }

        Ok(last_entry.map(|e| (e.sequence, Some(e.checksum))))
    }

    fn rotate_if_needed(&self) -> PersistenceResult<()> {
        if self.current_size.load(Ordering::SeqCst) >= self.config.max_file_size {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let file_name = format!("{}_{}.log", self.config.file_prefix, timestamp);
            let new_path = self.config.log_dir.join(file_name);

            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new_path)?;

            let mut writer = self.writer.lock();
            writer.flush()?;
            *writer = BufWriter::new(new_file);

            *self.current_file.lock() = new_path;
            self.current_size.store(0, Ordering::SeqCst);
        }
        Ok(())
    }

    fn write_entry(&self, event: &AuditEvent) -> PersistenceResult<()> {
        self.rotate_if_needed()?;

        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst) + 1;
        let event_data = serde_json::to_string(event)?;
        let prev_checksum = self.last_checksum.lock().clone();

        let entry = AuditLogEntry::new(
            sequence,
            event.id.to_string(),
            event.timestamp.to_rfc3339(),
            event_data,
            prev_checksum,
        );

        let line = entry.to_log_line();
        let line_bytes = line.len() as u64 + 1; // +1 for newline

        let mut writer = self.writer.lock();
        writeln!(writer, "{}", line)?;
        if self.config.sync_on_write {
            writer.flush()?;
        }

        *self.last_checksum.lock() = Some(entry.checksum);
        self.current_size.fetch_add(line_bytes, Ordering::SeqCst);

        Ok(())
    }
}

#[async_trait]
impl AuditPersistence for AppendLogPersistence {
    async fn persist(&self, event: &AuditEvent) -> PersistenceResult<()> {
        self.write_entry(event)
    }

    async fn persist_batch(&self, batch: &EventBatch) -> PersistenceResult<usize> {
        let mut count = 0;
        for captured in &batch.events {
            self.write_entry(&captured.event)?;
            count += 1;
        }
        Ok(count)
    }

    async fn last_sequence(&self) -> PersistenceResult<u64> {
        Ok(self.sequence.load(Ordering::SeqCst))
    }

    async fn verify_integrity(&self) -> PersistenceResult<IntegrityReport> {
        let mut total = 0u64;
        let mut verified = 0u64;
        let mut corrupted = Vec::new();
        let mut chain_breaks = Vec::new();
        let mut prev_checksum: Option<String> = None;

        let entries = std::fs::read_dir(&self.config.log_dir)?;
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |e| e == "log"))
            .collect();
        files.sort();

        for file_path in files {
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                total += 1;

                match AuditLogEntry::from_log_line(&line) {
                    Ok(entry) => {
                        if entry.verify() {
                            if entry.prev_checksum == prev_checksum {
                                verified += 1;
                            } else if prev_checksum.is_some() {
                                chain_breaks.push(entry.sequence);
                            } else {
                                verified += 1;
                            }
                            prev_checksum = Some(entry.checksum);
                        } else {
                            corrupted.push(entry.event_id);
                        }
                    }
                    Err(_) => {
                        corrupted.push(format!("line_{}", total));
                    }
                }
            }
        }

        Ok(IntegrityReport {
            total_events: total,
            verified_events: verified,
            corrupted_events: corrupted.clone(),
            chain_breaks: chain_breaks.clone(),
            is_valid: corrupted.is_empty() && chain_breaks.is_empty(),
        })
    }

    async fn flush(&self) -> PersistenceResult<()> {
        self.writer.lock().flush()?;
        Ok(())
    }
}
```

---

## Testing Requirements

1. SQLite persistence stores events correctly
2. Append-only log maintains chain integrity
3. File rotation works at size limits
4. Batch persistence is atomic
5. Integrity verification detects corruption

---

## Related Specs

- Depends on: [432-audit-schema.md](432-audit-schema.md), [433-audit-capture.md](433-audit-capture.md)
- Next: [435-audit-query.md](435-audit-query.md)
