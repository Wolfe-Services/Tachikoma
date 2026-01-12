# 096 - Loop Runner Core

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 096
**Status:** Planned
**Dependencies:** 011-common-core-types, 019-async-runtime, 026-logging-infrastructure
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement the core Ralph Wiggum Loop Runner - an async loop executor that manages Claude Code sessions, handles context window management, and orchestrates the continuous development cycle with automatic reboots.

---

## Acceptance Criteria

- [ ] `tachikoma-loop-runner` crate created
- [ ] Core `LoopRunner` struct with async execution
- [ ] Configurable loop settings
- [ ] Loop lifecycle management (start, pause, resume, stop)
- [ ] Integration with session management
- [ ] Event emission for all loop state changes
- [ ] Graceful shutdown handling
- [ ] Loop execution history tracking

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-loop-runner/Cargo.toml)

```toml
[package]
name = "tachikoma-loop-runner"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Ralph Wiggum Loop Runner for continuous Claude Code execution"

[dependencies]
tachikoma-common-core.workspace = true
tachikoma-common-async.workspace = true
tachikoma-common-config.workspace = true
tachikoma-logging.workspace = true

tokio = { workspace = true, features = ["full", "sync", "time"] }
async-trait = "0.1"
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }

[dev-dependencies]
tokio-test = "0.4"
proptest.workspace = true
mockall = "0.11"
```

### 2. Loop Configuration (src/config.rs)

```rust
//! Loop runner configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the Ralph Loop Runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Working directory for the loop.
    pub working_dir: PathBuf,

    /// Path to the prompt.md file.
    pub prompt_path: PathBuf,

    /// Maximum iterations (0 = unlimited).
    pub max_iterations: u32,

    /// Delay between iterations.
    #[serde(with = "humantime_serde")]
    pub iteration_delay: Duration,

    /// Context redline threshold (percentage 0-100).
    pub context_redline_percent: u8,

    /// Enable attended mode (requires user confirmation).
    pub attended_mode: bool,

    /// Stop conditions configuration.
    pub stop_conditions: StopConditionsConfig,

    /// Session configuration.
    pub session: SessionConfig,
}

/// Stop conditions configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopConditionsConfig {
    /// Stop after N consecutive test failures.
    pub max_test_failure_streak: u32,

    /// Stop after N iterations with no progress.
    pub max_no_progress_iterations: u32,

    /// Stop on specific error patterns.
    pub stop_on_patterns: Vec<String>,

    /// Custom stop condition scripts.
    pub custom_scripts: Vec<PathBuf>,
}

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Claude Code command to execute.
    pub claude_command: String,

    /// Session timeout.
    #[serde(with = "humantime_serde")]
    pub session_timeout: Duration,

    /// Enable session persistence.
    pub persist_sessions: bool,

    /// Session state directory.
    pub state_dir: PathBuf,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            working_dir: PathBuf::from("."),
            prompt_path: PathBuf::from("prompt.md"),
            max_iterations: 0,
            iteration_delay: Duration::from_secs(5),
            context_redline_percent: 85,
            attended_mode: false,
            stop_conditions: StopConditionsConfig::default(),
            session: SessionConfig::default(),
        }
    }
}

impl Default for StopConditionsConfig {
    fn default() -> Self {
        Self {
            max_test_failure_streak: 3,
            max_no_progress_iterations: 5,
            stop_on_patterns: vec![],
            custom_scripts: vec![],
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            claude_command: "claude".to_string(),
            session_timeout: Duration::from_secs(3600),
            persist_sessions: true,
            state_dir: PathBuf::from(".ralph/sessions"),
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

### 3. Loop State (src/state.rs)

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

    /// Add execution time.
    pub fn add_execution_time(&self, ms: u64) {
        self.total_execution_ms.fetch_add(ms, Ordering::Relaxed);
    }

    /// Get current failure streak.
    pub fn get_failure_streak(&self) -> u32 {
        self.test_failure_streak.load(Ordering::Relaxed)
    }

    /// Get no progress count.
    pub fn get_no_progress_count(&self) -> u32 {
        self.no_progress_count.load(Ordering::Relaxed)
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
    /// Last activity time.
    pub last_activity: RwLock<DateTime<Utc>>,
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
            last_activity: RwLock::new(Utc::now()),
            stop_reason: RwLock::new(None),
        }
    }

    /// Update the state.
    pub async fn set_state(&self, state: LoopState) {
        *self.state.write().await = state;
        *self.last_activity.write().await = Utc::now();
    }

    /// Get the current state.
    pub async fn get_state(&self) -> LoopState {
        *self.state.read().await
    }

    /// Mark as started.
    pub async fn mark_started(&self) {
        *self.started_at.write().await = Some(Utc::now());
        self.set_state(LoopState::Running).await;
    }

    /// Set stop reason.
    pub async fn set_stop_reason(&self, reason: impl Into<String>) {
        *self.stop_reason.write().await = Some(reason.into());
    }
}
```

### 4. Core Loop Runner (src/runner.rs)

```rust
//! Core loop runner implementation.

use crate::config::LoopConfig;
use crate::error::{LoopError, LoopResult};
use crate::events::{LoopEvent, LoopEventEmitter};
use crate::state::{LoopContext, LoopState};
use crate::session::SessionManager;
use crate::stop::StopConditionEvaluator;
use crate::LoopId;

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, instrument, warn};

/// The Ralph Wiggum Loop Runner.
pub struct LoopRunner {
    /// Loop configuration.
    config: LoopConfig,
    /// Loop context.
    context: Arc<LoopContext>,
    /// Session manager.
    session_manager: Arc<SessionManager>,
    /// Stop condition evaluator.
    stop_evaluator: Arc<StopConditionEvaluator>,
    /// Event emitter.
    event_emitter: Arc<LoopEventEmitter>,
    /// Shutdown signal sender.
    shutdown_tx: broadcast::Sender<()>,
    /// Command receiver.
    command_rx: RwLock<Option<mpsc::Receiver<LoopCommand>>>,
    /// Command sender (for external control).
    command_tx: mpsc::Sender<LoopCommand>,
}

/// Commands that can be sent to the loop runner.
#[derive(Debug, Clone)]
pub enum LoopCommand {
    /// Pause the loop.
    Pause,
    /// Resume the loop.
    Resume,
    /// Stop the loop.
    Stop,
    /// Force a reboot.
    ForceReboot,
    /// Skip current iteration.
    SkipIteration,
}

impl LoopRunner {
    /// Create a new loop runner.
    pub fn new(config: LoopConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (command_tx, command_rx) = mpsc::channel(32);
        let loop_id = LoopId::new();

        Self {
            config: config.clone(),
            context: Arc::new(LoopContext::new(loop_id)),
            session_manager: Arc::new(SessionManager::new(config.session.clone())),
            stop_evaluator: Arc::new(StopConditionEvaluator::new(config.stop_conditions.clone())),
            event_emitter: Arc::new(LoopEventEmitter::new()),
            shutdown_tx,
            command_rx: RwLock::new(Some(command_rx)),
            command_tx,
        }
    }

    /// Get the loop ID.
    pub fn id(&self) -> LoopId {
        self.context.id
    }

    /// Get a command sender for external control.
    pub fn command_sender(&self) -> mpsc::Sender<LoopCommand> {
        self.command_tx.clone()
    }

    /// Subscribe to loop events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<LoopEvent> {
        self.event_emitter.subscribe()
    }

    /// Get current loop state.
    pub async fn state(&self) -> LoopState {
        self.context.get_state().await
    }

    /// Get loop statistics snapshot.
    pub fn stats(&self) -> crate::state::LoopStatsSnapshot {
        self.context.stats.snapshot()
    }

    /// Run the loop.
    #[instrument(skip(self), fields(loop_id = %self.context.id))]
    pub async fn run(&self) -> LoopResult<()> {
        info!("Starting Ralph Loop Runner");

        // Validate we can start
        let current_state = self.context.get_state().await;
        if !current_state.can_start() {
            return Err(LoopError::InvalidState {
                current: current_state,
                action: "start",
            });
        }

        // Take ownership of command receiver
        let mut command_rx = self.command_rx.write().await.take()
            .ok_or(LoopError::AlreadyRunning)?;

        // Mark as started
        self.context.mark_started().await;
        self.event_emitter.emit(LoopEvent::Started {
            loop_id: self.context.id,
            config: self.config.clone(),
        });

        // Subscribe to shutdown
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Main loop
        let result = self.run_loop(&mut command_rx, &mut shutdown_rx).await;

        // Handle result and set final state
        match &result {
            Ok(()) => {
                self.context.set_state(LoopState::Completed).await;
                self.event_emitter.emit(LoopEvent::Completed {
                    loop_id: self.context.id,
                    stats: self.context.stats.snapshot(),
                });
            }
            Err(e) => {
                self.context.set_state(LoopState::Error).await;
                self.context.set_stop_reason(e.to_string()).await;
                self.event_emitter.emit(LoopEvent::Error {
                    loop_id: self.context.id,
                    error: e.to_string(),
                });
            }
        }

        info!("Ralph Loop Runner finished");
        result
    }

    /// Internal loop execution.
    async fn run_loop(
        &self,
        command_rx: &mut mpsc::Receiver<LoopCommand>,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> LoopResult<()> {
        let mut iteration = 0u32;

        loop {
            // Check for commands or shutdown
            tokio::select! {
                biased;

                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal");
                    self.context.set_state(LoopState::Stopped).await;
                    self.context.set_stop_reason("Shutdown signal received").await;
                    return Ok(());
                }

                Some(cmd) = command_rx.recv() => {
                    if self.handle_command(cmd).await? {
                        return Ok(());
                    }
                    continue;
                }

                _ = tokio::time::sleep(std::time::Duration::ZERO) => {
                    // Proceed with iteration
                }
            }

            // Check if paused
            if self.context.get_state().await == LoopState::Paused {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }

            // Check max iterations
            if self.config.max_iterations > 0 && iteration >= self.config.max_iterations {
                info!("Reached maximum iterations: {}", self.config.max_iterations);
                return Ok(());
            }

            // Check stop conditions
            if let Some(reason) = self.stop_evaluator.evaluate(&self.context.stats).await {
                warn!("Stop condition triggered: {}", reason);
                self.context.set_stop_reason(&reason).await;
                self.event_emitter.emit(LoopEvent::StopConditionTriggered {
                    loop_id: self.context.id,
                    reason: reason.clone(),
                });
                return Ok(());
            }

            // Execute iteration
            iteration += 1;
            debug!("Starting iteration {}", iteration);

            self.event_emitter.emit(LoopEvent::IterationStarted {
                loop_id: self.context.id,
                iteration,
            });

            let iteration_result = self.execute_iteration(iteration).await;

            match iteration_result {
                Ok(outcome) => {
                    self.context.stats.increment_iterations();

                    if outcome.needs_reboot {
                        self.perform_reboot().await?;
                    }

                    if outcome.made_progress {
                        self.context.stats.record_success();
                    } else {
                        self.context.stats.record_no_progress();
                    }

                    self.event_emitter.emit(LoopEvent::IterationCompleted {
                        loop_id: self.context.id,
                        iteration,
                        outcome: outcome.clone(),
                    });
                }
                Err(e) => {
                    error!("Iteration {} failed: {}", iteration, e);
                    self.context.stats.increment_iterations();
                    self.context.stats.record_failure();

                    self.event_emitter.emit(LoopEvent::IterationFailed {
                        loop_id: self.context.id,
                        iteration,
                        error: e.to_string(),
                    });
                }
            }

            // Delay before next iteration
            tokio::time::sleep(self.config.iteration_delay).await;
        }
    }

    /// Handle a command.
    async fn handle_command(&self, cmd: LoopCommand) -> LoopResult<bool> {
        match cmd {
            LoopCommand::Pause => {
                if self.context.get_state().await.can_pause() {
                    self.context.set_state(LoopState::Paused).await;
                    self.event_emitter.emit(LoopEvent::Paused {
                        loop_id: self.context.id,
                    });
                }
                Ok(false)
            }
            LoopCommand::Resume => {
                if self.context.get_state().await.can_resume() {
                    self.context.set_state(LoopState::Running).await;
                    self.event_emitter.emit(LoopEvent::Resumed {
                        loop_id: self.context.id,
                    });
                }
                Ok(false)
            }
            LoopCommand::Stop => {
                self.context.set_state(LoopState::Stopped).await;
                self.context.set_stop_reason("User requested stop").await;
                self.event_emitter.emit(LoopEvent::Stopped {
                    loop_id: self.context.id,
                    reason: "User requested stop".to_string(),
                });
                Ok(true) // Signal to exit loop
            }
            LoopCommand::ForceReboot => {
                self.perform_reboot().await?;
                Ok(false)
            }
            LoopCommand::SkipIteration => {
                debug!("Skipping current iteration");
                Ok(false)
            }
        }
    }

    /// Execute a single iteration.
    async fn execute_iteration(&self, iteration: u32) -> LoopResult<IterationOutcome> {
        let start_time = std::time::Instant::now();

        // Get or create session
        let session = self.session_manager.get_or_create_session().await?;

        // Load and apply prompt
        let prompt = self.load_prompt().await?;
        session.send_prompt(&prompt).await?;

        // Wait for session to complete or hit redline
        let session_result = session.wait_for_completion().await?;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        self.context.stats.add_execution_time(elapsed_ms);

        Ok(IterationOutcome {
            iteration,
            duration_ms: elapsed_ms,
            made_progress: session_result.made_changes,
            needs_reboot: session_result.context_usage > self.config.context_redline_percent,
            tests_passed: session_result.tests_passed,
            files_changed: session_result.files_changed,
        })
    }

    /// Load the prompt file.
    async fn load_prompt(&self) -> LoopResult<String> {
        let prompt_path = self.config.working_dir.join(&self.config.prompt_path);
        tokio::fs::read_to_string(&prompt_path)
            .await
            .map_err(|e| LoopError::PromptLoadFailed {
                path: prompt_path,
                source: e,
            })
    }

    /// Perform a context reboot.
    async fn perform_reboot(&self) -> LoopResult<()> {
        info!("Performing context reboot");

        self.context.set_state(LoopState::Rebooting).await;
        self.event_emitter.emit(LoopEvent::RebootStarted {
            loop_id: self.context.id,
        });

        // End current session
        self.session_manager.end_current_session().await?;

        // Create fresh session
        self.session_manager.create_fresh_session().await?;

        self.context.stats.record_reboot();
        self.context.set_state(LoopState::Running).await;
        self.event_emitter.emit(LoopEvent::RebootCompleted {
            loop_id: self.context.id,
        });

        Ok(())
    }

    /// Request shutdown.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Outcome of a single iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationOutcome {
    /// Iteration number.
    pub iteration: u32,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Whether progress was made.
    pub made_progress: bool,
    /// Whether a reboot is needed.
    pub needs_reboot: bool,
    /// Whether tests passed.
    pub tests_passed: Option<bool>,
    /// Number of files changed.
    pub files_changed: u32,
}
```

### 5. Library Root (src/lib.rs)

```rust
//! Ralph Wiggum Loop Runner.
//!
//! This crate provides the core loop execution logic for continuous
//! Claude Code development sessions with automatic context management.

#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod events;
pub mod runner;
pub mod session;
pub mod state;
pub mod stop;

pub use config::{LoopConfig, SessionConfig, StopConditionsConfig};
pub use error::{LoopError, LoopResult};
pub use events::LoopEvent;
pub use runner::{IterationOutcome, LoopCommand, LoopRunner};
pub use state::{LoopContext, LoopState, LoopStats, LoopStatsSnapshot};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a loop execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoopId(Uuid);

impl LoopId {
    /// Create a new random loop ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LoopId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LoopId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop_{}", self.0)
    }
}
```

---

## Testing Requirements

1. Loop starts and transitions through correct states
2. Commands (pause, resume, stop) work correctly
3. Shutdown signal properly terminates loop
4. Maximum iterations limit is respected
5. Event emission for all state changes
6. Statistics are correctly tracked
7. Concurrent access to loop state is safe

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Depends on: [019-async-runtime.md](../phase-01-common/019-async-runtime.md)
- Next: [097-loop-iteration.md](097-loop-iteration.md)
- Related: [100-session-management.md](100-session-management.md)
