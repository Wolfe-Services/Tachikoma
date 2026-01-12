//! Backends command implementation.

use clap::Parser;

use crate::cli::CommandContext;
use crate::error::CliError;

/// Manage AI backends
#[derive(Debug, Parser)]
pub struct BackendsCommand {
    // TODO: Add subcommands for backend management
}

impl BackendsCommand {
    pub async fn execute(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        println!("Backends command not yet implemented");
        Ok(())
    }
}