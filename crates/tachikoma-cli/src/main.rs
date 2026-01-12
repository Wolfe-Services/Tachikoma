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