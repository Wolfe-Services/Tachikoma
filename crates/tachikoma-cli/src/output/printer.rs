//! Output printer with format awareness.

use std::io::{self, Write};

use serde::Serialize;

use crate::cli::{CommandContext, OutputFormat};
use crate::output::{is_terminal, terminal_width, Table, TextFormatter};

/// Output configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub color: bool,
    pub width: usize,
    pub quiet: bool,
}

impl OutputConfig {
    pub fn from_context(ctx: &CommandContext) -> Self {
        let is_tty = is_terminal();
        Self {
            format: ctx.format,
            color: match ctx.color {
                clap::ColorChoice::Always => true,
                clap::ColorChoice::Never => false,
                clap::ColorChoice::Auto => is_tty,
            },
            width: if is_tty { terminal_width() } else { 80 },
            quiet: ctx.verbose == 0 && false, // TODO: Add quiet flag to CLI
        }
    }
}

/// Main output handler
pub struct Output {
    config: OutputConfig,
}

impl Output {
    pub fn new(ctx: &CommandContext) -> Self {
        Self {
            config: OutputConfig::from_context(ctx),
        }
    }

    pub fn with_config(config: OutputConfig) -> Self {
        Self { config }
    }

    /// Print a value with appropriate formatting
    pub fn print<T>(&self, value: &T) -> io::Result<()>
    where
        T: Serialize + std::fmt::Display,
    {
        match self.config.format {
            OutputFormat::Json => self.print_json(value),
            OutputFormat::Text => self.print_text(value),
        }
    }

    /// Print as JSON
    pub fn print_json<T: Serialize>(&self, value: &T) -> io::Result<()> {
        let json = serde_json::to_string_pretty(value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{json}");
        Ok(())
    }

    /// Print as human-readable text
    pub fn print_text<T: std::fmt::Display>(&self, value: &T) -> io::Result<()> {
        println!("{value}");
        Ok(())
    }

    /// Print a table
    pub fn print_table(&self, table: &Table) -> io::Result<()> {
        match self.config.format {
            OutputFormat::Json => {
                let rows = table.to_json_rows();
                let json = serde_json::to_string_pretty(&rows)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                println!("{json}");
            }
            OutputFormat::Text => {
                print!("{}", table.render(self.config.width, self.config.color));
            }
        }
        Ok(())
    }

    /// Print a message (respects quiet mode)
    pub fn message(&self, msg: &str) {
        if !self.config.quiet {
            println!("{msg}");
        }
    }

    /// Print a success message
    pub fn success(&self, msg: &str) {
        if !self.config.quiet {
            if self.config.color {
                println!("\x1b[32m{msg}\x1b[0m");
            } else {
                println!("{msg}");
            }
        }
    }

    /// Print a warning
    pub fn warning(&self, msg: &str) {
        if self.config.color {
            eprintln!("\x1b[33mWarning: {msg}\x1b[0m");
        } else {
            eprintln!("Warning: {msg}");
        }
    }

    /// Print an error
    pub fn error(&self, msg: &str) {
        if self.config.color {
            eprintln!("\x1b[31mError: {msg}\x1b[0m");
        } else {
            eprintln!("Error: {msg}");
        }
    }

    /// Print a hint
    pub fn hint(&self, msg: &str) {
        if !self.config.quiet {
            if self.config.color {
                println!("\x1b[36mHint: {msg}\x1b[0m");
            } else {
                println!("Hint: {msg}");
            }
        }
    }
}

/// Trait for types that can be output in multiple formats
pub trait Printable: Serialize + std::fmt::Display {
    /// Render as a table (optional)
    fn as_table(&self) -> Option<Table> {
        None
    }
}

/// Print list of items
pub fn print_list<T, I>(output: &Output, items: I, empty_msg: &str) -> io::Result<()>
where
    T: Serialize + std::fmt::Display,
    I: IntoIterator<Item = T>,
{
    let items: Vec<_> = items.into_iter().collect();

    if items.is_empty() {
        output.message(empty_msg);
        return Ok(());
    }

    match output.config.format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            println!("{json}");
        }
        OutputFormat::Text => {
            for item in items {
                println!("{item}");
            }
        }
    }

    Ok(())
}

/// Streaming output for long-running operations
pub struct StreamingOutput {
    output: Output,
    buffer: Vec<String>,
    buffer_size: usize,
}

impl StreamingOutput {
    pub fn new(output: Output) -> Self {
        Self {
            output,
            buffer: Vec::new(),
            buffer_size: 100, // Default buffer size
        }
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Add a line to the stream
    pub fn push(&mut self, line: String) {
        self.buffer.push(line);
        if self.buffer.len() >= self.buffer_size {
            self.flush().ok();
        }
    }

    /// Flush buffered lines
    pub fn flush(&mut self) -> io::Result<()> {
        for line in self.buffer.drain(..) {
            println!("{line}");
        }
        io::stdout().flush()
    }

    /// Finish the stream and flush remaining content
    pub fn finish(mut self) -> io::Result<()> {
        self.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::*;
    use tachikoma_common_config::TachikomaConfig;

    #[test]
    fn test_output_config_from_context() {
        let config = TachikomaConfig::default();
        let ctx = CommandContext {
            config,
            format: OutputFormat::Text,
            color: clap::ColorChoice::Auto,
            verbose: 1,
        };

        let output_config = OutputConfig::from_context(&ctx);
        assert_eq!(output_config.format, OutputFormat::Text);
        // Color depends on terminal detection, so we don't test exact value
    }

    #[test]
    fn test_streaming_output() {
        use std::io::Write;
        
        let config = OutputConfig {
            format: OutputFormat::Text,
            color: false,
            width: 80,
            quiet: false,
        };
        let output = Output::with_config(config);
        let mut streaming = StreamingOutput::new(output).with_buffer_size(2);
        
        streaming.push("line1".to_string());
        streaming.push("line2".to_string());
        // Should flush automatically when buffer size is reached
        
        streaming.finish().unwrap();
    }
}