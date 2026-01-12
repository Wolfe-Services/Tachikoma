# 109 - Loop State Persistence

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 109
**Status:** Planned
**Dependencies:** 096-loop-runner-core
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement loop state persistence for the Ralph Loop - saving and restoring loop state to enable resumption after crashes, restarts, or intentional pauses.

---

## Acceptance Criteria

- [ ] Serialize complete loop state
- [ ] Persistent storage to disk
- [ ] State versioning for compatibility
- [ ] Atomic state updates
- [ ] State recovery on startup
- [ ] Corruption detection and recovery
- [ ] State cleanup and rotation
- [ ] Optional cloud backup

---

## Implementation Details

### 1. State Types (src/state/types.rs)

```rust
//! State persistence types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete loop state for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedLoopState {
    /// State format version.
    pub version: u32,
    /// Loop identifier.
    pub loop_id: String,
    /// When this state was saved.
    pub saved_at: chrono::DateTime<chrono::Utc>,
    /// Core loop state.
    pub core: CoreLoopState,
    /// Session state.
    pub session: SessionState,
    /// Progress state.
    pub progress: ProgressState,
    /// Test state.
    pub tests: TestState,
    /// Stop condition state.
    pub stop_conditions: StopConditionState,
    /// Metrics snapshot.
    pub metrics: MetricsSnapshot,
    /// Custom state data.
    pub custom: HashMap<String, serde_json::Value>,
}

/// Current state format version.
pub const STATE_VERSION: u32 = 1;

/// Core loop execution state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreLoopState {
    /// Current loop status.
    pub status: LoopStatus,
    /// Current iteration number.
    pub iteration: u32,
    /// Loop start time.
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Last activity time.
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Working directory.
    pub working_dir: PathBuf,
    /// Prompt file path.
    pub prompt_path: PathBuf,
    /// Configuration hash (for detecting changes).
    pub config_hash: String,
}

/// Loop execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopStatus {
    Running,
    Paused,
    Rebooting,
    Stopped,
    Completed,
    Error,
}

/// Session state for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Current session ID.
    pub current_session_id: Option<String>,
    /// Session creation time.
    pub session_started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Iterations in current session.
    pub session_iterations: u32,
    /// Context usage.
    pub context_usage: u8,
    /// Session history (last N sessions).
    pub session_history: Vec<SessionHistoryEntry>,
}

/// Session history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistoryEntry {
    pub session_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: chrono::DateTime<chrono::Utc>,
    pub iterations: u32,
    pub end_reason: String,
}

/// Progress tracking state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressState {
    /// Iterations since last progress.
    pub iterations_since_progress: u32,
    /// Last progress iteration.
    pub last_progress_iteration: Option<u32>,
    /// Recent progress scores.
    pub recent_scores: Vec<f64>,
    /// Files modified in this run.
    pub modified_files: Vec<PathBuf>,
    /// Completed objectives.
    pub completed_objectives: Vec<String>,
    /// Pending objectives.
    pub pending_objectives: Vec<String>,
}

/// Test tracking state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestState {
    /// Current failure streak.
    pub failure_streak: u32,
    /// Passing test names.
    pub passing_tests: Vec<String>,
    /// Failing test names.
    pub failing_tests: Vec<String>,
    /// Flaky test names.
    pub flaky_tests: Vec<String>,
    /// Recently fixed tests.
    pub recently_fixed: Vec<String>,
}

/// Stop condition evaluation state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopConditionState {
    /// Progress toward each condition.
    pub condition_progress: HashMap<String, f64>,
    /// Last evaluation results.
    pub last_evaluation: Option<String>,
}

/// Metrics snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_iterations: u64,
    pub successful_iterations: u64,
    pub failed_iterations: u64,
    pub total_reboots: u64,
    pub total_execution_ms: u64,
}

/// Configuration for state persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePersistenceConfig {
    /// Enable state persistence.
    pub enabled: bool,
    /// State file path.
    pub state_path: PathBuf,
    /// Backup directory.
    pub backup_dir: PathBuf,
    /// Auto-save interval.
    #[serde(with = "humantime_serde")]
    pub auto_save_interval: std::time::Duration,
    /// Maximum backups to keep.
    pub max_backups: usize,
    /// Enable compression.
    pub compress: bool,
    /// Enable encryption (requires key).
    pub encrypt: bool,
}

impl Default for StatePersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            state_path: PathBuf::from(".ralph/state.json"),
            backup_dir: PathBuf::from(".ralph/backups"),
            auto_save_interval: std::time::Duration::from_secs(30),
            max_backups: 10,
            compress: false,
            encrypt: false,
        }
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

### 2. State Manager (src/state/manager.rs)

```rust
//! State persistence manager.

use super::types::{
    CoreLoopState, LoopStatus, MetricsSnapshot, PersistedLoopState, ProgressState,
    SessionHistoryEntry, SessionState, StatePersistenceConfig, StopConditionState,
    TestState, STATE_VERSION,
};
use crate::error::{LoopError, LoopResult};

use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manages loop state persistence.
pub struct StateManager {
    /// Configuration.
    config: StatePersistenceConfig,
    /// Current state.
    state: Arc<RwLock<Option<PersistedLoopState>>>,
    /// Whether state has unsaved changes.
    dirty: Arc<std::sync::atomic::AtomicBool>,
}

impl StateManager {
    /// Create a new state manager.
    pub fn new(config: StatePersistenceConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(None)),
            dirty: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Initialize state (load existing or create new).
    pub async fn initialize(&self, loop_id: &str) -> LoopResult<Option<PersistedLoopState>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Try to load existing state
        if self.config.state_path.exists() {
            match self.load().await {
                Ok(Some(state)) => {
                    // Verify it's for the same loop
                    if state.loop_id == loop_id {
                        info!("Loaded existing state for loop {}", loop_id);
                        *self.state.write().await = Some(state.clone());
                        return Ok(Some(state));
                    } else {
                        warn!(
                            "State file is for different loop ({}), starting fresh",
                            state.loop_id
                        );
                    }
                }
                Ok(None) => {
                    debug!("No existing state found");
                }
                Err(e) => {
                    warn!("Failed to load state, starting fresh: {}", e);
                }
            }
        }

        // Create new state
        let state = self.create_initial_state(loop_id);
        *self.state.write().await = Some(state.clone());
        self.save().await?;

        Ok(None) // No existing state loaded
    }

    /// Create initial state.
    fn create_initial_state(&self, loop_id: &str) -> PersistedLoopState {
        let now = chrono::Utc::now();
        PersistedLoopState {
            version: STATE_VERSION,
            loop_id: loop_id.to_string(),
            saved_at: now,
            core: CoreLoopState {
                status: LoopStatus::Running,
                iteration: 0,
                started_at: now,
                last_activity: now,
                working_dir: std::env::current_dir().unwrap_or_default(),
                prompt_path: std::path::PathBuf::from("prompt.md"),
                config_hash: String::new(),
            },
            session: SessionState {
                current_session_id: None,
                session_started_at: None,
                session_iterations: 0,
                context_usage: 0,
                session_history: vec![],
            },
            progress: ProgressState::default(),
            tests: TestState::default(),
            stop_conditions: StopConditionState::default(),
            metrics: MetricsSnapshot::default(),
            custom: std::collections::HashMap::new(),
        }
    }

    /// Load state from disk.
    async fn load(&self) -> LoopResult<Option<PersistedLoopState>> {
        if !self.config.state_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&self.config.state_path)
            .await
            .map_err(|e| LoopError::StateLoadFailed {
                path: self.config.state_path.clone(),
                source: e.to_string(),
            })?;

        let state: PersistedLoopState = serde_json::from_str(&content)
            .map_err(|e| LoopError::StateLoadFailed {
                path: self.config.state_path.clone(),
                source: e.to_string(),
            })?;

        // Check version compatibility
        if state.version > STATE_VERSION {
            return Err(LoopError::StateVersionMismatch {
                expected: STATE_VERSION,
                found: state.version,
            });
        }

        Ok(Some(state))
    }

    /// Save state to disk.
    pub async fn save(&self) -> LoopResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let state_guard = self.state.read().await;
        let state = match state_guard.as_ref() {
            Some(s) => s,
            None => return Ok(()),
        };

        // Update saved_at
        let mut state = state.clone();
        state.saved_at = chrono::Utc::now();

        // Ensure directory exists
        if let Some(parent) = self.config.state_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| LoopError::StateSaveFailed {
                    path: self.config.state_path.clone(),
                    source: e.to_string(),
                })?;
        }

        // Serialize
        let content = serde_json::to_string_pretty(&state)
            .map_err(|e| LoopError::StateSaveFailed {
                path: self.config.state_path.clone(),
                source: e.to_string(),
            })?;

        // Write atomically (write to temp, then rename)
        let temp_path = self.config.state_path.with_extension("tmp");
        tokio::fs::write(&temp_path, &content)
            .await
            .map_err(|e| LoopError::StateSaveFailed {
                path: self.config.state_path.clone(),
                source: e.to_string(),
            })?;

        tokio::fs::rename(&temp_path, &self.config.state_path)
            .await
            .map_err(|e| LoopError::StateSaveFailed {
                path: self.config.state_path.clone(),
                source: e.to_string(),
            })?;

        self.dirty.store(false, std::sync::atomic::Ordering::Relaxed);
        debug!("State saved to {:?}", self.config.state_path);

        Ok(())
    }

    /// Create a backup.
    pub async fn backup(&self) -> LoopResult<PathBuf> {
        if !self.config.state_path.exists() {
            return Err(LoopError::NoStateToBackup);
        }

        // Ensure backup directory exists
        tokio::fs::create_dir_all(&self.config.backup_dir)
            .await
            .map_err(|e| LoopError::BackupFailed { source: e.to_string() })?;

        // Create backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("state_{}.json", timestamp);
        let backup_path = self.config.backup_dir.join(backup_name);

        // Copy state file
        tokio::fs::copy(&self.config.state_path, &backup_path)
            .await
            .map_err(|e| LoopError::BackupFailed { source: e.to_string() })?;

        // Cleanup old backups
        self.cleanup_old_backups().await?;

        info!("Created backup at {:?}", backup_path);
        Ok(backup_path)
    }

    /// Cleanup old backups.
    async fn cleanup_old_backups(&self) -> LoopResult<()> {
        let mut entries: Vec<_> = tokio::fs::read_dir(&self.config.backup_dir)
            .await
            .map_err(|e| LoopError::BackupFailed { source: e.to_string() })?
            .filter_map(|e| async { e.ok() })
            .collect::<Vec<_>>()
            .await;

        // Sort by modification time (oldest first)
        entries.sort_by_key(|e| {
            e.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        // Remove oldest if over limit
        while entries.len() > self.config.max_backups {
            if let Some(entry) = entries.first() {
                tokio::fs::remove_file(entry.path()).await.ok();
                entries.remove(0);
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Update state with a modifier function.
    pub async fn update<F>(&self, modifier: F) -> LoopResult<()>
    where
        F: FnOnce(&mut PersistedLoopState),
    {
        let mut state_guard = self.state.write().await;
        if let Some(state) = state_guard.as_mut() {
            modifier(state);
            self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    /// Update core state.
    pub async fn update_core(&self, status: LoopStatus, iteration: u32) -> LoopResult<()> {
        self.update(|state| {
            state.core.status = status;
            state.core.iteration = iteration;
            state.core.last_activity = chrono::Utc::now();
        })
        .await
    }

    /// Update session state.
    pub async fn update_session(
        &self,
        session_id: Option<&str>,
        context_usage: u8,
    ) -> LoopResult<()> {
        self.update(|state| {
            state.session.current_session_id = session_id.map(String::from);
            state.session.context_usage = context_usage;
            if session_id.is_some() {
                state.session.session_iterations += 1;
            }
        })
        .await
    }

    /// Update progress state.
    pub async fn update_progress(&self, made_progress: bool, score: f64) -> LoopResult<()> {
        self.update(|state| {
            if made_progress {
                state.progress.iterations_since_progress = 0;
                state.progress.last_progress_iteration = Some(state.core.iteration);
            } else {
                state.progress.iterations_since_progress += 1;
            }
            state.progress.recent_scores.push(score);
            if state.progress.recent_scores.len() > 20 {
                state.progress.recent_scores.remove(0);
            }
        })
        .await
    }

    /// Update test state.
    pub async fn update_tests(&self, failure_streak: u32, passing: Vec<String>, failing: Vec<String>) -> LoopResult<()> {
        self.update(|state| {
            state.tests.failure_streak = failure_streak;
            state.tests.passing_tests = passing;
            state.tests.failing_tests = failing;
        })
        .await
    }

    /// Get current state.
    pub async fn get_state(&self) -> Option<PersistedLoopState> {
        self.state.read().await.clone()
    }

    /// Check if state has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Clear state.
    pub async fn clear(&self) -> LoopResult<()> {
        *self.state.write().await = None;

        if self.config.state_path.exists() {
            tokio::fs::remove_file(&self.config.state_path)
                .await
                .map_err(|e| LoopError::StateSaveFailed {
                    path: self.config.state_path.clone(),
                    source: e.to_string(),
                })?;
        }

        Ok(())
    }

    /// Start auto-save task.
    pub fn start_auto_save(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval = self.config.auto_save_interval;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                if self.is_dirty() {
                    if let Err(e) = self.save().await {
                        error!("Auto-save failed: {}", e);
                    }
                }
            }
        })
    }
}

use futures::StreamExt;
```

### 3. Module Root (src/state/mod.rs)

```rust
//! Loop state persistence.

pub mod manager;
pub mod types;

pub use manager::StateManager;
pub use types::{
    CoreLoopState, LoopStatus, MetricsSnapshot, PersistedLoopState, ProgressState,
    SessionHistoryEntry, SessionState, StatePersistenceConfig, StopConditionState,
    TestState, STATE_VERSION,
};
```

---

## Testing Requirements

1. State serializes and deserializes correctly
2. Atomic writes prevent corruption
3. Version mismatch is detected
4. Backups are created correctly
5. Old backups are cleaned up
6. Auto-save triggers on changes
7. State recovery works after crash
8. Clear removes all state

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Next: [110-attended-mode.md](110-attended-mode.md)
- Related: [101-fresh-context.md](101-fresh-context.md)
