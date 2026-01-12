# 148 - Decision Logging

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 148
**Status:** Planned
**Dependencies:** 144-round3-conflict, 146-convergence-detect
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement a comprehensive decision logging system that tracks all significant decisions made during Forge sessions, including conflict resolutions, synthesis choices, and convergence determinations.

---

## Acceptance Criteria

- [x] Decision log data structure
- [x] Automatic logging during orchestration
- [x] Decision categorization
- [x] Rationale capture
- [x] Decision timeline
- [x] Export to human-readable format

---

## Implementation Details

### 1. Decision Log Types (src/logging/decision_log.rs)

```rust
//! Decision logging for Forge sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ConflictResolution, ForgeSessionId, Participant, ResolutionStrategy};

/// A logged decision in a Forge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Unique decision ID.
    pub id: String,
    /// When the decision was made.
    pub timestamp: DateTime<Utc>,
    /// Which round this occurred in.
    pub round_number: usize,
    /// Category of decision.
    pub category: DecisionCategory,
    /// What the decision was about.
    pub subject: String,
    /// The decision made.
    pub decision: String,
    /// Rationale for the decision.
    pub rationale: String,
    /// Who/what made the decision.
    pub decision_maker: DecisionMaker,
    /// Alternatives considered.
    pub alternatives: Vec<Alternative>,
    /// Impact assessment.
    pub impact: DecisionImpact,
    /// Related decisions.
    pub related_decisions: Vec<String>,
    /// Additional context.
    pub context: HashMap<String, String>,
}

/// Category of decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionCategory {
    /// Conflict resolution between critics.
    ConflictResolution,
    /// Content inclusion/exclusion.
    ContentSelection,
    /// Structural organization.
    Structure,
    /// Convergence determination.
    Convergence,
    /// Refinement focus.
    RefinementFocus,
    /// Session control (pause, abort, etc.).
    SessionControl,
    /// Other.
    Other,
}

/// Who made the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionMaker {
    /// A model participant.
    Model(Participant),
    /// Automated by the system.
    Automated(String),
    /// Human operator.
    Human(String),
    /// Consensus of multiple models.
    Consensus(Vec<Participant>),
}

/// An alternative that was considered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    /// The alternative option.
    pub option: String,
    /// Why it was rejected.
    pub rejection_reason: String,
    /// Who proposed it.
    pub proposed_by: Option<String>,
}

/// Impact assessment of a decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionImpact {
    /// Severity (1-5).
    pub severity: u8,
    /// Scope of impact.
    pub scope: ImpactScope,
    /// Reversibility.
    pub reversible: bool,
    /// Affected areas.
    pub affected_areas: Vec<String>,
}

/// Scope of decision impact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactScope {
    /// Affects a single section.
    Section,
    /// Affects multiple sections.
    MultiSection,
    /// Affects the whole document.
    Document,
    /// Affects session flow.
    Session,
}

/// The decision log for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLog {
    /// Session this log belongs to.
    pub session_id: ForgeSessionId,
    /// All decisions.
    pub decisions: Vec<Decision>,
    /// Summary statistics.
    pub summary: DecisionSummary,
}

/// Summary of decisions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionSummary {
    /// Total decisions.
    pub total_count: usize,
    /// Count by category.
    pub by_category: HashMap<String, usize>,
    /// Count by decision maker type.
    pub by_maker: HashMap<String, usize>,
    /// Average impact severity.
    pub average_severity: f64,
}

impl DecisionLog {
    /// Create a new decision log.
    pub fn new(session_id: ForgeSessionId) -> Self {
        Self {
            session_id,
            decisions: Vec::new(),
            summary: DecisionSummary::default(),
        }
    }

    /// Add a decision.
    pub fn add(&mut self, decision: Decision) {
        self.summary.total_count += 1;

        *self.summary.by_category
            .entry(format!("{:?}", decision.category))
            .or_insert(0) += 1;

        let maker_type = match &decision.decision_maker {
            DecisionMaker::Model(_) => "model",
            DecisionMaker::Automated(_) => "automated",
            DecisionMaker::Human(_) => "human",
            DecisionMaker::Consensus(_) => "consensus",
        };
        *self.summary.by_maker.entry(maker_type.to_string()).or_insert(0) += 1;

        // Update average severity
        let total_severity: u32 = self.decisions.iter()
            .map(|d| d.impact.severity as u32)
            .sum::<u32>() + decision.impact.severity as u32;
        self.summary.average_severity = total_severity as f64 / (self.decisions.len() + 1) as f64;

        self.decisions.push(decision);
    }

    /// Get decisions by category.
    pub fn by_category(&self, category: DecisionCategory) -> Vec<&Decision> {
        self.decisions.iter()
            .filter(|d| d.category == category)
            .collect()
    }

    /// Get decisions for a round.
    pub fn by_round(&self, round: usize) -> Vec<&Decision> {
        self.decisions.iter()
            .filter(|d| d.round_number == round)
            .collect()
    }

    /// Get high-impact decisions.
    pub fn high_impact(&self) -> Vec<&Decision> {
        self.decisions.iter()
            .filter(|d| d.impact.severity >= 4)
            .collect()
    }

    /// Export to markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Decision Log - Session {}\n\n", self.session_id));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- Total decisions: {}\n", self.summary.total_count));
        md.push_str(&format!("- Average severity: {:.1}\n", self.summary.average_severity));
        md.push_str("\n### By Category\n");
        for (cat, count) in &self.summary.by_category {
            md.push_str(&format!("- {}: {}\n", cat, count));
        }
        md.push_str("\n");

        // Decisions by round
        md.push_str("## Decisions by Round\n\n");

        let max_round = self.decisions.iter().map(|d| d.round_number).max().unwrap_or(0);

        for round in 0..=max_round {
            let round_decisions = self.by_round(round);
            if !round_decisions.is_empty() {
                md.push_str(&format!("### Round {}\n\n", round));

                for decision in round_decisions {
                    md.push_str(&format!("#### {} ({})\n\n", decision.subject, decision.id));
                    md.push_str(&format!("**Category:** {:?}\n\n", decision.category));
                    md.push_str(&format!("**Decision:** {}\n\n", decision.decision));
                    md.push_str(&format!("**Rationale:** {}\n\n", decision.rationale));
                    md.push_str(&format!("**Impact:** Severity {}, {:?}, {}\n\n",
                        decision.impact.severity,
                        decision.impact.scope,
                        if decision.impact.reversible { "reversible" } else { "irreversible" }
                    ));

                    if !decision.alternatives.is_empty() {
                        md.push_str("**Alternatives Considered:**\n");
                        for alt in &decision.alternatives {
                            md.push_str(&format!("- {}: {}\n", alt.option, alt.rejection_reason));
                        }
                        md.push_str("\n");
                    }

                    md.push_str("---\n\n");
                }
            }
        }

        md
    }
}
```

### 2. Decision Logger (src/logging/logger.rs)

```rust
//! Decision logger for capturing decisions during orchestration.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    Alternative, ConflictResolution, Decision, DecisionCategory, DecisionImpact,
    DecisionLog, DecisionMaker, ForgeSessionId, ImpactScope, Participant,
    ResolutionStrategy,
};

/// Logger for capturing decisions.
pub struct DecisionLogger {
    log: Arc<RwLock<DecisionLog>>,
    current_round: Arc<RwLock<usize>>,
}

impl DecisionLogger {
    /// Create a new decision logger.
    pub fn new(session_id: ForgeSessionId) -> Self {
        Self {
            log: Arc::new(RwLock::new(DecisionLog::new(session_id))),
            current_round: Arc::new(RwLock::new(0)),
        }
    }

    /// Set the current round number.
    pub async fn set_round(&self, round: usize) {
        *self.current_round.write().await = round;
    }

    /// Log a conflict resolution decision.
    pub async fn log_conflict_resolution(
        &self,
        resolution: &ConflictResolution,
        strategy: ResolutionStrategy,
        decision_maker: DecisionMaker,
    ) {
        let round = *self.current_round.read().await;

        let alternatives: Vec<Alternative> = resolution.positions.iter()
            .filter(|p| p.position != resolution.resolution)
            .map(|p| Alternative {
                option: p.position.clone(),
                rejection_reason: "Not selected by resolution strategy".to_string(),
                proposed_by: Some(p.participant.display_name.clone()),
            })
            .collect();

        let decision = Decision {
            id: format!("dec_{}", Uuid::new_v4().to_string()[..8].to_string()),
            timestamp: chrono::Utc::now(),
            round_number: round,
            category: DecisionCategory::ConflictResolution,
            subject: resolution.issue.clone(),
            decision: resolution.resolution.clone(),
            rationale: resolution.rationale.clone(),
            decision_maker,
            alternatives,
            impact: DecisionImpact {
                severity: 3,
                scope: ImpactScope::Section,
                reversible: true,
                affected_areas: vec![resolution.issue.clone()],
            },
            related_decisions: vec![],
            context: [
                ("strategy".to_string(), format!("{:?}", strategy)),
            ].into_iter().collect(),
        };

        self.log.write().await.add(decision);
    }

    /// Log a content selection decision.
    pub async fn log_content_selection(
        &self,
        subject: &str,
        selected: &str,
        rationale: &str,
        alternatives: Vec<(&str, &str)>,
        participant: &Participant,
    ) {
        let round = *self.current_round.read().await;

        let decision = Decision {
            id: format!("dec_{}", Uuid::new_v4().to_string()[..8].to_string()),
            timestamp: chrono::Utc::now(),
            round_number: round,
            category: DecisionCategory::ContentSelection,
            subject: subject.to_string(),
            decision: selected.to_string(),
            rationale: rationale.to_string(),
            decision_maker: DecisionMaker::Model(participant.clone()),
            alternatives: alternatives.iter().map(|(opt, reason)| Alternative {
                option: opt.to_string(),
                rejection_reason: reason.to_string(),
                proposed_by: None,
            }).collect(),
            impact: DecisionImpact {
                severity: 2,
                scope: ImpactScope::Section,
                reversible: true,
                affected_areas: vec![subject.to_string()],
            },
            related_decisions: vec![],
            context: std::collections::HashMap::new(),
        };

        self.log.write().await.add(decision);
    }

    /// Log a convergence decision.
    pub async fn log_convergence_decision(
        &self,
        converged: bool,
        score: f64,
        rationale: &str,
        remaining_issues: &[String],
    ) {
        let round = *self.current_round.read().await;

        let decision = Decision {
            id: format!("dec_{}", Uuid::new_v4().to_string()[..8].to_string()),
            timestamp: chrono::Utc::now(),
            round_number: round,
            category: DecisionCategory::Convergence,
            subject: "Session Convergence".to_string(),
            decision: if converged {
                "Session has converged".to_string()
            } else {
                "Continue to next round".to_string()
            },
            rationale: rationale.to_string(),
            decision_maker: DecisionMaker::Automated("Convergence Detector".to_string()),
            alternatives: if converged {
                vec![]
            } else {
                vec![Alternative {
                    option: "Force convergence".to_string(),
                    rejection_reason: format!("Score {:.2} below threshold", score),
                    proposed_by: None,
                }]
            },
            impact: DecisionImpact {
                severity: if converged { 5 } else { 1 },
                scope: ImpactScope::Session,
                reversible: !converged,
                affected_areas: vec!["session_flow".to_string()],
            },
            related_decisions: vec![],
            context: [
                ("score".to_string(), format!("{:.3}", score)),
                ("remaining_issues".to_string(), remaining_issues.len().to_string()),
            ].into_iter().collect(),
        };

        self.log.write().await.add(decision);
    }

    /// Log a refinement focus decision.
    pub async fn log_refinement_focus(
        &self,
        focus_area: &str,
        rationale: &str,
        other_candidates: &[&str],
    ) {
        let round = *self.current_round.read().await;

        let decision = Decision {
            id: format!("dec_{}", Uuid::new_v4().to_string()[..8].to_string()),
            timestamp: chrono::Utc::now(),
            round_number: round,
            category: DecisionCategory::RefinementFocus,
            subject: "Refinement Focus Area".to_string(),
            decision: format!("Focus on: {}", focus_area),
            rationale: rationale.to_string(),
            decision_maker: DecisionMaker::Automated("Refinement Engine".to_string()),
            alternatives: other_candidates.iter().map(|c| Alternative {
                option: c.to_string(),
                rejection_reason: "Lower priority".to_string(),
                proposed_by: None,
            }).collect(),
            impact: DecisionImpact {
                severity: 2,
                scope: ImpactScope::Section,
                reversible: true,
                affected_areas: vec![focus_area.to_string()],
            },
            related_decisions: vec![],
            context: std::collections::HashMap::new(),
        };

        self.log.write().await.add(decision);
    }

    /// Log a session control decision.
    pub async fn log_session_control(
        &self,
        action: &str,
        reason: &str,
        by: &str,
    ) {
        let round = *self.current_round.read().await;

        let decision = Decision {
            id: format!("dec_{}", Uuid::new_v4().to_string()[..8].to_string()),
            timestamp: chrono::Utc::now(),
            round_number: round,
            category: DecisionCategory::SessionControl,
            subject: "Session Control".to_string(),
            decision: action.to_string(),
            rationale: reason.to_string(),
            decision_maker: DecisionMaker::Human(by.to_string()),
            alternatives: vec![],
            impact: DecisionImpact {
                severity: 4,
                scope: ImpactScope::Session,
                reversible: action == "pause",
                affected_areas: vec!["session_flow".to_string()],
            },
            related_decisions: vec![],
            context: std::collections::HashMap::new(),
        };

        self.log.write().await.add(decision);
    }

    /// Get the decision log.
    pub async fn get_log(&self) -> DecisionLog {
        self.log.read().await.clone()
    }

    /// Export to markdown.
    pub async fn export_markdown(&self) -> String {
        self.log.read().await.to_markdown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ForgeSessionId;

    #[tokio::test]
    async fn test_log_conflict_resolution() {
        let session_id = ForgeSessionId::new();
        let logger = DecisionLogger::new(session_id);

        let resolution = ConflictResolution {
            issue: "Error handling approach".to_string(),
            positions: vec![],
            resolution: "Use Result types".to_string(),
            rationale: "More idiomatic Rust".to_string(),
        };

        logger.log_conflict_resolution(
            &resolution,
            ResolutionStrategy::Compromise,
            DecisionMaker::Automated("Synthesizer".to_string()),
        ).await;

        let log = logger.get_log().await;
        assert_eq!(log.decisions.len(), 1);
        assert_eq!(log.decisions[0].category, DecisionCategory::ConflictResolution);
    }
}
```

---

## Testing Requirements

1. Decisions are logged with correct categories
2. Summary statistics update correctly
3. Markdown export is properly formatted
4. Decisions can be filtered by round/category
5. Impact assessment is captured
6. Alternatives are recorded

---

## Related Specs

- Depends on: [144-round3-conflict.md](144-round3-conflict.md)
- Depends on: [146-convergence-detect.md](146-convergence-detect.md)
- Next: [149-dissent-logging.md](149-dissent-logging.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md), [154-forge-output.md](154-forge-output.md)
