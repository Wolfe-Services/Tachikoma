# Spec 091: CLI Error Messages

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 091
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 002-error
- **Estimated Context**: ~10%

## Objective

Implement comprehensive error handling and user-friendly error messages for the CLI, including error categorization, helpful suggestions, and proper error formatting.

## Acceptance Criteria

- [x] Consistent error message format
- [x] Error categorization with codes
- [x] Contextual help suggestions
- [x] Color-coded error output
- [x] Error chain display
- [x] JSON error output mode
- [x] Exit codes for different error types
- [x] Debug mode with stack traces

## Implementation Details

### src/error.rs

```rust
//! CLI error handling and formatting.

use std::fmt;
use std::io;
use std::process::ExitCode;

use thiserror::Error;

/// CLI error type with rich context
#[derive(Debug, Error)]
pub enum CliError {
    #[error("{message}")]
    Config {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        hint: Option<String>,
    },

    #[error("{message}")]
    Io {
        message: String,
        #[source]
        source: io::Error,
        path: Option<std::path::PathBuf>,
    },

    #[error("{message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        url: Option<String>,
    },

    #[error("{message}")]
    Validation {
        message: String,
        field: Option<String>,
        expected: Option<String>,
        actual: Option<String>,
    },

    #[error("{message}")]
    NotFound {
        message: String,
        resource_type: String,
        resource_name: String,
        suggestions: Vec<String>,
    },

    #[error("{message}")]
    Permission {
        message: String,
        path: Option<std::path::PathBuf>,
        required: Option<String>,
    },

    #[error("{message}")]
    Command {
        message: String,
        command: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("{message}")]
    Backend {
        message: String,
        backend: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("{message}")]
    Tool {
        message: String,
        tool: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("{message}")]
    User {
        message: String,
        hint: Option<String>,
    },

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    /// Get the error code for this error
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config { .. } => "E001",
            Self::Io { .. } => "E002",
            Self::Network { .. } => "E003",
            Self::Validation { .. } => "E004",
            Self::NotFound { .. } => "E005",
            Self::Permission { .. } => "E006",
            Self::Command { .. } => "E007",
            Self::Backend { .. } => "E008",
            Self::Tool { .. } => "E009",
            Self::User { .. } => "E010",
            Self::Other(_) => "E999",
        }
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> ExitCode {
        let code = match self {
            Self::Config { .. } => 2,
            Self::Io { .. } => 3,
            Self::Network { .. } => 4,
            Self::Validation { .. } => 5,
            Self::NotFound { .. } => 6,
            Self::Permission { .. } => 7,
            Self::Command { .. } => 8,
            Self::Backend { .. } => 9,
            Self::Tool { .. } => 10,
            Self::User { .. } => 1,
            Self::Other(_) => 1,
        };
        ExitCode::from(code)
    }

    /// Get hint for this error if available
    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::Config { hint, .. } => hint.as_deref(),
            Self::User { hint, .. } => hint.as_deref(),
            Self::NotFound { suggestions, .. } if !suggestions.is_empty() => {
                Some("See suggestions below")
            }
            _ => None,
        }
    }

    /// Get suggestions for this error
    pub fn suggestions(&self) -> &[String] {
        match self {
            Self::NotFound { suggestions, .. } => suggestions,
            _ => &[],
        }
    }

    /// Create a config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
            hint: None,
        }
    }

    /// Create a config error with hint
    pub fn config_with_hint(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
            hint: Some(hint.into()),
        }
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source,
            path: None,
        }
    }

    /// Create an IO error with path
    pub fn io_with_path(
        message: impl Into<String>,
        source: io::Error,
        path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self::Io {
            message: message.into(),
            source,
            path: Some(path.into()),
        }
    }

    /// Create a not found error
    pub fn not_found(
        resource_type: impl Into<String>,
        resource_name: impl Into<String>,
    ) -> Self {
        let resource_type = resource_type.into();
        let resource_name = resource_name.into();
        Self::NotFound {
            message: format!("{resource_type} not found: {resource_name}"),
            resource_type,
            resource_name,
            suggestions: vec![],
        }
    }

    /// Create a not found error with suggestions
    pub fn not_found_with_suggestions(
        resource_type: impl Into<String>,
        resource_name: impl Into<String>,
        suggestions: Vec<String>,
    ) -> Self {
        let resource_type = resource_type.into();
        let resource_name = resource_name.into();
        Self::NotFound {
            message: format!("{resource_type} not found: {resource_name}"),
            resource_type,
            resource_name,
            suggestions,
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            expected: None,
            actual: None,
        }
    }

    /// Create a user error (user did something wrong)
    pub fn user(message: impl Into<String>) -> Self {
        Self::User {
            message: message.into(),
            hint: None,
        }
    }

    /// Create a user error with hint
    pub fn user_with_hint(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::User {
            message: message.into(),
            hint: Some(hint.into()),
        }
    }
}

// Conversion implementations
impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
            source: err,
            path: None,
        }
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config {
            message: format!("Invalid TOML: {err}"),
            source: Some(Box::new(err)),
            hint: Some("Check your configuration file syntax".to_string()),
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        Self::Validation {
            message: format!("Invalid JSON: {err}"),
            field: None,
            expected: None,
            actual: None,
        }
    }
}
```

### src/error/formatter.rs

```rust
//! Error formatting for CLI output.

use std::fmt::Write as FmtWrite;
use std::io::{self, IsTerminal, Write};

use crate::error::CliError;
use crate::output::color::{ColorMode, Styled, Color};

/// Error output formatter
pub struct ErrorFormatter {
    color_mode: ColorMode,
    verbose: bool,
}

impl ErrorFormatter {
    pub fn new() -> Self {
        Self {
            color_mode: if io::stderr().is_terminal() {
                ColorMode::Auto
            } else {
                ColorMode::Never
            },
            verbose: false,
        }
    }

    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Format and print an error
    pub fn print(&self, error: &CliError) {
        let formatted = self.format(error);
        eprintln!("{formatted}");
    }

    /// Format an error to a string
    pub fn format(&self, error: &CliError) -> String {
        let mut output = String::new();

        // Error prefix with code
        let error_prefix = Styled::new(format!("error[{}]:", error.code()))
            .with_color_mode(self.color_mode)
            .fg(Color::Red)
            .bold();

        writeln!(output, "{error_prefix} {error}").unwrap();

        // Source chain in verbose mode
        if self.verbose {
            self.format_source_chain(&mut output, error);
        }

        // Context information
        self.format_context(&mut output, error);

        // Hint
        if let Some(hint) = error.hint() {
            let hint_label = Styled::new("hint:")
                .with_color_mode(self.color_mode)
                .fg(Color::Cyan)
                .bold();
            writeln!(output, "\n{hint_label} {hint}").unwrap();
        }

        // Suggestions
        let suggestions = error.suggestions();
        if !suggestions.is_empty() {
            let suggest_label = Styled::new("suggestions:")
                .with_color_mode(self.color_mode)
                .fg(Color::Yellow)
                .bold();
            writeln!(output, "\n{suggest_label}").unwrap();

            for suggestion in suggestions {
                writeln!(output, "  - {suggestion}").unwrap();
            }
        }

        output
    }

    /// Format as JSON
    pub fn format_json(&self, error: &CliError) -> String {
        let json = serde_json::json!({
            "error": {
                "code": error.code(),
                "message": error.to_string(),
                "hint": error.hint(),
                "suggestions": error.suggestions(),
            }
        });

        serde_json::to_string_pretty(&json).unwrap_or_else(|_| error.to_string())
    }

    fn format_source_chain(&self, output: &mut String, error: &CliError) {
        use std::error::Error;

        let source_opt: Option<&dyn Error> = match error {
            CliError::Config { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn Error),
            CliError::Io { source, .. } => Some(source as &dyn Error),
            CliError::Network { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn Error),
            CliError::Command { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn Error),
            CliError::Backend { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn Error),
            CliError::Tool { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn Error),
            _ => None,
        };

        if let Some(source) = source_opt {
            let caused_label = Styled::new("caused by:")
                .with_color_mode(self.color_mode)
                .fg(Color::BrightBlack);

            writeln!(output, "\n{caused_label}").unwrap();
            writeln!(output, "  {source}").unwrap();

            // Walk the source chain
            let mut current = source.source();
            let mut depth = 1;
            while let Some(src) = current {
                writeln!(output, "  {}: {src}", depth).unwrap();
                current = src.source();
                depth += 1;
            }
        }
    }

    fn format_context(&self, output: &mut String, error: &CliError) {
        match error {
            CliError::Io { path: Some(p), .. } => {
                let path_label = Styled::new("path:")
                    .with_color_mode(self.color_mode)
                    .fg(Color::BrightBlack);
                writeln!(output, "\n  {path_label} {}", p.display()).unwrap();
            }
            CliError::Network { url: Some(url), .. } => {
                let url_label = Styled::new("url:")
                    .with_color_mode(self.color_mode)
                    .fg(Color::BrightBlack);
                writeln!(output, "\n  {url_label} {url}").unwrap();
            }
            CliError::NotFound {
                resource_type,
                resource_name,
                ..
            } => {
                let resource_label = Styled::new("resource:")
                    .with_color_mode(self.color_mode)
                    .fg(Color::BrightBlack);
                writeln!(output, "\n  {resource_label} {resource_type}/{resource_name}").unwrap();
            }
            CliError::Validation {
                field: Some(field),
                expected,
                actual,
                ..
            } => {
                let field_label = Styled::new("field:")
                    .with_color_mode(self.color_mode)
                    .fg(Color::BrightBlack);
                writeln!(output, "\n  {field_label} {field}").unwrap();

                if let Some(expected) = expected {
                    writeln!(output, "  expected: {expected}").unwrap();
                }
                if let Some(actual) = actual {
                    writeln!(output, "  actual: {actual}").unwrap();
                }
            }
            _ => {}
        }
    }
}

impl Default for ErrorFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Print an error with the default formatter
pub fn print_error(error: &CliError) {
    ErrorFormatter::new().print(error);
}

/// Print an error with verbose details
pub fn print_error_verbose(error: &CliError) {
    ErrorFormatter::new().verbose(true).print(error);
}
```

### src/error/handler.rs

```rust
//! Error handling utilities.

use std::panic;
use std::process::ExitCode;

use crate::cli::OutputFormat;
use crate::error::{CliError, ErrorFormatter};

/// Set up panic handler for user-friendly panic messages
pub fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        eprintln!("\n\x1b[31mInternal error:\x1b[0m {message}");
        eprintln!("\nThis is a bug in Tachikoma. Please report it at:");
        eprintln!("  https://github.com/tachikoma/tachikoma/issues");
        eprintln!("\nLocation: {location}");

        // Print backtrace if RUST_BACKTRACE is set
        if std::env::var("RUST_BACKTRACE").is_ok() {
            eprintln!("\nBacktrace:");
            eprintln!("{:?}", std::backtrace::Backtrace::capture());
        } else {
            eprintln!("\nSet RUST_BACKTRACE=1 for a backtrace.");
        }
    }));
}

/// Handle a result and exit appropriately
pub fn handle_result<T>(result: Result<T, CliError>, format: OutputFormat) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            match format {
                OutputFormat::Json => {
                    let json = ErrorFormatter::new().format_json(&error);
                    eprintln!("{json}");
                }
                OutputFormat::Text => {
                    ErrorFormatter::new().print(&error);
                }
            }
            error.exit_code()
        }
    }
}

/// Context for error reporting
pub struct ErrorContext {
    pub command: Option<String>,
    pub subcommand: Option<String>,
    pub args: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            command: None,
            subcommand: None,
            args: vec![],
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn with_subcommand(mut self, subcommand: impl Into<String>) -> Self {
        self.subcommand = Some(subcommand.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
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
    fn test_error_code() {
        let error = CliError::config("test");
        assert_eq!(error.code(), "E001");

        let error = CliError::not_found("tool", "test");
        assert_eq!(error.code(), "E005");
    }

    #[test]
    fn test_error_with_hint() {
        let error = CliError::config_with_hint(
            "Config not found",
            "Run 'tachikoma config init'"
        );

        assert_eq!(error.hint(), Some("Run 'tachikoma config init'"));
    }

    #[test]
    fn test_error_with_suggestions() {
        let error = CliError::not_found_with_suggestions(
            "backend",
            "antropic",
            vec!["anthropic".to_string()],
        );

        assert!(!error.suggestions().is_empty());
        assert_eq!(error.suggestions()[0], "anthropic");
    }

    #[test]
    fn test_error_formatter() {
        let error = CliError::config("Test error");
        let formatter = ErrorFormatter::new().color_mode(ColorMode::Never);
        let output = formatter.format(&error);

        assert!(output.contains("error[E001]:"));
        assert!(output.contains("Test error"));
    }

    #[test]
    fn test_error_json_format() {
        let error = CliError::config("Test error");
        let formatter = ErrorFormatter::new();
        let json = formatter.format_json(&error);

        assert!(json.contains("\"code\": \"E001\""));
        assert!(json.contains("Test error"));
    }
}
```

## Related Specs

- **002-error.md**: Core error handling
- **076-cli-crate.md**: Base CLI structure
- **080-cli-json.md**: JSON output mode
- **090-cli-help.md**: Help system integration
