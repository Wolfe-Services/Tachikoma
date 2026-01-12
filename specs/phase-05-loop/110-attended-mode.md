# 110 - Attended Mode

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 110
**Status:** Planned
**Dependencies:** 096-loop-runner-core
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement attended mode for the Ralph Loop - an interactive mode where the user monitors and can intervene at each iteration, approve changes, and guide the development process.

---

## Acceptance Criteria

- [ ] Pause before each iteration for approval
- [ ] Display iteration preview
- [ ] User can approve, skip, or modify
- [ ] Interactive command interface
- [ ] Change review before commit
- [ ] Breakpoint support
- [ ] Session notes/annotations
- [ ] Smooth transition to/from unattended

---

## Implementation Details

### 1. Attended Mode Types (src/attended/types.rs)

```rust
//! Attended mode type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for attended mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendedConfig {
    /// Enable attended mode.
    pub enabled: bool,
    /// Pause before iterations.
    pub pause_before_iteration: bool,
    /// Pause after iterations.
    pub pause_after_iteration: bool,
    /// Require approval for file changes.
    pub require_change_approval: bool,
    /// Auto-approve after timeout (None = wait forever).
    pub auto_approve_timeout: Option<std::time::Duration>,
    /// Show diff preview.
    pub show_diff: bool,
    /// Breakpoints (iteration numbers to pause at).
    pub breakpoints: Vec<u32>,
    /// Interactive commands enabled.
    pub interactive_commands: bool,
}

impl Default for AttendedConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            pause_before_iteration: true,
            pause_after_iteration: false,
            require_change_approval: true,
            auto_approve_timeout: None,
            show_diff: true,
            breakpoints: vec![],
            interactive_commands: true,
        }
    }
}

/// User decision at a pause point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserDecision {
    /// Continue with the iteration.
    Continue,
    /// Skip this iteration.
    Skip,
    /// Pause the loop.
    Pause,
    /// Stop the loop.
    Stop,
    /// Modify the prompt before continuing.
    ModifyPrompt,
    /// Force a reboot.
    ForceReboot,
    /// Enter debug mode.
    Debug,
}

/// Context provided to the user for decision making.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// Current iteration number.
    pub iteration: u32,
    /// Total iterations (0 if unlimited).
    pub total_iterations: u32,
    /// Current prompt.
    pub prompt: String,
    /// Last iteration summary.
    pub last_summary: Option<String>,
    /// Current test status.
    pub test_status: TestStatusSummary,
    /// Progress summary.
    pub progress_summary: ProgressSummary,
    /// Files that will be affected.
    pub affected_files: Vec<String>,
    /// Warnings or alerts.
    pub alerts: Vec<Alert>,
}

/// Test status summary for display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestStatusSummary {
    pub passing: u32,
    pub failing: u32,
    pub skipped: u32,
    pub failure_streak: u32,
}

/// Progress summary for display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressSummary {
    pub iterations_completed: u32,
    pub iterations_with_progress: u32,
    pub no_progress_streak: u32,
    pub files_changed: u32,
}

/// An alert/warning for the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert level.
    pub level: AlertLevel,
    /// Alert message.
    pub message: String,
    /// Suggested action.
    pub suggestion: Option<String>,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
}

/// Interactive command that can be issued.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InteractiveCommand {
    /// Show status.
    Status,
    /// Show help.
    Help,
    /// Show recent logs.
    Logs { count: Option<u32> },
    /// Show test results.
    Tests,
    /// Show metrics.
    Metrics,
    /// Add a note.
    Note { content: String },
    /// Set a breakpoint.
    SetBreakpoint { iteration: u32 },
    /// Remove a breakpoint.
    RemoveBreakpoint { iteration: u32 },
    /// Modify configuration.
    Config { key: String, value: String },
    /// Execute shell command.
    Shell { command: String },
    /// Show diff of changes.
    Diff { file: Option<String> },
    /// Rollback last changes.
    Rollback,
}

/// Session note/annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNote {
    /// When the note was added.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Iteration number (if applicable).
    pub iteration: Option<u32>,
    /// Note content.
    pub content: String,
    /// Note type.
    pub note_type: NoteType,
}

/// Type of session note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteType {
    /// General observation.
    Observation,
    /// Decision made.
    Decision,
    /// Issue discovered.
    Issue,
    /// Action taken.
    Action,
    /// Question/uncertainty.
    Question,
}
```

### 2. Attended Mode Controller (src/attended/controller.rs)

```rust
//! Attended mode controller.

use super::types::{
    Alert, AlertLevel, AttendedConfig, DecisionContext, InteractiveCommand,
    NoteType, ProgressSummary, SessionNote, TestStatusSummary, UserDecision,
};
use crate::error::{LoopError, LoopResult};

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, info, warn};

/// Controls attended mode interactions.
pub struct AttendedController {
    /// Configuration.
    config: RwLock<AttendedConfig>,
    /// Session notes.
    notes: RwLock<Vec<SessionNote>>,
    /// Decision request channel.
    decision_tx: mpsc::Sender<DecisionRequest>,
    /// Decision response receiver (for internal use).
    decision_rx: RwLock<Option<mpsc::Receiver<DecisionRequest>>>,
    /// UI interface.
    ui: Arc<dyn AttendedUI>,
}

/// Request for user decision.
struct DecisionRequest {
    context: DecisionContext,
    response_tx: oneshot::Sender<UserDecision>,
}

/// Interface for attended mode UI.
#[async_trait::async_trait]
pub trait AttendedUI: Send + Sync {
    /// Display context and get user decision.
    async fn request_decision(&self, context: &DecisionContext) -> LoopResult<UserDecision>;

    /// Display a message.
    async fn display_message(&self, message: &str);

    /// Display an alert.
    async fn display_alert(&self, alert: &Alert);

    /// Get interactive command from user.
    async fn get_command(&self) -> LoopResult<Option<InteractiveCommand>>;

    /// Display command result.
    async fn display_command_result(&self, result: &str);

    /// Display diff.
    async fn display_diff(&self, diff: &str);
}

/// Console-based UI implementation.
pub struct ConsoleUI;

#[async_trait::async_trait]
impl AttendedUI for ConsoleUI {
    async fn request_decision(&self, context: &DecisionContext) -> LoopResult<UserDecision> {
        // Display context
        println!("\n{}", "=".repeat(60));
        println!("ITERATION {} / {}", context.iteration,
            if context.total_iterations == 0 { "unlimited".to_string() }
            else { context.total_iterations.to_string() });
        println!("{}", "=".repeat(60));

        // Show test status
        println!("\nTests: {} passing, {} failing (streak: {})",
            context.test_status.passing,
            context.test_status.failing,
            context.test_status.failure_streak);

        // Show progress
        println!("Progress: {}/{} with progress, no-progress streak: {}",
            context.progress_summary.iterations_with_progress,
            context.progress_summary.iterations_completed,
            context.progress_summary.no_progress_streak);

        // Show alerts
        for alert in &context.alerts {
            let prefix = match alert.level {
                AlertLevel::Info => "[INFO]",
                AlertLevel::Warning => "[WARN]",
                AlertLevel::Error => "[ERROR]",
            };
            println!("{} {}", prefix, alert.message);
        }

        // Show affected files
        if !context.affected_files.is_empty() {
            println!("\nAffected files:");
            for file in &context.affected_files {
                println!("  - {}", file);
            }
        }

        // Prompt for decision
        println!("\nOptions:");
        println!("  [c]ontinue  - Run this iteration");
        println!("  [s]kip      - Skip this iteration");
        println!("  [p]ause     - Pause the loop");
        println!("  [q]uit      - Stop the loop");
        println!("  [m]odify    - Modify the prompt");
        println!("  [r]eboot    - Force context reboot");
        println!("  [d]ebug     - Enter debug mode");
        print!("\nChoice [c]: ");

        // Read input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();

        let decision = match input.as_str() {
            "" | "c" | "continue" => UserDecision::Continue,
            "s" | "skip" => UserDecision::Skip,
            "p" | "pause" => UserDecision::Pause,
            "q" | "quit" | "stop" => UserDecision::Stop,
            "m" | "modify" => UserDecision::ModifyPrompt,
            "r" | "reboot" => UserDecision::ForceReboot,
            "d" | "debug" => UserDecision::Debug,
            _ => {
                println!("Unknown option, continuing...");
                UserDecision::Continue
            }
        };

        Ok(decision)
    }

    async fn display_message(&self, message: &str) {
        println!("{}", message);
    }

    async fn display_alert(&self, alert: &Alert) {
        let prefix = match alert.level {
            AlertLevel::Info => "[INFO]",
            AlertLevel::Warning => "[WARN]",
            AlertLevel::Error => "[ERROR]",
        };
        println!("{} {}", prefix, alert.message);
        if let Some(suggestion) = &alert.suggestion {
            println!("  Suggestion: {}", suggestion);
        }
    }

    async fn get_command(&self) -> LoopResult<Option<InteractiveCommand>> {
        print!("cmd> ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        if input.is_empty() {
            return Ok(None);
        }

        let cmd = Self::parse_command(input);
        Ok(cmd)
    }

    async fn display_command_result(&self, result: &str) {
        println!("{}", result);
    }

    async fn display_diff(&self, diff: &str) {
        println!("{}", diff);
    }
}

impl ConsoleUI {
    fn parse_command(input: &str) -> Option<InteractiveCommand> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.first()?;

        match *cmd {
            "status" => Some(InteractiveCommand::Status),
            "help" => Some(InteractiveCommand::Help),
            "logs" => {
                let count = parts.get(1).and_then(|s| s.parse().ok());
                Some(InteractiveCommand::Logs { count })
            }
            "tests" => Some(InteractiveCommand::Tests),
            "metrics" => Some(InteractiveCommand::Metrics),
            "note" => {
                let content = parts[1..].join(" ");
                Some(InteractiveCommand::Note { content })
            }
            "break" | "breakpoint" => {
                let iteration = parts.get(1).and_then(|s| s.parse().ok())?;
                Some(InteractiveCommand::SetBreakpoint { iteration })
            }
            "diff" => {
                let file = parts.get(1).map(|s| s.to_string());
                Some(InteractiveCommand::Diff { file })
            }
            "rollback" => Some(InteractiveCommand::Rollback),
            _ => None,
        }
    }
}

impl AttendedController {
    /// Create a new attended controller.
    pub fn new(config: AttendedConfig, ui: Arc<dyn AttendedUI>) -> Self {
        let (decision_tx, decision_rx) = mpsc::channel(1);
        Self {
            config: RwLock::new(config),
            notes: RwLock::new(Vec::new()),
            decision_tx,
            decision_rx: RwLock::new(Some(decision_rx)),
            ui,
        }
    }

    /// Create with console UI.
    pub fn with_console(config: AttendedConfig) -> Self {
        Self::new(config, Arc::new(ConsoleUI))
    }

    /// Check if attended mode is enabled.
    pub async fn is_enabled(&self) -> bool {
        self.config.read().await.enabled
    }

    /// Request user decision before iteration.
    pub async fn before_iteration(&self, context: DecisionContext) -> LoopResult<UserDecision> {
        let config = self.config.read().await;

        if !config.enabled {
            return Ok(UserDecision::Continue);
        }

        // Check for breakpoint
        let at_breakpoint = config.breakpoints.contains(&context.iteration);

        if !config.pause_before_iteration && !at_breakpoint {
            return Ok(UserDecision::Continue);
        }

        // Check for auto-approve timeout
        if let Some(timeout) = config.auto_approve_timeout {
            match tokio::time::timeout(timeout, self.ui.request_decision(&context)).await {
                Ok(result) => result,
                Err(_) => {
                    info!("Auto-approving after timeout");
                    Ok(UserDecision::Continue)
                }
            }
        } else {
            self.ui.request_decision(&context).await
        }
    }

    /// Show iteration results and optionally pause.
    pub async fn after_iteration(&self, summary: &str) -> LoopResult<()> {
        let config = self.config.read().await;

        if !config.enabled || !config.pause_after_iteration {
            return Ok(());
        }

        self.ui.display_message(&format!("\nIteration complete:\n{}", summary)).await;

        // Handle interactive commands
        if config.interactive_commands {
            loop {
                if let Some(cmd) = self.ui.get_command().await? {
                    let result = self.execute_command(cmd).await?;
                    self.ui.display_command_result(&result).await;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Execute an interactive command.
    async fn execute_command(&self, cmd: InteractiveCommand) -> LoopResult<String> {
        match cmd {
            InteractiveCommand::Status => {
                Ok("Loop is running in attended mode.".to_string())
            }
            InteractiveCommand::Help => {
                Ok(r#"
Available commands:
  status     - Show loop status
  help       - Show this help
  logs [n]   - Show last n log entries
  tests      - Show test results
  metrics    - Show metrics
  note <msg> - Add a session note
  break <n>  - Set breakpoint at iteration n
  diff [f]   - Show diff (optionally for file f)
  rollback   - Rollback last changes
"#.to_string())
            }
            InteractiveCommand::Note { content } => {
                self.add_note(content, NoteType::Observation).await;
                Ok("Note added.".to_string())
            }
            InteractiveCommand::SetBreakpoint { iteration } => {
                let mut config = self.config.write().await;
                if !config.breakpoints.contains(&iteration) {
                    config.breakpoints.push(iteration);
                    config.breakpoints.sort();
                }
                Ok(format!("Breakpoint set at iteration {}", iteration))
            }
            InteractiveCommand::RemoveBreakpoint { iteration } => {
                let mut config = self.config.write().await;
                config.breakpoints.retain(|&i| i != iteration);
                Ok(format!("Breakpoint removed from iteration {}", iteration))
            }
            _ => Ok("Command not implemented yet.".to_string()),
        }
    }

    /// Add a session note.
    pub async fn add_note(&self, content: impl Into<String>, note_type: NoteType) {
        let note = SessionNote {
            timestamp: chrono::Utc::now(),
            iteration: None,
            content: content.into(),
            note_type,
        };
        self.notes.write().await.push(note);
    }

    /// Get all session notes.
    pub async fn get_notes(&self) -> Vec<SessionNote> {
        self.notes.read().await.clone()
    }

    /// Enable attended mode.
    pub async fn enable(&self) {
        self.config.write().await.enabled = true;
    }

    /// Disable attended mode.
    pub async fn disable(&self) {
        self.config.write().await.enabled = false;
    }

    /// Set breakpoint.
    pub async fn set_breakpoint(&self, iteration: u32) {
        let mut config = self.config.write().await;
        if !config.breakpoints.contains(&iteration) {
            config.breakpoints.push(iteration);
            config.breakpoints.sort();
        }
    }

    /// Clear all breakpoints.
    pub async fn clear_breakpoints(&self) {
        self.config.write().await.breakpoints.clear();
    }
}
```

### 3. Module Root (src/attended/mod.rs)

```rust
//! Attended mode for interactive loop control.

pub mod controller;
pub mod types;

pub use controller::{AttendedController, AttendedUI, ConsoleUI};
pub use types::{
    Alert, AlertLevel, AttendedConfig, DecisionContext, InteractiveCommand,
    NoteType, ProgressSummary, SessionNote, TestStatusSummary, UserDecision,
};
```

---

## Testing Requirements

1. Decision prompt displays correctly
2. User input parses correctly
3. Breakpoints trigger pause
4. Auto-approve timeout works
5. Interactive commands execute
6. Notes are persisted
7. Enable/disable works correctly
8. Context information is accurate

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Next: [111-unattended-mode.md](111-unattended-mode.md)
- Related: [112-mode-switching.md](112-mode-switching.md)
