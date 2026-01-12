//! Output formatting utilities for CLI commands.

mod color;
mod format;
mod icons;
mod printer;
mod progress;
mod table;
mod text;

pub use color::{
    Color, ColorContext, ColorDepth, ColorMode, Palette, Style, Styled,
    detect_color_support, strip_ansi, styled, success, error, warning, info, bold
};
pub use format::{Displayable, OutputFormat as InternalOutputFormat};
pub use icons::{Icons, IconContext};
pub use printer::{Output, OutputConfig};
pub use progress::{
    ProgressBar, ProgressStyle, Spinner, SpinnerHandle, MultiProgress, MultiProgressHandle,
    SPINNER_DOTS, SPINNER_LINE, SPINNER_ARROWS, SPINNER_BRAILLE
};
pub use table::{Table, TableBuilder, TableStyle, Column, Alignment};
pub use text::{TextFormatter, Wrapped};

use std::io::{self, Write};
use is_terminal::IsTerminal;
use serde::Serialize;

use crate::cli::{OutputFormat, CommandContext};
use crate::error::CliError;

/// Check if stdout is a terminal
pub fn is_terminal() -> bool {
    io::stdout().is_terminal()
}

/// Check if stderr is a terminal
pub fn is_stderr_terminal() -> bool {
    io::stderr().is_terminal()
}

/// Get terminal width (default 80 if not a terminal)
pub fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}

/// Output destination
pub enum Destination {
    Stdout,
    Stderr,
    File(std::fs::File),
    Buffer(Vec<u8>),
}

impl Write for Destination {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout => io::stdout().write(buf),
            Self::Stderr => io::stderr().write(buf),
            Self::File(f) => f.write(buf),
            Self::Buffer(b) => b.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdout => io::stdout().flush(),
            Self::Stderr => io::stderr().flush(),
            Self::File(f) => f.flush(),
            Self::Buffer(_) => Ok(()),
        }
    }
}

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
    let output = Output::new(ctx);
    match ctx.format {
        OutputFormat::Text => {
            println!("{}", value.format_text());
        },
        OutputFormat::Json => {
            let json = value.format_json().map_err(|e| {
                CliError::Other(anyhow::anyhow!("JSON serialization failed: {}", e))
            })?;
            println!("{}", json);
        },
    }
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

/// Print list of items
pub fn print_list<T, I>(ctx: &CommandContext, items: I, empty_msg: &str) -> Result<(), CliError>
where
    T: Serialize + std::fmt::Display,
    I: IntoIterator<Item = T>,
{
    let output = Output::new(ctx);
    let items: Vec<_> = items.into_iter().collect();

    if items.is_empty() {
        output.message(empty_msg);
        return Ok(());
    }

    match ctx.format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| CliError::Other(anyhow::anyhow!("JSON serialization failed: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Text => {
            for item in items {
                println!("{}", item);
            }
        }
    }

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