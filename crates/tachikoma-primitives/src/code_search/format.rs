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
                    "{}{}{}-{}{}{}-{}{}{}",
                    path_color, path, reset,
                    line_num_color, line_num, reset,
                    context_color, ctx, reset
                ).ok();
            }
        }

        let location = if config.column_numbers {
            format!(
                "{}{}{}:{}{}{}:{}{}{}",
                path_color, path, reset,
                line_num_color, m.line_number, reset,
                line_num_color, m.column, reset
            )
        } else if config.line_numbers {
            format!(
                "{}{}{}:{}{}{}",
                path_color, path, reset,
                line_num_color, m.line_number, reset
            )
        } else {
            format!("{}{}{}", path_color, path, reset)
        };

        // Highlight matches in the line content
        let highlighted_content = highlight_match(&m.line_content, &result.pattern);
        writeln!(output, "{}:{}", location, highlighted_content).ok();

        if config.context && !m.context_after.is_empty() {
            for (i, ctx) in m.context_after.iter().enumerate() {
                let line_num = m.line_number + 1 + i;
                writeln!(
                    output,
                    "{}{}{}-{}{}{}-{}{}{}",
                    path_color, path, reset,
                    line_num_color, line_num, reset,
                    context_color, ctx, reset
                ).ok();
            }
            writeln!(output, "{}", config.separator).ok();
        }
    }

    if result.truncated {
        writeln!(output, "\n[Results truncated. {} total matches found]", result.total_count).ok();
    }

    output
}

/// Highlight matches in the line content.
fn highlight_match(line: &str, pattern: &str) -> String {
    let match_color = "\x1b[1;31m"; // Bold red
    let reset = "\x1b[0m";
    
    // Try multiple highlighting strategies
    
    // First, try the pattern as-is (in case it's a valid regex)
    if let Ok(regex) = regex::Regex::new(pattern) {
        return regex.replace_all(line, &format!("{}{}{}", match_color, "$0", reset)).to_string();
    }
    
    // If that fails, try case-insensitive literal matching
    let escaped = regex::escape(pattern);
    if let Ok(regex) = regex::Regex::new(&format!("(?i){}", escaped)) {
        return regex.replace_all(line, &format!("{}{}{}", match_color, "$0", reset)).to_string();
    }
    
    // Fallback to simple case-insensitive string replacement
    let lower_line = line.to_lowercase();
    let lower_pattern = pattern.to_lowercase();
    
    let mut result = String::new();
    let mut last_end = 0;
    let mut search_from = 0;
    
    while let Some(match_pos) = lower_line[search_from..].find(&lower_pattern) {
        let actual_pos = search_from + match_pos;
        let match_end = actual_pos + pattern.len();
        
        // Add text before match
        result.push_str(&line[last_end..actual_pos]);
        
        // Add highlighted match (preserve original casing)
        result.push_str(&format!("{}{}{}", match_color, &line[actual_pos..match_end], reset));
        
        last_end = match_end;
        search_from = match_end;
    }
    
    // Add remaining text
    result.push_str(&line[last_end..]);
    
    // If no matches were found, return the original line
    if last_end == 0 {
        return line.to_string();
    }
    
    result
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

    #[test]
    fn test_colored_output() {
        let result = create_test_result();
        let config = FormatConfig::new().format(OutputFormat::Colored);
        let output = format_results(&result, &config);

        assert!(output.contains("\x1b[35m")); // Magenta color
        assert!(output.contains("\x1b[32m")); // Green color
        assert!(output.contains("\x1b[0m"));  // Reset
    }

    #[test]
    fn test_line_truncation() {
        let mut result = create_test_result();
        result.matches[0].line_content = "a".repeat(300);
        
        let config = FormatConfig::new();
        let output = format_results(&result, &config);

        assert!(output.contains("..."));
        assert!(output.len() < 350); // Should be truncated
    }

    #[test]
    fn test_summary_formatting() {
        let result = create_test_result();
        let summary = format_summary(&result);

        assert!(summary.contains("1 matches found"));
        assert!(summary.contains("main"));
        assert!(!summary.contains("truncated"));

        let mut truncated_result = result;
        truncated_result.truncated = true;
        let truncated_summary = format_summary(&truncated_result);
        assert!(truncated_summary.contains("truncated"));
    }

    #[test]
    fn test_match_highlighting() {
        // Test basic highlighting
        let highlighted = highlight_match("fn main() {}", "main");
        println!("Basic highlighting result: '{}'", highlighted);
        assert!(highlighted.contains("\x1b[1;31mmain\x1b[0m"));
        
        // Test case insensitive highlighting
        let highlighted = highlight_match("FN MAIN() {}", "main");
        println!("Case insensitive result: '{}'", highlighted);
        assert!(highlighted.contains("\x1b[1;31mMAIN\x1b[0m"));
        
        // Test regex pattern
        let highlighted = highlight_match("test123", r"\d+");
        assert!(highlighted.contains("\x1b[1;31m123\x1b[0m"));
        
        // Test no match
        let highlighted = highlight_match("hello world", "xyz");
        assert_eq!(highlighted, "hello world");
        
        // Test multiple matches
        let highlighted = highlight_match("test test test", "test");
        let match_count = highlighted.matches("\x1b[1;31mtest\x1b[0m").count();
        assert_eq!(match_count, 3);
    }

    #[test]
    fn test_format_config_builders() {
        let config = FormatConfig::new()
            .format(OutputFormat::Colored)
            .no_line_numbers()
            .with_columns()
            .no_context()
            .absolute_paths()
            .base_path("/tmp");

        assert!(matches!(config.format, OutputFormat::Colored));
        assert!(!config.line_numbers);
        assert!(config.column_numbers);
        assert!(!config.context);
        assert!(!config.relative_paths);
        assert_eq!(config.base_path, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn test_multiple_files_grouped() {
        let mut result = create_test_result();
        result.matches.push(SearchMatch {
            path: PathBuf::from("/project/src/lib.rs"),
            line_number: 5,
            column: 1,
            line_content: "pub fn main_function() {}".to_string(),
            context_before: vec![],
            context_after: vec![],
        });
        
        let config = FormatConfig::new().format(OutputFormat::Grouped);
        let output = format_results(&result, &config);
        
        // Should contain both files
        assert!(output.contains("main.rs"));
        assert!(output.contains("lib.rs"));
        // Should have file separators
        assert!(output.matches("===").count() >= 2);
    }
}