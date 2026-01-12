# 106 - Test Failure Tracking

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 106
**Status:** Planned
**Dependencies:** 104-stop-conditions, 097-loop-iteration
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement test failure tracking for the Ralph Loop - monitoring test execution results across iterations, tracking failure streaks, identifying flaky tests, and providing actionable insights.

---

## Acceptance Criteria

- [ ] Track test results across iterations
- [ ] Maintain failure streak counter
- [ ] Identify newly failing tests
- [ ] Identify fixed tests
- [ ] Detect flaky tests
- [ ] Generate test trend reports
- [ ] Integration with stop conditions
- [ ] Persist test history

---

## Implementation Details

### 1. Test Tracking Types (src/testing/types.rs)

```rust
//! Test tracking type definitions.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A tracked test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedTest {
    /// Test identifier.
    pub id: String,
    /// Test name.
    pub name: String,
    /// Test file/module.
    pub file: Option<String>,
    /// Current status.
    pub status: TestStatus,
    /// Number of consecutive passes.
    pub pass_streak: u32,
    /// Number of consecutive failures.
    pub fail_streak: u32,
    /// Total runs.
    pub total_runs: u32,
    /// Total passes.
    pub total_passes: u32,
    /// Total failures.
    pub total_failures: u32,
    /// Is this test flaky?
    pub is_flaky: bool,
    /// Last error message.
    pub last_error: Option<String>,
    /// History of recent results.
    pub recent_history: Vec<TestResultEntry>,
}

/// Status of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    /// Test is passing.
    Passing,
    /// Test is failing.
    Failing,
    /// Test was skipped.
    Skipped,
    /// Test status unknown.
    Unknown,
}

/// A single test result entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultEntry {
    /// When the test ran.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Iteration number.
    pub iteration: u32,
    /// Whether it passed.
    pub passed: bool,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Error message if failed.
    pub error: Option<String>,
}

impl TrackedTest {
    /// Create a new tracked test.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            file: None,
            status: TestStatus::Unknown,
            pass_streak: 0,
            fail_streak: 0,
            total_runs: 0,
            total_passes: 0,
            total_failures: 0,
            is_flaky: false,
            last_error: None,
            recent_history: Vec::new(),
        }
    }

    /// Record a test result.
    pub fn record(&mut self, passed: bool, duration_ms: u64, error: Option<String>, iteration: u32) {
        self.total_runs += 1;

        if passed {
            self.total_passes += 1;
            self.pass_streak += 1;
            self.fail_streak = 0;
            self.status = TestStatus::Passing;
        } else {
            self.total_failures += 1;
            self.fail_streak += 1;
            self.pass_streak = 0;
            self.status = TestStatus::Failing;
            self.last_error = error.clone();
        }

        // Update flaky detection
        self.update_flaky_status();

        // Record in history
        self.recent_history.push(TestResultEntry {
            timestamp: chrono::Utc::now(),
            iteration,
            passed,
            duration_ms,
            error,
        });

        // Keep last 20 results
        if self.recent_history.len() > 20 {
            self.recent_history.remove(0);
        }
    }

    /// Update flaky status based on history.
    fn update_flaky_status(&mut self) {
        if self.recent_history.len() < 5 {
            return;
        }

        // Count transitions between pass/fail
        let mut transitions = 0;
        for window in self.recent_history.windows(2) {
            if window[0].passed != window[1].passed {
                transitions += 1;
            }
        }

        // If more than 20% of runs are transitions, mark as flaky
        let transition_rate = transitions as f64 / self.recent_history.len() as f64;
        self.is_flaky = transition_rate > 0.2;
    }

    /// Get pass rate.
    pub fn pass_rate(&self) -> f64 {
        if self.total_runs == 0 {
            0.0
        } else {
            self.total_passes as f64 / self.total_runs as f64
        }
    }

    /// Check if recently fixed.
    pub fn is_recently_fixed(&self) -> bool {
        if self.recent_history.len() < 2 {
            return false;
        }

        // Was failing, now passing
        let prev = &self.recent_history[self.recent_history.len() - 2];
        let curr = &self.recent_history[self.recent_history.len() - 1];
        !prev.passed && curr.passed
    }

    /// Check if recently broken.
    pub fn is_recently_broken(&self) -> bool {
        if self.recent_history.len() < 2 {
            return false;
        }

        // Was passing, now failing
        let prev = &self.recent_history[self.recent_history.len() - 2];
        let curr = &self.recent_history[self.recent_history.len() - 1];
        prev.passed && !curr.passed
    }
}

/// Summary of test state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestSummary {
    /// Total tests tracked.
    pub total_tests: u32,
    /// Currently passing tests.
    pub passing: u32,
    /// Currently failing tests.
    pub failing: u32,
    /// Skipped tests.
    pub skipped: u32,
    /// Flaky tests.
    pub flaky: u32,
    /// Current failure streak (max across all tests).
    pub failure_streak: u32,
    /// Tests that were recently fixed.
    pub recently_fixed: Vec<String>,
    /// Tests that recently broke.
    pub recently_broken: Vec<String>,
}

/// Configuration for test tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTrackingConfig {
    /// Enable test tracking.
    pub enabled: bool,
    /// Framework to parse (auto-detect if None).
    pub framework: Option<TestFramework>,
    /// Maximum tests to track.
    pub max_tests: usize,
    /// Flaky threshold (transition rate).
    pub flaky_threshold: f64,
    /// Persist test history.
    pub persist: bool,
    /// Persistence path.
    pub persist_path: Option<std::path::PathBuf>,
}

impl Default for TestTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            framework: None,
            max_tests: 1000,
            flaky_threshold: 0.2,
            persist: true,
            persist_path: None,
        }
    }
}

/// Test framework type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestFramework {
    /// Rust/Cargo tests.
    Cargo,
    /// Jest (JavaScript).
    Jest,
    /// pytest (Python).
    Pytest,
    /// Go test.
    GoTest,
    /// JUnit (Java).
    JUnit,
    /// RSpec (Ruby).
    RSpec,
    /// Custom/other.
    Custom,
}
```

### 2. Test Tracker (src/testing/tracker.rs)

```rust
//! Test result tracking.

use super::types::{
    TestFramework, TestStatus, TestSummary, TestTrackingConfig, TrackedTest,
};
use crate::error::{LoopError, LoopResult};
use crate::iteration::TestResults;

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Tracks test results across iterations.
pub struct TestTracker {
    /// Configuration.
    config: TestTrackingConfig,
    /// Tracked tests by ID.
    tests: RwLock<HashMap<String, TrackedTest>>,
    /// Global failure streak.
    failure_streak: std::sync::atomic::AtomicU32,
    /// Last iteration's summary.
    last_summary: RwLock<Option<TestSummary>>,
}

impl TestTracker {
    /// Create a new test tracker.
    pub fn new(config: TestTrackingConfig) -> Self {
        Self {
            config,
            tests: RwLock::new(HashMap::new()),
            failure_streak: std::sync::atomic::AtomicU32::new(0),
            last_summary: RwLock::new(None),
        }
    }

    /// Process test results from an iteration.
    pub async fn process_results(
        &self,
        results: &TestResults,
        iteration: u32,
    ) -> LoopResult<TestSummary> {
        if !self.config.enabled {
            return Ok(TestSummary::default());
        }

        let mut tests = self.tests.write().await;

        // Update global failure streak
        if results.failed > 0 {
            self.failure_streak.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.failure_streak.store(0, std::sync::atomic::Ordering::Relaxed);
        }

        // Process individual test results
        for detail in &results.details {
            let test_id = self.make_test_id(&detail.name);

            let test = tests.entry(test_id.clone()).or_insert_with(|| {
                TrackedTest::new(&test_id, &detail.name)
            });

            let passed = matches!(detail.status, crate::iteration::TestStatus::Passed);
            test.record(passed, detail.duration_ms, detail.error.clone(), iteration);
        }

        // Enforce max tests limit
        if tests.len() > self.config.max_tests {
            self.prune_old_tests(&mut tests);
        }

        // Generate summary
        let summary = self.generate_summary(&tests);

        // Save last summary
        *self.last_summary.write().await = Some(summary.clone());

        // Persist if enabled
        if self.config.persist {
            self.persist(&tests).await?;
        }

        Ok(summary)
    }

    /// Parse test output to extract results.
    pub async fn parse_output(&self, output: &str, iteration: u32) -> LoopResult<Option<TestResults>> {
        let framework = self.detect_framework(output);

        match framework {
            Some(TestFramework::Cargo) => self.parse_cargo_output(output),
            Some(TestFramework::Jest) => self.parse_jest_output(output),
            Some(TestFramework::Pytest) => self.parse_pytest_output(output),
            Some(TestFramework::GoTest) => self.parse_go_output(output),
            _ => Ok(None),
        }
    }

    /// Detect test framework from output.
    fn detect_framework(&self, output: &str) -> Option<TestFramework> {
        if self.config.framework.is_some() {
            return self.config.framework;
        }

        if output.contains("test result:") || output.contains("running ") && output.contains(" test") {
            Some(TestFramework::Cargo)
        } else if output.contains("PASS ") || output.contains("FAIL ") && output.contains("Tests:") {
            Some(TestFramework::Jest)
        } else if output.contains("passed") && output.contains("pytest") {
            Some(TestFramework::Pytest)
        } else if output.contains("--- PASS:") || output.contains("--- FAIL:") {
            Some(TestFramework::GoTest)
        } else {
            None
        }
    }

    /// Parse Cargo test output.
    fn parse_cargo_output(&self, output: &str) -> LoopResult<Option<TestResults>> {
        let mut results = TestResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            details: vec![],
        };

        // Parse individual test results
        let test_pattern = regex::Regex::new(r"test (\S+) \.\.\. (\w+)").unwrap();
        for cap in test_pattern.captures_iter(output) {
            let name = cap[1].to_string();
            let status_str = &cap[2];

            let status = match status_str {
                "ok" => crate::iteration::TestStatus::Passed,
                "FAILED" => crate::iteration::TestStatus::Failed,
                "ignored" => crate::iteration::TestStatus::Skipped,
                _ => crate::iteration::TestStatus::Error,
            };

            results.total += 1;
            match status {
                crate::iteration::TestStatus::Passed => results.passed += 1,
                crate::iteration::TestStatus::Failed => results.failed += 1,
                crate::iteration::TestStatus::Skipped => results.skipped += 1,
                _ => {}
            }

            results.details.push(crate::iteration::TestDetail {
                name,
                status,
                duration_ms: 0, // Would need to parse timing
                error: None,
            });
        }

        // Parse summary line
        let summary_pattern = regex::Regex::new(r"(\d+) passed; (\d+) failed; (\d+) ignored").unwrap();
        if let Some(cap) = summary_pattern.captures(output) {
            results.passed = cap[1].parse().unwrap_or(0);
            results.failed = cap[2].parse().unwrap_or(0);
            results.skipped = cap[3].parse().unwrap_or(0);
            results.total = results.passed + results.failed + results.skipped;
        }

        if results.total > 0 {
            Ok(Some(results))
        } else {
            Ok(None)
        }
    }

    /// Parse Jest test output.
    fn parse_jest_output(&self, output: &str) -> LoopResult<Option<TestResults>> {
        let mut results = TestResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            details: vec![],
        };

        // Parse summary line
        let summary_pattern = regex::Regex::new(r"Tests:\s+(?:(\d+) passed)?(?:,\s+)?(?:(\d+) failed)?(?:,\s+)?(?:(\d+) skipped)?(?:,\s+)?(\d+) total").unwrap();
        if let Some(cap) = summary_pattern.captures(output) {
            results.passed = cap.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            results.failed = cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            results.skipped = cap.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            results.total = cap.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        }

        if results.total > 0 {
            Ok(Some(results))
        } else {
            Ok(None)
        }
    }

    /// Parse pytest output.
    fn parse_pytest_output(&self, output: &str) -> LoopResult<Option<TestResults>> {
        let mut results = TestResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            details: vec![],
        };

        // Parse summary line
        let summary_pattern = regex::Regex::new(r"(\d+) passed(?:,\s+(\d+) failed)?(?:,\s+(\d+) skipped)?").unwrap();
        if let Some(cap) = summary_pattern.captures(output) {
            results.passed = cap.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            results.failed = cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            results.skipped = cap.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            results.total = results.passed + results.failed + results.skipped;
        }

        if results.total > 0 {
            Ok(Some(results))
        } else {
            Ok(None)
        }
    }

    /// Parse Go test output.
    fn parse_go_output(&self, output: &str) -> LoopResult<Option<TestResults>> {
        let mut results = TestResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            details: vec![],
        };

        // Count PASS/FAIL lines
        for line in output.lines() {
            if line.contains("--- PASS:") {
                results.passed += 1;
                results.total += 1;
            } else if line.contains("--- FAIL:") {
                results.failed += 1;
                results.total += 1;
            } else if line.contains("--- SKIP:") {
                results.skipped += 1;
                results.total += 1;
            }
        }

        if results.total > 0 {
            Ok(Some(results))
        } else {
            Ok(None)
        }
    }

    /// Generate test ID from name.
    fn make_test_id(&self, name: &str) -> String {
        // Normalize test name for consistent tracking
        name.trim()
            .replace("::", "__")
            .replace(' ', "_")
            .to_lowercase()
    }

    /// Prune old tests to stay under limit.
    fn prune_old_tests(&self, tests: &mut HashMap<String, TrackedTest>) {
        // Remove tests that haven't been run recently
        let mut to_remove: Vec<String> = vec![];

        for (id, test) in tests.iter() {
            if test.recent_history.is_empty() {
                to_remove.push(id.clone());
            }
        }

        // If still over limit, remove by oldest last run
        if tests.len() - to_remove.len() > self.config.max_tests {
            let mut by_age: Vec<_> = tests.iter().collect();
            by_age.sort_by(|a, b| {
                let a_time = a.1.recent_history.last().map(|e| e.timestamp);
                let b_time = b.1.recent_history.last().map(|e| e.timestamp);
                a_time.cmp(&b_time)
            });

            let to_remove_count = tests.len() - self.config.max_tests;
            for (id, _) in by_age.into_iter().take(to_remove_count) {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            tests.remove(&id);
        }
    }

    /// Generate summary from current state.
    fn generate_summary(&self, tests: &HashMap<String, TrackedTest>) -> TestSummary {
        let mut summary = TestSummary {
            total_tests: tests.len() as u32,
            failure_streak: self.failure_streak.load(std::sync::atomic::Ordering::Relaxed),
            ..Default::default()
        };

        for test in tests.values() {
            match test.status {
                TestStatus::Passing => summary.passing += 1,
                TestStatus::Failing => summary.failing += 1,
                TestStatus::Skipped => summary.skipped += 1,
                TestStatus::Unknown => {}
            }

            if test.is_flaky {
                summary.flaky += 1;
            }

            if test.is_recently_fixed() {
                summary.recently_fixed.push(test.name.clone());
            }

            if test.is_recently_broken() {
                summary.recently_broken.push(test.name.clone());
            }
        }

        summary
    }

    /// Persist test state.
    async fn persist(&self, tests: &HashMap<String, TrackedTest>) -> LoopResult<()> {
        let path = match &self.config.persist_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let data = serde_json::to_string_pretty(tests)
            .map_err(|e| LoopError::SerializationError { source: e.to_string() })?;

        tokio::fs::write(path, data)
            .await
            .map_err(|e| LoopError::FileSystemError { source: e })?;

        Ok(())
    }

    /// Get current failure streak.
    pub fn get_failure_streak(&self) -> u32 {
        self.failure_streak.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get last summary.
    pub async fn get_last_summary(&self) -> Option<TestSummary> {
        self.last_summary.read().await.clone()
    }

    /// Get all tracked tests.
    pub async fn get_tests(&self) -> HashMap<String, TrackedTest> {
        self.tests.read().await.clone()
    }

    /// Get failing tests.
    pub async fn get_failing_tests(&self) -> Vec<TrackedTest> {
        self.tests
            .read()
            .await
            .values()
            .filter(|t| t.status == TestStatus::Failing)
            .cloned()
            .collect()
    }

    /// Get flaky tests.
    pub async fn get_flaky_tests(&self) -> Vec<TrackedTest> {
        self.tests
            .read()
            .await
            .values()
            .filter(|t| t.is_flaky)
            .cloned()
            .collect()
    }

    /// Reset failure streak.
    pub fn reset_failure_streak(&self) {
        self.failure_streak.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}
```

### 3. Module Root (src/testing/mod.rs)

```rust
//! Test result tracking.

pub mod tracker;
pub mod types;

pub use tracker::TestTracker;
pub use types::{
    TestFramework, TestStatus, TestSummary, TestTrackingConfig, TrackedTest,
};
```

---

## Testing Requirements

1. Cargo test output parses correctly
2. Jest test output parses correctly
3. Pytest test output parses correctly
4. Failure streak increments on failures
5. Failure streak resets on success
6. Flaky detection works correctly
7. Recently fixed/broken detection works
8. Test limit pruning works

---

## Related Specs

- Depends on: [104-stop-conditions.md](104-stop-conditions.md)
- Depends on: [097-loop-iteration.md](097-loop-iteration.md)
- Next: [107-no-progress.md](107-no-progress.md)
- Related: [105-stop-evaluation.md](105-stop-evaluation.md)
