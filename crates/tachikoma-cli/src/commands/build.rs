//! Build command implementation.

use async_trait::async_trait;
use clap::Parser;

use crate::cli::CommandContext;
use crate::commands::Execute;
use crate::error::CliError;

/// Build the project
#[derive(Debug, Parser)]
pub struct BuildCommand {
    /// Release mode
    #[arg(short, long)]
    pub release: bool,

    /// Watch for changes and rebuild
    #[arg(short, long)]
    pub watch: bool,
}

#[async_trait]
impl Execute for BuildCommand {
    async fn execute(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        println!("Build command not yet implemented");
        Ok(())
    }
}