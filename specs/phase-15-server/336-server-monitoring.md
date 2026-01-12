# 336 - Server Monitoring

**Phase:** 15 - Server
**Spec ID:** 336
**Status:** Planned
**Dependencies:** 327-health-endpoints, 328-metrics-endpoints
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement comprehensive server monitoring with resource tracking, alerting hooks, and observability integrations.

---

## Acceptance Criteria

- [ ] Resource monitoring (CPU, memory, disk)
- [ ] Connection tracking
- [ ] Request rate monitoring
- [ ] Error rate tracking
- [ ] Latency percentiles
- [ ] Alert threshold configuration
- [ ] Observability export

---

## Implementation Details

### 1. Resource Monitor (crates/tachikoma-server/src/monitoring/resources.rs)

```rust
//! Resource monitoring.

use serde::Serialize;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, warn};

/// Resource usage snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceSnapshot {
    /// CPU usage percentage (0-100).
    pub cpu_percent: f64,
    /// Memory usage in bytes.
    pub memory_bytes: u64,
    /// Memory usage percentage.
    pub memory_percent: f64,
    /// Open file descriptors.
    pub open_fds: u64,
    /// Thread count.
    pub thread_count: u64,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ResourceSnapshot {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_bytes: 0,
            memory_percent: 0.0,
            open_fds: 0,
            thread_count: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Resource monitor collecting system metrics.
pub struct ResourceMonitor {
    interval: Duration,
    sender: watch::Sender<ResourceSnapshot>,
    receiver: watch::Receiver<ResourceSnapshot>,
}

impl ResourceMonitor {
    pub fn new(interval: Duration) -> Self {
        let (sender, receiver) = watch::channel(ResourceSnapshot::default());
        Self {
            interval,
            sender,
            receiver,
        }
    }

    /// Start the monitoring loop.
    pub fn start(&self) {
        let interval = self.interval;
        let sender = self.sender.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                let snapshot = collect_resources();
                debug!(
                    cpu = snapshot.cpu_percent,
                    memory_mb = snapshot.memory_bytes / 1_000_000,
                    "Resource snapshot"
                );

                let _ = sender.send(snapshot);
            }
        });
    }

    /// Get the latest snapshot.
    pub fn get(&self) -> ResourceSnapshot {
        self.receiver.borrow().clone()
    }

    /// Subscribe to snapshots.
    pub fn subscribe(&self) -> watch::Receiver<ResourceSnapshot> {
        self.receiver.clone()
    }
}

#[cfg(target_os = "linux")]
fn collect_resources() -> ResourceSnapshot {
    use std::fs;

    let mut snapshot = ResourceSnapshot::default();
    snapshot.timestamp = chrono::Utc::now();

    // Read /proc/self/stat for CPU and memory
    if let Ok(stat) = fs::read_to_string("/proc/self/stat") {
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() > 23 {
            // Thread count is at index 19
            if let Ok(threads) = parts[19].parse::<u64>() {
                snapshot.thread_count = threads;
            }
        }
    }

    // Read /proc/self/status for memory
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb.parse::<u64>() {
                        snapshot.memory_bytes = kb * 1024;
                    }
                }
            }
        }
    }

    // Read /proc/self/fd for open file descriptors
    if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        snapshot.open_fds = entries.count() as u64;
    }

    // Get total memory for percentage calculation
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(total_kb) = kb.parse::<u64>() {
                        let total_bytes = total_kb * 1024;
                        if total_bytes > 0 {
                            snapshot.memory_percent =
                                (snapshot.memory_bytes as f64 / total_bytes as f64) * 100.0;
                        }
                    }
                }
            }
        }
    }

    snapshot
}

#[cfg(not(target_os = "linux"))]
fn collect_resources() -> ResourceSnapshot {
    // Fallback for non-Linux systems
    ResourceSnapshot::default()
}
```

### 2. Request Monitor (crates/tachikoma-server/src/monitoring/requests.rs)

```rust
//! Request monitoring and statistics.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Request statistics.
#[derive(Debug, Clone, Default)]
pub struct RequestStats {
    /// Total requests.
    pub total_requests: u64,
    /// Requests in the last minute.
    pub requests_per_minute: f64,
    /// Error count.
    pub error_count: u64,
    /// Error rate (percentage).
    pub error_rate: f64,
    /// Average latency (ms).
    pub avg_latency_ms: f64,
    /// P50 latency (ms).
    pub p50_latency_ms: f64,
    /// P95 latency (ms).
    pub p95_latency_ms: f64,
    /// P99 latency (ms).
    pub p99_latency_ms: f64,
    /// Max latency (ms).
    pub max_latency_ms: f64,
}

/// Request monitor tracking request metrics.
pub struct RequestMonitor {
    /// Total request counter.
    total_requests: AtomicU64,
    /// Error counter.
    error_count: AtomicU64,
    /// Recent request latencies (for percentile calculation).
    latencies: Arc<RwLock<VecDeque<RequestRecord>>>,
    /// Window duration.
    window: Duration,
}

struct RequestRecord {
    timestamp: Instant,
    latency_ms: f64,
    is_error: bool,
}

impl RequestMonitor {
    pub fn new(window: Duration) -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            latencies: Arc::new(RwLock::new(VecDeque::new())),
            window,
        }
    }

    /// Record a request.
    pub async fn record(&self, latency_ms: f64, is_error: bool) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        if is_error {
            self.error_count.fetch_add(1, Ordering::SeqCst);
        }

        let record = RequestRecord {
            timestamp: Instant::now(),
            latency_ms,
            is_error,
        };

        let mut latencies = self.latencies.write().await;
        latencies.push_back(record);

        // Clean old records
        let cutoff = Instant::now() - self.window;
        while let Some(front) = latencies.front() {
            if front.timestamp < cutoff {
                latencies.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get current statistics.
    pub async fn stats(&self) -> RequestStats {
        let latencies = self.latencies.read().await;
        let total = self.total_requests.load(Ordering::SeqCst);
        let errors = self.error_count.load(Ordering::SeqCst);

        if latencies.is_empty() {
            return RequestStats {
                total_requests: total,
                error_count: errors,
                ..Default::default()
            };
        }

        // Collect latency values
        let mut values: Vec<f64> = latencies.iter().map(|r| r.latency_ms).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let window_errors = latencies.iter().filter(|r| r.is_error).count();
        let window_total = latencies.len();

        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let p50 = percentile(&values, 50.0);
        let p95 = percentile(&values, 95.0);
        let p99 = percentile(&values, 99.0);
        let max = values.last().copied().unwrap_or(0.0);

        // Calculate requests per minute
        let oldest = latencies.front().map(|r| r.timestamp);
        let newest = latencies.back().map(|r| r.timestamp);
        let rpm = if let (Some(old), Some(new)) = (oldest, newest) {
            let duration = new.duration_since(old).as_secs_f64();
            if duration > 0.0 {
                (window_total as f64 / duration) * 60.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        RequestStats {
            total_requests: total,
            requests_per_minute: rpm,
            error_count: errors,
            error_rate: if window_total > 0 {
                (window_errors as f64 / window_total as f64) * 100.0
            } else {
                0.0
            },
            avg_latency_ms: avg,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            max_latency_ms: max,
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (p / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values.get(index).copied().unwrap_or(0.0)
}
```

### 3. Alert Manager (crates/tachikoma-server/src/monitoring/alerts.rs)

```rust
//! Alert management.

use super::{requests::RequestStats, resources::ResourceSnapshot};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert definition.
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
}

/// Alert thresholds configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU warning threshold (percentage).
    pub cpu_warning: f64,
    /// CPU critical threshold.
    pub cpu_critical: f64,
    /// Memory warning threshold (percentage).
    pub memory_warning: f64,
    /// Memory critical threshold.
    pub memory_critical: f64,
    /// Error rate warning threshold (percentage).
    pub error_rate_warning: f64,
    /// Error rate critical threshold.
    pub error_rate_critical: f64,
    /// P99 latency warning (ms).
    pub latency_warning_ms: f64,
    /// P99 latency critical (ms).
    pub latency_critical_ms: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 75.0,
            memory_critical: 90.0,
            error_rate_warning: 1.0,
            error_rate_critical: 5.0,
            latency_warning_ms: 500.0,
            latency_critical_ms: 2000.0,
        }
    }
}

/// Alert manager for generating and routing alerts.
pub struct AlertManager {
    thresholds: AlertThresholds,
    sender: mpsc::Sender<Alert>,
    receiver: mpsc::Receiver<Alert>,
}

impl AlertManager {
    pub fn new(thresholds: AlertThresholds) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        Self {
            thresholds,
            sender,
            receiver,
        }
    }

    /// Get alert sender for publishing alerts.
    pub fn sender(&self) -> mpsc::Sender<Alert> {
        self.sender.clone()
    }

    /// Check resources and generate alerts.
    pub async fn check_resources(&self, snapshot: &ResourceSnapshot) {
        // CPU alerts
        if snapshot.cpu_percent >= self.thresholds.cpu_critical {
            self.send_alert(Alert {
                id: "cpu_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("CPU usage critical: {:.1}%", snapshot.cpu_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({ "cpu_percent": snapshot.cpu_percent }),
            })
            .await;
        } else if snapshot.cpu_percent >= self.thresholds.cpu_warning {
            self.send_alert(Alert {
                id: "cpu_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("CPU usage high: {:.1}%", snapshot.cpu_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({ "cpu_percent": snapshot.cpu_percent }),
            })
            .await;
        }

        // Memory alerts
        if snapshot.memory_percent >= self.thresholds.memory_critical {
            self.send_alert(Alert {
                id: "memory_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("Memory usage critical: {:.1}%", snapshot.memory_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "memory_percent": snapshot.memory_percent,
                    "memory_bytes": snapshot.memory_bytes,
                }),
            })
            .await;
        } else if snapshot.memory_percent >= self.thresholds.memory_warning {
            self.send_alert(Alert {
                id: "memory_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("Memory usage high: {:.1}%", snapshot.memory_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "memory_percent": snapshot.memory_percent,
                    "memory_bytes": snapshot.memory_bytes,
                }),
            })
            .await;
        }
    }

    /// Check request metrics and generate alerts.
    pub async fn check_requests(&self, stats: &RequestStats) {
        // Error rate alerts
        if stats.error_rate >= self.thresholds.error_rate_critical {
            self.send_alert(Alert {
                id: "error_rate_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("Error rate critical: {:.2}%", stats.error_rate),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "error_rate": stats.error_rate,
                    "error_count": stats.error_count,
                }),
            })
            .await;
        } else if stats.error_rate >= self.thresholds.error_rate_warning {
            self.send_alert(Alert {
                id: "error_rate_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("Error rate elevated: {:.2}%", stats.error_rate),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "error_rate": stats.error_rate,
                    "error_count": stats.error_count,
                }),
            })
            .await;
        }

        // Latency alerts
        if stats.p99_latency_ms >= self.thresholds.latency_critical_ms {
            self.send_alert(Alert {
                id: "latency_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("P99 latency critical: {:.0}ms", stats.p99_latency_ms),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "p99_latency_ms": stats.p99_latency_ms,
                    "avg_latency_ms": stats.avg_latency_ms,
                }),
            })
            .await;
        } else if stats.p99_latency_ms >= self.thresholds.latency_warning_ms {
            self.send_alert(Alert {
                id: "latency_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("P99 latency elevated: {:.0}ms", stats.p99_latency_ms),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "p99_latency_ms": stats.p99_latency_ms,
                    "avg_latency_ms": stats.avg_latency_ms,
                }),
            })
            .await;
        }
    }

    async fn send_alert(&self, alert: Alert) {
        match alert.severity {
            AlertSeverity::Critical => error!(
                alert_id = %alert.id,
                message = %alert.message,
                "CRITICAL ALERT"
            ),
            AlertSeverity::Warning => warn!(
                alert_id = %alert.id,
                message = %alert.message,
                "WARNING ALERT"
            ),
            AlertSeverity::Info => info!(
                alert_id = %alert.id,
                message = %alert.message,
                "INFO ALERT"
            ),
        }

        let _ = self.sender.send(alert).await;
    }
}
```

---

## Testing Requirements

1. Resource collection accurate
2. Request stats calculated correctly
3. Percentiles accurate
4. Alerts triggered at thresholds
5. Alert routing works
6. Memory monitoring works
7. Error rate tracking accurate

---

## Related Specs

- Depends on: [327-health-endpoints.md](327-health-endpoints.md), [328-metrics-endpoints.md](328-metrics-endpoints.md)
- Next: [337-server-scaling.md](337-server-scaling.md)
- Used by: Operations, alerting
