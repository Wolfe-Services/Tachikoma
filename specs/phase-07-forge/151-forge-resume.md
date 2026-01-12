# 151 - Forge Session Resume

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 151
**Status:** Planned
**Dependencies:** 150-forge-persistence, 139-forge-rounds
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement session resumption functionality that allows interrupted or paused Forge sessions to be continued from their last state with proper context restoration.

---

## Acceptance Criteria

- [x] Resume from any saved state
- [x] Context reconstruction for models
- [x] Round continuity validation
- [x] Participant re-initialization
- [x] Progress preservation
- [x] Resume from specific round

---

## Implementation Details

### 1. Session Resumer (src/resume/resumer.rs)

```rust
//! Session resumption functionality.

use std::sync::Arc;

use crate::{
    ForgeConfig, ForgeError, ForgeOrchestrator, ForgeResult, ForgeRound, ForgeSession,
    ForgeSessionId, ForgeSessionStatus, OrchestratorControl, ParticipantManager,
    SessionStore,
};

/// Options for resuming a session.
#[derive(Debug, Clone)]
pub struct ResumeOptions {
    /// Resume from a specific round (None = continue from last).
    pub from_round: Option<usize>,
    /// Override the session configuration.
    pub config_override: Option<ResumeConfigOverride>,
    /// Skip validation checks.
    pub skip_validation: bool,
    /// Force resume even if status suggests otherwise.
    pub force: bool,
}

impl Default for ResumeOptions {
    fn default() -> Self {
        Self {
            from_round: None,
            config_override: None,
            skip_validation: false,
            force: false,
        }
    }
}

/// Configuration overrides for resume.
#[derive(Debug, Clone)]
pub struct ResumeConfigOverride {
    /// New max rounds limit.
    pub max_rounds: Option<usize>,
    /// New max cost limit.
    pub max_cost_usd: Option<f64>,
    /// New convergence threshold.
    pub convergence_threshold: Option<f64>,
    /// Switch to attended mode.
    pub attended: Option<bool>,
}

/// Result of preparing to resume.
#[derive(Debug)]
pub struct ResumePreparation {
    /// The session to resume.
    pub session: ForgeSession,
    /// Which round to start from.
    pub start_round: usize,
    /// Summary of what will happen.
    pub summary: ResumeSummary,
    /// Any warnings about the resume.
    pub warnings: Vec<String>,
}

/// Summary of resume operation.
#[derive(Debug)]
pub struct ResumeSummary {
    /// Rounds that will be preserved.
    pub preserved_rounds: usize,
    /// Rounds that will be replayed/discarded.
    pub discarded_rounds: usize,
    /// Estimated remaining cost.
    pub estimated_remaining_cost: f64,
    /// Estimated remaining rounds.
    pub estimated_remaining_rounds: usize,
}

/// Resumer for Forge sessions.
pub struct SessionResumer {
    store: Arc<SessionStore>,
    config: ForgeConfig,
}

impl SessionResumer {
    /// Create a new session resumer.
    pub fn new(store: Arc<SessionStore>, config: ForgeConfig) -> Self {
        Self { store, config }
    }

    /// Prepare to resume a session.
    pub async fn prepare(
        &self,
        session_id: &ForgeSessionId,
        options: ResumeOptions,
    ) -> ForgeResult<ResumePreparation> {
        // Load session
        let mut session = self.store.load(session_id).await?;

        // Validate resumability
        if !options.force {
            self.validate_resumable(&session)?;
        }

        // Apply config overrides
        if let Some(ref overrides) = options.config_override {
            self.apply_config_overrides(&mut session, overrides);
        }

        // Determine start round
        let start_round = options.from_round.unwrap_or(session.rounds.len());

        // Truncate rounds if resuming from earlier point
        let discarded_rounds = if start_round < session.rounds.len() {
            let discarded = session.rounds.len() - start_round;
            session.rounds.truncate(start_round);
            session.current_round = start_round;
            discarded
        } else {
            0
        };

        // Generate warnings
        let mut warnings = Vec::new();

        if discarded_rounds > 0 {
            warnings.push(format!(
                "{} rounds will be discarded by resuming from round {}",
                discarded_rounds, start_round
            ));
        }

        if session.total_cost_usd > self.config.limits.max_cost_usd * 0.8 {
            warnings.push(format!(
                "Session has used {:.1}% of cost budget",
                (session.total_cost_usd / self.config.limits.max_cost_usd) * 100.0
            ));
        }

        // Estimate remaining work
        let estimated_remaining_rounds = self.config.convergence.max_rounds
            .saturating_sub(start_round);
        let avg_cost_per_round = if start_round > 0 {
            session.total_cost_usd / start_round as f64
        } else {
            1.0 // Default estimate
        };
        let estimated_remaining_cost = avg_cost_per_round * estimated_remaining_rounds as f64;

        let summary = ResumeSummary {
            preserved_rounds: start_round,
            discarded_rounds,
            estimated_remaining_cost,
            estimated_remaining_rounds,
        };

        Ok(ResumePreparation {
            session,
            start_round,
            summary,
            warnings,
        })
    }

    /// Resume a session and return an orchestrator.
    pub async fn resume(
        &self,
        session_id: &ForgeSessionId,
        options: ResumeOptions,
        participants: Arc<ParticipantManager>,
    ) -> ForgeResult<(
        ForgeOrchestrator,
        tokio::sync::broadcast::Receiver<crate::ForgeEvent>,
        tokio::sync::mpsc::Sender<OrchestratorControl>,
    )> {
        let preparation = self.prepare(session_id, options).await?;

        // Log warnings
        for warning in &preparation.warnings {
            tracing::warn!("Resume warning: {}", warning);
        }

        // Update session status
        let mut session = preparation.session;
        session.status = ForgeSessionStatus::InProgress;
        session.updated_at = tachikoma_common_core::Timestamp::now();

        // Create orchestrator with existing session
        let (orchestrator, event_rx, control_tx) =
            ForgeOrchestrator::from_session(session, participants, self.config.clone());

        Ok((orchestrator, event_rx, control_tx))
    }

    /// Validate that a session can be resumed.
    fn validate_resumable(&self, session: &ForgeSession) -> ForgeResult<()> {
        match session.status {
            ForgeSessionStatus::Initialized
            | ForgeSessionStatus::InProgress
            | ForgeSessionStatus::Paused => Ok(()),

            ForgeSessionStatus::Converged => Err(ForgeError::Orchestration(
                "Session has already converged. Use --force to continue anyway.".to_string()
            )),

            ForgeSessionStatus::Complete => Err(ForgeError::Orchestration(
                "Session is complete. Use --force to reopen.".to_string()
            )),

            ForgeSessionStatus::Aborted => Err(ForgeError::Orchestration(
                "Session was aborted. Use --force to resume.".to_string()
            )),

            ForgeSessionStatus::TimedOut => Err(ForgeError::Orchestration(
                "Session timed out. Use --force to resume with extended limits.".to_string()
            )),
        }
    }

    /// Apply configuration overrides.
    fn apply_config_overrides(&self, session: &mut ForgeSession, overrides: &ResumeConfigOverride) {
        if let Some(max_rounds) = overrides.max_rounds {
            session.config.max_rounds = max_rounds;
        }

        if let Some(max_cost) = overrides.max_cost_usd {
            session.config.max_cost_usd = max_cost;
        }

        if let Some(threshold) = overrides.convergence_threshold {
            session.config.convergence_threshold = threshold;
        }

        if let Some(attended) = overrides.attended {
            session.config.attended = attended;
        }
    }

    /// Get resumable sessions.
    pub async fn list_resumable(&self) -> ForgeResult<Vec<crate::SessionMetadata>> {
        let all = self.store.list().await?;

        Ok(all.into_iter().filter(|s| {
            matches!(
                s.status,
                ForgeSessionStatus::Initialized
                    | ForgeSessionStatus::InProgress
                    | ForgeSessionStatus::Paused
            )
        }).collect())
    }

    /// Build context for model continuation.
    pub fn build_continuation_context(&self, session: &ForgeSession) -> ContinuationContext {
        let last_content = session.latest_draft().map(|s| s.to_string());

        let last_round_summary = session.rounds.last().map(|r| match r {
            ForgeRound::Draft(_) => "Initial draft completed".to_string(),
            ForgeRound::Critique(c) => format!(
                "Critique round with {} critiques, avg score: {:.0}",
                c.critiques.len(),
                c.critiques.iter().map(|c| c.score as f64).sum::<f64>()
                    / c.critiques.len().max(1) as f64
            ),
            ForgeRound::Synthesis(_) => "Synthesis completed".to_string(),
            ForgeRound::Refinement(r) => format!("Refinement on '{}' at depth {}", r.focus_area, r.depth),
            ForgeRound::Convergence(c) => format!(
                "Convergence check: score {:.2}, {}",
                c.score,
                if c.converged { "converged" } else { "not converged" }
            ),
        });

        let pending_issues: Vec<String> = session.rounds.iter().rev()
            .find_map(|r| match r {
                ForgeRound::Convergence(c) => Some(c.remaining_issues.clone()),
                _ => None,
            })
            .unwrap_or_default();

        ContinuationContext {
            topic_title: session.topic.title.clone(),
            topic_description: session.topic.description.clone(),
            rounds_completed: session.rounds.len(),
            last_content,
            last_round_summary,
            pending_issues,
            total_cost_so_far: session.total_cost_usd,
        }
    }
}

/// Context for model continuation.
#[derive(Debug, Clone)]
pub struct ContinuationContext {
    /// Topic title.
    pub topic_title: String,
    /// Topic description.
    pub topic_description: String,
    /// Rounds completed so far.
    pub rounds_completed: usize,
    /// Last draft content.
    pub last_content: Option<String>,
    /// Summary of last round.
    pub last_round_summary: Option<String>,
    /// Pending issues to address.
    pub pending_issues: Vec<String>,
    /// Total cost so far.
    pub total_cost_so_far: f64,
}

impl ContinuationContext {
    /// Format as a prompt prefix.
    pub fn to_prompt_prefix(&self) -> String {
        let mut prefix = format!(
            "## Session Continuation\n\n\
             This is a resumed brainstorming session.\n\n\
             **Topic:** {}\n\
             **Rounds completed:** {}\n",
            self.topic_title,
            self.rounds_completed
        );

        if let Some(ref summary) = self.last_round_summary {
            prefix.push_str(&format!("**Last round:** {}\n", summary));
        }

        if !self.pending_issues.is_empty() {
            prefix.push_str("\n**Pending issues to address:**\n");
            for issue in &self.pending_issues {
                prefix.push_str(&format!("- {}\n", issue));
            }
        }

        prefix.push_str("\n---\n\n");
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuation_context_format() {
        let ctx = ContinuationContext {
            topic_title: "Test Topic".to_string(),
            topic_description: "Description".to_string(),
            rounds_completed: 3,
            last_content: Some("Draft content".to_string()),
            last_round_summary: Some("Critique completed".to_string()),
            pending_issues: vec!["Issue 1".to_string(), "Issue 2".to_string()],
            total_cost_so_far: 1.5,
        };

        let prefix = ctx.to_prompt_prefix();

        assert!(prefix.contains("Test Topic"));
        assert!(prefix.contains("Rounds completed: 3"));
        assert!(prefix.contains("Issue 1"));
    }
}
```

---

## Testing Requirements

1. Sessions load correctly for resume
2. Round truncation works when resuming earlier
3. Config overrides apply correctly
4. Validation catches non-resumable states
5. Continuation context formats properly
6. Warnings generated for edge cases

---

## Related Specs

- Depends on: [150-forge-persistence.md](150-forge-persistence.md)
- Depends on: [139-forge-rounds.md](139-forge-rounds.md)
- Next: [152-forge-attended.md](152-forge-attended.md)
- Used by: [153-forge-cli.md](153-forge-cli.md)
