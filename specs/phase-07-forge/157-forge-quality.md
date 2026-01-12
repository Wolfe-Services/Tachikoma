# 157 - Forge Quality Metrics

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 157
**Status:** Planned
**Dependencies:** 142-round2-critique-collect, 146-convergence-detect
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement quality metrics tracking for Forge sessions, providing visibility into output quality progression, critique scores, and overall session effectiveness.

---

## Acceptance Criteria

- [ ] Quality score tracking over rounds
- [ ] Critique score aggregation
- [ ] Quality dimension breakdown
- [ ] Improvement trend analysis
- [ ] Quality gates/thresholds
- [ ] Quality report generation

---

## Implementation Details

### 1. Quality Tracker (src/quality/tracker.rs)

```rust
//! Quality metrics tracking for Forge sessions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{Critique, ForgeRound, ForgeSession, SuggestionCategory};

/// Tracks quality metrics for a session.
#[derive(Debug, Clone, Default)]
pub struct QualityTracker {
    /// Quality snapshots by round.
    snapshots: Vec<QualitySnapshot>,
    /// Running averages.
    running_averages: HashMap<String, RunningAverage>,
}

/// A quality snapshot for a round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    /// Round number.
    pub round_number: usize,
    /// Overall quality score (0-100).
    pub overall_score: f64,
    /// Scores by dimension.
    pub dimension_scores: HashMap<QualityDimension, f64>,
    /// Critique summary.
    pub critique_summary: Option<CritiqueSummary>,
    /// Trend compared to previous.
    pub trend: QualityTrend,
}

/// Quality dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityDimension {
    /// Factual correctness.
    Correctness,
    /// Clarity of expression.
    Clarity,
    /// Completeness of coverage.
    Completeness,
    /// Code quality (if applicable).
    CodeQuality,
    /// Architecture/design quality.
    Architecture,
    /// Performance considerations.
    Performance,
    /// Security considerations.
    Security,
    /// Overall coherence.
    Coherence,
}

/// Summary of critiques for a round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueSummary {
    /// Number of critiques.
    pub critique_count: usize,
    /// Average score.
    pub average_score: f64,
    /// Score standard deviation.
    pub score_std_dev: f64,
    /// Total strengths identified.
    pub total_strengths: usize,
    /// Total weaknesses identified.
    pub total_weaknesses: usize,
    /// Total suggestions.
    pub total_suggestions: usize,
    /// High priority suggestions.
    pub high_priority_suggestions: usize,
}

/// Quality trend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityTrend {
    Improving,
    Stable,
    Declining,
    Unknown,
}

/// Running average calculation.
#[derive(Debug, Clone, Default)]
struct RunningAverage {
    sum: f64,
    count: usize,
}

impl RunningAverage {
    fn add(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }

    fn average(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.sum / self.count as f64 }
    }
}

impl QualityTracker {
    /// Create a new quality tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record quality from a critique round.
    pub fn record_critique_round(&mut self, round_number: usize, critiques: &[Critique]) {
        if critiques.is_empty() {
            return;
        }

        // Calculate critique summary
        let scores: Vec<f64> = critiques.iter().map(|c| c.score as f64).collect();
        let average_score = scores.iter().sum::<f64>() / scores.len() as f64;

        let variance: f64 = scores.iter()
            .map(|s| (s - average_score).powi(2))
            .sum::<f64>() / scores.len() as f64;
        let std_dev = variance.sqrt();

        let total_strengths: usize = critiques.iter().map(|c| c.strengths.len()).sum();
        let total_weaknesses: usize = critiques.iter().map(|c| c.weaknesses.len()).sum();
        let total_suggestions: usize = critiques.iter().map(|c| c.suggestions.len()).sum();
        let high_priority: usize = critiques.iter()
            .flat_map(|c| &c.suggestions)
            .filter(|s| s.priority <= 2)
            .count();

        let critique_summary = CritiqueSummary {
            critique_count: critiques.len(),
            average_score,
            score_std_dev: std_dev,
            total_strengths,
            total_weaknesses,
            total_suggestions,
            high_priority_suggestions: high_priority,
        };

        // Calculate dimension scores from suggestions
        let dimension_scores = self.calculate_dimension_scores(critiques);

        // Determine trend
        let trend = self.calculate_trend(average_score);

        // Update running averages
        self.running_averages.entry("overall".to_string()).or_default().add(average_score);
        for (dim, score) in &dimension_scores {
            self.running_averages.entry(format!("{:?}", dim)).or_default().add(*score);
        }

        let snapshot = QualitySnapshot {
            round_number,
            overall_score: average_score,
            dimension_scores,
            critique_summary: Some(critique_summary),
            trend,
        };

        self.snapshots.push(snapshot);
    }

    /// Calculate dimension scores from critiques.
    fn calculate_dimension_scores(&self, critiques: &[Critique]) -> HashMap<QualityDimension, f64> {
        let mut dimension_counts: HashMap<QualityDimension, (usize, usize)> = HashMap::new();

        for critique in critiques {
            // Count issues by category
            for suggestion in &critique.suggestions {
                let dim = match suggestion.category {
                    SuggestionCategory::Correctness => QualityDimension::Correctness,
                    SuggestionCategory::Clarity => QualityDimension::Clarity,
                    SuggestionCategory::Completeness => QualityDimension::Completeness,
                    SuggestionCategory::CodeQuality => QualityDimension::CodeQuality,
                    SuggestionCategory::Architecture => QualityDimension::Architecture,
                    SuggestionCategory::Performance => QualityDimension::Performance,
                    SuggestionCategory::Security => QualityDimension::Security,
                    SuggestionCategory::Other => continue,
                };

                let entry = dimension_counts.entry(dim).or_insert((0, 0));
                entry.0 += 1; // Issue count
                entry.1 += suggestion.priority as usize; // Priority sum
            }
        }

        // Convert to scores (fewer issues = higher score)
        let mut scores = HashMap::new();

        for dim in [
            QualityDimension::Correctness,
            QualityDimension::Clarity,
            QualityDimension::Completeness,
            QualityDimension::CodeQuality,
            QualityDimension::Architecture,
            QualityDimension::Performance,
            QualityDimension::Security,
        ] {
            let (issue_count, priority_sum) = dimension_counts.get(&dim).copied().unwrap_or((0, 0));

            // Score: 100 - (issues * 10), weighted by priority
            let weighted_issues = if issue_count > 0 {
                issue_count as f64 * (priority_sum as f64 / issue_count as f64)
            } else {
                0.0
            };

            let score = (100.0 - weighted_issues * 5.0).clamp(0.0, 100.0);
            scores.insert(dim, score);
        }

        scores
    }

    /// Calculate trend compared to previous snapshot.
    fn calculate_trend(&self, current_score: f64) -> QualityTrend {
        if self.snapshots.is_empty() {
            return QualityTrend::Unknown;
        }

        let previous = self.snapshots.last().unwrap().overall_score;
        let diff = current_score - previous;

        if diff > 5.0 {
            QualityTrend::Improving
        } else if diff < -5.0 {
            QualityTrend::Declining
        } else {
            QualityTrend::Stable
        }
    }

    /// Get the latest snapshot.
    pub fn latest_snapshot(&self) -> Option<&QualitySnapshot> {
        self.snapshots.last()
    }

    /// Get all snapshots.
    pub fn all_snapshots(&self) -> &[QualitySnapshot] {
        &self.snapshots
    }

    /// Get quality report.
    pub fn generate_report(&self) -> QualityReport {
        let latest = self.latest_snapshot();

        let overall_trend = if self.snapshots.len() >= 3 {
            let recent: Vec<f64> = self.snapshots.iter()
                .rev()
                .take(3)
                .map(|s| s.overall_score)
                .collect();

            let improving = recent.windows(2).all(|w| w[0] >= w[1]);
            let declining = recent.windows(2).all(|w| w[0] <= w[1]);

            if improving { QualityTrend::Improving }
            else if declining { QualityTrend::Declining }
            else { QualityTrend::Stable }
        } else {
            QualityTrend::Unknown
        };

        let average_overall = self.running_averages.get("overall")
            .map(|ra| ra.average())
            .unwrap_or(0.0);

        let weakest_dimension = self.find_weakest_dimension();
        let strongest_dimension = self.find_strongest_dimension();

        QualityReport {
            current_score: latest.map(|s| s.overall_score).unwrap_or(0.0),
            average_score: average_overall,
            snapshots_count: self.snapshots.len(),
            overall_trend,
            weakest_dimension,
            strongest_dimension,
            improvement_needed: self.calculate_improvement_needed(),
        }
    }

    /// Find the weakest dimension across all snapshots.
    fn find_weakest_dimension(&self) -> Option<(QualityDimension, f64)> {
        let mut dimension_totals: HashMap<QualityDimension, (f64, usize)> = HashMap::new();

        for snapshot in &self.snapshots {
            for (dim, score) in &snapshot.dimension_scores {
                let entry = dimension_totals.entry(*dim).or_insert((0.0, 0));
                entry.0 += score;
                entry.1 += 1;
            }
        }

        dimension_totals.iter()
            .map(|(dim, (sum, count))| (*dim, sum / *count as f64))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    /// Find the strongest dimension.
    fn find_strongest_dimension(&self) -> Option<(QualityDimension, f64)> {
        let mut dimension_totals: HashMap<QualityDimension, (f64, usize)> = HashMap::new();

        for snapshot in &self.snapshots {
            for (dim, score) in &snapshot.dimension_scores {
                let entry = dimension_totals.entry(*dim).or_insert((0.0, 0));
                entry.0 += score;
                entry.1 += 1;
            }
        }

        dimension_totals.iter()
            .map(|(dim, (sum, count))| (*dim, sum / *count as f64))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    /// Calculate areas needing improvement.
    fn calculate_improvement_needed(&self) -> Vec<String> {
        let mut needs_improvement = Vec::new();

        if let Some(latest) = self.latest_snapshot() {
            for (dim, score) in &latest.dimension_scores {
                if *score < 70.0 {
                    needs_improvement.push(format!("{:?}: {:.0}/100", dim, score));
                }
            }
        }

        needs_improvement
    }

    /// Check if quality meets threshold.
    pub fn meets_quality_threshold(&self, threshold: f64) -> bool {
        self.latest_snapshot()
            .map(|s| s.overall_score >= threshold)
            .unwrap_or(false)
    }
}

/// Quality report summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Current quality score.
    pub current_score: f64,
    /// Average across all rounds.
    pub average_score: f64,
    /// Number of quality snapshots.
    pub snapshots_count: usize,
    /// Overall trend.
    pub overall_trend: QualityTrend,
    /// Weakest dimension.
    pub weakest_dimension: Option<(QualityDimension, f64)>,
    /// Strongest dimension.
    pub strongest_dimension: Option<(QualityDimension, f64)>,
    /// Areas needing improvement.
    pub improvement_needed: Vec<String>,
}

impl QualityReport {
    /// Format as markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::from("## Quality Report\n\n");

        md.push_str(&format!("**Current Score:** {:.0}/100\n\n", self.current_score));
        md.push_str(&format!("**Average Score:** {:.0}/100\n\n", self.average_score));
        md.push_str(&format!("**Trend:** {:?}\n\n", self.overall_trend));

        if let Some((dim, score)) = &self.strongest_dimension {
            md.push_str(&format!("**Strongest Area:** {:?} ({:.0}/100)\n\n", dim, score));
        }

        if let Some((dim, score)) = &self.weakest_dimension {
            md.push_str(&format!("**Weakest Area:** {:?} ({:.0}/100)\n\n", dim, score));
        }

        if !self.improvement_needed.is_empty() {
            md.push_str("**Areas Needing Improvement:**\n");
            for area in &self.improvement_needed {
                md.push_str(&format!("- {}\n", area));
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Suggestion;

    #[test]
    fn test_quality_tracking() {
        let mut tracker = QualityTracker::new();

        let critiques = vec![
            Critique {
                critic: crate::Participant::claude_sonnet(),
                strengths: vec!["Good structure".to_string()],
                weaknesses: vec!["Missing tests".to_string()],
                suggestions: vec![
                    Suggestion {
                        section: None,
                        text: "Add tests".to_string(),
                        priority: 2,
                        category: SuggestionCategory::Completeness,
                    }
                ],
                score: 75,
                raw_content: String::new(),
                tokens: Default::default(),
                duration_ms: 0,
            }
        ];

        tracker.record_critique_round(1, &critiques);

        assert_eq!(tracker.snapshots.len(), 1);
        assert_eq!(tracker.latest_snapshot().unwrap().overall_score, 75.0);
    }
}
```

---

## Testing Requirements

1. Quality scores calculate correctly
2. Dimension scores reflect suggestions
3. Trends detect improvement/decline
4. Reports summarize correctly
5. Thresholds work as expected
6. Multiple rounds track properly

---

## Related Specs

- Depends on: [142-round2-critique-collect.md](142-round2-critique-collect.md)
- Depends on: [146-convergence-detect.md](146-convergence-detect.md)
- Next: [158-forge-templates.md](158-forge-templates.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md), [154-forge-output.md](154-forge-output.md)
