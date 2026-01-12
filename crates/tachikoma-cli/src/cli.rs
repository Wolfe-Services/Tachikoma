//! CLI argument definitions using clap derive macros.

use std::path::PathBuf;

use clap::{ArgAction, ColorChoice, Parser, Subcommand, ValueHint};

use crate::commands::{
    BackendsCommand, ConfigCommand, DoctorCommand, 
    InitCommand, ToolsCommand, CompletionsCommand, ManpagesCommand,
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
    // === Project Management ===

    /// Initialize a new Tachikoma project
    #[command(visible_alias = "new")]
    Init(InitCommand),

    /// Run an agent or workflow
    Run(RunCommand),

    /// Build the project
    Build(BuildCommand),

    /// Run tests
    Test(TestCommand),

    // === Configuration ===

    /// Manage configuration
    Config(ConfigCommand),

    // === System ===

    /// Check system health and dependencies
    Doctor(DoctorCommand),

    // === Resources ===

    /// Manage MCP tools
    Tools(ToolsCommand),

    /// Manage AI backends
    Backends(BackendsCommand),

    /// Manage prompts
    Prompts(PromptsCommand),

    /// Manage workflows
    Workflows(WorkflowsCommand),

    // === Development ===

    /// Start development server
    Dev(DevCommand),

    /// Format code and configurations
    Fmt(FmtCommand),

    /// Lint code and configurations
    Lint(LintCommand),

    // === Utilities ===

    /// Generate shell completions
    #[command(hide = true)]
    Completions(CompletionsCommand),

    /// Generate man pages
    #[command(hide = true)]
    Manpages(ManpagesCommand),
}

/// Shell completions generation
#[derive(Debug, Parser)]
pub struct CompletionsCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

/// Generate man pages
#[derive(Debug, Parser)]
pub struct ManpagesCommand {
    /// Directory to output man pages
    #[arg(short, long, default_value = "man")]
    pub output: PathBuf,
}

impl Cli {
    /// Load configuration from file or default locations
    pub async fn load_config(&self) -> Result<tachikoma_common_config::TachikomaConfig, CliError> {
        use tachikoma_common_config::ConfigLoader;

        match &self.config {
            Some(path) => {
                // If a specific config path is provided, use its parent directory as project root
                let project_dir = path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."));
                let loader = ConfigLoader::new(project_dir);
                loader.load().map_err(CliError::Config)
            }
            None => {
                // Use current directory as project root
                let loader = ConfigLoader::default();
                loader.load().map_err(CliError::Config)
            }
        }
    }

    /// Execute the selected command
    pub async fn execute(self, config: tachikoma_common_config::TachikomaConfig) -> Result<(), CliError> {
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
            Command::Completions(cmd) => {
                cmd.execute(&ctx)?;
                Ok(())
            },
            Command::Manpages(cmd) => {
                cmd.execute(&ctx)?;
                Ok(())
            },
            _ => {
                eprintln!("Command not yet implemented");
                std::process::exit(1);
            }
        }
    }
}

impl CompletionsCommand {
    /// Execute the completions command
    pub fn execute(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        use clap::CommandFactory;
        use clap_complete::generate;
        use std::io;

        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        generate(self.shell, &mut cmd, name, &mut io::stdout());
        Ok(())
    }
}

/// Context passed to all commands
#[derive(Debug)]
pub struct CommandContext {
    pub config: tachikoma_common_config::TachikomaConfig,
    pub format: OutputFormat,
    pub color: ColorChoice,
    pub verbose: u8,
}