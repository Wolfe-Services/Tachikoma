use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tachikoma_database::database::migration::cli::{MigrateCli, MigrateCommand};

use crate::cli::CommandContext;
use crate::error::CliError;

/// Database migration management commands
#[derive(Debug, Parser)]
pub struct MigrateCommands {
    #[command(subcommand)]
    pub command: MigrateCommand,

    /// Database URL
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:tachikoma.db")]
    pub database_url: String,

    /// Migrations directory
    #[arg(long, default_value = "migrations")]
    pub migrations_dir: PathBuf,
}

impl MigrateCommands {
    /// Execute the migrate command
    pub async fn execute(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        let migrate_cli = MigrateCli {
            command: self.command.clone(),
            database_url: self.database_url.clone(),
            migrations_dir: self.migrations_dir.clone(),
        };

        migrate_cli.run().await.map_err(CliError::from)
    }
}