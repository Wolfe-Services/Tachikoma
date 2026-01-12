# 048 - Primitives Operation Logging

**Phase:** 2 - Five Primitives
**Spec ID:** 048
**Status:** Planned
**Dependencies:** 046-primitives-trait
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement comprehensive audit logging for all primitive operations to track actions, enable debugging, and support security review.

---

## Acceptance Criteria

- [x] Log all primitive invocations with inputs
- [x] Log operation outcomes (success/failure)
- [x] Capture timing information
- [x] Sensitive data redaction
- [x] Structured log format
- [x] Log rotation and retention

---

## Implementation Details

### 1. Audit Module (src/audit/mod.rs)

```rust
//! Audit logging for primitive operations.

mod entry;
mod logger;
mod redact;

pub use entry::{AuditEntry, AuditOutcome, OperationType};
pub use logger::{AuditLogger, AuditConfig};
pub use redact::Redactor;

use crate::context::PrimitiveContext;
use std::time::Duration;
use tracing::{info, warn, Span};

/// Log an operation start.
pub fn log_operation_start(
    ctx: &PrimitiveContext,
    operation: OperationType,
    inputs: &impl serde::Serialize,
) {
    let redacted = Redactor::default().redact_value(
        &serde_json::to_value(inputs).unwrap_or_default()
    );

    info!(
        operation_id = %ctx.operation_id,
        operation = %operation,
        inputs = %redacted,
        "Primitive operation started"
    );
}

/// Log an operation success.
pub fn log_operation_success(
    ctx: &PrimitiveContext,
    operation: OperationType,
    duration: Duration,
    output_summary: &str,
) {
    info!(
        operation_id = %ctx.operation_id,
        operation = %operation,
        duration_ms = duration.as_millis() as u64,
        output = %output_summary,
        "Primitive operation succeeded"
    );
}

/// Log an operation failure.
pub fn log_operation_failure(
    ctx: &PrimitiveContext,
    operation: OperationType,
    duration: Duration,
    error: &str,
) {
    warn!(
        operation_id = %ctx.operation_id,
        operation = %operation,
        duration_ms = duration.as_millis() as u64,
        error = %error,
        "Primitive operation failed"
    );
}
```

### 2. Audit Entry Types (src/audit/entry.rs)

```rust
//! Audit entry types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Type of primitive operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    ReadFile,
    ListFiles,
    Bash,
    EditFile,
    CodeSearch,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFile => write!(f, "read_file"),
            Self::ListFiles => write!(f, "list_files"),
            Self::Bash => write!(f, "bash"),
            Self::EditFile => write!(f, "edit_file"),
            Self::CodeSearch => write!(f, "code_search"),
        }
    }
}

/// Outcome of an operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure { error_code: String, message: String },
    Timeout,
    Cancelled,
}

impl AuditOutcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self::Failure {
            error_code: code.to_string(),
            message: message.to_string(),
        }
    }
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique operation ID.
    pub operation_id: String,
    /// Type of operation.
    pub operation: OperationType,
    /// Timestamp of operation.
    pub timestamp: DateTime<Utc>,
    /// Duration of operation.
    pub duration_ms: u64,
    /// Outcome of operation.
    pub outcome: AuditOutcome,
    /// Working directory.
    pub working_dir: PathBuf,
    /// Input parameters (redacted).
    pub inputs: serde_json::Value,
    /// Output summary (redacted).
    pub output_summary: Option<String>,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl AuditEntry {
    /// Create a new audit entry.
    pub fn new(
        operation_id: String,
        operation: OperationType,
        working_dir: PathBuf,
    ) -> Self {
        Self {
            operation_id,
            operation,
            timestamp: Utc::now(),
            duration_ms: 0,
            outcome: AuditOutcome::Success,
            working_dir,
            inputs: serde_json::Value::Null,
            output_summary: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the inputs.
    pub fn with_inputs(mut self, inputs: serde_json::Value) -> Self {
        self.inputs = inputs;
        self
    }

    /// Set the duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = duration.as_millis() as u64;
        self
    }

    /// Set the outcome.
    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set output summary.
    pub fn with_output(mut self, summary: &str) -> Self {
        self.output_summary = Some(summary.to_string());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Format as a log line.
    pub fn to_log_line(&self) -> String {
        format!(
            "[{}] {} {} {:?} {}ms - {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.operation_id,
            self.operation,
            self.outcome,
            self.duration_ms,
            self.output_summary.as_deref().unwrap_or(""),
        )
    }
}

/// Builder for audit entries during operation execution.
pub struct AuditBuilder {
    entry: AuditEntry,
    start_time: std::time::Instant,
}

impl AuditBuilder {
    /// Start building an audit entry.
    pub fn start(
        operation_id: String,
        operation: OperationType,
        working_dir: PathBuf,
    ) -> Self {
        Self {
            entry: AuditEntry::new(operation_id, operation, working_dir),
            start_time: std::time::Instant::now(),
        }
    }

    /// Set inputs.
    pub fn inputs(mut self, inputs: serde_json::Value) -> Self {
        self.entry.inputs = inputs;
        self
    }

    /// Complete with success.
    pub fn success(mut self, output_summary: &str) -> AuditEntry {
        self.entry.duration_ms = self.start_time.elapsed().as_millis() as u64;
        self.entry.outcome = AuditOutcome::Success;
        self.entry.output_summary = Some(output_summary.to_string());
        self.entry
    }

    /// Complete with failure.
    pub fn failure(mut self, error_code: &str, message: &str) -> AuditEntry {
        self.entry.duration_ms = self.start_time.elapsed().as_millis() as u64;
        self.entry.outcome = AuditOutcome::error(error_code, message);
        self.entry
    }

    /// Complete with timeout.
    pub fn timeout(mut self) -> AuditEntry {
        self.entry.duration_ms = self.start_time.elapsed().as_millis() as u64;
        self.entry.outcome = AuditOutcome::Timeout;
        self.entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            "op123".to_string(),
            OperationType::ReadFile,
            PathBuf::from("/project"),
        )
        .with_duration(Duration::from_millis(150))
        .with_output("Read 1024 bytes");

        assert_eq!(entry.operation_id, "op123");
        assert_eq!(entry.duration_ms, 150);
        assert!(entry.outcome.is_success());
    }

    #[test]
    fn test_audit_builder() {
        let builder = AuditBuilder::start(
            "op456".to_string(),
            OperationType::Bash,
            PathBuf::from("/tmp"),
        );

        std::thread::sleep(std::time::Duration::from_millis(10));

        let entry = builder.success("Command completed");
        assert!(entry.duration_ms >= 10);
        assert!(entry.outcome.is_success());
    }

    #[test]
    fn test_outcome_failure() {
        let outcome = AuditOutcome::error("FILE_NOT_FOUND", "File does not exist");
        assert!(!outcome.is_success());
    }
}
```

### 3. Audit Logger (src/audit/logger.rs)

```rust
//! Audit logger implementation.

use super::entry::AuditEntry;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

/// Audit logger configuration.
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Log file path.
    pub log_file: Option<PathBuf>,
    /// Maximum log file size in bytes.
    pub max_file_size: u64,
    /// Number of backup files to keep.
    pub max_backups: usize,
    /// Log to stdout.
    pub log_stdout: bool,
    /// Include inputs in logs.
    pub include_inputs: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            log_file: None,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_backups: 5,
            log_stdout: false,
            include_inputs: true,
        }
    }
}

/// Audit logger.
pub struct AuditLogger {
    config: AuditConfig,
    writer: Option<Arc<Mutex<BufWriter<File>>>>,
    current_size: Arc<Mutex<u64>>,
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(config: AuditConfig) -> std::io::Result<Self> {
        let writer = if let Some(ref path) = config.log_file {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            let size = file.metadata()?.len();
            Some((Arc::new(Mutex::new(BufWriter::new(file))), size))
        } else {
            None
        };

        let (writer, current_size) = match writer {
            Some((w, s)) => (Some(w), s),
            None => (None, 0),
        };

        Ok(Self {
            config,
            writer,
            current_size: Arc::new(Mutex::new(current_size)),
        })
    }

    /// Log an audit entry.
    pub fn log(&self, entry: &AuditEntry) -> std::io::Result<()> {
        let json = serde_json::to_string(entry)?;

        // Check for rotation
        self.maybe_rotate()?;

        // Write to file
        if let Some(ref writer) = self.writer {
            let mut w = writer.lock().unwrap();
            writeln!(w, "{}", json)?;
            w.flush()?;

            let mut size = self.current_size.lock().unwrap();
            *size += json.len() as u64 + 1;
        }

        // Write to stdout if configured
        if self.config.log_stdout {
            println!("{}", entry.to_log_line());
        }

        debug!(
            operation_id = %entry.operation_id,
            operation = %entry.operation,
            outcome = ?entry.outcome,
            "Audit entry logged"
        );

        Ok(())
    }

    /// Maybe rotate the log file.
    fn maybe_rotate(&self) -> std::io::Result<()> {
        let size = *self.current_size.lock().unwrap();
        if size < self.config.max_file_size {
            return Ok(());
        }

        if let Some(ref path) = self.config.log_file {
            self.rotate_file(path)?;
        }

        Ok(())
    }

    /// Rotate log file.
    fn rotate_file(&self, path: &PathBuf) -> std::io::Result<()> {
        debug!("Rotating audit log file");

        // Shift existing backups
        for i in (1..self.config.max_backups).rev() {
            let from = path.with_extension(format!("log.{}", i));
            let to = path.with_extension(format!("log.{}", i + 1));
            if from.exists() {
                std::fs::rename(&from, &to)?;
            }
        }

        // Rename current to .1
        let backup = path.with_extension("log.1");
        if path.exists() {
            std::fs::rename(path, &backup)?;
        }

        // Reset size counter
        *self.current_size.lock().unwrap() = 0;

        Ok(())
    }

    /// Get all entries from log file.
    pub fn read_entries(&self) -> std::io::Result<Vec<AuditEntry>> {
        let Some(ref path) = self.config.log_file else {
            return Ok(Vec::new());
        };

        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(path)?;
        let entries: Vec<AuditEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(entries)
    }
}

/// Global audit logger instance.
static AUDIT_LOGGER: std::sync::OnceLock<AuditLogger> = std::sync::OnceLock::new();

/// Initialize the global audit logger.
pub fn init_audit_logger(config: AuditConfig) -> std::io::Result<()> {
    let logger = AuditLogger::new(config)?;
    AUDIT_LOGGER
        .set(logger)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Logger already initialized"))
}

/// Log an entry using the global logger.
pub fn audit(entry: &AuditEntry) {
    if let Some(logger) = AUDIT_LOGGER.get() {
        if let Err(e) = logger.log(entry) {
            error!("Failed to write audit log: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::entry::{AuditOutcome, OperationType};
    use tempfile::tempdir;

    #[test]
    fn test_audit_logger() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("audit.log");

        let config = AuditConfig {
            log_file: Some(log_path.clone()),
            ..Default::default()
        };

        let logger = AuditLogger::new(config).unwrap();

        let entry = AuditEntry::new(
            "test".to_string(),
            OperationType::ReadFile,
            PathBuf::from("/tmp"),
        );

        logger.log(&entry).unwrap();

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("test"));
        assert!(content.contains("read_file"));
    }
}
```

### 4. Data Redaction (src/audit/redact.rs)

```rust
//! Sensitive data redaction.

use serde_json::Value;
use std::collections::HashSet;

/// Redactor for sensitive data.
pub struct Redactor {
    /// Fields to redact.
    sensitive_fields: HashSet<String>,
    /// Replacement string.
    replacement: String,
}

impl Default for Redactor {
    fn default() -> Self {
        let mut sensitive = HashSet::new();
        sensitive.insert("password".to_string());
        sensitive.insert("secret".to_string());
        sensitive.insert("token".to_string());
        sensitive.insert("api_key".to_string());
        sensitive.insert("apikey".to_string());
        sensitive.insert("auth".to_string());
        sensitive.insert("credential".to_string());
        sensitive.insert("private_key".to_string());

        Self {
            sensitive_fields: sensitive,
            replacement: "[REDACTED]".to_string(),
        }
    }
}

impl Redactor {
    /// Create a new redactor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a sensitive field.
    pub fn add_field(mut self, field: &str) -> Self {
        self.sensitive_fields.insert(field.to_lowercase());
        self
    }

    /// Set replacement string.
    pub fn replacement(mut self, s: &str) -> Self {
        self.replacement = s.to_string();
        self
    }

    /// Redact sensitive data from a JSON value.
    pub fn redact_value(&self, value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    let new_v = if self.is_sensitive(k) {
                        Value::String(self.replacement.clone())
                    } else {
                        self.redact_value(v)
                    };
                    new_map.insert(k.clone(), new_v);
                }
                Value::Object(new_map)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.redact_value(v)).collect())
            }
            _ => value.clone(),
        }
    }

    /// Check if a field name is sensitive.
    fn is_sensitive(&self, field: &str) -> bool {
        let lower = field.to_lowercase();
        self.sensitive_fields.iter().any(|s| lower.contains(s))
    }

    /// Redact a string that may contain sensitive patterns.
    pub fn redact_string(&self, s: &str) -> String {
        let mut result = s.to_string();

        // Redact common patterns
        let patterns = [
            (r"(?i)(password|passwd|pwd)\s*[=:]\s*\S+", "$1=[REDACTED]"),
            (r"(?i)(token|api_key|apikey)\s*[=:]\s*\S+", "$1=[REDACTED]"),
            (r"(?i)(bearer|basic)\s+\S+", "$1 [REDACTED]"),
        ];

        for (pattern, replacement) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                result = re.replace_all(&result, replacement).to_string();
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact_sensitive_field() {
        let redactor = Redactor::new();
        let input = json!({
            "username": "admin",
            "password": "secret123"
        });

        let output = redactor.redact_value(&input);

        assert_eq!(output["username"], "admin");
        assert_eq!(output["password"], "[REDACTED]");
    }

    #[test]
    fn test_redact_nested() {
        let redactor = Redactor::new();
        let input = json!({
            "config": {
                "api_key": "abc123"
            }
        });

        let output = redactor.redact_value(&input);
        assert_eq!(output["config"]["api_key"], "[REDACTED]");
    }

    #[test]
    fn test_redact_string() {
        let redactor = Redactor::new();
        let input = "curl -H 'Authorization: Bearer abc123' https://api.example.com";
        let output = redactor.redact_string(input);

        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("abc123"));
    }
}
```

---

## Testing Requirements

1. All operations are logged with correct data
2. Sensitive data is redacted
3. Log rotation works correctly
4. Audit entries can be read back
5. Timing information is accurate
6. Error outcomes include error details
7. Global logger initialization works

---

## Related Specs

- Depends on: [046-primitives-trait.md](046-primitives-trait.md)
- Next: [049-primitives-rate-limit.md](049-primitives-rate-limit.md)
- Related: [026-logging-infrastructure.md](../phase-01-common/026-logging-infrastructure.md)
