# Spec 084: Config Command

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 084
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 078-cli-subcommands
- **Estimated Context**: ~10%

## Objective

Implement the `tachikoma config` command for viewing, editing, and managing Tachikoma configuration from the command line.

## Acceptance Criteria

- [ ] `config get` - retrieve configuration values
- [ ] `config set` - set configuration values
- [ ] `config list` - list all configuration
- [ ] `config edit` - open config in editor
- [ ] `config path` - show configuration file paths
- [ ] `config validate` - validate configuration
- [ ] `config init` - initialize configuration
- [ ] Support for nested keys (dot notation)
- [ ] Environment variable display
- [ ] Configuration source tracking

## Implementation Details

### src/commands/config.rs

```rust
//! Configuration management commands.

use std::path::PathBuf;

use async_trait::async_trait;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::CommandContext;
use crate::commands::Execute;
use crate::error::CliError;
use crate::output::{Output, Table, Column, Alignment};

/// Configuration management commands
#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Get a configuration value
    Get(ConfigGetArgs),

    /// Set a configuration value
    Set(ConfigSetArgs),

    /// List all configuration values
    #[command(visible_alias = "ls")]
    List(ConfigListArgs),

    /// Open configuration in editor
    Edit(ConfigEditArgs),

    /// Show configuration file paths
    Path(ConfigPathArgs),

    /// Validate configuration
    Validate(ConfigValidateArgs),

    /// Initialize configuration
    Init(ConfigInitArgs),

    /// Reset configuration to defaults
    Reset(ConfigResetArgs),
}

#[derive(Debug, Args)]
pub struct ConfigGetArgs {
    /// Configuration key (supports dot notation, e.g., "backend.default")
    pub key: String,

    /// Show value only (no key)
    #[arg(short, long)]
    pub value_only: bool,
}

#[derive(Debug, Args)]
pub struct ConfigSetArgs {
    /// Configuration key
    pub key: String,

    /// Value to set
    pub value: String,

    /// Set in global config (default: local project)
    #[arg(short, long)]
    pub global: bool,

    /// Don't validate the value
    #[arg(long)]
    pub no_validate: bool,
}

#[derive(Debug, Args)]
pub struct ConfigListArgs {
    /// Show only keys matching prefix
    #[arg(short, long)]
    pub prefix: Option<String>,

    /// Show configuration sources
    #[arg(short, long)]
    pub sources: bool,

    /// Show environment variable mappings
    #[arg(short, long)]
    pub env: bool,

    /// Show as flat key=value pairs
    #[arg(long)]
    pub flat: bool,
}

#[derive(Debug, Args)]
pub struct ConfigEditArgs {
    /// Edit global config instead of local
    #[arg(short, long)]
    pub global: bool,

    /// Editor to use (default: $EDITOR or $VISUAL)
    #[arg(short, long)]
    pub editor: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigPathArgs {
    /// Show global config path
    #[arg(short, long)]
    pub global: bool,

    /// Show all config paths in order
    #[arg(short, long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct ConfigValidateArgs {
    /// Config file to validate (default: auto-detected)
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Show detailed validation results
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct ConfigInitArgs {
    /// Create global config instead of local
    #[arg(short, long)]
    pub global: bool,

    /// Overwrite existing config
    #[arg(short, long)]
    pub force: bool,

    /// Use interactive mode
    #[arg(short, long)]
    pub interactive: bool,
}

#[derive(Debug, Args)]
pub struct ConfigResetArgs {
    /// Reset specific key (or all if not specified)
    pub key: Option<String>,

    /// Reset global config
    #[arg(short, long)]
    pub global: bool,

    /// Don't prompt for confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[async_trait]
impl Execute for ConfigCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        match &self.action {
            ConfigAction::Get(args) => config_get(args, ctx).await,
            ConfigAction::Set(args) => config_set(args, ctx).await,
            ConfigAction::List(args) => config_list(args, ctx).await,
            ConfigAction::Edit(args) => config_edit(args, ctx).await,
            ConfigAction::Path(args) => config_path(args, ctx).await,
            ConfigAction::Validate(args) => config_validate(args, ctx).await,
            ConfigAction::Init(args) => config_init(args, ctx).await,
            ConfigAction::Reset(args) => config_reset(args, ctx).await,
        }
    }
}

async fn config_get(args: &ConfigGetArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let value = ctx
        .config
        .get(&args.key)
        .ok_or_else(|| CliError::InvalidArgument(format!("Key not found: {}", args.key)))?;

    if args.value_only {
        println!("{}", format_value(&value));
    } else {
        println!("{} = {}", args.key, format_value(&value));
    }

    Ok(())
}

async fn config_set(args: &ConfigSetArgs, ctx: &CommandContext) -> Result<(), CliError> {
    use tachikoma_config::ConfigWriter;

    let output = Output::new(ctx);

    // Parse the value
    let value = parse_config_value(&args.value)?;

    // Validate unless skipped
    if !args.no_validate {
        ctx.config.validate_key(&args.key, &value)?;
    }

    // Determine config file path
    let config_path = if args.global {
        tachikoma_config::global_config_path()?
    } else {
        tachikoma_config::local_config_path()?
    };

    // Write the value
    let writer = ConfigWriter::new(&config_path)?;
    writer.set(&args.key, value)?;
    writer.save()?;

    output.success(&format!("Set {} in {}", args.key, config_path.display()));

    Ok(())
}

async fn config_list(args: &ConfigListArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    // Flatten config to key-value pairs
    let entries = ctx.config.flatten();

    // Filter by prefix if specified
    let entries: Vec<_> = if let Some(prefix) = &args.prefix {
        entries
            .into_iter()
            .filter(|(k, _, _)| k.starts_with(prefix))
            .collect()
    } else {
        entries
    };

    if args.flat {
        for (key, value, _source) in &entries {
            println!("{}={}", key, format_value(value));
        }
        return Ok(());
    }

    if args.sources {
        let mut table = Table::new(vec![
            Column::new("Key").min_width(30),
            Column::new("Value").min_width(30),
            Column::new("Source").min_width(15),
        ]);

        for (key, value, source) in &entries {
            table.add_row(vec![
                key.clone(),
                format_value(value),
                source.to_string(),
            ]);
        }

        output.print_table(&table)?;
    } else {
        let mut table = Table::new(vec![
            Column::new("Key").min_width(35),
            Column::new("Value").min_width(40),
        ]);

        for (key, value, _) in &entries {
            table.add_row(vec![key.clone(), format_value(value)]);
        }

        output.print_table(&table)?;
    }

    if args.env {
        println!("\nEnvironment Variable Mappings:");
        for (key, env_var) in ctx.config.env_mappings() {
            let current = std::env::var(&env_var).ok();
            let status = if current.is_some() { "(set)" } else { "(not set)" };
            println!("  {} <- {} {}", key, env_var, status);
        }
    }

    Ok(())
}

async fn config_edit(args: &ConfigEditArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let config_path = if args.global {
        tachikoma_config::global_config_path()?
    } else {
        tachikoma_config::local_config_path()?
    };

    // Ensure config file exists
    if !config_path.exists() {
        return Err(CliError::InvalidArgument(format!(
            "Config file does not exist: {}. Run 'tachikoma config init' first.",
            config_path.display()
        )));
    }

    // Get editor
    let editor = args
        .editor
        .clone()
        .or_else(|| std::env::var("VISUAL").ok())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| "vi".to_string());

    // Open editor
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    if !status.success() {
        return Err(CliError::CommandFailed(format!(
            "Editor '{}' exited with error",
            editor
        )));
    }

    // Validate edited config
    println!("Validating configuration...");
    let result = tachikoma_config::Config::load_from_path(&config_path).await;

    match result {
        Ok(_) => {
            let output = Output::new(ctx);
            output.success("Configuration is valid");
        }
        Err(e) => {
            return Err(CliError::Validation(format!(
                "Configuration validation failed: {}",
                e
            )));
        }
    }

    Ok(())
}

async fn config_path(args: &ConfigPathArgs, ctx: &CommandContext) -> Result<(), CliError> {
    if args.all {
        println!("Configuration search paths (in priority order):");
        for (i, path) in ctx.config.search_paths().iter().enumerate() {
            let exists = path.exists();
            let marker = if exists { "*" } else { " " };
            println!("  {}. {} {}", i + 1, marker, path.display());
        }
        println!("\n  * = file exists");
    } else if args.global {
        let path = tachikoma_config::global_config_path()?;
        println!("{}", path.display());
    } else {
        let path = tachikoma_config::local_config_path()?;
        println!("{}", path.display());
    }

    Ok(())
}

async fn config_validate(args: &ConfigValidateArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let path = match &args.file {
        Some(p) => p.clone(),
        None => ctx.config.path().to_path_buf(),
    };

    println!("Validating: {}", path.display());

    // Load and validate
    match tachikoma_config::Config::load_from_path(&path).await {
        Ok(config) => {
            output.success("Configuration is valid");

            if args.verbose {
                println!("\nConfiguration Summary:");
                println!("  Project: {}", config.project.name);
                println!("  Version: {}", config.project.version);
                println!("  Tools: {}", config.tools.count());
                println!("  Backends: {}", config.backends.count());
            }

            Ok(())
        }
        Err(e) => {
            output.error(&format!("Validation failed: {e}"));
            Err(CliError::Validation(e.to_string()))
        }
    }
}

async fn config_init(args: &ConfigInitArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let config_path = if args.global {
        tachikoma_config::global_config_path()?
    } else {
        tachikoma_config::local_config_path()?
    };

    // Check if exists
    if config_path.exists() && !args.force {
        return Err(CliError::InvalidArgument(format!(
            "Config already exists: {}. Use --force to overwrite.",
            config_path.display()
        )));
    }

    let config = if args.interactive {
        create_config_interactive(ctx).await?
    } else {
        tachikoma_config::Config::default()
    };

    // Create parent directories
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write config
    let toml = toml::to_string_pretty(&config)
        .map_err(|e| CliError::Other(anyhow::anyhow!("Failed to serialize config: {}", e)))?;

    std::fs::write(&config_path, toml)?;

    output.success(&format!("Created configuration at {}", config_path.display()));

    Ok(())
}

async fn config_reset(args: &ConfigResetArgs, ctx: &CommandContext) -> Result<(), CliError> {
    use crate::prompts::{confirm, is_interactive};

    let output = Output::new(ctx);

    // Confirm unless --yes
    if !args.yes && is_interactive() {
        let msg = match &args.key {
            Some(key) => format!("Reset '{}' to default value?", key),
            None => "Reset all configuration to defaults?".to_string(),
        };

        if !confirm(&msg)? {
            println!("Aborted.");
            return Ok(());
        }
    }

    let config_path = if args.global {
        tachikoma_config::global_config_path()?
    } else {
        tachikoma_config::local_config_path()?
    };

    match &args.key {
        Some(key) => {
            // Reset single key
            let writer = tachikoma_config::ConfigWriter::new(&config_path)?;
            writer.reset_key(key)?;
            writer.save()?;
            output.success(&format!("Reset '{}' to default", key));
        }
        None => {
            // Reset entire config
            let config = tachikoma_config::Config::default();
            let toml = toml::to_string_pretty(&config)?;
            std::fs::write(&config_path, toml)?;
            output.success("Reset configuration to defaults");
        }
    }

    Ok(())
}

async fn create_config_interactive(
    ctx: &CommandContext,
) -> Result<tachikoma_config::Config, CliError> {
    use crate::prompts::{input, input_with_default, select};

    let project_name = input_with_default("Project name", "my-agent")?;
    let project_version = input_with_default("Version", "0.1.0")?;

    let backend_choices = vec!["anthropic", "openai", "local", "none"];
    let default_backend = select("Default AI backend", backend_choices)?;

    let mut config = tachikoma_config::Config::default();
    config.project.name = project_name;
    config.project.version = project_version.parse().unwrap_or_default();

    if default_backend != "none" {
        config.backends.default = Some(default_backend.to_string());
    }

    Ok(config)
}

fn format_value(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(arr) => {
            let items: Vec<_> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::Table(t) => toml::to_string_pretty(t).unwrap_or_else(|_| "{...}".to_string()),
        toml::Value::Datetime(dt) => dt.to_string(),
    }
}

fn parse_config_value(s: &str) -> Result<toml::Value, CliError> {
    // Try parsing as different types
    if let Ok(b) = s.parse::<bool>() {
        return Ok(toml::Value::Boolean(b));
    }
    if let Ok(i) = s.parse::<i64>() {
        return Ok(toml::Value::Integer(i));
    }
    if let Ok(f) = s.parse::<f64>() {
        return Ok(toml::Value::Float(f));
    }
    // JSON array or object
    if s.starts_with('[') || s.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(s) {
            return toml::Value::try_from(value)
                .map_err(|e| CliError::InvalidArgument(e.to_string()));
        }
    }
    // Default to string
    Ok(toml::Value::String(s.to_string()))
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_value_bool() {
        assert_eq!(parse_config_value("true").unwrap(), toml::Value::Boolean(true));
        assert_eq!(parse_config_value("false").unwrap(), toml::Value::Boolean(false));
    }

    #[test]
    fn test_parse_config_value_number() {
        assert_eq!(parse_config_value("42").unwrap(), toml::Value::Integer(42));
        assert_eq!(parse_config_value("3.14").unwrap(), toml::Value::Float(3.14));
    }

    #[test]
    fn test_parse_config_value_string() {
        assert_eq!(
            parse_config_value("hello").unwrap(),
            toml::Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_config_value_array() {
        let result = parse_config_value("[1, 2, 3]").unwrap();
        assert!(matches!(result, toml::Value::Array(_)));
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&toml::Value::Boolean(true)), "true");
        assert_eq!(format_value(&toml::Value::Integer(42)), "42");
        assert_eq!(format_value(&toml::Value::String("hello".to_string())), "hello");
    }
}
```

### Integration Tests

```rust
// tests/config_cmd.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_config_list() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["config", "list"])
        .assert()
        .success();
}

#[test]
fn test_config_path() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".toml"));
}

#[test]
fn test_config_init_in_temp_dir() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("tachikoma")
        .unwrap()
        .current_dir(&dir)
        .args(["config", "init"])
        .assert()
        .success();

    assert!(dir.path().join("tachikoma.toml").exists());
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **078-cli-subcommands.md**: Subcommand patterns
- **083-cli-prompts.md**: Interactive prompts
- **085-cli-doctor-cmd.md**: System diagnostics
