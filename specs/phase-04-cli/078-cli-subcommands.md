# Spec 078: CLI Subcommand Structure

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 078
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 077-cli-args
- **Estimated Context**: ~10%

## Objective

Define the hierarchical subcommand structure for Tachikoma CLI, organizing commands into logical groups with consistent patterns and shared functionality.

## Acceptance Criteria

- [x] Top-level command groups defined
- [x] Nested subcommand support
- [x] Command trait for consistent execution
- [x] Shared context passed to all commands
- [x] Command aliases supported
- [x] Hidden commands for internal use
- [x] Command groups with common behavior

## Implementation Details

### src/commands/mod.rs

```rust
//! Command module organization and shared traits.

mod backends;
mod config;
mod doctor;
mod init;
mod run;
mod tools;

pub use backends::BackendsCommand;
pub use config::ConfigCommand;
pub use doctor::DoctorCommand;
pub use init::InitCommand;
pub use run::RunCommand;
pub use tools::ToolsCommand;

use async_trait::async_trait;

use crate::cli::CommandContext;
use crate::error::CliError;
use crate::output::Output;

/// Trait for executable commands
#[async_trait]
pub trait Execute {
    /// Execute the command with the given context
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError>;
}

/// Trait for commands that produce output
#[async_trait]
pub trait ExecuteWithOutput {
    type Output: serde::Serialize;

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
    O: serde::Serialize + std::fmt::Display,
{
    let output = command.execute(ctx).await?;
    Output::new(ctx).print(&output)?;
    Ok(())
}
```

### Command Hierarchy

```rust
//! Complete command hierarchy definition.

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tachikoma")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    // ... global flags
}

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
    #[command(subcommand)]
    Config(ConfigCommand),

    // === System ===

    /// Check system health and dependencies
    Doctor(DoctorCommand),

    // === Resources ===

    /// Manage MCP tools
    #[command(subcommand)]
    Tools(ToolsCommand),

    /// Manage AI backends
    #[command(subcommand)]
    Backends(BackendsCommand),

    /// Manage prompts
    #[command(subcommand)]
    Prompts(PromptsCommand),

    /// Manage workflows
    #[command(subcommand)]
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
```

### src/commands/tools.rs

```rust
//! MCP tools management commands.

use async_trait::async_trait;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::CommandContext;
use crate::error::CliError;
use crate::commands::{Execute, ExecuteWithOutput};

/// Manage MCP tools
#[derive(Debug, Args)]
pub struct ToolsCommand {
    #[command(subcommand)]
    pub action: ToolsAction,
}

#[derive(Debug, Subcommand)]
pub enum ToolsAction {
    /// List available tools
    #[command(visible_alias = "ls")]
    List(ToolsListArgs),

    /// Show tool details
    #[command(visible_alias = "info")]
    Show(ToolsShowArgs),

    /// Install a tool from registry
    Install(ToolsInstallArgs),

    /// Uninstall a tool
    #[command(visible_alias = "rm")]
    Uninstall(ToolsUninstallArgs),

    /// Update tools
    #[command(visible_alias = "up")]
    Update(ToolsUpdateArgs),

    /// Search for tools
    Search(ToolsSearchArgs),

    /// Validate tool configuration
    Validate(ToolsValidateArgs),

    /// Test a tool
    Test(ToolsTestArgs),
}

#[derive(Debug, Args)]
pub struct ToolsListArgs {
    /// Show only enabled tools
    #[arg(long)]
    pub enabled: bool,

    /// Show only local tools
    #[arg(long)]
    pub local: bool,

    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ToolsListOutput {
    pub tools: Vec<ToolInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub source: String,
}

impl std::fmt::Display for ToolsListOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Available Tools ({}):", self.total)?;
        writeln!(f)?;
        for tool in &self.tools {
            let status = if tool.enabled { "+" } else { "-" };
            writeln!(f, "  [{status}] {} v{}", tool.name, tool.version)?;
            writeln!(f, "      {}", tool.description)?;
        }
        Ok(())
    }
}

#[async_trait]
impl ExecuteWithOutput for ToolsListArgs {
    type Output = ToolsListOutput;

    async fn execute(&self, ctx: &CommandContext) -> Result<Self::Output, CliError> {
        // Load tools from configuration
        let tools = ctx.config.tools.list().await?;

        let filtered: Vec<_> = tools
            .into_iter()
            .filter(|t| {
                if self.enabled && !t.enabled {
                    return false;
                }
                if self.local && !t.is_local() {
                    return false;
                }
                if let Some(cat) = &self.category {
                    if !t.categories.contains(cat) {
                        return false;
                    }
                }
                true
            })
            .map(|t| ToolInfo {
                name: t.name,
                version: t.version.to_string(),
                description: t.description,
                enabled: t.enabled,
                source: t.source.to_string(),
            })
            .collect();

        let total = filtered.len();
        Ok(ToolsListOutput {
            tools: filtered,
            total,
        })
    }
}

#[derive(Debug, Args)]
pub struct ToolsShowArgs {
    /// Tool name to show
    pub name: String,

    /// Show schema definition
    #[arg(long)]
    pub schema: bool,
}

#[derive(Debug, Args)]
pub struct ToolsInstallArgs {
    /// Tool name or URL to install
    pub source: String,

    /// Version to install
    #[arg(short, long)]
    pub version: Option<String>,

    /// Force reinstall
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct ToolsUninstallArgs {
    /// Tool name to uninstall
    pub name: String,

    /// Don't prompt for confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct ToolsUpdateArgs {
    /// Specific tool to update (updates all if not specified)
    pub name: Option<String>,

    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
}

#[derive(Debug, Args)]
pub struct ToolsSearchArgs {
    /// Search query
    pub query: String,

    /// Maximum results
    #[arg(short, long, default_value = "10")]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct ToolsValidateArgs {
    /// Tool name to validate
    pub name: Option<String>,

    /// Validate all tools
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct ToolsTestArgs {
    /// Tool name to test
    pub name: String,

    /// Input data (JSON)
    #[arg(short, long)]
    pub input: Option<String>,

    /// Input file
    #[arg(short = 'f', long)]
    pub input_file: Option<std::path::PathBuf>,
}

#[async_trait]
impl Execute for ToolsCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        match &self.action {
            ToolsAction::List(args) => {
                crate::commands::run_with_output(args, ctx).await
            }
            ToolsAction::Show(args) => args.execute(ctx).await,
            ToolsAction::Install(args) => args.execute(ctx).await,
            ToolsAction::Uninstall(args) => args.execute(ctx).await,
            ToolsAction::Update(args) => args.execute(ctx).await,
            ToolsAction::Search(args) => args.execute(ctx).await,
            ToolsAction::Validate(args) => args.execute(ctx).await,
            ToolsAction::Test(args) => args.execute(ctx).await,
        }
    }
}
```

### src/commands/backends.rs

```rust
//! AI backend management commands.

use async_trait::async_trait;
use clap::{Args, Subcommand};

use crate::cli::CommandContext;
use crate::error::CliError;
use crate::commands::Execute;

/// Manage AI backends
#[derive(Debug, Args)]
pub struct BackendsCommand {
    #[command(subcommand)]
    pub action: BackendsAction,
}

#[derive(Debug, Subcommand)]
pub enum BackendsAction {
    /// List configured backends
    #[command(visible_alias = "ls")]
    List(BackendsListArgs),

    /// Show backend details
    #[command(visible_alias = "info")]
    Show(BackendsShowArgs),

    /// Add a new backend
    Add(BackendsAddArgs),

    /// Remove a backend
    #[command(visible_alias = "rm")]
    Remove(BackendsRemoveArgs),

    /// Set default backend
    Default(BackendsDefaultArgs),

    /// Test backend connection
    Test(BackendsTestArgs),

    /// List available models for a backend
    Models(BackendsModelsArgs),
}

#[derive(Debug, Args)]
pub struct BackendsListArgs {
    /// Show detailed information
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct BackendsShowArgs {
    /// Backend name
    pub name: String,

    /// Show configuration details
    #[arg(long)]
    pub config: bool,
}

#[derive(Debug, Args)]
pub struct BackendsAddArgs {
    /// Backend name
    pub name: String,

    /// Backend type (anthropic, openai, local, etc.)
    #[arg(short = 't', long)]
    pub backend_type: String,

    /// API key (or use environment variable)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Base URL for the API
    #[arg(long)]
    pub base_url: Option<String>,

    /// Set as default backend
    #[arg(long)]
    pub default: bool,
}

#[derive(Debug, Args)]
pub struct BackendsRemoveArgs {
    /// Backend name to remove
    pub name: String,

    /// Don't prompt for confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct BackendsDefaultArgs {
    /// Backend name to set as default
    pub name: String,
}

#[derive(Debug, Args)]
pub struct BackendsTestArgs {
    /// Backend name to test
    pub name: Option<String>,

    /// Test with a simple prompt
    #[arg(long)]
    pub prompt: Option<String>,
}

#[derive(Debug, Args)]
pub struct BackendsModelsArgs {
    /// Backend name
    pub name: Option<String>,

    /// Filter by capability
    #[arg(short, long)]
    pub capability: Option<String>,
}

#[async_trait]
impl Execute for BackendsCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        match &self.action {
            BackendsAction::List(args) => list_backends(args, ctx).await,
            BackendsAction::Show(args) => show_backend(args, ctx).await,
            BackendsAction::Add(args) => add_backend(args, ctx).await,
            BackendsAction::Remove(args) => remove_backend(args, ctx).await,
            BackendsAction::Default(args) => set_default_backend(args, ctx).await,
            BackendsAction::Test(args) => test_backend(args, ctx).await,
            BackendsAction::Models(args) => list_models(args, ctx).await,
        }
    }
}

async fn list_backends(args: &BackendsListArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let backends = ctx.config.backends.list();

    if backends.is_empty() {
        println!("No backends configured.");
        println!("Run 'tachikoma backends add' to add one.");
        return Ok(());
    }

    println!("Configured Backends:\n");
    for backend in backends {
        let default_marker = if backend.is_default { " (default)" } else { "" };
        println!("  {} [{}]{}", backend.name, backend.backend_type, default_marker);

        if args.verbose {
            if let Some(url) = &backend.base_url {
                println!("    URL: {url}");
            }
            println!("    Models: {}", backend.models.len());
        }
    }

    Ok(())
}

async fn show_backend(_args: &BackendsShowArgs, _ctx: &CommandContext) -> Result<(), CliError> {
    todo!("Implement show_backend")
}

async fn add_backend(_args: &BackendsAddArgs, _ctx: &CommandContext) -> Result<(), CliError> {
    todo!("Implement add_backend")
}

async fn remove_backend(_args: &BackendsRemoveArgs, _ctx: &CommandContext) -> Result<(), CliError> {
    todo!("Implement remove_backend")
}

async fn set_default_backend(_args: &BackendsDefaultArgs, _ctx: &CommandContext) -> Result<(), CliError> {
    todo!("Implement set_default_backend")
}

async fn test_backend(_args: &BackendsTestArgs, _ctx: &CommandContext) -> Result<(), CliError> {
    todo!("Implement test_backend")
}

async fn list_models(_args: &BackendsModelsArgs, _ctx: &CommandContext) -> Result<(), CliError> {
    todo!("Implement list_models")
}
```

### Command Registration Pattern

```rust
//! Pattern for registering commands dynamically.

use std::collections::HashMap;
use async_trait::async_trait;

use crate::cli::CommandContext;
use crate::error::CliError;

/// Command factory trait
#[async_trait]
pub trait CommandFactory: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn aliases(&self) -> &[&str] { &[] }
    fn hidden(&self) -> bool { false }

    async fn execute(
        &self,
        args: &[String],
        ctx: &CommandContext,
    ) -> Result<(), CliError>;
}

/// Command registry for plugin commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandFactory>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, factory: Box<dyn CommandFactory>) {
        let name = factory.name().to_string();
        for alias in factory.aliases() {
            self.commands.insert(alias.to_string(), factory.clone_box());
        }
        self.commands.insert(name, factory);
    }

    pub fn get(&self, name: &str) -> Option<&dyn CommandFactory> {
        self.commands.get(name).map(|b| b.as_ref())
    }

    pub fn list(&self) -> impl Iterator<Item = &dyn CommandFactory> {
        self.commands.values().map(|b| b.as_ref())
    }
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_tools_command() {
        // Verify command structure
        let cmd = ToolsCommand::command();
        cmd.debug_assert();
    }

    #[test]
    fn tools_list_parsing() {
        let args = ToolsCommand::try_parse_from([
            "tools", "list", "--enabled", "--category", "filesystem"
        ]).unwrap();

        match args.action {
            ToolsAction::List(list_args) => {
                assert!(list_args.enabled);
                assert_eq!(list_args.category, Some("filesystem".to_string()));
            }
            _ => panic!("Expected List action"),
        }
    }

    #[test]
    fn tools_install_parsing() {
        let args = ToolsCommand::try_parse_from([
            "tools", "install", "my-tool", "--version", "1.0.0", "--force"
        ]).unwrap();

        match args.action {
            ToolsAction::Install(install_args) => {
                assert_eq!(install_args.source, "my-tool");
                assert_eq!(install_args.version, Some("1.0.0".to_string()));
                assert!(install_args.force);
            }
            _ => panic!("Expected Install action"),
        }
    }

    #[test]
    fn backends_add_parsing() {
        let args = BackendsCommand::try_parse_from([
            "backends", "add", "my-backend",
            "--backend-type", "openai",
            "--base-url", "https://api.example.com",
            "--default"
        ]).unwrap();

        match args.action {
            BackendsAction::Add(add_args) => {
                assert_eq!(add_args.name, "my-backend");
                assert_eq!(add_args.backend_type, "openai");
                assert_eq!(add_args.base_url, Some("https://api.example.com".to_string()));
                assert!(add_args.default);
            }
            _ => panic!("Expected Add action"),
        }
    }
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **077-cli-args.md**: Argument patterns
- **084-cli-config-cmd.md**: Config command implementation
- **085-cli-doctor-cmd.md**: Doctor command implementation
- **086-cli-tools-cmd.md**: Tools command implementation
- **087-cli-backends-cmd.md**: Backends command implementation
