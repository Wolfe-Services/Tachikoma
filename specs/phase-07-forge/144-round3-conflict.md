# 144 - Round 3: Conflict Resolution

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 144
**Status:** Planned
**Dependencies:** 143-round3-synthesis-prompts
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement the conflict resolution system that identifies disagreements between critics, facilitates resolution through the synthesizer, and logs decisions for transparency.

---

## Acceptance Criteria

- [ ] Automatic conflict detection from critiques
- [ ] Conflict categorization and prioritization
- [ ] Resolution strategy selection
- [ ] Synthesizer-guided resolution
- [ ] Resolution logging with rationale
- [ ] Human override support in attended mode

---

## Implementation Details

### 1. Conflict Detector (src/conflict/detector.rs)

```rust
//! Conflict detection in critiques.

use std::collections::HashMap;

use crate::{Critique, Participant, Suggestion, SuggestionCategory};

/// A detected conflict between critics.
#[derive(Debug, Clone)]
pub struct DetectedConflict {
    /// Unique conflict identifier.
    pub id: String,
    /// What the conflict is about.
    pub topic: ConflictTopic,
    /// Severity of the conflict (1-5).
    pub severity: u8,
    /// Positions held by different participants.
    pub positions: Vec<ConflictPosition>,
    /// Suggested resolution strategies.
    pub suggested_strategies: Vec<ResolutionStrategy>,
}

/// Position in a conflict.
#[derive(Debug, Clone)]
pub struct ConflictPosition {
    /// Who holds this position.
    pub participant: Participant,
    /// The position statement.
    pub statement: String,
    /// Evidence or reasoning.
    pub evidence: Vec<String>,
    /// Confidence level (0-100).
    pub confidence: u8,
}

/// Topic of a conflict.
#[derive(Debug, Clone)]
pub enum ConflictTopic {
    /// Disagreement about approach/architecture.
    Architecture(String),
    /// Disagreement about code quality assessment.
    CodeQuality(String),
    /// Disagreement about priority of a fix.
    Priority(String),
    /// Disagreement about correctness.
    Correctness(String),
    /// Disagreement about completeness.
    Completeness(String),
    /// Other type of conflict.
    Other(String),
}

impl ConflictTopic {
    /// Get the topic description.
    pub fn description(&self) -> &str {
        match self {
            Self::Architecture(s) => s,
            Self::CodeQuality(s) => s,
            Self::Priority(s) => s,
            Self::Correctness(s) => s,
            Self::Completeness(s) => s,
            Self::Other(s) => s,
        }
    }
}

/// Strategy for resolving a conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Go with the majority opinion.
    MajorityVote,
    /// Defer to the expert in this domain.
    DeferToExpert,
    /// Implement both solutions.
    ImplementBoth,
    /// Seek a middle ground.
    Compromise,
    /// Defer the decision.
    Defer,
    /// Escalate to human.
    EscalateToHuman,
}

/// Detect conflicts in a set of critiques.
pub fn detect_conflicts(critiques: &[Critique]) -> Vec<DetectedConflict> {
    let mut conflicts = Vec::new();

    // Detect assessment conflicts (strength vs weakness)
    conflicts.extend(detect_assessment_conflicts(critiques));

    // Detect suggestion conflicts
    conflicts.extend(detect_suggestion_conflicts(critiques));

    // Detect score discrepancies
    conflicts.extend(detect_score_conflicts(critiques));

    // Assign IDs and sort by severity
    for (i, conflict) in conflicts.iter_mut().enumerate() {
        conflict.id = format!("conflict_{}", i + 1);
    }

    conflicts.sort_by(|a, b| b.severity.cmp(&a.severity));

    conflicts
}

/// Detect conflicts where one critic praises and another criticizes the same aspect.
fn detect_assessment_conflicts(critiques: &[Critique]) -> Vec<DetectedConflict> {
    let mut conflicts = Vec::new();

    // Group assessments by topic
    let mut topics: HashMap<String, Vec<(&Critique, bool, &str)>> = HashMap::new();

    for critique in critiques {
        for strength in &critique.strengths {
            let topic = extract_topic_key(strength);
            topics.entry(topic).or_default().push((critique, true, strength));
        }
        for weakness in &critique.weaknesses {
            let topic = extract_topic_key(weakness);
            topics.entry(topic).or_default().push((critique, false, weakness));
        }
    }

    // Find topics with conflicting assessments
    for (topic, assessments) in topics {
        let positive: Vec<_> = assessments.iter().filter(|(_, pos, _)| *pos).collect();
        let negative: Vec<_> = assessments.iter().filter(|(_, pos, _)| !*pos).collect();

        if !positive.is_empty() && !negative.is_empty() {
            let mut positions = Vec::new();

            for (critique, _, statement) in &positive {
                positions.push(ConflictPosition {
                    participant: critique.critic.clone(),
                    statement: format!("Positive: {}", statement),
                    evidence: vec![],
                    confidence: 70,
                });
            }

            for (critique, _, statement) in &negative {
                positions.push(ConflictPosition {
                    participant: critique.critic.clone(),
                    statement: format!("Negative: {}", statement),
                    evidence: vec![],
                    confidence: 70,
                });
            }

            let severity = calculate_conflict_severity(&positive, &negative);

            conflicts.push(DetectedConflict {
                id: String::new(),
                topic: categorize_topic(&topic),
                severity,
                positions,
                suggested_strategies: suggest_resolution_strategies(severity, &positive, &negative),
            });
        }
    }

    conflicts
}

/// Detect conflicts in suggestions.
fn detect_suggestion_conflicts(critiques: &[Critique]) -> Vec<DetectedConflict> {
    let mut conflicts = Vec::new();

    // Group suggestions by section
    let mut by_section: HashMap<String, Vec<(&Critique, &Suggestion)>> = HashMap::new();

    for critique in critiques {
        for suggestion in &critique.suggestions {
            let section = suggestion.section.clone().unwrap_or_else(|| "general".to_string());
            by_section.entry(section).or_default().push((critique, suggestion));
        }
    }

    // Find conflicting suggestions for the same section
    for (section, suggestions) in by_section {
        // Look for contradictory suggestions
        for i in 0..suggestions.len() {
            for j in (i + 1)..suggestions.len() {
                let (c1, s1) = suggestions[i];
                let (c2, s2) = suggestions[j];

                if are_suggestions_contradictory(s1, s2) {
                    conflicts.push(DetectedConflict {
                        id: String::new(),
                        topic: ConflictTopic::Other(format!(
                            "Contradictory suggestions for section '{}'",
                            section
                        )),
                        severity: 3,
                        positions: vec![
                            ConflictPosition {
                                participant: c1.critic.clone(),
                                statement: s1.text.clone(),
                                evidence: vec![],
                                confidence: 70,
                            },
                            ConflictPosition {
                                participant: c2.critic.clone(),
                                statement: s2.text.clone(),
                                evidence: vec![],
                                confidence: 70,
                            },
                        ],
                        suggested_strategies: vec![
                            ResolutionStrategy::DeferToExpert,
                            ResolutionStrategy::Compromise,
                        ],
                    });
                }
            }
        }
    }

    conflicts
}

/// Detect significant score discrepancies.
fn detect_score_conflicts(critiques: &[Critique]) -> Vec<DetectedConflict> {
    let mut conflicts = Vec::new();

    if critiques.len() < 2 {
        return conflicts;
    }

    let scores: Vec<u8> = critiques.iter().map(|c| c.score).collect();
    let min_score = *scores.iter().min().unwrap();
    let max_score = *scores.iter().max().unwrap();

    // Flag if there's a large score discrepancy (> 30 points)
    if max_score - min_score > 30 {
        let high_scorer = critiques.iter().find(|c| c.score == max_score).unwrap();
        let low_scorer = critiques.iter().find(|c| c.score == min_score).unwrap();

        conflicts.push(DetectedConflict {
            id: String::new(),
            topic: ConflictTopic::Other("Significant score discrepancy".to_string()),
            severity: 2,
            positions: vec![
                ConflictPosition {
                    participant: high_scorer.critic.clone(),
                    statement: format!("Scored {}/100", max_score),
                    evidence: high_scorer.strengths.clone(),
                    confidence: 80,
                },
                ConflictPosition {
                    participant: low_scorer.critic.clone(),
                    statement: format!("Scored {}/100", min_score),
                    evidence: low_scorer.weaknesses.clone(),
                    confidence: 80,
                },
            ],
            suggested_strategies: vec![
                ResolutionStrategy::MajorityVote,
                ResolutionStrategy::DeferToExpert,
            ],
        });
    }

    conflicts
}

/// Extract a normalized topic key from text.
fn extract_topic_key(text: &str) -> String {
    let text = text.to_lowercase();

    // Extract key phrases
    let key_phrases = [
        "error handling", "type safety", "performance", "security",
        "documentation", "testing", "api design", "architecture",
        "code organization", "clarity", "completeness", "validation",
        "serialization", "concurrency", "memory", "logging",
    ];

    for phrase in key_phrases {
        if text.contains(phrase) {
            return phrase.to_string();
        }
    }

    // Fall back to stemmed first few words
    text.split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join("_")
}

/// Categorize a topic into a ConflictTopic variant.
fn categorize_topic(topic: &str) -> ConflictTopic {
    if topic.contains("architecture") || topic.contains("design") {
        ConflictTopic::Architecture(topic.to_string())
    } else if topic.contains("code") || topic.contains("quality") {
        ConflictTopic::CodeQuality(topic.to_string())
    } else if topic.contains("correct") || topic.contains("bug") || topic.contains("error") {
        ConflictTopic::Correctness(topic.to_string())
    } else if topic.contains("complete") || topic.contains("missing") {
        ConflictTopic::Completeness(topic.to_string())
    } else {
        ConflictTopic::Other(topic.to_string())
    }
}

/// Calculate conflict severity.
fn calculate_conflict_severity(
    positive: &[(&Critique, bool, &str)],
    negative: &[(&Critique, bool, &str)],
) -> u8 {
    // Higher severity if more critics are involved on each side
    let balance = (positive.len() as i32 - negative.len() as i32).abs();

    if balance == 0 {
        4 // Evenly split - high severity
    } else if balance == 1 {
        3 // Slight majority
    } else {
        2 // Clear majority
    }
}

/// Suggest resolution strategies based on conflict characteristics.
fn suggest_resolution_strategies(
    severity: u8,
    positive: &[(&Critique, bool, &str)],
    negative: &[(&Critique, bool, &str)],
) -> Vec<ResolutionStrategy> {
    let mut strategies = Vec::new();

    // If there's a clear majority, suggest voting
    if positive.len() != negative.len() {
        strategies.push(ResolutionStrategy::MajorityVote);
    }

    // Check if any participant is a domain expert
    let has_expert = positive.iter().chain(negative.iter())
        .any(|(c, _, _)| c.critic.role == crate::ParticipantRole::DomainExpert);

    if has_expert {
        strategies.push(ResolutionStrategy::DeferToExpert);
    }

    // High severity conflicts might need human input
    if severity >= 4 {
        strategies.push(ResolutionStrategy::EscalateToHuman);
    }

    // Always include compromise as an option
    strategies.push(ResolutionStrategy::Compromise);

    strategies
}

/// Check if two suggestions are contradictory.
fn are_suggestions_contradictory(s1: &Suggestion, s2: &Suggestion) -> bool {
    // Same category but opposite direction indicators
    if s1.category != s2.category {
        return false;
    }

    let text1 = s1.text.to_lowercase();
    let text2 = s2.text.to_lowercase();

    // Check for opposite actions
    let opposites = [
        ("add", "remove"),
        ("increase", "decrease"),
        ("simplify", "elaborate"),
        ("split", "merge"),
        ("keep", "remove"),
    ];

    for (word1, word2) in opposites {
        if (text1.contains(word1) && text2.contains(word2))
            || (text1.contains(word2) && text2.contains(word1))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_topic_key() {
        assert_eq!(extract_topic_key("Good error handling"), "error handling");
        assert_eq!(extract_topic_key("Performance could be better"), "performance");
    }

    #[test]
    fn test_are_suggestions_contradictory() {
        let s1 = Suggestion {
            section: Some("api".to_string()),
            text: "Add more parameters".to_string(),
            priority: 2,
            category: SuggestionCategory::Architecture,
        };

        let s2 = Suggestion {
            section: Some("api".to_string()),
            text: "Remove unnecessary parameters".to_string(),
            priority: 2,
            category: SuggestionCategory::Architecture,
        };

        assert!(are_suggestions_contradictory(&s1, &s2));
    }
}
```

### 2. Conflict Resolver (src/conflict/resolver.rs)

```rust
//! Conflict resolution execution.

use crate::{
    ConflictResolution, ConflictPosition as SessionConflictPosition, DetectedConflict,
    ForgeConfig, ForgeResult, ModelRequest, Participant, ParticipantManager,
    ResolutionStrategy,
};

/// Resolver for handling conflicts.
pub struct ConflictResolver<'a> {
    participants: &'a ParticipantManager,
    config: &'a ForgeConfig,
}

impl<'a> ConflictResolver<'a> {
    /// Create a new resolver.
    pub fn new(participants: &'a ParticipantManager, config: &'a ForgeConfig) -> Self {
        Self { participants, config }
    }

    /// Resolve a conflict using the specified strategy.
    pub async fn resolve(
        &self,
        conflict: &DetectedConflict,
        strategy: ResolutionStrategy,
        synthesizer: &Participant,
    ) -> ForgeResult<ConflictResolution> {
        match strategy {
            ResolutionStrategy::MajorityVote => self.resolve_by_majority(conflict),
            ResolutionStrategy::DeferToExpert => self.resolve_by_expert(conflict),
            ResolutionStrategy::Compromise => {
                self.resolve_by_synthesis(conflict, synthesizer).await
            }
            ResolutionStrategy::ImplementBoth => self.resolve_implement_both(conflict),
            ResolutionStrategy::Defer => self.resolve_defer(conflict),
            ResolutionStrategy::EscalateToHuman => self.resolve_escalate(conflict),
        }
    }

    /// Resolve by majority vote.
    fn resolve_by_majority(&self, conflict: &DetectedConflict) -> ForgeResult<ConflictResolution> {
        // Group positions by similarity
        let mut vote_counts: std::collections::HashMap<String, Vec<&crate::conflict::ConflictPosition>> =
            std::collections::HashMap::new();

        for position in &conflict.positions {
            // Simplify position to a vote direction
            let direction = if position.statement.to_lowercase().contains("positive")
                || position.statement.to_lowercase().contains("good")
                || position.statement.to_lowercase().contains("strong")
            {
                "positive"
            } else {
                "negative"
            };

            vote_counts.entry(direction.to_string()).or_default().push(position);
        }

        // Find majority
        let (winning_direction, winning_positions) = vote_counts
            .iter()
            .max_by_key(|(_, positions)| positions.len())
            .map(|(d, p)| (d.clone(), p.clone()))
            .unwrap_or_else(|| ("unclear".to_string(), vec![]));

        let resolution = if winning_direction == "positive" {
            "Keep the current approach as most reviewers found it acceptable."
        } else {
            "Address the concerns raised by the majority of reviewers."
        };

        Ok(ConflictResolution {
            issue: conflict.topic.description().to_string(),
            positions: conflict.positions.iter().map(|p| SessionConflictPosition {
                participant: p.participant.clone(),
                position: p.statement.clone(),
            }).collect(),
            resolution: resolution.to_string(),
            rationale: format!(
                "Majority vote: {} of {} reviewers supported this direction.",
                winning_positions.len(),
                conflict.positions.len()
            ),
        })
    }

    /// Resolve by deferring to domain expert.
    fn resolve_by_expert(&self, conflict: &DetectedConflict) -> ForgeResult<ConflictResolution> {
        // Find expert position
        let expert_position = conflict.positions.iter()
            .find(|p| p.participant.role == crate::ParticipantRole::DomainExpert)
            .or_else(|| conflict.positions.iter()
                .find(|p| p.participant.role == crate::ParticipantRole::CodeReviewer));

        if let Some(expert) = expert_position {
            Ok(ConflictResolution {
                issue: conflict.topic.description().to_string(),
                positions: conflict.positions.iter().map(|p| SessionConflictPosition {
                    participant: p.participant.clone(),
                    position: p.statement.clone(),
                }).collect(),
                resolution: expert.statement.clone(),
                rationale: format!(
                    "Deferred to {} ({:?}) as the domain expert.",
                    expert.participant.display_name,
                    expert.participant.role
                ),
            })
        } else {
            // Fall back to majority vote
            self.resolve_by_majority(conflict)
        }
    }

    /// Resolve by asking the synthesizer to find a compromise.
    async fn resolve_by_synthesis(
        &self,
        conflict: &DetectedConflict,
        synthesizer: &Participant,
    ) -> ForgeResult<ConflictResolution> {
        let prompt = build_resolution_prompt(conflict);

        let response = self.participants
            .send_request(synthesizer, prompt)
            .await?;

        // Parse resolution from response
        let (resolution, rationale) = parse_resolution_response(&response.content);

        Ok(ConflictResolution {
            issue: conflict.topic.description().to_string(),
            positions: conflict.positions.iter().map(|p| SessionConflictPosition {
                participant: p.participant.clone(),
                position: p.statement.clone(),
            }).collect(),
            resolution,
            rationale,
        })
    }

    /// Resolve by implementing both approaches.
    fn resolve_implement_both(&self, conflict: &DetectedConflict) -> ForgeResult<ConflictResolution> {
        Ok(ConflictResolution {
            issue: conflict.topic.description().to_string(),
            positions: conflict.positions.iter().map(|p| SessionConflictPosition {
                participant: p.participant.clone(),
                position: p.statement.clone(),
            }).collect(),
            resolution: "Implement both approaches where feasible, allowing configuration or runtime selection.".to_string(),
            rationale: "Both positions have merit; implementing both provides flexibility.".to_string(),
        })
    }

    /// Defer resolution for later.
    fn resolve_defer(&self, conflict: &DetectedConflict) -> ForgeResult<ConflictResolution> {
        Ok(ConflictResolution {
            issue: conflict.topic.description().to_string(),
            positions: conflict.positions.iter().map(|p| SessionConflictPosition {
                participant: p.participant.clone(),
                position: p.statement.clone(),
            }).collect(),
            resolution: "Deferred for future consideration.".to_string(),
            rationale: "This conflict requires more information or context to resolve definitively.".to_string(),
        })
    }

    /// Escalate to human decision maker.
    fn resolve_escalate(&self, conflict: &DetectedConflict) -> ForgeResult<ConflictResolution> {
        Ok(ConflictResolution {
            issue: conflict.topic.description().to_string(),
            positions: conflict.positions.iter().map(|p| SessionConflictPosition {
                participant: p.participant.clone(),
                position: p.statement.clone(),
            }).collect(),
            resolution: "ESCALATED: Requires human decision.".to_string(),
            rationale: "The conflict is significant enough to require human judgment.".to_string(),
        })
    }
}

/// Build a prompt for synthesizer-based resolution.
fn build_resolution_prompt(conflict: &DetectedConflict) -> ModelRequest {
    let system = r#"You are resolving a conflict between AI reviewers in a brainstorming session.

Your task is to find a balanced resolution that:
1. Acknowledges valid points from all sides
2. Provides a clear recommendation
3. Explains the rationale"#.to_string();

    let positions_text = conflict.positions.iter()
        .map(|p| format!("- **{}**: {}", p.participant.display_name, p.statement))
        .collect::<Vec<_>>()
        .join("\n");

    let user = format!(
        r#"## Conflict to Resolve

**Topic:** {}

**Severity:** {}/5

**Positions:**
{}

Please provide:
1. **Resolution:** A clear recommendation for what to do
2. **Rationale:** Why this resolution is best

Format your response as:
RESOLUTION: [your resolution]
RATIONALE: [your reasoning]"#,
        conflict.topic.description(),
        conflict.severity,
        positions_text
    );

    ModelRequest::new(system)
        .with_user_message(user)
        .with_temperature(0.5)
        .with_max_tokens(500)
}

/// Parse resolution and rationale from response.
fn parse_resolution_response(content: &str) -> (String, String) {
    let resolution = content
        .lines()
        .find(|l| l.starts_with("RESOLUTION:"))
        .map(|l| l.trim_start_matches("RESOLUTION:").trim().to_string())
        .unwrap_or_else(|| "Unable to determine resolution.".to_string());

    let rationale = content
        .lines()
        .find(|l| l.starts_with("RATIONALE:"))
        .map(|l| l.trim_start_matches("RATIONALE:").trim().to_string())
        .unwrap_or_else(|| "No rationale provided.".to_string());

    (resolution, rationale)
}
```

---

## Testing Requirements

1. Conflict detection identifies assessment contradictions
2. Conflict detection finds suggestion conflicts
3. Score discrepancy detection works correctly
4. Resolution strategies produce valid resolutions
5. Synthesizer-based resolution parses correctly
6. All resolution types have proper rationale

---

## Related Specs

- Depends on: [143-round3-synthesis-prompts.md](143-round3-synthesis-prompts.md)
- Next: [145-recursive-refine.md](145-recursive-refine.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
