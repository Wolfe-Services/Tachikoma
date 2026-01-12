//! Output formatting utilities for CLI commands.

use serde::Serialize;
use std::io::Write;

use crate::cli::{OutputFormat, CommandContext};
use crate::error::CliError;

/// Trait for types that can be formatted for output
pub trait FormattedOutput {
    fn format_text(&self) -> String;
    fn format_json(&self) -> Result<String, serde_json::Error>;
}

/// Print formatted output to stdout
pub fn print_output<T>(ctx: &CommandContext, value: &T) -> Result<(), CliError>
where
    T: FormattedOutput + Serialize,
{
    let output = match ctx.format {
        OutputFormat::Text => value.format_text(),
        OutputFormat::Json => value.format_json().map_err(|e| {
            CliError::Other(anyhow::anyhow!("JSON serialization failed: {}", e))
        })?,
    };

    println!("{}", output);
    Ok(())
}

/// Print formatted output to a writer
pub fn write_output<T, W>(ctx: &CommandContext, value: &T, mut writer: W) -> Result<(), CliError>
where
    T: FormattedOutput + Serialize,
    W: Write,
{
    let output = match ctx.format {
        OutputFormat::Text => value.format_text(),
        OutputFormat::Json => value.format_json().map_err(|e| {
            CliError::Other(anyhow::anyhow!("JSON serialization failed: {}", e))
        })?,
    };

    writeln!(writer, "{}", output).map_err(CliError::Io)?;
    Ok(())
}

/// Helper for simple string outputs
#[derive(Debug, Serialize)]
pub struct SimpleOutput {
    pub message: String,
}

impl SimpleOutput {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl FormattedOutput for SimpleOutput {
    fn format_text(&self) -> String {
        self.message.clone()
    }

    fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Helper for success/error status outputs
#[derive(Debug, Serialize)]
pub struct StatusOutput {
    pub status: String,
    pub message: String,
}

impl StatusOutput {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            status: "success".to_string(),
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: message.into(),
        }
    }
}

impl FormattedOutput for StatusOutput {
    fn format_text(&self) -> String {
        match self.status.as_str() {
            "success" => format!("✓ {}", self.message),
            "error" => format!("✗ {}", self.message),
            _ => format!("{}: {}", self.status, self.message),
        }
    }

    fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}