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
            .map(|s| format!("  • {}", s.as_ref()))
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

    /// Format a definition list
    pub fn definition_list<I>(&self, items: I) -> String
    where
        I: IntoIterator<Item = (String, String)>,
    {
        items
            .into_iter()
            .map(|(key, value)| self.key_value(&key, &value))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format code block with optional syntax highlighting hint
    pub fn code_block(&self, code: &str, _language: Option<&str>) -> String {
        let lines: Vec<_> = code
            .lines()
            .map(|line| format!("    {line}"))
            .collect();
        format!("```\n{}\n```", lines.join("\n"))
    }

    /// Format a horizontal rule
    pub fn hr(&self) -> String {
        "─".repeat(self.width.min(80))
    }

    /// Format an info box
    pub fn info_box(&self, title: &str, content: &str) -> String {
        let content_lines = self.wrap(content);
        let max_line_len = content_lines
            .lines()
            .map(|line| line.trim().len())
            .max()
            .unwrap_or(0);
        let box_width = max_line_len.max(title.len()).min(self.width);

        let mut result = String::new();
        result.push_str(&format!("┌{}┐\n", "─".repeat(box_width + 2)));
        result.push_str(&format!("│ {title:^box_width$} │\n"));
        result.push_str(&format!("├{}┤\n", "─".repeat(box_width + 2)));
        
        for line in content_lines.lines() {
            let trimmed = line.trim();
            result.push_str(&format!("│ {trimmed:<box_width$} │\n"));
        }
        
        result.push_str(&format!("└{}┘", "─".repeat(box_width + 2)));
        result
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

/// Helper for formatting progress indicators
pub struct ProgressIndicator {
    width: usize,
    filled_char: char,
    empty_char: char,
}

impl ProgressIndicator {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            filled_char: '█',
            empty_char: '░',
        }
    }

    pub fn with_chars(mut self, filled: char, empty: char) -> Self {
        self.filled_char = filled;
        self.empty_char = empty;
        self
    }

    /// Format a progress bar
    pub fn bar(&self, progress: f64, show_percent: bool) -> String {
        let progress = progress.clamp(0.0, 1.0);
        let filled_width = (progress * self.width as f64) as usize;
        let empty_width = self.width.saturating_sub(filled_width);

        let bar = format!(
            "{}{}",
            self.filled_char.to_string().repeat(filled_width),
            self.empty_char.to_string().repeat(empty_width)
        );

        if show_percent {
            format!("{bar} {:.1}%", progress * 100.0)
        } else {
            bar
        }
    }

    /// Format a spinner (for indeterminate progress)
    pub fn spinner(&self, frame: usize) -> String {
        const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let frame_char = FRAMES[frame % FRAMES.len()];
        format!("{frame_char} ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_wrap() {
        let formatter = TextFormatter::new(20);
        let wrapped = formatter.wrap("This is a very long text that should wrap to multiple lines");
        assert!(wrapped.lines().count() > 1);
        
        // Each line should be within width
        for line in wrapped.lines() {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_text_wrap_with_indent() {
        let formatter = TextFormatter::new(20).with_indent(4);
        let wrapped = formatter.wrap("This should be indented");
        assert!(wrapped.starts_with("    "));
    }

    #[test]
    fn test_key_value_formatting() {
        let formatter = TextFormatter::new(80);
        let output = formatter.key_value("Name", "Tachikoma");
        assert!(output.contains("Name"));
        assert!(output.contains("Tachikoma"));
    }

    #[test]
    fn test_section_header() {
        let formatter = TextFormatter::new(80);
        let section = formatter.section("Overview");
        assert!(section.contains("Overview"));
        assert!(section.contains("========"));
    }

    #[test]
    fn test_bullet_list() {
        let formatter = TextFormatter::new(80);
        let items = vec!["First item", "Second item", "Third item"];
        let list = formatter.bullet_list(items);
        assert!(list.contains("• First item"));
        assert!(list.contains("• Second item"));
    }

    #[test]
    fn test_numbered_list() {
        let formatter = TextFormatter::new(80);
        let items = vec!["First", "Second", "Third"];
        let list = formatter.numbered_list(items);
        assert!(list.contains("1. First"));
        assert!(list.contains("2. Second"));
        assert!(list.contains("3. Third"));
    }

    #[test]
    fn test_wrapped_display() {
        let text = "This is a long line that should wrap";
        let wrapped = Wrapped::new(text, 20);
        let output = wrapped.to_string();
        assert!(output.lines().count() > 1);
    }

    #[test]
    fn test_progress_bar() {
        let indicator = ProgressIndicator::new(10);
        let bar = indicator.bar(0.5, true);
        assert!(bar.contains("50.0%"));
        assert_eq!(bar.chars().filter(|&c| c == '█').count(), 5);
    }

    #[test]
    fn test_spinner() {
        let indicator = ProgressIndicator::new(10);
        let spinner1 = indicator.spinner(0);
        let spinner2 = indicator.spinner(1);
        assert_ne!(spinner1, spinner2);
        assert!(spinner1.len() > 0);
    }

    #[test]
    fn test_code_block() {
        let formatter = TextFormatter::new(80);
        let code = "fn main() {\n    println!(\"Hello\");\n}";
        let block = formatter.code_block(code, Some("rust"));
        assert!(block.starts_with("```"));
        assert!(block.ends_with("```"));
        assert!(block.contains("    fn main()"));
    }

    #[test]
    fn test_info_box() {
        let formatter = TextFormatter::new(80);
        let box_output = formatter.info_box("Important", "This is important information");
        assert!(box_output.contains("Important"));
        assert!(box_output.contains("┌"));
        assert!(box_output.contains("└"));
        assert!(box_output.contains("important information"));
    }
}