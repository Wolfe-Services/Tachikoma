# 147 - Convergence Metrics

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 147
**Status:** Planned
**Dependencies:** 146-convergence-detect
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Define and implement the metrics used for measuring convergence, including agreement scoring, change velocity, semantic similarity, and custom metric support.

---

## Acceptance Criteria

- [ ] Agreement score calculation
- [ ] Change velocity tracking
- [ ] Issue count metrics
- [ ] Semantic similarity measurement
- [ ] Section stability tracking
- [ ] Custom metric extensibility
- [ ] Metric visualization data

---

## Implementation Details

### 1. Metrics System (src/convergence/metrics.rs)

```rust
//! Convergence metrics implementation.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{Critique, ForgeRound, ForgeSession, SuggestionCategory};

/// A convergence metric calculator.
pub trait MetricCalculator: Send + Sync {
    /// Name of the metric.
    fn name(&self) -> &str;

    /// Calculate the metric value (0.0 to 1.0).
    fn calculate(&self, session: &ForgeSession) -> f64;

    /// Get the weight for this metric in overall score.
    fn weight(&self) -> f64;

    /// Description of what the metric measures.
    fn description(&self) -> &str;
}

/// Registry of metric calculators.
pub struct MetricsRegistry {
    calculators: HashMap<String, Box<dyn MetricCalculator>>,
}

impl MetricsRegistry {
    /// Create with default metrics.
    pub fn default_metrics() -> Self {
        let mut registry = Self {
            calculators: HashMap::new(),
        };

        registry.register(Box::new(AgreementScoreMetric));
        registry.register(Box::new(ChangeVelocityMetric::new()));
        registry.register(Box::new(IssueCountMetric));
        registry.register(Box::new(SemanticSimilarityMetric::new()));
        registry.register(Box::new(SectionStabilityMetric));
        registry.register(Box::new(QualityTrendMetric));

        registry
    }

    /// Register a metric calculator.
    pub fn register(&mut self, calculator: Box<dyn MetricCalculator>) {
        self.calculators.insert(calculator.name().to_string(), calculator);
    }

    /// Calculate all metrics.
    pub fn calculate_all(&self, session: &ForgeSession) -> MetricsSnapshot {
        let mut values = HashMap::new();
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (name, calc) in &self.calculators {
            let value = calc.calculate(session);
            let weight = calc.weight();

            values.insert(name.clone(), MetricValue {
                value,
                weight,
                description: calc.description().to_string(),
            });

            weighted_sum += value * weight;
            total_weight += weight;
        }

        let overall = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        MetricsSnapshot {
            overall,
            metrics: values,
            round_number: session.rounds.len(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// A snapshot of metric values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Overall convergence score.
    pub overall: f64,
    /// Individual metric values.
    pub metrics: HashMap<String, MetricValue>,
    /// Round number this was calculated for.
    pub round_number: usize,
    /// When the snapshot was taken.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A single metric value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// The metric value (0.0-1.0).
    pub value: f64,
    /// Weight in overall score.
    pub weight: f64,
    /// Description.
    pub description: String,
}

// --- Built-in Metric Implementations ---

/// Agreement score based on critique scores.
pub struct AgreementScoreMetric;

impl MetricCalculator for AgreementScoreMetric {
    fn name(&self) -> &str {
        "agreement_score"
    }

    fn calculate(&self, session: &ForgeSession) -> f64 {
        let critiques = get_recent_critiques(session);

        if critiques.is_empty() {
            return 0.5; // Neutral when no critiques
        }

        // Calculate average score
        let avg_score: f64 = critiques.iter()
            .map(|c| c.score as f64)
            .sum::<f64>() / critiques.len() as f64;

        // Calculate variance
        let variance: f64 = critiques.iter()
            .map(|c| (c.score as f64 - avg_score).powi(2))
            .sum::<f64>() / critiques.len() as f64;

        let std_dev = variance.sqrt();

        // Higher score, lower variance = better agreement
        // Normalize to 0-1
        let score_component = avg_score / 100.0;
        let agreement_component = 1.0 - (std_dev / 30.0).min(1.0);

        (score_component * 0.6) + (agreement_component * 0.4)
    }

    fn weight(&self) -> f64 {
        0.30
    }

    fn description(&self) -> &str {
        "Measures how much critics agree on the quality of the draft"
    }
}

/// Change velocity between rounds.
pub struct ChangeVelocityMetric {
    /// Cache of previous content for comparison.
    content_history: Vec<String>,
}

impl ChangeVelocityMetric {
    pub fn new() -> Self {
        Self {
            content_history: Vec::new(),
        }
    }
}

impl MetricCalculator for ChangeVelocityMetric {
    fn name(&self) -> &str {
        "change_velocity"
    }

    fn calculate(&self, session: &ForgeSession) -> f64 {
        let contents: Vec<&str> = session.rounds.iter()
            .filter_map(|r| get_round_content(r))
            .collect();

        if contents.len() < 2 {
            return 0.0;
        }

        // Calculate change between last two versions
        let last = contents.last().unwrap();
        let prev = contents.get(contents.len() - 2).unwrap();

        let similarity = jaccard_similarity(last, prev);

        // High similarity = low change = good for convergence
        similarity
    }

    fn weight(&self) -> f64 {
        0.25
    }

    fn description(&self) -> &str {
        "Measures how much content is changing between rounds (less change = converging)"
    }
}

/// Issue count from critiques.
pub struct IssueCountMetric;

impl MetricCalculator for IssueCountMetric {
    fn name(&self) -> &str {
        "issue_count"
    }

    fn calculate(&self, session: &ForgeSession) -> f64 {
        let critiques = get_recent_critiques(session);

        if critiques.is_empty() {
            return 0.5;
        }

        // Count total issues
        let total_issues: usize = critiques.iter()
            .map(|c| {
                c.weaknesses.len()
                    + c.suggestions.iter()
                        .filter(|s| s.priority <= 2) // Only high-priority suggestions
                        .count()
            })
            .sum();

        // Also count critical issues
        let critical_issues: usize = critiques.iter()
            .map(|c| {
                c.suggestions.iter()
                    .filter(|s| {
                        s.priority == 1
                            || matches!(s.category, SuggestionCategory::Correctness | SuggestionCategory::Security)
                    })
                    .count()
            })
            .sum();

        // Score: 0 issues = 1.0, many issues = 0.0
        let issue_score = 1.0 - (total_issues as f64 / 15.0).min(1.0);
        let critical_penalty = (critical_issues as f64 * 0.1).min(0.3);

        (issue_score - critical_penalty).max(0.0)
    }

    fn weight(&self) -> f64 {
        0.25
    }

    fn description(&self) -> &str {
        "Measures the number of remaining issues identified by critics"
    }
}

/// Semantic similarity using simple n-gram analysis.
pub struct SemanticSimilarityMetric {
    /// Window size for comparison.
    window_size: usize,
}

impl SemanticSimilarityMetric {
    pub fn new() -> Self {
        Self { window_size: 3 }
    }
}

impl MetricCalculator for SemanticSimilarityMetric {
    fn name(&self) -> &str {
        "semantic_similarity"
    }

    fn calculate(&self, session: &ForgeSession) -> f64 {
        let contents: Vec<&str> = session.rounds.iter()
            .filter_map(|r| get_round_content(r))
            .rev()
            .take(self.window_size)
            .collect();

        if contents.len() < 2 {
            return 0.0;
        }

        // Calculate pairwise similarities
        let mut total_sim = 0.0;
        let mut count = 0;

        for i in 0..contents.len() {
            for j in (i + 1)..contents.len() {
                total_sim += ngram_similarity(contents[i], contents[j], 3);
                count += 1;
            }
        }

        if count > 0 { total_sim / count as f64 } else { 0.0 }
    }

    fn weight(&self) -> f64 {
        0.10
    }

    fn description(&self) -> &str {
        "Measures semantic similarity between recent versions"
    }
}

/// Section stability metric.
pub struct SectionStabilityMetric;

impl MetricCalculator for SectionStabilityMetric {
    fn name(&self) -> &str {
        "section_stability"
    }

    fn calculate(&self, session: &ForgeSession) -> f64 {
        let contents: Vec<&str> = session.rounds.iter()
            .filter_map(|r| get_round_content(r))
            .rev()
            .take(3)
            .collect();

        if contents.len() < 2 {
            return 0.5;
        }

        // Extract and compare section headers
        let section_sets: Vec<Vec<String>> = contents.iter()
            .map(|c| extract_section_headers(c))
            .collect();

        // Check stability
        let reference = &section_sets[0];
        let mut stability_scores = Vec::new();

        for other in &section_sets[1..] {
            let common = reference.iter()
                .filter(|h| other.contains(h))
                .count();
            let total = reference.len().max(other.len());

            if total > 0 {
                stability_scores.push(common as f64 / total as f64);
            }
        }

        if stability_scores.is_empty() {
            0.5
        } else {
            stability_scores.iter().sum::<f64>() / stability_scores.len() as f64
        }
    }

    fn weight(&self) -> f64 {
        0.10
    }

    fn description(&self) -> &str {
        "Measures how stable the document structure is across rounds"
    }
}

/// Quality trend metric.
pub struct QualityTrendMetric;

impl MetricCalculator for QualityTrendMetric {
    fn name(&self) -> &str {
        "quality_trend"
    }

    fn calculate(&self, session: &ForgeSession) -> f64 {
        // Get scores from recent critique rounds
        let scores: Vec<f64> = session.rounds.iter()
            .filter_map(|r| match r {
                ForgeRound::Critique(c) => {
                    let avg = c.critiques.iter()
                        .map(|crit| crit.score as f64)
                        .sum::<f64>() / c.critiques.len().max(1) as f64;
                    Some(avg)
                }
                _ => None,
            })
            .collect();

        if scores.len() < 2 {
            return 0.5;
        }

        // Calculate trend (are scores improving?)
        let mut improvements = 0;
        for window in scores.windows(2) {
            if window[1] > window[0] {
                improvements += 1;
            }
        }

        let improvement_ratio = improvements as f64 / (scores.len() - 1) as f64;

        // Also consider final score level
        let final_score = scores.last().unwrap_or(&50.0) / 100.0;

        (improvement_ratio * 0.4) + (final_score * 0.6)
    }

    fn weight(&self) -> f64 {
        0.15
    }

    fn description(&self) -> &str {
        "Measures whether quality is improving over rounds"
    }
}

// --- Helper Functions ---

/// Get recent critiques from session.
fn get_recent_critiques(session: &ForgeSession) -> Vec<&Critique> {
    session.rounds.iter().rev()
        .find_map(|r| match r {
            ForgeRound::Critique(c) => Some(&c.critiques),
            _ => None,
        })
        .map(|c| c.iter().collect())
        .unwrap_or_default()
}

/// Get content from a round.
fn get_round_content(round: &ForgeRound) -> Option<&str> {
    match round {
        ForgeRound::Draft(d) => Some(&d.content),
        ForgeRound::Synthesis(s) => Some(&s.merged_content),
        ForgeRound::Refinement(r) => Some(&r.refined_content),
        _ => None,
    }
}

/// Calculate Jaccard similarity between two texts.
fn jaccard_similarity(a: &str, b: &str) -> f64 {
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

/// Calculate n-gram similarity.
fn ngram_similarity(a: &str, b: &str, n: usize) -> f64 {
    fn get_ngrams(text: &str, n: usize) -> std::collections::HashSet<String> {
        let words: Vec<_> = text.to_lowercase().split_whitespace().collect();
        if words.len() < n {
            return std::collections::HashSet::new();
        }

        words.windows(n)
            .map(|w| w.join(" "))
            .collect()
    }

    let a_ngrams = get_ngrams(a, n);
    let b_ngrams = get_ngrams(b, n);

    let intersection = a_ngrams.intersection(&b_ngrams).count();
    let union = a_ngrams.union(&b_ngrams).count();

    if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
}

/// Extract section headers from content.
fn extract_section_headers(content: &str) -> Vec<String> {
    content.lines()
        .filter(|l| l.starts_with('#'))
        .map(|l| l.trim_start_matches('#').trim().to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_similarity() {
        let a = "the quick brown fox";
        let b = "the quick red fox";

        let sim = jaccard_similarity(a, b);
        assert!(sim > 0.5);
        assert!(sim < 1.0);
    }

    #[test]
    fn test_ngram_similarity() {
        let a = "the quick brown fox jumps";
        let b = "the quick brown dog runs";

        let sim = ngram_similarity(a, b, 2);
        assert!(sim > 0.0);
        assert!(sim < 1.0);
    }

    #[test]
    fn test_extract_section_headers() {
        let content = "# Title\n\nSome text\n\n## Section 1\n\nMore text\n\n## Section 2";
        let headers = extract_section_headers(content);

        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0], "title");
        assert_eq!(headers[1], "section 1");
    }
}
```

---

## Testing Requirements

1. All default metrics produce valid scores (0-1)
2. Agreement score handles varying critique counts
3. Change velocity detects stabilization
4. Issue count scales appropriately
5. Section stability detects structural changes
6. Custom metrics can be registered

---

## Related Specs

- Depends on: [146-convergence-detect.md](146-convergence-detect.md)
- Next: [148-decision-logging.md](148-decision-logging.md)
- Used by: [146-convergence-detect.md](146-convergence-detect.md)
