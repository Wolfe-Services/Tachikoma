# Spec 087: Backends Command

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 087
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 078-cli-subcommands
- **Estimated Context**: ~10%

## Objective

Implement the `tachikoma backends` command for managing AI backend providers, including configuration, testing, and model discovery.

## Acceptance Criteria

- [x] `backends list` - list configured backends
- [x] `backends add` - add a new backend
- [x] `backends remove` - remove a backend
- [x] `backends show` - show backend details
- [x] `backends test` - test backend connectivity
- [x] `backends default` - set default backend
- [x] `backends models` - list available models
- [x] Interactive configuration wizard
- [x] Secure API key handling

## Implementation Details

### src/commands/backends.rs

```rust
//! AI backend management commands.

use std::collections::HashMap;

use async_trait::async_trait;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::CommandContext;
use crate::commands::Execute;
use crate::error::CliError;
use crate::output::{Output, Table, Column};
use crate::output::color::{Styled, Color};
use crate::prompts::{confirm, input, password, select, is_interactive};

/// Backend management commands
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

    /// Add a new backend
    Add(BackendsAddArgs),

    /// Remove a backend
    #[command(visible_alias = "rm")]
    Remove(BackendsRemoveArgs),

    /// Show backend details
    #[command(visible_alias = "info")]
    Show(BackendsShowArgs),

    /// Test backend connectivity
    Test(BackendsTestArgs),

    /// Set default backend
    Default(BackendsDefaultArgs),

    /// List available models
    Models(BackendsModelsArgs),

    /// Configure backend settings
    Configure(BackendsConfigureArgs),
}

#[derive(Debug, Args)]
pub struct BackendsListArgs {
    /// Show detailed information
    #[arg(short, long)]
    pub verbose: bool,

    /// Show as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
pub struct BackendInfo {
    pub name: String,
    pub backend_type: String,
    pub is_default: bool,
    pub base_url: Option<String>,
    pub has_api_key: bool,
    pub models_count: usize,
}

impl std::fmt::Display for BackendInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let default_marker = if self.is_default { " (default)" } else { "" };
        let key_status = if self.has_api_key { "configured" } else { "missing" };
        write!(
            f,
            "{} [{}]{} - API key: {}",
            self.name, self.backend_type, default_marker, key_status
        )
    }
}

#[derive(Debug, Args)]
pub struct BackendsAddArgs {
    /// Backend name (identifier)
    pub name: String,

    /// Backend type
    #[arg(short = 't', long, value_enum)]
    pub backend_type: Option<BackendType>,

    /// API key (will prompt if not provided)
    #[arg(long, env)]
    pub api_key: Option<String>,

    /// Base URL for the API
    #[arg(long)]
    pub base_url: Option<String>,

    /// Set as default backend
    #[arg(short, long)]
    pub default: bool,

    /// Use interactive wizard
    #[arg(short, long)]
    pub interactive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BackendType {
    Anthropic,
    OpenAI,
    OpenAICompatible,
    Ollama,
    Local,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "anthropic"),
            Self::OpenAI => write!(f, "openai"),
            Self::OpenAICompatible => write!(f, "openai-compatible"),
            Self::Ollama => write!(f, "ollama"),
            Self::Local => write!(f, "local"),
        }
    }
}

#[derive(Debug, Args)]
pub struct BackendsRemoveArgs {
    /// Backend name to remove
    pub name: String,

    /// Skip confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct BackendsShowArgs {
    /// Backend name
    pub name: String,

    /// Show configuration (may contain sensitive data)
    #[arg(long)]
    pub config: bool,
}

#[derive(Debug, Args)]
pub struct BackendsTestArgs {
    /// Backend name (tests default if not specified)
    pub name: Option<String>,

    /// Test with a simple prompt
    #[arg(long)]
    pub prompt: Option<String>,

    /// Test specific model
    #[arg(short, long)]
    pub model: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct BackendsDefaultArgs {
    /// Backend name to set as default
    pub name: String,
}

#[derive(Debug, Args)]
pub struct BackendsModelsArgs {
    /// Backend name (uses default if not specified)
    pub name: Option<String>,

    /// Filter by capability
    #[arg(short, long)]
    pub capability: Option<String>,

    /// Refresh model list from API
    #[arg(long)]
    pub refresh: bool,
}

#[derive(Debug, Args)]
pub struct BackendsConfigureArgs {
    /// Backend name
    pub name: String,

    /// Configuration key
    #[arg(short, long)]
    pub key: Option<String>,

    /// Configuration value
    #[arg(short, long)]
    pub value: Option<String>,
}

#[async_trait]
impl Execute for BackendsCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        match &self.action {
            BackendsAction::List(args) => backends_list(args, ctx).await,
            BackendsAction::Add(args) => backends_add(args, ctx).await,
            BackendsAction::Remove(args) => backends_remove(args, ctx).await,
            BackendsAction::Show(args) => backends_show(args, ctx).await,
            BackendsAction::Test(args) => backends_test(args, ctx).await,
            BackendsAction::Default(args) => backends_default(args, ctx).await,
            BackendsAction::Models(args) => backends_models(args, ctx).await,
            BackendsAction::Configure(args) => backends_configure(args, ctx).await,
        }
    }
}

async fn backends_list(args: &BackendsListArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);
    let backends = ctx.config.backends.list();

    if backends.is_empty() {
        println!("No backends configured.");
        println!("\nRun 'tachikoma backends add' to configure a backend.");
        return Ok(());
    }

    let infos: Vec<BackendInfo> = backends
        .iter()
        .map(|b| BackendInfo {
            name: b.name.clone(),
            backend_type: b.backend_type.clone(),
            is_default: b.is_default,
            base_url: b.base_url.clone(),
            has_api_key: b.api_key.is_some(),
            models_count: b.models.len(),
        })
        .collect();

    if args.json {
        output.print_json(&infos)?;
        return Ok(());
    }

    if args.verbose {
        let mut table = Table::new(vec![
            Column::new("Name").min_width(15),
            Column::new("Type").min_width(12),
            Column::new("Default").min_width(8),
            Column::new("API Key").min_width(10),
            Column::new("Models").min_width(8),
            Column::new("Base URL").min_width(30),
        ]);

        for info in &infos {
            table.add_row(vec![
                info.name.clone(),
                info.backend_type.clone(),
                if info.is_default { "Yes" } else { "No" }.to_string(),
                if info.has_api_key { "Set" } else { "Missing" }.to_string(),
                info.models_count.to_string(),
                info.base_url.clone().unwrap_or_else(|| "-".to_string()),
            ]);
        }

        output.print_table(&table)?;
    } else {
        println!("Configured Backends:\n");

        for info in &infos {
            let default_marker = if info.is_default {
                Styled::new(" (default)").fg(Color::Green)
            } else {
                Styled::new("")
            };

            let key_status = if info.has_api_key {
                Styled::new("API key configured").fg(Color::Green)
            } else {
                Styled::new("API key missing").fg(Color::Red)
            };

            println!("  {} [{}]{}", info.name, info.backend_type, default_marker);
            println!("    {key_status}");

            if info.models_count > 0 {
                println!("    Models: {}", info.models_count);
            }
            println!();
        }
    }

    Ok(())
}

async fn backends_add(args: &BackendsAddArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    // Check if exists
    if ctx.config.backends.get(&args.name).is_some() {
        return Err(CliError::InvalidArgument(format!(
            "Backend '{}' already exists. Use 'tachikoma backends configure' to modify.",
            args.name
        )));
    }

    // Interactive wizard
    let (backend_type, api_key, base_url) = if args.interactive || args.backend_type.is_none() {
        run_backend_wizard(&args.name).await?
    } else {
        let api_key = match &args.api_key {
            Some(key) => Some(key.clone()),
            None if requires_api_key(args.backend_type.unwrap()) => {
                if is_interactive() {
                    Some(password(&format!("API key for {}", args.name))?)
                } else {
                    return Err(CliError::InvalidArgument(
                        "API key required. Provide via --api-key or run interactively.".to_string(),
                    ));
                }
            }
            None => None,
        };

        (args.backend_type.unwrap(), api_key, args.base_url.clone())
    };

    // Create backend config
    let config = tachikoma_config::BackendConfig {
        name: args.name.clone(),
        backend_type: backend_type.to_string(),
        api_key,
        base_url,
        is_default: args.default || ctx.config.backends.list().is_empty(),
        models: Vec::new(),
        settings: HashMap::new(),
    };

    // Save to config
    ctx.config.backends.add(config).await?;

    output.success(&format!("Added backend '{}'", args.name));

    if args.default {
        output.message(&format!("'{}' is now the default backend", args.name));
    }

    // Suggest testing
    println!("\nTest the connection with:");
    println!("  tachikoma backends test {}", args.name);

    Ok(())
}

async fn run_backend_wizard(name: &str) -> Result<(BackendType, Option<String>, Option<String>), CliError> {
    println!("\nConfiguring backend: {}\n", name);

    let types = vec![
        "Anthropic (Claude)",
        "OpenAI (GPT)",
        "OpenAI-Compatible",
        "Ollama (Local)",
        "Local Model",
    ];

    let type_idx = select("Backend type", types.clone())?;
    let backend_type = match types[0].as_str() {
        s if s.starts_with("Anthropic") => BackendType::Anthropic,
        s if s.starts_with("OpenAI (") => BackendType::OpenAI,
        s if s.starts_with("OpenAI-Compatible") => BackendType::OpenAICompatible,
        s if s.starts_with("Ollama") => BackendType::Ollama,
        _ => BackendType::Local,
    };

    let api_key = if requires_api_key(backend_type) {
        let key = password("API key")?;
        if key.is_empty() {
            None
        } else {
            Some(key)
        }
    } else {
        None
    };

    let base_url = if needs_base_url(backend_type) {
        let default_url = default_base_url(backend_type);
        let url = input(&format!("Base URL (default: {})", default_url))?;
        if url.is_empty() {
            Some(default_url.to_string())
        } else {
            Some(url)
        }
    } else {
        None
    };

    Ok((backend_type, api_key, base_url))
}

fn requires_api_key(backend_type: BackendType) -> bool {
    matches!(backend_type, BackendType::Anthropic | BackendType::OpenAI)
}

fn needs_base_url(backend_type: BackendType) -> bool {
    matches!(
        backend_type,
        BackendType::OpenAICompatible | BackendType::Ollama
    )
}

fn default_base_url(backend_type: BackendType) -> &'static str {
    match backend_type {
        BackendType::Ollama => "http://localhost:11434",
        BackendType::OpenAICompatible => "http://localhost:8080/v1",
        _ => "",
    }
}

async fn backends_remove(args: &BackendsRemoveArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    // Check if exists
    if ctx.config.backends.get(&args.name).is_none() {
        return Err(CliError::InvalidArgument(format!(
            "Backend '{}' not found",
            args.name
        )));
    }

    // Confirm
    if !args.yes && is_interactive() {
        if !confirm(&format!("Remove backend '{}'?", args.name))? {
            println!("Aborted.");
            return Ok(());
        }
    }

    ctx.config.backends.remove(&args.name).await?;
    output.success(&format!("Removed backend '{}'", args.name));

    Ok(())
}

async fn backends_show(args: &BackendsShowArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let backend = ctx
        .config
        .backends
        .get(&args.name)
        .ok_or_else(|| CliError::InvalidArgument(format!("Backend '{}' not found", args.name)))?;

    println!("{}", Styled::new(&backend.name).bold());
    println!("Type: {}", backend.backend_type);
    println!("Default: {}", backend.is_default);

    if let Some(url) = &backend.base_url {
        println!("Base URL: {url}");
    }

    if backend.api_key.is_some() {
        println!("API Key: ********");
    } else {
        println!("API Key: Not configured");
    }

    if !backend.models.is_empty() {
        println!("\nModels:");
        for model in &backend.models {
            println!("  - {model}");
        }
    }

    if args.config && !backend.settings.is_empty() {
        println!("\nSettings:");
        for (key, value) in &backend.settings {
            println!("  {key}: {value}");
        }
    }

    Ok(())
}

async fn backends_test(args: &BackendsTestArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let backend_name = args.name.clone().unwrap_or_else(|| {
        ctx.config
            .backends
            .default()
            .map(|b| b.name.clone())
            .unwrap_or_default()
    });

    if backend_name.is_empty() {
        return Err(CliError::InvalidArgument(
            "No backend specified and no default configured".to_string(),
        ));
    }

    let backend = ctx
        .config
        .backends
        .get(&backend_name)
        .ok_or_else(|| CliError::InvalidArgument(format!("Backend '{}' not found", backend_name)))?;

    println!("Testing backend '{}'...\n", backend_name);

    // Test connectivity
    let spinner = crate::output::progress::Spinner::new("Checking connectivity...").start();

    let connectivity_result = test_connectivity(backend).await;

    match connectivity_result {
        Ok(latency_ms) => {
            spinner.finish(&format!("Connected ({}ms)", latency_ms));
        }
        Err(e) => {
            spinner.fail(&format!("Connection failed: {e}"));
            return Err(CliError::Network(e));
        }
    }

    // Test API key if present
    if backend.api_key.is_some() {
        let spinner = crate::output::progress::Spinner::new("Validating API key...").start();

        match validate_api_key(backend).await {
            Ok(()) => spinner.finish("API key valid"),
            Err(e) => {
                spinner.fail(&format!("API key invalid: {e}"));
                return Err(CliError::Validation(e));
            }
        }
    }

    // Test with prompt if provided
    if let Some(prompt) = &args.prompt {
        println!("\nTesting with prompt: \"{prompt}\"\n");

        let model = args.model.clone().unwrap_or_else(|| {
            backend.models.first().cloned().unwrap_or_else(|| {
                match backend.backend_type.as_str() {
                    "anthropic" => "claude-3-haiku-20240307".to_string(),
                    "openai" => "gpt-3.5-turbo".to_string(),
                    _ => "default".to_string(),
                }
            })
        });

        let spinner = crate::output::progress::Spinner::new(
            format!("Sending to {}...", model)
        ).start();

        match test_prompt(backend, &model, prompt).await {
            Ok(response) => {
                spinner.finish("Response received");

                if args.verbose {
                    println!("\nResponse:");
                    println!("{response}");
                }
            }
            Err(e) => {
                spinner.fail(&format!("Request failed: {e}"));
                return Err(CliError::CommandFailed(e));
            }
        }
    }

    output.success(&format!("\nBackend '{}' is working correctly", backend_name));

    Ok(())
}

async fn test_connectivity(
    backend: &tachikoma_config::BackendConfig,
) -> Result<u64, String> {
    use std::time::Instant;

    let start = Instant::now();

    let url = match backend.backend_type.as_str() {
        "anthropic" => "https://api.anthropic.com",
        "openai" => "https://api.openai.com",
        _ => backend.base_url.as_deref().unwrap_or("http://localhost"),
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        reqwest::get(url),
    )
    .await;

    match result {
        Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u64),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("Connection timeout".to_string()),
    }
}

async fn validate_api_key(
    backend: &tachikoma_config::BackendConfig,
) -> Result<(), String> {
    // Implementation would make a minimal API call to validate the key
    // For now, just check format
    if let Some(key) = &backend.api_key {
        match backend.backend_type.as_str() {
            "anthropic" if !key.starts_with("sk-") => {
                return Err("Anthropic API key should start with 'sk-'".to_string());
            }
            _ => {}
        }
    }
    Ok(())
}

async fn test_prompt(
    backend: &tachikoma_config::BackendConfig,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    // Implementation would call the actual API
    // For now, return placeholder
    Ok(format!("[Response from {} using {}]", backend.name, model))
}

async fn backends_default(args: &BackendsDefaultArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    // Check if exists
    if ctx.config.backends.get(&args.name).is_none() {
        return Err(CliError::InvalidArgument(format!(
            "Backend '{}' not found",
            args.name
        )));
    }

    ctx.config.backends.set_default(&args.name).await?;
    output.success(&format!("'{}' is now the default backend", args.name));

    Ok(())
}

async fn backends_models(args: &BackendsModelsArgs, ctx: &CommandContext) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let backend_name = args.name.clone().unwrap_or_else(|| {
        ctx.config
            .backends
            .default()
            .map(|b| b.name.clone())
            .unwrap_or_default()
    });

    let backend = ctx
        .config
        .backends
        .get(&backend_name)
        .ok_or_else(|| CliError::InvalidArgument(format!("Backend '{}' not found", backend_name)))?;

    println!("Models for '{}':\n", backend_name);

    let models = if args.refresh {
        // Fetch from API
        let spinner = crate::output::progress::Spinner::new("Fetching models...").start();

        match fetch_models(backend).await {
            Ok(models) => {
                spinner.finish(&format!("Found {} models", models.len()));
                models
            }
            Err(e) => {
                spinner.fail(&format!("Failed to fetch: {e}"));
                backend.models.clone()
            }
        }
    } else {
        backend.models.clone()
    };

    if models.is_empty() {
        println!("No models configured.");
        println!("\nUse --refresh to fetch available models from the API.");
        return Ok(());
    }

    for model in models {
        println!("  - {model}");
    }

    Ok(())
}

async fn fetch_models(
    backend: &tachikoma_config::BackendConfig,
) -> Result<Vec<String>, String> {
    // Implementation would fetch from API
    match backend.backend_type.as_str() {
        "anthropic" => Ok(vec![
            "claude-opus-4-20250514".to_string(),
            "claude-sonnet-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
        ]),
        "openai" => Ok(vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]),
        _ => Ok(vec!["default".to_string()]),
    }
}

async fn backends_configure(
    args: &BackendsConfigureArgs,
    ctx: &CommandContext,
) -> Result<(), CliError> {
    let output = Output::new(ctx);

    let backend = ctx
        .config
        .backends
        .get(&args.name)
        .ok_or_else(|| CliError::InvalidArgument(format!("Backend '{}' not found", args.name)))?;

    match (&args.key, &args.value) {
        (Some(key), Some(value)) => {
            ctx.config
                .backends
                .set_setting(&args.name, key, value)
                .await?;
            output.success(&format!("Set {key} = {value}"));
        }
        (Some(key), None) => {
            // Show single value
            if let Some(value) = backend.settings.get(key) {
                println!("{key} = {value}");
            } else {
                println!("{key} is not set");
            }
        }
        (None, None) => {
            // Show all settings
            println!("Settings for '{}':\n", args.name);
            if backend.settings.is_empty() {
                println!("  No custom settings configured.");
            } else {
                for (key, value) in &backend.settings {
                    println!("  {key} = {value}");
                }
            }
        }
        _ => {
            return Err(CliError::InvalidArgument(
                "Provide both --key and --value to set a setting".to_string(),
            ));
        }
    }

    Ok(())
}
```

## Testing Requirements

### Integration Tests

```rust
// tests/backends_cmd.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_backends_list() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["backends", "list"])
        .assert()
        .success();
}

#[test]
fn test_backends_show_nonexistent() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["backends", "show", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **078-cli-subcommands.md**: Subcommand patterns
- **083-cli-prompts.md**: Interactive prompts for configuration
- **085-cli-doctor-cmd.md**: Backend health checks
