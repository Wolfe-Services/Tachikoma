# Spec 090: Help System

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 090
- **Status**: Planned
- **Dependencies**: 076-cli-crate
- **Estimated Context**: ~8%

## Objective

Implement a comprehensive help system for the CLI with consistent formatting, examples, and context-sensitive help across all commands.

## Acceptance Criteria

- [ ] Consistent help formatting across commands
- [ ] Command examples in help output
- [ ] Long help with `--help` vs short help with `-h`
- [ ] Subcommand help discovery
- [ ] Context-sensitive help suggestions
- [ ] Colorized help output
- [ ] Help search functionality
- [ ] Online documentation links

## Implementation Details

### src/help/mod.rs

```rust
//! Help system for the CLI.

mod formatter;
mod examples;
mod suggestions;

pub use formatter::HelpFormatter;
pub use examples::CommandExamples;
pub use suggestions::HelpSuggestions;

use clap::Command;

/// Custom help template for consistent formatting
pub const HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

/// Extended help template with examples
pub const LONG_HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}";

/// Configure clap command with custom help settings
pub fn configure_help(cmd: Command) -> Command {
    cmd.help_template(HELP_TEMPLATE)
        .disable_help_flag(false)
        .disable_help_subcommand(false)
        .arg_required_else_help(true)
        .subcommand_required(false)
        .after_help(get_after_help())
        .after_long_help(get_after_long_help())
}

fn get_after_help() -> &'static str {
    "Run 'tachikoma <command> --help' for more information on a command."
}

fn get_after_long_help() -> &'static str {
    "\
Examples:
  tachikoma init my-project          Create a new project
  tachikoma doctor                   Check system health
  tachikoma backends add anthropic   Add an AI backend
  tachikoma tools list               List available tools

Documentation: https://docs.tachikoma.dev
Report bugs: https://github.com/tachikoma/tachikoma/issues"
}
```

### src/help/formatter.rs

```rust
//! Custom help formatter.

use std::io::Write;

use clap::{Arg, ArgAction, Command};

use crate::output::color::{ColorMode, Styled, Color};

/// Custom help formatter with color support
pub struct HelpFormatter {
    color_mode: ColorMode,
    max_width: usize,
}

impl HelpFormatter {
    pub fn new() -> Self {
        Self {
            color_mode: ColorMode::Auto,
            max_width: crate::output::terminal_width(),
        }
    }

    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Format help for a command
    pub fn format(&self, cmd: &Command) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.format_header(cmd));
        output.push('\n');

        // Usage
        output.push_str(&self.format_usage(cmd));
        output.push_str("\n\n");

        // Description
        if let Some(about) = cmd.get_about() {
            output.push_str(&self.format_about(about.to_string()));
            output.push_str("\n\n");
        }

        // Subcommands
        let subcommands: Vec<_> = cmd.get_subcommands().collect();
        if !subcommands.is_empty() {
            output.push_str(&self.format_subcommands(&subcommands));
            output.push('\n');
        }

        // Arguments
        let args: Vec<_> = cmd.get_arguments().collect();
        let positionals: Vec<_> = args.iter()
            .filter(|a| a.is_positional())
            .cloned()
            .collect();
        let options: Vec<_> = args.iter()
            .filter(|a| !a.is_positional())
            .cloned()
            .collect();

        if !positionals.is_empty() {
            output.push_str(&self.format_arguments(&positionals));
            output.push('\n');
        }

        if !options.is_empty() {
            output.push_str(&self.format_options(&options));
            output.push('\n');
        }

        output
    }

    fn format_header(&self, cmd: &Command) -> String {
        let name = Styled::new(cmd.get_name())
            .with_color_mode(self.color_mode)
            .fg(Color::Green)
            .bold();

        let version = cmd.get_version().unwrap_or("0.0.0");

        format!("{name} {version}")
    }

    fn format_usage(&self, cmd: &Command) -> String {
        let header = Styled::new("Usage:")
            .with_color_mode(self.color_mode)
            .fg(Color::Yellow)
            .bold();

        let mut usage = cmd.get_name().to_string();

        // Add subcommand placeholder if has subcommands
        if cmd.has_subcommands() {
            usage.push_str(" <COMMAND>");
        }

        // Add options placeholder if has options
        let has_options = cmd.get_arguments().any(|a| !a.is_positional());
        if has_options {
            usage.push_str(" [OPTIONS]");
        }

        // Add positional arguments
        for arg in cmd.get_arguments() {
            if arg.is_positional() {
                let name = arg.get_id().as_str().to_uppercase();
                if arg.is_required_set() {
                    usage.push_str(&format!(" <{name}>"));
                } else {
                    usage.push_str(&format!(" [{name}]"));
                }
            }
        }

        format!("{header} {usage}")
    }

    fn format_about(&self, about: String) -> String {
        textwrap::fill(&about, self.max_width)
    }

    fn format_subcommands(&self, subcommands: &[&Command]) -> String {
        let header = Styled::new("Commands:")
            .with_color_mode(self.color_mode)
            .fg(Color::Yellow)
            .bold();

        let mut output = format!("{header}\n");

        // Calculate max name width
        let max_width = subcommands
            .iter()
            .map(|c| c.get_name().len())
            .max()
            .unwrap_or(0)
            .max(12);

        for cmd in subcommands {
            if cmd.is_hide_set() {
                continue;
            }

            let name = Styled::new(cmd.get_name())
                .with_color_mode(self.color_mode)
                .fg(Color::Cyan);

            let about = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();

            output.push_str(&format!(
                "  {name:<max_width$}  {about}\n",
                max_width = max_width
            ));
        }

        output
    }

    fn format_arguments(&self, args: &[&Arg]) -> String {
        let header = Styled::new("Arguments:")
            .with_color_mode(self.color_mode)
            .fg(Color::Yellow)
            .bold();

        let mut output = format!("{header}\n");

        for arg in args {
            let name = Styled::new(format!("<{}>", arg.get_id().as_str().to_uppercase()))
                .with_color_mode(self.color_mode)
                .fg(Color::Cyan);

            let help = arg.get_help().map(|s| s.to_string()).unwrap_or_default();

            let required = if arg.is_required_set() {
                Styled::new(" (required)")
                    .with_color_mode(self.color_mode)
                    .fg(Color::Red)
            } else {
                Styled::new("")
            };

            output.push_str(&format!("  {name}{required}\n"));
            if !help.is_empty() {
                output.push_str(&format!("      {help}\n"));
            }
        }

        output
    }

    fn format_options(&self, args: &[&Arg]) -> String {
        let header = Styled::new("Options:")
            .with_color_mode(self.color_mode)
            .fg(Color::Yellow)
            .bold();

        let mut output = format!("{header}\n");

        for arg in args {
            let mut names = Vec::new();

            if let Some(short) = arg.get_short() {
                names.push(format!("-{short}"));
            }

            if let Some(long) = arg.get_long() {
                names.push(format!("--{long}"));
            }

            // Add value name for arguments that take values
            let value_name = if !matches!(arg.get_action(), ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count) {
                arg.get_value_names()
                    .map(|n| format!(" <{}>", n.first().map(|s| s.as_str()).unwrap_or("VALUE")))
                    .unwrap_or_else(|| " <VALUE>".to_string())
            } else {
                String::new()
            };

            let name_str = Styled::new(format!("{}{}", names.join(", "), value_name))
                .with_color_mode(self.color_mode)
                .fg(Color::Cyan);

            let help = arg.get_help().map(|s| s.to_string()).unwrap_or_default();

            // Add default value if present
            let default = if let Some(default) = arg.get_default_values().first() {
                format!(
                    " {}",
                    Styled::new(format!("[default: {}]", default.to_string_lossy()))
                        .with_color_mode(self.color_mode)
                        .fg(Color::BrightBlack)
                )
            } else {
                String::new()
            };

            output.push_str(&format!("  {name_str}\n"));
            if !help.is_empty() || !default.is_empty() {
                output.push_str(&format!("      {help}{default}\n"));
            }
        }

        output
    }
}

impl Default for HelpFormatter {
    fn default() -> Self {
        Self::new()
    }
}
```

### src/help/examples.rs

```rust
//! Command examples.

use std::collections::HashMap;

/// Command examples collection
pub struct CommandExamples {
    examples: HashMap<&'static str, Vec<Example>>,
}

/// A single example
pub struct Example {
    pub description: &'static str,
    pub command: &'static str,
    pub output: Option<&'static str>,
}

impl CommandExamples {
    pub fn new() -> Self {
        let mut examples = HashMap::new();

        // Init command examples
        examples.insert("init", vec![
            Example {
                description: "Create a new project in the current directory",
                command: "tachikoma init my-project",
                output: None,
            },
            Example {
                description: "Create a project with tools template",
                command: "tachikoma init my-project --template tools",
                output: None,
            },
            Example {
                description: "Create project interactively",
                command: "tachikoma init",
                output: None,
            },
        ]);

        // Doctor command examples
        examples.insert("doctor", vec![
            Example {
                description: "Run all health checks",
                command: "tachikoma doctor",
                output: Some("System check: OK\nConfig check: OK"),
            },
            Example {
                description: "Run specific category checks",
                command: "tachikoma doctor --category backends",
                output: None,
            },
            Example {
                description: "Output as JSON",
                command: "tachikoma --format json doctor",
                output: Some("{\"checks\": [...]}"),
            },
        ]);

        // Config command examples
        examples.insert("config", vec![
            Example {
                description: "List all configuration",
                command: "tachikoma config list",
                output: None,
            },
            Example {
                description: "Get a specific value",
                command: "tachikoma config get backend.default",
                output: Some("anthropic"),
            },
            Example {
                description: "Set a configuration value",
                command: "tachikoma config set agent.temperature 0.8",
                output: None,
            },
        ]);

        // Tools command examples
        examples.insert("tools", vec![
            Example {
                description: "List installed tools",
                command: "tachikoma tools list",
                output: None,
            },
            Example {
                description: "Install a tool",
                command: "tachikoma tools install filesystem",
                output: None,
            },
            Example {
                description: "Search for tools",
                command: "tachikoma tools search database",
                output: None,
            },
            Example {
                description: "Test a tool",
                command: "tachikoma tools test filesystem --input '{\"path\": \".\"}'",
                output: None,
            },
        ]);

        // Backends command examples
        examples.insert("backends", vec![
            Example {
                description: "List configured backends",
                command: "tachikoma backends list",
                output: None,
            },
            Example {
                description: "Add Anthropic backend",
                command: "tachikoma backends add anthropic --backend-type anthropic",
                output: None,
            },
            Example {
                description: "Test backend connectivity",
                command: "tachikoma backends test anthropic",
                output: None,
            },
            Example {
                description: "List available models",
                command: "tachikoma backends models --refresh",
                output: None,
            },
        ]);

        Self { examples }
    }

    /// Get examples for a command
    pub fn get(&self, command: &str) -> Option<&Vec<Example>> {
        self.examples.get(command)
    }

    /// Format examples for display
    pub fn format(&self, command: &str, color: bool) -> Option<String> {
        let examples = self.get(command)?;

        let mut output = String::new();
        output.push_str("Examples:\n");

        for example in examples {
            output.push_str(&format!("\n  # {}\n", example.description));

            if color {
                output.push_str(&format!("  \x1b[36m$ {}\x1b[0m\n", example.command));
            } else {
                output.push_str(&format!("  $ {}\n", example.command));
            }

            if let Some(expected_output) = example.output {
                for line in expected_output.lines() {
                    output.push_str(&format!("  {line}\n"));
                }
            }
        }

        Some(output)
    }
}

impl Default for CommandExamples {
    fn default() -> Self {
        Self::new()
    }
}
```

### src/help/suggestions.rs

```rust
//! Context-sensitive help suggestions.

use std::collections::HashMap;

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
    pub fn format(suggestions: &[&str]) -> String {
        if suggestions.is_empty() {
            return String::new();
        }

        let mut output = String::from("\nSuggestions:\n");
        for suggestion in suggestions {
            output.push_str(&format!("  * {suggestion}\n"));
        }
        output
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
```

## Testing Requirements

### Unit Tests

```rust
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
    fn test_examples_format() {
        let examples = CommandExamples::new();
        let formatted = examples.format("init", false);

        assert!(formatted.is_some());
        assert!(formatted.unwrap().contains("tachikoma init"));
    }
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **091-cli-errors.md**: Error help integration
- **094-cli-man.md**: Man page generation
