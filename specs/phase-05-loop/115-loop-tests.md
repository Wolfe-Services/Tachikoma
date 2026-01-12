# 115 - Loop Tests

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 115
**Status:** Planned
**Dependencies:** All Phase 5 specs
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement comprehensive tests for the Ralph Loop Runner - unit tests, integration tests, and end-to-end tests to ensure reliability and correctness of the loop execution system.

---

## Acceptance Criteria

- [ ] Unit tests for all core modules
- [ ] Integration tests for component interaction
- [ ] End-to-end tests for complete loop execution
- [ ] Mock session support for testing
- [ ] Property-based tests for state machines
- [ ] Performance benchmarks
- [ ] Test fixtures and utilities
- [ ] CI/CD integration

---

## Implementation Details

### 1. Test Utilities (tests/utils.rs)

```rust
//! Test utilities and fixtures for loop testing.

use tachikoma_loop_runner::{
    config::{LoopConfig, SessionConfig, StopConditionsConfig},
    iteration::{Iteration, IterationResult, TestResults},
    session::{Session, SessionConfig as SessConfig, SessionId},
    state::{LoopState, LoopStats},
    LoopId, LoopRunner,
};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Test context with temporary directory and utilities.
pub struct TestContext {
    pub temp_dir: TempDir,
    pub working_dir: PathBuf,
    pub config: LoopConfig,
}

impl TestContext {
    /// Create a new test context.
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let working_dir = temp_dir.path().to_path_buf();

        let config = LoopConfig {
            working_dir: working_dir.clone(),
            prompt_path: working_dir.join("prompt.md"),
            max_iterations: 10,
            iteration_delay: Duration::from_millis(10),
            context_redline_percent: 85,
            attended_mode: false,
            stop_conditions: StopConditionsConfig::default(),
            session: SessionConfig::default(),
        };

        Self {
            temp_dir,
            working_dir,
            config,
        }
    }

    /// Create a prompt file.
    pub async fn create_prompt(&self, content: &str) {
        tokio::fs::write(&self.config.prompt_path, content)
            .await
            .expect("Failed to create prompt");
    }

    /// Create a test file.
    pub async fn create_file(&self, name: &str, content: &str) {
        let path = self.working_dir.join(name);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        tokio::fs::write(&path, content)
            .await
            .expect("Failed to create file");
    }

    /// Read a file.
    pub async fn read_file(&self, name: &str) -> String {
        let path = self.working_dir.join(name);
        tokio::fs::read_to_string(&path)
            .await
            .unwrap_or_default()
    }

    /// Check if file exists.
    pub fn file_exists(&self, name: &str) -> bool {
        self.working_dir.join(name).exists()
    }
}

/// Mock session for testing.
pub struct MockSession {
    id: SessionId,
    responses: RwLock<Vec<MockResponse>>,
    response_index: RwLock<usize>,
    context_usage: RwLock<u8>,
}

/// A mock response.
#[derive(Clone)]
pub struct MockResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub delay: Duration,
    pub context_increase: u8,
}

impl Default for MockResponse {
    fn default() -> Self {
        Self {
            stdout: "Done.".to_string(),
            stderr: String::new(),
            exit_code: 0,
            delay: Duration::from_millis(10),
            context_increase: 5,
        }
    }
}

impl MockSession {
    /// Create a new mock session.
    pub fn new() -> Self {
        Self {
            id: SessionId::new(),
            responses: RwLock::new(vec![MockResponse::default()]),
            response_index: RwLock::new(0),
            context_usage: RwLock::new(0),
        }
    }

    /// Add a response.
    pub async fn add_response(&self, response: MockResponse) {
        self.responses.write().await.push(response);
    }

    /// Set responses.
    pub async fn set_responses(&self, responses: Vec<MockResponse>) {
        *self.responses.write().await = responses;
        *self.response_index.write().await = 0;
    }

    /// Get next response.
    pub async fn next_response(&self) -> MockResponse {
        let responses = self.responses.read().await;
        let mut index = self.response_index.write().await;

        let response = responses.get(*index).cloned().unwrap_or_default();
        *index = (*index + 1) % responses.len().max(1);

        // Update context usage
        let mut usage = self.context_usage.write().await;
        *usage = (*usage + response.context_increase).min(100);

        response
    }

    /// Get context usage.
    pub async fn context_usage(&self) -> u8 {
        *self.context_usage.read().await
    }

    /// Reset context.
    pub async fn reset_context(&self) {
        *self.context_usage.write().await = 0;
    }
}

/// Builder for creating test iterations.
pub struct IterationBuilder {
    iteration: Iteration,
}

impl IterationBuilder {
    pub fn new(number: u32) -> Self {
        Self {
            iteration: Iteration::new(number, "Test prompt".to_string()),
        }
    }

    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.iteration.prompt = prompt.to_string();
        self
    }

    pub fn with_result(mut self, result: IterationResult) -> Self {
        self.iteration.result = Some(result);
        self.iteration.status = tachikoma_loop_runner::iteration::IterationStatus::Completed;
        self
    }

    pub fn failed(mut self, error: &str) -> Self {
        self.iteration.fail(error);
        self
    }

    pub fn build(self) -> Iteration {
        self.iteration
    }
}

/// Builder for creating test iteration results.
pub struct IterationResultBuilder {
    result: IterationResult,
}

impl IterationResultBuilder {
    pub fn new() -> Self {
        Self {
            result: IterationResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                files_modified: vec![],
                files_created: vec![],
                files_deleted: vec![],
                tests_run: false,
                test_results: None,
                context_usage_percent: 10,
                progress_detected: true,
                summary: None,
                duration_ms: 100,
            },
        }
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.result.exit_code = code;
        self
    }

    pub fn with_stdout(mut self, stdout: &str) -> Self {
        self.result.stdout = stdout.to_string();
        self
    }

    pub fn with_files_modified(mut self, files: Vec<&str>) -> Self {
        self.result.files_modified = files.into_iter().map(PathBuf::from).collect();
        self
    }

    pub fn with_test_results(mut self, passed: u32, failed: u32) -> Self {
        self.result.tests_run = true;
        self.result.test_results = Some(TestResults {
            total: passed + failed,
            passed,
            failed,
            skipped: 0,
            details: vec![],
        });
        self
    }

    pub fn with_context_usage(mut self, percent: u8) -> Self {
        self.result.context_usage_percent = percent;
        self
    }

    pub fn with_progress(mut self, progress: bool) -> Self {
        self.result.progress_detected = progress;
        self
    }

    pub fn build(self) -> IterationResult {
        self.result
    }
}

/// Assert helpers.
pub mod assertions {
    use super::*;

    /// Assert loop completed successfully.
    pub async fn assert_loop_completed(runner: &LoopRunner) {
        let state = runner.state().await;
        assert!(
            matches!(state, LoopState::Completed),
            "Expected Completed, got {:?}",
            state
        );
    }

    /// Assert loop stopped.
    pub async fn assert_loop_stopped(runner: &LoopRunner) {
        let state = runner.state().await;
        assert!(
            matches!(state, LoopState::Stopped),
            "Expected Stopped, got {:?}",
            state
        );
    }

    /// Assert iteration count.
    pub fn assert_iterations(stats: &tachikoma_loop_runner::state::LoopStatsSnapshot, expected: u32) {
        assert_eq!(
            stats.iterations, expected,
            "Expected {} iterations, got {}",
            expected, stats.iterations
        );
    }

    /// Assert no failures.
    pub fn assert_no_failures(stats: &tachikoma_loop_runner::state::LoopStatsSnapshot) {
        assert_eq!(
            stats.failed_iterations, 0,
            "Expected no failures, got {}",
            stats.failed_iterations
        );
    }
}
```

### 2. Unit Tests (tests/unit/mod.rs)

```rust
//! Unit tests for loop components.

mod test_loop_state;
mod test_stop_conditions;
mod test_progress_detection;
mod test_session;
mod test_hooks;

// Re-export for convenience
pub use test_loop_state::*;
pub use test_stop_conditions::*;
```

### 3. State Unit Tests (tests/unit/test_loop_state.rs)

```rust
//! Unit tests for loop state management.

use tachikoma_loop_runner::state::{LoopContext, LoopState, LoopStats};
use tachikoma_loop_runner::LoopId;

#[test]
fn test_loop_state_transitions() {
    // Test valid transitions
    assert!(LoopState::Idle.can_start());
    assert!(LoopState::Running.can_pause());
    assert!(LoopState::Paused.can_resume());

    // Test invalid transitions
    assert!(!LoopState::Running.can_start());
    assert!(!LoopState::Paused.can_pause());
    assert!(!LoopState::Completed.can_resume());
}

#[test]
fn test_loop_state_terminal() {
    assert!(LoopState::Completed.is_terminal());
    assert!(LoopState::Error.is_terminal());
    assert!(LoopState::Stopped.is_terminal());

    assert!(!LoopState::Running.is_terminal());
    assert!(!LoopState::Paused.is_terminal());
    assert!(!LoopState::Idle.is_terminal());
}

#[test]
fn test_loop_stats_increment() {
    let stats = LoopStats::default();

    stats.increment_iterations();
    stats.increment_iterations();

    let snapshot = stats.snapshot();
    assert_eq!(snapshot.iterations, 2);
}

#[test]
fn test_loop_stats_success_resets_streak() {
    let stats = LoopStats::default();

    stats.record_failure();
    stats.record_failure();
    assert_eq!(stats.get_failure_streak(), 2);

    stats.record_success();
    assert_eq!(stats.get_failure_streak(), 0);
}

#[tokio::test]
async fn test_loop_context_state_changes() {
    let context = LoopContext::new(LoopId::new());

    assert_eq!(context.get_state().await, LoopState::Idle);

    context.set_state(LoopState::Running).await;
    assert_eq!(context.get_state().await, LoopState::Running);

    context.mark_started().await;
    assert!(context.started_at.read().await.is_some());
}
```

### 4. Stop Condition Tests (tests/unit/test_stop_conditions.rs)

```rust
//! Unit tests for stop conditions.

use tachikoma_loop_runner::stop::{
    evaluate_condition, StopCondition, StopConditionContext,
};
use std::path::PathBuf;
use std::time::Duration;

fn default_context() -> StopConditionContext {
    StopConditionContext {
        iteration: 5,
        start_time: chrono::Utc::now() - chrono::Duration::minutes(10),
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

#[test]
fn test_max_iterations_not_reached() {
    let condition = StopCondition::max_iterations(10);
    let ctx = default_context();

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(!result.is_met);
    assert_eq!(result.progress.unwrap(), 0.5); // 5/10
}

#[test]
fn test_max_iterations_reached() {
    let condition = StopCondition::max_iterations(5);
    let ctx = default_context();

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(result.is_met);
}

#[test]
fn test_test_failure_streak() {
    let condition = StopCondition::test_failure_streak(3);
    let mut ctx = default_context();
    ctx.test_failure_streak = 3;

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(result.is_met);
}

#[test]
fn test_output_pattern_string() {
    let condition = StopCondition::output_pattern("SUCCESS");
    let mut ctx = default_context();
    ctx.recent_output = "Task completed with SUCCESS".to_string();

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(result.is_met);
}

#[test]
fn test_output_pattern_no_match() {
    let condition = StopCondition::output_pattern("SUCCESS");
    let mut ctx = default_context();
    ctx.recent_output = "Task failed".to_string();

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(!result.is_met);
}

#[test]
fn test_composite_all() {
    let condition = StopCondition::max_iterations(10)
        .and(StopCondition::test_failure_streak(5));

    let ctx = default_context(); // iteration=5, streak=0

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(!result.is_met); // Neither condition met
}

#[test]
fn test_composite_any() {
    let condition = StopCondition::max_iterations(3)
        .or(StopCondition::test_failure_streak(5));

    let ctx = default_context(); // iteration=5

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(result.is_met); // First condition met (5 >= 3)
}

#[test]
fn test_not_condition() {
    let condition = StopCondition::max_iterations(10).not();
    let ctx = default_context(); // iteration=5

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(result.is_met); // NOT (5 >= 10) = true
}

#[test]
fn test_never_condition() {
    let condition = StopCondition::Never;
    let ctx = default_context();

    let result = evaluate_condition(&condition, &ctx).unwrap();
    assert!(!result.is_met);
}
```

### 5. Integration Tests (tests/integration/mod.rs)

```rust
//! Integration tests for loop components.

mod test_loop_execution;
mod test_session_lifecycle;
mod test_reboot_cycle;
```

### 6. Loop Execution Integration Tests (tests/integration/test_loop_execution.rs)

```rust
//! Integration tests for loop execution.

use crate::utils::{assertions, IterationResultBuilder, MockSession, TestContext};
use tachikoma_loop_runner::{LoopRunner, LoopConfig};
use std::time::Duration;

#[tokio::test]
async fn test_loop_runs_to_max_iterations() {
    let ctx = TestContext::new();
    ctx.create_prompt("Test prompt").await;

    let mut config = ctx.config.clone();
    config.max_iterations = 5;

    let runner = LoopRunner::new(config);
    runner.run().await.unwrap();

    let stats = runner.stats();
    assertions::assert_iterations(&stats, 5);
}

#[tokio::test]
async fn test_loop_stops_on_command() {
    let ctx = TestContext::new();
    ctx.create_prompt("Test prompt").await;

    let mut config = ctx.config.clone();
    config.max_iterations = 100;
    config.iteration_delay = Duration::from_millis(50);

    let runner = LoopRunner::new(config);
    let cmd_sender = runner.command_sender();

    // Run in background
    let runner_handle = tokio::spawn({
        let runner = runner;
        async move {
            runner.run().await
        }
    });

    // Stop after a short delay
    tokio::time::sleep(Duration::from_millis(200)).await;
    cmd_sender.send(tachikoma_loop_runner::LoopCommand::Stop).await.unwrap();

    let result = runner_handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_loop_pause_resume() {
    let ctx = TestContext::new();
    ctx.create_prompt("Test prompt").await;

    let mut config = ctx.config.clone();
    config.max_iterations = 100;
    config.iteration_delay = Duration::from_millis(10);

    let runner = LoopRunner::new(config);
    let cmd_sender = runner.command_sender();

    let runner_handle = tokio::spawn({
        let runner = runner;
        async move {
            runner.run().await
        }
    });

    // Pause
    tokio::time::sleep(Duration::from_millis(50)).await;
    cmd_sender.send(tachikoma_loop_runner::LoopCommand::Pause).await.unwrap();

    // Wait while paused
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Resume
    cmd_sender.send(tachikoma_loop_runner::LoopCommand::Resume).await.unwrap();

    // Let it run more
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Stop
    cmd_sender.send(tachikoma_loop_runner::LoopCommand::Stop).await.unwrap();

    runner_handle.await.unwrap().unwrap();
}
```

### 7. Benchmark Tests (benches/loop_benchmarks.rs)

```rust
//! Performance benchmarks for loop operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tachikoma_loop_runner::stop::{
    evaluate_condition, StopCondition, StopConditionContext,
};
use std::path::PathBuf;

fn benchmark_stop_condition_evaluation(c: &mut Criterion) {
    let condition = StopCondition::max_iterations(100)
        .and(StopCondition::test_failure_streak(5))
        .or(StopCondition::output_pattern("DONE"));

    let ctx = StopConditionContext {
        iteration: 50,
        start_time: chrono::Utc::now(),
        current_time: chrono::Utc::now(),
        recent_output: "Some output here".to_string(),
        test_failure_streak: 2,
        iterations_since_progress: 3,
        passed_tests: vec!["test1".to_string(), "test2".to_string()],
        failed_tests: vec![],
        has_error: false,
        error_message: None,
        user_signal: false,
        working_dir: PathBuf::from("."),
    };

    c.bench_function("evaluate_composite_condition", |b| {
        b.iter(|| {
            let result = evaluate_condition(black_box(&condition), black_box(&ctx));
            black_box(result)
        })
    });
}

fn benchmark_state_transitions(c: &mut Criterion) {
    use tachikoma_loop_runner::state::LoopStats;

    c.bench_function("stats_record_iteration", |b| {
        let stats = LoopStats::default();
        b.iter(|| {
            stats.increment_iterations();
            stats.record_success();
            black_box(stats.snapshot())
        })
    });
}

criterion_group!(
    benches,
    benchmark_stop_condition_evaluation,
    benchmark_state_transitions,
);
criterion_main!(benches);
```

### 8. Test Configuration (tests/lib.rs)

```rust
//! Test configuration and common setup.

pub mod utils;

#[cfg(test)]
mod unit;

#[cfg(test)]
mod integration;

/// Setup test logging.
pub fn setup_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
}
```

---

## Testing Requirements

1. All unit tests pass
2. Integration tests verify component interaction
3. E2E tests complete full loop cycles
4. Mock session enables testing without real Claude
5. Property tests verify state machine invariants
6. Benchmarks provide performance baseline
7. Test coverage > 80%
8. CI runs all tests on each commit

---

## Related Specs

- Depends on: All Phase 5 specs
- Integrates with: [008-test-infrastructure.md](../phase-00-setup/008-test-infrastructure.md)
- Related: [009-ci-pipeline.md](../phase-00-setup/009-ci-pipeline.md)
