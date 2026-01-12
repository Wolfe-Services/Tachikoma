# 437 - Audit Export

**Phase:** 20 - Audit System
**Spec ID:** 437
**Status:** Planned
**Dependencies:** 435-audit-query
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement audit event export functionality supporting multiple formats (JSON, CSV, SIEM) for compliance reporting and external analysis.

---

## Acceptance Criteria

- [ ] JSON export with streaming support
- [ ] CSV export for spreadsheet analysis
- [ ] SIEM format export (CEF, LEEF)
- [ ] Configurable field selection
- [ ] Progress tracking for large exports

---

## Implementation Details

### 1. Export Types (src/export.rs)

```rust
//! Audit export functionality.

use crate::{AuditEvent, AuditQuery};
use serde::{Deserialize, Serialize};
use std::io::Write;
use thiserror::Error;

/// Export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// JSON Lines format (one event per line).
    JsonLines,
    /// Pretty-printed JSON array.
    JsonPretty,
    /// CSV format.
    Csv,
    /// Common Event Format (CEF) for SIEM.
    Cef,
    /// Log Event Extended Format (LEEF) for QRadar.
    Leef,
}

/// Export configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Output format.
    pub format: ExportFormat,
    /// Fields to include (empty = all).
    #[serde(default)]
    pub fields: Vec<String>,
    /// Include metadata in export.
    #[serde(default = "default_true")]
    pub include_metadata: bool,
    /// Compress output with gzip.
    #[serde(default)]
    pub compress: bool,
    /// CEF/LEEF device vendor.
    #[serde(default = "default_vendor")]
    pub device_vendor: String,
    /// CEF/LEEF device product.
    #[serde(default = "default_product")]
    pub device_product: String,
    /// CEF/LEEF device version.
    #[serde(default = "default_version")]
    pub device_version: String,
}

fn default_true() -> bool { true }
fn default_vendor() -> String { "Tachikoma".to_string() }
fn default_product() -> String { "AuditSystem".to_string() }
fn default_version() -> String { "1.0".to_string() }

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::JsonLines,
            fields: Vec::new(),
            include_metadata: true,
            compress: false,
            device_vendor: default_vendor(),
            device_product: default_product(),
            device_version: default_version(),
        }
    }
}

/// Export progress callback.
pub type ProgressCallback = Box<dyn Fn(ExportProgress) + Send>;

/// Export progress information.
#[derive(Debug, Clone)]
pub struct ExportProgress {
    pub exported: u64,
    pub total: Option<u64>,
    pub bytes_written: u64,
    pub percent_complete: Option<f32>,
}

/// Export error.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("query error: {0}")]
    Query(String),
    #[error("format error: {0}")]
    Format(String),
}

/// Export result.
pub type ExportResult<T> = Result<T, ExportError>;

/// Trait for export writers.
pub trait ExportWriter: Send {
    /// Write the export header.
    fn write_header(&mut self) -> ExportResult<()>;

    /// Write a single event.
    fn write_event(&mut self, event: &AuditEvent) -> ExportResult<()>;

    /// Write the export footer.
    fn write_footer(&mut self) -> ExportResult<()>;

    /// Flush all buffered data.
    fn flush(&mut self) -> ExportResult<()>;

    /// Get bytes written so far.
    fn bytes_written(&self) -> u64;
}
```

### 2. JSON Export (src/json_export.rs)

```rust
//! JSON export implementation.

use crate::{export::*, AuditEvent};
use std::io::{BufWriter, Write};

/// JSON Lines exporter.
pub struct JsonLinesExporter<W: Write> {
    writer: BufWriter<W>,
    bytes_written: u64,
    config: ExportConfig,
}

impl<W: Write> JsonLinesExporter<W> {
    /// Create a new JSON Lines exporter.
    pub fn new(writer: W, config: ExportConfig) -> Self {
        Self {
            writer: BufWriter::new(writer),
            bytes_written: 0,
            config,
        }
    }
}

impl<W: Write + Send> ExportWriter for JsonLinesExporter<W> {
    fn write_header(&mut self) -> ExportResult<()> {
        // No header for JSON Lines
        Ok(())
    }

    fn write_event(&mut self, event: &AuditEvent) -> ExportResult<()> {
        let json = if self.config.fields.is_empty() {
            serde_json::to_string(event)?
        } else {
            // Filter to selected fields
            let value = serde_json::to_value(event)?;
            let filtered = filter_fields(&value, &self.config.fields);
            serde_json::to_string(&filtered)?
        };

        writeln!(self.writer, "{}", json)?;
        self.bytes_written += json.len() as u64 + 1;
        Ok(())
    }

    fn write_footer(&mut self) -> ExportResult<()> {
        Ok(())
    }

    fn flush(&mut self) -> ExportResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

/// Pretty JSON array exporter.
pub struct JsonPrettyExporter<W: Write> {
    writer: BufWriter<W>,
    bytes_written: u64,
    first_event: bool,
    config: ExportConfig,
}

impl<W: Write> JsonPrettyExporter<W> {
    pub fn new(writer: W, config: ExportConfig) -> Self {
        Self {
            writer: BufWriter::new(writer),
            bytes_written: 0,
            first_event: true,
            config,
        }
    }
}

impl<W: Write + Send> ExportWriter for JsonPrettyExporter<W> {
    fn write_header(&mut self) -> ExportResult<()> {
        write!(self.writer, "[\n")?;
        self.bytes_written += 2;
        Ok(())
    }

    fn write_event(&mut self, event: &AuditEvent) -> ExportResult<()> {
        if !self.first_event {
            write!(self.writer, ",\n")?;
            self.bytes_written += 2;
        }
        self.first_event = false;

        let json = serde_json::to_string_pretty(event)?;
        // Indent each line
        for line in json.lines() {
            write!(self.writer, "  {}\n", line)?;
            self.bytes_written += line.len() as u64 + 3;
        }
        Ok(())
    }

    fn write_footer(&mut self) -> ExportResult<()> {
        write!(self.writer, "]\n")?;
        self.bytes_written += 2;
        Ok(())
    }

    fn flush(&mut self) -> ExportResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

fn filter_fields(value: &serde_json::Value, fields: &[String]) -> serde_json::Value {
    if let serde_json::Value::Object(map) = value {
        let filtered: serde_json::Map<String, serde_json::Value> = map
            .iter()
            .filter(|(k, _)| fields.contains(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::Value::Object(filtered)
    } else {
        value.clone()
    }
}
```

### 3. CSV Export (src/csv_export.rs)

```rust
//! CSV export implementation.

use crate::{export::*, AuditEvent};
use std::io::{BufWriter, Write};

/// Default CSV fields.
const DEFAULT_CSV_FIELDS: &[&str] = &[
    "id", "timestamp", "category", "action", "severity",
    "actor_type", "actor_id", "target_type", "target_id",
    "outcome", "correlation_id",
];

/// CSV exporter.
pub struct CsvExporter<W: Write> {
    writer: BufWriter<W>,
    bytes_written: u64,
    fields: Vec<String>,
}

impl<W: Write> CsvExporter<W> {
    pub fn new(writer: W, config: ExportConfig) -> Self {
        let fields = if config.fields.is_empty() {
            DEFAULT_CSV_FIELDS.iter().map(|s| s.to_string()).collect()
        } else {
            config.fields
        };

        Self {
            writer: BufWriter::new(writer),
            bytes_written: 0,
            fields,
        }
    }

    fn escape_csv(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }
}

impl<W: Write + Send> ExportWriter for CsvExporter<W> {
    fn write_header(&mut self) -> ExportResult<()> {
        let header = self.fields.join(",");
        writeln!(self.writer, "{}", header)?;
        self.bytes_written += header.len() as u64 + 1;
        Ok(())
    }

    fn write_event(&mut self, event: &AuditEvent) -> ExportResult<()> {
        let value = serde_json::to_value(event)?;
        let obj = value.as_object().unwrap();

        let values: Vec<String> = self.fields.iter().map(|field| {
            match obj.get(field) {
                Some(serde_json::Value::String(s)) => Self::escape_csv(s),
                Some(serde_json::Value::Null) => String::new(),
                Some(v) => Self::escape_csv(&v.to_string()),
                None => String::new(),
            }
        }).collect();

        let line = values.join(",");
        writeln!(self.writer, "{}", line)?;
        self.bytes_written += line.len() as u64 + 1;
        Ok(())
    }

    fn write_footer(&mut self) -> ExportResult<()> {
        Ok(())
    }

    fn flush(&mut self) -> ExportResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}
```

### 4. SIEM Export (src/siem_export.rs)

```rust
//! SIEM format export (CEF, LEEF).

use crate::{export::*, AuditEvent, AuditSeverity};
use std::io::{BufWriter, Write};

/// CEF (Common Event Format) exporter.
pub struct CefExporter<W: Write> {
    writer: BufWriter<W>,
    bytes_written: u64,
    config: ExportConfig,
}

impl<W: Write> CefExporter<W> {
    pub fn new(writer: W, config: ExportConfig) -> Self {
        Self {
            writer: BufWriter::new(writer),
            bytes_written: 0,
            config,
        }
    }

    fn severity_to_cef(severity: &AuditSeverity) -> u8 {
        match severity {
            AuditSeverity::Info => 1,
            AuditSeverity::Low => 3,
            AuditSeverity::Medium => 5,
            AuditSeverity::High => 7,
            AuditSeverity::Critical => 10,
        }
    }

    fn escape_cef(s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('|', "\\|")
         .replace('=', "\\=")
         .replace('\n', "\\n")
    }
}

impl<W: Write + Send> ExportWriter for CefExporter<W> {
    fn write_header(&mut self) -> ExportResult<()> {
        Ok(())
    }

    fn write_event(&mut self, event: &AuditEvent) -> ExportResult<()> {
        // CEF:Version|Device Vendor|Device Product|Device Version|Signature ID|Name|Severity|Extension
        let cef_severity = Self::severity_to_cef(&event.severity);
        let signature_id = format!("{:?}", event.action);
        let name = format!("{} - {:?}", event.category, event.action);

        let mut extensions = Vec::new();
        extensions.push(format!("rt={}", event.timestamp.timestamp_millis()));
        extensions.push(format!("eventId={}", event.id));

        if let Some(ref target) = event.target {
            extensions.push(format!("dhost={}", Self::escape_cef(&target.resource_id)));
        }

        if let Some(ref ip) = event.ip_address {
            extensions.push(format!("src={}", ip));
        }

        if let Some(ref correlation_id) = event.correlation_id {
            extensions.push(format!("externalId={}", Self::escape_cef(correlation_id)));
        }

        let line = format!(
            "CEF:0|{}|{}|{}|{}|{}|{}|{}",
            Self::escape_cef(&self.config.device_vendor),
            Self::escape_cef(&self.config.device_product),
            Self::escape_cef(&self.config.device_version),
            Self::escape_cef(&signature_id),
            Self::escape_cef(&name),
            cef_severity,
            extensions.join(" ")
        );

        writeln!(self.writer, "{}", line)?;
        self.bytes_written += line.len() as u64 + 1;
        Ok(())
    }

    fn write_footer(&mut self) -> ExportResult<()> {
        Ok(())
    }

    fn flush(&mut self) -> ExportResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

/// LEEF (Log Event Extended Format) exporter for IBM QRadar.
pub struct LeefExporter<W: Write> {
    writer: BufWriter<W>,
    bytes_written: u64,
    config: ExportConfig,
}

impl<W: Write> LeefExporter<W> {
    pub fn new(writer: W, config: ExportConfig) -> Self {
        Self {
            writer: BufWriter::new(writer),
            bytes_written: 0,
            config,
        }
    }
}

impl<W: Write + Send> ExportWriter for LeefExporter<W> {
    fn write_header(&mut self) -> ExportResult<()> {
        Ok(())
    }

    fn write_event(&mut self, event: &AuditEvent) -> ExportResult<()> {
        // LEEF:Version|Vendor|Product|Version|EventID|key=value\tkey=value
        let event_id = format!("{:?}", event.action);

        let mut attrs = Vec::new();
        attrs.push(format!("devTime={}", event.timestamp.to_rfc3339()));
        attrs.push(format!("cat={}", event.category));
        attrs.push(format!("sev={:?}", event.severity));

        if let Some(ref ip) = event.ip_address {
            attrs.push(format!("src={}", ip));
        }

        let line = format!(
            "LEEF:2.0|{}|{}|{}|{}|{}",
            self.config.device_vendor,
            self.config.device_product,
            self.config.device_version,
            event_id,
            attrs.join("\t")
        );

        writeln!(self.writer, "{}", line)?;
        self.bytes_written += line.len() as u64 + 1;
        Ok(())
    }

    fn write_footer(&mut self) -> ExportResult<()> {
        Ok(())
    }

    fn flush(&mut self) -> ExportResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}
```

---

## Testing Requirements

1. JSON export produces valid JSON
2. CSV export handles special characters
3. CEF format is SIEM-compatible
4. Field filtering works correctly
5. Progress tracking is accurate

---

## Related Specs

- Depends on: [435-audit-query.md](435-audit-query.md)
- Next: [438-audit-search.md](438-audit-search.md)
