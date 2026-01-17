//! Init command implementation.

use clap::Parser;
use std::path::Path;
use std::fs;
use dialoguer::{Confirm, Select, Input};
use console::{style, Emoji};

use crate::cli::CommandContext;
use crate::error::CliError;
use tachikoma_common_config::{TachikomaConfig, env::ApiKeys};

static SPIDER: Emoji<'_, '_> = Emoji("🕷️", "");
static CHECK: Emoji<'_, '_> = Emoji("✓", "");
static CROSS: Emoji<'_, '_> = Emoji("✗", "");
static PARTY: Emoji<'_, '_> = Emoji("🎉", "");

/// Initialize a new Tachikoma project
#[derive(Debug, Parser)]
pub struct InitCommand {
    /// Skip interactive prompts and use defaults
    #[arg(short, long)]
    quick: bool,
    
    /// Default brain backend
    #[arg(long)]
    brain: Option<String>,
    
    /// Default think-tank/oracle backend
    #[arg(long)]
    oracle: Option<String>,
}

#[derive(Debug)]
struct SystemStatus {
    rust_version: Option<String>,
    node_version: Option<String>,
    jj_version: Option<String>,
    git_version: Option<String>,
    ripgrep_available: bool,
    anthropic_key: bool,
    openai_key: bool,
    google_key: bool,
}

impl InitCommand {
    pub async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        if !self.quick {
            println!("{} {}", SPIDER, style("Tachikoma Setup").bold());
            println!();
        }

        // Check system status
        let status = self.check_system().await;
        
        if !self.quick {
            self.display_system_status(&status);
        }

        // Create or update configuration
        let config = if self.quick {
            self.create_quick_config()?
        } else {
            self.create_interactive_config(&status)?
        };

        // Ensure .tachikoma directory exists
        fs::create_dir_all(".tachikoma")
            .map_err(|e| CliError::Io(format!("Failed to create .tachikoma directory: {}", e)))?;

        // Write configuration
        let config_path = Path::new(".tachikoma/config.yaml");
        let config_yaml = serde_yaml::to_string(&config)
            .map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e).into()))?;
        
        fs::write(config_path, config_yaml)
            .map_err(|e| CliError::Io(format!("Failed to write config file: {}", e)))?;

        if !self.quick {
            println!("  {} Created .tachikoma/config.yaml", CHECK);
        }

        // Create specs directory and sample
        self.create_specs_directory()?;
        
        if !self.quick {
            println!("  {} Created specs/README.md", CHECK);
            println!("  {} Created specs/001-getting-started.md", CHECK);
        }

        // Show completion message
        if self.quick {
            println!("{} Quick setup with defaults...", SPIDER);
            println!("  {} Environment OK", CHECK);
            println!("  {} Config created", CHECK);
            println!("  {} Ready to go!", CHECK);
            println!();
            println!("Run `tachikoma run` to start.");
        } else {
            println!();
            println!("{} Setup complete!", PARTY);
            println!();
            println!("Next steps:");
            println!("  1. Run `tachikoma chat` to create your first spec");
            println!("  2. Run `tachikoma run` to implement it");
            println!("  3. Run `tachikoma loop` for continuous development");
            println!();
            println!("Happy coding, Major! {}", SPIDER);
        }

        Ok(())
    }

    async fn check_system(&self) -> SystemStatus {
        SystemStatus {
            rust_version: self.check_tool_version("rustc", "--version").await,
            node_version: self.check_tool_version("node", "--version").await,
            jj_version: self.check_tool_version("jj", "--version").await,
            git_version: self.check_tool_version("git", "--version").await,
            ripgrep_available: self.check_tool_available("rg").await,
            anthropic_key: ApiKeys::anthropic().is_some(),
            openai_key: ApiKeys::openai().is_some(),
            google_key: ApiKeys::google().is_some(),
        }
    }

    async fn check_tool_version(&self, cmd: &str, arg: &str) -> Option<String> {
        use tokio::process::Command;
        
        match Command::new(cmd).arg(arg).output().await {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Extract version from first line, handling different formats
                output_str.lines().next()
                    .and_then(|line| {
                        // Common patterns: "tool 1.2.3", "tool version 1.2.3", etc.
                        line.split_whitespace()
                            .find(|word| word.chars().next().unwrap_or(' ').is_ascii_digit())
                            .map(|s| s.to_string())
                    })
            }
            _ => None,
        }
    }

    async fn check_tool_available(&self, cmd: &str) -> bool {
        use tokio::process::Command;
        
        Command::new(cmd)
            .arg("--help")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn display_system_status(&self, status: &SystemStatus) {
        println!("Checking your environment...");
        
        if let Some(ref version) = status.rust_version {
            println!("  {} Rust {} installed", CHECK, version);
        } else {
            println!("  {} Rust not found - install from https://rustup.rs/", CROSS);
        }

        if let Some(ref version) = status.node_version {
            println!("  {} Node.js {} installed", CHECK, version);
        } else {
            println!("  {} Node.js not found - install from https://nodejs.org/", CROSS);
        }

        if let Some(ref version) = status.jj_version {
            println!("  {} jj {} installed", CHECK, version);
        } else if let Some(ref version) = status.git_version {
            println!("  {} git {} installed", CHECK, version);
        } else {
            println!("  {} No version control found - install jj or git", CROSS);
        }

        if status.ripgrep_available {
            println!("  {} ripgrep installed", CHECK);
        }

        if status.anthropic_key {
            println!("  {} ANTHROPIC_API_KEY found", CHECK);
        }
        if status.openai_key {
            println!("  {} OPENAI_API_KEY found", CHECK);
        }
        if status.google_key {
            println!("  {} GOOGLE_API_KEY found", CHECK);
        }

        if !status.anthropic_key && !status.openai_key && !status.google_key {
            println!("  {} No API keys found - set ANTHROPIC_API_KEY or others", CROSS);
        }

        println!();
    }

    fn create_quick_config(&self) -> Result<TachikomaConfig, CliError> {
        let mut config = TachikomaConfig::default();
        
        if let Some(ref brain) = self.brain {
            config.backend.brain = brain.clone();
        }
        
        if let Some(ref oracle) = self.oracle {
            config.backend.think_tank = oracle.clone();
        }

        Ok(config)
    }

    fn create_interactive_config(&self, status: &SystemStatus) -> Result<TachikomaConfig, CliError> {
        println!("Configure your project:");
        
        // Brain selection
        let brain_options = vec!["claude", "openai", "gemini"];
        let brain_default = if status.anthropic_key { 0 } 
                          else if status.openai_key { 1 } 
                          else if status.google_key { 2 } 
                          else { 0 };
        
        let brain_idx = Select::new()
            .with_prompt("Default AI backend (brain)")
            .items(&brain_options)
            .default(brain_default)
            .interact()
            .map_err(|e| CliError::Io(format!("Input error: {}", e)))?;
        
        // Oracle selection  
        let oracle_options = vec!["o3", "claude", "gemini"];
        let oracle_idx = Select::new()
            .with_prompt("Default think-tank (oracle)")
            .items(&oracle_options)
            .default(0)
            .interact()
            .map_err(|e| CliError::Io(format!("Input error: {}", e)))?;
        
        // Attended mode
        let attended = Confirm::new()
            .with_prompt("Enable attended mode by default?")
            .default(true)
            .interact()
            .map_err(|e| CliError::Io(format!("Input error: {}", e)))?;

        // Build config
        let mut config = TachikomaConfig::default();
        config.backend.brain = brain_options[brain_idx].to_string();
        config.backend.think_tank = oracle_options[oracle_idx].to_string();
        config.policies.attended_by_default = attended;

        Ok(config)
    }

    fn create_specs_directory(&self) -> Result<(), CliError> {
        // Create specs directory
        fs::create_dir_all("specs")
            .map_err(|e| CliError::Io(format!("Failed to create specs directory: {}", e)))?;

        // Create README.md
        let readme_content = include_str!("../templates/specs_readme.md");
        fs::write("specs/README.md", readme_content)
            .map_err(|e| CliError::Io(format!("Failed to create specs/README.md: {}", e)))?;

        // Create getting started spec
        let getting_started_content = include_str!("../templates/getting_started_spec.md");
        fs::write("specs/001-getting-started.md", getting_started_content)
            .map_err(|e| CliError::Io(format!("Failed to create getting started spec: {}", e)))?;

        Ok(())
    }
}