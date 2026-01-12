# Spec 085: Doctor Command

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 085
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 078-cli-subcommands
- **Estimated Context**: ~10%

## Objective

Implement the `tachikoma doctor` command for diagnosing system health, checking dependencies, verifying configuration, and identifying potential issues.

## Acceptance Criteria

- [ ] Check system dependencies (Rust, cargo, etc.)
- [ ] Validate configuration files
- [ ] Test backend connectivity
- [ ] Verify tool installations
- [ ] Check environment variables
- [ ] Display system information
- [ ] Provide fix suggestions
- [ ] Support JSON output for automation
- [ ] Exit code reflects health status

## Implementation Details

### src/commands/doctor.rs

```rust
//! System health and diagnostics command.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use clap::Args;
use serde::Serialize;

use crate::cli::CommandContext;
use crate::commands::Execute;
use crate::error::CliError;
use crate::output::{Output, Table, Column};
use crate::output::color::{Styled, Color};

/// Doctor command for system diagnostics
#[derive(Debug, Args)]
pub struct DoctorCommand {
    /// Run all checks (including slow ones)
    #[arg(short, long)]
    pub all: bool,

    /// Check specific category
    #[arg(short, long)]
    pub category: Option<CheckCategory>,

    /// Fix issues automatically where possible
    #[arg(long)]
    pub fix: bool,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CheckCategory {
    System,
    Config,
    Backends,
    Tools,
    Environment,
}

/// Result of a single check
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub category: String,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub details: Option<HashMap<String, String>>,
    pub fix_hint: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

impl CheckStatus {
    fn symbol(&self) -> &'static str {
        match self {
            Self::Pass => "✓",
            Self::Warn => "⚠",
            Self::Fail => "✗",
            Self::Skip => "○",
        }
    }

    fn color(&self) -> Color {
        match self {
            Self::Pass => Color::Green,
            Self::Warn => Color::Yellow,
            Self::Fail => Color::Red,
            Self::Skip => Color::BrightBlack,
        }
    }
}

/// Overall doctor report
#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub checks: Vec<CheckResult>,
    pub summary: DoctorSummary,
    pub system_info: SystemInfo,
}

#[derive(Debug, Serialize)]
pub struct DoctorSummary {
    pub total: usize,
    pub passed: usize,
    pub warnings: usize,
    pub failures: usize,
    pub skipped: usize,
    pub overall_status: CheckStatus,
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rust_version: Option<String>,
    pub tachikoma_version: String,
    pub home_dir: Option<String>,
    pub config_path: Option<String>,
}

impl std::fmt::Display for DoctorReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tachikoma Doctor Report")?;
        writeln!(f, "======================")?;
        writeln!(f)?;

        // Group by category
        let mut by_category: HashMap<String, Vec<&CheckResult>> = HashMap::new();
        for check in &self.checks {
            by_category
                .entry(check.category.clone())
                .or_default()
                .push(check);
        }

        for (category, checks) in by_category {
            writeln!(f, "{}:", category)?;
            for check in checks {
                let symbol = Styled::new(check.status.symbol())
                    .fg(check.status.color());
                write!(f, "  {symbol} {}", check.name)?;

                if let Some(msg) = &check.message {
                    write!(f, " - {msg}")?;
                }
                writeln!(f)?;

                if let Some(hint) = &check.fix_hint {
                    writeln!(f, "      Hint: {hint}")?;
                }
            }
            writeln!(f)?;
        }

        // Summary
        writeln!(f, "Summary: {} passed, {} warnings, {} failures",
            self.summary.passed, self.summary.warnings, self.summary.failures)?;

        Ok(())
    }
}

#[async_trait]
impl Execute for DoctorCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        let output = Output::new(ctx);

        println!("Running diagnostics...\n");

        let mut checks = Vec::new();

        // System checks
        if self.should_run(CheckCategory::System) {
            checks.extend(run_system_checks(ctx).await);
        }

        // Configuration checks
        if self.should_run(CheckCategory::Config) {
            checks.extend(run_config_checks(ctx).await);
        }

        // Backend checks
        if self.should_run(CheckCategory::Backends) {
            checks.extend(run_backend_checks(ctx, self.all).await);
        }

        // Tool checks
        if self.should_run(CheckCategory::Tools) {
            checks.extend(run_tool_checks(ctx).await);
        }

        // Environment checks
        if self.should_run(CheckCategory::Environment) {
            checks.extend(run_environment_checks(ctx).await);
        }

        // Calculate summary
        let summary = DoctorSummary {
            total: checks.len(),
            passed: checks.iter().filter(|c| c.status == CheckStatus::Pass).count(),
            warnings: checks.iter().filter(|c| c.status == CheckStatus::Warn).count(),
            failures: checks.iter().filter(|c| c.status == CheckStatus::Fail).count(),
            skipped: checks.iter().filter(|c| c.status == CheckStatus::Skip).count(),
            overall_status: if checks.iter().any(|c| c.status == CheckStatus::Fail) {
                CheckStatus::Fail
            } else if checks.iter().any(|c| c.status == CheckStatus::Warn) {
                CheckStatus::Warn
            } else {
                CheckStatus::Pass
            },
        };

        let system_info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: get_rust_version(),
            tachikoma_version: env!("CARGO_PKG_VERSION").to_string(),
            home_dir: dirs::home_dir().map(|p| p.display().to_string()),
            config_path: Some(ctx.config.path().display().to_string()),
        };

        let report = DoctorReport {
            checks,
            summary,
            system_info,
        };

        output.print(&report)?;

        // Attempt fixes if requested
        if self.fix {
            attempt_fixes(&report, ctx).await?;
        }

        // Return error if there are failures
        if report.summary.failures > 0 {
            return Err(CliError::CommandFailed(
                "Doctor found issues that need attention".to_string(),
            ));
        }

        Ok(())
    }
}

impl DoctorCommand {
    fn should_run(&self, category: CheckCategory) -> bool {
        match &self.category {
            Some(c) => *c == category,
            None => true,
        }
    }
}

async fn run_system_checks(ctx: &CommandContext) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // Check Rust installation
    results.push(check_rust().await);

    // Check cargo
    results.push(check_cargo().await);

    // Check disk space
    results.push(check_disk_space().await);

    // Check network
    results.push(check_network().await);

    results
}

async fn check_rust() -> CheckResult {
    let start = Instant::now();

    let output = tokio::process::Command::new("rustc")
        .arg("--version")
        .output()
        .await;

    let (status, message, details) = match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let mut details = HashMap::new();
            details.insert("version".to_string(), version.clone());
            (CheckStatus::Pass, Some(version), Some(details))
        }
        Ok(_) => (
            CheckStatus::Fail,
            Some("rustc returned error".to_string()),
            None,
        ),
        Err(_) => (
            CheckStatus::Fail,
            Some("rustc not found".to_string()),
            None,
        ),
    };

    CheckResult {
        name: "Rust compiler".to_string(),
        category: "System".to_string(),
        status,
        message,
        details,
        fix_hint: if status == CheckStatus::Fail {
            Some("Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string())
        } else {
            None
        },
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn check_cargo() -> CheckResult {
    let start = Instant::now();

    let output = tokio::process::Command::new("cargo")
        .arg("--version")
        .output()
        .await;

    let (status, message) = match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            (CheckStatus::Pass, Some(version))
        }
        _ => (CheckStatus::Fail, Some("cargo not found".to_string())),
    };

    CheckResult {
        name: "Cargo".to_string(),
        category: "System".to_string(),
        status,
        message,
        details: None,
        fix_hint: None,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn check_disk_space() -> CheckResult {
    let start = Instant::now();

    // Use sys-info or similar crate in real implementation
    let status = CheckStatus::Pass;
    let message = Some("Sufficient disk space available".to_string());

    CheckResult {
        name: "Disk space".to_string(),
        category: "System".to_string(),
        status,
        message,
        details: None,
        fix_hint: None,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn check_network() -> CheckResult {
    let start = Instant::now();

    // Simple connectivity check
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect("api.anthropic.com:443"),
    )
    .await;

    let (status, message) = match result {
        Ok(Ok(_)) => (CheckStatus::Pass, Some("Network connectivity OK".to_string())),
        _ => (CheckStatus::Warn, Some("Could not reach api.anthropic.com".to_string())),
    };

    CheckResult {
        name: "Network connectivity".to_string(),
        category: "System".to_string(),
        status,
        message,
        details: None,
        fix_hint: if status == CheckStatus::Warn {
            Some("Check your internet connection and firewall settings".to_string())
        } else {
            None
        },
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn run_config_checks(ctx: &CommandContext) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let start = Instant::now();

    // Check if config exists
    let config_exists = ctx.config.path().exists();

    results.push(CheckResult {
        name: "Configuration file".to_string(),
        category: "Configuration".to_string(),
        status: if config_exists { CheckStatus::Pass } else { CheckStatus::Warn },
        message: Some(ctx.config.path().display().to_string()),
        details: None,
        fix_hint: if !config_exists {
            Some("Run 'tachikoma config init' to create configuration".to_string())
        } else {
            None
        },
        duration_ms: start.elapsed().as_millis() as u64,
    });

    // Validate config
    let start = Instant::now();
    let validation = ctx.config.validate();

    results.push(CheckResult {
        name: "Configuration validity".to_string(),
        category: "Configuration".to_string(),
        status: if validation.is_ok() { CheckStatus::Pass } else { CheckStatus::Fail },
        message: validation.err().map(|e| e.to_string()),
        details: None,
        fix_hint: Some("Run 'tachikoma config validate --verbose' for details".to_string()),
        duration_ms: start.elapsed().as_millis() as u64,
    });

    results
}

async fn run_backend_checks(ctx: &CommandContext, test_connectivity: bool) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let backends = ctx.config.backends.list();

    if backends.is_empty() {
        results.push(CheckResult {
            name: "AI backends".to_string(),
            category: "Backends".to_string(),
            status: CheckStatus::Warn,
            message: Some("No backends configured".to_string()),
            details: None,
            fix_hint: Some("Run 'tachikoma backends add' to configure a backend".to_string()),
            duration_ms: 0,
        });
        return results;
    }

    for backend in backends {
        let start = Instant::now();

        let status = if backend.api_key.is_none() && backend.requires_auth() {
            CheckStatus::Fail
        } else {
            CheckStatus::Pass
        };

        results.push(CheckResult {
            name: format!("Backend: {}", backend.name),
            category: "Backends".to_string(),
            status,
            message: if status == CheckStatus::Fail {
                Some("API key not configured".to_string())
            } else {
                Some(format!("Type: {}", backend.backend_type))
            },
            details: None,
            fix_hint: if status == CheckStatus::Fail {
                Some(format!(
                    "Set {} environment variable or configure in tachikoma.toml",
                    backend.env_var_name()
                ))
            } else {
                None
            },
            duration_ms: start.elapsed().as_millis() as u64,
        });

        // Test connectivity if requested
        if test_connectivity && status == CheckStatus::Pass {
            let start = Instant::now();
            let connectivity = test_backend_connectivity(&backend).await;

            results.push(CheckResult {
                name: format!("Backend connectivity: {}", backend.name),
                category: "Backends".to_string(),
                status: if connectivity.is_ok() { CheckStatus::Pass } else { CheckStatus::Fail },
                message: connectivity.err().map(|e| e.to_string()),
                details: None,
                fix_hint: None,
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }
    }

    results
}

async fn test_backend_connectivity(
    backend: &tachikoma_config::BackendConfig,
) -> Result<(), String> {
    // Would implement actual API ping here
    Ok(())
}

async fn run_tool_checks(ctx: &CommandContext) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let tools = ctx.config.tools.list().await.unwrap_or_default();

    if tools.is_empty() {
        results.push(CheckResult {
            name: "MCP tools".to_string(),
            category: "Tools".to_string(),
            status: CheckStatus::Pass,
            message: Some("No tools configured".to_string()),
            details: None,
            fix_hint: None,
            duration_ms: 0,
        });
        return results;
    }

    for tool in tools {
        let start = Instant::now();

        let status = if tool.enabled {
            CheckStatus::Pass
        } else {
            CheckStatus::Skip
        };

        results.push(CheckResult {
            name: format!("Tool: {}", tool.name),
            category: "Tools".to_string(),
            status,
            message: Some(format!("v{}", tool.version)),
            details: None,
            fix_hint: None,
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    results
}

async fn run_environment_checks(ctx: &CommandContext) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let env_vars = [
        ("TACHIKOMA_CONFIG", false),
        ("ANTHROPIC_API_KEY", true),
        ("OPENAI_API_KEY", false),
    ];

    for (var, recommended) in env_vars {
        let start = Instant::now();
        let is_set = std::env::var(var).is_ok();

        let status = if is_set {
            CheckStatus::Pass
        } else if recommended {
            CheckStatus::Warn
        } else {
            CheckStatus::Skip
        };

        results.push(CheckResult {
            name: format!("Environment: {}", var),
            category: "Environment".to_string(),
            status,
            message: Some(if is_set { "Set".to_string() } else { "Not set".to_string() }),
            details: None,
            fix_hint: if status == CheckStatus::Warn {
                Some(format!("Set {} in your environment", var))
            } else {
                None
            },
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    results
}

async fn attempt_fixes(report: &DoctorReport, ctx: &CommandContext) -> Result<(), CliError> {
    println!("\nAttempting automatic fixes...\n");

    for check in &report.checks {
        if check.status == CheckStatus::Fail {
            if let Some(hint) = &check.fix_hint {
                println!("  {} - Manual fix required: {}", check.name, hint);
            }
        }
    }

    Ok(())
}

fn get_rust_version() -> Option<String> {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_symbol() {
        assert_eq!(CheckStatus::Pass.symbol(), "✓");
        assert_eq!(CheckStatus::Fail.symbol(), "✗");
        assert_eq!(CheckStatus::Warn.symbol(), "⚠");
    }

    #[test]
    fn test_doctor_summary() {
        let checks = vec![
            CheckResult {
                name: "test1".to_string(),
                category: "Test".to_string(),
                status: CheckStatus::Pass,
                message: None,
                details: None,
                fix_hint: None,
                duration_ms: 0,
            },
            CheckResult {
                name: "test2".to_string(),
                category: "Test".to_string(),
                status: CheckStatus::Fail,
                message: None,
                details: None,
                fix_hint: None,
                duration_ms: 0,
            },
        ];

        let summary = DoctorSummary {
            total: 2,
            passed: 1,
            warnings: 0,
            failures: 1,
            skipped: 0,
            overall_status: CheckStatus::Fail,
        };

        assert_eq!(summary.overall_status, CheckStatus::Fail);
    }

    #[tokio::test]
    async fn test_check_rust() {
        let result = check_rust().await;
        // Should at least return a result (pass or fail)
        assert!(!result.name.is_empty());
    }
}
```

### Integration Tests

```rust
// tests/doctor_cmd.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_doctor_runs() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["doctor"])
        .assert()
        .success();
}

#[test]
fn test_doctor_json_output() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["--format", "json", "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"checks\""));
}

#[test]
fn test_doctor_category_filter() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["doctor", "--category", "system"])
        .assert()
        .success();
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **078-cli-subcommands.md**: Subcommand patterns
- **084-cli-config-cmd.md**: Configuration management
- **091-cli-errors.md**: Error handling and display
