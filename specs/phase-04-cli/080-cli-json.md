# Spec 080: JSON Output Mode

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 080
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 079-cli-output
- **Estimated Context**: ~8%

## Objective

Implement comprehensive JSON output mode for all CLI commands, enabling machine-readable output for scripting, automation, and integration with other tools.

## Acceptance Criteria

- [ ] `--format json` flag produces valid JSON output
- [ ] All commands support JSON output mode
- [ ] JSON schemas documented for each output type
- [ ] Pretty-printed JSON by default, compact with `--compact`
- [ ] NDJSON (newline-delimited JSON) for streaming output
- [ ] Error output as structured JSON
- [ ] Consistent field naming conventions

## Implementation Details

### src/output/json.rs

```rust
//! JSON output formatting utilities.

use std::io::{self, Write};

use serde::Serialize;
use serde_json::{json, Map, Value};

/// JSON output configuration
#[derive(Debug, Clone, Copy)]
pub struct JsonConfig {
    /// Pretty print with indentation
    pub pretty: bool,
    /// Include null fields
    pub include_null: bool,
    /// Streaming mode (NDJSON)
    pub streaming: bool,
}

impl Default for JsonConfig {
    fn default() -> Self {
        Self {
            pretty: true,
            include_null: false,
            streaming: false,
        }
    }
}

impl JsonConfig {
    pub fn compact() -> Self {
        Self {
            pretty: false,
            ..Default::default()
        }
    }

    pub fn streaming() -> Self {
        Self {
            pretty: false,
            streaming: true,
            ..Default::default()
        }
    }
}

/// JSON output writer
pub struct JsonOutput<W: Write> {
    writer: W,
    config: JsonConfig,
}

impl<W: Write> JsonOutput<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            config: JsonConfig::default(),
        }
    }

    pub fn with_config(writer: W, config: JsonConfig) -> Self {
        Self { writer, config }
    }

    /// Write a single value
    pub fn write<T: Serialize>(&mut self, value: &T) -> io::Result<()> {
        let json_str = if self.config.pretty {
            serde_json::to_string_pretty(value)
        } else {
            serde_json::to_string(value)
        }
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        writeln!(self.writer, "{json_str}")
    }

    /// Write multiple values (for streaming/NDJSON)
    pub fn write_stream<T, I>(&mut self, items: I) -> io::Result<()>
    where
        T: Serialize,
        I: IntoIterator<Item = T>,
    {
        for item in items {
            let json_str = serde_json::to_string(&item)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writeln!(self.writer, "{json_str}")?;
        }
        Ok(())
    }

    /// Flush the writer
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl JsonOutput<io::Stdout> {
    pub fn stdout() -> Self {
        Self::new(io::stdout())
    }
}

/// Envelope wrapper for API-style JSON output
#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonMetadata>,
}

impl<T> JsonEnvelope<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: None,
        }
    }

    pub fn error(error: JsonError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: JsonMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Structured error for JSON output
#[derive(Debug, Serialize)]
pub struct JsonError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl JsonError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            help: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Metadata for JSON responses
#[derive(Debug, Serialize)]
pub struct JsonMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
}

impl JsonMetadata {
    pub fn new() -> Self {
        Self {
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            duration_ms: None,
            pagination: None,
        }
    }

    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration_ms = Some(duration.as_millis() as u64);
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationMeta) -> Self {
        self.pagination = Some(pagination);
        self
    }
}

impl Default for JsonMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

impl PaginationMeta {
    pub fn new(total: usize, offset: usize, limit: usize) -> Self {
        Self {
            total,
            offset,
            limit,
            has_more: offset + limit < total,
        }
    }
}

/// List response wrapper
#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub count: usize,
}

impl<T> ListResponse<T> {
    pub fn new(items: Vec<T>) -> Self {
        let count = items.len();
        Self { items, count }
    }
}

impl<T> From<Vec<T>> for ListResponse<T> {
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

/// Trait for types with JSON schema
pub trait JsonSchema {
    /// Return the JSON schema for this type
    fn json_schema() -> Value;
}

/// Common output types with schemas
pub mod schemas {
    use super::*;

    /// Tool information schema
    pub fn tool_info() -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "version": { "type": "string" },
                "description": { "type": "string" },
                "enabled": { "type": "boolean" },
                "source": { "type": "string" },
                "categories": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["name", "version", "description", "enabled"]
        })
    }

    /// Backend information schema
    pub fn backend_info() -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "type": { "type": "string" },
                "is_default": { "type": "boolean" },
                "base_url": { "type": "string" },
                "models": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["name", "type", "is_default"]
        })
    }

    /// Config entry schema
    pub fn config_entry() -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": { "type": "string" },
                "value": {},
                "source": { "type": "string" },
                "mutable": { "type": "boolean" }
            },
            "required": ["key", "value"]
        })
    }

    /// Doctor check result schema
    pub fn doctor_check() -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "status": {
                    "type": "string",
                    "enum": ["pass", "warn", "fail", "skip"]
                },
                "message": { "type": "string" },
                "details": { "type": "object" }
            },
            "required": ["name", "status"]
        })
    }
}

/// Convert from CLI error to JSON error
impl From<&crate::error::CliError> for JsonError {
    fn from(err: &crate::error::CliError) -> Self {
        use crate::error::CliError;

        let (code, message) = match err {
            CliError::Config(e) => ("CONFIG_ERROR", e.to_string()),
            CliError::Io(e) => ("IO_ERROR", e.to_string()),
            CliError::InvalidArgument(msg) => ("INVALID_ARGUMENT", msg.clone()),
            CliError::CommandFailed(msg) => ("COMMAND_FAILED", msg.clone()),
            CliError::Network(msg) => ("NETWORK_ERROR", msg.clone()),
            CliError::Validation(msg) => ("VALIDATION_ERROR", msg.clone()),
            CliError::Other(e) => ("ERROR", e.to_string()),
        };

        JsonError::new(code, message)
    }
}
```

### JSON Output Wrapper for Commands

```rust
//! Command execution with JSON output support.

use std::time::Instant;

use serde::Serialize;

use crate::cli::{CommandContext, OutputFormat};
use crate::error::CliError;
use crate::output::json::{JsonEnvelope, JsonError, JsonMetadata};

/// Execute a command and format output as JSON if requested
pub async fn execute_with_json<T, F, Fut>(
    ctx: &CommandContext,
    f: F,
) -> Result<(), CliError>
where
    T: Serialize + std::fmt::Display,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, CliError>>,
{
    let start = Instant::now();
    let result = f().await;
    let duration = start.elapsed();

    match ctx.format {
        OutputFormat::Json => {
            let metadata = JsonMetadata::new().with_duration(duration);

            let envelope = match result {
                Ok(data) => JsonEnvelope::success(data).with_metadata(metadata),
                Err(ref e) => JsonEnvelope::<T>::error(JsonError::from(e))
                    .with_metadata(metadata),
            };

            println!("{}", serde_json::to_string_pretty(&envelope)?);

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
        OutputFormat::Text => {
            match result {
                Ok(data) => {
                    println!("{data}");
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }
}

/// Print error as JSON or text based on context
pub fn print_error(ctx: &CommandContext, error: &CliError) {
    match ctx.format {
        OutputFormat::Json => {
            let envelope = JsonEnvelope::<()>::error(JsonError::from(error));
            if let Ok(json) = serde_json::to_string_pretty(&envelope) {
                eprintln!("{json}");
            } else {
                eprintln!("{{\"success\":false,\"error\":{{\"message\":\"{error}\"}}}}");
            }
        }
        OutputFormat::Text => {
            eprintln!("Error: {error}");
        }
    }
}
```

### Example Command with JSON Output

```rust
//! Example command demonstrating JSON output.

use async_trait::async_trait;
use clap::Args;
use serde::Serialize;

use crate::cli::CommandContext;
use crate::commands::ExecuteWithOutput;
use crate::error::CliError;

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Show detailed status
    #[arg(long)]
    pub detailed: bool,
}

#[derive(Debug, Serialize)]
pub struct StatusOutput {
    pub project_name: String,
    pub version: String,
    pub tools_count: usize,
    pub backends_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<StatusDetails>,
}

#[derive(Debug, Serialize)]
pub struct StatusDetails {
    pub tools: Vec<String>,
    pub backends: Vec<String>,
    pub config_path: String,
}

impl std::fmt::Display for StatusOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Project: {}", self.project_name)?;
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Tools: {}", self.tools_count)?;
        writeln!(f, "Backends: {}", self.backends_count)?;

        if let Some(details) = &self.details {
            writeln!(f, "\nTools:")?;
            for tool in &details.tools {
                writeln!(f, "  - {tool}")?;
            }
            writeln!(f, "\nBackends:")?;
            for backend in &details.backends {
                writeln!(f, "  - {backend}")?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ExecuteWithOutput for StatusArgs {
    type Output = StatusOutput;

    async fn execute(&self, ctx: &CommandContext) -> Result<Self::Output, CliError> {
        let tools = ctx.config.tools.list().await?;
        let backends = ctx.config.backends.list();

        let details = if self.detailed {
            Some(StatusDetails {
                tools: tools.iter().map(|t| t.name.clone()).collect(),
                backends: backends.iter().map(|b| b.name.clone()).collect(),
                config_path: ctx.config.path().display().to_string(),
            })
        } else {
            None
        };

        Ok(StatusOutput {
            project_name: ctx.config.project.name.clone(),
            version: ctx.config.project.version.to_string(),
            tools_count: tools.len(),
            backends_count: backends.len(),
            details,
        })
    }
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_envelope_success() {
        let envelope = JsonEnvelope::success("test data");
        let json = serde_json::to_string(&envelope).unwrap();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test data\""));
    }

    #[test]
    fn test_json_envelope_error() {
        let error = JsonError::new("TEST_ERROR", "Test error message");
        let envelope = JsonEnvelope::<()>::error(error);
        let json = serde_json::to_string(&envelope).unwrap();

        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"code\":\"TEST_ERROR\""));
    }

    #[test]
    fn test_json_metadata() {
        let meta = JsonMetadata::new()
            .with_duration(std::time::Duration::from_millis(150));
        let json = serde_json::to_string(&meta).unwrap();

        assert!(json.contains("\"duration_ms\":150"));
        assert!(json.contains("\"version\""));
    }

    #[test]
    fn test_list_response() {
        let list: ListResponse<String> = vec!["a".to_string(), "b".to_string()].into();
        let json = serde_json::to_string(&list).unwrap();

        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"items\""));
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(100, 20, 10);
        assert!(meta.has_more);

        let meta = PaginationMeta::new(25, 20, 10);
        assert!(!meta.has_more);
    }

    #[test]
    fn test_json_output_streaming() {
        let mut buffer = Vec::new();
        let mut output = JsonOutput::with_config(&mut buffer, JsonConfig::streaming());

        output.write_stream(&["item1", "item2", "item3"]).unwrap();

        let content = String::from_utf8(buffer).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **079-cli-output.md**: Output formatting system
- **091-cli-errors.md**: Error output formatting
