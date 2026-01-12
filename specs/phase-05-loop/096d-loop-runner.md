# 096d - Core Loop Runner

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 096d
**Status:** Planned
**Dependencies:** 096c-loop-state
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Implement the core `LoopRunner` struct with async execution, command handling, and lifecycle management.

---

## Acceptance Criteria

- [ ] `LoopRunner` struct with async `run()` method
- [ ] `LoopCommand` enum for external control
- [ ] Command handling (pause, resume, stop)
- [ ] Graceful shutdown handling
- [ ] Event emission integration

---

## Implementation Details

### 1. Core Runner (src/runner.rs)

```rust
//! Core loop runner implementation.

use crate::config::LoopConfig;
use crate::error::{LoopError, LoopResult};
use crate::events::{LoopEvent, LoopEventEmitter};
use crate::state::{LoopContext, LoopState};
use crate::LoopId;

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, instrument, warn};

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

/// Outcome of a single iteration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// The Ralph Wiggum Loop Runner.
pub struct LoopRunner {
    /// Loop configuration.
    config: LoopConfig,
    /// Loop context.
    context: Arc<LoopContext>,
    /// Event emitter.
    event_emitter: Arc<LoopEventEmitter>,
    /// Shutdown signal sender.
    shutdown_tx: broadcast::Sender<()>,
    /// Command receiver.
    command_rx: RwLock<Option<mpsc::Receiver<LoopCommand>>>,
    /// Command sender (for external control).
    command_tx: mpsc::Sender<LoopCommand>,
}

impl LoopRunner {
    /// Create a new loop runner.
    pub fn new(config: LoopConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (command_tx, command_rx) = mpsc::channel(32);
        let loop_id = LoopId::new();

        Self {
            config,
            context: Arc::new(LoopContext::new(loop_id)),
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

        let current_state = self.context.get_state().await;
        if !current_state.can_start() {
            return Err(LoopError::InvalidState {
                current: current_state,
                action: "start",
            });
        }

        let mut command_rx = self.command_rx.write().await.take()
            .ok_or(LoopError::AlreadyRunning)?;

        self.context.set_state(LoopState::Running).await;
        self.event_emitter.emit(LoopEvent::Started {
            loop_id: self.context.id,
            config: self.config.clone(),
        });

        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let result = self.run_loop(&mut command_rx, &mut shutdown_rx).await;

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
                self.event_emitter.emit(LoopEvent::Error {
                    loop_id: self.context.id,
                    error: e.to_string(),
                });
            }
        }

        info!("Ralph Loop Runner finished");
        result
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
                self.event_emitter.emit(LoopEvent::Stopped {
                    loop_id: self.context.id,
                    reason: "User requested stop".to_string(),
                });
                Ok(true) // Signal to exit loop
            }
            LoopCommand::ForceReboot => {
                debug!("Force reboot requested");
                Ok(false)
            }
            LoopCommand::SkipIteration => {
                debug!("Skipping current iteration");
                Ok(false)
            }
        }
    }

    /// Request shutdown.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Internal loop execution (stub - see 097 for full implementation).
    async fn run_loop(
        &self,
        _command_rx: &mut mpsc::Receiver<LoopCommand>,
        _shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> LoopResult<()> {
        // Implementation in 097-loop-iteration.md
        Ok(())
    }
}
```

---

## Testing Requirements

1. Loop starts and transitions through correct states
2. Commands (pause, resume, stop) work correctly
3. Shutdown signal properly terminates loop
4. Event emission for all state changes

---

## Related Specs

- Depends on: [096c-loop-state.md](096c-loop-state.md)
- Next: [097-loop-iteration.md](097-loop-iteration.md)
