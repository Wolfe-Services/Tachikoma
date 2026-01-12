# 139 - Forge Round Orchestration

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 139
**Status:** Planned
**Dependencies:** 136-forge-session-types, 138-forge-participants
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement the core round orchestration engine that drives Forge sessions through draft, critique, synthesis, and refinement rounds with proper state management and error recovery.

---

## Acceptance Criteria

- [x] `ForgeOrchestrator` as the main execution engine
- [x] State machine for round progression
- [x] Support for all round types (draft, critique, synthesis, refinement)
- [x] Pause/resume capability for attended mode
- [x] Error recovery with round retry
- [x] Event emission for progress tracking
- [x] Timeout handling per round

---

## Implementation Details

### 1. Orchestrator Core (src/orchestrator.rs)

```rust
//! Forge session orchestrator.

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{timeout, Duration};

use crate::{
    BrainstormTopic, ConvergenceRound, CritiqueRound, DraftRound, ForgeConfig,
    ForgeError, ForgeResult, ForgeRound, ForgeSession, ForgeSessionConfig,
    ForgeSessionStatus, Participant, ParticipantManager, RefinementRound,
    SynthesisRound, TokenCount,
};

/// Events emitted during orchestration.
#[derive(Debug, Clone)]
pub enum ForgeEvent {
    /// Session started.
    SessionStarted { session_id: String },
    /// Round started.
    RoundStarted { round_number: usize, round_type: String },
    /// Round completed.
    RoundCompleted { round_number: usize, round_type: String },
    /// Participant responded.
    ParticipantResponse { participant: String, tokens: TokenCount },
    /// Convergence check result.
    ConvergenceCheck { score: f64, converged: bool },
    /// Cost update.
    CostUpdate { total_cost: f64, budget_remaining: f64 },
    /// Session paused (attended mode).
    SessionPaused { reason: String },
    /// Session completed.
    SessionCompleted { final_output: String },
    /// Error occurred.
    Error { message: String, recoverable: bool },
}

/// Orchestrator for running Forge sessions.
pub struct ForgeOrchestrator {
    /// Session being orchestrated.
    session: Arc<RwLock<ForgeSession>>,
    /// Participant manager.
    participants: Arc<ParticipantManager>,
    /// Configuration.
    config: ForgeConfig,
    /// Event sender.
    event_tx: broadcast::Sender<ForgeEvent>,
    /// Control channel for pause/resume.
    control_rx: mpsc::Receiver<OrchestratorControl>,
    /// Control sender (for cloning).
    control_tx: mpsc::Sender<OrchestratorControl>,
    /// Is paused.
    is_paused: Arc<RwLock<bool>>,
}

/// Control commands for the orchestrator.
#[derive(Debug, Clone)]
pub enum OrchestratorControl {
    /// Pause the session.
    Pause,
    /// Resume the session.
    Resume,
    /// Abort the session.
    Abort,
    /// Skip current round.
    SkipRound,
    /// Inject user feedback.
    InjectFeedback(String),
}

impl ForgeOrchestrator {
    /// Create a new orchestrator.
    pub fn new(
        topic: BrainstormTopic,
        participants: Arc<ParticipantManager>,
        config: ForgeConfig,
    ) -> (Self, broadcast::Receiver<ForgeEvent>, mpsc::Sender<OrchestratorControl>) {
        let session = ForgeSession::new(topic.title.clone(), topic);
        let (event_tx, event_rx) = broadcast::channel(100);
        let (control_tx, control_rx) = mpsc::channel(10);

        let orchestrator = Self {
            session: Arc::new(RwLock::new(session)),
            participants,
            config,
            event_tx,
            control_rx,
            control_tx: control_tx.clone(),
            is_paused: Arc::new(RwLock::new(false)),
        };

        (orchestrator, event_rx, control_tx)
    }

    /// Run the orchestration loop.
    pub async fn run(&mut self) -> ForgeResult<ForgeSession> {
        // Initialize session
        {
            let mut session = self.session.write().await;
            session.status = ForgeSessionStatus::InProgress;
        }

        self.emit(ForgeEvent::SessionStarted {
            session_id: self.session.read().await.id.to_string(),
        });

        // Main orchestration loop
        loop {
            // Check for control messages
            if let Ok(control) = self.control_rx.try_recv() {
                match self.handle_control(control).await? {
                    ControlResult::Continue => {}
                    ControlResult::Abort => break,
                    ControlResult::Paused => {
                        self.wait_for_resume().await?;
                    }
                }
            }

            // Check termination conditions
            if self.should_terminate().await? {
                break;
            }

            // Execute next round
            let round_result = self.execute_next_round().await;

            match round_result {
                Ok(round) => {
                    self.record_round(round).await?;
                }
                Err(e) if e.is_recoverable() => {
                    self.emit(ForgeEvent::Error {
                        message: e.to_string(),
                        recoverable: true,
                    });
                    // Retry after delay
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    self.emit(ForgeEvent::Error {
                        message: e.to_string(),
                        recoverable: false,
                    });
                    return Err(e);
                }
            }
        }

        // Finalize session
        self.finalize_session().await
    }

    /// Execute the next appropriate round.
    async fn execute_next_round(&mut self) -> ForgeResult<ForgeRound> {
        let session = self.session.read().await;
        let round_number = session.rounds.len();

        let round_type = self.determine_next_round_type(&session);
        drop(session);

        self.emit(ForgeEvent::RoundStarted {
            round_number,
            round_type: round_type.clone(),
        });

        let round = match round_type.as_str() {
            "draft" => self.execute_draft_round(round_number).await?,
            "critique" => self.execute_critique_round(round_number).await?,
            "synthesis" => self.execute_synthesis_round(round_number).await?,
            "refinement" => self.execute_refinement_round(round_number).await?,
            "convergence" => self.execute_convergence_round(round_number).await?,
            _ => return Err(ForgeError::Orchestration(
                format!("Unknown round type: {}", round_type)
            )),
        };

        self.emit(ForgeEvent::RoundCompleted {
            round_number,
            round_type,
        });

        Ok(round)
    }

    /// Determine what type of round should come next.
    fn determine_next_round_type(&self, session: &ForgeSession) -> String {
        if session.rounds.is_empty() {
            return "draft".to_string();
        }

        let last_round = session.rounds.last().unwrap();

        match last_round {
            ForgeRound::Draft(_) => "critique".to_string(),
            ForgeRound::Critique(_) => "synthesis".to_string(),
            ForgeRound::Synthesis(_) => {
                // Check if we should do refinement or convergence
                if session.config.recursive_refinement
                    && session.rounds.len() < self.config.convergence.max_rounds - 1
                {
                    "convergence".to_string()
                } else {
                    "convergence".to_string()
                }
            }
            ForgeRound::Refinement(_) => "convergence".to_string(),
            ForgeRound::Convergence(c) => {
                if c.converged {
                    "complete".to_string()
                } else {
                    "critique".to_string()
                }
            }
        }
    }

    /// Execute a draft round.
    async fn execute_draft_round(&self, round_number: usize) -> ForgeResult<ForgeRound> {
        let drafter = self.participants.get_drafter().await?;
        let topic = &self.session.read().await.topic;

        let request = crate::prompts::build_draft_prompt(topic, &self.config);

        let timeout_duration = Duration::from_secs(self.config.rounds.draft.timeout_secs);
        let response = timeout(
            timeout_duration,
            self.participants.send_request(&drafter, request),
        )
        .await
        .map_err(|_| ForgeError::Timeout("Draft round timed out".to_string()))??;

        self.emit(ForgeEvent::ParticipantResponse {
            participant: drafter.display_name.clone(),
            tokens: response.tokens.clone(),
        });

        Ok(ForgeRound::Draft(DraftRound {
            round_number,
            drafter,
            content: response.content,
            prompt: "draft".to_string(),
            timestamp: response.timestamp,
            tokens: response.tokens,
            duration_ms: response.duration_ms,
        }))
    }

    /// Execute a critique round.
    async fn execute_critique_round(&self, round_number: usize) -> ForgeResult<ForgeRound> {
        let critics = self.participants.get_critics(
            self.config.rounds.critique.min_critiques
        ).await?;

        let current_content = self.session.read().await.latest_draft()
            .ok_or_else(|| ForgeError::Orchestration("No draft to critique".to_string()))?
            .to_string();

        let topic = &self.session.read().await.topic.clone();

        // Build critique requests
        let requests: Vec<_> = critics.iter().map(|critic| {
            let request = crate::prompts::build_critique_prompt(
                &current_content,
                topic,
                critic,
                &self.config,
            );
            (critic.clone(), request)
        }).collect();

        // Execute in parallel if configured
        let timeout_duration = Duration::from_secs(self.config.rounds.critique.timeout_secs);
        let responses = if self.config.rounds.critique.parallel {
            timeout(
                timeout_duration,
                self.participants.send_parallel_requests(requests),
            )
            .await
            .map_err(|_| ForgeError::Timeout("Critique round timed out".to_string()))?
        } else {
            let mut results = Vec::new();
            for (participant, request) in requests {
                let result = timeout(
                    timeout_duration,
                    self.participants.send_request(&participant, request),
                )
                .await
                .map_err(|_| ForgeError::Timeout("Critique timed out".to_string()))?;
                results.push(result);
            }
            results
        };

        // Parse critiques
        let mut critiques = Vec::new();
        for (response_result, critic) in responses.into_iter().zip(critics.iter()) {
            match response_result {
                Ok(response) => {
                    self.emit(ForgeEvent::ParticipantResponse {
                        participant: critic.display_name.clone(),
                        tokens: response.tokens.clone(),
                    });

                    let critique = crate::parsing::parse_critique(&response, critic)?;
                    critiques.push(critique);
                }
                Err(e) => {
                    // Log but continue with other critiques
                    self.emit(ForgeEvent::Error {
                        message: format!("Critique from {} failed: {}", critic.display_name, e),
                        recoverable: true,
                    });
                }
            }
        }

        if critiques.len() < self.config.rounds.critique.min_critiques {
            return Err(ForgeError::Orchestration(
                format!("Only {} critiques received, need {}",
                    critiques.len(),
                    self.config.rounds.critique.min_critiques)
            ));
        }

        Ok(ForgeRound::Critique(CritiqueRound {
            round_number,
            critiques,
            timestamp: tachikoma_common_core::Timestamp::now(),
        }))
    }

    /// Execute a synthesis round.
    async fn execute_synthesis_round(&self, round_number: usize) -> ForgeResult<ForgeRound> {
        let synthesizer = self.participants.get_synthesizer().await?;

        let session = self.session.read().await;
        let current_content = session.latest_draft()
            .ok_or_else(|| ForgeError::Orchestration("No content to synthesize".to_string()))?
            .to_string();

        // Get most recent critiques
        let critiques = session.rounds.iter().rev()
            .find_map(|r| match r {
                ForgeRound::Critique(c) => Some(&c.critiques),
                _ => None,
            })
            .ok_or_else(|| ForgeError::Orchestration("No critiques to synthesize".to_string()))?;

        let topic = &session.topic.clone();
        drop(session);

        let request = crate::prompts::build_synthesis_prompt(
            &current_content,
            critiques,
            topic,
            &self.config,
        );

        let timeout_duration = Duration::from_secs(self.config.rounds.synthesis.timeout_secs);
        let response = timeout(
            timeout_duration,
            self.participants.send_request(&synthesizer, request),
        )
        .await
        .map_err(|_| ForgeError::Timeout("Synthesis round timed out".to_string()))??;

        self.emit(ForgeEvent::ParticipantResponse {
            participant: synthesizer.display_name.clone(),
            tokens: response.tokens.clone(),
        });

        let (merged_content, resolved_conflicts, changes) =
            crate::parsing::parse_synthesis(&response)?;

        Ok(ForgeRound::Synthesis(SynthesisRound {
            round_number,
            synthesizer,
            merged_content,
            resolved_conflicts,
            changes,
            timestamp: response.timestamp,
            tokens: response.tokens,
            duration_ms: response.duration_ms,
        }))
    }

    /// Execute a refinement round.
    async fn execute_refinement_round(&self, round_number: usize) -> ForgeResult<ForgeRound> {
        let refiner = self.participants.get_drafter().await?;

        let session = self.session.read().await;
        let current_content = session.latest_draft()
            .ok_or_else(|| ForgeError::Orchestration("No content to refine".to_string()))?
            .to_string();

        // Determine refinement depth and focus
        let current_depth = session.rounds.iter()
            .filter(|r| matches!(r, ForgeRound::Refinement(_)))
            .count();

        let focus_areas = &self.config.rounds.refinement.focus_areas;
        let focus_area = focus_areas
            .get(current_depth % focus_areas.len())
            .cloned()
            .unwrap_or_else(|| "general".to_string());

        let topic = &session.topic.clone();
        drop(session);

        let request = crate::prompts::build_refinement_prompt(
            &current_content,
            &focus_area,
            current_depth,
            topic,
            &self.config,
        );

        let timeout_duration = Duration::from_secs(self.config.rounds.refinement.timeout_secs);
        let response = timeout(
            timeout_duration,
            self.participants.send_request(&refiner, request),
        )
        .await
        .map_err(|_| ForgeError::Timeout("Refinement round timed out".to_string()))??;

        self.emit(ForgeEvent::ParticipantResponse {
            participant: refiner.display_name.clone(),
            tokens: response.tokens.clone(),
        });

        Ok(ForgeRound::Refinement(RefinementRound {
            round_number,
            refiner,
            focus_area,
            refined_content: response.content,
            depth: current_depth + 1,
            timestamp: response.timestamp,
            tokens: response.tokens,
            duration_ms: response.duration_ms,
        }))
    }

    /// Execute a convergence check round.
    async fn execute_convergence_round(&self, round_number: usize) -> ForgeResult<ForgeRound> {
        let participants = self.participants.active_participants().await;

        let session = self.session.read().await;
        let current_content = session.latest_draft()
            .ok_or_else(|| ForgeError::Orchestration("No content to check".to_string()))?
            .to_string();
        let topic = &session.topic.clone();
        drop(session);

        // Get convergence votes from each participant
        let mut votes = Vec::new();
        let mut total_tokens = TokenCount::default();

        for participant in &participants {
            let request = crate::prompts::build_convergence_prompt(
                &current_content,
                topic,
                &self.config,
            );

            let response = self.participants.send_request(participant, request).await?;
            total_tokens.add(&response.tokens);

            let vote = crate::parsing::parse_convergence_vote(&response, participant)?;
            votes.push(vote);
        }

        // Calculate convergence score
        let agreeing = votes.iter().filter(|v| v.agrees).count();
        let total = votes.len();
        let score = agreeing as f64 / total as f64;

        let converged = score >= self.config.convergence.threshold
            && agreeing >= self.config.convergence.min_consensus;

        // Collect remaining issues
        let remaining_issues: Vec<String> = votes.iter()
            .flat_map(|v| v.concerns.clone())
            .collect();

        self.emit(ForgeEvent::ConvergenceCheck { score, converged });

        Ok(ForgeRound::Convergence(ConvergenceRound {
            round_number,
            score,
            converged,
            votes,
            remaining_issues,
            timestamp: tachikoma_common_core::Timestamp::now(),
            tokens: total_tokens,
        }))
    }

    /// Check if session should terminate.
    async fn should_terminate(&self) -> ForgeResult<bool> {
        let session = self.session.read().await;

        // Check if converged
        if session.is_converged() {
            return Ok(true);
        }

        // Check round limit
        if session.rounds.len() >= self.config.convergence.max_rounds {
            return Ok(true);
        }

        // Check cost limit
        if session.total_cost_usd >= self.config.limits.max_cost_usd {
            return Ok(true);
        }

        // Check time limit
        let elapsed = session.created_at.elapsed();
        if elapsed.num_seconds() as u64 >= self.config.limits.max_duration_secs {
            return Ok(true);
        }

        Ok(false)
    }

    /// Record a completed round.
    async fn record_round(&self, round: ForgeRound) -> ForgeResult<()> {
        let mut session = self.session.write().await;

        // Update token counts
        let round_tokens = round.tokens();
        session.total_tokens.add(&round_tokens);

        // Update cost (simplified - actual would look up model costs)
        let round_cost = (round_tokens.total() as f64 / 1000.0) * 0.01;
        session.total_cost_usd += round_cost;

        // Update status if converged
        if let ForgeRound::Convergence(ref c) = round {
            if c.converged {
                session.status = ForgeSessionStatus::Converged;
            }
        }

        // Record round
        session.rounds.push(round);
        session.current_round = session.rounds.len();
        session.updated_at = tachikoma_common_core::Timestamp::now();

        // Emit cost update
        self.emit(ForgeEvent::CostUpdate {
            total_cost: session.total_cost_usd,
            budget_remaining: self.config.limits.max_cost_usd - session.total_cost_usd,
        });

        Ok(())
    }

    /// Handle a control message.
    async fn handle_control(&mut self, control: OrchestratorControl) -> ForgeResult<ControlResult> {
        match control {
            OrchestratorControl::Pause => {
                *self.is_paused.write().await = true;
                self.emit(ForgeEvent::SessionPaused {
                    reason: "User requested pause".to_string(),
                });
                Ok(ControlResult::Paused)
            }
            OrchestratorControl::Resume => {
                *self.is_paused.write().await = false;
                Ok(ControlResult::Continue)
            }
            OrchestratorControl::Abort => {
                let mut session = self.session.write().await;
                session.status = ForgeSessionStatus::Aborted;
                Ok(ControlResult::Abort)
            }
            OrchestratorControl::SkipRound => {
                // Just continue to next round
                Ok(ControlResult::Continue)
            }
            OrchestratorControl::InjectFeedback(feedback) => {
                // Add feedback as a synthetic critique
                // Implementation would add to session state
                Ok(ControlResult::Continue)
            }
        }
    }

    /// Wait for resume signal.
    async fn wait_for_resume(&mut self) -> ForgeResult<()> {
        while *self.is_paused.read().await {
            if let Some(control) = self.control_rx.recv().await {
                if let OrchestratorControl::Resume = control {
                    *self.is_paused.write().await = false;
                    break;
                }
                if let OrchestratorControl::Abort = control {
                    return Err(ForgeError::Orchestration("Session aborted".to_string()));
                }
            }
        }
        Ok(())
    }

    /// Finalize session and return result.
    async fn finalize_session(&self) -> ForgeResult<ForgeSession> {
        let mut session = self.session.write().await;

        if session.status == ForgeSessionStatus::Converged {
            session.status = ForgeSessionStatus::Complete;
        }

        let final_output = session.latest_draft().unwrap_or_default().to_string();

        self.emit(ForgeEvent::SessionCompleted {
            final_output: final_output.clone(),
        });

        Ok(session.clone())
    }

    /// Emit an event.
    fn emit(&self, event: ForgeEvent) {
        let _ = self.event_tx.send(event);
    }
}

/// Result of handling a control message.
enum ControlResult {
    Continue,
    Paused,
    Abort,
}
```

---

## Testing Requirements

1. Complete draft -> critique -> synthesis cycle works
2. Convergence detection triggers session completion
3. Timeout handling aborts stuck rounds
4. Pause/resume works in attended mode
5. Cost and token tracking is accurate
6. Event emission includes all expected events
7. Error recovery retries appropriate errors

---

## Related Specs

- Depends on: [136-forge-session-types.md](136-forge-session-types.md)
- Depends on: [138-forge-participants.md](138-forge-participants.md)
- Next: [140-round1-draft.md](140-round1-draft.md)
- Used by: [152-forge-attended.md](152-forge-attended.md), [153-forge-cli.md](153-forge-cli.md)
