# 096c - Loop State Management

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 096c
**Status:** Planned
**Dependencies:** 096b-loop-config
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define loop state types including state machine, statistics tracking, and context for concurrent access.

---

## Acceptance Criteria

- [ ] `LoopState` enum with all states
- [ ] `LoopStats` with atomic counters
- [ ] `LoopStatsSnapshot` for serialization
- [ ] `LoopContext` for state management
- [ ] Thread-safe state access

---

## Implementation Details

### 1. State Types (src/state.rs)

```rust
//! Loop runner state management.

use crate::LoopId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Current state of the loop runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopState {
    /// Loop has not started.
    Idle,
    /// Loop is running.
    Running,
    /// Loop is paused.
    Paused,
    /// Loop is performing a reboot.
    Rebooting,
    /// Loop completed successfully.
    Completed,
    /// Loop stopped due to error.
    Error,
    /// Loop stopped by user.
    Stopped,
}

impl LoopState {
    /// Is the loop in a terminal state?
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Error | Self::Stopped)
    }

    /// Is the loop active (running or rebooting)?
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Rebooting)
    }

    /// Can the loop be started?
    pub fn can_start(&self) -> bool {
        matches!(self, Self::Idle | Self::Stopped | Self::Error)
    }

    /// Can the loop be paused?
    pub fn can_pause(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Can the loop be resumed?
    pub fn can_resume(&self) -> bool {
        matches!(self, Self::Paused)
    }
}

/// Runtime statistics for the loop.
#[derive(Debug)]
pub struct LoopStats {
    /// Total iterations completed.
    pub iterations: AtomicU32,
    /// Successful iterations.
    pub successful_iterations: AtomicU32,
    /// Failed iterations.
    pub failed_iterations: AtomicU32,
    /// Total reboots performed.
    pub reboots: AtomicU32,
    /// Total execution time in milliseconds.
    pub total_execution_ms: AtomicU64,
    /// Current test failure streak.
    pub test_failure_streak: AtomicU32,
    /// Iterations since last progress.
    pub no_progress_count: AtomicU32,
}

impl Default for LoopStats {
    fn default() -> Self {
        Self {
            iterations: AtomicU32::new(0),
            successful_iterations: AtomicU32::new(0),
            failed_iterations: AtomicU32::new(0),
            reboots: AtomicU32::new(0),
            total_execution_ms: AtomicU64::new(0),
            test_failure_streak: AtomicU32::new(0),
            no_progress_count: AtomicU32::new(0),
        }
    }
}

impl LoopStats {
    /// Increment iteration count.
    pub fn increment_iterations(&self) {
        self.iterations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful iteration.
    pub fn record_success(&self) {
        self.successful_iterations.fetch_add(1, Ordering::Relaxed);
        self.test_failure_streak.store(0, Ordering::Relaxed);
        self.no_progress_count.store(0, Ordering::Relaxed);
    }

    /// Record a failed iteration.
    pub fn record_failure(&self) {
        self.failed_iterations.fetch_add(1, Ordering::Relaxed);
        self.test_failure_streak.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a reboot.
    pub fn record_reboot(&self) {
        self.reboots.fetch_add(1, Ordering::Relaxed);
    }

    /// Record no progress.
    pub fn record_no_progress(&self) {
        self.no_progress_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current failure streak.
    pub fn get_failure_streak(&self) -> u32 {
        self.test_failure_streak.load(Ordering::Relaxed)
    }

    /// Create a snapshot of current stats.
    pub fn snapshot(&self) -> LoopStatsSnapshot {
        LoopStatsSnapshot {
            iterations: self.iterations.load(Ordering::Relaxed),
            successful_iterations: self.successful_iterations.load(Ordering::Relaxed),
            failed_iterations: self.failed_iterations.load(Ordering::Relaxed),
            reboots: self.reboots.load(Ordering::Relaxed),
            total_execution_ms: self.total_execution_ms.load(Ordering::Relaxed),
            test_failure_streak: self.test_failure_streak.load(Ordering::Relaxed),
            no_progress_count: self.no_progress_count.load(Ordering::Relaxed),
        }
    }
}

/// Serializable snapshot of loop stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStatsSnapshot {
    pub iterations: u32,
    pub successful_iterations: u32,
    pub failed_iterations: u32,
    pub reboots: u32,
    pub total_execution_ms: u64,
    pub test_failure_streak: u32,
    pub no_progress_count: u32,
}

/// Complete loop context including state and stats.
pub struct LoopContext {
    /// Unique loop identifier.
    pub id: LoopId,
    /// Current state.
    pub state: RwLock<LoopState>,
    /// Runtime statistics.
    pub stats: Arc<LoopStats>,
    /// Start time.
    pub started_at: RwLock<Option<DateTime<Utc>>>,
    /// Stop reason if stopped.
    pub stop_reason: RwLock<Option<String>>,
}

impl LoopContext {
    /// Create a new loop context.
    pub fn new(id: LoopId) -> Self {
        Self {
            id,
            state: RwLock::new(LoopState::Idle),
            stats: Arc::new(LoopStats::default()),
            started_at: RwLock::new(None),
            stop_reason: RwLock::new(None),
        }
    }

    /// Update the state.
    pub async fn set_state(&self, state: LoopState) {
        *self.state.write().await = state;
    }

    /// Get the current state.
    pub async fn get_state(&self) -> LoopState {
        *self.state.read().await
    }
}
```

---

## Testing Requirements

1. State transitions are valid
2. Statistics are correctly tracked
3. Concurrent access is safe

---

## Related Specs

- Depends on: [096b-loop-config.md](096b-loop-config.md)
- Next: [096d-loop-runner.md](096d-loop-runner.md)
