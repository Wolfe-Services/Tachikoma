# 153 - Forge CLI Command

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 153
**Status:** Planned
**Dependencies:** 139-forge-rounds, 150-forge-persistence, 151-forge-resume, 152-forge-attended
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement the command-line interface for Forge, allowing users to start, manage, and interact with brainstorming sessions from the terminal.

---

## Acceptance Criteria

- [x] `forge new` command to start sessions
- [x] `forge resume` command for continuation
- [x] `forge list` command for session listing
- [x] `forge show` command for session details
- [x] `forge export` command for output export
- [x] `--attended` flag for interactive mode
- [x] Progress display and status updates

---

## Implementation Details

### 1. CLI Structure (src/cli/forge.rs)

```rust
//! Forge CLI command implementation.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::{ForgeSessionId, OutputType};

/// Forge - Multi-model brainstorming for specifications.
#[derive(Parser)]
#[command(name = "forge")]
#[command(about = "Multi-model AI brainstorming for specifications")]
pub struct ForgeCli {
    #[command(subcommand)]
    pub command: ForgeCommand,
}

/// Forge subcommands.
#[derive(Subcommand)]
pub enum ForgeCommand {
    /// Start a new brainstorming session.
    New(NewArgs),

    /// Resume an existing session.
    Resume(ResumeArgs),

    /// List all sessions.
    List(ListArgs),

    /// Show session details.
    Show(ShowArgs),

    /// Export session output.
    Export(ExportArgs),

    /// Delete a session.
    Delete(DeleteArgs),

    /// Configure forge settings.
    Config(ConfigArgs),
}

/// Arguments for `forge new`.
#[derive(Args)]
pub struct NewArgs {
    /// Title for the brainstorming topic.
    #[arg(short, long)]
    pub title: String,

    /// Description of what to brainstorm.
    #[arg(short, long)]
    pub description: String,

    /// Output type (spec, code, docs, design).
    #[arg(short, long, default_value = "spec")]
    pub output_type: String,

    /// Run in attended (interactive) mode.
    #[arg(long)]
    pub attended: bool,

    /// Constraints to apply.
    #[arg(short, long)]
    pub constraint: Vec<String>,

    /// Reference files to include.
    #[arg(short, long)]
    pub reference: Vec<PathBuf>,

    /// Maximum cost in USD.
    #[arg(long)]
    pub max_cost: Option<f64>,

    /// Maximum rounds.
    #[arg(long)]
    pub max_rounds: Option<usize>,

    /// Convergence threshold (0.0-1.0).
    #[arg(long)]
    pub threshold: Option<f64>,

    /// Number of critics.
    #[arg(long, default_value = "2")]
    pub critics: usize,

    /// Output file for results.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Session name (defaults to title).
    #[arg(long)]
    pub name: Option<String>,
}

/// Arguments for `forge resume`.
#[derive(Args)]
pub struct ResumeArgs {
    /// Session ID to resume.
    #[arg(short, long)]
    pub session: Option<String>,

    /// Resume from specific round.
    #[arg(long)]
    pub from_round: Option<usize>,

    /// Run in attended mode.
    #[arg(long)]
    pub attended: bool,

    /// Override max cost.
    #[arg(long)]
    pub max_cost: Option<f64>,

    /// Override max rounds.
    #[arg(long)]
    pub max_rounds: Option<usize>,

    /// Force resume even if session is complete.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `forge list`.
#[derive(Args)]
pub struct ListArgs {
    /// Filter by status (active, complete, all).
    #[arg(short, long, default_value = "all")]
    pub status: String,

    /// Output format (table, json).
    #[arg(short, long, default_value = "table")]
    pub format: String,

    /// Limit number of results.
    #[arg(short, long)]
    pub limit: Option<usize>,
}

/// Arguments for `forge show`.
#[derive(Args)]
pub struct ShowArgs {
    /// Session ID to show.
    pub session: String,

    /// Show full round details.
    #[arg(long)]
    pub rounds: bool,

    /// Show decision log.
    #[arg(long)]
    pub decisions: bool,

    /// Show dissent log.
    #[arg(long)]
    pub dissents: bool,

    /// Output format (text, json).
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

/// Arguments for `forge export`.
#[derive(Args)]
pub struct ExportArgs {
    /// Session ID to export.
    pub session: String,

    /// Output file.
    #[arg(short, long)]
    pub output: PathBuf,

    /// Export format (md, json, yaml).
    #[arg(short, long, default_value = "md")]
    pub format: String,

    /// Include metadata.
    #[arg(long)]
    pub include_metadata: bool,

    /// Include decision log.
    #[arg(long)]
    pub include_decisions: bool,

    /// Include dissent log.
    #[arg(long)]
    pub include_dissents: bool,
}

/// Arguments for `forge delete`.
#[derive(Args)]
pub struct DeleteArgs {
    /// Session ID to delete.
    pub session: String,

    /// Skip confirmation.
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for `forge config`.
#[derive(Args)]
pub struct ConfigArgs {
    /// Show current configuration.
    #[arg(long)]
    pub show: bool,

    /// Set a configuration value.
    #[arg(long)]
    pub set: Option<String>,

    /// Reset to defaults.
    #[arg(long)]
    pub reset: bool,
}
```

### 2. CLI Executor (src/cli/executor.rs)

```rust
//! CLI command execution.

use std::sync::Arc;

use crate::{
    AttendedController, BrainstormTopic, ExportArgs, ForgeConfig, ForgeOrchestrator,
    ForgeResult, ForgeSessionId, ListArgs, NewArgs, OutputType, Reference,
    ReferenceType, ResumeArgs, ResumeConfigOverride, ResumeOptions,
    SessionResumer, SessionStore, ShowArgs, TerminalAttendedUI,
    create_participant_manager,
};

/// Execute the forge CLI.
pub async fn execute_forge(command: crate::ForgeCommand) -> ForgeResult<()> {
    let config = load_forge_config()?;

    match command {
        crate::ForgeCommand::New(args) => execute_new(args, config).await,
        crate::ForgeCommand::Resume(args) => execute_resume(args, config).await,
        crate::ForgeCommand::List(args) => execute_list(args, config).await,
        crate::ForgeCommand::Show(args) => execute_show(args, config).await,
        crate::ForgeCommand::Export(args) => execute_export(args, config).await,
        crate::ForgeCommand::Delete(args) => execute_delete(args, config).await,
        crate::ForgeCommand::Config(args) => execute_config(args, config).await,
    }
}

/// Execute `forge new`.
async fn execute_new(args: NewArgs, mut config: ForgeConfig) -> ForgeResult<()> {
    // Parse output type
    let output_type = match args.output_type.to_lowercase().as_str() {
        "spec" | "specification" => OutputType::Specification,
        "code" => OutputType::Code,
        "docs" | "documentation" => OutputType::Documentation,
        "design" => OutputType::Design,
        _ => OutputType::Freeform,
    };

    // Build topic
    let mut topic = BrainstormTopic::new(&args.title, &args.description);
    topic.output_type = output_type;

    for constraint in args.constraint {
        topic = topic.with_constraint(constraint);
    }

    for ref_path in args.reference {
        let content = tokio::fs::read_to_string(&ref_path).await
            .map_err(|e| crate::ForgeError::Io(format!("Failed to read reference: {}", e)))?;

        topic = topic.with_reference(Reference {
            name: ref_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("reference")
                .to_string(),
            content,
            ref_type: ReferenceType::Inline,
        });
    }

    // Apply config overrides
    if let Some(max_cost) = args.max_cost {
        config.limits.max_cost_usd = max_cost;
    }
    if let Some(max_rounds) = args.max_rounds {
        config.convergence.max_rounds = max_rounds;
    }
    if let Some(threshold) = args.threshold {
        config.convergence.threshold = threshold;
    }
    config.defaults.attended = args.attended;

    // Initialize components
    let participants = Arc::new(create_participant_manager(config.clone())?);
    let store = Arc::new(SessionStore::new(config.clone()).await?);

    // Create orchestrator
    let (mut orchestrator, event_rx, control_tx) =
        ForgeOrchestrator::new(topic, participants.clone(), config.clone());

    println!("Starting Forge session: {}", args.title);
    println!("Output type: {:?}", output_type);
    println!("Attended mode: {}", args.attended);
    println!();

    // Run with or without attended mode
    let result = if args.attended {
        let session = orchestrator.session_arc();
        let ui = Arc::new(TerminalAttendedUI);
        let mut controller = AttendedController::new(control_tx, event_rx, session, ui);

        // Run orchestrator in background
        let orch_handle = tokio::spawn(async move {
            orchestrator.run().await
        });

        // Run attended controller
        controller.run().await?;

        orch_handle.await
            .map_err(|e| crate::ForgeError::Orchestration(format!("Orchestrator task failed: {}", e)))?
    } else {
        // Run with progress display
        let progress_handle = spawn_progress_display(event_rx);
        let result = orchestrator.run().await;
        drop(progress_handle);
        result
    }?;

    // Save final session
    let save_path = store.save(&result).await?;
    println!("\nSession saved to: {}", save_path.display());

    // Export if output specified
    if let Some(output_path) = args.output {
        let content = result.latest_draft().unwrap_or_default();
        tokio::fs::write(&output_path, content).await
            .map_err(|e| crate::ForgeError::Io(format!("Failed to write output: {}", e)))?;
        println!("Output written to: {}", output_path.display());
    }

    // Print summary
    print_session_summary(&result);

    Ok(())
}

/// Execute `forge resume`.
async fn execute_resume(args: ResumeArgs, config: ForgeConfig) -> ForgeResult<()> {
    let store = Arc::new(SessionStore::new(config.clone()).await?);

    // Find session to resume
    let session_id = if let Some(id_str) = args.session {
        id_str.parse::<ForgeSessionId>()
            .map_err(|_| crate::ForgeError::NotFound("Invalid session ID".to_string()))?
    } else {
        // Find most recent resumable session
        let resumer = SessionResumer::new(store.clone(), config.clone());
        let resumable = resumer.list_resumable().await?;

        if resumable.is_empty() {
            return Err(crate::ForgeError::NotFound("No resumable sessions found".to_string()));
        }

        println!("Available sessions to resume:");
        for (i, session) in resumable.iter().enumerate() {
            println!("  {}: {} - {} ({} rounds)", i + 1, session.id, session.name, session.round_count);
        }

        println!("\nSelect session (1-{}): ", resumable.len());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();

        let idx: usize = input.trim().parse()
            .map_err(|_| crate::ForgeError::Orchestration("Invalid selection".to_string()))?;

        if idx < 1 || idx > resumable.len() {
            return Err(crate::ForgeError::Orchestration("Invalid selection".to_string()));
        }

        resumable[idx - 1].id.clone()
    };

    // Build resume options
    let mut options = ResumeOptions::default();
    options.from_round = args.from_round;
    options.force = args.force;

    if args.max_cost.is_some() || args.max_rounds.is_some() {
        options.config_override = Some(ResumeConfigOverride {
            max_cost_usd: args.max_cost,
            max_rounds: args.max_rounds,
            convergence_threshold: None,
            attended: Some(args.attended),
        });
    }

    // Resume
    let participants = Arc::new(create_participant_manager(config.clone())?);
    let resumer = SessionResumer::new(store.clone(), config.clone());

    let (mut orchestrator, event_rx, control_tx) =
        resumer.resume(&session_id, options, participants).await?;

    println!("Resuming session: {}", session_id);

    // Run (similar to new)
    let result = if args.attended {
        let session = orchestrator.session_arc();
        let ui = Arc::new(TerminalAttendedUI);
        let mut controller = AttendedController::new(control_tx, event_rx, session, ui);

        let orch_handle = tokio::spawn(async move {
            orchestrator.run().await
        });

        controller.run().await?;

        orch_handle.await
            .map_err(|e| crate::ForgeError::Orchestration(format!("Task failed: {}", e)))?
    } else {
        let progress_handle = spawn_progress_display(event_rx);
        let result = orchestrator.run().await;
        drop(progress_handle);
        result
    }?;

    store.save(&result).await?;
    print_session_summary(&result);

    Ok(())
}

/// Execute `forge list`.
async fn execute_list(args: ListArgs, config: ForgeConfig) -> ForgeResult<()> {
    let store = SessionStore::new(config).await?;
    let mut sessions = store.list().await?;

    // Filter by status
    match args.status.as_str() {
        "active" => {
            sessions.retain(|s| {
                matches!(s.status,
                    crate::ForgeSessionStatus::InProgress |
                    crate::ForgeSessionStatus::Paused |
                    crate::ForgeSessionStatus::Initialized)
            });
        }
        "complete" => {
            sessions.retain(|s| {
                matches!(s.status,
                    crate::ForgeSessionStatus::Complete |
                    crate::ForgeSessionStatus::Converged)
            });
        }
        _ => {} // "all" - no filter
    }

    // Apply limit
    if let Some(limit) = args.limit {
        sessions.truncate(limit);
    }

    // Output
    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&sessions)
                .map_err(|e| crate::ForgeError::Serialization(e.to_string()))?;
            println!("{}", json);
        }
        _ => {
            println!("{:<40} {:<20} {:<10} {:<8} {:<10}",
                "ID", "Name", "Status", "Rounds", "Cost");
            println!("{}", "-".repeat(90));

            for session in sessions {
                println!("{:<40} {:<20} {:<10} {:<8} ${:<.2}",
                    session.id.to_string(),
                    truncate_string(&session.name, 20),
                    format!("{:?}", session.status),
                    session.round_count,
                    session.total_cost_usd);
            }
        }
    }

    Ok(())
}

/// Execute `forge show`.
async fn execute_show(args: ShowArgs, config: ForgeConfig) -> ForgeResult<()> {
    let store = SessionStore::new(config).await?;

    let session_id: ForgeSessionId = args.session.parse()
        .map_err(|_| crate::ForgeError::NotFound("Invalid session ID".to_string()))?;

    let session = store.load(&session_id).await?;

    println!("Session: {}", session.id);
    println!("Name: {}", session.name);
    println!("Status: {:?}", session.status);
    println!("Topic: {}", session.topic.title);
    println!("Rounds: {}", session.rounds.len());
    println!("Cost: ${:.2}", session.total_cost_usd);
    println!("Created: {}", session.created_at);
    println!("Updated: {}", session.updated_at);

    if args.rounds {
        println!("\n## Rounds\n");
        for (i, round) in session.rounds.iter().enumerate() {
            println!("### Round {}: {:?}", i, round_type_name(round));
            // Print round details
        }
    }

    Ok(())
}

/// Execute `forge export`.
async fn execute_export(args: ExportArgs, config: ForgeConfig) -> ForgeResult<()> {
    let store = SessionStore::new(config).await?;

    let session_id: ForgeSessionId = args.session.parse()
        .map_err(|_| crate::ForgeError::NotFound("Invalid session ID".to_string()))?;

    let session = store.load(&session_id).await?;

    let content = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&session)
            .map_err(|e| crate::ForgeError::Serialization(e.to_string()))?,
        "yaml" => serde_yaml::to_string(&session)
            .map_err(|e| crate::ForgeError::Serialization(e.to_string()))?,
        _ => {
            // Markdown - export the final draft
            session.latest_draft().unwrap_or_default().to_string()
        }
    };

    tokio::fs::write(&args.output, content).await
        .map_err(|e| crate::ForgeError::Io(format!("Failed to write: {}", e)))?;

    println!("Exported to: {}", args.output.display());

    Ok(())
}

/// Execute `forge delete`.
async fn execute_delete(args: DeleteArgs, config: ForgeConfig) -> ForgeResult<()> {
    let store = SessionStore::new(config).await?;

    let session_id: ForgeSessionId = args.session.parse()
        .map_err(|_| crate::ForgeError::NotFound("Invalid session ID".to_string()))?;

    if !args.force {
        println!("Delete session {}? [y/N]: ", session_id);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    store.delete(&session_id).await?;
    println!("Session {} deleted.", session_id);

    Ok(())
}

/// Execute `forge config`.
async fn execute_config(args: crate::ConfigArgs, config: ForgeConfig) -> ForgeResult<()> {
    if args.show {
        let yaml = serde_yaml::to_string(&config)
            .map_err(|e| crate::ForgeError::Serialization(e.to_string()))?;
        println!("{}", yaml);
    }

    Ok(())
}

// Helper functions

fn load_forge_config() -> ForgeResult<ForgeConfig> {
    // Try to load from .tachikoma/forge/config.yaml
    let config_path = std::path::Path::new(".tachikoma/forge/config.yaml");

    if config_path.exists() {
        crate::config_loader::load_config(Some(config_path))
    } else {
        Ok(ForgeConfig::default())
    }
}

fn spawn_progress_display(mut rx: tokio::sync::broadcast::Receiver<crate::ForgeEvent>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                crate::ForgeEvent::RoundStarted { round_number, round_type } => {
                    println!("Round {}: Starting {}...", round_number, round_type);
                }
                crate::ForgeEvent::RoundCompleted { round_number, round_type } => {
                    println!("Round {}: {} complete", round_number, round_type);
                }
                crate::ForgeEvent::ConvergenceCheck { score, converged } => {
                    println!("Convergence: {:.2} - {}", score, if converged { "CONVERGED" } else { "continuing" });
                }
                crate::ForgeEvent::CostUpdate { total_cost, .. } => {
                    print!("\rCost: ${:.2}", total_cost);
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
                crate::ForgeEvent::Error { message, .. } => {
                    eprintln!("Error: {}", message);
                }
                _ => {}
            }
        }
    })
}

fn print_session_summary(session: &crate::ForgeSession) {
    println!("\n=== Session Summary ===");
    println!("Status: {:?}", session.status);
    println!("Rounds: {}", session.rounds.len());
    println!("Total cost: ${:.2}", session.total_cost_usd);
    println!("Duration: {}s", session.created_at.elapsed().num_seconds());

    if let Some(content) = session.latest_draft() {
        println!("Output length: {} characters", content.len());
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn round_type_name(round: &crate::ForgeRound) -> &'static str {
    match round {
        crate::ForgeRound::Draft(_) => "Draft",
        crate::ForgeRound::Critique(_) => "Critique",
        crate::ForgeRound::Synthesis(_) => "Synthesis",
        crate::ForgeRound::Refinement(_) => "Refinement",
        crate::ForgeRound::Convergence(_) => "Convergence",
    }
}
```

---

## Testing Requirements

1. `forge new` creates sessions correctly
2. `forge resume` finds and continues sessions
3. `forge list` displays sessions properly
4. `forge show` displays details correctly
5. `forge export` outputs valid files
6. Attended mode activates correctly
7. Progress display updates in real-time

---

## Related Specs

- Depends on: [139-forge-rounds.md](139-forge-rounds.md)
- Depends on: [150-forge-persistence.md](150-forge-persistence.md)
- Depends on: [151-forge-resume.md](151-forge-resume.md)
- Depends on: [152-forge-attended.md](152-forge-attended.md)
- Next: [154-forge-output.md](154-forge-output.md)
- Used by: Main CLI entry point
