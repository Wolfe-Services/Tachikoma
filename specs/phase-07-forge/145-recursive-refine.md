# 145 - Recursive Refinement

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 145
**Status:** Planned
**Dependencies:** 144-round3-conflict, 139-forge-rounds
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement recursive refinement that allows deep, focused improvement of specific sections or aspects of the draft through multiple passes, with depth tracking and termination conditions.

---

## Acceptance Criteria

- [ ] Focus area selection for refinement
- [ ] Depth tracking and limits
- [ ] Quality improvement measurement
- [ ] Termination conditions (convergence, max depth)
- [ ] Section-specific refinement
- [ ] Refinement history tracking

---

## Implementation Details

### 1. Refinement Engine (src/rounds/refinement.rs)

```rust
//! Recursive refinement implementation.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;

use crate::{
    BrainstormTopic, ForgeConfig, ForgeError, ForgeResult, ModelRequest,
    Participant, ParticipantManager, RefinementRound, TokenCount,
};

/// Engine for recursive refinement.
pub struct RefinementEngine<'a> {
    participants: &'a ParticipantManager,
    config: &'a ForgeConfig,
    /// Tracks refinement depth per focus area.
    depth_tracker: HashMap<String, usize>,
    /// Quality scores per focus area.
    quality_tracker: HashMap<String, Vec<f64>>,
}

impl<'a> RefinementEngine<'a> {
    /// Create a new refinement engine.
    pub fn new(participants: &'a ParticipantManager, config: &'a ForgeConfig) -> Self {
        Self {
            participants,
            config,
            depth_tracker: HashMap::new(),
            quality_tracker: HashMap::new(),
        }
    }

    /// Execute a refinement round.
    pub async fn refine(
        &mut self,
        round_number: usize,
        content: &str,
        topic: &BrainstormTopic,
        previous_issues: &[String],
    ) -> ForgeResult<RefinementRound> {
        // Select focus area for this refinement
        let focus_area = self.select_focus_area(content, previous_issues);

        // Check depth limit
        let current_depth = *self.depth_tracker.get(&focus_area).unwrap_or(&0);
        if current_depth >= self.config.rounds.refinement.max_depth {
            return Err(ForgeError::Orchestration(format!(
                "Max refinement depth ({}) reached for '{}'",
                self.config.rounds.refinement.max_depth,
                focus_area
            )));
        }

        // Get refiner
        let refiner = self.participants.get_drafter().await?;

        // Build refinement request
        let request = self.build_refinement_request(
            content,
            &focus_area,
            current_depth,
            topic,
            previous_issues,
        );

        // Execute with timeout
        let timeout_duration = Duration::from_secs(self.config.rounds.refinement.timeout_secs);
        let start = Instant::now();

        let response = timeout(
            timeout_duration,
            self.participants.send_request(&refiner, request),
        )
        .await
        .map_err(|_| ForgeError::Timeout("Refinement timed out".to_string()))??;

        // Update tracking
        self.depth_tracker.insert(focus_area.clone(), current_depth + 1);

        // Extract and score the refined content
        let refined_content = extract_refined_content(&response.content);
        let quality_score = self.assess_refinement_quality(content, &refined_content, &focus_area);
        self.quality_tracker
            .entry(focus_area.clone())
            .or_default()
            .push(quality_score);

        Ok(RefinementRound {
            round_number,
            refiner,
            focus_area,
            refined_content,
            depth: current_depth + 1,
            timestamp: response.timestamp,
            tokens: response.tokens,
            duration_ms: response.duration_ms,
        })
    }

    /// Select the focus area for refinement.
    fn select_focus_area(&self, content: &str, previous_issues: &[String]) -> String {
        // Priority: previous issues > configured focus areas > least-refined areas

        // Check for issues that map to focus areas
        for issue in previous_issues {
            let issue_lower = issue.to_lowercase();
            for area in &self.config.rounds.refinement.focus_areas {
                if issue_lower.contains(&area.to_lowercase()) {
                    // Check if not already at max depth
                    if *self.depth_tracker.get(area).unwrap_or(&0) < self.config.rounds.refinement.max_depth {
                        return area.clone();
                    }
                }
            }
        }

        // Find least-refined configured area
        let mut min_depth = usize::MAX;
        let mut selected = self.config.rounds.refinement.focus_areas
            .first()
            .cloned()
            .unwrap_or_else(|| "general".to_string());

        for area in &self.config.rounds.refinement.focus_areas {
            let depth = *self.depth_tracker.get(area).unwrap_or(&0);
            if depth < min_depth {
                min_depth = depth;
                selected = area.clone();
            }
        }

        selected
    }

    /// Build the refinement request.
    fn build_refinement_request(
        &self,
        content: &str,
        focus_area: &str,
        current_depth: usize,
        topic: &BrainstormTopic,
        previous_issues: &[String],
    ) -> ModelRequest {
        let system = self.build_refinement_system_prompt(focus_area, current_depth);
        let user = self.build_refinement_user_prompt(content, focus_area, topic, previous_issues);

        ModelRequest::new(system)
            .with_user_message(user)
            .with_temperature(0.5 + (current_depth as f32 * 0.1).min(0.3)) // Slightly more creative at deeper levels
            .with_max_tokens(8192)
    }

    /// Build system prompt for refinement.
    fn build_refinement_system_prompt(&self, focus_area: &str, depth: usize) -> String {
        let focus_instructions = get_focus_area_instructions(focus_area);

        format!(
            r#"You are refining a draft specification, focusing specifically on: {focus_area}

This is refinement pass {depth} of maximum {max_depth} for this focus area.

{focus_instructions}

Refinement Guidelines:
1. Focus ONLY on the specified area - don't make unrelated changes
2. Make targeted, high-impact improvements
3. Preserve the overall structure and other aspects
4. Be more aggressive with changes at lower depths, more conservative at higher depths
5. Clearly mark any significant structural changes

Output the complete refined document, not just the changed sections."#,
            focus_area = focus_area,
            depth = depth + 1,
            max_depth = self.config.rounds.refinement.max_depth,
            focus_instructions = focus_instructions,
        )
    }

    /// Build user prompt for refinement.
    fn build_refinement_user_prompt(
        &self,
        content: &str,
        focus_area: &str,
        topic: &BrainstormTopic,
        previous_issues: &[String],
    ) -> String {
        let relevant_issues: Vec<_> = previous_issues
            .iter()
            .filter(|issue| issue.to_lowercase().contains(&focus_area.to_lowercase()))
            .collect();

        let issues_section = if relevant_issues.is_empty() {
            "No specific issues flagged for this area.".to_string()
        } else {
            relevant_issues
                .iter()
                .map(|i| format!("- {}", i))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            r#"# Refinement Request

## Topic
**Title:** {title}
**Focus Area:** {focus_area}

## Current Draft

<draft>
{content}
</draft>

## Issues to Address in This Focus Area
{issues}

## Your Task

Refine the draft with a specific focus on improving the **{focus_area}** aspect.

For this refinement:
1. Identify specific weaknesses in {focus_area}
2. Make targeted improvements
3. Ensure changes don't negatively impact other aspects
4. Output the COMPLETE refined document

Begin your refinement:"#,
            title = topic.title,
            focus_area = focus_area,
            content = content,
            issues = issues_section,
        )
    }

    /// Assess the quality improvement from refinement.
    fn assess_refinement_quality(
        &self,
        original: &str,
        refined: &str,
        focus_area: &str,
    ) -> f64 {
        // Simple heuristic assessment
        let mut score = 50.0;

        // Check if content actually changed
        let change_ratio = calculate_change_ratio(original, refined);
        if change_ratio < 0.01 {
            return 30.0; // Minimal change
        }
        if change_ratio > 0.5 {
            score -= 10.0; // Too much change might indicate problems
        }

        // Focus-area specific checks
        match focus_area {
            "code_quality" => {
                if refined.contains("```rust") || refined.contains("```") {
                    score += 10.0;
                }
                if refined.matches("fn ").count() >= original.matches("fn ").count() {
                    score += 5.0;
                }
            }
            "completeness" => {
                // Check if content grew
                if refined.len() > original.len() {
                    score += 10.0;
                }
                // Check for more sections
                if refined.matches("## ").count() >= original.matches("## ").count() {
                    score += 5.0;
                }
            }
            "clarity" => {
                // Check for shorter sentences (simpler)
                let original_avg_sentence = average_sentence_length(original);
                let refined_avg_sentence = average_sentence_length(refined);
                if refined_avg_sentence < original_avg_sentence {
                    score += 10.0;
                }
            }
            _ => {}
        }

        score.clamp(0.0, 100.0)
    }

    /// Check if further refinement is needed.
    pub fn should_continue_refining(&self) -> bool {
        // Check if any focus area hasn't reached max depth
        for area in &self.config.rounds.refinement.focus_areas {
            let depth = *self.depth_tracker.get(area).unwrap_or(&0);
            if depth < self.config.rounds.refinement.max_depth {
                return true;
            }
        }

        // Check if quality is improving
        for scores in self.quality_tracker.values() {
            if scores.len() >= 2 {
                let recent = scores.last().unwrap();
                let previous = scores[scores.len() - 2];
                if recent - previous > 5.0 {
                    // Still improving
                    return true;
                }
            }
        }

        false
    }

    /// Get refinement statistics.
    pub fn get_stats(&self) -> RefinementStats {
        RefinementStats {
            total_refinements: self.depth_tracker.values().sum(),
            refinements_by_area: self.depth_tracker.clone(),
            quality_by_area: self.quality_tracker
                .iter()
                .map(|(k, v)| (k.clone(), v.last().copied().unwrap_or(0.0)))
                .collect(),
        }
    }
}

/// Statistics about refinement progress.
#[derive(Debug, Clone)]
pub struct RefinementStats {
    pub total_refinements: usize,
    pub refinements_by_area: HashMap<String, usize>,
    pub quality_by_area: HashMap<String, f64>,
}

/// Get instructions for a specific focus area.
fn get_focus_area_instructions(focus_area: &str) -> &'static str {
    match focus_area.to_lowercase().as_str() {
        "code_quality" => {
            r#"Code Quality Focus:
- Ensure code examples are correct and idiomatic
- Add error handling where missing
- Improve naming and documentation
- Check for potential bugs or edge cases
- Ensure code is testable"#
        }
        "completeness" => {
            r#"Completeness Focus:
- Identify missing sections or topics
- Expand thin areas with more detail
- Add examples where helpful
- Ensure all requirements are addressed
- Fill in any [TODO] or [TBD] markers"#
        }
        "clarity" => {
            r#"Clarity Focus:
- Simplify complex explanations
- Break up long paragraphs
- Add helpful transitions
- Define jargon and acronyms
- Improve sentence structure"#
        }
        "performance" => {
            r#"Performance Focus:
- Identify potential performance issues
- Suggest optimizations
- Consider memory usage
- Think about scalability
- Add performance-related documentation"#
        }
        "security" => {
            r#"Security Focus:
- Identify potential security vulnerabilities
- Ensure proper input validation
- Check for sensitive data handling
- Consider authentication/authorization
- Add security-related documentation"#
        }
        _ => {
            r#"General Refinement:
- Improve overall quality
- Fix any obvious issues
- Enhance readability
- Ensure consistency"#
        }
    }
}

/// Extract the refined content from the response.
fn extract_refined_content(response: &str) -> String {
    // Look for content after common headers
    let markers = [
        "## Improved Draft",
        "## Refined Draft",
        "## Complete Refined Document",
        "# Refined Version",
    ];

    for marker in markers {
        if let Some(pos) = response.find(marker) {
            let start = pos + marker.len();
            return response[start..].trim().to_string();
        }
    }

    // If no marker, assume the whole response is the refined content
    response.trim().to_string()
}

/// Calculate the ratio of changed content.
fn calculate_change_ratio(original: &str, refined: &str) -> f64 {
    let original_words: std::collections::HashSet<_> =
        original.split_whitespace().collect();
    let refined_words: std::collections::HashSet<_> =
        refined.split_whitespace().collect();

    let intersection = original_words.intersection(&refined_words).count();
    let union = original_words.union(&refined_words).count();

    if union == 0 {
        0.0
    } else {
        1.0 - (intersection as f64 / union as f64)
    }
}

/// Calculate average sentence length.
fn average_sentence_length(text: &str) -> f64 {
    let sentences: Vec<_> = text.split(['.', '!', '?'])
        .filter(|s| !s.trim().is_empty())
        .collect();

    if sentences.is_empty() {
        0.0
    } else {
        let total_words: usize = sentences.iter()
            .map(|s| s.split_whitespace().count())
            .sum();
        total_words as f64 / sentences.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_change_ratio() {
        let original = "the quick brown fox";
        let refined = "the quick red fox jumps";

        let ratio = calculate_change_ratio(original, refined);
        assert!(ratio > 0.0 && ratio < 1.0);
    }

    #[test]
    fn test_average_sentence_length() {
        let text = "This is short. This is a much longer sentence with more words.";
        let avg = average_sentence_length(text);
        assert!(avg > 3.0 && avg < 10.0);
    }

    #[test]
    fn test_extract_refined_content() {
        let response = "Here is my analysis.\n\n## Refined Draft\n\nThis is the refined content.";
        let content = extract_refined_content(response);
        assert_eq!(content, "This is the refined content.");
    }
}
```

---

## Testing Requirements

1. Focus area selection prioritizes issues
2. Depth tracking prevents infinite recursion
3. Quality assessment produces reasonable scores
4. Termination conditions work correctly
5. Refined content extraction is robust
6. Statistics tracking is accurate

---

## Related Specs

- Depends on: [144-round3-conflict.md](144-round3-conflict.md)
- Depends on: [139-forge-rounds.md](139-forge-rounds.md)
- Next: [146-convergence-detect.md](146-convergence-detect.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
