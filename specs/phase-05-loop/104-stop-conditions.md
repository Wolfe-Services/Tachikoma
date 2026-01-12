# 104 - Stop Condition Types

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 104
**Status:** Planned
**Dependencies:** 096-loop-runner-core
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Define the stop condition types for the Ralph Loop - the various conditions that can cause the loop to terminate, including success criteria, failure thresholds, and custom conditions.

---

## Acceptance Criteria

- [x] Stop condition type definitions
- [x] Built-in stop conditions (iterations, time, etc.)
- [x] Test-based stop conditions
- [x] Progress-based stop conditions
- [x] Custom condition support
- [x] Condition composition (AND/OR)
- [x] Serializable condition configurations
- [x] Condition priority system

---

## Implementation Details

### 1. Stop Condition Types (src/stop/types.rs)

```rust
//! Stop condition type definitions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// A condition that can stop the loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StopCondition {
    /// Stop after a number of iterations.
    MaxIterations {
        count: u32,
    },

    /// Stop after a duration.
    MaxDuration {
        #[serde(with = "humantime_serde")]
        duration: Duration,
    },

    /// Stop after N consecutive test failures.
    TestFailureStreak {
        count: u32,
    },

    /// Stop when all tests pass.
    AllTestsPass,

    /// Stop when specific tests pass.
    SpecificTestsPass {
        tests: Vec<String>,
    },

    /// Stop after N iterations with no progress.
    NoProgress {
        iterations: u32,
    },

    /// Stop when a file is created.
    FileCreated {
        path: PathBuf,
    },

    /// Stop when a file contains specific content.
    FileContains {
        path: PathBuf,
        content: String,
    },

    /// Stop when a pattern is matched in output.
    OutputPattern {
        pattern: String,
        #[serde(default)]
        is_regex: bool,
    },

    /// Stop when a specific error occurs.
    OnError {
        pattern: Option<String>,
    },

    /// Stop when a custom script returns success.
    CustomScript {
        script: PathBuf,
        #[serde(default)]
        args: Vec<String>,
    },

    /// Stop when user signals (attended mode).
    UserSignal,

    /// Composite: all conditions must be true.
    All {
        conditions: Vec<StopCondition>,
    },

    /// Composite: any condition must be true.
    Any {
        conditions: Vec<StopCondition>,
    },

    /// Negation of a condition.
    Not {
        condition: Box<StopCondition>,
    },

    /// Never stop (must be manually stopped).
    Never,
}

impl StopCondition {
    /// Create a max iterations condition.
    pub fn max_iterations(count: u32) -> Self {
        Self::MaxIterations { count }
    }

    /// Create a max duration condition.
    pub fn max_duration(duration: Duration) -> Self {
        Self::MaxDuration { duration }
    }

    /// Create a test failure streak condition.
    pub fn test_failure_streak(count: u32) -> Self {
        Self::TestFailureStreak { count }
    }

    /// Create a no progress condition.
    pub fn no_progress(iterations: u32) -> Self {
        Self::NoProgress { iterations }
    }

    /// Create an output pattern condition.
    pub fn output_pattern(pattern: impl Into<String>) -> Self {
        Self::OutputPattern {
            pattern: pattern.into(),
            is_regex: false,
        }
    }

    /// Create a regex pattern condition.
    pub fn output_regex(pattern: impl Into<String>) -> Self {
        Self::OutputPattern {
            pattern: pattern.into(),
            is_regex: true,
        }
    }

    /// Combine with AND.
    pub fn and(self, other: StopCondition) -> Self {
        match self {
            Self::All { mut conditions } => {
                conditions.push(other);
                Self::All { conditions }
            }
            _ => Self::All {
                conditions: vec![self, other],
            },
        }
    }

    /// Combine with OR.
    pub fn or(self, other: StopCondition) -> Self {
        match self {
            Self::Any { mut conditions } => {
                conditions.push(other);
                Self::Any { conditions }
            }
            _ => Self::Any {
                conditions: vec![self, other],
            },
        }
    }

    /// Negate the condition.
    pub fn not(self) -> Self {
        Self::Not {
            condition: Box::new(self),
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> String {
        match self {
            Self::MaxIterations { count } => format!("after {} iterations", count),
            Self::MaxDuration { duration } => {
                format!("after {}", humantime::format_duration(*duration))
            }
            Self::TestFailureStreak { count } => {
                format!("after {} consecutive test failures", count)
            }
            Self::AllTestsPass => "when all tests pass".to_string(),
            Self::SpecificTestsPass { tests } => {
                format!("when tests pass: {}", tests.join(", "))
            }
            Self::NoProgress { iterations } => {
                format!("after {} iterations with no progress", iterations)
            }
            Self::FileCreated { path } => format!("when {} is created", path.display()),
            Self::FileContains { path, content } => {
                format!("when {} contains '{}'", path.display(), content)
            }
            Self::OutputPattern { pattern, is_regex } => {
                if *is_regex {
                    format!("when output matches regex /{}/", pattern)
                } else {
                    format!("when output contains '{}'", pattern)
                }
            }
            Self::OnError { pattern } => match pattern {
                Some(p) => format!("on error matching '{}'", p),
                None => "on any error".to_string(),
            },
            Self::CustomScript { script, .. } => {
                format!("when script {} succeeds", script.display())
            }
            Self::UserSignal => "on user signal".to_string(),
            Self::All { conditions } => {
                let descs: Vec<_> = conditions.iter().map(|c| c.description()).collect();
                format!("when ALL: [{}]", descs.join(" AND "))
            }
            Self::Any { conditions } => {
                let descs: Vec<_> = conditions.iter().map(|c| c.description()).collect();
                format!("when ANY: [{}]", descs.join(" OR "))
            }
            Self::Not { condition } => format!("NOT ({})", condition.description()),
            Self::Never => "never (manual stop only)".to_string(),
        }
    }

    /// Get the priority of this condition (higher = checked first).
    pub fn priority(&self) -> u32 {
        match self {
            Self::OnError { .. } => 100,    // Check errors first
            Self::UserSignal => 90,          // User signals are high priority
            Self::MaxIterations { .. } => 80,
            Self::MaxDuration { .. } => 80,
            Self::TestFailureStreak { .. } => 70,
            Self::NoProgress { .. } => 70,
            Self::AllTestsPass => 60,
            Self::SpecificTestsPass { .. } => 60,
            Self::OutputPattern { .. } => 50,
            Self::FileCreated { .. } => 40,
            Self::FileContains { .. } => 40,
            Self::CustomScript { .. } => 30,
            Self::All { .. } => 20,
            Self::Any { .. } => 20,
            Self::Not { .. } => 10,
            Self::Never => 0,
        }
    }
}

/// Result of evaluating a stop condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopConditionResult {
    /// The condition that was evaluated.
    pub condition: StopCondition,
    /// Whether the condition is met.
    pub is_met: bool,
    /// Reason/details for the result.
    pub reason: Option<String>,
    /// Progress toward meeting the condition (0.0-1.0).
    pub progress: Option<f64>,
}

impl StopConditionResult {
    /// Create a result indicating condition is met.
    pub fn met(condition: StopCondition, reason: impl Into<String>) -> Self {
        Self {
            condition,
            is_met: true,
            reason: Some(reason.into()),
            progress: Some(1.0),
        }
    }

    /// Create a result indicating condition is not met.
    pub fn not_met(condition: StopCondition) -> Self {
        Self {
            condition,
            is_met: false,
            reason: None,
            progress: None,
        }
    }

    /// Create a result with progress.
    pub fn with_progress(condition: StopCondition, progress: f64, reason: Option<String>) -> Self {
        Self {
            condition,
            is_met: progress >= 1.0,
            reason,
            progress: Some(progress.clamp(0.0, 1.0)),
        }
    }
}

/// Context provided to stop condition evaluation.
#[derive(Debug, Clone)]
pub struct StopConditionContext {
    /// Current iteration number.
    pub iteration: u32,
    /// Loop start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Current time.
    pub current_time: chrono::DateTime<chrono::Utc>,
    /// Recent output.
    pub recent_output: String,
    /// Test failure streak.
    pub test_failure_streak: u32,
    /// Iterations since progress.
    pub iterations_since_progress: u32,
    /// Tests that passed.
    pub passed_tests: Vec<String>,
    /// Tests that failed.
    pub failed_tests: Vec<String>,
    /// Whether an error occurred.
    pub has_error: bool,
    /// Error message if any.
    pub error_message: Option<String>,
    /// User signal received.
    pub user_signal: bool,
    /// Working directory.
    pub working_dir: PathBuf,
}

impl Default for StopConditionContext {
    fn default() -> Self {
        Self {
            iteration: 0,
            start_time: chrono::Utc::now(),
            current_time: chrono::Utc::now(),
            recent_output: String::new(),
            test_failure_streak: 0,
            iterations_since_progress: 0,
            passed_tests: vec![],
            failed_tests: vec![],
            has_error: false,
            error_message: None,
            user_signal: false,
            working_dir: PathBuf::from("."),
        }
    }
}

/// Configuration for a set of stop conditions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopConditionsConfig {
    /// Primary stop conditions (any can trigger stop).
    pub conditions: Vec<StopCondition>,
    /// Success conditions (stop and mark as success).
    pub success_conditions: Vec<StopCondition>,
    /// Failure conditions (stop and mark as failure).
    pub failure_conditions: Vec<StopCondition>,
    /// Whether to stop on first condition met.
    pub stop_on_first: bool,
}

impl StopConditionsConfig {
    /// Create a new config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a stop condition.
    pub fn add_condition(mut self, condition: StopCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add a success condition.
    pub fn add_success(mut self, condition: StopCondition) -> Self {
        self.success_conditions.push(condition);
        self
    }

    /// Add a failure condition.
    pub fn add_failure(mut self, condition: StopCondition) -> Self {
        self.failure_conditions.push(condition);
        self
    }
}

mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&humantime::format_duration(*duration).to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        humantime::parse_duration(&s).map_err(serde::de::Error::custom)
    }
}
```

### 2. Built-in Conditions (src/stop/builtin.rs)

```rust
//! Built-in stop condition implementations.

use super::types::{StopCondition, StopConditionContext, StopConditionResult};
use crate::error::LoopResult;

use std::path::Path;
use tracing::debug;

/// Evaluate a single stop condition.
pub fn evaluate_condition(
    condition: &StopCondition,
    ctx: &StopConditionContext,
) -> LoopResult<StopConditionResult> {
    match condition {
        StopCondition::MaxIterations { count } => {
            let progress = ctx.iteration as f64 / *count as f64;
            if ctx.iteration >= *count {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    format!("Reached {} iterations", count),
                ))
            } else {
                Ok(StopConditionResult::with_progress(
                    condition.clone(),
                    progress,
                    Some(format!("Iteration {}/{}", ctx.iteration, count)),
                ))
            }
        }

        StopCondition::MaxDuration { duration } => {
            let elapsed = ctx.current_time - ctx.start_time;
            let elapsed_secs = elapsed.num_seconds() as u64;
            let limit_secs = duration.as_secs();
            let progress = elapsed_secs as f64 / limit_secs as f64;

            if elapsed_secs >= limit_secs {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    format!("Reached duration limit of {:?}", duration),
                ))
            } else {
                Ok(StopConditionResult::with_progress(
                    condition.clone(),
                    progress,
                    Some(format!("{}s / {}s elapsed", elapsed_secs, limit_secs)),
                ))
            }
        }

        StopCondition::TestFailureStreak { count } => {
            let progress = ctx.test_failure_streak as f64 / *count as f64;
            if ctx.test_failure_streak >= *count {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    format!("{} consecutive test failures", count),
                ))
            } else {
                Ok(StopConditionResult::with_progress(
                    condition.clone(),
                    progress,
                    Some(format!("{}/{} failures", ctx.test_failure_streak, count)),
                ))
            }
        }

        StopCondition::AllTestsPass => {
            if ctx.failed_tests.is_empty() && !ctx.passed_tests.is_empty() {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    format!("All {} tests passed", ctx.passed_tests.len()),
                ))
            } else {
                let total = ctx.passed_tests.len() + ctx.failed_tests.len();
                let progress = if total > 0 {
                    ctx.passed_tests.len() as f64 / total as f64
                } else {
                    0.0
                };
                Ok(StopConditionResult::with_progress(
                    condition.clone(),
                    progress,
                    Some(format!(
                        "{}/{} tests passing",
                        ctx.passed_tests.len(),
                        total
                    )),
                ))
            }
        }

        StopCondition::SpecificTestsPass { tests } => {
            let passing: Vec<_> = tests
                .iter()
                .filter(|t| ctx.passed_tests.contains(t))
                .collect();
            let progress = passing.len() as f64 / tests.len() as f64;

            if passing.len() == tests.len() {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    "All specified tests passed".to_string(),
                ))
            } else {
                let missing: Vec<_> = tests
                    .iter()
                    .filter(|t| !ctx.passed_tests.contains(t))
                    .collect();
                Ok(StopConditionResult::with_progress(
                    condition.clone(),
                    progress,
                    Some(format!(
                        "{}/{} passing, waiting for: {}",
                        passing.len(),
                        tests.len(),
                        missing.iter().take(3).cloned().cloned().collect::<Vec<_>>().join(", ")
                    )),
                ))
            }
        }

        StopCondition::NoProgress { iterations } => {
            let progress = ctx.iterations_since_progress as f64 / *iterations as f64;
            if ctx.iterations_since_progress >= *iterations {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    format!("No progress for {} iterations", iterations),
                ))
            } else {
                Ok(StopConditionResult::with_progress(
                    condition.clone(),
                    progress,
                    Some(format!(
                        "{}/{} iterations without progress",
                        ctx.iterations_since_progress, iterations
                    )),
                ))
            }
        }

        StopCondition::FileCreated { path } => {
            let full_path = ctx.working_dir.join(path);
            if full_path.exists() {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    format!("File created: {}", path.display()),
                ))
            } else {
                Ok(StopConditionResult::not_met(condition.clone()))
            }
        }

        StopCondition::FileContains { path, content } => {
            let full_path = ctx.working_dir.join(path);
            if full_path.exists() {
                if let Ok(file_content) = std::fs::read_to_string(&full_path) {
                    if file_content.contains(content) {
                        return Ok(StopConditionResult::met(
                            condition.clone(),
                            format!("File {} contains expected content", path.display()),
                        ));
                    }
                }
            }
            Ok(StopConditionResult::not_met(condition.clone()))
        }

        StopCondition::OutputPattern { pattern, is_regex } => {
            let matches = if *is_regex {
                regex::Regex::new(pattern)
                    .map(|re| re.is_match(&ctx.recent_output))
                    .unwrap_or(false)
            } else {
                ctx.recent_output.contains(pattern)
            };

            if matches {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    format!("Output matched pattern: {}", pattern),
                ))
            } else {
                Ok(StopConditionResult::not_met(condition.clone()))
            }
        }

        StopCondition::OnError { pattern } => {
            if ctx.has_error {
                let matches = match (pattern, &ctx.error_message) {
                    (Some(p), Some(msg)) => msg.contains(p),
                    (None, _) => true,
                    _ => false,
                };

                if matches {
                    return Ok(StopConditionResult::met(
                        condition.clone(),
                        format!(
                            "Error occurred: {}",
                            ctx.error_message.as_deref().unwrap_or("unknown")
                        ),
                    ));
                }
            }
            Ok(StopConditionResult::not_met(condition.clone()))
        }

        StopCondition::CustomScript { script, args } => {
            let full_path = ctx.working_dir.join(script);
            evaluate_custom_script(&full_path, args, condition)
        }

        StopCondition::UserSignal => {
            if ctx.user_signal {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    "User signal received".to_string(),
                ))
            } else {
                Ok(StopConditionResult::not_met(condition.clone()))
            }
        }

        StopCondition::All { conditions } => {
            let mut all_met = true;
            let mut progress_sum = 0.0;

            for cond in conditions {
                let result = evaluate_condition(cond, ctx)?;
                if !result.is_met {
                    all_met = false;
                }
                progress_sum += result.progress.unwrap_or(0.0);
            }

            let avg_progress = progress_sum / conditions.len() as f64;

            if all_met {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    "All conditions met".to_string(),
                ))
            } else {
                Ok(StopConditionResult::with_progress(
                    condition.clone(),
                    avg_progress,
                    None,
                ))
            }
        }

        StopCondition::Any { conditions } => {
            let mut max_progress = 0.0;

            for cond in conditions {
                let result = evaluate_condition(cond, ctx)?;
                if result.is_met {
                    return Ok(StopConditionResult::met(
                        condition.clone(),
                        result.reason.unwrap_or_else(|| "Condition met".to_string()),
                    ));
                }
                if let Some(p) = result.progress {
                    max_progress = max_progress.max(p);
                }
            }

            Ok(StopConditionResult::with_progress(
                condition.clone(),
                max_progress,
                None,
            ))
        }

        StopCondition::Not { condition: inner } => {
            let result = evaluate_condition(inner, ctx)?;
            if result.is_met {
                Ok(StopConditionResult::not_met(condition.clone()))
            } else {
                Ok(StopConditionResult::met(
                    condition.clone(),
                    "Negated condition not met".to_string(),
                ))
            }
        }

        StopCondition::Never => Ok(StopConditionResult::not_met(condition.clone())),
    }
}

/// Evaluate a custom script.
fn evaluate_custom_script(
    script: &Path,
    args: &[String],
    condition: &StopCondition,
) -> LoopResult<StopConditionResult> {
    use std::process::Command;

    if !script.exists() {
        debug!("Custom script not found: {:?}", script);
        return Ok(StopConditionResult::not_met(condition.clone()));
    }

    let output = Command::new(script)
        .args(args)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(StopConditionResult::met(
                condition.clone(),
                format!("Script succeeded: {}", stdout.trim()),
            ))
        }
        Ok(_) => Ok(StopConditionResult::not_met(condition.clone())),
        Err(e) => {
            debug!("Custom script error: {}", e);
            Ok(StopConditionResult::not_met(condition.clone()))
        }
    }
}
```

### 3. Module Root (src/stop/mod.rs)

```rust
//! Stop condition definitions and evaluation.

pub mod builtin;
pub mod types;

pub use builtin::evaluate_condition;
pub use types::{
    StopCondition, StopConditionContext, StopConditionResult, StopConditionsConfig,
};
```

---

## Testing Requirements

1. MaxIterations triggers at correct count
2. MaxDuration triggers after elapsed time
3. TestFailureStreak counts correctly
4. Pattern matching works for strings and regex
5. File conditions detect file changes
6. Composite conditions evaluate correctly
7. Priority ordering is respected
8. Custom scripts execute and return results

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Next: [105-stop-evaluation.md](105-stop-evaluation.md)
- Related: [106-test-failure-tracking.md](106-test-failure-tracking.md)
