# 149 - Dissent Logging

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 149
**Status:** Planned
**Dependencies:** 148-decision-logging, 142-round2-critique-collect
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement a dissent logging system that preserves minority opinions, rejected suggestions, and unresolved concerns from the brainstorming process for transparency and future reference.

---

## Acceptance Criteria

- [x] Dissent record data structure
- [x] Automatic capture of minority opinions
- [x] Rejected suggestion tracking
- [x] Unresolved concern preservation
- [x] Dissent severity classification
- [x] Export with rationale

---

## Implementation Details

### 1. Dissent Types (src/logging/dissent_log.rs)

```rust
//! Dissent logging for minority opinions and rejected suggestions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ForgeSessionId, Participant, SuggestionCategory};

/// A recorded dissent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dissent {
    /// Unique dissent ID.
    pub id: String,
    /// When the dissent was recorded.
    pub timestamp: DateTime<Utc>,
    /// Which round this occurred in.
    pub round_number: usize,
    /// Type of dissent.
    pub dissent_type: DissentType,
    /// Who raised the dissent.
    pub raised_by: Participant,
    /// The dissenting opinion/suggestion.
    pub content: String,
    /// Why it was not adopted.
    pub rejection_reason: Option<String>,
    /// Severity/importance (1-5).
    pub severity: u8,
    /// Category if applicable.
    pub category: Option<SuggestionCategory>,
    /// Related topic/section.
    pub related_area: Option<String>,
    /// Has this been acknowledged.
    pub acknowledged: bool,
    /// Any follow-up notes.
    pub notes: Vec<DissentNote>,
}

/// Type of dissent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DissentType {
    /// Minority opinion on quality/approach.
    MinorityOpinion,
    /// Rejected suggestion.
    RejectedSuggestion,
    /// Unresolved concern.
    UnresolvedConcern,
    /// Overruled critique.
    OverruledCritique,
    /// Disagreement with convergence.
    ConvergenceDisagreement,
}

/// A note added to a dissent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DissentNote {
    /// When the note was added.
    pub timestamp: DateTime<Utc>,
    /// Who added the note.
    pub author: String,
    /// The note content.
    pub content: String,
}

/// The dissent log for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DissentLog {
    /// Session this log belongs to.
    pub session_id: ForgeSessionId,
    /// All dissents.
    pub dissents: Vec<Dissent>,
    /// Summary statistics.
    pub summary: DissentSummary,
}

/// Summary of dissents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DissentSummary {
    /// Total dissents.
    pub total_count: usize,
    /// Count by type.
    pub by_type: HashMap<String, usize>,
    /// Count by participant.
    pub by_participant: HashMap<String, usize>,
    /// High severity count.
    pub high_severity_count: usize,
    /// Unacknowledged count.
    pub unacknowledged_count: usize,
}

impl DissentLog {
    /// Create a new dissent log.
    pub fn new(session_id: ForgeSessionId) -> Self {
        Self {
            session_id,
            dissents: Vec::new(),
            summary: DissentSummary::default(),
        }
    }

    /// Add a dissent.
    pub fn add(&mut self, dissent: Dissent) {
        self.summary.total_count += 1;

        *self.summary.by_type
            .entry(format!("{:?}", dissent.dissent_type))
            .or_insert(0) += 1;

        *self.summary.by_participant
            .entry(dissent.raised_by.display_name.clone())
            .or_insert(0) += 1;

        if dissent.severity >= 4 {
            self.summary.high_severity_count += 1;
        }

        if !dissent.acknowledged {
            self.summary.unacknowledged_count += 1;
        }

        self.dissents.push(dissent);
    }

    /// Mark a dissent as acknowledged.
    pub fn acknowledge(&mut self, dissent_id: &str, note: Option<DissentNote>) {
        if let Some(dissent) = self.dissents.iter_mut().find(|d| d.id == dissent_id) {
            if !dissent.acknowledged {
                dissent.acknowledged = true;
                self.summary.unacknowledged_count =
                    self.summary.unacknowledged_count.saturating_sub(1);
            }
            if let Some(n) = note {
                dissent.notes.push(n);
            }
        }
    }

    /// Get dissents by type.
    pub fn by_type(&self, dissent_type: DissentType) -> Vec<&Dissent> {
        self.dissents.iter()
            .filter(|d| d.dissent_type == dissent_type)
            .collect()
    }

    /// Get dissents by participant.
    pub fn by_participant(&self, participant: &str) -> Vec<&Dissent> {
        self.dissents.iter()
            .filter(|d| d.raised_by.display_name == participant)
            .collect()
    }

    /// Get high severity dissents.
    pub fn high_severity(&self) -> Vec<&Dissent> {
        self.dissents.iter()
            .filter(|d| d.severity >= 4)
            .collect()
    }

    /// Get unacknowledged dissents.
    pub fn unacknowledged(&self) -> Vec<&Dissent> {
        self.dissents.iter()
            .filter(|d| !d.acknowledged)
            .collect()
    }

    /// Export to markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Dissent Log - Session {}\n\n", self.session_id));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- Total dissents: {}\n", self.summary.total_count));
        md.push_str(&format!("- High severity: {}\n", self.summary.high_severity_count));
        md.push_str(&format!("- Unacknowledged: {}\n", self.summary.unacknowledged_count));

        md.push_str("\n### By Type\n");
        for (dtype, count) in &self.summary.by_type {
            md.push_str(&format!("- {}: {}\n", dtype, count));
        }

        md.push_str("\n### By Participant\n");
        for (participant, count) in &self.summary.by_participant {
            md.push_str(&format!("- {}: {}\n", participant, count));
        }

        // High severity dissents
        let high_sev = self.high_severity();
        if !high_sev.is_empty() {
            md.push_str("\n## High Severity Dissents\n\n");
            md.push_str("These dissents were marked as high importance and warrant attention.\n\n");

            for dissent in high_sev {
                md.push_str(&self.format_dissent(dissent));
            }
        }

        // Unacknowledged
        let unack = self.unacknowledged();
        if !unack.is_empty() {
            md.push_str("\n## Unacknowledged Dissents\n\n");

            for dissent in unack {
                md.push_str(&self.format_dissent(dissent));
            }
        }

        // All dissents by round
        md.push_str("\n## All Dissents by Round\n\n");

        let max_round = self.dissents.iter().map(|d| d.round_number).max().unwrap_or(0);

        for round in 0..=max_round {
            let round_dissents: Vec<_> = self.dissents.iter()
                .filter(|d| d.round_number == round)
                .collect();

            if !round_dissents.is_empty() {
                md.push_str(&format!("### Round {}\n\n", round));

                for dissent in round_dissents {
                    md.push_str(&self.format_dissent(dissent));
                }
            }
        }

        md
    }

    /// Format a single dissent for markdown.
    fn format_dissent(&self, dissent: &Dissent) -> String {
        let status = if dissent.acknowledged { "Acknowledged" } else { "Pending" };
        let severity_label = match dissent.severity {
            1 => "Minor",
            2 => "Low",
            3 => "Medium",
            4 => "High",
            5 => "Critical",
            _ => "Unknown",
        };

        let mut s = format!(
            "#### {} ({}) - {}\n\n",
            dissent.id,
            status,
            severity_label
        );

        s.push_str(&format!("**Type:** {:?}\n\n", dissent.dissent_type));
        s.push_str(&format!("**Raised by:** {}\n\n", dissent.raised_by.display_name));
        s.push_str(&format!("**Content:**\n> {}\n\n", dissent.content));

        if let Some(ref reason) = dissent.rejection_reason {
            s.push_str(&format!("**Rejection reason:** {}\n\n", reason));
        }

        if let Some(ref area) = dissent.related_area {
            s.push_str(&format!("**Related area:** {}\n\n", area));
        }

        if !dissent.notes.is_empty() {
            s.push_str("**Notes:**\n");
            for note in &dissent.notes {
                s.push_str(&format!("- [{}] {}: {}\n",
                    note.timestamp.format("%Y-%m-%d %H:%M"),
                    note.author,
                    note.content
                ));
            }
            s.push('\n');
        }

        s.push_str("---\n\n");
        s
    }
}
```

### 2. Dissent Collector (src/logging/dissent_collector.rs)

```rust
//! Automatic dissent collection during orchestration.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    ConflictResolution, ConvergenceVote, Critique, Dissent, DissentLog, DissentNote,
    DissentType, ForgeSessionId, Participant, Suggestion,
};

/// Collector for automatic dissent capture.
pub struct DissentCollector {
    log: Arc<RwLock<DissentLog>>,
    current_round: Arc<RwLock<usize>>,
}

impl DissentCollector {
    /// Create a new dissent collector.
    pub fn new(session_id: ForgeSessionId) -> Self {
        Self {
            log: Arc::new(RwLock::new(DissentLog::new(session_id))),
            current_round: Arc::new(RwLock::new(0)),
        }
    }

    /// Set the current round number.
    pub async fn set_round(&self, round: usize) {
        *self.current_round.write().await = round;
    }

    /// Collect dissents from critiques after synthesis.
    pub async fn collect_from_synthesis(
        &self,
        critiques: &[Critique],
        adopted_suggestions: &[String],
        final_content: &str,
    ) {
        let round = *self.current_round.read().await;

        for critique in critiques {
            // Check for rejected high-priority suggestions
            for suggestion in &critique.suggestions {
                let adopted = adopted_suggestions.iter()
                    .any(|s| similarity(&suggestion.text, s) > 0.7);

                if !adopted && suggestion.priority <= 2 {
                    let dissent = Dissent {
                        id: format!("dis_{}", &Uuid::new_v4().to_string()[..8]),
                        timestamp: chrono::Utc::now(),
                        round_number: round,
                        dissent_type: DissentType::RejectedSuggestion,
                        raised_by: critique.critic.clone(),
                        content: suggestion.text.clone(),
                        rejection_reason: Some("Not incorporated in synthesis".to_string()),
                        severity: 5 - suggestion.priority, // Invert: priority 1 -> severity 4
                        category: Some(suggestion.category),
                        related_area: suggestion.section.clone(),
                        acknowledged: false,
                        notes: vec![],
                    };

                    self.log.write().await.add(dissent);
                }
            }

            // Check for unaddressed weaknesses
            for weakness in &critique.weaknesses {
                let addressed = final_content.to_lowercase()
                    .contains(&extract_key_phrase(weakness).to_lowercase());

                if !addressed {
                    let dissent = Dissent {
                        id: format!("dis_{}", &Uuid::new_v4().to_string()[..8]),
                        timestamp: chrono::Utc::now(),
                        round_number: round,
                        dissent_type: DissentType::UnresolvedConcern,
                        raised_by: critique.critic.clone(),
                        content: weakness.clone(),
                        rejection_reason: None,
                        severity: 2,
                        category: None,
                        related_area: None,
                        acknowledged: false,
                        notes: vec![],
                    };

                    self.log.write().await.add(dissent);
                }
            }
        }
    }

    /// Collect dissents from conflict resolution.
    pub async fn collect_from_conflict_resolution(
        &self,
        resolution: &ConflictResolution,
    ) {
        let round = *self.current_round.read().await;

        // Find positions that were not chosen
        for position in &resolution.positions {
            if position.position != resolution.resolution {
                let dissent = Dissent {
                    id: format!("dis_{}", &Uuid::new_v4().to_string()[..8]),
                    timestamp: chrono::Utc::now(),
                    round_number: round,
                    dissent_type: DissentType::OverruledCritique,
                    raised_by: position.participant.clone(),
                    content: position.position.clone(),
                    rejection_reason: Some(resolution.rationale.clone()),
                    severity: 3,
                    category: None,
                    related_area: Some(resolution.issue.clone()),
                    acknowledged: false,
                    notes: vec![],
                };

                self.log.write().await.add(dissent);
            }
        }
    }

    /// Collect dissents from convergence votes.
    pub async fn collect_from_convergence(
        &self,
        votes: &[ConvergenceVote],
        converged: bool,
    ) {
        let round = *self.current_round.read().await;

        // If converged, dissenting voters become dissents
        if converged {
            for vote in votes {
                if !vote.agrees {
                    let concerns = if vote.concerns.is_empty() {
                        "Unspecified concerns".to_string()
                    } else {
                        vote.concerns.join("; ")
                    };

                    let dissent = Dissent {
                        id: format!("dis_{}", &Uuid::new_v4().to_string()[..8]),
                        timestamp: chrono::Utc::now(),
                        round_number: round,
                        dissent_type: DissentType::ConvergenceDisagreement,
                        raised_by: vote.participant.clone(),
                        content: format!(
                            "Disagreed with convergence (score: {}). Concerns: {}",
                            vote.score,
                            concerns
                        ),
                        rejection_reason: Some("Session converged by majority".to_string()),
                        severity: 4, // High severity for convergence disagreement
                        category: None,
                        related_area: None,
                        acknowledged: false,
                        notes: vec![],
                    };

                    self.log.write().await.add(dissent);
                }
            }
        }
    }

    /// Collect minority opinions when scores diverge significantly.
    pub async fn collect_minority_opinions(
        &self,
        critiques: &[Critique],
    ) {
        let round = *self.current_round.read().await;

        if critiques.len() < 2 {
            return;
        }

        // Calculate average score
        let avg_score: f64 = critiques.iter()
            .map(|c| c.score as f64)
            .sum::<f64>() / critiques.len() as f64;

        // Find outliers (> 20 points from average)
        for critique in critiques {
            let deviation = (critique.score as f64 - avg_score).abs();

            if deviation > 20.0 {
                let is_higher = critique.score as f64 > avg_score;

                let dissent = Dissent {
                    id: format!("dis_{}", &Uuid::new_v4().to_string()[..8]),
                    timestamp: chrono::Utc::now(),
                    round_number: round,
                    dissent_type: DissentType::MinorityOpinion,
                    raised_by: critique.critic.clone(),
                    content: format!(
                        "Scored {} (avg: {:.0}) - {} than peers. Key points: {}",
                        critique.score,
                        avg_score,
                        if is_higher { "significantly higher" } else { "significantly lower" },
                        if is_higher {
                            critique.strengths.join(", ")
                        } else {
                            critique.weaknesses.join(", ")
                        }
                    ),
                    rejection_reason: None,
                    severity: 2,
                    category: None,
                    related_area: None,
                    acknowledged: false,
                    notes: vec![],
                };

                self.log.write().await.add(dissent);
            }
        }
    }

    /// Add a manual dissent.
    pub async fn add_manual_dissent(
        &self,
        dissent_type: DissentType,
        content: &str,
        raised_by: Participant,
        severity: u8,
    ) {
        let round = *self.current_round.read().await;

        let dissent = Dissent {
            id: format!("dis_{}", &Uuid::new_v4().to_string()[..8]),
            timestamp: chrono::Utc::now(),
            round_number: round,
            dissent_type,
            raised_by,
            content: content.to_string(),
            rejection_reason: None,
            severity,
            category: None,
            related_area: None,
            acknowledged: false,
            notes: vec![],
        };

        self.log.write().await.add(dissent);
    }

    /// Acknowledge a dissent.
    pub async fn acknowledge(&self, dissent_id: &str, note: &str, author: &str) {
        let note = DissentNote {
            timestamp: chrono::Utc::now(),
            author: author.to_string(),
            content: note.to_string(),
        };

        self.log.write().await.acknowledge(dissent_id, Some(note));
    }

    /// Get the dissent log.
    pub async fn get_log(&self) -> DissentLog {
        self.log.read().await.clone()
    }

    /// Export to markdown.
    pub async fn export_markdown(&self) -> String {
        self.log.read().await.to_markdown()
    }
}

/// Calculate text similarity.
fn similarity(a: &str, b: &str) -> f64 {
    let a_words: std::collections::HashSet<_> = a.to_lowercase().split_whitespace().collect();
    let b_words: std::collections::HashSet<_> = b.to_lowercase().split_whitespace().collect();

    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();

    if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
}

/// Extract key phrase from text.
fn extract_key_phrase(text: &str) -> String {
    text.split_whitespace()
        .take(5)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        assert!(similarity("add error handling", "error handling needed") > 0.3);
        assert!(similarity("completely different", "nothing alike") < 0.3);
    }
}
```

---

## Testing Requirements

1. Rejected suggestions are captured correctly
2. Minority opinions identified from score deviation
3. Conflict resolution dissents recorded
4. Convergence disagreements tracked
5. Acknowledgment updates status correctly
6. Markdown export includes all dissents

---

## Related Specs

- Depends on: [148-decision-logging.md](148-decision-logging.md)
- Depends on: [142-round2-critique-collect.md](142-round2-critique-collect.md)
- Next: [150-forge-persistence.md](150-forge-persistence.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md), [154-forge-output.md](154-forge-output.md)
