# 143 - Round 3: Synthesis Prompt Construction

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 143
**Status:** Planned
**Dependencies:** 142-round2-critique-collect, 158-forge-templates
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement synthesis prompt construction that guides the synthesizer model to merge multiple critiques into an improved draft, explicitly handling conflicts and tracking changes.

---

## Acceptance Criteria

- [ ] Synthesis prompt incorporating all critiques
- [ ] Conflict identification guidance
- [ ] Change tracking instructions
- [ ] Weighted suggestion prioritization
- [ ] Context-efficient critique summarization
- [ ] Structured output format for synthesis

---

## Implementation Details

### 1. Synthesis Prompt Builder (src/prompts/synthesis.rs)

```rust
//! Synthesis prompt construction.

use crate::{
    BrainstormTopic, Change, ChangeType, ConflictResolution, Critique, ForgeConfig,
    ModelRequest, Suggestion, SuggestionCategory,
};

/// Build a synthesis prompt from critiques.
pub fn build_synthesis_prompt(
    current_content: &str,
    critiques: &[Critique],
    topic: &BrainstormTopic,
    config: &ForgeConfig,
) -> ModelRequest {
    let system = build_synthesis_system_prompt(topic);
    let user = build_synthesis_user_prompt(current_content, critiques, topic, config);

    ModelRequest::new(system)
        .with_user_message(user)
        .with_max_tokens(8192) // Synthesis often needs more tokens
        .with_temperature(0.6)
}

/// Build system prompt for synthesis.
fn build_synthesis_system_prompt(topic: &BrainstormTopic) -> String {
    format!(
        r#"You are the synthesizer in a multi-model brainstorming session.

Your role is to:
1. Analyze critiques from multiple AI reviewers
2. Identify areas of consensus and conflict
3. Merge improvements into an updated draft
4. Resolve conflicts with clear rationale
5. Track all changes made

Key principles:
- Give weight to suggestions that multiple critics agree on
- When critics disagree, consider the strength of their arguments
- Maintain the original structure unless changes are necessary
- Preserve what works well while addressing weaknesses
- Be explicit about trade-offs when resolving conflicts

Output Type: {output_type:?}

Your synthesis will be reviewed by the same critics, so ensure changes are justified."#,
        output_type = topic.output_type
    )
}

/// Build user prompt for synthesis.
fn build_synthesis_user_prompt(
    current_content: &str,
    critiques: &[Critique],
    topic: &BrainstormTopic,
    config: &ForgeConfig,
) -> String {
    // Summarize and categorize critiques
    let critique_summary = summarize_critiques(critiques);
    let consensus_points = find_consensus_points(critiques);
    let conflict_points = find_conflict_points(critiques);
    let prioritized_suggestions = prioritize_suggestions(critiques);

    format!(
        r#"# Synthesis Request

## Original Topic
**Title:** {title}
**Description:** {description}

## Current Draft

<current_draft>
{draft}
</current_draft>

## Critique Summary

### Overall Scores
{scores}

### Consensus Points (Multiple critics agree)
{consensus}

### Conflict Points (Critics disagree)
{conflicts}

### Prioritized Suggestions
{suggestions}

## Individual Critiques

{individual_critiques}

## Your Task

Create an improved version of the draft that:
1. Addresses the consensus points (high priority)
2. Resolves conflicts with clear rationale
3. Incorporates high-priority suggestions
4. Preserves identified strengths

## Required Output Format

Your response MUST follow this structure:

```synthesis
## Conflict Resolutions

### Conflict 1: [Brief description]
- **Position A:** [Who said what]
- **Position B:** [Who said what]
- **Resolution:** [What you decided]
- **Rationale:** [Why this resolution]

[Repeat for each conflict]

## Changes Made

### Change 1
- **Section:** [Section name]
- **Type:** [addition/modification/deletion/restructure]
- **Description:** [What changed]
- **Based on:** [Which suggestions/critiques]

[Repeat for each significant change]

## Improved Draft

[The complete improved draft goes here]
```

## Guidelines

1. Every significant change should be traceable to critique feedback
2. If you disagree with a suggestion, explain why you didn't incorporate it
3. Maintain consistency in style and terminology
4. The improved draft should be complete and standalone
5. Focus on substantive improvements, not just cosmetic changes

Begin your synthesis:"#,
        title = topic.title,
        description = topic.description,
        draft = truncate_for_context(current_content, 40_000),
        scores = format_scores(critiques),
        consensus = format_consensus(&consensus_points),
        conflicts = format_conflicts(&conflict_points),
        suggestions = format_prioritized_suggestions(&prioritized_suggestions),
        individual_critiques = format_individual_critiques(critiques),
    )
}

/// Summarize critiques for context.
fn summarize_critiques(critiques: &[Critique]) -> CritiqueSummary {
    let mut summary = CritiqueSummary::default();

    for critique in critiques {
        summary.total_count += 1;
        summary.average_score += critique.score as f64;

        for strength in &critique.strengths {
            *summary.strength_counts.entry(strength.clone()).or_insert(0) += 1;
        }

        for weakness in &critique.weaknesses {
            *summary.weakness_counts.entry(weakness.clone()).or_insert(0) += 1;
        }
    }

    if summary.total_count > 0 {
        summary.average_score /= summary.total_count as f64;
    }

    summary
}

#[derive(Default)]
struct CritiqueSummary {
    total_count: usize,
    average_score: f64,
    strength_counts: std::collections::HashMap<String, usize>,
    weakness_counts: std::collections::HashMap<String, usize>,
}

/// Find points where multiple critics agree.
fn find_consensus_points(critiques: &[Critique]) -> Vec<ConsensusPoint> {
    let mut points = Vec::new();
    let threshold = (critiques.len() as f64 * 0.5).ceil() as usize;

    // Aggregate similar suggestions
    let mut suggestion_groups: std::collections::HashMap<String, Vec<&Suggestion>> =
        std::collections::HashMap::new();

    for critique in critiques {
        for suggestion in &critique.suggestions {
            // Group by category and approximate content
            let key = format!("{:?}:{}", suggestion.category, &suggestion.text[..suggestion.text.len().min(50)]);
            suggestion_groups.entry(key).or_default().push(suggestion);
        }
    }

    // Find groups with multiple supporters
    for (key, suggestions) in suggestion_groups {
        if suggestions.len() >= threshold {
            points.push(ConsensusPoint {
                category: suggestions[0].category,
                description: suggestions[0].text.clone(),
                support_count: suggestions.len(),
                average_priority: suggestions.iter().map(|s| s.priority as f64).sum::<f64>()
                    / suggestions.len() as f64,
            });
        }
    }

    // Sort by support and priority
    points.sort_by(|a, b| {
        b.support_count.cmp(&a.support_count)
            .then(a.average_priority.partial_cmp(&b.average_priority).unwrap())
    });

    points
}

#[derive(Debug)]
struct ConsensusPoint {
    category: SuggestionCategory,
    description: String,
    support_count: usize,
    average_priority: f64,
}

/// Find points where critics disagree.
fn find_conflict_points(critiques: &[Critique]) -> Vec<ConflictPoint> {
    let mut conflicts = Vec::new();

    // Look for contradictory assessments
    // e.g., one says "good error handling" another says "weak error handling"

    let mut assessments: std::collections::HashMap<String, Vec<(&Critique, bool)>> =
        std::collections::HashMap::new();

    // Check strengths vs weaknesses
    for critique in critiques {
        for strength in &critique.strengths {
            let topic = extract_topic(strength);
            assessments.entry(topic).or_default().push((critique, true));
        }
        for weakness in &critique.weaknesses {
            let topic = extract_topic(weakness);
            assessments.entry(topic).or_default().push((critique, false));
        }
    }

    // Find topics with both positive and negative assessments
    for (topic, votes) in assessments {
        let positives: Vec<_> = votes.iter().filter(|(_, pos)| *pos).collect();
        let negatives: Vec<_> = votes.iter().filter(|(_, pos)| !*pos).collect();

        if !positives.is_empty() && !negatives.is_empty() {
            conflicts.push(ConflictPoint {
                topic: topic.clone(),
                positive_views: positives.iter().map(|(c, _)| c.critic.display_name.clone()).collect(),
                negative_views: negatives.iter().map(|(c, _)| c.critic.display_name.clone()).collect(),
            });
        }
    }

    conflicts
}

#[derive(Debug)]
struct ConflictPoint {
    topic: String,
    positive_views: Vec<String>,
    negative_views: Vec<String>,
}

/// Extract the main topic from an assessment.
fn extract_topic(text: &str) -> String {
    // Simple extraction - get key words
    let text = text.to_lowercase();
    let key_topics = [
        "error handling", "type safety", "performance", "security",
        "documentation", "testing", "architecture", "api design",
        "code organization", "clarity", "completeness",
    ];

    for topic in key_topics {
        if text.contains(topic) {
            return topic.to_string();
        }
    }

    // Fall back to first few words
    text.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
}

/// Prioritize suggestions across all critiques.
fn prioritize_suggestions(critiques: &[Critique]) -> Vec<PrioritizedSuggestion> {
    let mut suggestions: Vec<PrioritizedSuggestion> = Vec::new();

    for critique in critiques {
        for suggestion in &critique.suggestions {
            // Check if similar suggestion exists
            let similar = suggestions.iter_mut().find(|s| {
                s.category == suggestion.category
                    && similarity(&s.text, &suggestion.text) > 0.7
            });

            if let Some(existing) = similar {
                existing.supporters.push(critique.critic.display_name.clone());
                existing.combined_priority = (existing.combined_priority + suggestion.priority as f64) / 2.0;
            } else {
                suggestions.push(PrioritizedSuggestion {
                    category: suggestion.category,
                    text: suggestion.text.clone(),
                    section: suggestion.section.clone(),
                    supporters: vec![critique.critic.display_name.clone()],
                    combined_priority: suggestion.priority as f64,
                });
            }
        }
    }

    // Sort by support count (desc) then priority (asc)
    suggestions.sort_by(|a, b| {
        b.supporters.len().cmp(&a.supporters.len())
            .then(a.combined_priority.partial_cmp(&b.combined_priority).unwrap())
    });

    suggestions
}

#[derive(Debug)]
struct PrioritizedSuggestion {
    category: SuggestionCategory,
    text: String,
    section: Option<String>,
    supporters: Vec<String>,
    combined_priority: f64,
}

/// Simple similarity measure.
fn similarity(a: &str, b: &str) -> f64 {
    let a_words: std::collections::HashSet<_> = a.to_lowercase().split_whitespace().collect();
    let b_words: std::collections::HashSet<_> = b.to_lowercase().split_whitespace().collect();

    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

// Formatting helpers

fn format_scores(critiques: &[Critique]) -> String {
    critiques
        .iter()
        .map(|c| format!("- **{}:** {}/100", c.critic.display_name, c.score))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_consensus(points: &[ConsensusPoint]) -> String {
    if points.is_empty() {
        return "No strong consensus points identified.".to_string();
    }

    points
        .iter()
        .take(5)
        .map(|p| format!(
            "- [{:?}] {} (supported by {} critics, priority: {:.1})",
            p.category, p.description, p.support_count, p.average_priority
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_conflicts(conflicts: &[ConflictPoint]) -> String {
    if conflicts.is_empty() {
        return "No significant conflicts identified.".to_string();
    }

    conflicts
        .iter()
        .take(3)
        .map(|c| format!(
            "- **{}:** Positive view ({}) vs Negative view ({})",
            c.topic,
            c.positive_views.join(", "),
            c.negative_views.join(", ")
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_prioritized_suggestions(suggestions: &[PrioritizedSuggestion]) -> String {
    suggestions
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, s)| format!(
            "{}. [{:?}] {} (from: {})",
            i + 1,
            s.category,
            s.text,
            s.supporters.join(", ")
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_individual_critiques(critiques: &[Critique]) -> String {
    critiques
        .iter()
        .map(|c| {
            let strengths = c.strengths.iter()
                .map(|s| format!("  - {}", s))
                .collect::<Vec<_>>()
                .join("\n");
            let weaknesses = c.weaknesses.iter()
                .map(|w| format!("  - {}", w))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "### {} (Score: {})\n\n**Strengths:**\n{}\n\n**Weaknesses:**\n{}\n",
                c.critic.display_name, c.score, strengths, weaknesses
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n\n")
}

fn truncate_for_context(content: &str, max_chars: usize) -> &str {
    if content.len() <= max_chars {
        content
    } else {
        &content[..max_chars]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        assert!(similarity("error handling is good", "good error handling") > 0.5);
        assert!(similarity("completely different", "nothing alike") < 0.3);
    }

    #[test]
    fn test_extract_topic() {
        assert_eq!(extract_topic("The error handling is excellent"), "error handling");
        assert_eq!(extract_topic("Type safety could be improved"), "type safety");
    }
}
```

---

## Testing Requirements

1. Prompts include all critique data appropriately
2. Consensus detection identifies shared concerns
3. Conflict detection finds contradictions
4. Suggestion prioritization weights correctly
5. Output format is parseable
6. Long content is truncated appropriately

---

## Related Specs

- Depends on: [142-round2-critique-collect.md](142-round2-critique-collect.md)
- Depends on: [158-forge-templates.md](158-forge-templates.md)
- Next: [144-round3-conflict.md](144-round3-conflict.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
