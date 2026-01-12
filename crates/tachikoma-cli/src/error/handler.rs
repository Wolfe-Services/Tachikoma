//! Error handling utilities.

use std::panic;
use std::process::ExitCode;

use crate::cli::OutputFormat;
use crate::error::{CliError, ErrorFormatter};
use crate::output::color::ColorMode;

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
pub fn handle_result<T>(
    result: Result<T, CliError>,
    format: OutputFormat,
    color_mode: ColorMode,
    verbose: bool,
) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            match format {
                OutputFormat::Json => {
                    let json = ErrorFormatter::new().format_json(&error);
                    eprintln!("{json}");
                }
                OutputFormat::Text => {
                    ErrorFormatter::new()
                        .color_mode(color_mode)
                        .verbose(verbose)
                        .print(&error);
                }
            }
            error.exit_code()
        }
    }
}

/// Context for error reporting
#[derive(Debug, Clone)]
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

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced error handling with context
pub fn handle_error_with_context(
    error: CliError,
    context: &ErrorContext,
    format: OutputFormat,
    color_mode: ColorMode,
    verbose: bool,
) -> ExitCode {
    // Enhance error with context information if possible
    let enhanced_error = enhance_error_with_context(error, context);

    match format {
        OutputFormat::Json => {
            let mut json = ErrorFormatter::new().format_json(&enhanced_error);
            
            // Add context to JSON if available
            if context.command.is_some() || context.subcommand.is_some() {
                if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&json) {
                    if let Some(error_obj) = value.get_mut("error") {
                        if let Some(cmd) = &context.command {
                            error_obj["context"] = serde_json::json!({
                                "command": cmd,
                                "subcommand": context.subcommand,
                                "args": context.args
                            });
                        }
                    }
                    json = serde_json::to_string_pretty(&value).unwrap_or(json);
                }
            }
            
            eprintln!("{json}");
        }
        OutputFormat::Text => {
            ErrorFormatter::new()
                .color_mode(color_mode)
                .verbose(verbose)
                .print(&enhanced_error);
        }
    }

    enhanced_error.exit_code()
}

/// Enhance error messages with context
fn enhance_error_with_context(error: CliError, context: &ErrorContext) -> CliError {
    match &error {
        CliError::NotFound { resource_type, resource_name, suggestions } => {
            // Add command-specific suggestions
            let mut enhanced_suggestions = suggestions.clone();
            
            if let Some(cmd) = &context.command {
                match (cmd.as_str(), resource_type.as_str()) {
                    ("tools", "tool") => {
                        enhanced_suggestions.extend([
                            format!("Use 'tachikoma tools list' to see available tools"),
                            format!("Use 'tachikoma tools install {resource_name}' to install a tool"),
                        ]);
                    }
                    ("backends", "backend") => {
                        enhanced_suggestions.extend([
                            format!("Use 'tachikoma backends list' to see available backends"),
                            format!("Use 'tachikoma backends add {resource_name}' to add a backend"),
                        ]);
                    }
                    ("config", "configuration") => {
                        enhanced_suggestions.extend([
                            format!("Use 'tachikoma config init' to create a configuration"),
                            format!("Use 'tachikoma config show' to see current configuration"),
                        ]);
                    }
                    _ => {}
                }
            }
            
            if !enhanced_suggestions.is_empty() {
                CliError::not_found_with_suggestions(
                    resource_type.clone(),
                    resource_name.clone(),
                    enhanced_suggestions,
                )
            } else {
                error
            }
        }
        CliError::User { message, hint } => {
            // Enhance user errors with command context
            let enhanced_hint = if let Some(cmd) = &context.command {
                match hint {
                    Some(h) => Some(format!("{h}. Use 'tachikoma {cmd} --help' for usage information.")),
                    None => Some(format!("Use 'tachikoma {cmd} --help' for usage information.")),
                }
            } else {
                hint.clone()
            };
            
            CliError::User {
                message: message.clone(),
                hint: enhanced_hint,
            }
        }
        _ => error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let ctx = ErrorContext::new()
            .with_command("tools")
            .with_subcommand("list");
        
        assert_eq!(ctx.command, Some("tools".to_string()));
        assert_eq!(ctx.subcommand, Some("list".to_string()));
    }

    #[test]
    fn test_enhance_not_found_error() {
        let error = CliError::not_found("tool", "nonexistent");
        let ctx = ErrorContext::new().with_command("tools");
        
        let enhanced = enhance_error_with_context(error, &ctx);
        
        if let CliError::NotFound { suggestions, .. } = enhanced {
            assert!(!suggestions.is_empty());
            assert!(suggestions.iter().any(|s| s.contains("tools list")));
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[test]
    fn test_enhance_user_error() {
        let error = CliError::user("Invalid argument");
        let ctx = ErrorContext::new().with_command("config");
        
        let enhanced = enhance_error_with_context(error, &ctx);
        
        if let CliError::User { hint: Some(hint), .. } = enhanced {
            assert!(hint.contains("config --help"));
        } else {
            panic!("Expected User error with hint");
        }
    }

    #[test]
    fn test_handle_result_success() {
        let result: Result<(), CliError> = Ok(());
        let exit_code = handle_result(
            result,
            OutputFormat::Text,
            ColorMode::Never,
            false,
        );
        assert_eq!(exit_code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_handle_result_error() {
        let result: Result<(), CliError> = Err(CliError::user("test error"));
        let exit_code = handle_result(
            result,
            OutputFormat::Text,
            ColorMode::Never,
            false,
        );
        assert_ne!(exit_code, ExitCode::SUCCESS);
    }
}