# 112 - Mode Switching

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 112
**Status:** Planned
**Dependencies:** 110-attended-mode, 111-unattended-mode
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement seamless mode switching between attended and unattended modes for the Ralph Loop - allowing users to transition between interactive and autonomous operation without stopping the loop.

---

## Acceptance Criteria

- [ ] Switch from attended to unattended
- [ ] Switch from unattended to attended
- [ ] State preservation during switch
- [ ] Pending decisions handled
- [ ] Notification of mode changes
- [ ] Scheduled mode switching
- [ ] Mode-specific configurations persist
- [ ] Smooth transition without iteration loss

---

## Implementation Details

### 1. Mode Switching Types (src/mode/types.rs)

```rust
//! Mode switching type definitions.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Operating mode of the loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    /// Interactive mode with user oversight.
    Attended,
    /// Autonomous mode without user intervention.
    Unattended,
    /// Hybrid mode with conditional prompting.
    Hybrid,
}

impl Default for OperatingMode {
    fn default() -> Self {
        Self::Unattended
    }
}

/// Configuration for mode switching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSwitchConfig {
    /// Allow mode switching.
    pub enabled: bool,
    /// Default mode on startup.
    pub default_mode: OperatingMode,
    /// Scheduled mode changes.
    pub schedule: Vec<ScheduledModeChange>,
    /// Conditions that trigger mode switch.
    pub triggers: ModeTriggers,
    /// Hybrid mode configuration.
    pub hybrid: HybridConfig,
}

impl Default for ModeSwitchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_mode: OperatingMode::Unattended,
            schedule: vec![],
            triggers: ModeTriggers::default(),
            hybrid: HybridConfig::default(),
        }
    }
}

/// A scheduled mode change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledModeChange {
    /// Time to switch (HH:MM).
    pub at_time: String,
    /// Days to apply (0 = Sunday, None = every day).
    pub days: Option<Vec<u8>>,
    /// Mode to switch to.
    pub mode: OperatingMode,
}

/// Conditions that trigger automatic mode switches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTriggers {
    /// Switch to attended on consecutive failures.
    pub attend_on_failures: Option<u32>,
    /// Switch to attended on error patterns.
    pub attend_on_patterns: Vec<String>,
    /// Switch to unattended after idle time.
    pub unattend_after_idle: Option<Duration>,
    /// Switch to attended on test regression.
    pub attend_on_test_regression: bool,
    /// Switch to attended before completion.
    pub attend_before_completion: bool,
}

impl Default for ModeTriggers {
    fn default() -> Self {
        Self {
            attend_on_failures: Some(3),
            attend_on_patterns: vec![
                "CRITICAL".to_string(),
                "BREAKING".to_string(),
            ],
            unattend_after_idle: None,
            attend_on_test_regression: true,
            attend_before_completion: true,
        }
    }
}

/// Hybrid mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    /// Prompt every N iterations.
    pub prompt_every: Option<u32>,
    /// Prompt on significant changes.
    pub prompt_on_significant_changes: bool,
    /// Threshold for significant changes (files).
    pub significant_change_threshold: u32,
    /// Auto-approve timeout.
    pub auto_approve_timeout: Option<Duration>,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            prompt_every: Some(10),
            prompt_on_significant_changes: true,
            significant_change_threshold: 5,
            auto_approve_timeout: Some(Duration::from_secs(60)),
        }
    }
}

/// Request to switch modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSwitchRequest {
    /// Target mode.
    pub target_mode: OperatingMode,
    /// Reason for switch.
    pub reason: ModeSwitchReason,
    /// When the switch should happen.
    pub timing: SwitchTiming,
}

/// Reason for mode switch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModeSwitchReason {
    /// User requested.
    UserRequest,
    /// Scheduled change.
    Scheduled { schedule_id: usize },
    /// Triggered by condition.
    Triggered { trigger: String },
    /// API request.
    ApiRequest,
    /// Automatic (e.g., completion).
    Automatic,
}

/// When the switch should happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwitchTiming {
    /// Switch immediately.
    Immediate,
    /// Switch after current iteration.
    AfterIteration,
    /// Switch at next pause point.
    AtPause,
}

/// Result of a mode switch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSwitchResult {
    /// Whether switch succeeded.
    pub success: bool,
    /// Previous mode.
    pub previous_mode: OperatingMode,
    /// New mode.
    pub new_mode: OperatingMode,
    /// When the switch occurred.
    pub switched_at: chrono::DateTime<chrono::Utc>,
    /// Iteration at switch.
    pub iteration: u32,
    /// Error if failed.
    pub error: Option<String>,
}

/// History entry for mode changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeHistoryEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub from_mode: OperatingMode,
    pub to_mode: OperatingMode,
    pub reason: ModeSwitchReason,
    pub iteration: u32,
}
```

### 2. Mode Controller (src/mode/controller.rs)

```rust
//! Mode switching controller.

use super::types::{
    HybridConfig, ModeHistoryEntry, ModeSwitchConfig, ModeSwitchReason, ModeSwitchRequest,
    ModeSwitchResult, ModeTriggers, OperatingMode, SwitchTiming,
};
use crate::attended::AttendedController;
use crate::error::{LoopError, LoopResult};
use crate::unattended::UnattendedController;

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};

/// Controls mode switching between attended and unattended.
pub struct ModeController {
    /// Configuration.
    config: RwLock<ModeSwitchConfig>,
    /// Current mode.
    current_mode: RwLock<OperatingMode>,
    /// Attended controller.
    attended: Arc<AttendedController>,
    /// Unattended controller.
    unattended: Arc<UnattendedController>,
    /// Pending switch request.
    pending_switch: RwLock<Option<ModeSwitchRequest>>,
    /// Mode change history.
    history: RwLock<Vec<ModeHistoryEntry>>,
    /// Event broadcaster.
    event_tx: broadcast::Sender<ModeEvent>,
    /// Switch request channel.
    switch_tx: mpsc::Sender<ModeSwitchRequest>,
    switch_rx: RwLock<Option<mpsc::Receiver<ModeSwitchRequest>>>,
}

/// Events emitted by the mode controller.
#[derive(Debug, Clone)]
pub enum ModeEvent {
    /// Mode switched.
    ModeSwitched {
        from: OperatingMode,
        to: OperatingMode,
        reason: ModeSwitchReason,
    },
    /// Mode switch pending.
    SwitchPending {
        to: OperatingMode,
        timing: SwitchTiming,
    },
    /// Mode switch failed.
    SwitchFailed {
        to: OperatingMode,
        error: String,
    },
    /// Trigger condition detected.
    TriggerDetected {
        trigger: String,
        suggested_mode: OperatingMode,
    },
}

impl ModeController {
    /// Create a new mode controller.
    pub fn new(
        config: ModeSwitchConfig,
        attended: Arc<AttendedController>,
        unattended: Arc<UnattendedController>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(64);
        let (switch_tx, switch_rx) = mpsc::channel(8);
        let initial_mode = config.default_mode;

        Self {
            config: RwLock::new(config),
            current_mode: RwLock::new(initial_mode),
            attended,
            unattended,
            pending_switch: RwLock::new(None),
            history: RwLock::new(Vec::new()),
            event_tx,
            switch_tx,
            switch_rx: RwLock::new(Some(switch_rx)),
        }
    }

    /// Get current mode.
    pub async fn current_mode(&self) -> OperatingMode {
        *self.current_mode.read().await
    }

    /// Subscribe to mode events.
    pub fn subscribe(&self) -> broadcast::Receiver<ModeEvent> {
        self.event_tx.subscribe()
    }

    /// Get switch request sender.
    pub fn switch_sender(&self) -> mpsc::Sender<ModeSwitchRequest> {
        self.switch_tx.clone()
    }

    /// Request a mode switch.
    pub async fn request_switch(&self, request: ModeSwitchRequest) -> LoopResult<()> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err(LoopError::ModeSwitchDisabled);
        }

        let current = *self.current_mode.read().await;
        if current == request.target_mode {
            debug!("Already in requested mode: {:?}", current);
            return Ok(());
        }

        match request.timing {
            SwitchTiming::Immediate => {
                self.perform_switch(request).await
            }
            _ => {
                *self.pending_switch.write().await = Some(request.clone());
                self.emit(ModeEvent::SwitchPending {
                    to: request.target_mode,
                    timing: request.timing,
                });
                Ok(())
            }
        }
    }

    /// Check for pending switches (call at appropriate times).
    pub async fn check_pending(&self, timing: SwitchTiming, iteration: u32) -> LoopResult<Option<ModeSwitchResult>> {
        let pending = self.pending_switch.read().await.clone();

        if let Some(request) = pending {
            if request.timing == timing {
                *self.pending_switch.write().await = None;
                return self.perform_switch(request).await.map(Some);
            }
        }

        // Check scheduled switches
        if let Some(result) = self.check_scheduled_switches().await? {
            return Ok(Some(result));
        }

        Ok(None)
    }

    /// Check for trigger conditions.
    pub async fn check_triggers(
        &self,
        consecutive_failures: u32,
        output: &str,
        test_regressed: bool,
        near_completion: bool,
        iteration: u32,
    ) -> LoopResult<Option<ModeSwitchResult>> {
        let config = self.config.read().await;
        let triggers = &config.triggers;
        let current = *self.current_mode.read().await;

        // Only trigger switches to attended from unattended
        if current != OperatingMode::Unattended {
            return Ok(None);
        }

        // Check failure threshold
        if let Some(threshold) = triggers.attend_on_failures {
            if consecutive_failures >= threshold {
                info!("Switching to attended mode due to {} consecutive failures", consecutive_failures);
                self.emit(ModeEvent::TriggerDetected {
                    trigger: format!("consecutive_failures:{}", consecutive_failures),
                    suggested_mode: OperatingMode::Attended,
                });
                return self.request_switch(ModeSwitchRequest {
                    target_mode: OperatingMode::Attended,
                    reason: ModeSwitchReason::Triggered {
                        trigger: "consecutive_failures".to_string(),
                    },
                    timing: SwitchTiming::Immediate,
                }).await.map(|_| Some(ModeSwitchResult {
                    success: true,
                    previous_mode: OperatingMode::Unattended,
                    new_mode: OperatingMode::Attended,
                    switched_at: chrono::Utc::now(),
                    iteration,
                    error: None,
                }));
            }
        }

        // Check pattern triggers
        for pattern in &triggers.attend_on_patterns {
            if output.contains(pattern) {
                info!("Switching to attended mode due to pattern: {}", pattern);
                self.emit(ModeEvent::TriggerDetected {
                    trigger: format!("pattern:{}", pattern),
                    suggested_mode: OperatingMode::Attended,
                });
                return self.request_switch(ModeSwitchRequest {
                    target_mode: OperatingMode::Attended,
                    reason: ModeSwitchReason::Triggered {
                        trigger: format!("pattern:{}", pattern),
                    },
                    timing: SwitchTiming::Immediate,
                }).await.map(|_| Some(ModeSwitchResult {
                    success: true,
                    previous_mode: OperatingMode::Unattended,
                    new_mode: OperatingMode::Attended,
                    switched_at: chrono::Utc::now(),
                    iteration,
                    error: None,
                }));
            }
        }

        // Check test regression
        if triggers.attend_on_test_regression && test_regressed {
            info!("Switching to attended mode due to test regression");
            self.emit(ModeEvent::TriggerDetected {
                trigger: "test_regression".to_string(),
                suggested_mode: OperatingMode::Attended,
            });
            return self.request_switch(ModeSwitchRequest {
                target_mode: OperatingMode::Attended,
                reason: ModeSwitchReason::Triggered {
                    trigger: "test_regression".to_string(),
                },
                timing: SwitchTiming::Immediate,
            }).await.map(|_| Some(ModeSwitchResult {
                success: true,
                previous_mode: OperatingMode::Unattended,
                new_mode: OperatingMode::Attended,
                switched_at: chrono::Utc::now(),
                iteration,
                error: None,
            }));
        }

        // Check completion
        if triggers.attend_before_completion && near_completion {
            info!("Switching to attended mode for completion review");
            self.emit(ModeEvent::TriggerDetected {
                trigger: "near_completion".to_string(),
                suggested_mode: OperatingMode::Attended,
            });
            return self.request_switch(ModeSwitchRequest {
                target_mode: OperatingMode::Attended,
                reason: ModeSwitchReason::Triggered {
                    trigger: "near_completion".to_string(),
                },
                timing: SwitchTiming::Immediate,
            }).await.map(|_| Some(ModeSwitchResult {
                success: true,
                previous_mode: OperatingMode::Unattended,
                new_mode: OperatingMode::Attended,
                switched_at: chrono::Utc::now(),
                iteration,
                error: None,
            }));
        }

        Ok(None)
    }

    /// Check for scheduled mode switches.
    async fn check_scheduled_switches(&self) -> LoopResult<Option<ModeSwitchResult>> {
        let config = self.config.read().await;
        let now = chrono::Local::now();
        let current_time = now.format("%H:%M").to_string();
        let current_day = now.weekday().num_days_from_sunday() as u8;

        for (idx, schedule) in config.schedule.iter().enumerate() {
            // Check time
            if schedule.at_time != current_time {
                continue;
            }

            // Check day
            if let Some(days) = &schedule.days {
                if !days.contains(&current_day) {
                    continue;
                }
            }

            // This schedule matches
            let current = *self.current_mode.read().await;
            if current == schedule.mode {
                continue;
            }

            info!("Scheduled mode switch to {:?}", schedule.mode);
            drop(config); // Release lock before calling request_switch

            return self.request_switch(ModeSwitchRequest {
                target_mode: schedule.mode,
                reason: ModeSwitchReason::Scheduled { schedule_id: idx },
                timing: SwitchTiming::Immediate,
            }).await.map(|_| Some(ModeSwitchResult {
                success: true,
                previous_mode: current,
                new_mode: schedule.mode,
                switched_at: chrono::Utc::now(),
                iteration: 0,
                error: None,
            }));
        }

        Ok(None)
    }

    /// Perform the actual mode switch.
    async fn perform_switch(&self, request: ModeSwitchRequest) -> LoopResult<ModeSwitchResult> {
        let previous = *self.current_mode.read().await;

        info!("Switching mode: {:?} -> {:?}", previous, request.target_mode);

        // Update controllers
        match request.target_mode {
            OperatingMode::Attended => {
                self.attended.enable().await;
                // Unattended remains active for safety monitoring
            }
            OperatingMode::Unattended => {
                self.attended.disable().await;
            }
            OperatingMode::Hybrid => {
                // Both active with hybrid logic
                self.attended.enable().await;
            }
        }

        // Update current mode
        *self.current_mode.write().await = request.target_mode;

        // Record in history
        let entry = ModeHistoryEntry {
            timestamp: chrono::Utc::now(),
            from_mode: previous,
            to_mode: request.target_mode,
            reason: request.reason.clone(),
            iteration: 0, // Would need to pass this in
        };
        self.history.write().await.push(entry);

        // Emit event
        self.emit(ModeEvent::ModeSwitched {
            from: previous,
            to: request.target_mode,
            reason: request.reason,
        });

        Ok(ModeSwitchResult {
            success: true,
            previous_mode: previous,
            new_mode: request.target_mode,
            switched_at: chrono::Utc::now(),
            iteration: 0,
            error: None,
        })
    }

    /// Check if hybrid mode should prompt (for hybrid mode).
    pub async fn should_hybrid_prompt(&self, iteration: u32, files_changed: u32) -> bool {
        let config = self.config.read().await;
        let hybrid = &config.hybrid;

        // Check iteration interval
        if let Some(interval) = hybrid.prompt_every {
            if iteration > 0 && iteration % interval == 0 {
                return true;
            }
        }

        // Check significant changes
        if hybrid.prompt_on_significant_changes {
            if files_changed >= hybrid.significant_change_threshold {
                return true;
            }
        }

        false
    }

    /// Get mode history.
    pub async fn get_history(&self) -> Vec<ModeHistoryEntry> {
        self.history.read().await.clone()
    }

    /// Emit event.
    fn emit(&self, event: ModeEvent) {
        let _ = self.event_tx.send(event);
    }
}

use chrono::Datelike;
```

### 3. Module Root (src/mode/mod.rs)

```rust
//! Mode switching between attended and unattended operation.

pub mod controller;
pub mod types;

pub use controller::{ModeController, ModeEvent};
pub use types::{
    HybridConfig, ModeHistoryEntry, ModeSwitchConfig, ModeSwitchReason,
    ModeSwitchRequest, ModeSwitchResult, ModeTriggers, OperatingMode,
    ScheduledModeChange, SwitchTiming,
};
```

---

## Testing Requirements

1. Switch from attended to unattended works
2. Switch from unattended to attended works
3. Pending switches are processed at correct timing
4. Scheduled switches trigger at correct time
5. Trigger conditions cause mode switch
6. Hybrid mode prompts at correct intervals
7. History is recorded correctly
8. Events are emitted for all switches

---

## Related Specs

- Depends on: [110-attended-mode.md](110-attended-mode.md)
- Depends on: [111-unattended-mode.md](111-unattended-mode.md)
- Next: [113-loop-hooks.md](113-loop-hooks.md)
- Related: [096-loop-runner-core.md](096-loop-runner-core.md)
