# 071 - Backend Health Checks

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 071
**Status:** Planned
**Dependencies:** 051-backend-trait, 070-backend-factory
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement health checking infrastructure for backends, including periodic health monitoring, status reporting, and automatic failover when backends become unavailable.

---

## Acceptance Criteria

- [x] Health check interface on all backends
- [x] Periodic health monitoring
- [x] Health status aggregation
- [x] Automatic failover support
- [x] Health history tracking
- [x] Alerting on status changes

---

## Implementation Details

### 1. Health Types (src/health/types.rs)

```rust
//! Health check types.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Health status of a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Backend is healthy and responding.
    Healthy,
    /// Backend is responding but degraded.
    Degraded,
    /// Backend is not responding.
    Unhealthy,
    /// Health status is unknown.
    Unknown,
}

impl HealthStatus {
    /// Check if the backend is usable.
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Check if the backend is fully healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Result of a health check.
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Status of the backend.
    pub status: HealthStatus,
    /// Response latency.
    pub latency: Duration,
    /// Timestamp of the check.
    pub timestamp: Instant,
    /// Optional error message.
    pub error: Option<String>,
    /// Additional details.
    pub details: HealthDetails,
}

impl HealthCheckResult {
    /// Create a healthy result.
    pub fn healthy(latency: Duration) -> Self {
        Self {
            status: HealthStatus::Healthy,
            latency,
            timestamp: Instant::now(),
            error: None,
            details: HealthDetails::default(),
        }
    }

    /// Create a degraded result.
    pub fn degraded(latency: Duration, reason: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            latency,
            timestamp: Instant::now(),
            error: Some(reason.into()),
            details: HealthDetails::default(),
        }
    }

    /// Create an unhealthy result.
    pub fn unhealthy(error: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            latency: Duration::ZERO,
            timestamp: Instant::now(),
            error: Some(error.into()),
            details: HealthDetails::default(),
        }
    }

    /// Set details.
    pub fn with_details(mut self, details: HealthDetails) -> Self {
        self.details = details;
        self
    }
}

/// Additional health details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthDetails {
    /// API version reported.
    pub api_version: Option<String>,
    /// Model availability.
    pub models_available: Option<bool>,
    /// Rate limit status.
    pub rate_limit_remaining: Option<u32>,
    /// Current load estimate.
    pub load_percent: Option<f32>,
}

/// Health check configuration.
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between checks.
    pub interval: Duration,
    /// Timeout for each check.
    pub timeout: Duration,
    /// Number of failures before unhealthy.
    pub failure_threshold: u32,
    /// Number of successes before healthy.
    pub success_threshold: u32,
    /// Latency threshold for degraded status.
    pub degraded_latency: Duration,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
            success_threshold: 2,
            degraded_latency: Duration::from_secs(5),
        }
    }
}
```

### 2. Health Monitor (src/health/monitor.rs)

```rust
//! Health monitoring for backends.

use super::types::{HealthCheckConfig, HealthCheckResult, HealthStatus};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tachikoma_backends_core::Backend;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Health monitor for a single backend.
pub struct HealthMonitor {
    backend: Arc<dyn Backend>,
    config: HealthCheckConfig,
    state: Arc<RwLock<MonitorState>>,
    event_tx: Option<mpsc::Sender<HealthEvent>>,
}

/// Internal monitor state.
#[derive(Debug)]
struct MonitorState {
    /// Current status.
    status: HealthStatus,
    /// Recent check results.
    history: VecDeque<HealthCheckResult>,
    /// Consecutive failures.
    consecutive_failures: u32,
    /// Consecutive successes.
    consecutive_successes: u32,
    /// Last check time.
    last_check: Option<Instant>,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            history: VecDeque::with_capacity(100),
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_check: None,
        }
    }
}

/// Health event for notifications.
#[derive(Debug, Clone)]
pub struct HealthEvent {
    pub backend_name: String,
    pub old_status: HealthStatus,
    pub new_status: HealthStatus,
    pub timestamp: Instant,
}

impl HealthMonitor {
    /// Create a new health monitor.
    pub fn new(backend: Arc<dyn Backend>, config: HealthCheckConfig) -> Self {
        Self {
            backend,
            config,
            state: Arc::new(RwLock::new(MonitorState::default())),
            event_tx: None,
        }
    }

    /// Set event channel for notifications.
    pub fn with_events(mut self, tx: mpsc::Sender<HealthEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Get current health status.
    pub async fn status(&self) -> HealthStatus {
        self.state.read().await.status
    }

    /// Get recent health history.
    pub async fn history(&self) -> Vec<HealthCheckResult> {
        self.state.read().await.history.iter().cloned().collect()
    }

    /// Perform a single health check.
    pub async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        let result = match tokio::time::timeout(
            self.config.timeout,
            self.backend.health_check(),
        )
        .await
        {
            Ok(Ok(true)) => {
                let latency = start.elapsed();
                if latency > self.config.degraded_latency {
                    HealthCheckResult::degraded(latency, "High latency")
                } else {
                    HealthCheckResult::healthy(latency)
                }
            }
            Ok(Ok(false)) => HealthCheckResult::unhealthy("Health check returned false"),
            Ok(Err(e)) => HealthCheckResult::unhealthy(e.to_string()),
            Err(_) => HealthCheckResult::unhealthy("Health check timed out"),
        };

        // Update state
        self.update_state(result.clone()).await;

        result
    }

    /// Update internal state with check result.
    async fn update_state(&self, result: HealthCheckResult) {
        let mut state = self.state.write().await;
        let old_status = state.status;

        // Update counters
        if result.status.is_healthy() {
            state.consecutive_successes += 1;
            state.consecutive_failures = 0;
        } else {
            state.consecutive_failures += 1;
            state.consecutive_successes = 0;
        }

        // Determine new status
        let new_status = if state.consecutive_failures >= self.config.failure_threshold {
            HealthStatus::Unhealthy
        } else if state.consecutive_successes >= self.config.success_threshold {
            if result.status == HealthStatus::Degraded {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            }
        } else {
            // Keep previous status during transition
            state.status
        };

        state.status = new_status;
        state.last_check = Some(Instant::now());

        // Add to history (keep last 100)
        if state.history.len() >= 100 {
            state.history.pop_front();
        }
        state.history.push_back(result);

        // Emit event on status change
        if old_status != new_status {
            info!(
                backend = %self.backend.name(),
                old = ?old_status,
                new = ?new_status,
                "Health status changed"
            );

            if let Some(tx) = &self.event_tx {
                let event = HealthEvent {
                    backend_name: self.backend.name().to_string(),
                    old_status,
                    new_status,
                    timestamp: Instant::now(),
                };
                let _ = tx.send(event).await;
            }
        }
    }

    /// Start periodic monitoring.
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let monitor = self;
        tokio::spawn(async move {
            let mut interval = interval(monitor.config.interval);
            loop {
                interval.tick().await;
                debug!(backend = %monitor.backend.name(), "Running health check");
                monitor.check().await;
            }
        })
    }
}
```

### 3. Aggregate Health (src/health/aggregate.rs)

```rust
//! Aggregate health status for multiple backends.

use super::monitor::{HealthEvent, HealthMonitor};
use super::types::{HealthCheckConfig, HealthStatus};
use crate::factory::BackendProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tachikoma_backends_core::Backend;
use tokio::sync::{mpsc, RwLock};
use tracing::info;

/// Aggregate health checker for all backends.
pub struct AggregateHealth {
    monitors: RwLock<HashMap<BackendProvider, Arc<HealthMonitor>>>,
    config: HealthCheckConfig,
    event_tx: mpsc::Sender<HealthEvent>,
    event_rx: RwLock<Option<mpsc::Receiver<HealthEvent>>>,
}

impl AggregateHealth {
    /// Create a new aggregate health checker.
    pub fn new(config: HealthCheckConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            monitors: RwLock::new(HashMap::new()),
            config,
            event_tx: tx,
            event_rx: RwLock::new(Some(rx)),
        }
    }

    /// Add a backend to monitor.
    pub async fn add_backend(&self, provider: BackendProvider, backend: Arc<dyn Backend>) {
        let monitor = Arc::new(
            HealthMonitor::new(backend, self.config.clone())
                .with_events(self.event_tx.clone()),
        );

        self.monitors.write().await.insert(provider, monitor);
    }

    /// Remove a backend.
    pub async fn remove_backend(&self, provider: BackendProvider) {
        self.monitors.write().await.remove(&provider);
    }

    /// Get status for a specific backend.
    pub async fn status(&self, provider: BackendProvider) -> Option<HealthStatus> {
        let monitors = self.monitors.read().await;
        if let Some(monitor) = monitors.get(&provider) {
            Some(monitor.status().await)
        } else {
            None
        }
    }

    /// Get status for all backends.
    pub async fn all_statuses(&self) -> HashMap<BackendProvider, HealthStatus> {
        let monitors = self.monitors.read().await;
        let mut statuses = HashMap::new();

        for (provider, monitor) in monitors.iter() {
            statuses.insert(*provider, monitor.status().await);
        }

        statuses
    }

    /// Get overall system health.
    pub async fn overall_health(&self) -> HealthStatus {
        let statuses = self.all_statuses().await;

        if statuses.is_empty() {
            return HealthStatus::Unknown;
        }

        let healthy_count = statuses.values().filter(|s| s.is_healthy()).count();
        let usable_count = statuses.values().filter(|s| s.is_usable()).count();

        if healthy_count == statuses.len() {
            HealthStatus::Healthy
        } else if usable_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    /// Get healthy backends.
    pub async fn healthy_backends(&self) -> Vec<BackendProvider> {
        self.all_statuses()
            .await
            .into_iter()
            .filter(|(_, s)| s.is_usable())
            .map(|(p, _)| p)
            .collect()
    }

    /// Start all monitors.
    pub async fn start_all(&self) {
        let monitors = self.monitors.read().await;
        for (provider, monitor) in monitors.iter() {
            info!(provider = %provider, "Starting health monitor");
            Arc::clone(monitor).start();
        }
    }

    /// Take the event receiver.
    pub async fn take_event_receiver(&self) -> Option<mpsc::Receiver<HealthEvent>> {
        self.event_rx.write().await.take()
    }

    /// Check all backends now.
    pub async fn check_all(&self) -> HashMap<BackendProvider, HealthStatus> {
        let monitors = self.monitors.read().await;
        let mut results = HashMap::new();

        for (provider, monitor) in monitors.iter() {
            let result = monitor.check().await;
            results.insert(*provider, result.status);
        }

        results
    }
}

impl Default for AggregateHealth {
    fn default() -> Self {
        Self::new(HealthCheckConfig::default())
    }
}
```

### 4. Module Exports (src/health/mod.rs)

```rust
//! Health checking for backends.

mod aggregate;
mod monitor;
mod types;

pub use aggregate::AggregateHealth;
pub use monitor::{HealthEvent, HealthMonitor};
pub use types::{HealthCheckConfig, HealthCheckResult, HealthDetails, HealthStatus};
```

---

## Testing Requirements

1. Health check detects healthy backends
2. Health check detects failures
3. Status transitions respect thresholds
4. Aggregate health reflects all backends
5. Events are emitted on status changes

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Depends on: [070-backend-factory.md](070-backend-factory.md)
- Next: [072-backend-rate-limit.md](072-backend-rate-limit.md)
