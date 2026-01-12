# 026 - Logging Infrastructure

**Phase:** 1 - Core Common Crates
**Spec ID:** 026
**Status:** Planned
**Dependencies:** 011-common-core-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Set up structured logging using the `tracing` crate with configurable outputs, log levels, and formatters.

---

## Acceptance Criteria

- [ ] tracing subscriber configuration
- [ ] Multiple output targets (stderr, file)
- [ ] JSON and pretty formatters
- [ ] Log level filtering
- [ ] Environment-based configuration

---

## Implementation Details

### 1. Logging Module (crates/tachikoma-common-log/src/lib.rs)

```rust
//! Logging infrastructure for Tachikoma.

use std::io;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Logging configuration.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Minimum log level.
    pub level: LogLevel,
    /// Output format.
    pub format: LogFormat,
    /// Log file path (if file logging enabled).
    pub file_path: Option<PathBuf>,
    /// Include timestamps.
    pub timestamps: bool,
    /// Include source location.
    pub source_location: bool,
    /// Include span events.
    pub span_events: bool,
}

/// Log level.
#[derive(Debug, Clone, Copy, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl LogLevel {
    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

/// Log output format.
#[derive(Debug, Clone, Copy, Default)]
pub enum LogFormat {
    /// Human-readable pretty format.
    #[default]
    Pretty,
    /// Compact single-line format.
    Compact,
    /// JSON structured format.
    Json,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            format: LogFormat::default(),
            file_path: None,
            timestamps: true,
            source_location: false,
            span_events: false,
        }
    }
}

impl LogConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(level) = std::env::var("TACHIKOMA_LOG_LEVEL") {
            if let Some(l) = LogLevel::parse(&level) {
                config.level = l;
            }
        } else if let Ok(level) = std::env::var("RUST_LOG") {
            if let Some(l) = LogLevel::parse(&level) {
                config.level = l;
            }
        }

        if let Ok(format) = std::env::var("TACHIKOMA_LOG_FORMAT") {
            config.format = match format.to_lowercase().as_str() {
                "json" => LogFormat::Json,
                "compact" => LogFormat::Compact,
                _ => LogFormat::Pretty,
            };
        }

        config
    }
}

/// Initialize logging with the given configuration.
pub fn init(config: LogConfig) -> Result<(), LogError> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{:?}", Level::from(config.level))));

    let span_events = if config.span_events {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    match config.format {
        LogFormat::Pretty => {
            let layer = fmt::layer()
                .with_ansi(true)
                .with_target(true)
                .with_file(config.source_location)
                .with_line_number(config.source_location)
                .with_span_events(span_events);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
        LogFormat::Compact => {
            let layer = fmt::layer()
                .compact()
                .with_ansi(true)
                .with_span_events(span_events);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
        LogFormat::Json => {
            let layer = fmt::layer()
                .json()
                .with_span_events(span_events);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
    }

    Ok(())
}

/// Logging errors.
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("failed to initialize logging: {0}")]
    InitError(String),

    #[error("failed to open log file: {0}")]
    FileError(#[from] io::Error),
}

/// Convenience macros re-exported.
pub use tracing::{debug, error, info, trace, warn};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_parse() {
        assert!(matches!(LogLevel::parse("info"), Some(LogLevel::Info)));
        assert!(matches!(LogLevel::parse("DEBUG"), Some(LogLevel::Debug)));
        assert!(matches!(LogLevel::parse("Warning"), Some(LogLevel::Warn)));
    }
}
```

### 2. Crate Setup

```toml
[package]
name = "tachikoma-common-log"
version.workspace = true
edition.workspace = true

[dependencies]
tracing.workspace = true
tracing-subscriber = { workspace = true, features = ["env-filter", "json"] }
thiserror.workspace = true
```

---

## Testing Requirements

1. Logging initializes without error
2. Log level filtering works
3. Different formats produce correct output
4. Environment variables are respected

---

## Related Specs

- Depends on: [011-common-core-types.md](011-common-core-types.md)
- Next: [027-tracing-setup.md](027-tracing-setup.md)
