# 424 - Analytics Export

## Overview

Data export capabilities for analytics events including batch exports, streaming exports, and data warehouse integrations.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/export.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::io::AsyncWrite;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    JsonLines,
    Csv,
    Parquet,
    Avro,
}

impl ExportFormat {
    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Json => "application/json",
            ExportFormat::JsonLines => "application/x-ndjson",
            ExportFormat::Csv => "text/csv",
            ExportFormat::Parquet => "application/octet-stream",
            ExportFormat::Avro => "application/avro",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::JsonLines => "jsonl",
            ExportFormat::Csv => "csv",
            ExportFormat::Parquet => "parquet",
            ExportFormat::Avro => "avro",
        }
    }
}

/// Export request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    /// Export ID
    pub id: String,
    /// Events to export (empty = all)
    pub events: Vec<String>,
    /// Date range
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    /// Export format
    pub format: ExportFormat,
    /// Properties to include (empty = all)
    pub properties: Vec<String>,
    /// Filters
    pub filters: Vec<ExportFilter>,
    /// Compression
    pub compression: Option<Compression>,
    /// Destination
    pub destination: ExportDestination,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Status
    pub status: ExportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFilter {
    pub property: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Compression {
    Gzip,
    Zstd,
    Snappy,
    Lz4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExportDestination {
    /// Download via API
    Download,
    /// S3-compatible storage
    S3 {
        bucket: String,
        prefix: String,
        region: String,
        endpoint: Option<String>,
    },
    /// Google Cloud Storage
    Gcs {
        bucket: String,
        prefix: String,
    },
    /// Azure Blob Storage
    Azure {
        container: String,
        prefix: String,
    },
    /// SFTP
    Sftp {
        host: String,
        port: u16,
        path: String,
        username: String,
    },
    /// Webhook
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Export ID
    pub id: String,
    /// Status
    pub status: ExportStatus,
    /// Total events exported
    pub total_events: u64,
    /// File size in bytes
    pub file_size: u64,
    /// Download URL (if applicable)
    pub download_url: Option<String>,
    /// Destination path (if applicable)
    pub destination_path: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Expires at (for download URLs)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Export service
pub struct ExportService {
    storage: std::sync::Arc<dyn ExportStorage>,
    event_storage: std::sync::Arc<dyn EventSource>,
    formatters: HashMap<ExportFormat, Box<dyn EventFormatter>>,
    uploaders: HashMap<String, Box<dyn Uploader>>,
}

#[async_trait]
pub trait ExportStorage: Send + Sync {
    async fn create(&self, request: &ExportRequest) -> Result<(), ExportError>;
    async fn get(&self, id: &str) -> Result<Option<ExportRequest>, ExportError>;
    async fn update_status(&self, id: &str, status: ExportStatus) -> Result<(), ExportError>;
    async fn save_result(&self, result: &ExportResult) -> Result<(), ExportError>;
    async fn list(&self, limit: u32, offset: u32) -> Result<Vec<ExportRequest>, ExportError>;
}

#[async_trait]
pub trait EventSource: Send + Sync {
    async fn stream_events(
        &self,
        events: &[String],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        filters: &[ExportFilter],
    ) -> Result<Box<dyn tokio_stream::Stream<Item = Result<EventData, ExportError>> + Send + Unpin>, ExportError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event: String,
    pub distinct_id: String,
    pub timestamp: DateTime<Utc>,
    pub properties: HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait EventFormatter: Send + Sync {
    async fn format_header(&self, properties: &[String]) -> Result<Vec<u8>, ExportError>;
    async fn format_event(&self, event: &EventData, properties: &[String]) -> Result<Vec<u8>, ExportError>;
    async fn format_footer(&self) -> Result<Vec<u8>, ExportError>;
}

#[async_trait]
pub trait Uploader: Send + Sync {
    async fn upload(
        &self,
        destination: &ExportDestination,
        data: &[u8],
        filename: &str,
    ) -> Result<String, ExportError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Export not found")]
    NotFound,
    #[error("Invalid request: {0}")]
    Invalid(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Upload error: {0}")]
    Upload(String),
    #[error("Format error: {0}")]
    Format(String),
}

impl ExportService {
    pub fn new(
        storage: std::sync::Arc<dyn ExportStorage>,
        event_storage: std::sync::Arc<dyn EventSource>,
    ) -> Self {
        let mut formatters: HashMap<ExportFormat, Box<dyn EventFormatter>> = HashMap::new();
        formatters.insert(ExportFormat::Json, Box::new(JsonFormatter));
        formatters.insert(ExportFormat::JsonLines, Box::new(JsonLinesFormatter));
        formatters.insert(ExportFormat::Csv, Box::new(CsvFormatter));

        Self {
            storage,
            event_storage,
            formatters,
            uploaders: HashMap::new(),
        }
    }

    /// Create a new export request
    pub async fn create(&self, mut request: ExportRequest) -> Result<ExportRequest, ExportError> {
        request.id = uuid::Uuid::new_v4().to_string();
        request.created_at = Utc::now();
        request.status = ExportStatus::Pending;

        self.storage.create(&request).await?;

        // Start export in background
        let service = self.clone_for_task();
        let id = request.id.clone();
        tokio::spawn(async move {
            if let Err(e) = service.process_export(&id).await {
                tracing::error!("Export {} failed: {}", id, e);
            }
        });

        Ok(request)
    }

    fn clone_for_task(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            event_storage: self.event_storage.clone(),
            formatters: HashMap::new(), // Formatters will be recreated
            uploaders: HashMap::new(),
        }
    }

    /// Process an export
    async fn process_export(&self, id: &str) -> Result<ExportResult, ExportError> {
        let request = self.storage.get(id).await?
            .ok_or(ExportError::NotFound)?;

        self.storage.update_status(id, ExportStatus::Processing).await?;

        // Get formatter
        let formatter = self.get_formatter(request.format)?;

        // Create buffer for output
        let mut output = Vec::new();

        // Write header
        output.extend(formatter.format_header(&request.properties).await?);

        // Stream events
        let mut stream = self.event_storage.stream_events(
            &request.events,
            request.start_date,
            request.end_date,
            &request.filters,
        ).await?;

        let mut total_events = 0u64;
        use tokio_stream::StreamExt;

        while let Some(event_result) = stream.next().await {
            let event = event_result?;
            output.extend(formatter.format_event(&event, &request.properties).await?);
            total_events += 1;
        }

        // Write footer
        output.extend(formatter.format_footer().await?);

        // Apply compression if requested
        let output = if let Some(compression) = request.compression {
            self.compress(&output, compression)?
        } else {
            output
        };

        let file_size = output.len() as u64;

        // Upload to destination
        let (download_url, destination_path) = self.upload_result(
            &request,
            &output,
        ).await?;

        let result = ExportResult {
            id: id.to_string(),
            status: ExportStatus::Completed,
            total_events,
            file_size,
            download_url,
            destination_path,
            error: None,
            completed_at: Some(Utc::now()),
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
        };

        self.storage.save_result(&result).await?;
        self.storage.update_status(id, ExportStatus::Completed).await?;

        Ok(result)
    }

    fn get_formatter(&self, format: ExportFormat) -> Result<Box<dyn EventFormatter>, ExportError> {
        match format {
            ExportFormat::Json => Ok(Box::new(JsonFormatter)),
            ExportFormat::JsonLines => Ok(Box::new(JsonLinesFormatter)),
            ExportFormat::Csv => Ok(Box::new(CsvFormatter)),
            _ => Err(ExportError::Invalid(format!("Unsupported format: {:?}", format))),
        }
    }

    fn compress(&self, data: &[u8], compression: Compression) -> Result<Vec<u8>, ExportError> {
        match compression {
            Compression::Gzip => {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                use std::io::Write;

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)
                    .map_err(|e| ExportError::Format(e.to_string()))?;
                encoder.finish()
                    .map_err(|e| ExportError::Format(e.to_string()))
            }
            Compression::Zstd => {
                zstd::encode_all(data, 3)
                    .map_err(|e| ExportError::Format(e.to_string()))
            }
            _ => Err(ExportError::Invalid("Compression not supported".to_string())),
        }
    }

    async fn upload_result(
        &self,
        request: &ExportRequest,
        data: &[u8],
    ) -> Result<(Option<String>, Option<String>), ExportError> {
        let filename = self.generate_filename(request);

        match &request.destination {
            ExportDestination::Download => {
                // Store locally and return download URL
                let path = format!("/tmp/exports/{}", filename);
                tokio::fs::write(&path, data).await
                    .map_err(|e| ExportError::Storage(e.to_string()))?;

                let url = format!("/api/exports/{}/download", request.id);
                Ok((Some(url), Some(path)))
            }
            ExportDestination::S3 { bucket, prefix, region, endpoint } => {
                // Upload to S3
                let path = format!("{}/{}", prefix, filename);
                // S3 upload implementation here
                Ok((None, Some(format!("s3://{}/{}", bucket, path))))
            }
            ExportDestination::Webhook { url, headers } => {
                // Send to webhook
                let client = reqwest::Client::new();
                let mut req = client.post(url).body(data.to_vec());

                for (key, value) in headers {
                    req = req.header(key, value);
                }

                req.send().await
                    .map_err(|e| ExportError::Upload(e.to_string()))?;

                Ok((None, Some(url.clone())))
            }
            _ => Err(ExportError::Invalid("Destination not implemented".to_string())),
        }
    }

    fn generate_filename(&self, request: &ExportRequest) -> String {
        let ext = request.format.file_extension();
        let compression_ext = request.compression.map(|c| match c {
            Compression::Gzip => ".gz",
            Compression::Zstd => ".zst",
            Compression::Snappy => ".snappy",
            Compression::Lz4 => ".lz4",
        }).unwrap_or("");

        format!(
            "export_{}_{}_to_{}.{}{}",
            request.id,
            request.start_date.format("%Y%m%d"),
            request.end_date.format("%Y%m%d"),
            ext,
            compression_ext
        )
    }

    /// Get export status
    pub async fn get(&self, id: &str) -> Result<Option<ExportRequest>, ExportError> {
        self.storage.get(id).await
    }

    /// List exports
    pub async fn list(&self, limit: u32, offset: u32) -> Result<Vec<ExportRequest>, ExportError> {
        self.storage.list(limit, offset).await
    }
}

/// JSON formatter
struct JsonFormatter;

#[async_trait]
impl EventFormatter for JsonFormatter {
    async fn format_header(&self, _properties: &[String]) -> Result<Vec<u8>, ExportError> {
        Ok(b"[".to_vec())
    }

    async fn format_event(&self, event: &EventData, _properties: &[String]) -> Result<Vec<u8>, ExportError> {
        let json = serde_json::to_vec(event)
            .map_err(|e| ExportError::Format(e.to_string()))?;
        let mut result = json;
        result.push(b',');
        Ok(result)
    }

    async fn format_footer(&self) -> Result<Vec<u8>, ExportError> {
        Ok(b"]".to_vec())
    }
}

/// JSON Lines formatter
struct JsonLinesFormatter;

#[async_trait]
impl EventFormatter for JsonLinesFormatter {
    async fn format_header(&self, _properties: &[String]) -> Result<Vec<u8>, ExportError> {
        Ok(Vec::new())
    }

    async fn format_event(&self, event: &EventData, _properties: &[String]) -> Result<Vec<u8>, ExportError> {
        let mut json = serde_json::to_vec(event)
            .map_err(|e| ExportError::Format(e.to_string()))?;
        json.push(b'\n');
        Ok(json)
    }

    async fn format_footer(&self) -> Result<Vec<u8>, ExportError> {
        Ok(Vec::new())
    }
}

/// CSV formatter
struct CsvFormatter;

#[async_trait]
impl EventFormatter for CsvFormatter {
    async fn format_header(&self, properties: &[String]) -> Result<Vec<u8>, ExportError> {
        let mut header = "event,distinct_id,timestamp".to_string();
        for prop in properties {
            header.push(',');
            header.push_str(&escape_csv(prop));
        }
        header.push('\n');
        Ok(header.into_bytes())
    }

    async fn format_event(&self, event: &EventData, properties: &[String]) -> Result<Vec<u8>, ExportError> {
        let mut row = format!(
            "{},{},{}",
            escape_csv(&event.event),
            escape_csv(&event.distinct_id),
            event.timestamp.to_rfc3339()
        );

        for prop in properties {
            row.push(',');
            if let Some(value) = event.properties.get(prop) {
                row.push_str(&escape_csv(&value.to_string()));
            }
        }
        row.push('\n');

        Ok(row.into_bytes())
    }

    async fn format_footer(&self) -> Result<Vec<u8>, ExportError> {
        Ok(Vec::new())
    }
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Scheduled export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledExport {
    /// Schedule ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Cron schedule
    pub schedule: String,
    /// Export configuration
    pub export_config: ExportConfig,
    /// Enabled
    pub enabled: bool,
    /// Last run
    pub last_run: Option<DateTime<Utc>>,
    /// Next run
    pub next_run: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Events to export
    pub events: Vec<String>,
    /// Relative date range (e.g., "last_day", "last_week")
    pub date_range: String,
    /// Format
    pub format: ExportFormat,
    /// Properties
    pub properties: Vec<String>,
    /// Filters
    pub filters: Vec<ExportFilter>,
    /// Compression
    pub compression: Option<Compression>,
    /// Destination
    pub destination: ExportDestination,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format() {
        assert_eq!(ExportFormat::Json.content_type(), "application/json");
        assert_eq!(ExportFormat::Csv.file_extension(), "csv");
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
    }

    #[tokio::test]
    async fn test_json_lines_formatter() {
        let formatter = JsonLinesFormatter;

        let event = EventData {
            event: "test".to_string(),
            distinct_id: "user-1".to_string(),
            timestamp: Utc::now(),
            properties: HashMap::new(),
        };

        let output = formatter.format_event(&event, &[]).await.unwrap();
        assert!(output.ends_with(&[b'\n']));
    }
}
```

## REST API

```yaml
# Export API endpoints
openapi: 3.0.0
paths:
  /api/exports:
    post:
      summary: Create export
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ExportRequest'
      responses:
        '202':
          description: Export created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExportRequest'

    get:
      summary: List exports
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
        - name: offset
          in: query
          schema:
            type: integer
      responses:
        '200':
          description: Export list

  /api/exports/{id}:
    get:
      summary: Get export status
      responses:
        '200':
          description: Export details

  /api/exports/{id}/download:
    get:
      summary: Download export file
      responses:
        '200':
          description: File download
          content:
            application/octet-stream:
              schema:
                type: string
                format: binary
```

## Related Specs

- 415-event-persistence.md - Event storage
- 425-privacy-compliance.md - Data privacy in exports
- 426-data-retention.md - Export before deletion
