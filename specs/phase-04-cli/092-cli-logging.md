# Spec 092: CLI Logging Integration

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 092
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 003-logging
- **Estimated Context**: ~8%

## Objective

Implement logging integration for the CLI, connecting the tracing framework with CLI output, supporting log levels, file logging, and structured logging output.

## Acceptance Criteria

- [x] Log level control via CLI flags (-v, -vv, -vvv)
- [x] Quiet mode suppresses non-error logs
- [x] Log output to file
- [x] JSON log format option
- [x] Color-coded log levels
- [x] Integration with tracing spans
- [x] Progress indicator compatibility
- [x] Debug mode with full traces

## Implementation Details

### src/logging/mod.rs

```rust
//! CLI logging integration with tracing.

mod format;
mod layer;
mod writer;

pub use format::LogFormat;
pub use layer::CliLayer;
pub use writer::LogWriter;

use std::path::PathBuf;

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

use crate::output::color::ColorMode;

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Verbosity level (0-3)
    pub verbosity: u8,
    /// Quiet mode
    pub quiet: bool,
    /// Color mode
    pub color: ColorMode,
    /// Log to file
    pub log_file: Option<PathBuf>,
    /// JSON format
    pub json: bool,
    /// Show timestamps
    pub timestamps: bool,
    /// Show targets (module paths)
    pub targets: bool,
    /// Show thread IDs
    pub thread_ids: bool,
    /// Show span information
    pub spans: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            verbosity: 0,
            quiet: false,
            color: ColorMode::Auto,
            log_file: None,
            json: false,
            timestamps: false,
            targets: false,
            thread_ids: false,
            spans: false,
        }
    }
}

impl LogConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn verbosity(mut self, level: u8) -> Self {
        self.verbosity = level;
        self
    }

    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn color(mut self, color: ColorMode) -> Self {
        self.color = color;
        self
    }

    pub fn log_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.log_file = Some(path.into());
        self
    }

    pub fn json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    /// Get the log level filter string
    pub fn level_filter(&self) -> String {
        if self.quiet {
            return "error".to_string();
        }

        match self.verbosity {
            0 => "warn".to_string(),
            1 => "info".to_string(),
            2 => "debug".to_string(),
            _ => "trace".to_string(),
        }
    }
}

/// Initialize logging with the given configuration
pub fn init(config: &LogConfig) {
    // Build the filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level_filter()));

    // Build the subscriber
    let subscriber = tracing_subscriber::registry().with(filter);

    if config.json {
        // JSON format for structured logging
        let json_layer = fmt::layer()
            .json()
            .with_span_list(config.spans)
            .with_current_span(config.spans)
            .with_thread_ids(config.thread_ids)
            .with_file(config.verbosity >= 2)
            .with_line_number(config.verbosity >= 2);

        subscriber.with(json_layer).init();
    } else {
        // Human-readable format
        let use_color = config.color.should_color();

        let fmt_layer = fmt::layer()
            .with_ansi(use_color)
            .with_level(true)
            .with_target(config.targets || config.verbosity >= 2)
            .with_thread_ids(config.thread_ids)
            .with_file(config.verbosity >= 3)
            .with_line_number(config.verbosity >= 3);

        let fmt_layer = if config.timestamps {
            fmt_layer.with_timer(fmt::time::UtcTime::rfc_3339()).boxed()
        } else {
            fmt_layer.without_time().boxed()
        };

        // Add file logging if configured
        if let Some(path) = &config.log_file {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Failed to open log file");

            let file_layer = fmt::layer()
                .with_writer(file)
                .with_ansi(false)
                .with_timer(fmt::time::UtcTime::rfc_3339());

            subscriber
                .with(fmt_layer)
                .with(file_layer)
                .init();
        } else {
            subscriber.with(fmt_layer).init();
        }
    }
}

/// Initialize logging from CLI flags
pub fn init_from_cli(verbosity: u8, quiet: bool, color: clap::ColorChoice) {
    let config = LogConfig::new()
        .verbosity(verbosity)
        .quiet(quiet)
        .color(color.into());

    init(&config);
}
```

### src/logging/format.rs

```rust
//! Custom log formatting.

use std::fmt;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext,
    FormattedFields,
};
use tracing_subscriber::registry::LookupSpan;

use crate::output::color::{ColorMode, Styled, Color};

/// Custom log format for CLI output
pub struct CliLogFormat {
    color_mode: ColorMode,
    show_target: bool,
    show_timestamp: bool,
}

impl CliLogFormat {
    pub fn new() -> Self {
        Self {
            color_mode: ColorMode::Auto,
            show_target: false,
            show_timestamp: false,
        }
    }

    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    pub fn show_target(mut self, show: bool) -> Self {
        self.show_target = show;
        self
    }

    pub fn show_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    fn format_level(&self, level: &Level) -> impl fmt::Display {
        let (text, color) = match *level {
            Level::ERROR => ("ERROR", Color::Red),
            Level::WARN => ("WARN ", Color::Yellow),
            Level::INFO => ("INFO ", Color::Green),
            Level::DEBUG => ("DEBUG", Color::Blue),
            Level::TRACE => ("TRACE", Color::Magenta),
        };

        Styled::new(text)
            .with_color_mode(self.color_mode)
            .fg(color)
            .bold()
    }
}

impl Default for CliLogFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, N> FormatEvent<S, N> for CliLogFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Timestamp
        if self.show_timestamp {
            let now = chrono::Utc::now();
            write!(
                writer,
                "{} ",
                Styled::new(now.format("%H:%M:%S%.3f").to_string())
                    .with_color_mode(self.color_mode)
                    .fg(Color::BrightBlack)
            )?;
        }

        // Level
        write!(writer, "{} ", self.format_level(event.metadata().level()))?;

        // Target
        if self.show_target {
            write!(
                writer,
                "{}: ",
                Styled::new(event.metadata().target())
                    .with_color_mode(self.color_mode)
                    .fg(Color::BrightBlack)
            )?;
        }

        // Span context
        if let Some(scope) = ctx.event_scope() {
            let mut seen = false;
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;
                seen = true;

                let ext = span.extensions();
                if let Some(fields) = ext.get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write!(writer, "{{{fields}}}")?;
                    }
                }
                write!(writer, ": ")?;
            }
            if seen {
                write!(writer, " ")?;
            }
        }

        // Event fields
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

/// Log format selection
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogFormat {
    #[default]
    Pretty,
    Compact,
    Json,
    Full,
}

impl LogFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pretty" => Some(Self::Pretty),
            "compact" => Some(Self::Compact),
            "json" => Some(Self::Json),
            "full" => Some(Self::Full),
            _ => None,
        }
    }
}
```

### src/logging/layer.rs

```rust
//! Custom tracing layer for CLI output.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tracing::{Event, Subscriber, Level};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use crate::output::color::{Styled, Color, ColorMode};

/// CLI-aware tracing layer that handles progress indicators
pub struct CliLayer {
    /// Whether a progress indicator is active
    progress_active: Arc<AtomicBool>,
    /// Color mode
    color_mode: ColorMode,
    /// Minimum level to display
    min_level: Level,
}

impl CliLayer {
    pub fn new() -> Self {
        Self {
            progress_active: Arc::new(AtomicBool::new(false)),
            color_mode: ColorMode::Auto,
            min_level: Level::WARN,
        }
    }

    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    pub fn min_level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    /// Get a handle to control progress state
    pub fn progress_handle(&self) -> ProgressHandle {
        ProgressHandle {
            active: self.progress_active.clone(),
        }
    }

    fn should_log(&self, level: &Level) -> bool {
        *level <= self.min_level
    }
}

impl Default for CliLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Subscriber> Layer<S> for CliLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if !self.should_log(event.metadata().level()) {
            return;
        }

        // If progress is active, clear the line first
        if self.progress_active.load(Ordering::Relaxed) {
            eprint!("\r\x1b[K");
        }

        // Format and print the event
        let level = event.metadata().level();
        let level_str = match *level {
            Level::ERROR => Styled::new("error").fg(Color::Red).bold(),
            Level::WARN => Styled::new("warn").fg(Color::Yellow).bold(),
            Level::INFO => Styled::new("info").fg(Color::Green).bold(),
            Level::DEBUG => Styled::new("debug").fg(Color::Blue).bold(),
            Level::TRACE => Styled::new("trace").fg(Color::Magenta).bold(),
        }
        .with_color_mode(self.color_mode);

        // Extract message from event
        struct MessageVisitor(String);

        impl tracing::field::Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{:?}", value);
                }
            }

            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    self.0 = value.to_string();
                }
            }
        }

        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        eprintln!("{level_str}: {}", visitor.0);
    }
}

/// Handle to control progress indicator state
pub struct ProgressHandle {
    active: Arc<AtomicBool>,
}

impl ProgressHandle {
    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}
```

### src/logging/writer.rs

```rust
//! Log writers for different outputs.

use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufWriter};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Log writer that can write to multiple destinations
pub struct LogWriter {
    writers: Vec<Box<dyn Write + Send>>,
}

impl LogWriter {
    pub fn new() -> Self {
        Self { writers: vec![] }
    }

    pub fn add_stderr(mut self) -> Self {
        self.writers.push(Box::new(io::stderr()));
        self
    }

    pub fn add_file(mut self, path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        self.writers.push(Box::new(BufWriter::new(file)));
        Ok(self)
    }

    pub fn add_buffer(mut self, buffer: Arc<Mutex<Vec<u8>>>) -> Self {
        self.writers.push(Box::new(BufferWriter(buffer)));
        self
    }
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for writer in &mut self.writers {
            writer.write_all(buf)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        for writer in &mut self.writers {
            writer.flush()?;
        }
        Ok(())
    }
}

impl Default for LogWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Writer to a shared buffer
struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buffer = self.0.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "lock poisoned")
        })?;
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Rotating log writer
pub struct RotatingLogWriter {
    path: std::path::PathBuf,
    max_size: u64,
    max_files: usize,
    current_file: Option<BufWriter<File>>,
    current_size: u64,
}

impl RotatingLogWriter {
    pub fn new(path: impl AsRef<Path>, max_size: u64, max_files: usize) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            max_size,
            max_files,
            current_file: None,
            current_size: 0,
        }
    }

    fn rotate(&mut self) -> io::Result<()> {
        // Close current file
        self.current_file = None;

        // Rotate existing files
        for i in (1..self.max_files).rev() {
            let from = self.path.with_extension(format!("log.{}", i));
            let to = self.path.with_extension(format!("log.{}", i + 1));

            if from.exists() {
                std::fs::rename(&from, &to)?;
            }
        }

        // Move current log to .1
        if self.path.exists() {
            let rotated = self.path.with_extension("log.1");
            std::fs::rename(&self.path, &rotated)?;
        }

        // Open new file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        self.current_file = Some(BufWriter::new(file));
        self.current_size = 0;

        Ok(())
    }

    fn ensure_file(&mut self) -> io::Result<()> {
        if self.current_file.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;

            self.current_size = file.metadata()?.len();
            self.current_file = Some(BufWriter::new(file));
        }
        Ok(())
    }
}

impl Write for RotatingLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.ensure_file()?;

        if self.current_size + buf.len() as u64 > self.max_size {
            self.rotate()?;
        }

        let n = self.current_file.as_mut().unwrap().write(buf)?;
        self.current_size += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.current_file {
            file.flush()?;
        }
        Ok(())
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
    fn test_log_config_level_filter() {
        let config = LogConfig::new().verbosity(0);
        assert_eq!(config.level_filter(), "warn");

        let config = LogConfig::new().verbosity(1);
        assert_eq!(config.level_filter(), "info");

        let config = LogConfig::new().verbosity(2);
        assert_eq!(config.level_filter(), "debug");

        let config = LogConfig::new().verbosity(3);
        assert_eq!(config.level_filter(), "trace");
    }

    #[test]
    fn test_log_config_quiet() {
        let config = LogConfig::new().quiet(true);
        assert_eq!(config.level_filter(), "error");
    }

    #[test]
    fn test_log_writer_buffer() {
        use std::sync::{Arc, Mutex};

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut writer = LogWriter::new().add_buffer(buffer.clone());

        writer.write_all(b"test message").unwrap();
        writer.flush().unwrap();

        let content = buffer.lock().unwrap();
        assert_eq!(&*content, b"test message");
    }
}
```

## Related Specs

- **003-logging.md**: Core logging framework
- **076-cli-crate.md**: Base CLI structure
- **082-cli-progress.md**: Progress indicator integration
