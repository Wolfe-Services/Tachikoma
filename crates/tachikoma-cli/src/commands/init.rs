//! Init command implementation.

use clap::Parser;

use crate::cli::CommandContext;
use crate::error::CliError;

/// Initialize a new Tachikoma project
#[derive(Debug, Parser)]
pub struct InitCommand {
    // TODO: Add options for project initialization
}

impl InitCommand {
    pub async fn execute(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        println!("Init command not yet implemented");
        Ok(())
    }
}