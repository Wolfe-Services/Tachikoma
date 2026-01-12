# Spec 086: Tools Command

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 086
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 078-cli-subcommands
- **Estimated Context**: ~10%

## Objective

Implement the `tachikoma tools` command for managing MCP tools, including listing, installing, configuring, and testing tools from registries and local sources.

## Acceptance Criteria

- [x] `tools list` - list installed and available tools
- [x] `tools show` - show tool details and schema
- [x] `tools install` - install from registry or path
- [x] `tools uninstall` - remove installed tools
- [x] `tools update` - update to latest versions
- [x] `tools search` - search tool registries
- [x] `tools validate` - validate tool configurations
- [x] `tools test` - test tool execution
- [x] `tools enable/disable` - toggle tool availability

## Implementation Details

### src/commands/tools.rs

```rust
//! MCP tools management commands.

use std::path::PathBuf;

use async_trait::async_trait;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::CommandContext;
use crate::commands::Execute;
use crate::error::CliError;
use crate::output::{Output, Table, Column, Alignment};
use crate::output::color::{Styled, Color};
use crate::prompts::{confirm, is_interactive};

/// Tools management commands
#[derive(Debug, Args)]
pub struct ToolsCommand {
    #[command(subcommand)]
    pub action: ToolsAction,
}

#[derive(Debug, Subcommand)]
pub enum ToolsAction {
    /// List tools
    #[command(visible_alias = "ls")]
    List(ToolsListArgs),

    /// Show tool details
    #[command(visible_alias = "info")]
    Show(ToolsShowArgs),

    /// Install a tool
    #[command(visible_alias = "add")]
    Install(ToolsInstallArgs),

    /// Uninstall a tool
    #[command(visible_alias = "rm", visible_alias = "remove")]
    Uninstall(ToolsUninstallArgs),

    /// Update tools
    #[command(visible_alias = "upgrade")]
    Update(ToolsUpdateArgs),

    /// Search for tools
    Search(ToolsSearchArgs),

    /// Validate tool configuration
    Validate(ToolsValidateArgs),

    /// Test a tool
    Test(ToolsTestArgs),

    /// Enable a tool
    Enable(ToolsToggleArgs),

    /// Disable a tool
    Disable(ToolsToggleArgs),

    /// Create a new tool
    Create(ToolsCreateArgs),
}

#[derive(Debug, Args)]
pub struct ToolsListArgs {
    /// Show only enabled tools
    #[arg(long)]
    pub enabled: bool,

    /// Show only disabled tools
    #[arg(long)]
    pub disabled: bool,

    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,

    /// Show detailed output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Serialize)]
pub struct ToolListItem {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub source: String,
    pub categories: Vec<String>,
}

impl std::fmt::Display for ToolListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.enabled {
            Styled::new("+").fg(Color::Green)
        } else {
            Styled::new("-").fg(Color::Red)
        };
        write!(f, "[{status}] {} v{} - {}", self.name, self.version, self.description)
    }
}

#[derive(Debug, Args)]
pub struct ToolsShowArgs {
    /// Tool name
    pub name: String,

    /// Show input schema
    #[arg(long)]
    pub schema: bool,

    /// Show usage examples
    #[arg(long)]
    pub examples: bool,
}

#[derive(Debug, Args)]
pub struct ToolsInstallArgs {
    /// Tool source (name from registry, URL, or local path)
    pub source: String,

    /// Specific version to install
    #[arg(short, long)]
    pub version: Option<String>,

    /// Force reinstall if already installed
    #[arg(short, long)]
    pub force: bool,

    /// Install from local path
    #[arg(long)]
    pub path: bool,

    /// Enable tool after installation
    #[arg(long, default_value = "true")]
    pub enable: bool,
}

#[derive(Debug, Args)]
pub struct ToolsUninstallArgs {
    /// Tool name(s) to uninstall
    #[arg(required = true)]
    pub names: Vec<String>,

    /// Skip confirmation prompt
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

    /// Include pre-release versions
    #[arg(long)]
    pub prerelease: bool,
}

#[derive(Debug, Args)]
pub struct ToolsSearchArgs {
    /// Search query
    pub query: String,

    /// Maximum number of results
    #[arg(short, long, default_value = "20")]
    pub limit: u32,

    /// Search in specific registry
    #[arg(long)]
    pub registry: Option<String>,

    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,
}

#[derive(Debug, Args)]
pub struct ToolsValidateArgs {
    /// Tool name (validates all if not specified)
    pub name: Option<String>,

    /// Show detailed validation output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct ToolsTestArgs {
    /// Tool name to test
    pub name: String,

    /// Input JSON string
    #[arg(short, long)]
    pub input: Option<String>,

    /// Input from file
    #[arg(short = 'f', long)]
    pub file: Option<PathBuf>,

    /// Timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,
}

#[derive(Debug, Args)]
pub struct ToolsToggleArgs {
    /// Tool name(s)
    #[arg(required = true)]
    pub names: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ToolsCreateArgs {
    /// Tool name
    pub name: String,

    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Tool template to use
    #[arg(short, long, default_value = "basic")]
    pub template: String,
}

#[async_trait]
impl Execute for ToolsCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        match &self.action {
            ToolsAction::List(args) => tools_list(args, ctx).await,
            ToolsAction::Show(args) => tools_show(args, ctx).await,
            ToolsAction::Install(args) => tools_install(args, ctx).await,
            ToolsAction::Uninstall(args) => tools_uninstall(args, ctx).await,
            ToolsAction::Update(args) => tools_update(args, ctx).await,
            ToolsAction::Search(args) => tools_search(args, ctx).await,
            ToolsAction::Validate(args) => tools_validate(args, ctx).await,
            ToolsAction::Test(args) => tools_test(args, ctx).await,
            ToolsAction::Enable(args) => tools_toggle(args, ctx, true).await,
            ToolsAction::Disable(args) => tools_toggle(args, ctx, false).await,
            ToolsAction::Create(args) => tools_create(args, ctx).await,
        }
    }
}

async fn tools_list(args: &ToolsListArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let tools = ctx.config.tools.list().await?;

    // Filter tools
    let tools: Vec<_> = tools
        .into_iter()
        .filter(|t| {
            if args.enabled && !t.enabled {
                return false;
            }
            if args.disabled && t.enabled {
                return false;
            }
            if let Some(cat) = &args.category {
                if !t.categories.contains(cat) {
                    return false;
                }
            }
            true
        })
        .collect();

    if tools.is_empty() {
        println!("No tools found matching criteria.");
        println!("\nRun 'tachikoma tools search <query>' to find tools.");
        return Ok(());
    }

    if args.verbose {
        let mut table = Table::new(vec![
            Column::new("Status").min_width(6),
            Column::new("Name").min_width(20),
            Column::new("Version").min_width(10),
            Column::new("Description").min_width(40),
            Column::new("Source").min_width(15),
        ]);

        for tool in &tools {
            let status = if tool.enabled { "+" } else { "-" };
            table.add_row(vec![
                status.to_string(),
                tool.name.clone(),
                tool.version.to_string(),
                tool.description.clone(),
                tool.source.to_string(),
            ]);
        }

        output.print_table(&table)?;
    } else {
        println!("Installed Tools ({}):\n", tools.len());

        for tool in &tools {
            let status = if tool.enabled {
                Styled::new("+").fg(Color::Green)
            } else {
                Styled::new("-").fg(Color::BrightBlack)
            };

            let name = if tool.enabled {
                Styled::new(&tool.name).fg(Color::White)
            } else {
                Styled::new(&tool.name).fg(Color::BrightBlack)
            };

            println!("  [{status}] {name} v{}", tool.version);

            if !tool.description.is_empty() {
                let desc = Styled::new(&tool.description).fg(Color::BrightBlack);
                println!("      {desc}");
            }
        }
    }

    Ok(())
}

async fn tools_show(args: &ToolsShowArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let tool = ctx
        .config
        .tools
        .get(&args.name)
        .await?
        .ok_or_else(|| CliError::InvalidArgument(format!("Tool not found: {}", args.name)))?;

    println!("{}", Styled::new(&tool.name).bold());
    println!("Version: {}", tool.version);
    println!("Description: {}", tool.description);
    println!("Enabled: {}", tool.enabled);
    println!("Source: {}", tool.source);

    if !tool.categories.is_empty() {
        println!("Categories: {}", tool.categories.join(", "));
    }

    if args.schema {
        println!("\nInput Schema:");
        let schema = serde_json::to_string_pretty(&tool.input_schema)
            .unwrap_or_else(|_| "{}".to_string());
        println!("{schema}");
    }

    if args.examples && !tool.examples.is_empty() {
        println!("\nExamples:");
        for (i, example) in tool.examples.iter().enumerate() {
            println!("  {}. {}", i + 1, example.description);
            let input = serde_json::to_string_pretty(&example.input)
                .unwrap_or_else(|_| "{}".to_string());
            println!("     Input: {input}");
        }
    }

    Ok(())
}

async fn tools_install(args: &ToolsInstallArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    // Check if already installed
    if !args.force {
        if let Some(existing) = ctx.config.tools.get(&args.source).await? {
            return Err(CliError::InvalidArgument(format!(
                "Tool '{}' is already installed (v{}). Use --force to reinstall.",
                args.source, existing.version
            )));
        }
    }

    println!("Installing tool: {}", args.source);

    // Determine source type
    let source = if args.path {
        ToolSource::Local(PathBuf::from(&args.source))
    } else if args.source.starts_with("http://") || args.source.starts_with("https://") {
        ToolSource::Url(args.source.clone())
    } else {
        ToolSource::Registry {
            name: args.source.clone(),
            version: args.version.clone(),
        }
    };

    // Create spinner for installation
    let spinner = crate::output::progress::Spinner::new("Installing...").start();

    // Install the tool
    let result = ctx.config.tools.install(source).await;

    match result {
        Ok(tool) => {
            spinner.finish(&format!("Installed {} v{}", tool.name, tool.version));

            if args.enable {
                ctx.config.tools.set_enabled(&tool.name, true).await?;
                output.success(&format!("Tool '{}' is enabled", tool.name));
            }

            Ok(())
        }
        Err(e) => {
            spinner.fail(&format!("Installation failed: {e}"));
            Err(CliError::CommandFailed(e.to_string()))
        }
    }
}

enum ToolSource {
    Registry { name: String, version: Option<String> },
    Url(String),
    Local(PathBuf),
}

async fn tools_uninstall(args: &ToolsUninstallArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    // Confirm unless --yes
    if !args.yes && is_interactive() {
        let msg = if args.names.len() == 1 {
            format!("Uninstall tool '{}'?", args.names[0])
        } else {
            format!("Uninstall {} tools?", args.names.len())
        };

        if !confirm(&msg)? {
            println!("Aborted.");
            return Ok(());
        }
    }

    for name in &args.names {
        match ctx.config.tools.uninstall(name).await {
            Ok(()) => output.success(&format!("Uninstalled '{name}'")),
            Err(e) => output.error(&format!("Failed to uninstall '{name}': {e}")),
        }
    }

    Ok(())
}

async fn tools_update(args: &ToolsUpdateArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let tools_to_check: Vec<_> = match &args.name {
        Some(name) => {
            let tool = ctx
                .config
                .tools
                .get(name)
                .await?
                .ok_or_else(|| CliError::InvalidArgument(format!("Tool not found: {name}")))?;
            vec![tool]
        }
        None => ctx.config.tools.list().await?,
    };

    if tools_to_check.is_empty() {
        println!("No tools installed.");
        return Ok(());
    }

    println!("Checking for updates...\n");

    let mut updates_available = Vec::new();

    for tool in &tools_to_check {
        let latest = ctx
            .config
            .tools
            .check_update(&tool.name, args.prerelease)
            .await?;

        if let Some(new_version) = latest {
            if new_version > tool.version {
                updates_available.push((tool.clone(), new_version.clone()));
                println!(
                    "  {} {} -> {}",
                    tool.name,
                    Styled::new(&tool.version.to_string()).fg(Color::Yellow),
                    Styled::new(&new_version.to_string()).fg(Color::Green)
                );
            } else {
                println!(
                    "  {} {} (up to date)",
                    tool.name,
                    Styled::new(&tool.version.to_string()).fg(Color::Green)
                );
            }
        }
    }

    if updates_available.is_empty() {
        println!("\nAll tools are up to date.");
        return Ok(());
    }

    if args.check {
        println!("\n{} update(s) available.", updates_available.len());
        return Ok(());
    }

    // Perform updates
    println!("\nUpdating...\n");

    for (tool, new_version) in updates_available {
        let spinner = crate::output::progress::Spinner::new(
            format!("Updating {} to v{}", tool.name, new_version)
        ).start();

        match ctx.config.tools.update(&tool.name, Some(new_version.clone())).await {
            Ok(()) => spinner.finish(&format!("Updated {} to v{}", tool.name, new_version)),
            Err(e) => spinner.fail(&format!("Failed: {e}")),
        }
    }

    Ok(())
}

async fn tools_search(args: &ToolsSearchArgs, ctx: &CommandContext) -> Result<(), CliError> {
    println!("Searching for '{}'...\n", args.query);

    let results = ctx
        .config
        .tools
        .search(&args.query, args.limit, args.registry.as_deref())
        .await?;

    if results.is_empty() {
        println!("No tools found matching '{}'.", args.query);
        return Ok(());
    }

    println!("Found {} tool(s):\n", results.len());

    for tool in results {
        let installed = ctx.config.tools.get(&tool.name).await?.is_some();
        let status = if installed {
            Styled::new("[installed]").fg(Color::Green)
        } else {
            Styled::new("").fg(Color::White)
        };

        println!("  {} v{} {}", tool.name, tool.version, status);
        println!("    {}", tool.description);
        if !tool.categories.is_empty() {
            println!(
                "    Categories: {}",
                Styled::new(tool.categories.join(", ")).fg(Color::BrightBlack)
            );
        }
        println!();
    }

    Ok(())
}

async fn tools_validate(args: &ToolsValidateArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let tools: Vec<_> = match &args.name {
        Some(name) => {
            let tool = ctx
                .config
                .tools
                .get(name)
                .await?
                .ok_or_else(|| CliError::InvalidArgument(format!("Tool not found: {name}")))?;
            vec![tool]
        }
        None => ctx.config.tools.list().await?,
    };

    let mut all_valid = true;

    for tool in tools {
        print!("Validating '{}'... ", tool.name);

        match ctx.config.tools.validate(&tool.name).await {
            Ok(validation) => {
                if validation.is_valid {
                    println!("{}", Styled::new("OK").fg(Color::Green));
                } else {
                    all_valid = false;
                    println!("{}", Styled::new("INVALID").fg(Color::Red));

                    if args.verbose {
                        for error in &validation.errors {
                            println!("    - {error}");
                        }
                    }
                }

                if args.verbose && !validation.warnings.is_empty() {
                    for warning in &validation.warnings {
                        println!(
                            "    {} {}",
                            Styled::new("warning:").fg(Color::Yellow),
                            warning
                        );
                    }
                }
            }
            Err(e) => {
                all_valid = false;
                println!("{} {e}", Styled::new("ERROR").fg(Color::Red));
            }
        }
    }

    if all_valid {
        output.success("All tools are valid");
    } else {
        return Err(CliError::Validation("Some tools have validation errors".to_string()));
    }

    Ok(())
}

async fn tools_test(args: &ToolsTestArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    // Parse input
    let input: serde_json::Value = match (&args.input, &args.file) {
        (Some(json_str), _) => {
            serde_json::from_str(json_str)
                .map_err(|e| CliError::InvalidArgument(format!("Invalid JSON: {e}")))?
        }
        (_, Some(path)) => {
            let content = std::fs::read_to_string(path)?;
            serde_json::from_str(&content)
                .map_err(|e| CliError::InvalidArgument(format!("Invalid JSON in file: {e}")))?
        }
        (None, None) => serde_json::json!({}),
    };

    println!("Testing tool '{}' with input:", args.name);
    println!("{}", serde_json::to_string_pretty(&input)?);
    println!();

    let spinner = crate::output::progress::Spinner::new("Executing...").start();

    let timeout = std::time::Duration::from_secs(args.timeout);

    match ctx.config.tools.execute(&args.name, input, timeout).await {
        Ok(result) => {
            spinner.finish("Execution complete");

            println!("\nOutput:");
            println!("{}", serde_json::to_string_pretty(&result)?);

            Ok(())
        }
        Err(e) => {
            spinner.fail(&format!("Execution failed: {e}"));
            Err(CliError::CommandFailed(e.to_string()))
        }
    }
}

async fn tools_toggle(
    args: &ToolsToggleArgs,
    ctx: &CommandContext,
    enable: bool,
) -> Result<(), CliError> {
    let output = Output::new(ctx);
    let action = if enable { "Enabled" } else { "Disabled" };

    for name in &args.names {
        match ctx.config.tools.set_enabled(name, enable).await {
            Ok(()) => output.success(&format!("{action} '{name}'")),
            Err(e) => output.error(&format!("Failed to toggle '{name}': {e}")),
        }
    }

    Ok(())
}

async fn tools_create(args: &ToolsCreateArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let tool_dir = args.output.join(&args.name);

    if tool_dir.exists() {
        return Err(CliError::InvalidArgument(format!(
            "Directory already exists: {}",
            tool_dir.display()
        )));
    }

    std::fs::create_dir_all(&tool_dir)?;

    // Create tool files based on template
    let manifest = format!(
        r#"[tool]
name = "{name}"
version = "0.1.0"
description = "A new MCP tool"

[tool.input_schema]
type = "object"
properties = {{ }}

[[tool.examples]]
description = "Example usage"
input = {{ }}
"#,
        name = args.name
    );

    std::fs::write(tool_dir.join("tool.toml"), manifest)?;

    // Create main source file
    let main_rs = r#"//! Tool implementation

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Input {
    // Define input fields here
}

#[derive(Debug, Serialize)]
pub struct Output {
    // Define output fields here
}

pub async fn execute(input: Input) -> Result<Output, Box<dyn std::error::Error>> {
    // Implement tool logic here
    todo!()
}
"#;

    std::fs::write(tool_dir.join("src/lib.rs"), main_rs)?;

    output.success(&format!("Created tool at {}", tool_dir.display()));
    println!("\nNext steps:");
    println!("  1. Edit {}/tool.toml to define your tool", tool_dir.display());
    println!("  2. Implement your tool in {}/src/lib.rs", tool_dir.display());
    println!("  3. Install with: tachikoma tools install --path {}", tool_dir.display());

    Ok(())
}
```

## Testing Requirements

### Integration Tests

```rust
// tests/tools_cmd.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_tools_list() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["tools", "list"])
        .assert()
        .success();
}

#[test]
fn test_tools_search() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["tools", "search", "filesystem"])
        .assert()
        .success();
}

#[test]
fn test_tools_show_nonexistent() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["tools", "show", "nonexistent-tool"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **078-cli-subcommands.md**: Subcommand patterns
- **082-cli-progress.md**: Progress indicators for install
- **083-cli-prompts.md**: Confirmation prompts
