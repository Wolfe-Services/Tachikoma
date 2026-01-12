# 103 - Automatic Reboot

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 103
**Status:** Planned
**Dependencies:** 101-fresh-context, 102-redline-detection
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement automatic reboot orchestration for the Ralph Loop - coordinating context reboot triggers, state preservation, and seamless session transitions without manual intervention.

---

## Acceptance Criteria

- [x] Automatic reboot on redline detection
- [x] Configurable reboot policies
- [x] Pre-reboot hooks execution
- [x] State preservation during reboot
- [x] Post-reboot hooks execution
- [x] Reboot rate limiting
- [x] Graceful vs immediate reboot modes
- [x] Reboot metrics and history

---

## Implementation Details

### 1. Auto-Reboot Types (src/reboot/types.rs)

```rust
//! Auto-reboot type definitions.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for automatic reboots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRebootConfig {
    /// Enable automatic reboots.
    pub enabled: bool,

    /// Reboot mode.
    pub mode: RebootMode,

    /// Minimum time between reboots.
    #[serde(with = "humantime_serde")]
    pub min_reboot_interval: Duration,

    /// Maximum reboots per hour (0 = unlimited).
    pub max_reboots_per_hour: u32,

    /// Cooldown after failed reboot.
    #[serde(with = "humantime_serde")]
    pub failure_cooldown: Duration,

    /// Maximum consecutive reboot failures before stopping.
    pub max_consecutive_failures: u32,

    /// Pre-reboot delay for graceful mode.
    #[serde(with = "humantime_serde")]
    pub graceful_delay: Duration,

    /// Enable pre-reboot hooks.
    pub enable_pre_hooks: bool,

    /// Enable post-reboot hooks.
    pub enable_post_hooks: bool,

    /// Preserve conversation summary.
    pub preserve_summary: bool,

    /// Triggers that initiate reboot.
    pub triggers: RebootTriggers,
}

impl Default for AutoRebootConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: RebootMode::Graceful,
            min_reboot_interval: Duration::from_secs(300), // 5 minutes
            max_reboots_per_hour: 10,
            failure_cooldown: Duration::from_secs(60),
            max_consecutive_failures: 3,
            graceful_delay: Duration::from_secs(5),
            enable_pre_hooks: true,
            enable_post_hooks: true,
            preserve_summary: true,
            triggers: RebootTriggers::default(),
        }
    }
}

/// Triggers that can initiate a reboot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebootTriggers {
    /// Trigger on context redline.
    pub on_redline: bool,
    /// Trigger on degradation detection.
    pub on_degradation: bool,
    /// Trigger after N iterations.
    pub after_iterations: Option<u32>,
    /// Trigger after duration.
    #[serde(with = "option_duration_serde")]
    pub after_duration: Option<Duration>,
    /// Trigger on specific patterns in output.
    pub on_patterns: Vec<String>,
}

impl Default for RebootTriggers {
    fn default() -> Self {
        Self {
            on_redline: true,
            on_degradation: true,
            after_iterations: None,
            after_duration: None,
            on_patterns: vec![],
        }
    }
}

/// Reboot mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RebootMode {
    /// Allow current task to complete first.
    Graceful,
    /// Reboot immediately.
    Immediate,
    /// Ask user before rebooting.
    Confirm,
}

/// Reason for a reboot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RebootReason {
    /// Context hit redline threshold.
    Redline { usage_percent: u8 },
    /// Degradation was detected.
    Degradation { details: String },
    /// Iteration limit reached.
    IterationLimit { iterations: u32 },
    /// Duration limit reached.
    DurationLimit { duration_secs: u64 },
    /// Pattern matched in output.
    PatternMatch { pattern: String },
    /// Manual request.
    Manual,
    /// Error recovery.
    ErrorRecovery { error: String },
}

/// Result of a reboot operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebootResult {
    /// Whether the reboot succeeded.
    pub success: bool,
    /// The reason for the reboot.
    pub reason: RebootReason,
    /// Time taken for the reboot.
    pub duration_ms: u64,
    /// Old session ID.
    pub old_session_id: Option<String>,
    /// New session ID.
    pub new_session_id: Option<String>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Pre-hook results.
    pub pre_hook_results: Vec<HookResult>,
    /// Post-hook results.
    pub post_hook_results: Vec<HookResult>,
}

/// Result of a hook execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Hook name.
    pub name: String,
    /// Whether it succeeded.
    pub success: bool,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Output if any.
    pub output: Option<String>,
    /// Error if failed.
    pub error: Option<String>,
}

/// Reboot history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebootHistoryEntry {
    /// When the reboot occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The reboot result.
    pub result: RebootResult,
    /// Iteration number at reboot.
    pub iteration: u32,
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

mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => serializer.serialize_some(&humantime::format_duration(*d).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => humantime::parse_duration(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}
```

### 2. Auto-Reboot Manager (src/reboot/manager.rs)

```rust
//! Auto-reboot management.

use super::types::{
    AutoRebootConfig, HookResult, RebootHistoryEntry, RebootMode, RebootReason, RebootResult,
};
use crate::context::{ContextManager, FreshContextResult};
use crate::error::{LoopError, LoopResult};
use crate::redline::{RedlineCheckResult, RedlineDetector};

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Manages automatic reboots.
pub struct AutoRebootManager {
    /// Configuration.
    config: AutoRebootConfig,
    /// Context manager for performing reboots.
    context_manager: Arc<ContextManager>,
    /// Redline detector.
    redline_detector: Arc<RedlineDetector>,
    /// Reboot history.
    history: RwLock<VecDeque<RebootHistoryEntry>>,
    /// Last reboot time.
    last_reboot: RwLock<Option<chrono::DateTime<chrono::Utc>>>,
    /// Consecutive failure count.
    consecutive_failures: std::sync::atomic::AtomicU32,
    /// Pre-reboot hooks.
    pre_hooks: RwLock<Vec<Box<dyn RebootHook>>>,
    /// Post-reboot hooks.
    post_hooks: RwLock<Vec<Box<dyn RebootHook>>>,
    /// Current iteration.
    current_iteration: std::sync::atomic::AtomicU32,
    /// Session start time.
    session_start: RwLock<chrono::DateTime<chrono::Utc>>,
}

/// A reboot hook.
#[async_trait::async_trait]
pub trait RebootHook: Send + Sync {
    /// Hook name.
    fn name(&self) -> &str;

    /// Execute the hook.
    async fn execute(&self, reason: &RebootReason) -> LoopResult<Option<String>>;
}

impl AutoRebootManager {
    /// Create a new auto-reboot manager.
    pub fn new(
        config: AutoRebootConfig,
        context_manager: Arc<ContextManager>,
        redline_detector: Arc<RedlineDetector>,
    ) -> Self {
        Self {
            config,
            context_manager,
            redline_detector,
            history: RwLock::new(VecDeque::new()),
            last_reboot: RwLock::new(None),
            consecutive_failures: std::sync::atomic::AtomicU32::new(0),
            pre_hooks: RwLock::new(Vec::new()),
            post_hooks: RwLock::new(Vec::new()),
            current_iteration: std::sync::atomic::AtomicU32::new(0),
            session_start: RwLock::new(chrono::Utc::now()),
        }
    }

    /// Register a pre-reboot hook.
    pub async fn register_pre_hook(&self, hook: Box<dyn RebootHook>) {
        self.pre_hooks.write().await.push(hook);
    }

    /// Register a post-reboot hook.
    pub async fn register_post_hook(&self, hook: Box<dyn RebootHook>) {
        self.post_hooks.write().await.push(hook);
    }

    /// Check if reboot is needed and perform if necessary.
    #[instrument(skip(self, redline_result))]
    pub async fn check_and_reboot(
        &self,
        redline_result: &RedlineCheckResult,
        output: &str,
    ) -> LoopResult<Option<RebootResult>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Check if reboot is needed
        let reason = self.should_reboot(redline_result, output).await?;

        match reason {
            Some(reason) => {
                // Check rate limiting
                if !self.can_reboot().await? {
                    warn!("Reboot rate limited, skipping");
                    return Ok(None);
                }

                // Perform reboot
                let result = self.perform_reboot(reason).await;

                // Update state
                if result.success {
                    self.consecutive_failures.store(0, std::sync::atomic::Ordering::Relaxed);
                } else {
                    let failures = self.consecutive_failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    if failures >= self.config.max_consecutive_failures {
                        error!("Max consecutive reboot failures reached");
                        return Err(LoopError::MaxRebootFailures {
                            count: failures,
                        });
                    }
                }

                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Determine if reboot is needed.
    async fn should_reboot(
        &self,
        redline_result: &RedlineCheckResult,
        output: &str,
    ) -> LoopResult<Option<RebootReason>> {
        let triggers = &self.config.triggers;

        // Check redline trigger
        if triggers.on_redline && redline_result.is_redline {
            return Ok(Some(RebootReason::Redline {
                usage_percent: redline_result.usage_percent,
            }));
        }

        // Check degradation trigger
        if triggers.on_degradation && redline_result.degradation_detected {
            return Ok(Some(RebootReason::Degradation {
                details: "Performance degradation detected".to_string(),
            }));
        }

        // Check iteration trigger
        if let Some(limit) = triggers.after_iterations {
            let current = self.current_iteration.load(std::sync::atomic::Ordering::Relaxed);
            if current >= limit {
                return Ok(Some(RebootReason::IterationLimit {
                    iterations: current,
                }));
            }
        }

        // Check duration trigger
        if let Some(limit) = triggers.after_duration {
            let start = *self.session_start.read().await;
            let elapsed = chrono::Utc::now() - start;
            if elapsed.num_seconds() as u64 >= limit.as_secs() {
                return Ok(Some(RebootReason::DurationLimit {
                    duration_secs: elapsed.num_seconds() as u64,
                }));
            }
        }

        // Check pattern triggers
        for pattern in &triggers.on_patterns {
            if output.contains(pattern) {
                return Ok(Some(RebootReason::PatternMatch {
                    pattern: pattern.clone(),
                }));
            }
        }

        Ok(None)
    }

    /// Check if we can reboot (rate limiting).
    async fn can_reboot(&self) -> LoopResult<bool> {
        let now = chrono::Utc::now();

        // Check minimum interval
        if let Some(last) = *self.last_reboot.read().await {
            let elapsed = now - last;
            if elapsed.num_milliseconds() < self.config.min_reboot_interval.as_millis() as i64 {
                return Ok(false);
            }
        }

        // Check hourly limit
        if self.config.max_reboots_per_hour > 0 {
            let history = self.history.read().await;
            let hour_ago = now - chrono::Duration::hours(1);
            let recent_count = history
                .iter()
                .filter(|e| e.timestamp > hour_ago && e.result.success)
                .count();

            if recent_count >= self.config.max_reboots_per_hour as usize {
                return Ok(false);
            }
        }

        // Check failure cooldown
        let failures = self.consecutive_failures.load(std::sync::atomic::Ordering::Relaxed);
        if failures > 0 {
            if let Some(last) = *self.last_reboot.read().await {
                let elapsed = now - last;
                let cooldown = self.config.failure_cooldown.as_millis() as i64 * failures as i64;
                if elapsed.num_milliseconds() < cooldown {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Perform the reboot.
    #[instrument(skip(self))]
    async fn perform_reboot(&self, reason: RebootReason) -> RebootResult {
        info!("Performing reboot: {:?}", reason);
        let start = std::time::Instant::now();

        // Execute pre-hooks
        let pre_hook_results = if self.config.enable_pre_hooks {
            self.execute_hooks(&self.pre_hooks, &reason).await
        } else {
            vec![]
        };

        // Check if any pre-hook failed and we should abort
        let pre_hook_failed = pre_hook_results.iter().any(|r| !r.success);
        if pre_hook_failed {
            warn!("Pre-hook failed, aborting reboot");
            return RebootResult {
                success: false,
                reason,
                duration_ms: start.elapsed().as_millis() as u64,
                old_session_id: None,
                new_session_id: None,
                error: Some("Pre-hook failed".to_string()),
                pre_hook_results,
                post_hook_results: vec![],
            };
        }

        // Graceful delay if configured
        if self.config.mode == RebootMode::Graceful {
            debug!("Waiting for graceful delay");
            tokio::time::sleep(self.config.graceful_delay).await;
        }

        // Perform the context transition
        let context_result = self.context_manager.create_fresh_context().await;

        let (success, old_id, new_id, error) = match context_result {
            Ok(result) => (
                true,
                result.old_session_id.map(|id| id.to_string()),
                Some(result.new_session_id.to_string()),
                None,
            ),
            Err(e) => (false, None, None, Some(e.to_string())),
        };

        // Execute post-hooks
        let post_hook_results = if success && self.config.enable_post_hooks {
            self.execute_hooks(&self.post_hooks, &reason).await
        } else {
            vec![]
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        // Update state
        *self.last_reboot.write().await = Some(chrono::Utc::now());
        self.current_iteration.store(0, std::sync::atomic::Ordering::Relaxed);
        *self.session_start.write().await = chrono::Utc::now();

        if success {
            self.redline_detector.reset_after_reboot();
        }

        let result = RebootResult {
            success,
            reason: reason.clone(),
            duration_ms,
            old_session_id: old_id,
            new_session_id: new_id,
            error,
            pre_hook_results,
            post_hook_results,
        };

        // Record in history
        let iteration = self.current_iteration.load(std::sync::atomic::Ordering::Relaxed);
        self.record_history(RebootHistoryEntry {
            timestamp: chrono::Utc::now(),
            result: result.clone(),
            iteration,
        })
        .await;

        info!("Reboot completed: success={}, duration={}ms", result.success, result.duration_ms);

        result
    }

    /// Execute hooks.
    async fn execute_hooks(
        &self,
        hooks: &RwLock<Vec<Box<dyn RebootHook>>>,
        reason: &RebootReason,
    ) -> Vec<HookResult> {
        let hooks = hooks.read().await;
        let mut results = Vec::new();

        for hook in hooks.iter() {
            let start = std::time::Instant::now();
            let result = hook.execute(reason).await;
            let duration_ms = start.elapsed().as_millis() as u64;

            results.push(match result {
                Ok(output) => HookResult {
                    name: hook.name().to_string(),
                    success: true,
                    duration_ms,
                    output,
                    error: None,
                },
                Err(e) => HookResult {
                    name: hook.name().to_string(),
                    success: false,
                    duration_ms,
                    output: None,
                    error: Some(e.to_string()),
                },
            });
        }

        results
    }

    /// Record history entry.
    async fn record_history(&self, entry: RebootHistoryEntry) {
        let mut history = self.history.write().await;
        history.push_back(entry);

        // Keep last 100 entries
        while history.len() > 100 {
            history.pop_front();
        }
    }

    /// Trigger a manual reboot.
    pub async fn manual_reboot(&self) -> RebootResult {
        self.perform_reboot(RebootReason::Manual).await
    }

    /// Get reboot history.
    pub async fn get_history(&self) -> Vec<RebootHistoryEntry> {
        self.history.read().await.iter().cloned().collect()
    }

    /// Increment iteration counter.
    pub fn increment_iteration(&self) {
        self.current_iteration.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get stats.
    pub async fn stats(&self) -> RebootStats {
        let history = self.history.read().await;
        let now = chrono::Utc::now();
        let hour_ago = now - chrono::Duration::hours(1);

        RebootStats {
            total_reboots: history.len(),
            successful_reboots: history.iter().filter(|e| e.result.success).count(),
            reboots_last_hour: history.iter().filter(|e| e.timestamp > hour_ago).count(),
            consecutive_failures: self.consecutive_failures.load(std::sync::atomic::Ordering::Relaxed),
            last_reboot: self.last_reboot.read().await.clone(),
        }
    }
}

/// Reboot statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebootStats {
    pub total_reboots: usize,
    pub successful_reboots: usize,
    pub reboots_last_hour: usize,
    pub consecutive_failures: u32,
    pub last_reboot: Option<chrono::DateTime<chrono::Utc>>,
}

use serde::{Deserialize, Serialize};
```

### 3. Module Root (src/reboot/mod.rs)

```rust
//! Automatic reboot management.

pub mod manager;
pub mod types;

pub use manager::{AutoRebootManager, RebootHook, RebootStats};
pub use types::{
    AutoRebootConfig, HookResult, RebootHistoryEntry, RebootMode,
    RebootReason, RebootResult, RebootTriggers,
};
```

---

## Testing Requirements

1. Auto-reboot triggers on redline
2. Rate limiting prevents excessive reboots
3. Pre-hooks execute before reboot
4. Post-hooks execute after successful reboot
5. Failed hooks abort reboot
6. Manual reboot bypasses triggers
7. History is properly recorded
8. Consecutive failures tracked correctly

---

## Related Specs

- Depends on: [101-fresh-context.md](101-fresh-context.md)
- Depends on: [102-redline-detection.md](102-redline-detection.md)
- Next: [104-stop-conditions.md](104-stop-conditions.md)
- Related: [113-loop-hooks.md](113-loop-hooks.md)
