# Spec 076: CLI Crate Structure

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 076
- **Status**: Planned
- **Dependencies**: 001-workspace, 002-error
- **Estimated Context**: ~10%

## Objective

Create the foundational CLI crate structure using clap with derive macros, establishing the binary entry point and core command infrastructure for Tachikoma.

## Acceptance Criteria

- [x] CLI crate created at `crates/tachikoma-cli`
- [x] Binary named `tachikoma` produced
- [x] Clap derive macros configured with proper settings
- [x] Version, author, about information from Cargo.toml
- [x] Global flags defined (--verbose, --quiet, --config, --color)
- [x] Async runtime properly initialized
- [x] Clean error handling at entry point
- [x] Exit codes properly defined

## Implementation Details

### Cargo.toml

```toml
[package]
name = "tachikoma-cli"
version = "0.1.0"
edition = "2024"
authors = ["Tachikoma Contributors"]
description = "CLI for Tachikoma AI Agent Development Framework"
license = "MIT OR Apache-2.0"
readme = "README.md"
repository = "https://github.com/tachikoma/tachikoma"
keywords = ["ai", "agent", "cli", "mcp"]
categories = ["command-line-utilities", "development-tools"]

[[bin]]
name = "tachikoma"
path = "src/main.rs"

[dependencies]
tachikoma-core = { path = "../tachikoma-core" }
tachikoma-config = { path = "../tachikoma-config" }
tachikoma-mcp = { path = "../tachikoma-mcp" }

clap = { version = "4.5", features = ["derive", "env", "string", "wrap_help"] }
clap_complete = "4.5"
tokio = { version = "1.40", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
thiserror = "2.0"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.14"
```

### src/main.rs

```rust
//! Tachikoma CLI - AI Agent Development Framework
//!
//! Main entry point for the `tachikoma` binary.

use std::process::ExitCode;

use clap::Parser;
use tracing::error;

mod cli;
mod commands;
mod error;
mod output;

use cli::Cli;
use error::CliError;

/// Application exit codes
#[repr(u8)]
pub enum Exit {
    Success = 0,
    GeneralError = 1,
    ConfigError = 2,
    IoError = 3,
    NetworkError = 4,
    ValidationError = 5,
    Interrupted = 130,
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> Self {
        ExitCode::from(exit as u8)
    }
}

fn main() -> ExitCode {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing based on verbosity
    init_tracing(&cli);

    // Run the async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    match runtime.block_on(run(cli)) {
        Ok(()) => Exit::Success.into(),
        Err(e) => {
            error!("{e}");
            e.exit_code().into()
        }
    }
}

async fn run(cli: Cli) -> Result<(), CliError> {
    // Load configuration if specified
    let config = cli.load_config().await?;

    // Execute the command
    cli.execute(config).await
}

fn init_tracing(cli: &Cli) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = match cli.verbose {
        0 if cli.quiet => EnvFilter::new("error"),
        0 => EnvFilter::new("warn"),
        1 => EnvFilter::new("info"),
        2 => EnvFilter::new("debug"),
        _ => EnvFilter::new("trace"),
    };

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(cli.verbose >= 2));

    subscriber.init();
}
```

### src/cli.rs

```rust
//! CLI argument definitions using clap derive macros.

use std::path::PathBuf;

use clap::{ArgAction, ColorChoice, Parser, Subcommand, ValueHint};

use crate::commands::{
    BackendsCommand, ConfigCommand, DoctorCommand, InitCommand, ToolsCommand,
};
use crate::error::CliError;

/// Tachikoma - AI Agent Development Framework
///
/// Build, test, and deploy AI agents with MCP integration.
#[derive(Debug, Parser)]
#[command(
    name = "tachikoma",
    author,
    version,
    about,
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true,
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
)]
pub struct Cli {
    /// Increase verbosity level (-v, -vv, -vvv)
    #[arg(
        short,
        long,
        action = ArgAction::Count,
        global = true,
        help = "Increase verbosity level"
    )]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(
        short,
        long,
        global = true,
        conflicts_with = "verbose",
        help = "Suppress non-error output"
    )]
    pub quiet: bool,

    /// Path to configuration file
    #[arg(
        short,
        long,
        global = true,
        env = "TACHIKOMA_CONFIG",
        value_hint = ValueHint::FilePath,
        help = "Path to configuration file"
    )]
    pub config: Option<PathBuf>,

    /// When to use colors
    #[arg(
        long,
        global = true,
        default_value = "auto",
        value_enum,
        help = "When to use terminal colors"
    )]
    pub color: ColorChoice,

    /// Output format
    #[arg(
        long,
        global = true,
        default_value = "text",
        help = "Output format (text, json)"
    )]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Command,
}

/// Output format selection
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

/// Available subcommands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a new Tachikoma project
    Init(InitCommand),

    /// Manage configuration
    Config(ConfigCommand),

    /// Check system health and dependencies
    Doctor(DoctorCommand),

    /// Manage MCP tools
    Tools(ToolsCommand),

    /// Manage AI backends
    Backends(BackendsCommand),

    /// Generate shell completions
    #[command(hide = true)]
    Completions(CompletionsCommand),
}

/// Shell completions generation
#[derive(Debug, Parser)]
pub struct CompletionsCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

impl Cli {
    /// Load configuration from file or default locations
    pub async fn load_config(&self) -> Result<tachikoma_config::Config, CliError> {
        use tachikoma_config::{Config, ConfigLoader};

        let loader = ConfigLoader::new();

        match &self.config {
            Some(path) => loader.load_from_path(path).await.map_err(CliError::Config),
            None => loader.load_default().await.map_err(CliError::Config),
        }
    }

    /// Execute the selected command
    pub async fn execute(self, config: tachikoma_config::Config) -> Result<(), CliError> {
        let ctx = CommandContext {
            config,
            format: self.format,
            color: self.color,
            verbose: self.verbose,
        };

        match self.command {
            Command::Init(cmd) => cmd.execute(&ctx).await,
            Command::Config(cmd) => cmd.execute(&ctx).await,
            Command::Doctor(cmd) => cmd.execute(&ctx).await,
            Command::Tools(cmd) => cmd.execute(&ctx).await,
            Command::Backends(cmd) => cmd.execute(&ctx).await,
            Command::Completions(cmd) => cmd.execute(&ctx),
        }
    }
}

/// Context passed to all commands
#[derive(Debug)]
pub struct CommandContext {
    pub config: tachikoma_config::Config,
    pub format: OutputFormat,
    pub color: ColorChoice,
    pub verbose: u8,
}
```

### src/error.rs

```rust
//! CLI-specific error types.

use thiserror::Error;

use crate::Exit;

/// CLI error type
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(#[from] tachikoma_config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    /// Get the appropriate exit code for this error
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Config(_) => Exit::ConfigError.into(),
            Self::Io(_) => Exit::IoError.into(),
            Self::Network(_) => Exit::NetworkError.into(),
            Self::Validation(_) => Exit::ValidationError.into(),
            _ => Exit::GeneralError.into(),
        }
    }
}
```

### src/commands/mod.rs

```rust
//! Command implementations.

mod backends;
mod config;
mod doctor;
mod init;
mod tools;

pub use backends::BackendsCommand;
pub use config::ConfigCommand;
pub use doctor::DoctorCommand;
pub use init::InitCommand;
pub use tools::ToolsCommand;
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify the CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_verbose_flags() {
        let cli = Cli::try_parse_from(["tachikoma", "-vvv", "doctor"]).unwrap();
        assert_eq!(cli.verbose, 3);
    }

    #[test]
    fn parse_quiet_flag() {
        let cli = Cli::try_parse_from(["tachikoma", "--quiet", "doctor"]).unwrap();
        assert!(cli.quiet);
    }

    #[test]
    fn verbose_quiet_conflict() {
        let result = Cli::try_parse_from(["tachikoma", "-v", "--quiet", "doctor"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_config_path() {
        let cli = Cli::try_parse_from([
            "tachikoma",
            "--config",
            "/path/to/config.toml",
            "doctor",
        ])
        .unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.toml")));
    }

    #[test]
    fn parse_color_choice() {
        let cli = Cli::try_parse_from(["tachikoma", "--color", "never", "doctor"]).unwrap();
        assert_eq!(cli.color, ColorChoice::Never);
    }

    #[test]
    fn parse_json_format() {
        let cli = Cli::try_parse_from(["tachikoma", "--format", "json", "doctor"]).unwrap();
        assert_eq!(cli.format, OutputFormat::Json);
    }
}
```

### Integration Tests

```rust
// tests/cli_integration.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_flag() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI Agent Development Framework"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_no_args_shows_help() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_invalid_subcommand() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

## Related Specs

- **077-cli-args.md**: Detailed argument parsing patterns
- **078-cli-subcommands.md**: Subcommand structure and implementation
- **079-cli-output.md**: Output formatting utilities
- **091-cli-errors.md**: Error message formatting
