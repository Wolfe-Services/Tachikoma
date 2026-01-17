//! Audit export functionality.

use serde::{Deserialize, Serialize};
use std::io::Write;
use tachikoma_audit_types::AuditEvent;
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

fn default_true() -> bool {
    true
}

fn default_vendor() -> String {
    "Tachikoma".to_string()
}

fn default_product() -> String {
    "AuditSystem".to_string()
}

fn default_version() -> String {
    "1.0".to_string()
}

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

impl ExportProgress {
    /// Create a new progress instance.
    pub fn new(exported: u64, total: Option<u64>, bytes_written: u64) -> Self {
        let percent_complete = total.map(|t| if t > 0 { exported as f32 / t as f32 * 100.0 } else { 0.0 });
        Self {
            exported,
            total,
            bytes_written,
            percent_complete,
        }
    }
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