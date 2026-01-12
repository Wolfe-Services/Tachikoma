# 105 - Stop Condition Evaluation

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 105
**Status:** Planned
**Dependencies:** 104-stop-conditions
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement the stop condition evaluation system for the Ralph Loop - coordinating the evaluation of multiple conditions, handling priorities, and determining when the loop should terminate.

---

## Acceptance Criteria

- [ ] `StopConditionEvaluator` for coordinated evaluation
- [ ] Priority-based evaluation order
- [ ] Parallel evaluation where possible
- [ ] Caching of condition results
- [ ] Event emission on condition changes
- [ ] Evaluation history tracking
- [ ] Performance optimization
- [ ] Debug/verbose mode for troubleshooting

---

## Implementation Details

### 1. Evaluator Types (src/stop/evaluator_types.rs)

```rust
//! Types for stop condition evaluation.

use super::types::{StopCondition, StopConditionResult};
use serde::{Deserialize, Serialize};

/// Result of evaluating all stop conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Whether any stop condition was met.
    pub should_stop: bool,
    /// The condition that triggered the stop (if any).
    pub triggered_by: Option<StopCondition>,
    /// Whether the stop is a success or failure.
    pub is_success: bool,
    /// All condition results.
    pub condition_results: Vec<StopConditionResult>,
    /// Timestamp of evaluation.
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
    /// Evaluation duration in microseconds.
    pub duration_us: u64,
}

impl EvaluationResult {
    /// Create a result indicating no stop.
    pub fn continue_loop(results: Vec<StopConditionResult>, duration_us: u64) -> Self {
        Self {
            should_stop: false,
            triggered_by: None,
            is_success: false,
            condition_results: results,
            evaluated_at: chrono::Utc::now(),
            duration_us,
        }
    }

    /// Create a result indicating stop.
    pub fn stop(
        triggered_by: StopCondition,
        is_success: bool,
        results: Vec<StopConditionResult>,
        duration_us: u64,
    ) -> Self {
        Self {
            should_stop: true,
            triggered_by: Some(triggered_by),
            is_success,
            condition_results: results,
            evaluated_at: chrono::Utc::now(),
            duration_us,
        }
    }

    /// Get overall progress toward stopping (max of all conditions).
    pub fn overall_progress(&self) -> f64 {
        self.condition_results
            .iter()
            .filter_map(|r| r.progress)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }
}

/// Configuration for the evaluator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorConfig {
    /// Enable parallel evaluation.
    pub parallel_evaluation: bool,
    /// Maximum parallel evaluations.
    pub max_parallel: usize,
    /// Enable result caching.
    pub enable_cache: bool,
    /// Cache TTL in milliseconds.
    pub cache_ttl_ms: u64,
    /// Enable debug logging.
    pub debug: bool,
    /// Skip slow conditions after timeout.
    pub skip_slow_conditions: bool,
    /// Timeout for individual conditions (ms).
    pub condition_timeout_ms: u64,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            parallel_evaluation: true,
            max_parallel: 4,
            enable_cache: true,
            cache_ttl_ms: 1000,
            debug: false,
            skip_slow_conditions: true,
            condition_timeout_ms: 5000,
        }
    }
}

/// History entry for evaluation tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationHistoryEntry {
    /// Iteration number.
    pub iteration: u32,
    /// The evaluation result.
    pub result: EvaluationResult,
    /// Conditions that progressed.
    pub progressed_conditions: Vec<ConditionProgress>,
}

/// Progress of a condition over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionProgress {
    /// The condition.
    pub condition: StopCondition,
    /// Previous progress.
    pub previous_progress: Option<f64>,
    /// Current progress.
    pub current_progress: f64,
    /// Change in progress.
    pub delta: f64,
}

/// Event emitted during evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvaluatorEvent {
    /// Evaluation started.
    Started {
        iteration: u32,
        condition_count: usize,
    },
    /// Condition was evaluated.
    ConditionEvaluated {
        condition: StopCondition,
        result: StopConditionResult,
    },
    /// Condition made progress.
    ConditionProgressed {
        condition: StopCondition,
        previous: f64,
        current: f64,
    },
    /// Stop condition was triggered.
    StopTriggered {
        condition: StopCondition,
        reason: String,
        is_success: bool,
    },
    /// Evaluation completed.
    Completed {
        should_stop: bool,
        duration_us: u64,
    },
    /// Condition evaluation timed out.
    ConditionTimeout {
        condition: StopCondition,
    },
}
```

### 2. Stop Condition Evaluator (src/stop/evaluator.rs)

```rust
//! Stop condition evaluation coordination.

use super::builtin::evaluate_condition;
use super::evaluator_types::{
    ConditionProgress, EvaluationHistoryEntry, EvaluationResult, EvaluatorConfig, EvaluatorEvent,
};
use super::types::{StopCondition, StopConditionContext, StopConditionResult, StopConditionsConfig};
use crate::error::{LoopError, LoopResult};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, instrument, trace, warn};

/// Evaluates stop conditions for the loop.
pub struct StopConditionEvaluator {
    /// Configuration.
    config: EvaluatorConfig,
    /// Stop conditions to evaluate.
    conditions: StopConditionsConfig,
    /// Result cache.
    cache: RwLock<ConditionCache>,
    /// Previous progress for each condition.
    previous_progress: RwLock<HashMap<String, f64>>,
    /// Evaluation history.
    history: RwLock<Vec<EvaluationHistoryEntry>>,
    /// Event emitter.
    event_tx: broadcast::Sender<EvaluatorEvent>,
}

/// Cache for condition results.
struct ConditionCache {
    results: HashMap<String, CachedResult>,
}

struct CachedResult {
    result: StopConditionResult,
    cached_at: std::time::Instant,
}

impl ConditionCache {
    fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    fn get(&self, key: &str, ttl_ms: u64) -> Option<&StopConditionResult> {
        self.results.get(key).and_then(|cached| {
            if cached.cached_at.elapsed().as_millis() < ttl_ms as u128 {
                Some(&cached.result)
            } else {
                None
            }
        })
    }

    fn set(&mut self, key: String, result: StopConditionResult) {
        self.results.insert(
            key,
            CachedResult {
                result,
                cached_at: std::time::Instant::now(),
            },
        );
    }

    fn clear(&mut self) {
        self.results.clear();
    }
}

impl StopConditionEvaluator {
    /// Create a new evaluator.
    pub fn new(conditions: StopConditionsConfig) -> Self {
        Self::with_config(conditions, EvaluatorConfig::default())
    }

    /// Create with custom config.
    pub fn with_config(conditions: StopConditionsConfig, config: EvaluatorConfig) -> Self {
        let (event_tx, _) = broadcast::channel(64);
        Self {
            config,
            conditions,
            cache: RwLock::new(ConditionCache::new()),
            previous_progress: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            event_tx,
        }
    }

    /// Subscribe to evaluation events.
    pub fn subscribe(&self) -> broadcast::Receiver<EvaluatorEvent> {
        self.event_tx.subscribe()
    }

    /// Evaluate all conditions.
    #[instrument(skip(self, ctx))]
    pub async fn evaluate(&self, ctx: &StopConditionContext) -> LoopResult<EvaluationResult> {
        let start = std::time::Instant::now();

        let condition_count = self.conditions.conditions.len()
            + self.conditions.success_conditions.len()
            + self.conditions.failure_conditions.len();

        self.emit(EvaluatorEvent::Started {
            iteration: ctx.iteration,
            condition_count,
        });

        // Gather all conditions with their types
        let mut all_conditions: Vec<(StopCondition, ConditionType)> = Vec::new();

        for cond in &self.conditions.conditions {
            all_conditions.push((cond.clone(), ConditionType::Normal));
        }
        for cond in &self.conditions.success_conditions {
            all_conditions.push((cond.clone(), ConditionType::Success));
        }
        for cond in &self.conditions.failure_conditions {
            all_conditions.push((cond.clone(), ConditionType::Failure));
        }

        // Sort by priority
        all_conditions.sort_by(|a, b| b.0.priority().cmp(&a.0.priority()));

        // Evaluate conditions
        let results = if self.config.parallel_evaluation {
            self.evaluate_parallel(&all_conditions, ctx).await?
        } else {
            self.evaluate_sequential(&all_conditions, ctx).await?
        };

        // Track progress
        let progressed = self.track_progress(&results).await;

        // Check for triggered conditions
        let (should_stop, triggered_by, is_success) = self.check_triggered(&results);

        let duration_us = start.elapsed().as_micros() as u64;

        // Create result
        let result = if should_stop {
            let triggered = triggered_by.unwrap();
            self.emit(EvaluatorEvent::StopTriggered {
                condition: triggered.clone(),
                reason: results
                    .iter()
                    .find(|r| r.is_met)
                    .and_then(|r| r.reason.clone())
                    .unwrap_or_default(),
                is_success,
            });
            EvaluationResult::stop(triggered, is_success, results, duration_us)
        } else {
            EvaluationResult::continue_loop(results, duration_us)
        };

        // Record history
        self.record_history(ctx.iteration, result.clone(), progressed).await;

        self.emit(EvaluatorEvent::Completed {
            should_stop: result.should_stop,
            duration_us,
        });

        Ok(result)
    }

    /// Evaluate conditions sequentially.
    async fn evaluate_sequential(
        &self,
        conditions: &[(StopCondition, ConditionType)],
        ctx: &StopConditionContext,
    ) -> LoopResult<Vec<StopConditionResult>> {
        let mut results = Vec::new();

        for (condition, _ctype) in conditions {
            let result = self.evaluate_single(condition, ctx).await?;

            self.emit(EvaluatorEvent::ConditionEvaluated {
                condition: condition.clone(),
                result: result.clone(),
            });

            results.push(result.clone());

            // Early exit if stop_on_first and condition met
            if self.conditions.stop_on_first && result.is_met {
                break;
            }
        }

        Ok(results)
    }

    /// Evaluate conditions in parallel.
    async fn evaluate_parallel(
        &self,
        conditions: &[(StopCondition, ConditionType)],
        ctx: &StopConditionContext,
    ) -> LoopResult<Vec<StopConditionResult>> {
        use futures::stream::{self, StreamExt};

        let results: Vec<_> = stream::iter(conditions.iter())
            .map(|(condition, _)| async move {
                let timeout = std::time::Duration::from_millis(self.config.condition_timeout_ms);
                match tokio::time::timeout(timeout, self.evaluate_single(condition, ctx)).await {
                    Ok(result) => result,
                    Err(_) => {
                        warn!("Condition evaluation timed out: {:?}", condition);
                        self.emit(EvaluatorEvent::ConditionTimeout {
                            condition: condition.clone(),
                        });
                        Ok(StopConditionResult::not_met(condition.clone()))
                    }
                }
            })
            .buffer_unordered(self.config.max_parallel)
            .collect()
            .await;

        // Flatten results
        let mut flattened = Vec::new();
        for result in results {
            flattened.push(result?);
        }

        // Emit events
        for result in &flattened {
            self.emit(EvaluatorEvent::ConditionEvaluated {
                condition: result.condition.clone(),
                result: result.clone(),
            });
        }

        Ok(flattened)
    }

    /// Evaluate a single condition.
    async fn evaluate_single(
        &self,
        condition: &StopCondition,
        ctx: &StopConditionContext,
    ) -> LoopResult<StopConditionResult> {
        let cache_key = format!("{:?}", condition);

        // Check cache
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key, self.config.cache_ttl_ms) {
                trace!("Cache hit for condition: {:?}", condition);
                return Ok(cached.clone());
            }
        }

        // Evaluate
        let result = evaluate_condition(condition, ctx)?;

        // Update cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.set(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Track progress changes.
    async fn track_progress(&self, results: &[StopConditionResult]) -> Vec<ConditionProgress> {
        let mut progressed = Vec::new();
        let mut prev_progress = self.previous_progress.write().await;

        for result in results {
            if let Some(current) = result.progress {
                let key = format!("{:?}", result.condition);
                let previous = prev_progress.get(&key).copied();
                let delta = current - previous.unwrap_or(0.0);

                if delta.abs() > 0.001 {
                    progressed.push(ConditionProgress {
                        condition: result.condition.clone(),
                        previous_progress: previous,
                        current_progress: current,
                        delta,
                    });

                    self.emit(EvaluatorEvent::ConditionProgressed {
                        condition: result.condition.clone(),
                        previous: previous.unwrap_or(0.0),
                        current,
                    });
                }

                prev_progress.insert(key, current);
            }
        }

        progressed
    }

    /// Check if any condition triggered stop.
    fn check_triggered(
        &self,
        results: &[StopConditionResult],
    ) -> (bool, Option<StopCondition>, bool) {
        // Check failure conditions first
        for result in results {
            if result.is_met {
                for fail_cond in &self.conditions.failure_conditions {
                    if format!("{:?}", result.condition) == format!("{:?}", fail_cond) {
                        return (true, Some(result.condition.clone()), false);
                    }
                }
            }
        }

        // Check success conditions
        for result in results {
            if result.is_met {
                for success_cond in &self.conditions.success_conditions {
                    if format!("{:?}", result.condition) == format!("{:?}", success_cond) {
                        return (true, Some(result.condition.clone()), true);
                    }
                }
            }
        }

        // Check normal conditions
        for result in results {
            if result.is_met {
                for cond in &self.conditions.conditions {
                    if format!("{:?}", result.condition) == format!("{:?}", cond) {
                        return (true, Some(result.condition.clone()), false);
                    }
                }
            }
        }

        (false, None, false)
    }

    /// Record evaluation in history.
    async fn record_history(
        &self,
        iteration: u32,
        result: EvaluationResult,
        progressed: Vec<ConditionProgress>,
    ) {
        let mut history = self.history.write().await;
        history.push(EvaluationHistoryEntry {
            iteration,
            result,
            progressed_conditions: progressed,
        });

        // Keep last 100 entries
        while history.len() > 100 {
            history.remove(0);
        }
    }

    /// Emit an event.
    fn emit(&self, event: EvaluatorEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Get evaluation history.
    pub async fn get_history(&self) -> Vec<EvaluationHistoryEntry> {
        self.history.read().await.clone()
    }

    /// Clear cache.
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// Add a condition at runtime.
    pub fn add_condition(&mut self, condition: StopCondition) {
        self.conditions.conditions.push(condition);
    }

    /// Remove a condition at runtime.
    pub fn remove_condition(&mut self, index: usize) {
        if index < self.conditions.conditions.len() {
            self.conditions.conditions.remove(index);
        }
    }

    /// Get condition count.
    pub fn condition_count(&self) -> usize {
        self.conditions.conditions.len()
            + self.conditions.success_conditions.len()
            + self.conditions.failure_conditions.len()
    }
}

/// Type of stop condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConditionType {
    Normal,
    Success,
    Failure,
}
```

### 3. Update Module Root (src/stop/mod.rs)

```rust
//! Stop condition definitions and evaluation.

pub mod builtin;
pub mod evaluator;
pub mod evaluator_types;
pub mod types;

pub use builtin::evaluate_condition;
pub use evaluator::StopConditionEvaluator;
pub use evaluator_types::{
    ConditionProgress, EvaluationHistoryEntry, EvaluationResult, EvaluatorConfig, EvaluatorEvent,
};
pub use types::{
    StopCondition, StopConditionContext, StopConditionResult, StopConditionsConfig,
};
```

---

## Testing Requirements

1. Sequential evaluation respects priority order
2. Parallel evaluation produces same results
3. Cache returns valid cached results
4. Cache invalidates after TTL
5. Progress tracking detects changes
6. History is properly maintained
7. Events are emitted for all state changes
8. Timeout handling works for slow conditions

---

## Related Specs

- Depends on: [104-stop-conditions.md](104-stop-conditions.md)
- Next: [106-test-failure-tracking.md](106-test-failure-tracking.md)
- Related: [096-loop-runner-core.md](096-loop-runner-core.md)
