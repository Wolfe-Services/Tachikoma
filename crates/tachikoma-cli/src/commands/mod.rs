//! Command module organization and shared traits.

mod backends;
mod completions;
mod config;
mod doctor;
mod init;
mod manpages;
mod migrate;
mod tools;

pub use backends::BackendsCommand;
pub use completions::CompletionsCommand;
pub use config::ConfigCommand;
pub use doctor::DoctorCommand;
pub use init::InitCommand;
pub use manpages::ManpagesCommand;
pub use migrate::MigrateCommands;
pub use tools::ToolsCommand;

use async_trait::async_trait;

use crate::cli::CommandContext;
use crate::error::CliError;
use crate::output::{print_output, FormattedOutput};

/// Trait for executable commands
#[async_trait]
pub trait Execute {
    /// Execute the command with the given context
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError>;
}

/// Trait for commands that produce output
#[async_trait]
pub trait ExecuteWithOutput {
    type Output: serde::Serialize + FormattedOutput;

    /// Execute and return structured output
    async fn execute(&self, ctx: &CommandContext) -> Result<Self::Output, CliError>;
}

/// Helper to run a command with output formatting
pub async fn run_with_output<C, O>(
    command: &C,
    ctx: &CommandContext,
) -> Result<(), CliError>
where
    C: ExecuteWithOutput<Output = O>,
    O: serde::Serialize + FormattedOutput,
{
    let output = command.execute(ctx).await?;
    print_output(ctx, &output)?;
    Ok(())
}