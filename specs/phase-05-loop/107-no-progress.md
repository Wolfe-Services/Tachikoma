# 107 - No Progress Detection

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 107
**Status:** Planned
**Dependencies:** 104-stop-conditions, 097-loop-iteration
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement no-progress detection for the Ralph Loop - identifying when iterations are not making meaningful progress toward the goal, enabling early termination of unproductive loops.

---

## Acceptance Criteria

- [ ] Define progress indicators
- [ ] Track progress across iterations
- [ ] Configurable progress metrics
- [ ] No-progress streak counter
- [ ] Integration with stop conditions
- [ ] Multiple detection strategies
- [ ] Progress velocity tracking
- [ ] Progress reporting

---

## Implementation Details

### 1. Progress Types (src/progress/types.rs)

```rust
//! Progress detection types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Indicators of progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressIndicators {
    /// Files were modified.
    pub files_modified: bool,
    /// Files were created.
    pub files_created: bool,
    /// Tests improved (more passing or fewer failing).
    pub tests_improved: bool,
    /// New tests were added.
    pub tests_added: bool,
    /// Build succeeded when it was failing.
    pub build_fixed: bool,
    /// Linting issues reduced.
    pub lint_improved: bool,
    /// Code coverage increased.
    pub coverage_increased: bool,
    /// Custom progress markers found in output.
    pub custom_markers: Vec<String>,
    /// Numeric metrics.
    pub metrics: HashMap<String, f64>,
}

impl ProgressIndicators {
    /// Check if any progress was made.
    pub fn any_progress(&self) -> bool {
        self.files_modified
            || self.files_created
            || self.tests_improved
            || self.tests_added
            || self.build_fixed
            || self.lint_improved
            || self.coverage_increased
            || !self.custom_markers.is_empty()
    }

    /// Calculate a progress score (0.0 - 1.0).
    pub fn progress_score(&self) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Weighted indicators
        let weights = [
            (self.files_modified, 0.3),
            (self.files_created, 0.2),
            (self.tests_improved, 0.4),
            (self.tests_added, 0.2),
            (self.build_fixed, 0.5),
            (self.lint_improved, 0.1),
            (self.coverage_increased, 0.2),
        ];

        for (indicator, weight) in weights {
            max_score += weight;
            if indicator {
                score += weight;
            }
        }

        // Custom markers
        if !self.custom_markers.is_empty() {
            score += 0.3;
            max_score += 0.3;
        }

        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }
}

impl Default for ProgressIndicators {
    fn default() -> Self {
        Self {
            files_modified: false,
            files_created: false,
            tests_improved: false,
            tests_added: false,
            build_fixed: false,
            lint_improved: false,
            coverage_increased: false,
            custom_markers: vec![],
            metrics: HashMap::new(),
        }
    }
}

/// Configuration for progress detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressConfig {
    /// Detection strategy.
    pub strategy: ProgressStrategy,
    /// Custom progress markers to look for.
    pub custom_markers: Vec<String>,
    /// Minimum progress score to consider progress made.
    pub min_progress_score: f64,
    /// Track metrics history.
    pub track_metrics: bool,
    /// Metrics to track.
    pub tracked_metrics: Vec<MetricConfig>,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            strategy: ProgressStrategy::Combined,
            custom_markers: vec![
                "[PROGRESS]".to_string(),
                "DONE:".to_string(),
                "Completed:".to_string(),
            ],
            min_progress_score: 0.1,
            track_metrics: true,
            tracked_metrics: vec![],
        }
    }
}

/// Progress detection strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStrategy {
    /// Only consider file changes.
    FileChanges,
    /// Only consider test improvements.
    TestResults,
    /// Custom markers only.
    CustomMarkers,
    /// Combine all indicators.
    Combined,
    /// Use metric velocity.
    MetricVelocity,
}

/// Configuration for a tracked metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfig {
    /// Metric name.
    pub name: String,
    /// Pattern to extract value from output.
    pub pattern: String,
    /// Direction that indicates progress.
    pub direction: MetricDirection,
    /// Weight in combined score.
    pub weight: f64,
}

/// Direction of metric improvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    /// Higher is better.
    Higher,
    /// Lower is better.
    Lower,
}

/// Progress state over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressState {
    /// Iterations since last progress.
    pub iterations_since_progress: u32,
    /// Last iteration that made progress.
    pub last_progress_iteration: Option<u32>,
    /// Progress velocity (rate of progress).
    pub velocity: f64,
    /// Recent progress scores.
    pub recent_scores: Vec<f64>,
    /// Metric history.
    pub metric_history: HashMap<String, Vec<MetricSample>>,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            iterations_since_progress: 0,
            last_progress_iteration: None,
            velocity: 0.0,
            recent_scores: vec![],
            metric_history: HashMap::new(),
        }
    }
}

/// A metric sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    /// Iteration number.
    pub iteration: u32,
    /// Value.
    pub value: f64,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Report on progress state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    /// Whether progress was made this iteration.
    pub made_progress: bool,
    /// Progress score.
    pub score: f64,
    /// Indicators that were positive.
    pub positive_indicators: Vec<String>,
    /// Current no-progress streak.
    pub no_progress_streak: u32,
    /// Current velocity.
    pub velocity: f64,
    /// Recommendation.
    pub recommendation: ProgressRecommendation,
}

/// Recommendation based on progress analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressRecommendation {
    /// Continue, good progress.
    Continue,
    /// Continue but monitor.
    Monitor,
    /// Consider changing approach.
    ConsiderChange,
    /// Stop due to no progress.
    Stop,
}
```

### 2. Progress Detector (src/progress/detector.rs)

```rust
//! Progress detection implementation.

use super::types::{
    MetricDirection, MetricSample, ProgressConfig, ProgressIndicators, ProgressRecommendation,
    ProgressReport, ProgressState, ProgressStrategy,
};
use crate::error::LoopResult;
use crate::iteration::IterationResult;
use crate::testing::TestSummary;

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, trace};

/// Detects progress across iterations.
pub struct ProgressDetector {
    /// Configuration.
    config: ProgressConfig,
    /// Current progress state.
    state: RwLock<ProgressState>,
    /// Previous test summary for comparison.
    previous_tests: RwLock<Option<TestSummary>>,
    /// Previous file hashes for comparison.
    previous_files: RwLock<HashMap<String, String>>,
}

impl ProgressDetector {
    /// Create a new progress detector.
    pub fn new(config: ProgressConfig) -> Self {
        Self {
            config,
            state: RwLock::new(ProgressState::default()),
            previous_tests: RwLock::new(None),
            previous_files: RwLock::new(HashMap::new()),
        }
    }

    /// Detect progress from iteration result.
    pub async fn detect(
        &self,
        result: &IterationResult,
        test_summary: Option<&TestSummary>,
        iteration: u32,
    ) -> LoopResult<ProgressReport> {
        let indicators = self.collect_indicators(result, test_summary).await;
        let score = self.calculate_score(&indicators);

        // Update state
        let (no_progress_streak, velocity) = self.update_state(score, iteration).await;

        // Generate report
        let made_progress = score >= self.config.min_progress_score;
        let positive_indicators = self.get_positive_indicators(&indicators);
        let recommendation = self.get_recommendation(no_progress_streak, velocity);

        // Store current state for next comparison
        if let Some(tests) = test_summary {
            *self.previous_tests.write().await = Some(tests.clone());
        }

        Ok(ProgressReport {
            made_progress,
            score,
            positive_indicators,
            no_progress_streak,
            velocity,
            recommendation,
        })
    }

    /// Collect all progress indicators.
    async fn collect_indicators(
        &self,
        result: &IterationResult,
        test_summary: Option<&TestSummary>,
    ) -> ProgressIndicators {
        let mut indicators = ProgressIndicators::default();

        // File changes
        indicators.files_modified = !result.files_modified.is_empty();
        indicators.files_created = !result.files_created.is_empty();

        // Test improvements
        if let Some(tests) = test_summary {
            let prev = self.previous_tests.read().await;
            if let Some(prev_tests) = prev.as_ref() {
                indicators.tests_improved = tests.passing > prev_tests.passing
                    || tests.failing < prev_tests.failing;
                indicators.tests_added = tests.total_tests > prev_tests.total_tests;
            }
        }

        // Custom markers in output
        for marker in &self.config.custom_markers {
            if result.stdout.contains(marker) {
                indicators.custom_markers.push(marker.clone());
            }
        }

        // Extract tracked metrics
        for metric_config in &self.config.tracked_metrics {
            if let Some(value) = self.extract_metric(&result.stdout, &metric_config.pattern) {
                indicators.metrics.insert(metric_config.name.clone(), value);
            }
        }

        indicators
    }

    /// Calculate progress score based on strategy.
    fn calculate_score(&self, indicators: &ProgressIndicators) -> f64 {
        match self.config.strategy {
            ProgressStrategy::FileChanges => {
                if indicators.files_modified || indicators.files_created {
                    1.0
                } else {
                    0.0
                }
            }
            ProgressStrategy::TestResults => {
                if indicators.tests_improved || indicators.tests_added {
                    1.0
                } else {
                    0.0
                }
            }
            ProgressStrategy::CustomMarkers => {
                if !indicators.custom_markers.is_empty() {
                    1.0
                } else {
                    0.0
                }
            }
            ProgressStrategy::Combined => indicators.progress_score(),
            ProgressStrategy::MetricVelocity => {
                // Handled separately in update_state
                indicators.progress_score()
            }
        }
    }

    /// Update progress state.
    async fn update_state(&self, score: f64, iteration: u32) -> (u32, f64) {
        let mut state = self.state.write().await;

        // Update recent scores
        state.recent_scores.push(score);
        if state.recent_scores.len() > 10 {
            state.recent_scores.remove(0);
        }

        // Calculate velocity (change in average score)
        let velocity = if state.recent_scores.len() >= 3 {
            let recent_avg: f64 = state.recent_scores.iter().rev().take(3).sum::<f64>() / 3.0;
            let older_avg: f64 = state.recent_scores.iter().take(3).sum::<f64>() / 3.0;
            recent_avg - older_avg
        } else {
            0.0
        };
        state.velocity = velocity;

        // Update no-progress streak
        if score >= self.config.min_progress_score {
            state.iterations_since_progress = 0;
            state.last_progress_iteration = Some(iteration);
        } else {
            state.iterations_since_progress += 1;
        }

        (state.iterations_since_progress, velocity)
    }

    /// Get positive indicators as strings.
    fn get_positive_indicators(&self, indicators: &ProgressIndicators) -> Vec<String> {
        let mut positive = Vec::new();

        if indicators.files_modified {
            positive.push("files_modified".to_string());
        }
        if indicators.files_created {
            positive.push("files_created".to_string());
        }
        if indicators.tests_improved {
            positive.push("tests_improved".to_string());
        }
        if indicators.tests_added {
            positive.push("tests_added".to_string());
        }
        if indicators.build_fixed {
            positive.push("build_fixed".to_string());
        }
        if indicators.lint_improved {
            positive.push("lint_improved".to_string());
        }
        if indicators.coverage_increased {
            positive.push("coverage_increased".to_string());
        }

        for marker in &indicators.custom_markers {
            positive.push(format!("marker:{}", marker));
        }

        positive
    }

    /// Get recommendation based on progress state.
    fn get_recommendation(&self, streak: u32, velocity: f64) -> ProgressRecommendation {
        // Velocity-based recommendations
        if velocity > 0.1 {
            return ProgressRecommendation::Continue;
        }

        // Streak-based recommendations
        match streak {
            0..=2 => ProgressRecommendation::Continue,
            3..=5 => ProgressRecommendation::Monitor,
            6..=10 => ProgressRecommendation::ConsiderChange,
            _ => ProgressRecommendation::Stop,
        }
    }

    /// Extract metric value from output using pattern.
    fn extract_metric(&self, output: &str, pattern: &str) -> Option<f64> {
        regex::Regex::new(pattern)
            .ok()
            .and_then(|re| re.captures(output))
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Get current state.
    pub async fn get_state(&self) -> ProgressState {
        self.state.read().await.clone()
    }

    /// Get iterations since progress.
    pub async fn iterations_since_progress(&self) -> u32 {
        self.state.read().await.iterations_since_progress
    }

    /// Reset progress tracking.
    pub async fn reset(&self) {
        *self.state.write().await = ProgressState::default();
        *self.previous_tests.write().await = None;
        *self.previous_files.write().await = HashMap::new();
    }

    /// Record a metric value.
    pub async fn record_metric(&self, name: &str, value: f64, iteration: u32) {
        let mut state = self.state.write().await;
        let samples = state.metric_history.entry(name.to_string()).or_insert_with(Vec::new);
        samples.push(MetricSample {
            iteration,
            value,
            timestamp: chrono::Utc::now(),
        });

        // Keep last 50 samples
        if samples.len() > 50 {
            samples.remove(0);
        }
    }

    /// Get metric trend.
    pub async fn get_metric_trend(&self, name: &str) -> Option<MetricTrend> {
        let state = self.state.read().await;
        let samples = state.metric_history.get(name)?;

        if samples.len() < 3 {
            return None;
        }

        let values: Vec<f64> = samples.iter().map(|s| s.value).collect();
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let recent_avg = values.iter().rev().take(3).sum::<f64>() / 3.0;

        let direction = if recent_avg > avg * 1.05 {
            TrendDirection::Increasing
        } else if recent_avg < avg * 0.95 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        Some(MetricTrend {
            name: name.to_string(),
            current: *values.last()?,
            average: avg,
            direction,
        })
    }
}

/// Trend analysis for a metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrend {
    pub name: String,
    pub current: f64,
    pub average: f64,
    pub direction: TrendDirection,
}

/// Direction of trend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

use serde::{Deserialize, Serialize};
```

### 3. Module Root (src/progress/mod.rs)

```rust
//! Progress detection for the loop.

pub mod detector;
pub mod types;

pub use detector::{MetricTrend, ProgressDetector, TrendDirection};
pub use types::{
    MetricConfig, MetricDirection, MetricSample, ProgressConfig, ProgressIndicators,
    ProgressRecommendation, ProgressReport, ProgressState, ProgressStrategy,
};
```

---

## Testing Requirements

1. File changes detected as progress
2. Test improvements detected as progress
3. Custom markers detected in output
4. No-progress streak increments correctly
5. Velocity calculation is accurate
6. Recommendations match streak thresholds
7. Metric extraction works
8. Reset clears all state

---

## Related Specs

- Depends on: [104-stop-conditions.md](104-stop-conditions.md)
- Depends on: [097-loop-iteration.md](097-loop-iteration.md)
- Next: [108-loop-metrics.md](108-loop-metrics.md)
- Related: [106-test-failure-tracking.md](106-test-failure-tracking.md)
