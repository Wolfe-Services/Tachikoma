# 155 - Forge Timeout Handling

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 155
**Status:** Planned
**Dependencies:** 139-forge-rounds, 137-forge-config
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive timeout handling for Forge sessions, including per-round timeouts, session-level limits, and graceful degradation when limits are exceeded.

---

## Acceptance Criteria

- [x] Per-round timeout enforcement
- [x] Session-level time limit
- [x] Graceful timeout recovery
- [x] Timeout event notification
- [x] Configurable timeout values
- [x] Extension mechanism for attended mode

---

## Implementation Details

### 1. Timeout Manager (src/timeout/manager.rs)

```rust
//! Timeout management for Forge sessions.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, timeout};

use crate::{ForgeConfig, ForgeError, ForgeEvent, ForgeResult, ForgeSessionStatus};

/// Manages timeouts for a Forge session.
pub struct TimeoutManager {
    config: ForgeConfig,
    /// Session start time.
    session_start: Instant,
    /// Current round start time.
    round_start: Arc<RwLock<Option<Instant>>>,
    /// Event sender for timeout notifications.
    event_tx: broadcast::Sender<ForgeEvent>,
    /// Timeout extension requests.
    extension_rx: mpsc::Receiver<TimeoutExtension>,
    /// Current extensions.
    extensions: Arc<RwLock<TimeoutExtensions>>,
}

/// Timeout extension request.
#[derive(Debug, Clone)]
pub struct TimeoutExtension {
    /// What to extend.
    pub target: ExtensionTarget,
    /// Additional duration in seconds.
    pub additional_secs: u64,
    /// Reason for extension.
    pub reason: String,
}

/// What can be extended.
#[derive(Debug, Clone, Copy)]
pub enum ExtensionTarget {
    /// Current round.
    CurrentRound,
    /// Session overall.
    Session,
    /// Both.
    Both,
}

/// Active extensions.
#[derive(Debug, Clone, Default)]
pub struct TimeoutExtensions {
    /// Extra round time.
    pub round_extension_secs: u64,
    /// Extra session time.
    pub session_extension_secs: u64,
}

/// Timeout check result.
#[derive(Debug, Clone)]
pub enum TimeoutStatus {
    /// Within limits.
    Ok,
    /// Warning - approaching limit.
    Warning { remaining_secs: u64, message: String },
    /// Exceeded limit.
    Exceeded { limit_type: TimeoutType, by_secs: u64 },
}

/// Type of timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutType {
    Round,
    Session,
}

impl TimeoutManager {
    /// Create a new timeout manager.
    pub fn new(
        config: ForgeConfig,
        event_tx: broadcast::Sender<ForgeEvent>,
    ) -> (Self, mpsc::Sender<TimeoutExtension>) {
        let (extension_tx, extension_rx) = mpsc::channel(10);

        let manager = Self {
            config,
            session_start: Instant::now(),
            round_start: Arc::new(RwLock::new(None)),
            event_tx,
            extension_rx,
            extensions: Arc::new(RwLock::new(TimeoutExtensions::default())),
        };

        (manager, extension_tx)
    }

    /// Mark start of a new round.
    pub async fn start_round(&self) {
        *self.round_start.write().await = Some(Instant::now());
    }

    /// Mark end of current round.
    pub async fn end_round(&self) {
        *self.round_start.write().await = None;
    }

    /// Check current timeout status.
    pub async fn check_status(&self, round_type: &str) -> TimeoutStatus {
        // Check session timeout first
        let session_status = self.check_session_timeout().await;
        if let TimeoutStatus::Exceeded { .. } = session_status {
            return session_status;
        }

        // Check round timeout
        let round_status = self.check_round_timeout(round_type).await;
        if let TimeoutStatus::Exceeded { .. } = round_status {
            return round_status;
        }

        // Return warning if either has one
        if let TimeoutStatus::Warning { .. } = round_status {
            return round_status;
        }

        session_status
    }

    /// Check session-level timeout.
    async fn check_session_timeout(&self) -> TimeoutStatus {
        let extensions = self.extensions.read().await;
        let max_duration = Duration::from_secs(
            self.config.limits.max_duration_secs + extensions.session_extension_secs
        );

        let elapsed = self.session_start.elapsed();

        if elapsed > max_duration {
            return TimeoutStatus::Exceeded {
                limit_type: TimeoutType::Session,
                by_secs: (elapsed - max_duration).as_secs(),
            };
        }

        let remaining = max_duration - elapsed;
        let warning_threshold = max_duration.mul_f64(0.1); // Warn at 10% remaining

        if remaining < warning_threshold {
            return TimeoutStatus::Warning {
                remaining_secs: remaining.as_secs(),
                message: format!(
                    "Session timeout in {} seconds",
                    remaining.as_secs()
                ),
            };
        }

        TimeoutStatus::Ok
    }

    /// Check round-level timeout.
    async fn check_round_timeout(&self, round_type: &str) -> TimeoutStatus {
        let round_start = self.round_start.read().await;
        let Some(start) = *round_start else {
            return TimeoutStatus::Ok;
        };

        let max_duration = self.get_round_timeout(round_type).await;
        let elapsed = start.elapsed();

        if elapsed > max_duration {
            return TimeoutStatus::Exceeded {
                limit_type: TimeoutType::Round,
                by_secs: (elapsed - max_duration).as_secs(),
            };
        }

        let remaining = max_duration - elapsed;
        let warning_threshold = max_duration.mul_f64(0.2); // Warn at 20% remaining

        if remaining < warning_threshold {
            return TimeoutStatus::Warning {
                remaining_secs: remaining.as_secs(),
                message: format!(
                    "{} round timeout in {} seconds",
                    round_type,
                    remaining.as_secs()
                ),
            };
        }

        TimeoutStatus::Ok
    }

    /// Get timeout for a round type.
    async fn get_round_timeout(&self, round_type: &str) -> Duration {
        let extensions = self.extensions.read().await;
        let base_secs = match round_type.to_lowercase().as_str() {
            "draft" => self.config.rounds.draft.timeout_secs,
            "critique" => self.config.rounds.critique.timeout_secs,
            "synthesis" => self.config.rounds.synthesis.timeout_secs,
            "refinement" => self.config.rounds.refinement.timeout_secs,
            _ => 120, // Default
        };

        Duration::from_secs(base_secs + extensions.round_extension_secs)
    }

    /// Process extension requests.
    pub async fn process_extensions(&mut self) {
        while let Ok(ext) = self.extension_rx.try_recv() {
            let mut extensions = self.extensions.write().await;

            match ext.target {
                ExtensionTarget::CurrentRound => {
                    extensions.round_extension_secs += ext.additional_secs;
                }
                ExtensionTarget::Session => {
                    extensions.session_extension_secs += ext.additional_secs;
                }
                ExtensionTarget::Both => {
                    extensions.round_extension_secs += ext.additional_secs;
                    extensions.session_extension_secs += ext.additional_secs;
                }
            }

            tracing::info!(
                "Timeout extended: {:?} by {}s - {}",
                ext.target,
                ext.additional_secs,
                ext.reason
            );
        }
    }

    /// Get time remaining for session.
    pub async fn session_time_remaining(&self) -> Duration {
        let extensions = self.extensions.read().await;
        let max_duration = Duration::from_secs(
            self.config.limits.max_duration_secs + extensions.session_extension_secs
        );

        let elapsed = self.session_start.elapsed();
        max_duration.saturating_sub(elapsed)
    }

    /// Get time elapsed in current round.
    pub async fn round_time_elapsed(&self) -> Option<Duration> {
        self.round_start.read().await.map(|start| start.elapsed())
    }

    /// Emit timeout warning event.
    fn emit_warning(&self, message: &str) {
        let _ = self.event_tx.send(ForgeEvent::Error {
            message: format!("Timeout warning: {}", message),
            recoverable: true,
        });
    }
}

/// Wrapper for executing with timeout.
pub async fn with_round_timeout<T, F, Fut>(
    timeout_secs: u64,
    round_type: &str,
    f: F,
) -> ForgeResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ForgeResult<T>>,
{
    let duration = Duration::from_secs(timeout_secs);

    match timeout(duration, f()).await {
        Ok(result) => result,
        Err(_) => Err(ForgeError::Timeout(format!(
            "{} round timed out after {}s",
            round_type,
            timeout_secs
        ))),
    }
}

/// Wrapper for executing with timeout and retries.
pub async fn with_timeout_retry<T, F, Fut>(
    timeout_secs: u64,
    max_retries: usize,
    operation_name: &str,
    f: F,
) -> ForgeResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = ForgeResult<T>>,
{
    let duration = Duration::from_secs(timeout_secs);

    for attempt in 0..=max_retries {
        match timeout(duration, f()).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(e)) if e.is_recoverable() && attempt < max_retries => {
                let delay = Duration::from_secs(2u64.pow(attempt as u32));
                tracing::warn!(
                    "{} attempt {} failed: {}. Retrying in {:?}",
                    operation_name,
                    attempt + 1,
                    e,
                    delay
                );
                tokio::time::sleep(delay).await;
            }
            Ok(Err(e)) => return Err(e),
            Err(_) if attempt < max_retries => {
                let delay = Duration::from_secs(2u64.pow(attempt as u32));
                tracing::warn!(
                    "{} attempt {} timed out. Retrying in {:?}",
                    operation_name,
                    attempt + 1,
                    delay
                );
                tokio::time::sleep(delay).await;
            }
            Err(_) => {
                return Err(ForgeError::Timeout(format!(
                    "{} timed out after {} attempts",
                    operation_name,
                    max_retries + 1
                )));
            }
        }
    }

    Err(ForgeError::Timeout(format!(
        "{} failed after all retries",
        operation_name
    )))
}

/// Monitor task for timeout warnings.
pub fn spawn_timeout_monitor(
    config: ForgeConfig,
    event_tx: broadcast::Sender<ForgeEvent>,
    session_start: Instant,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(30));
        let max_duration = Duration::from_secs(config.limits.max_duration_secs);

        loop {
            check_interval.tick().await;

            let elapsed = session_start.elapsed();
            let remaining = max_duration.saturating_sub(elapsed);

            // Warn at 10%, 5%, and 1% remaining
            for threshold in [0.1, 0.05, 0.01] {
                let threshold_duration = max_duration.mul_f64(threshold);
                if remaining <= threshold_duration && remaining > Duration::ZERO {
                    let _ = event_tx.send(ForgeEvent::Error {
                        message: format!(
                            "Session timeout warning: {} seconds remaining",
                            remaining.as_secs()
                        ),
                        recoverable: true,
                    });
                    break;
                }
            }

            if remaining == Duration::ZERO {
                let _ = event_tx.send(ForgeEvent::Error {
                    message: "Session has timed out".to_string(),
                    recoverable: false,
                });
                break;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_round_timeout_success() {
        let result = with_round_timeout(1, "test", || async {
            Ok::<_, ForgeError>("success".to_string())
        }).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_with_round_timeout_expires() {
        let result = with_round_timeout(1, "test", || async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok::<_, ForgeError>("success".to_string())
        }).await;

        assert!(matches!(result, Err(ForgeError::Timeout(_))));
    }
}
```

---

## Testing Requirements

1. Round timeouts enforce limits correctly
2. Session timeouts enforce total time
3. Extensions add to base timeout
4. Warnings emit at configured thresholds
5. Retry logic handles timeout correctly
6. Monitor task emits warnings

---

## Related Specs

- Depends on: [139-forge-rounds.md](139-forge-rounds.md)
- Depends on: [137-forge-config.md](137-forge-config.md)
- Next: [156-forge-cost.md](156-forge-cost.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
