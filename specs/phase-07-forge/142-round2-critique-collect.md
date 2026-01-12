# 142 - Round 2: Critique Collection and Parsing

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 142
**Status:** Planned
**Dependencies:** 141-round2-critique-prompts
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement critique collection from multiple models in parallel, with robust parsing of structured critique responses into actionable data structures.

---

## Acceptance Criteria

- [x] Parallel critique execution
- [x] Structured critique parsing
- [x] Fallback parsing for malformed responses
- [x] Critique aggregation and deduplication
- [x] Score normalization across models
- [x] Partial failure handling

---

## Implementation Details

### 1. Critique Collector (src/rounds/critique_collector.rs)

```rust
//! Critique collection and coordination.

use std::time::{Duration, Instant};
use futures::future::join_all;
use tokio::time::timeout;

use crate::{
    BrainstormTopic, Critique, CritiqueRound, ForgeConfig, ForgeError, ForgeResult,
    ModelRequest, ModelResponse, Participant, ParticipantManager, Suggestion,
    SuggestionCategory, TokenCount,
};

/// Collector for critique rounds.
pub struct CritiqueCollector<'a> {
    participants: &'a ParticipantManager,
    config: &'a ForgeConfig,
}

impl<'a> CritiqueCollector<'a> {
    /// Create a new collector.
    pub fn new(participants: &'a ParticipantManager, config: &'a ForgeConfig) -> Self {
        Self { participants, config }
    }

    /// Collect critiques from all designated critics.
    pub async fn collect(
        &self,
        round_number: usize,
        draft_content: &str,
        topic: &BrainstormTopic,
    ) -> ForgeResult<CritiqueRound> {
        // Get critics
        let critics = self.participants.get_critics(
            self.config.rounds.critique.min_critiques
        ).await?;

        // Build requests for each critic
        let requests: Vec<(Participant, ModelRequest)> = critics.iter().map(|critic| {
            let request = crate::prompts::build_critique_prompt(
                draft_content,
                topic,
                critic,
                self.config,
            );
            (critic.clone(), request)
        }).collect();

        // Execute critiques
        let responses = if self.config.rounds.critique.parallel {
            self.collect_parallel(requests).await
        } else {
            self.collect_sequential(requests).await
        };

        // Parse responses into critiques
        let mut critiques = Vec::new();
        let mut errors = Vec::new();

        for (response_result, critic) in responses.into_iter().zip(critics.iter()) {
            match response_result {
                Ok(response) => {
                    match parse_critique(&response, critic) {
                        Ok(critique) => critiques.push(critique),
                        Err(e) => {
                            // Try fallback parsing
                            match fallback_parse_critique(&response, critic) {
                                Ok(critique) => critiques.push(critique),
                                Err(_) => errors.push((critic.display_name.clone(), e)),
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push((critic.display_name.clone(), e));
                }
            }
        }

        // Check minimum critiques
        if critiques.len() < self.config.rounds.critique.min_critiques {
            let error_summary = errors
                .iter()
                .map(|(name, e)| format!("{}: {}", name, e))
                .collect::<Vec<_>>()
                .join("; ");

            return Err(ForgeError::Orchestration(format!(
                "Only {} of {} required critiques collected. Errors: {}",
                critiques.len(),
                self.config.rounds.critique.min_critiques,
                error_summary
            )));
        }

        // Normalize scores across critiques
        normalize_scores(&mut critiques);

        Ok(CritiqueRound {
            round_number,
            critiques,
            timestamp: tachikoma_common_core::Timestamp::now(),
        })
    }

    /// Collect critiques in parallel.
    async fn collect_parallel(
        &self,
        requests: Vec<(Participant, ModelRequest)>,
    ) -> Vec<ForgeResult<ModelResponse>> {
        let timeout_duration = Duration::from_secs(self.config.rounds.critique.timeout_secs);

        let futures = requests.into_iter().map(|(participant, request)| {
            let manager = self.participants;
            async move {
                timeout(
                    timeout_duration,
                    manager.send_request(&participant, request),
                )
                .await
                .map_err(|_| ForgeError::Timeout("Critique timed out".to_string()))?
            }
        });

        join_all(futures).await
    }

    /// Collect critiques sequentially.
    async fn collect_sequential(
        &self,
        requests: Vec<(Participant, ModelRequest)>,
    ) -> Vec<ForgeResult<ModelResponse>> {
        let timeout_duration = Duration::from_secs(self.config.rounds.critique.timeout_secs);
        let mut results = Vec::new();

        for (participant, request) in requests {
            let result = timeout(
                timeout_duration,
                self.participants.send_request(&participant, request),
            )
            .await
            .map_err(|_| ForgeError::Timeout("Critique timed out".to_string()))
            .and_then(|r| r);

            results.push(result);
        }

        results
    }
}

/// Parse a critique from a model response.
pub fn parse_critique(response: &ModelResponse, critic: &Participant) -> ForgeResult<Critique> {
    let content = &response.content;

    // Try to parse structured format
    let strengths = parse_strengths(content)?;
    let weaknesses = parse_weaknesses(content)?;
    let suggestions = parse_suggestions(content)?;
    let score = parse_score(content)?;

    Ok(Critique {
        critic: critic.clone(),
        strengths,
        weaknesses,
        suggestions,
        score,
        raw_content: content.clone(),
        tokens: response.tokens.clone(),
        duration_ms: response.duration_ms,
    })
}

/// Parse strengths from critique content.
fn parse_strengths(content: &str) -> ForgeResult<Vec<String>> {
    let section = extract_section(content, "Strengths")
        .or_else(|| extract_section(content, "strengths"))
        .ok_or_else(|| ForgeError::Parse("No strengths section found".to_string()))?;

    Ok(parse_bullet_list(&section))
}

/// Parse weaknesses from critique content.
fn parse_weaknesses(content: &str) -> ForgeResult<Vec<String>> {
    let section = extract_section(content, "Weaknesses")
        .or_else(|| extract_section(content, "weaknesses"))
        .ok_or_else(|| ForgeError::Parse("No weaknesses section found".to_string()))?;

    Ok(parse_bullet_list(&section))
}

/// Parse suggestions from critique content.
fn parse_suggestions(content: &str) -> ForgeResult<Vec<Suggestion>> {
    let mut suggestions = Vec::new();

    // Find all suggestion blocks
    let suggestion_pattern = regex::Regex::new(
        r"### Suggestion \d+\s*([\s\S]*?)(?=### Suggestion \d+|## Overall|$)"
    ).unwrap();

    for cap in suggestion_pattern.captures_iter(content) {
        if let Some(suggestion_text) = cap.get(1) {
            if let Some(suggestion) = parse_single_suggestion(suggestion_text.as_str()) {
                suggestions.push(suggestion);
            }
        }
    }

    // Fallback: look for numbered suggestions
    if suggestions.is_empty() {
        let numbered_pattern = regex::Regex::new(
            r"\d+\.\s*\*\*([^*]+)\*\*:?\s*(.+?)(?=\d+\.\s*\*\*|$)"
        ).unwrap();

        for cap in numbered_pattern.captures_iter(content) {
            let category = cap.get(1).map(|m| m.as_str()).unwrap_or("other");
            let text = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            if !text.is_empty() {
                suggestions.push(Suggestion {
                    section: None,
                    text: text.to_string(),
                    priority: 3,
                    category: parse_category(category),
                });
            }
        }
    }

    Ok(suggestions)
}

/// Parse a single suggestion block.
fn parse_single_suggestion(text: &str) -> Option<Suggestion> {
    let section = extract_field(text, "Section");
    let category_str = extract_field(text, "Category").unwrap_or_else(|| "other".to_string());
    let priority_str = extract_field(text, "Priority").unwrap_or_else(|| "3".to_string());
    let description = extract_field(text, "Description")?;

    let priority: u8 = priority_str.trim().parse().unwrap_or(3);
    let category = parse_category(&category_str);

    Some(Suggestion {
        section,
        text: description,
        priority,
        category,
    })
}

/// Parse score from critique content.
fn parse_score(content: &str) -> ForgeResult<u8> {
    // Look for "Score: XX" pattern
    let score_pattern = regex::Regex::new(r"\*\*Score\*\*:?\s*(\d+)").unwrap();

    if let Some(cap) = score_pattern.captures(content) {
        if let Some(score_match) = cap.get(1) {
            if let Ok(score) = score_match.as_str().parse::<u8>() {
                return Ok(score.min(100));
            }
        }
    }

    // Fallback pattern
    let fallback_pattern = regex::Regex::new(r"(?i)score[:\s]+(\d+)").unwrap();

    if let Some(cap) = fallback_pattern.captures(content) {
        if let Some(score_match) = cap.get(1) {
            if let Ok(score) = score_match.as_str().parse::<u8>() {
                return Ok(score.min(100));
            }
        }
    }

    Err(ForgeError::Parse("Could not parse score from critique".to_string()))
}

/// Fallback parser for less structured responses.
pub fn fallback_parse_critique(response: &ModelResponse, critic: &Participant) -> ForgeResult<Critique> {
    let content = &response.content;

    // More lenient parsing
    let lines: Vec<&str> = content.lines().collect();

    let mut strengths = Vec::new();
    let mut weaknesses = Vec::new();
    let mut suggestions = Vec::new();
    let mut score = 70u8; // Default score

    let mut current_section = "";

    for line in lines {
        let line = line.trim();

        // Detect section headers
        if line.to_lowercase().contains("strength") {
            current_section = "strengths";
            continue;
        }
        if line.to_lowercase().contains("weakness") || line.to_lowercase().contains("issue") {
            current_section = "weaknesses";
            continue;
        }
        if line.to_lowercase().contains("suggestion") || line.to_lowercase().contains("recommend") {
            current_section = "suggestions";
            continue;
        }

        // Parse score
        if line.to_lowercase().contains("score") {
            if let Some(num) = extract_number(line) {
                score = (num as u8).min(100);
            }
        }

        // Collect bullet points
        if line.starts_with('-') || line.starts_with('*') || line.starts_with("•") {
            let item = line.trim_start_matches(|c| c == '-' || c == '*' || c == '•' || c == ' ');

            match current_section {
                "strengths" => strengths.push(item.to_string()),
                "weaknesses" => weaknesses.push(item.to_string()),
                "suggestions" => suggestions.push(Suggestion {
                    section: None,
                    text: item.to_string(),
                    priority: 3,
                    category: SuggestionCategory::Other,
                }),
                _ => {}
            }
        }
    }

    // Require at least some content
    if strengths.is_empty() && weaknesses.is_empty() && suggestions.is_empty() {
        return Err(ForgeError::Parse(
            "Could not extract any critique content".to_string()
        ));
    }

    Ok(Critique {
        critic: critic.clone(),
        strengths,
        weaknesses,
        suggestions,
        score,
        raw_content: content.clone(),
        tokens: response.tokens.clone(),
        duration_ms: response.duration_ms,
    })
}

/// Extract a section from content.
fn extract_section(content: &str, header: &str) -> Option<String> {
    let pattern = format!(r"(?i)##\s*{}\s*\n([\s\S]*?)(?=##|$)", regex::escape(header));
    let re = regex::Regex::new(&pattern).ok()?;

    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Extract a field value.
fn extract_field(text: &str, field: &str) -> Option<String> {
    let pattern = format!(r"(?i)\*\*{}\*\*:?\s*(.+)", regex::escape(field));
    let re = regex::Regex::new(&pattern).ok()?;

    re.captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Parse a bullet list.
fn parse_bullet_list(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('-') || line.starts_with('*') || line.starts_with("•") {
                Some(
                    line.trim_start_matches(|c| c == '-' || c == '*' || c == '•' || c == ' ')
                        .to_string()
                )
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse a category string.
fn parse_category(s: &str) -> SuggestionCategory {
    match s.to_lowercase().trim() {
        "correctness" => SuggestionCategory::Correctness,
        "clarity" => SuggestionCategory::Clarity,
        "completeness" => SuggestionCategory::Completeness,
        "code_quality" | "code quality" => SuggestionCategory::CodeQuality,
        "architecture" => SuggestionCategory::Architecture,
        "performance" => SuggestionCategory::Performance,
        "security" => SuggestionCategory::Security,
        _ => SuggestionCategory::Other,
    }
}

/// Extract a number from text.
fn extract_number(text: &str) -> Option<u64> {
    let re = regex::Regex::new(r"(\d+)").ok()?;
    re.captures(text)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Normalize scores across critiques.
fn normalize_scores(critiques: &mut [Critique]) {
    if critiques.is_empty() {
        return;
    }

    // Calculate mean and std dev
    let scores: Vec<f64> = critiques.iter().map(|c| c.score as f64).collect();
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;
    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
    let std_dev = variance.sqrt();

    // Only normalize if there's significant variance
    if std_dev > 15.0 {
        for critique in critiques.iter_mut() {
            let z_score = (critique.score as f64 - mean) / std_dev;
            // Convert to 0-100 scale centered at 70
            let normalized = (70.0 + z_score * 15.0).clamp(0.0, 100.0);
            critique.score = normalized as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bullet_list() {
        let text = "- First item\n- Second item\n* Third item";
        let items = parse_bullet_list(text);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "First item");
    }

    #[test]
    fn test_parse_category() {
        assert_eq!(parse_category("correctness"), SuggestionCategory::Correctness);
        assert_eq!(parse_category("Code Quality"), SuggestionCategory::CodeQuality);
        assert_eq!(parse_category("unknown"), SuggestionCategory::Other);
    }

    #[test]
    fn test_normalize_scores() {
        let mut critiques = vec![
            Critique {
                score: 50,
                ..Default::default()
            },
            Critique {
                score: 90,
                ..Default::default()
            },
        ];

        normalize_scores(&mut critiques);

        // Scores should be closer to each other now
        assert!(critiques[0].score > 50);
        assert!(critiques[1].score < 90);
    }

    #[test]
    fn test_extract_section() {
        let content = "## Strengths\n- Good\n- Better\n## Weaknesses\n- Bad";
        let section = extract_section(content, "Strengths").unwrap();

        assert!(section.contains("Good"));
        assert!(!section.contains("Bad"));
    }
}
```

---

## Testing Requirements

1. Parallel collection completes within timeout
2. Structured critique parsing extracts all fields
3. Fallback parsing handles edge cases
4. Score normalization produces reasonable results
5. Partial failures don't block successful critiques
6. Minimum critique threshold is enforced

---

## Related Specs

- Depends on: [141-round2-critique-prompts.md](141-round2-critique-prompts.md)
- Next: [143-round3-synthesis-prompts.md](143-round3-synthesis-prompts.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
