# 045 - Code Search Result Formatting

**Phase:** 2 - Five Primitives
**Spec ID:** 045
**Status:** Planned
**Dependencies:** 044-code-search-json
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement result formatting for code search output with multiple display modes, syntax highlighting hooks, and structured output formats.

---

## Acceptance Criteria

- [x] Multiple output formats (plain, colored, JSON)
- [x] Grouped by file option
- [x] Customizable context display
- [x] Match highlighting
- [x] Path formatting (relative/absolute)
- [x] Summary statistics formatting

---

## Implementation Details

### 1. Formatting Module (src/code_search/format.rs)

```rust
//! Result formatting for code search.

use crate::result::{CodeSearchResult, SearchMatch};
use serde::Serialize;
use std::fmt::Write as FmtWrite;
use std::path::Path;

/// Output format options.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Plain text output.
    #[default]
    Plain,
    /// Colored output for terminal.
    Colored,
    /// JSON output.
    Json,
    /// Grouped by file.
    Grouped,
}

/// Formatting configuration.
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Output format.
    pub format: OutputFormat,
    /// Show line numbers.
    pub line_numbers: bool,
    /// Show column numbers.
    pub column_numbers: bool,
    /// Show context lines.
    pub context: bool,
    /// Use relative paths.
    pub relative_paths: bool,
    /// Base path for relative path computation.
    pub base_path: Option<std::path::PathBuf>,
    /// Maximum line length before truncation.
    pub max_line_length: Option<usize>,
    /// Show file headers.
    pub file_headers: bool,
    /// Separator between matches.
    pub separator: String,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Plain,
            line_numbers: true,
            column_numbers: false,
            context: true,
            relative_paths: true,
            base_path: None,
            max_line_length: Some(200),
            file_headers: true,
            separator: "--".to_string(),
        }
    }
}

impl FormatConfig {
    /// Create new default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set output format.
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Hide line numbers.
    pub fn no_line_numbers(mut self) -> Self {
        self.line_numbers = false;
        self
    }

    /// Show column numbers.
    pub fn with_columns(mut self) -> Self {
        self.column_numbers = true;
        self
    }

    /// Hide context.
    pub fn no_context(mut self) -> Self {
        self.context = false;
        self
    }

    /// Use absolute paths.
    pub fn absolute_paths(mut self) -> Self {
        self.relative_paths = false;
        self
    }

    /// Set base path for relative paths.
    pub fn base_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.base_path = Some(path.into());
        self
    }
}

/// Format search results.
pub fn format_results(result: &CodeSearchResult, config: &FormatConfig) -> String {
    match config.format {
        OutputFormat::Plain => format_plain(result, config),
        OutputFormat::Colored => format_colored(result, config),
        OutputFormat::Json => format_json(result),
        OutputFormat::Grouped => format_grouped(result, config),
    }
}

/// Plain text formatting.
fn format_plain(result: &CodeSearchResult, config: &FormatConfig) -> String {
    let mut output = String::new();

    for m in &result.matches {
        let path = format_path(&m.path, config);

        if config.context && !m.context_before.is_empty() {
            for (i, ctx) in m.context_before.iter().enumerate() {
                let line_num = m.line_number - m.context_before.len() + i;
                writeln!(output, "{}-{}-{}", path, line_num, truncate_line(ctx, config)).ok();
            }
        }

        let location = if config.column_numbers {
            format!("{}:{}:{}", path, m.line_number, m.column)
        } else if config.line_numbers {
            format!("{}:{}", path, m.line_number)
        } else {
            path.to_string()
        };

        writeln!(output, "{}:{}", location, truncate_line(&m.line_content, config)).ok();

        if config.context && !m.context_after.is_empty() {
            for (i, ctx) in m.context_after.iter().enumerate() {
                let line_num = m.line_number + 1 + i;
                writeln!(output, "{}-{}-{}", path, line_num, truncate_line(ctx, config)).ok();
            }
        }

        if config.context && (!m.context_before.is_empty() || !m.context_after.is_empty()) {
            writeln!(output, "{}", config.separator).ok();
        }
    }

    if result.truncated {
        writeln!(output, "\n[Results truncated. {} total matches found]", result.total_count).ok();
    }

    output
}

/// Colored terminal formatting.
fn format_colored(result: &CodeSearchResult, config: &FormatConfig) -> String {
    let mut output = String::new();

    for m in &result.matches {
        let path = format_path(&m.path, config);

        // ANSI color codes
        let path_color = "\x1b[35m"; // Magenta
        let line_num_color = "\x1b[32m"; // Green
        let match_color = "\x1b[1;31m"; // Bold red
        let context_color = "\x1b[90m"; // Gray
        let reset = "\x1b[0m";

        if config.context && !m.context_before.is_empty() {
            for (i, ctx) in m.context_before.iter().enumerate() {
                let line_num = m.line_number - m.context_before.len() + i;
                writeln!(
                    output,
                    "{}{}{}-{}{}{}-{}{}",
                    path_color, path, reset,
                    line_num_color, line_num, reset,
                    context_color, ctx
                ).ok();
            }
        }

        let location = format!(
            "{}{}{}:{}{}{}",
            path_color, path, reset,
            line_num_color, m.line_number, reset
        );

        // TODO: Highlight the actual match within the line
        writeln!(output, "{}:{}", location, m.line_content).ok();

        if config.context && !m.context_after.is_empty() {
            for (i, ctx) in m.context_after.iter().enumerate() {
                let line_num = m.line_number + 1 + i;
                writeln!(
                    output,
                    "{}{}{}-{}{}{}-{}{}",
                    path_color, path, reset,
                    line_num_color, line_num, reset,
                    context_color, ctx
                ).ok();
            }
            writeln!(output, "{}", config.separator).ok();
        }
    }

    output
}

/// JSON formatting.
fn format_json(result: &CodeSearchResult) -> String {
    #[derive(Serialize)]
    struct JsonOutput<'a> {
        pattern: &'a str,
        total_count: usize,
        truncated: bool,
        matches: Vec<JsonMatch<'a>>,
    }

    #[derive(Serialize)]
    struct JsonMatch<'a> {
        path: &'a Path,
        line_number: usize,
        column: usize,
        content: &'a str,
        context_before: &'a [String],
        context_after: &'a [String],
    }

    let output = JsonOutput {
        pattern: &result.pattern,
        total_count: result.total_count,
        truncated: result.truncated,
        matches: result
            .matches
            .iter()
            .map(|m| JsonMatch {
                path: &m.path,
                line_number: m.line_number,
                column: m.column,
                content: &m.line_content,
                context_before: &m.context_before,
                context_after: &m.context_after,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}

/// Grouped by file formatting.
fn format_grouped(result: &CodeSearchResult, config: &FormatConfig) -> String {
    use std::collections::BTreeMap;

    let mut by_file: BTreeMap<&Path, Vec<&SearchMatch>> = BTreeMap::new();

    for m in &result.matches {
        by_file.entry(&m.path).or_default().push(m);
    }

    let mut output = String::new();

    for (path, matches) in by_file {
        let path_str = format_path(path, config);
        writeln!(output, "\n{}", path_str).ok();
        writeln!(output, "{}", "=".repeat(path_str.len())).ok();

        for m in matches {
            if config.context && !m.context_before.is_empty() {
                for ctx in &m.context_before {
                    writeln!(output, "  {}", ctx).ok();
                }
            }

            if config.line_numbers {
                writeln!(output, "{:>4}: {}", m.line_number, m.line_content).ok();
            } else {
                writeln!(output, "  {}", m.line_content).ok();
            }

            if config.context && !m.context_after.is_empty() {
                for ctx in &m.context_after {
                    writeln!(output, "  {}", ctx).ok();
                }
                writeln!(output).ok();
            }
        }
    }

    if result.truncated {
        writeln!(output, "\n[{} matches total, results truncated]", result.total_count).ok();
    }

    output
}

/// Format path according to config.
fn format_path(path: &Path, config: &FormatConfig) -> String {
    if config.relative_paths {
        if let Some(ref base) = config.base_path {
            if let Ok(relative) = path.strip_prefix(base) {
                return relative.display().to_string();
            }
        }
    }
    path.display().to_string()
}

/// Truncate line to max length.
fn truncate_line(line: &str, config: &FormatConfig) -> String {
    if let Some(max) = config.max_line_length {
        if line.len() > max {
            return format!("{}...", &line[..max]);
        }
    }
    line.to_string()
}

/// Format summary statistics.
pub fn format_summary(result: &CodeSearchResult) -> String {
    format!(
        "{} matches found for pattern '{}'{}",
        result.total_count,
        result.pattern,
        if result.truncated { " (truncated)" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::ExecutionMetadata;
    use std::path::PathBuf;
    use std::time::Duration;

    fn create_test_result() -> CodeSearchResult {
        CodeSearchResult {
            matches: vec![
                SearchMatch {
                    path: PathBuf::from("/project/src/main.rs"),
                    line_number: 10,
                    column: 5,
                    line_content: "fn main() {}".to_string(),
                    context_before: vec!["// comment".to_string()],
                    context_after: vec!["    println!();".to_string()],
                },
            ],
            pattern: "main".to_string(),
            total_count: 1,
            truncated: false,
            metadata: ExecutionMetadata {
                duration: Duration::from_millis(100),
                operation_id: "test".to_string(),
                primitive: "code_search".to_string(),
            },
        }
    }

    #[test]
    fn test_format_plain() {
        let result = create_test_result();
        let config = FormatConfig::new();
        let output = format_results(&result, &config);

        assert!(output.contains("main.rs"));
        assert!(output.contains("fn main()"));
    }

    #[test]
    fn test_format_json() {
        let result = create_test_result();
        let config = FormatConfig::new().format(OutputFormat::Json);
        let output = format_results(&result, &config);

        assert!(output.contains("\"pattern\""));
        assert!(output.contains("\"main\""));
    }

    #[test]
    fn test_format_grouped() {
        let result = create_test_result();
        let config = FormatConfig::new().format(OutputFormat::Grouped);
        let output = format_results(&result, &config);

        assert!(output.contains("main.rs"));
        assert!(output.contains("==="));
    }

    #[test]
    fn test_relative_paths() {
        let result = create_test_result();
        let config = FormatConfig::new()
            .base_path("/project");
        let output = format_results(&result, &config);

        assert!(output.contains("src/main.rs"));
        assert!(!output.contains("/project/src/main.rs"));
    }
}
```

---

## Testing Requirements

1. Plain format shows all expected fields
2. Colored format includes ANSI codes
3. JSON format is valid JSON
4. Grouped format groups by file
5. Relative paths are computed correctly
6. Line truncation works
7. Summary statistics are accurate
8. Context lines are displayed correctly

---

## Related Specs

- Depends on: [044-code-search-json.md](044-code-search-json.md)
- Next: [046-primitives-trait.md](046-primitives-trait.md)
- Related: [033-read-file-errors.md](033-read-file-errors.md)
