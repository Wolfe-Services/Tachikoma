# 146 - Convergence Detection

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 146
**Status:** Planned
**Dependencies:** 145-recursive-refine, 147-convergence-metrics
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement convergence detection that determines when participating models have reached sufficient agreement on the output, enabling automatic session termination or triggering additional refinement rounds.

---

## Acceptance Criteria

- [x] Multi-metric convergence scoring
- [x] Participant voting mechanism
- [x] Threshold-based termination
- [x] Early convergence detection
- [x] Stall detection (no improvement)
- [x] Configurable convergence criteria

---

## Implementation Details

### 1. Convergence Detector (src/convergence/detector.rs)

```rust
//! Convergence detection for Forge sessions.

use std::collections::HashMap;

use crate::{
    ConvergenceConfig, ConvergenceMetric, ConvergenceRound, ConvergenceVote,
    Critique, ForgeConfig, ForgeResult, ForgeRound, ForgeSession, ModelRequest,
    Participant, ParticipantManager, TokenCount,
};

/// Detector for convergence conditions.
pub struct ConvergenceDetector<'a> {
    participants: &'a ParticipantManager,
    config: &'a ForgeConfig,
    /// History of convergence scores.
    score_history: Vec<ConvergenceScore>,
}

/// A convergence score with breakdown.
#[derive(Debug, Clone)]
pub struct ConvergenceScore {
    /// Overall score (0.0-1.0).
    pub overall: f64,
    /// Per-metric scores.
    pub metrics: HashMap<ConvergenceMetric, f64>,
    /// Number of agreeing participants.
    pub agreeing_count: usize,
    /// Total participants.
    pub total_participants: usize,
    /// Round number.
    pub round_number: usize,
}

impl<'a> ConvergenceDetector<'a> {
    /// Create a new convergence detector.
    pub fn new(participants: &'a ParticipantManager, config: &'a ForgeConfig) -> Self {
        Self {
            participants,
            config,
            score_history: Vec::new(),
        }
    }

    /// Check convergence for the current session state.
    pub async fn check_convergence(
        &mut self,
        session: &ForgeSession,
    ) -> ForgeResult<ConvergenceRound> {
        let round_number = session.rounds.len();

        // Skip if we haven't done minimum rounds
        if round_number < self.config.convergence.min_rounds {
            return Ok(self.create_not_converged_round(
                round_number,
                0.0,
                vec![],
                vec!["Minimum rounds not yet reached".to_string()],
            ));
        }

        // Calculate metric scores
        let metrics = self.calculate_metrics(session)?;

        // Get participant votes
        let participants = self.participants.active_participants().await;
        let votes = self.collect_votes(session, &participants).await?;

        // Calculate overall score
        let overall_score = self.calculate_overall_score(&metrics, &votes);

        // Determine if converged
        let agreeing = votes.iter().filter(|v| v.agrees).count();
        let converged = self.is_converged(overall_score, agreeing, &participants);

        // Collect remaining issues
        let remaining_issues: Vec<String> = votes
            .iter()
            .filter(|v| !v.agrees)
            .flat_map(|v| v.concerns.clone())
            .collect();

        // Record score
        self.score_history.push(ConvergenceScore {
            overall: overall_score,
            metrics: metrics.clone(),
            agreeing_count: agreeing,
            total_participants: participants.len(),
            round_number,
        });

        // Calculate tokens used
        let tokens = TokenCount {
            input: votes.iter().map(|v| 100).sum(), // Estimated
            output: votes.len() as u64 * 50,
        };

        Ok(ConvergenceRound {
            round_number,
            score: overall_score,
            converged,
            votes,
            remaining_issues,
            timestamp: tachikoma_common_core::Timestamp::now(),
            tokens,
        })
    }

    /// Calculate metric scores.
    fn calculate_metrics(
        &self,
        session: &ForgeSession,
    ) -> ForgeResult<HashMap<ConvergenceMetric, f64>> {
        let mut metrics = HashMap::new();

        for metric in &self.config.convergence.metrics {
            let score = match metric {
                ConvergenceMetric::AgreementScore => {
                    self.calculate_agreement_score(session)
                }
                ConvergenceMetric::ChangeVelocity => {
                    self.calculate_change_velocity(session)
                }
                ConvergenceMetric::IssueCount => {
                    self.calculate_issue_score(session)
                }
                ConvergenceMetric::SemanticSimilarity => {
                    self.calculate_semantic_similarity(session)
                }
                ConvergenceMetric::SectionStability => {
                    self.calculate_section_stability(session)
                }
            };

            metrics.insert(*metric, score);
        }

        Ok(metrics)
    }

    /// Calculate agreement score from recent critiques.
    fn calculate_agreement_score(&self, session: &ForgeSession) -> f64 {
        // Get most recent critique round
        let recent_critiques = session.rounds.iter().rev()
            .find_map(|r| match r {
                ForgeRound::Critique(c) => Some(&c.critiques),
                _ => None,
            });

        if let Some(critiques) = recent_critiques {
            if critiques.is_empty() {
                return 0.0;
            }

            // Average score normalized to 0-1
            let avg_score: f64 = critiques.iter()
                .map(|c| c.score as f64)
                .sum::<f64>() / critiques.len() as f64;

            // Also consider score variance
            let variance: f64 = critiques.iter()
                .map(|c| (c.score as f64 - avg_score).powi(2))
                .sum::<f64>() / critiques.len() as f64;

            // Higher score and lower variance = better agreement
            let score_component = avg_score / 100.0;
            let variance_component = 1.0 - (variance.sqrt() / 50.0).min(1.0);

            (score_component + variance_component) / 2.0
        } else {
            0.0
        }
    }

    /// Calculate change velocity (rate of changes between rounds).
    fn calculate_change_velocity(&self, session: &ForgeSession) -> f64 {
        if session.rounds.len() < 2 {
            return 0.0;
        }

        // Get content from last two synthesis/draft rounds
        let contents: Vec<&str> = session.rounds.iter().rev()
            .filter_map(|r| match r {
                ForgeRound::Draft(d) => Some(d.content.as_str()),
                ForgeRound::Synthesis(s) => Some(s.merged_content.as_str()),
                ForgeRound::Refinement(r) => Some(r.refined_content.as_str()),
                _ => None,
            })
            .take(2)
            .collect();

        if contents.len() < 2 {
            return 0.0;
        }

        // Calculate similarity (inverse of change)
        let similarity = calculate_text_similarity(contents[0], contents[1]);

        // High similarity = low change velocity = good for convergence
        similarity
    }

    /// Calculate issue score (fewer issues = higher score).
    fn calculate_issue_score(&self, session: &ForgeSession) -> f64 {
        let recent_critiques = session.rounds.iter().rev()
            .find_map(|r| match r {
                ForgeRound::Critique(c) => Some(&c.critiques),
                _ => None,
            });

        if let Some(critiques) = recent_critiques {
            let total_issues: usize = critiques.iter()
                .map(|c| c.weaknesses.len() + c.suggestions.len())
                .sum();

            // Normalize: 0 issues = 1.0, 20+ issues = 0.0
            1.0 - (total_issues as f64 / 20.0).min(1.0)
        } else {
            0.5
        }
    }

    /// Calculate semantic similarity between recent versions.
    fn calculate_semantic_similarity(&self, session: &ForgeSession) -> f64 {
        // Simplified: use text similarity as proxy
        self.calculate_change_velocity(session)
    }

    /// Calculate section stability (how stable individual sections are).
    fn calculate_section_stability(&self, session: &ForgeSession) -> f64 {
        if session.rounds.len() < 2 {
            return 0.0;
        }

        // Get section counts from recent rounds
        let section_counts: Vec<usize> = session.rounds.iter().rev()
            .filter_map(|r| match r {
                ForgeRound::Draft(d) => Some(count_sections(&d.content)),
                ForgeRound::Synthesis(s) => Some(count_sections(&s.merged_content)),
                ForgeRound::Refinement(r) => Some(count_sections(&r.refined_content)),
                _ => None,
            })
            .take(3)
            .collect();

        if section_counts.len() < 2 {
            return 0.5;
        }

        // Check if section count is stable
        let is_stable = section_counts.windows(2)
            .all(|w| (w[0] as i32 - w[1] as i32).abs() <= 1);

        if is_stable { 0.9 } else { 0.5 }
    }

    /// Collect convergence votes from participants.
    async fn collect_votes(
        &self,
        session: &ForgeSession,
        participants: &[Participant],
    ) -> ForgeResult<Vec<ConvergenceVote>> {
        let current_content = session.latest_draft().unwrap_or_default();

        let mut votes = Vec::new();

        for participant in participants {
            let request = build_vote_request(current_content, &session.topic);
            let response = self.participants.send_request(participant, request).await?;

            let vote = parse_vote_response(&response.content, participant);
            votes.push(vote);
        }

        Ok(votes)
    }

    /// Calculate overall convergence score.
    fn calculate_overall_score(
        &self,
        metrics: &HashMap<ConvergenceMetric, f64>,
        votes: &[ConvergenceVote],
    ) -> f64 {
        // Weighted average of metrics
        let metric_weights: HashMap<ConvergenceMetric, f64> = [
            (ConvergenceMetric::AgreementScore, 0.3),
            (ConvergenceMetric::ChangeVelocity, 0.25),
            (ConvergenceMetric::IssueCount, 0.25),
            (ConvergenceMetric::SemanticSimilarity, 0.1),
            (ConvergenceMetric::SectionStability, 0.1),
        ].into_iter().collect();

        let metric_score: f64 = metrics.iter()
            .map(|(m, s)| s * metric_weights.get(m).unwrap_or(&0.1))
            .sum();

        // Include vote agreement
        let vote_score = if votes.is_empty() {
            0.5
        } else {
            let avg_vote_score: f64 = votes.iter()
                .map(|v| v.score as f64 / 100.0)
                .sum::<f64>() / votes.len() as f64;
            avg_vote_score
        };

        // Combined score
        (metric_score * 0.6) + (vote_score * 0.4)
    }

    /// Determine if session has converged.
    fn is_converged(
        &self,
        score: f64,
        agreeing_count: usize,
        participants: &[Participant],
    ) -> bool {
        // Check threshold
        if score < self.config.convergence.threshold {
            return false;
        }

        // Check consensus requirement
        if self.config.convergence.require_unanimous {
            return agreeing_count == participants.len();
        }

        agreeing_count >= self.config.convergence.min_consensus
    }

    /// Create a not-converged round.
    fn create_not_converged_round(
        &self,
        round_number: usize,
        score: f64,
        votes: Vec<ConvergenceVote>,
        issues: Vec<String>,
    ) -> ConvergenceRound {
        ConvergenceRound {
            round_number,
            score,
            converged: false,
            votes,
            remaining_issues: issues,
            timestamp: tachikoma_common_core::Timestamp::now(),
            tokens: TokenCount::default(),
        }
    }

    /// Detect if the session is stalled (not improving).
    pub fn is_stalled(&self) -> bool {
        if self.score_history.len() < 3 {
            return false;
        }

        // Check last 3 scores
        let recent: Vec<_> = self.score_history.iter().rev().take(3).collect();

        // Stalled if scores are not improving
        let improving = recent.windows(2)
            .any(|w| w[1].overall > w[0].overall + 0.02);

        !improving
    }

    /// Get convergence trend.
    pub fn get_trend(&self) -> ConvergenceTrend {
        if self.score_history.len() < 2 {
            return ConvergenceTrend::Unknown;
        }

        let recent: Vec<_> = self.score_history.iter().rev().take(3).collect();
        let diffs: Vec<f64> = recent.windows(2)
            .map(|w| w[0].overall - w[1].overall)
            .collect();

        let avg_diff = diffs.iter().sum::<f64>() / diffs.len() as f64;

        if avg_diff > 0.05 {
            ConvergenceTrend::Improving
        } else if avg_diff < -0.05 {
            ConvergenceTrend::Degrading
        } else {
            ConvergenceTrend::Stable
        }
    }
}

/// Trend of convergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceTrend {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Build request for convergence vote.
fn build_vote_request(content: &str, topic: &crate::BrainstormTopic) -> ModelRequest {
    let system = r#"You are evaluating whether a draft has reached a satisfactory state.

Evaluate the draft and provide:
1. Whether you agree it's ready (yes/no)
2. A score from 0-100
3. Any remaining concerns"#.to_string();

    let user = format!(
        r#"## Topic: {}

## Draft to Evaluate
{}

## Instructions

Is this draft ready for finalization?

Respond in this exact format:
AGREES: [yes/no]
SCORE: [0-100]
CONCERNS:
- [concern 1]
- [concern 2]
(or "none" if no concerns)"#,
        topic.title,
        truncate_for_vote(content)
    );

    ModelRequest::new(system)
        .with_user_message(user)
        .with_temperature(0.3)
        .with_max_tokens(500)
}

/// Parse vote response.
fn parse_vote_response(content: &str, participant: &Participant) -> ConvergenceVote {
    let agrees = content.to_lowercase().contains("agrees: yes");

    let score: u8 = content.lines()
        .find(|l| l.to_lowercase().starts_with("score:"))
        .and_then(|l| {
            l.split(':').nth(1)
                .and_then(|s| s.trim().parse().ok())
        })
        .unwrap_or(50);

    let concerns: Vec<String> = content.lines()
        .skip_while(|l| !l.to_lowercase().contains("concerns"))
        .skip(1)
        .filter(|l| l.starts_with('-') || l.starts_with('*'))
        .map(|l| l.trim_start_matches(['-', '*', ' ']).to_string())
        .filter(|s| !s.is_empty() && s.to_lowercase() != "none")
        .collect();

    ConvergenceVote {
        participant: participant.clone(),
        agrees,
        score,
        concerns,
    }
}

/// Calculate text similarity.
fn calculate_text_similarity(a: &str, b: &str) -> f64 {
    let a_words: std::collections::HashSet<_> = a.to_lowercase()
        .split_whitespace()
        .collect();
    let b_words: std::collections::HashSet<_> = b.to_lowercase()
        .split_whitespace()
        .collect();

    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();

    if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
}

/// Count sections in content.
fn count_sections(content: &str) -> usize {
    content.lines()
        .filter(|l| l.starts_with('#'))
        .count()
}

/// Truncate content for vote request.
fn truncate_for_vote(content: &str) -> &str {
    const MAX_CHARS: usize = 30_000;
    if content.len() <= MAX_CHARS {
        content
    } else {
        &content[..MAX_CHARS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_text_similarity() {
        let a = "the quick brown fox";
        let b = "the quick red fox";

        let sim = calculate_text_similarity(a, b);
        assert!(sim > 0.5 && sim < 1.0);
    }

    #[test]
    fn test_parse_vote_response() {
        let content = "AGREES: yes\nSCORE: 85\nCONCERNS:\n- Minor typo";
        let participant = Participant::claude_sonnet();

        let vote = parse_vote_response(content, &participant);

        assert!(vote.agrees);
        assert_eq!(vote.score, 85);
        assert_eq!(vote.concerns.len(), 1);
    }
}
```

---

## Testing Requirements

1. Metric calculation produces valid scores
2. Vote collection handles various response formats
3. Convergence determination respects configuration
4. Stall detection identifies lack of progress
5. Trend analysis is accurate
6. Overall score combines metrics appropriately

---

## Related Specs

- Depends on: [145-recursive-refine.md](145-recursive-refine.md)
- Depends on: [147-convergence-metrics.md](147-convergence-metrics.md)
- Next: [147-convergence-metrics.md](147-convergence-metrics.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
