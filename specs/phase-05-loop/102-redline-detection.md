# 102 - Context Redline Detection

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 102
**Status:** Planned
**Dependencies:** 100-session-management, 101-fresh-context
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement context redline detection for the Ralph Loop - monitoring context window usage and detecting when a session is approaching capacity, triggering automatic reboots before degradation occurs.

---

## Acceptance Criteria

- [ ] Monitor context window usage percentage
- [ ] Configurable redline thresholds
- [ ] Multiple detection strategies
- [ ] Warning events before redline
- [ ] Output parsing for usage indicators
- [ ] Token estimation heuristics
- [ ] Degradation pattern detection
- [ ] Integration with auto-reboot

---

## Implementation Details

### 1. Redline Types (src/redline/types.rs)

```rust
//! Redline detection types.

use serde::{Deserialize, Serialize};

/// Context window usage level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextLevel {
    /// Plenty of context available (0-50%).
    Low,
    /// Moderate usage (50-70%).
    Medium,
    /// High usage, should plan for reboot (70-85%).
    High,
    /// Critical, reboot recommended (85-95%).
    Warning,
    /// At redline, immediate reboot needed (95%+).
    Redline,
}

impl ContextLevel {
    /// Get level from percentage.
    pub fn from_percent(percent: u8) -> Self {
        match percent {
            0..=50 => Self::Low,
            51..=70 => Self::Medium,
            71..=85 => Self::High,
            86..=95 => Self::Warning,
            _ => Self::Redline,
        }
    }

    /// Should we warn about context usage?
    pub fn should_warn(&self) -> bool {
        matches!(self, Self::Warning | Self::Redline)
    }

    /// Should we trigger a reboot?
    pub fn should_reboot(&self) -> bool {
        matches!(self, Self::Redline)
    }
}

/// Configuration for redline detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedlineConfig {
    /// Percentage at which to trigger redline (default: 85).
    pub redline_threshold: u8,

    /// Percentage at which to warn (default: 70).
    pub warning_threshold: u8,

    /// Enable automatic reboots at redline.
    pub auto_reboot: bool,

    /// Minimum iterations before allowing reboot.
    pub min_iterations_before_reboot: u32,

    /// Detection strategy to use.
    pub strategy: DetectionStrategy,

    /// Enable degradation pattern detection.
    pub detect_degradation: bool,

    /// Number of samples for degradation detection.
    pub degradation_sample_size: usize,
}

impl Default for RedlineConfig {
    fn default() -> Self {
        Self {
            redline_threshold: 85,
            warning_threshold: 70,
            auto_reboot: true,
            min_iterations_before_reboot: 3,
            strategy: DetectionStrategy::Hybrid,
            detect_degradation: true,
            degradation_sample_size: 5,
        }
    }
}

/// Strategy for detecting context usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionStrategy {
    /// Parse output for explicit usage indicators.
    OutputParsing,
    /// Estimate based on token counts.
    TokenEstimation,
    /// Detect behavioral degradation patterns.
    BehaviorAnalysis,
    /// Combine multiple strategies.
    Hybrid,
}

/// Result of a redline check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedlineCheckResult {
    /// Current usage percentage (0-100).
    pub usage_percent: u8,

    /// Current context level.
    pub level: ContextLevel,

    /// Whether redline is triggered.
    pub is_redline: bool,

    /// Whether degradation was detected.
    pub degradation_detected: bool,

    /// Confidence in the measurement (0.0-1.0).
    pub confidence: f64,

    /// Estimated tokens used.
    pub estimated_tokens: Option<u64>,

    /// Detection method used.
    pub detection_method: DetectionStrategy,

    /// Recommendation.
    pub recommendation: RedlineRecommendation,
}

/// Recommendation based on redline check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedlineRecommendation {
    /// Continue normally.
    Continue,
    /// Finish current task then reboot.
    FinishAndReboot,
    /// Reboot immediately.
    ImmediateReboot,
    /// Manual intervention needed.
    ManualIntervention,
}

/// A sample of context state for tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSample {
    /// When the sample was taken.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Usage percentage at sample time.
    pub usage_percent: u8,
    /// Iteration number.
    pub iteration: u32,
    /// Response quality score (if available).
    pub quality_score: Option<f64>,
    /// Response latency in ms.
    pub latency_ms: Option<u64>,
}
```

### 2. Redline Detector (src/redline/detector.rs)

```rust
//! Redline detection implementation.

use super::types::{
    ContextLevel, ContextSample, DetectionStrategy, RedlineCheckResult,
    RedlineConfig, RedlineRecommendation,
};
use crate::error::{LoopError, LoopResult};

use std::collections::VecDeque;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Detects context redline conditions.
pub struct RedlineDetector {
    /// Configuration.
    config: RedlineConfig,
    /// Recent context samples.
    samples: RwLock<VecDeque<ContextSample>>,
    /// Current iteration count.
    iteration: std::sync::atomic::AtomicU32,
    /// Iterations since last reboot.
    iterations_since_reboot: std::sync::atomic::AtomicU32,
}

impl RedlineDetector {
    /// Create a new detector.
    pub fn new(config: RedlineConfig) -> Self {
        Self {
            config,
            samples: RwLock::new(VecDeque::new()),
            iteration: std::sync::atomic::AtomicU32::new(0),
            iterations_since_reboot: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Check for redline condition.
    pub async fn check(&self, output: &str, latency_ms: Option<u64>) -> RedlineCheckResult {
        let iteration = self.iteration.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.iterations_since_reboot.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Get usage based on strategy
        let (usage_percent, confidence, method) = match self.config.strategy {
            DetectionStrategy::OutputParsing => self.detect_from_output(output),
            DetectionStrategy::TokenEstimation => self.estimate_from_tokens(output),
            DetectionStrategy::BehaviorAnalysis => self.analyze_behavior(output, latency_ms).await,
            DetectionStrategy::Hybrid => self.hybrid_detection(output, latency_ms).await,
        };

        let level = ContextLevel::from_percent(usage_percent);

        // Record sample
        let sample = ContextSample {
            timestamp: chrono::Utc::now(),
            usage_percent,
            iteration,
            quality_score: self.estimate_quality(output),
            latency_ms,
        };
        self.record_sample(sample).await;

        // Check for degradation
        let degradation_detected = if self.config.detect_degradation {
            self.detect_degradation().await
        } else {
            false
        };

        // Determine if redline
        let is_redline = usage_percent >= self.config.redline_threshold || degradation_detected;

        // Get recommendation
        let recommendation = self.get_recommendation(usage_percent, degradation_detected);

        // Estimate tokens
        let estimated_tokens = self.estimate_token_count(output);

        RedlineCheckResult {
            usage_percent,
            level,
            is_redline,
            degradation_detected,
            confidence,
            estimated_tokens,
            detection_method: method,
            recommendation,
        }
    }

    /// Detect usage from output parsing.
    fn detect_from_output(&self, output: &str) -> (u8, f64, DetectionStrategy) {
        // Look for explicit context indicators in Claude's output
        let patterns = [
            // Common patterns Claude might output
            (r"[Cc]ontext[:\s]+(\d+)%", 1.0),
            (r"(\d+)%\s+(?:of\s+)?context", 0.9),
            (r"context\s+window[:\s]+(\d+)%", 1.0),
            (r"token[s]?\s+used[:\s]+(\d+)%", 0.8),
        ];

        for (pattern, confidence) in patterns {
            if let Some(caps) = regex::Regex::new(pattern)
                .ok()
                .and_then(|re| re.captures(output))
            {
                if let Some(m) = caps.get(1) {
                    if let Ok(percent) = m.as_str().parse::<u8>() {
                        return (percent.min(100), confidence, DetectionStrategy::OutputParsing);
                    }
                }
            }
        }

        // No explicit indicator found, return estimate
        (0, 0.0, DetectionStrategy::OutputParsing)
    }

    /// Estimate usage from token counts.
    fn estimate_from_tokens(&self, output: &str) -> (u8, f64, DetectionStrategy) {
        // Rough estimation: ~4 characters per token
        let char_count = output.len();
        let estimated_tokens = (char_count / 4) as u64;

        // Assume 100K token context window
        const CONTEXT_WINDOW: u64 = 100_000;

        let samples = self.samples.try_read();
        let total_tokens = if let Ok(samples) = samples {
            samples.iter().filter_map(|s| {
                // Estimate tokens from previous outputs
                None::<u64> // This would need actual token tracking
            }).sum::<u64>() + estimated_tokens
        } else {
            estimated_tokens
        };

        let usage = ((total_tokens as f64 / CONTEXT_WINDOW as f64) * 100.0) as u8;
        (usage.min(100), 0.5, DetectionStrategy::TokenEstimation)
    }

    /// Analyze behavior for degradation.
    async fn analyze_behavior(&self, output: &str, latency_ms: Option<u64>) -> (u8, f64, DetectionStrategy) {
        let samples = self.samples.read().await;

        // Check for increasing latency trend
        let latency_increasing = if samples.len() >= 3 {
            let recent: Vec<_> = samples.iter().rev().take(3).collect();
            recent.windows(2).all(|w| {
                match (w[0].latency_ms, w[1].latency_ms) {
                    (Some(a), Some(b)) => a > b,
                    _ => false,
                }
            })
        } else {
            false
        };

        // Check for quality degradation
        let quality_degrading = if samples.len() >= 3 {
            let recent: Vec<_> = samples.iter().rev().take(3).collect();
            recent.windows(2).all(|w| {
                match (w[0].quality_score, w[1].quality_score) {
                    (Some(a), Some(b)) => a < b,
                    _ => false,
                }
            })
        } else {
            false
        };

        // Estimate usage based on behavioral indicators
        let base_usage = if !samples.is_empty() {
            samples.back().map(|s| s.usage_percent).unwrap_or(0)
        } else {
            0
        };

        let usage = if latency_increasing && quality_degrading {
            (base_usage + 20).min(100)
        } else if latency_increasing || quality_degrading {
            (base_usage + 10).min(100)
        } else {
            base_usage
        };

        (usage, 0.6, DetectionStrategy::BehaviorAnalysis)
    }

    /// Hybrid detection combining multiple strategies.
    async fn hybrid_detection(&self, output: &str, latency_ms: Option<u64>) -> (u8, f64, DetectionStrategy) {
        let (output_usage, output_conf, _) = self.detect_from_output(output);
        let (token_usage, token_conf, _) = self.estimate_from_tokens(output);
        let (behavior_usage, behavior_conf, _) = self.analyze_behavior(output, latency_ms).await;

        // Weight by confidence
        let total_confidence = output_conf + token_conf + behavior_conf;

        if total_confidence == 0.0 {
            return (token_usage, 0.3, DetectionStrategy::Hybrid);
        }

        // If we have explicit output parsing, trust it most
        if output_conf > 0.5 {
            return (output_usage, output_conf, DetectionStrategy::Hybrid);
        }

        // Otherwise weighted average
        let weighted_usage = (
            (output_usage as f64 * output_conf) +
            (token_usage as f64 * token_conf) +
            (behavior_usage as f64 * behavior_conf)
        ) / total_confidence;

        let average_conf = total_confidence / 3.0;

        (weighted_usage as u8, average_conf, DetectionStrategy::Hybrid)
    }

    /// Record a context sample.
    async fn record_sample(&self, sample: ContextSample) {
        let mut samples = self.samples.write().await;
        samples.push_back(sample);

        // Keep only recent samples
        while samples.len() > self.config.degradation_sample_size * 2 {
            samples.pop_front();
        }
    }

    /// Detect degradation patterns.
    async fn detect_degradation(&self) -> bool {
        let samples = self.samples.read().await;

        if samples.len() < self.config.degradation_sample_size {
            return false;
        }

        let recent: Vec<_> = samples.iter().rev().take(self.config.degradation_sample_size).collect();

        // Check for consistent usage increase
        let usage_increasing = recent.windows(2).all(|w| w[0].usage_percent >= w[1].usage_percent);

        // Check for quality degradation
        let quality_scores: Vec<_> = recent.iter().filter_map(|s| s.quality_score).collect();
        let quality_degrading = if quality_scores.len() >= 3 {
            let avg_early: f64 = quality_scores[quality_scores.len()/2..].iter().sum::<f64>()
                / (quality_scores.len() / 2) as f64;
            let avg_late: f64 = quality_scores[..quality_scores.len()/2].iter().sum::<f64>()
                / (quality_scores.len() / 2) as f64;
            avg_late < avg_early * 0.8 // 20% degradation
        } else {
            false
        };

        // Check for latency increase
        let latencies: Vec<_> = recent.iter().filter_map(|s| s.latency_ms).collect();
        let latency_increasing = if latencies.len() >= 3 {
            let avg_early: f64 = latencies[latencies.len()/2..].iter().map(|&x| x as f64).sum::<f64>()
                / (latencies.len() / 2) as f64;
            let avg_late: f64 = latencies[..latencies.len()/2].iter().map(|&x| x as f64).sum::<f64>()
                / (latencies.len() / 2) as f64;
            avg_late > avg_early * 1.5 // 50% slower
        } else {
            false
        };

        (usage_increasing && quality_degrading) || (quality_degrading && latency_increasing)
    }

    /// Get recommendation based on state.
    fn get_recommendation(&self, usage: u8, degradation: bool) -> RedlineRecommendation {
        let iterations = self.iterations_since_reboot.load(std::sync::atomic::Ordering::Relaxed);

        // Don't recommend reboot too early
        if iterations < self.config.min_iterations_before_reboot {
            return RedlineRecommendation::Continue;
        }

        if degradation && usage >= self.config.warning_threshold {
            RedlineRecommendation::ImmediateReboot
        } else if usage >= self.config.redline_threshold {
            RedlineRecommendation::ImmediateReboot
        } else if usage >= self.config.warning_threshold {
            RedlineRecommendation::FinishAndReboot
        } else if degradation {
            RedlineRecommendation::FinishAndReboot
        } else {
            RedlineRecommendation::Continue
        }
    }

    /// Estimate response quality.
    fn estimate_quality(&self, output: &str) -> Option<f64> {
        // Simple heuristics for quality
        let mut score = 1.0;

        // Penalize very short responses
        if output.len() < 50 {
            score *= 0.5;
        }

        // Penalize repetitive patterns
        let words: Vec<_> = output.split_whitespace().collect();
        if words.len() > 10 {
            let unique = words.iter().collect::<std::collections::HashSet<_>>().len();
            let repetition_ratio = unique as f64 / words.len() as f64;
            if repetition_ratio < 0.3 {
                score *= 0.6;
            }
        }

        // Penalize error indicators
        if output.contains("I apologize") || output.contains("I'm sorry") {
            score *= 0.8;
        }

        Some(score)
    }

    /// Estimate token count.
    fn estimate_token_count(&self, output: &str) -> Option<u64> {
        // Rough approximation: ~4 characters per token
        Some((output.len() / 4) as u64)
    }

    /// Reset after reboot.
    pub fn reset_after_reboot(&self) {
        self.iterations_since_reboot.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current level.
    pub async fn current_level(&self) -> ContextLevel {
        let samples = self.samples.read().await;
        samples
            .back()
            .map(|s| ContextLevel::from_percent(s.usage_percent))
            .unwrap_or(ContextLevel::Low)
    }
}
```

### 3. Module Root (src/redline/mod.rs)

```rust
//! Context redline detection.

pub mod detector;
pub mod types;

pub use detector::RedlineDetector;
pub use types::{
    ContextLevel, ContextSample, DetectionStrategy, RedlineCheckResult,
    RedlineConfig, RedlineRecommendation,
};
```

---

## Testing Requirements

1. Output parsing extracts usage percentages
2. Token estimation produces reasonable values
3. Degradation detection identifies patterns
4. Hybrid detection combines strategies
5. Recommendations match usage levels
6. Samples are properly maintained
7. Reset clears iteration count
8. Thresholds are respected

---

## Related Specs

- Depends on: [100-session-management.md](100-session-management.md)
- Depends on: [101-fresh-context.md](101-fresh-context.md)
- Next: [103-auto-reboot.md](103-auto-reboot.md)
- Related: [108-loop-metrics.md](108-loop-metrics.md)
