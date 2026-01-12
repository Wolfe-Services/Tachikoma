//! Logging infrastructure for Tachikoma.

use std::io;
use std::path::PathBuf;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
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

impl From<LogLevel> for tracing_subscriber::filter::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
            LogLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
            LogLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
            LogLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
            LogLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
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

        if let Ok(file_path) = std::env::var("TACHIKOMA_LOG_FILE") {
            config.file_path = Some(PathBuf::from(file_path));
        }

        if let Ok(source_location) = std::env::var("TACHIKOMA_LOG_SOURCE") {
            config.source_location = source_location.to_lowercase() == "true" || source_location == "1";
        }

        if let Ok(span_events) = std::env::var("TACHIKOMA_LOG_SPANS") {
            config.span_events = span_events.to_lowercase() == "true" || span_events == "1";
        }

        config
    }
}

/// Initialize logging with the given configuration.
pub fn init(config: LogConfig) -> Result<(), LogError> {
    let level_str = match config.level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug", 
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    };
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level_str));

    let get_span_events = || {
        if config.span_events {
            FmtSpan::NEW | FmtSpan::CLOSE
        } else {
            FmtSpan::NONE
        }
    };

    // Build the registry
    let registry = tracing_subscriber::registry().with(filter);

    match (&config.format, &config.file_path) {
        (LogFormat::Pretty, None) => {
            // Pretty format to stderr only
            let layer = fmt::layer()
                .with_ansi(true)
                .with_target(true)
                .with_file(config.source_location)
                .with_line_number(config.source_location)
                .with_span_events(get_span_events());

            registry
                .with(layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
        (LogFormat::Pretty, Some(file_path)) => {
            // Pretty format to stderr and file
            let stderr_layer = fmt::layer()
                .with_ansi(true)
                .with_target(true)
                .with_file(config.source_location)
                .with_line_number(config.source_location)
                .with_span_events(get_span_events());

            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;

            let file_layer = fmt::layer()
                .with_writer(file)
                .with_ansi(false)
                .with_target(true)
                .with_file(config.source_location)
                .with_line_number(config.source_location)
                .with_span_events(get_span_events());

            registry
                .with(stderr_layer)
                .with(file_layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
        (LogFormat::Compact, None) => {
            // Compact format to stderr only
            let layer = fmt::layer()
                .compact()
                .with_ansi(true)
                .with_span_events(get_span_events());

            registry
                .with(layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
        (LogFormat::Compact, Some(file_path)) => {
            // Compact format to stderr and file
            let stderr_layer = fmt::layer()
                .compact()
                .with_ansi(true)
                .with_span_events(get_span_events());

            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;

            let file_layer = fmt::layer()
                .compact()
                .with_writer(file)
                .with_ansi(false)
                .with_span_events(get_span_events());

            registry
                .with(stderr_layer)
                .with(file_layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
        (LogFormat::Json, None) => {
            // JSON format to stderr only
            let layer = fmt::layer()
                .json()
                .with_span_events(get_span_events());

            registry
                .with(layer)
                .try_init()
                .map_err(|e| LogError::InitError(e.to_string()))?;
        }
        (LogFormat::Json, Some(file_path)) => {
            // JSON format to stderr and file
            let stderr_layer = fmt::layer()
                .json()
                .with_span_events(get_span_events());

            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;

            let file_layer = fmt::layer()
                .json()
                .with_writer(file)
                .with_span_events(get_span_events());

            registry
                .with(stderr_layer)
                .with(file_layer)
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

/// Convenience macros re-exported from tracing.
pub use tracing::{debug, error, info, trace, warn};

/// Distributed tracing utilities.
pub mod spans;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_log_level_parse() {
        assert!(matches!(LogLevel::parse("info"), Some(LogLevel::Info)));
        assert!(matches!(LogLevel::parse("DEBUG"), Some(LogLevel::Debug)));
        assert!(matches!(LogLevel::parse("Warning"), Some(LogLevel::Warn)));
        assert!(matches!(LogLevel::parse("warn"), Some(LogLevel::Warn)));
        assert!(matches!(LogLevel::parse("error"), Some(LogLevel::Error)));
        assert!(matches!(LogLevel::parse("trace"), Some(LogLevel::Trace)));
        assert!(matches!(LogLevel::parse("invalid"), None));
    }

    #[test]
    fn test_log_level_from() {
        use tracing_subscriber::filter::LevelFilter;
        assert_eq!(LevelFilter::from(LogLevel::Trace), LevelFilter::TRACE);
        assert_eq!(LevelFilter::from(LogLevel::Debug), LevelFilter::DEBUG);
        assert_eq!(LevelFilter::from(LogLevel::Info), LevelFilter::INFO);
        assert_eq!(LevelFilter::from(LogLevel::Warn), LevelFilter::WARN);
        assert_eq!(LevelFilter::from(LogLevel::Error), LevelFilter::ERROR);
    }

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert!(matches!(config.level, LogLevel::Info));
        assert!(matches!(config.format, LogFormat::Pretty));
        assert!(config.file_path.is_none());
        assert!(config.timestamps);
        assert!(!config.source_location);
        assert!(!config.span_events);
    }

    #[test]
    fn test_config_from_env() {
        // Save original env vars
        let original_log_level = env::var("TACHIKOMA_LOG_LEVEL").ok();
        let original_log_format = env::var("TACHIKOMA_LOG_FORMAT").ok();
        let original_log_file = env::var("TACHIKOMA_LOG_FILE").ok();
        let original_log_source = env::var("TACHIKOMA_LOG_SOURCE").ok();
        let original_log_spans = env::var("TACHIKOMA_LOG_SPANS").ok();

        // Set test env vars
        env::set_var("TACHIKOMA_LOG_LEVEL", "debug");
        env::set_var("TACHIKOMA_LOG_FORMAT", "json");
        env::set_var("TACHIKOMA_LOG_FILE", "/tmp/test.log");
        env::set_var("TACHIKOMA_LOG_SOURCE", "true");
        env::set_var("TACHIKOMA_LOG_SPANS", "1");

        let config = LogConfig::from_env();
        assert!(matches!(config.level, LogLevel::Debug));
        assert!(matches!(config.format, LogFormat::Json));
        assert_eq!(config.file_path.unwrap(), PathBuf::from("/tmp/test.log"));
        assert!(config.source_location);
        assert!(config.span_events);

        // Clean up env vars
        env::remove_var("TACHIKOMA_LOG_LEVEL");
        env::remove_var("TACHIKOMA_LOG_FORMAT");
        env::remove_var("TACHIKOMA_LOG_FILE");
        env::remove_var("TACHIKOMA_LOG_SOURCE");
        env::remove_var("TACHIKOMA_LOG_SPANS");

        // Restore original env vars if they existed
        if let Some(val) = original_log_level {
            env::set_var("TACHIKOMA_LOG_LEVEL", val);
        }
        if let Some(val) = original_log_format {
            env::set_var("TACHIKOMA_LOG_FORMAT", val);
        }
        if let Some(val) = original_log_file {
            env::set_var("TACHIKOMA_LOG_FILE", val);
        }
        if let Some(val) = original_log_source {
            env::set_var("TACHIKOMA_LOG_SOURCE", val);
        }
        if let Some(val) = original_log_spans {
            env::set_var("TACHIKOMA_LOG_SPANS", val);
        }
    }

    #[test]
    fn test_rust_log_fallback() {
        // Save original env vars
        let original_tachikoma_log_level = env::var("TACHIKOMA_LOG_LEVEL").ok();
        let original_rust_log = env::var("RUST_LOG").ok();

        // Ensure TACHIKOMA_LOG_LEVEL is not set
        env::remove_var("TACHIKOMA_LOG_LEVEL");
        env::set_var("RUST_LOG", "warn");

        let config = LogConfig::from_env();
        assert!(matches!(config.level, LogLevel::Warn));

        // Clean up
        env::remove_var("RUST_LOG");

        // Restore original env vars if they existed
        if let Some(val) = original_tachikoma_log_level {
            env::set_var("TACHIKOMA_LOG_LEVEL", val);
        }
        if let Some(val) = original_rust_log {
            env::set_var("RUST_LOG", val);
        }
    }
}