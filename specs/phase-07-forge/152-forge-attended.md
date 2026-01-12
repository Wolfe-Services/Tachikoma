# 152 - Forge Attended Mode

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 152
**Status:** Planned
**Dependencies:** 139-forge-rounds, 153-forge-cli
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement attended mode for Forge sessions where a human operator can observe, intervene, provide feedback, and guide the brainstorming process in real-time.

---

## Acceptance Criteria

- [ ] Pause/resume between rounds
- [ ] Human feedback injection
- [ ] Override model decisions
- [ ] Skip/repeat rounds
- [ ] Real-time progress display
- [ ] Interactive conflict resolution
- [ ] Force convergence option

---

## Implementation Details

### 1. Attended Mode Controller (src/attended/controller.rs)

```rust
//! Attended mode controller for human-in-the-loop operation.

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::{
    ConflictResolution, DetectedConflict, ForgeEvent, ForgeResult, ForgeRound,
    ForgeSession, OrchestratorControl, Participant,
};

/// Controller for attended mode operation.
pub struct AttendedController {
    /// Control channel to orchestrator.
    control_tx: mpsc::Sender<OrchestratorControl>,
    /// Event receiver.
    event_rx: broadcast::Receiver<ForgeEvent>,
    /// Current session state.
    session: Arc<RwLock<ForgeSession>>,
    /// Pending human decisions.
    pending_decisions: Arc<RwLock<Vec<PendingDecision>>>,
    /// UI callback for prompts.
    ui_callback: Arc<dyn AttendedUI + Send + Sync>,
}

/// Interface for attended mode UI.
#[async_trait::async_trait]
pub trait AttendedUI: Send + Sync {
    /// Display a message.
    async fn display_message(&self, message: &str);

    /// Display progress.
    async fn display_progress(&self, progress: &ProgressInfo);

    /// Prompt for confirmation.
    async fn confirm(&self, prompt: &str) -> bool;

    /// Prompt for text input.
    async fn prompt_input(&self, prompt: &str) -> Option<String>;

    /// Prompt for choice selection.
    async fn prompt_choice(&self, prompt: &str, choices: &[String]) -> Option<usize>;

    /// Display round summary.
    async fn display_round_summary(&self, round: &ForgeRound);

    /// Display conflict for resolution.
    async fn display_conflict(&self, conflict: &DetectedConflict) -> ConflictAction;
}

/// Progress information for display.
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Current round.
    pub round: usize,
    /// Max rounds.
    pub max_rounds: usize,
    /// Current phase.
    pub phase: String,
    /// Current cost.
    pub cost: f64,
    /// Max cost.
    pub max_cost: f64,
    /// Convergence score.
    pub convergence_score: Option<f64>,
    /// Time elapsed.
    pub elapsed_secs: u64,
}

/// Action for conflict resolution.
#[derive(Debug, Clone)]
pub enum ConflictAction {
    /// Accept the suggested resolution.
    Accept,
    /// Choose a specific position.
    ChoosePosition(usize),
    /// Provide custom resolution.
    Custom(String),
    /// Defer to automatic resolution.
    Defer,
}

/// A pending decision for human input.
#[derive(Debug, Clone)]
pub struct PendingDecision {
    /// Decision ID.
    pub id: String,
    /// Type of decision.
    pub decision_type: PendingDecisionType,
    /// Context for the decision.
    pub context: String,
    /// Available options.
    pub options: Vec<String>,
}

/// Type of pending decision.
#[derive(Debug, Clone)]
pub enum PendingDecisionType {
    /// Round completion review.
    RoundReview,
    /// Conflict resolution.
    ConflictResolution,
    /// Convergence decision.
    ConvergenceDecision,
    /// Continue or stop.
    ContinueDecision,
}

impl AttendedController {
    /// Create a new attended controller.
    pub fn new(
        control_tx: mpsc::Sender<OrchestratorControl>,
        event_rx: broadcast::Receiver<ForgeEvent>,
        session: Arc<RwLock<ForgeSession>>,
        ui_callback: Arc<dyn AttendedUI + Send + Sync>,
    ) -> Self {
        Self {
            control_tx,
            event_rx,
            session,
            pending_decisions: Arc::new(RwLock::new(Vec::new())),
            ui_callback,
        }
    }

    /// Run the attended mode loop.
    pub async fn run(&mut self) -> ForgeResult<()> {
        loop {
            // Wait for event
            match self.event_rx.recv().await {
                Ok(event) => {
                    if !self.handle_event(event).await? {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }

        Ok(())
    }

    /// Handle a forge event.
    async fn handle_event(&mut self, event: ForgeEvent) -> ForgeResult<bool> {
        match event {
            ForgeEvent::SessionStarted { session_id } => {
                self.ui_callback.display_message(
                    &format!("Session {} started in attended mode", session_id)
                ).await;
            }

            ForgeEvent::RoundStarted { round_number, round_type } => {
                self.ui_callback.display_message(
                    &format!("Starting round {}: {}", round_number, round_type)
                ).await;
            }

            ForgeEvent::RoundCompleted { round_number, round_type } => {
                // Get the completed round
                let session = self.session.read().await;
                if let Some(round) = session.rounds.get(round_number) {
                    self.ui_callback.display_round_summary(round).await;
                }
                drop(session);

                // Ask to continue
                if !self.prompt_continue(&round_type).await? {
                    self.control_tx.send(OrchestratorControl::Pause).await
                        .map_err(|_| crate::ForgeError::Orchestration("Failed to pause".to_string()))?;

                    // Wait for explicit resume
                    return self.wait_for_resume().await;
                }
            }

            ForgeEvent::ConvergenceCheck { score, converged } => {
                self.ui_callback.display_message(
                    &format!("Convergence check: score {:.2}, {}",
                        score,
                        if converged { "CONVERGED" } else { "not converged" })
                ).await;

                if !converged {
                    // Ask if should force convergence
                    let force = self.ui_callback.confirm(
                        "Session has not converged. Force convergence anyway?"
                    ).await;

                    if force {
                        // Would need to implement force convergence
                        self.ui_callback.display_message("Forcing convergence...").await;
                    }
                }
            }

            ForgeEvent::CostUpdate { total_cost, budget_remaining } => {
                let session = self.session.read().await;
                let progress = ProgressInfo {
                    round: session.current_round,
                    max_rounds: session.config.max_rounds,
                    phase: "In progress".to_string(),
                    cost: total_cost,
                    max_cost: session.config.max_cost_usd,
                    convergence_score: None,
                    elapsed_secs: session.created_at.elapsed().num_seconds() as u64,
                };
                drop(session);

                self.ui_callback.display_progress(&progress).await;

                // Warn if cost is high
                if budget_remaining < total_cost * 0.2 {
                    self.ui_callback.display_message(
                        &format!("WARNING: Only ${:.2} budget remaining!", budget_remaining)
                    ).await;
                }
            }

            ForgeEvent::SessionCompleted { final_output } => {
                self.ui_callback.display_message("Session completed!").await;
                self.ui_callback.display_message(
                    &format!("Final output length: {} characters", final_output.len())
                ).await;
                return Ok(false);
            }

            ForgeEvent::Error { message, recoverable } => {
                self.ui_callback.display_message(
                    &format!("ERROR{}: {}", if recoverable { " (recoverable)" } else { "" }, message)
                ).await;

                if !recoverable {
                    return Ok(false);
                }
            }

            _ => {}
        }

        Ok(true)
    }

    /// Prompt to continue after round.
    async fn prompt_continue(&self, round_type: &str) -> ForgeResult<bool> {
        let prompt = format!(
            "{} round completed. Continue to next round?",
            round_type
        );

        Ok(self.ui_callback.confirm(&prompt).await)
    }

    /// Wait for explicit resume command.
    async fn wait_for_resume(&mut self) -> ForgeResult<bool> {
        self.ui_callback.display_message(
            "Session paused. Enter feedback or 'continue' to resume."
        ).await;

        loop {
            let input = self.ui_callback.prompt_input("Action: ").await;

            match input.as_deref() {
                Some("continue") | Some("resume") | Some("c") => {
                    self.control_tx.send(OrchestratorControl::Resume).await
                        .map_err(|_| crate::ForgeError::Orchestration("Failed to resume".to_string()))?;
                    return Ok(true);
                }

                Some("abort") | Some("quit") | Some("q") => {
                    self.control_tx.send(OrchestratorControl::Abort).await
                        .map_err(|_| crate::ForgeError::Orchestration("Failed to abort".to_string()))?;
                    return Ok(false);
                }

                Some("skip") => {
                    self.control_tx.send(OrchestratorControl::SkipRound).await
                        .map_err(|_| crate::ForgeError::Orchestration("Failed to skip".to_string()))?;
                    self.control_tx.send(OrchestratorControl::Resume).await
                        .map_err(|_| crate::ForgeError::Orchestration("Failed to resume".to_string()))?;
                    return Ok(true);
                }

                Some(feedback) if feedback.starts_with("feedback:") => {
                    let feedback_text = feedback.trim_start_matches("feedback:").trim();
                    self.control_tx.send(OrchestratorControl::InjectFeedback(feedback_text.to_string())).await
                        .map_err(|_| crate::ForgeError::Orchestration("Failed to inject feedback".to_string()))?;
                    self.ui_callback.display_message("Feedback recorded.").await;
                }

                Some("help") | Some("?") => {
                    self.ui_callback.display_message(
                        "Commands:\n\
                         - continue/resume/c: Continue to next round\n\
                         - abort/quit/q: Stop the session\n\
                         - skip: Skip the next round\n\
                         - feedback: <text>: Inject feedback for next round\n\
                         - help/?: Show this help"
                    ).await;
                }

                Some(other) => {
                    self.ui_callback.display_message(
                        &format!("Unknown command: {}. Type 'help' for options.", other)
                    ).await;
                }

                None => {
                    // User cancelled
                    return Ok(false);
                }
            }
        }
    }

    /// Inject human feedback.
    pub async fn inject_feedback(&self, feedback: &str) -> ForgeResult<()> {
        self.control_tx.send(OrchestratorControl::InjectFeedback(feedback.to_string())).await
            .map_err(|_| crate::ForgeError::Orchestration("Failed to inject feedback".to_string()))?;
        Ok(())
    }

    /// Force a pause.
    pub async fn pause(&self) -> ForgeResult<()> {
        self.control_tx.send(OrchestratorControl::Pause).await
            .map_err(|_| crate::ForgeError::Orchestration("Failed to pause".to_string()))?;
        Ok(())
    }

    /// Resume the session.
    pub async fn resume(&self) -> ForgeResult<()> {
        self.control_tx.send(OrchestratorControl::Resume).await
            .map_err(|_| crate::ForgeError::Orchestration("Failed to resume".to_string()))?;
        Ok(())
    }

    /// Abort the session.
    pub async fn abort(&self) -> ForgeResult<()> {
        self.control_tx.send(OrchestratorControl::Abort).await
            .map_err(|_| crate::ForgeError::Orchestration("Failed to abort".to_string()))?;
        Ok(())
    }
}

/// Terminal-based attended UI implementation.
pub struct TerminalAttendedUI;

#[async_trait::async_trait]
impl AttendedUI for TerminalAttendedUI {
    async fn display_message(&self, message: &str) {
        println!("\n{}", message);
    }

    async fn display_progress(&self, progress: &ProgressInfo) {
        println!(
            "\r[Round {}/{}] {} | Cost: ${:.2}/${:.2} | {}",
            progress.round,
            progress.max_rounds,
            progress.phase,
            progress.cost,
            progress.max_cost,
            format_duration(progress.elapsed_secs)
        );
    }

    async fn confirm(&self, prompt: &str) -> bool {
        println!("{} [y/n]: ", prompt);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        input.trim().to_lowercase().starts_with('y')
    }

    async fn prompt_input(&self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    async fn prompt_choice(&self, prompt: &str, choices: &[String]) -> Option<usize> {
        println!("{}", prompt);
        for (i, choice) in choices.iter().enumerate() {
            println!("  {}: {}", i + 1, choice);
        }

        let input = self.prompt_input("Choice: ").await?;
        input.parse::<usize>().ok().map(|n| n.saturating_sub(1))
    }

    async fn display_round_summary(&self, round: &ForgeRound) {
        println!("\n--- Round Summary ---");
        match round {
            ForgeRound::Draft(d) => {
                println!("Draft by: {}", d.drafter.display_name);
                println!("Content length: {} chars", d.content.len());
                println!("Tokens: {} in, {} out", d.tokens.input, d.tokens.output);
            }
            ForgeRound::Critique(c) => {
                println!("Critiques received: {}", c.critiques.len());
                for crit in &c.critiques {
                    println!("  - {}: Score {}", crit.critic.display_name, crit.score);
                }
            }
            ForgeRound::Synthesis(s) => {
                println!("Synthesized by: {}", s.synthesizer.display_name);
                println!("Conflicts resolved: {}", s.resolved_conflicts.len());
                println!("Changes made: {}", s.changes.len());
            }
            ForgeRound::Refinement(r) => {
                println!("Refined by: {}", r.refiner.display_name);
                println!("Focus area: {}", r.focus_area);
                println!("Depth: {}", r.depth);
            }
            ForgeRound::Convergence(c) => {
                println!("Convergence score: {:.2}", c.score);
                println!("Converged: {}", c.converged);
                println!("Remaining issues: {}", c.remaining_issues.len());
            }
        }
        println!("---\n");
    }

    async fn display_conflict(&self, conflict: &DetectedConflict) -> ConflictAction {
        println!("\n=== Conflict Detected ===");
        println!("Topic: {}", conflict.topic.description());
        println!("Severity: {}/5", conflict.severity);
        println!("\nPositions:");

        for (i, pos) in conflict.positions.iter().enumerate() {
            println!("  {}: {} - {}", i + 1, pos.participant.display_name, pos.statement);
        }

        println!("\nSuggested strategies: {:?}", conflict.suggested_strategies);

        let choices = vec![
            "Accept automatic resolution".to_string(),
            "Choose a position".to_string(),
            "Provide custom resolution".to_string(),
            "Defer".to_string(),
        ];

        match self.prompt_choice("How to resolve?", &choices).await {
            Some(0) => ConflictAction::Accept,
            Some(1) => {
                let pos_choices: Vec<String> = conflict.positions.iter()
                    .map(|p| format!("{}: {}", p.participant.display_name, p.statement))
                    .collect();
                self.prompt_choice("Which position?", &pos_choices).await
                    .map(ConflictAction::ChoosePosition)
                    .unwrap_or(ConflictAction::Defer)
            }
            Some(2) => {
                self.prompt_input("Enter custom resolution: ").await
                    .map(ConflictAction::Custom)
                    .unwrap_or(ConflictAction::Defer)
            }
            _ => ConflictAction::Defer,
        }
    }
}

/// Format duration as human-readable string.
fn format_duration(secs: u64) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}
```

---

## Testing Requirements

1. Pause/resume controls work correctly
2. Feedback injection is processed
3. Round summaries display correctly
4. User input is handled properly
5. Cost warnings trigger at threshold
6. Conflict resolution choices work

---

## Related Specs

- Depends on: [139-forge-rounds.md](139-forge-rounds.md)
- Depends on: [153-forge-cli.md](153-forge-cli.md)
- Next: [153-forge-cli.md](153-forge-cli.md)
- Used by: [153-forge-cli.md](153-forge-cli.md)
