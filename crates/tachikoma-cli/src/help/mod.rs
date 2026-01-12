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