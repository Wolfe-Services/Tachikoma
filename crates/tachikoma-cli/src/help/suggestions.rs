//! Context-sensitive help suggestions.

use std::collections::HashMap;

use crate::output::color::{ColorMode, Styled, Color};

/// Help suggestions based on context
pub struct HelpSuggestions {
    suggestions: HashMap<SuggestionContext, Vec<&'static str>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SuggestionContext {
    /// No configuration found
    NoConfig,
    /// No backends configured
    NoBackends,
    /// No tools installed
    NoTools,
    /// Command failed
    CommandFailed(String),
    /// Unknown command
    UnknownCommand(String),
}

impl HelpSuggestions {
    pub fn new() -> Self {
        let mut suggestions = HashMap::new();

        suggestions.insert(SuggestionContext::NoConfig, vec![
            "Run 'tachikoma init' to create a new project",
            "Run 'tachikoma config init' to create a configuration file",
        ]);

        suggestions.insert(SuggestionContext::NoBackends, vec![
            "Run 'tachikoma backends add' to configure an AI backend",
            "Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable",
        ]);

        suggestions.insert(SuggestionContext::NoTools, vec![
            "Run 'tachikoma tools search' to find available tools",
            "Run 'tachikoma tools install filesystem' to install a tool",
        ]);

        Self { suggestions }
    }

    /// Get suggestions for a context
    pub fn get(&self, context: &SuggestionContext) -> Vec<&'static str> {
        self.suggestions.get(context).cloned().unwrap_or_default()
    }

    /// Suggest similar commands for typos
    pub fn suggest_command(input: &str, available: &[&str]) -> Option<String> {
        let input_lower = input.to_lowercase();

        // Find closest match using edit distance
        let mut best_match: Option<(&str, usize)> = None;

        for cmd in available {
            let distance = levenshtein(&input_lower, &cmd.to_lowercase());

            // Only suggest if distance is reasonable (less than half the word length)
            if distance <= input.len() / 2 + 1 {
                match best_match {
                    Some((_, best_dist)) if distance < best_dist => {
                        best_match = Some((cmd, distance));
                    }
                    None => {
                        best_match = Some((cmd, distance));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(cmd, _)| format!("Did you mean '{cmd}'?"))
    }

    /// Format suggestions for display
    pub fn format(suggestions: &[&str], color_mode: ColorMode) -> String {
        if suggestions.is_empty() {
            return String::new();
        }

        let header = Styled::new("Suggestions:")
            .with_color_mode(color_mode)
            .fg(Color::Yellow)
            .bold();

        let mut output = format!("\n{header}\n");
        
        for suggestion in suggestions {
            let bullet = Styled::new("*")
                .with_color_mode(color_mode)
                .fg(Color::Cyan);
            output.push_str(&format!("  {bullet} {suggestion}\n"));
        }
        output
    }

    /// Format command suggestion for typos
    pub fn format_command_suggestion(suggestion: &str, color_mode: ColorMode) -> String {
        let hint = Styled::new(suggestion)
            .with_color_mode(color_mode)
            .fg(Color::Yellow);
        format!("\n{hint}")
    }
}

impl Default for HelpSuggestions {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<_> = a.chars().collect();
    let b_chars: Vec<_> = b.chars().collect();

    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };

            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("init", "init"), 0);
        assert_eq!(levenshtein("init", "inot"), 1);
        assert_eq!(levenshtein("config", "confg"), 1);
        assert_eq!(levenshtein("tools", "tols"), 1);
    }

    #[test]
    fn test_suggest_command() {
        let commands = vec!["init", "doctor", "config", "tools", "backends"];

        assert_eq!(
            HelpSuggestions::suggest_command("initt", &commands),
            Some("Did you mean 'init'?".to_string())
        );

        assert_eq!(
            HelpSuggestions::suggest_command("docter", &commands),
            Some("Did you mean 'doctor'?".to_string())
        );

        // No match for very different input
        assert!(HelpSuggestions::suggest_command("xyz", &commands).is_none());
    }

    #[test]
    fn test_suggestions_format() {
        let suggestions = vec!["Run 'tachikoma init'", "Set ANTHROPIC_API_KEY"];
        let formatted = HelpSuggestions::format(&suggestions, ColorMode::Never);
        
        assert!(formatted.contains("Suggestions:"));
        assert!(formatted.contains("Run 'tachikoma init'"));
        assert!(formatted.contains("Set ANTHROPIC_API_KEY"));
    }

    #[test]
    fn test_context_suggestions() {
        let suggestions = HelpSuggestions::new();
        
        let no_config_suggestions = suggestions.get(&SuggestionContext::NoConfig);
        assert!(!no_config_suggestions.is_empty());
        assert!(no_config_suggestions.iter().any(|s| s.contains("tachikoma init")));
        
        let no_backends_suggestions = suggestions.get(&SuggestionContext::NoBackends);
        assert!(!no_backends_suggestions.is_empty());
        assert!(no_backends_suggestions.iter().any(|s| s.contains("backends add")));
    }
}