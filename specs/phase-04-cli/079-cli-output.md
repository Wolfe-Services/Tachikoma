# Spec 079: CLI Output Formatting

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 079
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 080-cli-json, 081-cli-color
- **Estimated Context**: ~10%

## Objective

Implement a flexible output formatting system that supports multiple output modes (text, JSON, table) with consistent styling and structure across all CLI commands.

## Acceptance Criteria

- [ ] Output trait for consistent formatting
- [ ] Text output with human-readable formatting
- [ ] Table output for structured data
- [ ] Support for output redirection detection
- [ ] Streaming output for long-running operations
- [ ] Width-aware formatting for terminals
- [ ] Quiet mode support
- [ ] Output buffering options

## Implementation Details

### src/output/mod.rs

```rust
//! Output formatting utilities.

mod format;
mod printer;
mod table;
mod text;

pub use format::{Displayable, OutputFormat};
pub use printer::{Output, OutputConfig};
pub use table::{Table, TableBuilder, TableStyle};
pub use text::{TextFormatter, Wrapped};

use std::io::{self, IsTerminal, Write};

use crate::cli::{CommandContext, OutputFormat as CliOutputFormat};

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
```

### src/output/printer.rs

```rust
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
            quiet: ctx.verbose == 0 && false, // Implement quiet flag
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
                println!("{}", serde_json::to_string_pretty(&rows)?);
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
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        OutputFormat::Text => {
            for item in items {
                println!("{item}");
            }
        }
    }

    Ok(())
}
```

### src/output/table.rs

```rust
//! Table formatting for CLI output.

use serde::Serialize;
use serde_json::Value;

/// Table rendering style
#[derive(Debug, Clone, Copy, Default)]
pub enum TableStyle {
    #[default]
    Plain,
    Bordered,
    Markdown,
    Compact,
}

/// Column alignment
#[derive(Debug, Clone, Copy, Default)]
pub enum Alignment {
    #[default]
    Left,
    Right,
    Center,
}

/// Table column definition
#[derive(Debug, Clone)]
pub struct Column {
    pub header: String,
    pub alignment: Alignment,
    pub min_width: usize,
    pub max_width: Option<usize>,
    pub color: Option<&'static str>,
}

impl Column {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            alignment: Alignment::Left,
            min_width: 0,
            max_width: None,
            color: None,
        }
    }

    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn color(mut self, color: &'static str) -> Self {
        self.color = Some(color);
        self
    }
}

/// Table structure
#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    style: TableStyle,
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            style: TableStyle::default(),
        }
    }

    pub fn style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    pub fn add_row(&mut self, row: Vec<impl Into<String>>) {
        self.rows.push(row.into_iter().map(Into::into).collect());
    }

    pub fn add_rows(&mut self, rows: impl IntoIterator<Item = Vec<impl Into<String>>>) {
        for row in rows {
            self.add_row(row);
        }
    }

    /// Calculate column widths
    fn calculate_widths(&self, max_total: usize) -> Vec<usize> {
        let mut widths: Vec<usize> = self
            .columns
            .iter()
            .map(|c| c.header.len().max(c.min_width))
            .collect();

        // Consider row content
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Apply max widths
        for (i, col) in self.columns.iter().enumerate() {
            if let Some(max) = col.max_width {
                widths[i] = widths[i].min(max);
            }
        }

        // Ensure fits in terminal
        let total: usize = widths.iter().sum::<usize>() + (widths.len() - 1) * 3;
        if total > max_total && widths.len() > 1 {
            let excess = total - max_total;
            let reduce_per = excess / widths.len() + 1;
            for w in &mut widths {
                *w = (*w).saturating_sub(reduce_per).max(5);
            }
        }

        widths
    }

    /// Render the table to a string
    pub fn render(&self, max_width: usize, color: bool) -> String {
        let widths = self.calculate_widths(max_width);
        let mut output = String::new();

        match self.style {
            TableStyle::Plain => self.render_plain(&mut output, &widths, color),
            TableStyle::Bordered => self.render_bordered(&mut output, &widths, color),
            TableStyle::Markdown => self.render_markdown(&mut output, &widths),
            TableStyle::Compact => self.render_compact(&mut output, &widths, color),
        }

        output
    }

    fn render_plain(&self, output: &mut String, widths: &[usize], color: bool) {
        // Header
        let header: Vec<_> = self
            .columns
            .iter()
            .zip(widths)
            .map(|(col, &w)| self.format_cell(&col.header, w, col.alignment))
            .collect();

        if color {
            output.push_str("\x1b[1m");
        }
        output.push_str(&header.join("   "));
        if color {
            output.push_str("\x1b[0m");
        }
        output.push('\n');

        // Separator
        let sep: Vec<_> = widths.iter().map(|&w| "-".repeat(w)).collect();
        output.push_str(&sep.join("   "));
        output.push('\n');

        // Rows
        for row in &self.rows {
            let cells: Vec<_> = row
                .iter()
                .zip(&self.columns)
                .zip(widths)
                .map(|((cell, col), &w)| self.format_cell(cell, w, col.alignment))
                .collect();
            output.push_str(&cells.join("   "));
            output.push('\n');
        }
    }

    fn render_bordered(&self, output: &mut String, widths: &[usize], color: bool) {
        let horiz = |output: &mut String, left: &str, mid: &str, right: &str| {
            output.push_str(left);
            let parts: Vec<_> = widths.iter().map(|&w| "─".repeat(w + 2)).collect();
            output.push_str(&parts.join(mid));
            output.push_str(right);
            output.push('\n');
        };

        // Top border
        horiz(output, "┌", "┬", "┐");

        // Header
        output.push_str("│");
        for (i, col) in self.columns.iter().enumerate() {
            let cell = self.format_cell(&col.header, widths[i], col.alignment);
            if color {
                output.push_str(&format!(" \x1b[1m{cell}\x1b[0m │"));
            } else {
                output.push_str(&format!(" {cell} │"));
            }
        }
        output.push('\n');

        // Header separator
        horiz(output, "├", "┼", "┤");

        // Rows
        for row in &self.rows {
            output.push_str("│");
            for (i, cell) in row.iter().enumerate() {
                let col = &self.columns[i];
                let formatted = self.format_cell(cell, widths[i], col.alignment);
                output.push_str(&format!(" {formatted} │"));
            }
            output.push('\n');
        }

        // Bottom border
        horiz(output, "└", "┴", "┘");
    }

    fn render_markdown(&self, output: &mut String, widths: &[usize]) {
        // Header
        output.push('|');
        for (i, col) in self.columns.iter().enumerate() {
            let cell = self.format_cell(&col.header, widths[i], col.alignment);
            output.push_str(&format!(" {cell} |"));
        }
        output.push('\n');

        // Separator with alignment
        output.push('|');
        for (i, col) in self.columns.iter().enumerate() {
            let w = widths[i];
            let sep = match col.alignment {
                Alignment::Left => format!(":{}", "-".repeat(w)),
                Alignment::Right => format!("{}:", "-".repeat(w)),
                Alignment::Center => format!(":{}:", "-".repeat(w.saturating_sub(1))),
            };
            output.push_str(&format!(" {sep} |"));
        }
        output.push('\n');

        // Rows
        for row in &self.rows {
            output.push('|');
            for (i, cell) in row.iter().enumerate() {
                let col = &self.columns[i];
                let formatted = self.format_cell(cell, widths[i], col.alignment);
                output.push_str(&format!(" {formatted} |"));
            }
            output.push('\n');
        }
    }

    fn render_compact(&self, output: &mut String, widths: &[usize], _color: bool) {
        for row in &self.rows {
            let cells: Vec<_> = row
                .iter()
                .zip(&self.columns)
                .zip(widths)
                .map(|((cell, col), &w)| self.format_cell(cell, w, col.alignment))
                .collect();
            output.push_str(&cells.join(" "));
            output.push('\n');
        }
    }

    fn format_cell(&self, content: &str, width: usize, alignment: Alignment) -> String {
        let content = if content.len() > width {
            format!("{}...", &content[..width.saturating_sub(3)])
        } else {
            content.to_string()
        };

        match alignment {
            Alignment::Left => format!("{content:<width$}"),
            Alignment::Right => format!("{content:>width$}"),
            Alignment::Center => format!("{content:^width$}"),
        }
    }

    /// Convert to JSON-serializable rows
    pub fn to_json_rows(&self) -> Vec<serde_json::Map<String, Value>> {
        self.rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, cell) in row.iter().enumerate() {
                    if i < self.columns.len() {
                        map.insert(
                            self.columns[i].header.clone(),
                            Value::String(cell.clone()),
                        );
                    }
                }
                map
            })
            .collect()
    }
}

/// Builder for creating tables
pub struct TableBuilder {
    columns: Vec<Column>,
    style: TableStyle,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            style: TableStyle::default(),
        }
    }

    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn columns(mut self, columns: impl IntoIterator<Item = Column>) -> Self {
        self.columns.extend(columns);
        self
    }

    pub fn style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Table {
        Table::new(self.columns).style(self.style)
    }
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

### src/output/text.rs

```rust
//! Text formatting utilities.

use textwrap::{wrap, Options, WordSplitter};

/// Text formatter with terminal awareness
pub struct TextFormatter {
    width: usize,
    indent: usize,
}

impl TextFormatter {
    pub fn new(width: usize) -> Self {
        Self { width, indent: 0 }
    }

    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Wrap text to fit terminal width
    pub fn wrap(&self, text: &str) -> String {
        let options = Options::new(self.width.saturating_sub(self.indent))
            .word_splitter(WordSplitter::NoHyphenation)
            .initial_indent(&" ".repeat(self.indent))
            .subsequent_indent(&" ".repeat(self.indent));

        wrap(text, options).join("\n")
    }

    /// Format a key-value pair
    pub fn key_value(&self, key: &str, value: &str) -> String {
        let key_width = 20.min(self.width / 3);
        format!("{key:key_width$} {value}")
    }

    /// Format a section header
    pub fn section(&self, title: &str) -> String {
        format!("\n{title}\n{}\n", "=".repeat(title.len()))
    }

    /// Format a subsection header
    pub fn subsection(&self, title: &str) -> String {
        format!("\n{title}\n{}\n", "-".repeat(title.len()))
    }

    /// Format a bullet list
    pub fn bullet_list<I, S>(&self, items: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        items
            .into_iter()
            .map(|s| format!("  * {}", s.as_ref()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format a numbered list
    pub fn numbered_list<I, S>(&self, items: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        items
            .into_iter()
            .enumerate()
            .map(|(i, s)| format!("  {}. {}", i + 1, s.as_ref()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Wrapper for text that implements Display with wrapping
pub struct Wrapped<'a> {
    text: &'a str,
    width: usize,
}

impl<'a> Wrapped<'a> {
    pub fn new(text: &'a str, width: usize) -> Self {
        Self { text, width }
    }
}

impl std::fmt::Display for Wrapped<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatter = TextFormatter::new(self.width);
        write!(f, "{}", formatter.wrap(self.text))
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
    fn test_table_plain() {
        let mut table = Table::new(vec![
            Column::new("Name"),
            Column::new("Value").align(Alignment::Right),
        ]);
        table.add_row(vec!["foo", "123"]);
        table.add_row(vec!["bar", "456"]);

        let output = table.render(80, false);
        assert!(output.contains("Name"));
        assert!(output.contains("foo"));
        assert!(output.contains("123"));
    }

    #[test]
    fn test_table_markdown() {
        let mut table = Table::new(vec![
            Column::new("A"),
            Column::new("B"),
        ]).style(TableStyle::Markdown);
        table.add_row(vec!["1", "2"]);

        let output = table.render(80, false);
        assert!(output.contains("|"));
        assert!(output.contains("---"));
    }

    #[test]
    fn test_text_wrap() {
        let formatter = TextFormatter::new(20);
        let wrapped = formatter.wrap("This is a long text that should wrap");
        assert!(wrapped.lines().count() > 1);
    }

    #[test]
    fn test_table_json_rows() {
        let mut table = Table::new(vec![
            Column::new("name"),
            Column::new("value"),
        ]);
        table.add_row(vec!["test", "123"]);

        let rows = table.to_json_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name").unwrap(), "test");
    }
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **080-cli-json.md**: JSON output mode
- **081-cli-color.md**: ANSI color support
- **091-cli-errors.md**: Error output formatting
