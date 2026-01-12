# 108 - Loop Metrics

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 108
**Status:** Planned
**Dependencies:** 096-loop-runner-core, 028-metrics-foundation
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement comprehensive metrics collection for the Ralph Loop - tracking execution statistics, performance data, resource usage, and enabling observability and debugging.

---

## Acceptance Criteria

- [x] Iteration metrics (duration, success rate)
- [x] Session metrics (context usage, reboots)
- [x] Test metrics (pass rate, failure trends)
- [x] Progress metrics (velocity, efficiency)
- [x] Resource metrics (memory, CPU hints)
- [x] Metric aggregation and rollups
- [x] Metric export (Prometheus format)
- [x] Real-time metric streaming

---

## Implementation Details

### 1. Metric Types (src/metrics/types.rs)

```rust
//! Loop metrics type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// A metric measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Metric {
    /// Counter (monotonically increasing).
    Counter {
        name: String,
        value: u64,
        labels: HashMap<String, String>,
    },
    /// Gauge (can go up or down).
    Gauge {
        name: String,
        value: f64,
        labels: HashMap<String, String>,
    },
    /// Histogram (distribution of values).
    Histogram {
        name: String,
        count: u64,
        sum: f64,
        buckets: Vec<HistogramBucket>,
        labels: HashMap<String, String>,
    },
    /// Summary (quantiles).
    Summary {
        name: String,
        count: u64,
        sum: f64,
        quantiles: Vec<Quantile>,
        labels: HashMap<String, String>,
    },
}

/// A histogram bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Upper bound (inclusive).
    pub le: f64,
    /// Cumulative count.
    pub count: u64,
}

/// A quantile measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    /// Quantile (0-1).
    pub quantile: f64,
    /// Value at quantile.
    pub value: f64,
}

impl Metric {
    /// Create a counter.
    pub fn counter(name: impl Into<String>, value: u64) -> Self {
        Self::Counter {
            name: name.into(),
            value,
            labels: HashMap::new(),
        }
    }

    /// Create a gauge.
    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self::Gauge {
            name: name.into(),
            value,
            labels: HashMap::new(),
        }
    }

    /// Add a label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        match &mut self {
            Self::Counter { labels, .. }
            | Self::Gauge { labels, .. }
            | Self::Histogram { labels, .. }
            | Self::Summary { labels, .. } => {
                labels.insert(key.into(), value.into());
            }
        }
        self
    }

    /// Get metric name.
    pub fn name(&self) -> &str {
        match self {
            Self::Counter { name, .. }
            | Self::Gauge { name, .. }
            | Self::Histogram { name, .. }
            | Self::Summary { name, .. } => name,
        }
    }
}

/// Aggregated loop metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoopMetrics {
    /// Total iterations.
    pub total_iterations: u64,
    /// Successful iterations.
    pub successful_iterations: u64,
    /// Failed iterations.
    pub failed_iterations: u64,
    /// Total reboots.
    pub total_reboots: u64,
    /// Total execution time.
    pub total_execution_time: Duration,
    /// Average iteration duration.
    pub avg_iteration_duration: Duration,
    /// Current context usage.
    pub context_usage_percent: u8,
    /// Test metrics.
    pub tests: TestMetrics,
    /// Progress metrics.
    pub progress: ProgressMetrics,
    /// Session metrics.
    pub session: SessionMetrics,
}

/// Test-related metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestMetrics {
    /// Total tests.
    pub total_tests: u32,
    /// Passing tests.
    pub passing_tests: u32,
    /// Failing tests.
    pub failing_tests: u32,
    /// Test pass rate (0-1).
    pub pass_rate: f64,
    /// Current failure streak.
    pub failure_streak: u32,
    /// Flaky test count.
    pub flaky_tests: u32,
}

/// Progress-related metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressMetrics {
    /// Iterations with progress.
    pub iterations_with_progress: u64,
    /// Progress rate (0-1).
    pub progress_rate: f64,
    /// Current no-progress streak.
    pub no_progress_streak: u32,
    /// Progress velocity.
    pub velocity: f64,
    /// Files changed total.
    pub files_changed: u64,
}

/// Session-related metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total sessions created.
    pub sessions_created: u64,
    /// Active sessions.
    pub active_sessions: u32,
    /// Average session duration.
    pub avg_session_duration: Duration,
    /// Context reboot count.
    pub context_reboots: u64,
    /// Average context usage at reboot.
    pub avg_context_at_reboot: f64,
}

/// Configuration for metrics collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection.
    pub enabled: bool,
    /// Metric prefix for namespacing.
    pub prefix: String,
    /// Collection interval.
    #[serde(with = "humantime_serde")]
    pub collection_interval: Duration,
    /// Enable histogram metrics.
    pub enable_histograms: bool,
    /// Histogram buckets for durations.
    pub duration_buckets: Vec<f64>,
    /// Export format.
    pub export_format: MetricsExportFormat,
    /// Export path/endpoint.
    pub export_target: Option<String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "ralph_loop".to_string(),
            collection_interval: Duration::from_secs(10),
            enable_histograms: true,
            duration_buckets: vec![
                0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0,
            ],
            export_format: MetricsExportFormat::Prometheus,
            export_target: None,
        }
    }
}

/// Metrics export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricsExportFormat {
    /// Prometheus text format.
    Prometheus,
    /// JSON format.
    Json,
    /// StatsD format.
    StatsD,
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

### 2. Metrics Collector (src/metrics/collector.rs)

```rust
//! Metrics collection implementation.

use super::types::{
    HistogramBucket, LoopMetrics, Metric, MetricsConfig, MetricsExportFormat,
    ProgressMetrics, SessionMetrics, TestMetrics,
};
use crate::error::LoopResult;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info};

/// Collects and manages loop metrics.
pub struct MetricsCollector {
    /// Configuration.
    config: MetricsConfig,
    /// Counters.
    counters: RwLock<HashMap<String, Arc<AtomicU64>>>,
    /// Gauges.
    gauges: RwLock<HashMap<String, Arc<AtomicU64>>>,
    /// Histograms.
    histograms: RwLock<HashMap<String, HistogramData>>,
    /// Loop start time.
    start_time: Instant,
    /// Metric update broadcast.
    update_tx: broadcast::Sender<LoopMetrics>,
}

/// Internal histogram data.
struct HistogramData {
    buckets: Vec<f64>,
    counts: Vec<AtomicU64>,
    sum: AtomicU64,
    count: AtomicU64,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new(config: MetricsConfig) -> Self {
        let (update_tx, _) = broadcast::channel(64);
        Self {
            config,
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
            update_tx,
        }
    }

    /// Subscribe to metric updates.
    pub fn subscribe(&self) -> broadcast::Receiver<LoopMetrics> {
        self.update_tx.subscribe()
    }

    /// Increment a counter.
    pub async fn increment(&self, name: &str, value: u64) {
        if !self.config.enabled {
            return;
        }

        let full_name = self.prefixed_name(name);
        let mut counters = self.counters.write().await;

        let counter = counters
            .entry(full_name)
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));

        counter.fetch_add(value, Ordering::Relaxed);
    }

    /// Set a gauge value.
    pub async fn set_gauge(&self, name: &str, value: f64) {
        if !self.config.enabled {
            return;
        }

        let full_name = self.prefixed_name(name);
        let mut gauges = self.gauges.write().await;

        let gauge = gauges
            .entry(full_name)
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));

        gauge.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Record a histogram observation.
    pub async fn observe(&self, name: &str, value: f64) {
        if !self.config.enabled || !self.config.enable_histograms {
            return;
        }

        let full_name = self.prefixed_name(name);
        let mut histograms = self.histograms.write().await;

        let histogram = histograms.entry(full_name).or_insert_with(|| {
            let buckets = self.config.duration_buckets.clone();
            let counts = buckets.iter().map(|_| AtomicU64::new(0)).collect();
            HistogramData {
                buckets,
                counts,
                sum: AtomicU64::new(0),
                count: AtomicU64::new(0),
            }
        });

        // Update counts
        histogram.count.fetch_add(1, Ordering::Relaxed);
        histogram
            .sum
            .fetch_add(value.to_bits(), Ordering::Relaxed);

        // Update bucket counts
        for (i, bucket) in histogram.buckets.iter().enumerate() {
            if value <= *bucket {
                histogram.counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Record iteration timing.
    pub async fn record_iteration(&self, duration: Duration, success: bool) {
        self.increment("iterations_total", 1).await;
        self.observe("iteration_duration_seconds", duration.as_secs_f64()).await;

        if success {
            self.increment("iterations_success", 1).await;
        } else {
            self.increment("iterations_failed", 1).await;
        }
    }

    /// Record reboot.
    pub async fn record_reboot(&self, context_usage: u8) {
        self.increment("reboots_total", 1).await;
        self.observe("context_at_reboot_percent", context_usage as f64).await;
    }

    /// Record session creation.
    pub async fn record_session_created(&self) {
        self.increment("sessions_created_total", 1).await;
    }

    /// Record test results.
    pub async fn record_tests(&self, passed: u32, failed: u32, total: u32) {
        self.set_gauge("tests_total", total as f64).await;
        self.set_gauge("tests_passing", passed as f64).await;
        self.set_gauge("tests_failing", failed as f64).await;

        if total > 0 {
            self.set_gauge("test_pass_rate", passed as f64 / total as f64).await;
        }
    }

    /// Record progress.
    pub async fn record_progress(&self, made_progress: bool, score: f64) {
        if made_progress {
            self.increment("iterations_with_progress", 1).await;
        }
        self.set_gauge("progress_score", score).await;
    }

    /// Get current aggregated metrics.
    pub async fn get_metrics(&self) -> LoopMetrics {
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;

        let get_counter = |name: &str| -> u64 {
            counters
                .get(&self.prefixed_name(name))
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0)
        };

        let get_gauge = |name: &str| -> f64 {
            gauges
                .get(&self.prefixed_name(name))
                .map(|g| f64::from_bits(g.load(Ordering::Relaxed)))
                .unwrap_or(0.0)
        };

        let total_iterations = get_counter("iterations_total");
        let successful_iterations = get_counter("iterations_success");
        let elapsed = self.start_time.elapsed();

        let avg_duration = if total_iterations > 0 {
            elapsed / total_iterations as u32
        } else {
            Duration::ZERO
        };

        LoopMetrics {
            total_iterations,
            successful_iterations,
            failed_iterations: get_counter("iterations_failed"),
            total_reboots: get_counter("reboots_total"),
            total_execution_time: elapsed,
            avg_iteration_duration: avg_duration,
            context_usage_percent: get_gauge("context_usage") as u8,
            tests: TestMetrics {
                total_tests: get_gauge("tests_total") as u32,
                passing_tests: get_gauge("tests_passing") as u32,
                failing_tests: get_gauge("tests_failing") as u32,
                pass_rate: get_gauge("test_pass_rate"),
                failure_streak: get_gauge("test_failure_streak") as u32,
                flaky_tests: get_gauge("flaky_tests") as u32,
            },
            progress: ProgressMetrics {
                iterations_with_progress: get_counter("iterations_with_progress"),
                progress_rate: if total_iterations > 0 {
                    get_counter("iterations_with_progress") as f64 / total_iterations as f64
                } else {
                    0.0
                },
                no_progress_streak: get_gauge("no_progress_streak") as u32,
                velocity: get_gauge("progress_velocity"),
                files_changed: get_counter("files_changed"),
            },
            session: SessionMetrics {
                sessions_created: get_counter("sessions_created_total"),
                active_sessions: get_gauge("active_sessions") as u32,
                avg_session_duration: Duration::ZERO, // Would need more tracking
                context_reboots: get_counter("reboots_total"),
                avg_context_at_reboot: 0.0, // Would need histogram analysis
            },
        }
    }

    /// Export metrics in configured format.
    pub async fn export(&self) -> LoopResult<String> {
        match self.config.export_format {
            MetricsExportFormat::Prometheus => self.export_prometheus().await,
            MetricsExportFormat::Json => self.export_json().await,
            MetricsExportFormat::StatsD => self.export_statsd().await,
        }
    }

    /// Export in Prometheus format.
    async fn export_prometheus(&self) -> LoopResult<String> {
        let mut output = String::new();
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;

        // Counters
        for (name, counter) in counters.iter() {
            output.push_str(&format!(
                "# TYPE {} counter\n{} {}\n",
                name,
                name,
                counter.load(Ordering::Relaxed)
            ));
        }

        // Gauges
        for (name, gauge) in gauges.iter() {
            output.push_str(&format!(
                "# TYPE {} gauge\n{} {}\n",
                name,
                name,
                f64::from_bits(gauge.load(Ordering::Relaxed))
            ));
        }

        // Histograms
        for (name, histogram) in histograms.iter() {
            output.push_str(&format!("# TYPE {} histogram\n", name));

            let mut cumulative = 0u64;
            for (i, bucket) in histogram.buckets.iter().enumerate() {
                cumulative += histogram.counts[i].load(Ordering::Relaxed);
                output.push_str(&format!(
                    "{}_bucket{{le=\"{}\"}} {}\n",
                    name, bucket, cumulative
                ));
            }
            output.push_str(&format!(
                "{}_bucket{{le=\"+Inf\"}} {}\n",
                name,
                histogram.count.load(Ordering::Relaxed)
            ));
            output.push_str(&format!(
                "{}_sum {}\n",
                name,
                f64::from_bits(histogram.sum.load(Ordering::Relaxed))
            ));
            output.push_str(&format!(
                "{}_count {}\n",
                name,
                histogram.count.load(Ordering::Relaxed)
            ));
        }

        Ok(output)
    }

    /// Export in JSON format.
    async fn export_json(&self) -> LoopResult<String> {
        let metrics = self.get_metrics().await;
        serde_json::to_string_pretty(&metrics)
            .map_err(|e| crate::error::LoopError::SerializationError { source: e.to_string() })
    }

    /// Export in StatsD format.
    async fn export_statsd(&self) -> LoopResult<String> {
        let mut output = String::new();
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;

        for (name, counter) in counters.iter() {
            output.push_str(&format!(
                "{}:{}|c\n",
                name,
                counter.load(Ordering::Relaxed)
            ));
        }

        for (name, gauge) in gauges.iter() {
            output.push_str(&format!(
                "{}:{}|g\n",
                name,
                f64::from_bits(gauge.load(Ordering::Relaxed))
            ));
        }

        Ok(output)
    }

    /// Get prefixed metric name.
    fn prefixed_name(&self, name: &str) -> String {
        format!("{}_{}", self.config.prefix, name)
    }

    /// Broadcast current metrics.
    pub async fn broadcast(&self) {
        let metrics = self.get_metrics().await;
        let _ = self.update_tx.send(metrics);
    }

    /// Reset all metrics.
    pub async fn reset(&self) {
        self.counters.write().await.clear();
        self.gauges.write().await.clear();
        self.histograms.write().await.clear();
    }
}
```

### 3. Module Root (src/metrics/mod.rs)

```rust
//! Loop metrics collection and export.

pub mod collector;
pub mod types;

pub use collector::MetricsCollector;
pub use types::{
    HistogramBucket, LoopMetrics, Metric, MetricsConfig, MetricsExportFormat,
    ProgressMetrics, Quantile, SessionMetrics, TestMetrics,
};
```

---

## Testing Requirements

1. Counter increments correctly
2. Gauge updates correctly
3. Histogram buckets populate correctly
4. Prometheus export format is valid
5. JSON export is valid JSON
6. Aggregated metrics are accurate
7. Broadcast sends updates
8. Reset clears all metrics

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Depends on: [028-metrics-foundation.md](../phase-01-common/028-metrics-foundation.md)
- Next: [109-loop-state.md](109-loop-state.md)
- Related: [107-no-progress.md](107-no-progress.md)
