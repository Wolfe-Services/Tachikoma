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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::color::ColorMode;

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

    #[test]
    fn test_error_with_hint() {
        let error = CliError::config_with_hint(
            "Config not found",
            "Run 'tachikoma config init'"
        );
        let formatter = ErrorFormatter::new().color_mode(ColorMode::Never);
        let output = formatter.format(&error);

        assert!(output.contains("hint:"));
        assert!(output.contains("Run 'tachikoma config init'"));
    }

    #[test]
    fn test_error_with_suggestions() {
        let error = CliError::not_found_with_suggestions(
            "backend",
            "antropic",
            vec!["anthropic".to_string()],
        );
        let formatter = ErrorFormatter::new().color_mode(ColorMode::Never);
        let output = formatter.format(&error);

        assert!(output.contains("suggestions:"));
        assert!(output.contains("- anthropic"));
    }

    #[test]
    fn test_verbose_mode() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = CliError::io("Failed to read file", io_error);
        let formatter = ErrorFormatter::new()
            .color_mode(ColorMode::Never)
            .verbose(true);
        let output = formatter.format(&error);

        assert!(output.contains("caused by:"));
    }
}