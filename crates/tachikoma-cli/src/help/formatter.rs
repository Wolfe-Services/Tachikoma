//! Custom help formatter.

use std::fmt::Write;

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

            let _ = writeln!(
                output,
                "  {name:<max_width$}  {about}",
                name = name,
                max_width = max_width,
                about = about
            );
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

            let _ = writeln!(output, "  {name}{required}");
            if !help.is_empty() {
                let _ = writeln!(output, "      {help}");
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

            let _ = writeln!(output, "  {name_str}");
            if !help.is_empty() || !default.is_empty() {
                let _ = writeln!(output, "      {help}{default}");
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