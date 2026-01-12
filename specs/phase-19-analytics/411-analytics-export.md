# Spec 411: Analytics Data Export

## Phase
19 - Analytics/Telemetry

## Spec ID
411

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 409: Analytics Storage (data persistence)
- Spec 410: Analytics Aggregation (aggregated data)

## Estimated Context
~9%

---

## Objective

Implement data export capabilities for analytics, enabling users to extract raw events and aggregated metrics in various formats for external analysis, backup, and compliance purposes.

---

## Acceptance Criteria

- [ ] Implement export to multiple formats (JSON, CSV, Parquet)
- [ ] Support filtering by time range, category, and event type
- [ ] Create streaming export for large datasets
- [ ] Implement compression for exports
- [ ] Support incremental exports
- [ ] Create scheduled export functionality
- [ ] Implement export validation and checksums
- [ ] Support export to external destinations

---

## Implementation Details

### Export System

```rust
// src/analytics/export.rs

use crate::analytics::aggregation::AggregatedMetric;
use crate::analytics::storage::{AnalyticsStorage, StorageError};
use crate::analytics::types::{AnalyticsEvent, EventCategory, EventType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Export file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// JSON array format
    Json,
    /// Newline-delimited JSON
    Ndjson,
    /// Comma-separated values
    Csv,
    /// Apache Parquet (columnar)
    Parquet,
}

impl ExportFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Ndjson => "ndjson",
            Self::Csv => "csv",
            Self::Parquet => "parquet",
        }
    }

    pub fn mime_type(&self) -> &str {
        match self {
            Self::Json => "application/json",
            Self::Ndjson => "application/x-ndjson",
            Self::Csv => "text/csv",
            Self::Parquet => "application/vnd.apache.parquet",
        }
    }
}

/// Export filter criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportFilter {
    /// Start time (inclusive)
    pub start_time: Option<DateTime<Utc>>,
    /// End time (exclusive)
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by categories
    pub categories: Vec<EventCategory>,
    /// Filter by event types
    pub event_types: Vec<EventType>,
    /// Filter by session ID
    pub session_id: Option<String>,
    /// Maximum number of events
    pub limit: Option<usize>,
}

impl ExportFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn with_categories(mut self, categories: Vec<EventCategory>) -> Self {
        self.categories = categories;
        self
    }

    pub fn with_event_types(mut self, event_types: Vec<EventType>) -> Self {
        self.event_types = event_types;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &AnalyticsEvent) -> bool {
        if let Some(start) = self.start_time {
            if event.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if event.timestamp >= end {
                return false;
            }
        }

        if !self.categories.is_empty() && !self.categories.contains(&event.category) {
            return false;
        }

        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type) {
            return false;
        }

        if let Some(ref session_id) = self.session_id {
            if event.session_id.map(|id| id.to_string()) != Some(session_id.clone()) {
                return false;
            }
        }

        true
    }
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Output format
    pub format: ExportFormat,
    /// Compress output
    pub compress: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Include metadata header
    pub include_metadata: bool,
    /// Pretty print (for JSON)
    pub pretty: bool,
    /// Chunk size for streaming
    pub chunk_size: usize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            compress: true,
            compression_level: 6,
            include_metadata: true,
            pretty: false,
            chunk_size: 1000,
        }
    }
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Number of events
    pub event_count: u64,
    /// Time range covered
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filters applied
    pub filters: ExportFilter,
    /// Export format
    pub format: ExportFormat,
    /// SHA-256 checksum of data
    pub checksum: Option<String>,
    /// Export version
    pub version: String,
}

/// Export result
#[derive(Debug)]
pub struct ExportResult {
    /// Path to exported file
    pub path: PathBuf,
    /// Export metadata
    pub metadata: ExportMetadata,
    /// File size in bytes
    pub file_size: u64,
}

/// Trait for export destinations
#[async_trait]
pub trait ExportDestination: Send + Sync {
    /// Write data to the destination
    async fn write(&mut self, data: &[u8]) -> Result<(), ExportError>;
    /// Finalize the export
    async fn finalize(&mut self) -> Result<(), ExportError>;
}

/// File export destination
pub struct FileDestination {
    writer: BufWriter<Box<dyn Write + Send>>,
    path: PathBuf,
    hasher: Sha256,
}

impl FileDestination {
    pub fn new(path: PathBuf, compress: bool, compression_level: u32) -> Result<Self, ExportError> {
        let file = File::create(&path)
            .map_err(|e| ExportError::IoError(e.to_string()))?;

        let writer: Box<dyn Write + Send> = if compress {
            Box::new(GzEncoder::new(file, Compression::new(compression_level)))
        } else {
            Box::new(file)
        };

        Ok(Self {
            writer: BufWriter::new(writer),
            path,
            hasher: Sha256::new(),
        })
    }

    pub fn checksum(&self) -> String {
        format!("{:x}", self.hasher.clone().finalize())
    }
}

#[async_trait]
impl ExportDestination for FileDestination {
    async fn write(&mut self, data: &[u8]) -> Result<(), ExportError> {
        self.writer
            .write_all(data)
            .map_err(|e| ExportError::IoError(e.to_string()))?;
        self.hasher.update(data);
        Ok(())
    }

    async fn finalize(&mut self) -> Result<(), ExportError> {
        self.writer
            .flush()
            .map_err(|e| ExportError::IoError(e.to_string()))?;
        Ok(())
    }
}

/// Analytics data exporter
pub struct Exporter {
    storage: Arc<dyn AnalyticsStorage>,
}

impl Exporter {
    pub fn new(storage: Arc<dyn AnalyticsStorage>) -> Self {
        Self { storage }
    }

    /// Export events to a file
    pub async fn export_to_file(
        &self,
        path: impl AsRef<Path>,
        filter: ExportFilter,
        options: ExportOptions,
    ) -> Result<ExportResult, ExportError> {
        let path = path.as_ref().to_path_buf();
        let extension = if options.compress {
            format!("{}.gz", options.format.extension())
        } else {
            options.format.extension().to_string()
        };

        let final_path = path.with_extension(extension);

        let mut destination = FileDestination::new(
            final_path.clone(),
            options.compress,
            options.compression_level,
        )?;

        let metadata = self
            .export_events(&filter, &options, &mut destination)
            .await?;

        destination.finalize().await?;

        let file_size = std::fs::metadata(&final_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ExportResult {
            path: final_path,
            metadata: ExportMetadata {
                checksum: Some(destination.checksum()),
                ..metadata
            },
            file_size,
        })
    }

    /// Export events to a destination
    async fn export_events(
        &self,
        filter: &ExportFilter,
        options: &ExportOptions,
        destination: &mut dyn ExportDestination,
    ) -> Result<ExportMetadata, ExportError> {
        let start = filter.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(365));
        let end = filter.end_time.unwrap_or_else(Utc::now);

        let events = self
            .storage
            .query_by_time(start, end, filter.limit)
            .await
            .map_err(|e| ExportError::StorageError(e.to_string()))?;

        let filtered_events: Vec<_> = events
            .into_iter()
            .filter(|e| filter.matches(e))
            .collect();

        let event_count = filtered_events.len() as u64;

        let time_range = if !filtered_events.is_empty() {
            let min_time = filtered_events.iter().map(|e| e.timestamp).min().unwrap();
            let max_time = filtered_events.iter().map(|e| e.timestamp).max().unwrap();
            Some((min_time, max_time))
        } else {
            None
        };

        // Write data based on format
        match options.format {
            ExportFormat::Json => {
                self.write_json(&filtered_events, destination, options.pretty)
                    .await?;
            }
            ExportFormat::Ndjson => {
                self.write_ndjson(&filtered_events, destination).await?;
            }
            ExportFormat::Csv => {
                self.write_csv(&filtered_events, destination).await?;
            }
            ExportFormat::Parquet => {
                self.write_parquet(&filtered_events, destination).await?;
            }
        }

        Ok(ExportMetadata {
            exported_at: Utc::now(),
            event_count,
            time_range,
            filters: filter.clone(),
            format: options.format,
            checksum: None,
            version: "1.0".to_string(),
        })
    }

    async fn write_json(
        &self,
        events: &[AnalyticsEvent],
        destination: &mut dyn ExportDestination,
        pretty: bool,
    ) -> Result<(), ExportError> {
        let json = if pretty {
            serde_json::to_string_pretty(events)
        } else {
            serde_json::to_string(events)
        }
        .map_err(|e| ExportError::SerializationError(e.to_string()))?;

        destination.write(json.as_bytes()).await
    }

    async fn write_ndjson(
        &self,
        events: &[AnalyticsEvent],
        destination: &mut dyn ExportDestination,
    ) -> Result<(), ExportError> {
        for event in events {
            let line = serde_json::to_string(event)
                .map_err(|e| ExportError::SerializationError(e.to_string()))?;
            destination.write(line.as_bytes()).await?;
            destination.write(b"\n").await?;
        }
        Ok(())
    }

    async fn write_csv(
        &self,
        events: &[AnalyticsEvent],
        destination: &mut dyn ExportDestination,
    ) -> Result<(), ExportError> {
        // Write header
        let header = "id,category,event_type,timestamp,session_id,priority,data\n";
        destination.write(header.as_bytes()).await?;

        // Write rows
        for event in events {
            let row = format!(
                "{:?},{:?},{:?},{},{},{:?},{}\n",
                event.id,
                event.category,
                event.event_type,
                event.timestamp.to_rfc3339(),
                event.session_id.map(|id| id.to_string()).unwrap_or_default(),
                event.priority,
                serde_json::to_string(&event.data)
                    .map_err(|e| ExportError::SerializationError(e.to_string()))?
                    .replace(',', "\\,"),
            );
            destination.write(row.as_bytes()).await?;
        }

        Ok(())
    }

    async fn write_parquet(
        &self,
        events: &[AnalyticsEvent],
        destination: &mut dyn ExportDestination,
    ) -> Result<(), ExportError> {
        // Parquet writing would require arrow/parquet crate
        // For now, fall back to JSON
        self.write_json(events, destination, false).await
    }

    /// Export aggregated metrics
    pub async fn export_metrics(
        &self,
        metrics: &[AggregatedMetric],
        path: impl AsRef<Path>,
        options: ExportOptions,
    ) -> Result<ExportResult, ExportError> {
        let path = path.as_ref().to_path_buf();
        let extension = if options.compress {
            format!("{}.gz", options.format.extension())
        } else {
            options.format.extension().to_string()
        };

        let final_path = path.with_extension(extension);

        let mut destination = FileDestination::new(
            final_path.clone(),
            options.compress,
            options.compression_level,
        )?;

        let json = if options.pretty {
            serde_json::to_string_pretty(metrics)
        } else {
            serde_json::to_string(metrics)
        }
        .map_err(|e| ExportError::SerializationError(e.to_string()))?;

        destination.write(json.as_bytes()).await?;
        destination.finalize().await?;

        let file_size = std::fs::metadata(&final_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ExportResult {
            path: final_path,
            metadata: ExportMetadata {
                exported_at: Utc::now(),
                event_count: metrics.len() as u64,
                time_range: None,
                filters: ExportFilter::default(),
                format: options.format,
                checksum: Some(destination.checksum()),
                version: "1.0".to_string(),
            },
            file_size,
        })
    }
}

/// Scheduled export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledExport {
    /// Export identifier
    pub id: String,
    /// Cron expression for schedule
    pub schedule: String,
    /// Export filter
    pub filter: ExportFilter,
    /// Export options
    pub options: ExportOptions,
    /// Output directory
    pub output_dir: PathBuf,
    /// File name template
    pub filename_template: String,
    /// Keep last N exports
    pub retention_count: Option<u32>,
}

/// Export scheduler
pub struct ExportScheduler {
    exporter: Arc<Exporter>,
    schedules: Arc<Mutex<Vec<ScheduledExport>>>,
}

impl ExportScheduler {
    pub fn new(exporter: Arc<Exporter>) -> Self {
        Self {
            exporter,
            schedules: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a scheduled export
    pub async fn add_schedule(&self, schedule: ScheduledExport) {
        let mut schedules = self.schedules.lock().await;
        schedules.push(schedule);
    }

    /// Remove a scheduled export
    pub async fn remove_schedule(&self, id: &str) {
        let mut schedules = self.schedules.lock().await;
        schedules.retain(|s| s.id != id);
    }

    /// Run pending exports
    pub async fn run_pending(&self) -> Result<Vec<ExportResult>, ExportError> {
        let schedules = self.schedules.lock().await;
        let mut results = Vec::new();

        for schedule in schedules.iter() {
            let filename = format!(
                "{}_{}.{}",
                schedule.filename_template,
                Utc::now().format("%Y%m%d_%H%M%S"),
                schedule.options.format.extension()
            );

            let path = schedule.output_dir.join(&filename);

            let result = self
                .exporter
                .export_to_file(&path, schedule.filter.clone(), schedule.options.clone())
                .await?;

            results.push(result);

            // Clean up old exports if retention is configured
            if let Some(retention) = schedule.retention_count {
                self.cleanup_old_exports(&schedule.output_dir, retention).await?;
            }
        }

        Ok(results)
    }

    async fn cleanup_old_exports(
        &self,
        dir: &Path,
        keep_count: u32,
    ) -> Result<(), ExportError> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| ExportError::IoError(e.to_string()))?
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        entries.reverse();

        for entry in entries.into_iter().skip(keep_count as usize) {
            std::fs::remove_file(entry.path())
                .map_err(|e| ExportError::IoError(e.to_string()))?;
        }

        Ok(())
    }
}

/// Export errors
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::config::StorageConfig;
    use crate::analytics::storage::SqliteAnalyticsStorage;
    use crate::analytics::types::{EventBatch, EventBuilder};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_export_filter() {
        let event = EventBuilder::new(EventType::MissionCreated).build();

        let filter = ExportFilter::new()
            .with_categories(vec![EventCategory::Usage]);

        assert!(filter.matches(&event));

        let filter = ExportFilter::new()
            .with_categories(vec![EventCategory::Error]);

        assert!(!filter.matches(&event));
    }

    #[tokio::test]
    async fn test_export_to_file() {
        let storage = Arc::new(
            SqliteAnalyticsStorage::in_memory(StorageConfig::default()).unwrap()
        );

        // Add test events
        let events: Vec<_> = (0..10)
            .map(|_| EventBuilder::new(EventType::FeatureUsed).build())
            .collect();
        let batch = EventBatch::new(events, 1);
        storage.store_batch(&batch).await.unwrap();

        let exporter = Exporter::new(storage);
        let dir = tempdir().unwrap();
        let path = dir.path().join("export");

        let result = exporter
            .export_to_file(
                &path,
                ExportFilter::new(),
                ExportOptions {
                    format: ExportFormat::Json,
                    compress: false,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(result.metadata.event_count, 10);
        assert!(result.path.exists());
    }

    #[tokio::test]
    async fn test_ndjson_export() {
        let storage = Arc::new(
            SqliteAnalyticsStorage::in_memory(StorageConfig::default()).unwrap()
        );

        let events: Vec<_> = (0..5)
            .map(|_| EventBuilder::new(EventType::SessionStarted).build())
            .collect();
        let batch = EventBatch::new(events, 1);
        storage.store_batch(&batch).await.unwrap();

        let exporter = Exporter::new(storage);
        let dir = tempdir().unwrap();
        let path = dir.path().join("export");

        let result = exporter
            .export_to_file(
                &path,
                ExportFilter::new(),
                ExportOptions {
                    format: ExportFormat::Ndjson,
                    compress: false,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let content = std::fs::read_to_string(&result.path).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 5);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Filter matching logic
   - Format serialization
   - Checksum calculation
   - File path generation

2. **Integration Tests**
   - Full export pipeline
   - Large dataset export
   - Scheduled exports

3. **Format Tests**
   - JSON validity
   - CSV parsing
   - NDJSON line integrity

---

## Related Specs

- Spec 406: Analytics Types
- Spec 409: Analytics Storage
- Spec 423: Export Formats
- Spec 424: Data Retention
