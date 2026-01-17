//! Doctor command implementation.

use clap::Parser;
use console::{style, Emoji};
use std::path::Path;

use crate::cli::CommandContext;
use crate::error::CliError;
use tachikoma_common_config::{TachikomaConfig, Detection, env::ApiKeys};

static SPIDER: Emoji<'_, '_> = Emoji("🕷️", "");
static CHECK: Emoji<'_, '_> = Emoji("✓", "");
static CROSS: Emoji<'_, '_> = Emoji("✗", "");
static WARNING: Emoji<'_, '_> = Emoji("⚠", "!");
static GREEN: Emoji<'_, '_> = Emoji("🟢", "G");
static YELLOW: Emoji<'_, '_> = Emoji("🟡", "Y");
static RED: Emoji<'_, '_> = Emoji("🔴", "R");

/// Check system health and dependencies
#[derive(Debug, Parser)]
pub struct DoctorCommand {
    /// Show verbose output with recommendations
    #[arg(short, long)]
    verbose: bool,
    
    /// Only check specific component (tools, keys, config)
    #[arg(long)]
    check: Option<String>,
}

impl DoctorCommand {
    pub async fn execute(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        println!("{} {}", SPIDER, style("Tachikoma Health Check").bold());
        println!();

        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut all_good = true;

        // Check environment tools
        if self.should_check("tools") {
            let (tool_issues, tool_warnings) = self.check_environment().await;
            issues.extend(tool_issues);
            warnings.extend(tool_warnings);
            if !issues.is_empty() || !warnings.is_empty() {
                all_good = false;
            }
            println!();
        }

        // Check API keys
        if self.should_check("keys") {
            let (key_issues, key_warnings) = self.check_api_keys();
            issues.extend(key_issues);
            warnings.extend(key_warnings);
            if !issues.is_empty() || !warnings.is_empty() {
                all_good = false;
            }
            println!();
        }

        // Check configuration
        if self.should_check("config") {
            let (config_issues, config_warnings) = self.check_configuration().await;
            issues.extend(config_issues);
            warnings.extend(config_warnings);
            if !issues.is_empty() || !warnings.is_empty() {
                all_good = false;
            }
            println!();
        }

        // Summary
        if !issues.is_empty() {
            println!("{}", style("Issues found:").red().bold());
            for (i, issue) in issues.iter().enumerate() {
                println!("  {}. {}", i + 1, issue);
            }
            println!();
        }

        if !warnings.is_empty() {
            println!("{}", style("Warnings:").yellow().bold());
            for (i, warning) in warnings.iter().enumerate() {
                println!("  {}. {}", i + 1, warning);
            }
            println!();
        }

        // Overall status
        let (status_emoji, status_text) = if !issues.is_empty() {
            (RED, style("Not ready - fix issues above").red())
        } else if !warnings.is_empty() {
            (YELLOW, style("Ready with warnings").yellow())
        } else {
            (GREEN, style("All systems go!").green())
        };

        println!("Overall: {} {}", status_emoji, status_text);

        // Show recommendations in verbose mode
        if self.verbose && (!issues.is_empty() || !warnings.is_empty()) {
            self.show_recommendations().await;
        }

        Ok(())
    }

    fn should_check(&self, component: &str) -> bool {
        match &self.check {
            Some(filter) => filter == component,
            None => true,
        }
    }

    async fn check_environment(&self) -> (Vec<String>, Vec<String>) {
        println!("{}", style("Environment:").bold());
        
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        let status = Detection::scan().await;

        // Check Rust
        if let Some(rust) = status.tools.get("rust") {
            if rust.available {
                if let Some(ref version) = rust.version {
                    println!("  {} Rust {}", CHECK, version);
                } else {
                    println!("  {} Rust installed", CHECK);
                }
            } else {
                println!("  {} Rust not found", CROSS);
                issues.push("Install Rust: https://rustup.rs/".to_string());
            }
        }

        // Check Node.js
        if let Some(node) = status.tools.get("node") {
            if node.available {
                if let Some(ref version) = node.version {
                    println!("  {} Node.js {}", CHECK, version);
                } else {
                    println!("  {} Node.js installed", CHECK);
                }
            } else {
                println!("  {} Node.js not found", CROSS);
                issues.push("Install Node.js: https://nodejs.org/".to_string());
            }
        }

        // Check version control
        let jj = status.tools.get("jj").map(|t| t.available).unwrap_or(false);
        let git = status.tools.get("git").map(|t| t.available).unwrap_or(false);
        
        if jj {
            if let Some(jj_tool) = status.tools.get("jj") {
                if let Some(ref version) = jj_tool.version {
                    println!("  {} jj {}", CHECK, version);
                } else {
                    println!("  {} jj installed", CHECK);
                }
            }
        } else if git {
            if let Some(git_tool) = status.tools.get("git") {
                if let Some(ref version) = git_tool.version {
                    println!("  {} git {}", CHECK, version);
                } else {
                    println!("  {} git installed", CHECK);
                }
            }
        } else {
            println!("  {} No version control found", CROSS);
            issues.push("Install jj (recommended) or git".to_string());
        }

        // Check optional tools
        if let Some(rg) = status.tools.get("ripgrep") {
            if rg.available {
                println!("  {} ripgrep installed", CHECK);
            } else {
                println!("  {} ripgrep not found (code_search won't work)", WARNING);
                warnings.push("Install ripgrep for better code search".to_string());
            }
        }

        (issues, warnings)
    }

    fn check_api_keys(&self) -> (Vec<String>, Vec<String>) {
        println!("{}", style("API Keys:").bold());
        
        let mut issues = Vec::new();
        let warnings = Vec::new();
        
        let anthropic = ApiKeys::anthropic().is_some();
        let openai = ApiKeys::openai().is_some();
        let google = ApiKeys::google().is_some();

        if anthropic {
            println!("  {} ANTHROPIC_API_KEY", CHECK);
        } else {
            println!("  {} ANTHROPIC_API_KEY (Claude)", CROSS);
        }

        if openai {
            println!("  {} OPENAI_API_KEY", CHECK);
        } else {
            println!("  {} OPENAI_API_KEY (optional)", CROSS);
        }

        if google {
            println!("  {} GOOGLE_API_KEY", CHECK);
        } else {
            println!("  {} GOOGLE_API_KEY (optional)", CROSS);
        }

        if !anthropic && !openai && !google {
            issues.push("Set at least one API key: ANTHROPIC_API_KEY, OPENAI_API_KEY, or GOOGLE_API_KEY".to_string());
        }

        (issues, warnings)
    }

    async fn check_configuration(&self) -> (Vec<String>, Vec<String>) {
        println!("{}", style("Configuration:").bold());
        
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check if config file exists
        let config_path = Path::new(".tachikoma/config.yaml");
        if config_path.exists() {
            println!("  {} .tachikoma/config.yaml exists", CHECK);
            
            // Try to load and validate config
            match std::fs::read_to_string(config_path) {
                Ok(content) => {
                    match serde_yaml::from_str::<TachikomaConfig>(&content) {
                        Ok(config) => {
                            println!("  {} Configuration is valid", CHECK);
                            
                            // Check if backends are available
                            if ApiKeys::for_backend(&config.backend.brain).is_some() {
                                println!("  {} Backend '{}' is available", CHECK, config.backend.brain);
                            } else {
                                println!("  {} Backend '{}' missing API key", WARNING, config.backend.brain);
                                warnings.push(format!("Set API key for backend '{}'", config.backend.brain));
                            }
                        }
                        Err(e) => {
                            println!("  {} Configuration has errors", CROSS);
                            issues.push(format!("Fix config syntax: {}", e));
                        }
                    }
                }
                Err(e) => {
                    println!("  {} Cannot read config file", CROSS);
                    issues.push(format!("Fix config file permissions: {}", e));
                }
            }
        } else {
            println!("  {} .tachikoma/config.yaml missing", CROSS);
            issues.push("Run `tachikoma init` to create configuration".to_string());
        }

        // Check specs directory
        let specs_path = Path::new("specs");
        if specs_path.exists() && specs_path.is_dir() {
            // Count spec files
            if let Ok(entries) = std::fs::read_dir(specs_path) {
                let spec_count = entries
                    .filter_map(Result::ok)
                    .filter(|entry| {
                        entry.path().extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "md")
                            .unwrap_or(false)
                    })
                    .count();
                
                if spec_count > 0 {
                    println!("  {} Specs directory found ({} specs)", CHECK, spec_count);
                } else {
                    println!("  {} Specs directory empty", WARNING);
                    warnings.push("Create some specs to get started".to_string());
                }
            } else {
                println!("  {} Cannot read specs directory", WARNING);
                warnings.push("Check specs directory permissions".to_string());
            }
        } else {
            println!("  {} Specs directory missing", WARNING);
            warnings.push("Create specs/ directory for your specifications".to_string());
        }

        (issues, warnings)
    }

    async fn show_recommendations(&self) {
        println!();
        println!("{}", style("Recommendations:").bold());
        
        let recommendations = Detection::install_recommendations();
        
        for (tool, commands) in recommendations {
            println!("  {}", style(format!("Install {}:", tool)).bold());
            for command in commands {
                println!("    {}", command);
            }
            println!();
        }
        
        println!("  {}", style("Set API Keys:").bold());
        println!("    export ANTHROPIC_API_KEY=your-key-here");
        println!("    export OPENAI_API_KEY=your-key-here");
        println!("    export GOOGLE_API_KEY=your-key-here");
        println!();
        
        println!("  {}", style("Initialize Project:").bold());
        println!("    tachikoma init");
        println!();
    }
}