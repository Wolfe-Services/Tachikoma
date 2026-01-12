# Spec 418: Performance Metrics

## Phase
19 - Analytics/Telemetry

## Spec ID
418

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 408: Analytics Collector (event collection)

## Estimated Context
~10%

---

## Objective

Implement comprehensive performance monitoring and metrics collection for Tachikoma, tracking latencies, throughput, resource usage, and system health to enable optimization and troubleshooting.

---

## Acceptance Criteria

- [ ] Track request/response latencies
- [ ] Monitor memory and CPU usage
- [ ] Measure operation throughput
- [ ] Implement histogram metrics
- [ ] Support custom performance counters
- [ ] Create real-time performance dashboards data
- [ ] Enable performance anomaly detection
- [ ] Provide optimization recommendations

---

## Implementation Details

### Performance Metrics

```rust
// src/analytics/performance.rs

use crate::analytics::collector::EventCollector;
use crate::analytics::types::{
    EventBuilder, EventData, EventType, PerformanceEventData,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration as StdDuration};
use tokio::sync::RwLock;

/// Metric type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// Timing/latency metric
    Timer,
    /// Counter metric
    Counter,
    /// Gauge metric (current value)
    Gauge,
    /// Histogram metric
    Histogram,
}

/// A single metric measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Tags for dimensions
    pub tags: HashMap<String, String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Timer for measuring operation duration
pub struct Timer {
    name: String,
    start: Instant,
    tags: HashMap<String, String>,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            tags: HashMap::new(),
        }
    }

    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    pub fn finish(self) -> Measurement {
        Measurement {
            name: self.name,
            metric_type: MetricType::Timer,
            value: self.elapsed_ms(),
            unit: "ms".to_string(),
            tags: self.tags,
            timestamp: Utc::now(),
        }
    }
}

/// Histogram for tracking value distributions
#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    buckets: Vec<f64>,
    counts: Vec<AtomicU64>,
    sum: AtomicU64,
    count: AtomicU64,
    min: AtomicU64,
    max: AtomicU64,
}

impl Histogram {
    pub fn new(name: &str, buckets: Vec<f64>) -> Self {
        let counts = buckets.iter().map(|_| AtomicU64::new(0)).collect();

        Self {
            name: name.to_string(),
            buckets,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    pub fn observe(&self, value: f64) {
        // Update counts
        for (i, bucket) in self.buckets.iter().enumerate() {
            if value <= *bucket {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }

        // Update sum and count
        self.sum.fetch_add(value.to_bits(), Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);

        // Update min/max
        let value_bits = value.to_bits();
        let mut current_min = self.min.load(Ordering::Relaxed);
        while value_bits < current_min {
            match self.min.compare_exchange(
                current_min,
                value_bits,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(c) => current_min = c,
            }
        }

        let mut current_max = self.max.load(Ordering::Relaxed);
        while value_bits > current_max {
            match self.max.compare_exchange(
                current_max,
                value_bits,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(c) => current_max = c,
            }
        }
    }

    pub fn snapshot(&self) -> HistogramSnapshot {
        let counts: Vec<u64> = self
            .counts
            .iter()
            .map(|c| c.load(Ordering::Relaxed))
            .collect();

        let count = self.count.load(Ordering::Relaxed);
        let sum = f64::from_bits(self.sum.load(Ordering::Relaxed));
        let min = f64::from_bits(self.min.load(Ordering::Relaxed));
        let max = f64::from_bits(self.max.load(Ordering::Relaxed));

        HistogramSnapshot {
            name: self.name.clone(),
            buckets: self.buckets.clone(),
            counts,
            sum,
            count,
            min: if count > 0 { min } else { 0.0 },
            max: if count > 0 { max } else { 0.0 },
            mean: if count > 0 { sum / count as f64 } else { 0.0 },
        }
    }
}

/// Snapshot of histogram data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSnapshot {
    pub name: String,
    pub buckets: Vec<f64>,
    pub counts: Vec<u64>,
    pub sum: f64,
    pub count: u64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

impl HistogramSnapshot {
    pub fn percentile(&self, p: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let target = (p * self.count as f64) as u64;
        for (i, count) in self.counts.iter().enumerate() {
            if *count >= target {
                return self.buckets[i];
            }
        }

        *self.buckets.last().unwrap_or(&0.0)
    }
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Memory usage in bytes
    pub memory_used_bytes: u64,
    /// Memory total in bytes
    pub memory_total_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Disk usage in bytes
    pub disk_used_bytes: u64,
    /// Disk total in bytes
    pub disk_total_bytes: u64,
    /// Number of active threads
    pub thread_count: u32,
    /// Open file descriptors
    pub fd_count: u32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ResourceMetrics {
    pub fn memory_utilization(&self) -> f64 {
        if self.memory_total_bytes == 0 {
            return 0.0;
        }
        self.memory_used_bytes as f64 / self.memory_total_bytes as f64
    }

    pub fn disk_utilization(&self) -> f64 {
        if self.disk_total_bytes == 0 {
            return 0.0;
        }
        self.disk_used_bytes as f64 / self.disk_total_bytes as f64
    }
}

/// Latency statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyStats {
    pub count: u64,
    pub mean_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub stddev_ms: f64,
}

/// Throughput statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThroughputStats {
    /// Requests per second
    pub requests_per_second: f64,
    /// Tokens per second
    pub tokens_per_second: f64,
    /// Bytes per second
    pub bytes_per_second: f64,
    /// Window size in seconds
    pub window_seconds: u64,
}

/// Performance metrics collector
pub struct PerformanceCollector {
    /// Event collector
    collector: Arc<EventCollector>,
    /// Registered histograms
    histograms: Arc<RwLock<HashMap<String, Arc<Histogram>>>>,
    /// Counter metrics
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// Gauge metrics
    gauges: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// Recent measurements for aggregation
    measurements: Arc<RwLock<Vec<Measurement>>>,
    /// Resource metrics history
    resource_history: Arc<RwLock<Vec<ResourceMetrics>>>,
}

impl PerformanceCollector {
    pub fn new(collector: Arc<EventCollector>) -> Self {
        let perf = Self {
            collector,
            histograms: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            measurements: Arc::new(RwLock::new(Vec::new())),
            resource_history: Arc::new(RwLock::new(Vec::new())),
        };

        // Register default histograms
        perf
    }

    /// Start a new timer
    pub fn start_timer(&self, name: &str) -> Timer {
        Timer::new(name)
    }

    /// Record a timing measurement
    pub async fn record_timing(
        &self,
        name: &str,
        duration_ms: f64,
        tags: HashMap<String, String>,
    ) {
        let measurement = Measurement {
            name: name.to_string(),
            metric_type: MetricType::Timer,
            value: duration_ms,
            unit: "ms".to_string(),
            tags: tags.clone(),
            timestamp: Utc::now(),
        };

        // Store measurement
        {
            let mut measurements = self.measurements.write().await;
            measurements.push(measurement);

            // Trim old measurements (keep last hour)
            let cutoff = Utc::now() - Duration::hours(1);
            measurements.retain(|m| m.timestamp >= cutoff);
        }

        // Update histogram if registered
        if let Some(histogram) = self.histograms.read().await.get(name) {
            histogram.observe(duration_ms);
        }

        // Emit event
        let event = EventBuilder::new(EventType::ResponseLatency)
            .data(EventData::Performance(PerformanceEventData {
                metric: name.to_string(),
                value: duration_ms,
                unit: "ms".to_string(),
                tags,
            }))
            .build();

        self.collector.collect(event).await.ok();
    }

    /// Increment a counter
    pub async fn increment(&self, name: &str, delta: u64) {
        let mut counters = self.counters.write().await;
        let counter = counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(delta, Ordering::Relaxed);
    }

    /// Set a gauge value
    pub async fn gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), AtomicU64::new(value.to_bits()));

        // Emit event
        let event = EventBuilder::new(EventType::MemoryUsage)
            .data(EventData::Performance(PerformanceEventData {
                metric: name.to_string(),
                value,
                unit: "".to_string(),
                tags: HashMap::new(),
            }))
            .build();

        self.collector.collect(event).await.ok();
    }

    /// Register a histogram
    pub async fn register_histogram(&self, name: &str, buckets: Vec<f64>) {
        let histogram = Arc::new(Histogram::new(name, buckets));
        let mut histograms = self.histograms.write().await;
        histograms.insert(name.to_string(), histogram);
    }

    /// Record to a histogram
    pub async fn observe(&self, name: &str, value: f64) {
        if let Some(histogram) = self.histograms.read().await.get(name) {
            histogram.observe(value);
        }
    }

    /// Get histogram snapshot
    pub async fn get_histogram(&self, name: &str) -> Option<HistogramSnapshot> {
        self.histograms
            .read()
            .await
            .get(name)
            .map(|h| h.snapshot())
    }

    /// Get counter value
    pub async fn get_counter(&self, name: &str) -> u64 {
        self.counters
            .read()
            .await
            .get(name)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get gauge value
    pub async fn get_gauge(&self, name: &str) -> f64 {
        self.gauges
            .read()
            .await
            .get(name)
            .map(|g| f64::from_bits(g.load(Ordering::Relaxed)))
            .unwrap_or(0.0)
    }

    /// Record resource metrics
    pub async fn record_resources(&self, metrics: ResourceMetrics) {
        let mut history = self.resource_history.write().await;
        history.push(metrics.clone());

        // Keep last hour
        let cutoff = Utc::now() - Duration::hours(1);
        history.retain(|m| m.timestamp >= cutoff);

        // Emit events
        let event = EventBuilder::new(EventType::MemoryUsage)
            .data(EventData::Performance(PerformanceEventData {
                metric: "memory_used".to_string(),
                value: metrics.memory_used_bytes as f64,
                unit: "bytes".to_string(),
                tags: HashMap::new(),
            }))
            .build();

        self.collector.collect(event).await.ok();

        let event = EventBuilder::new(EventType::CpuUsage)
            .data(EventData::Performance(PerformanceEventData {
                metric: "cpu_usage".to_string(),
                value: metrics.cpu_usage_percent,
                unit: "percent".to_string(),
                tags: HashMap::new(),
            }))
            .build();

        self.collector.collect(event).await.ok();
    }

    /// Get latency stats for a metric
    pub async fn get_latency_stats(&self, name: &str) -> LatencyStats {
        let measurements = self.measurements.read().await;

        let timings: Vec<f64> = measurements
            .iter()
            .filter(|m| m.name == name && m.metric_type == MetricType::Timer)
            .map(|m| m.value)
            .collect();

        if timings.is_empty() {
            return LatencyStats::default();
        }

        let mut sorted = timings.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;
        let min = sorted[0];
        let max = sorted[count - 1];

        let variance: f64 = sorted
            .iter()
            .map(|v| (*v - mean).powi(2))
            .sum::<f64>()
            / count as f64;
        let stddev = variance.sqrt();

        let p = |pct: f64| sorted[(count as f64 * pct) as usize].min(max);

        LatencyStats {
            count: count as u64,
            mean_ms: mean,
            min_ms: min,
            max_ms: max,
            p50_ms: p(0.5),
            p90_ms: p(0.9),
            p95_ms: p(0.95),
            p99_ms: p(0.99).min(max),
            stddev_ms: stddev,
        }
    }

    /// Calculate throughput stats
    pub async fn get_throughput_stats(&self, window_seconds: u64) -> ThroughputStats {
        let measurements = self.measurements.read().await;
        let cutoff = Utc::now() - Duration::seconds(window_seconds as i64);

        let recent: Vec<_> = measurements
            .iter()
            .filter(|m| m.timestamp >= cutoff)
            .collect();

        let request_count = recent.len();

        ThroughputStats {
            requests_per_second: request_count as f64 / window_seconds as f64,
            tokens_per_second: 0.0, // Would need token data
            bytes_per_second: 0.0,  // Would need byte data
            window_seconds,
        }
    }

    /// Get all metrics summary
    pub async fn get_summary(&self) -> PerformanceSummary {
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;
        let resources = self.resource_history.read().await;

        let counter_values: HashMap<String, u64> = counters
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();

        let gauge_values: HashMap<String, f64> = gauges
            .iter()
            .map(|(k, v)| (k.clone(), f64::from_bits(v.load(Ordering::Relaxed))))
            .collect();

        let histogram_snapshots: HashMap<String, HistogramSnapshot> = histograms
            .iter()
            .map(|(k, v)| (k.clone(), v.snapshot()))
            .collect();

        let latest_resources = resources.last().cloned();

        PerformanceSummary {
            counters: counter_values,
            gauges: gauge_values,
            histograms: histogram_snapshots,
            resources: latest_resources,
            collected_at: Utc::now(),
        }
    }

    /// Detect anomalies in latency
    pub async fn detect_anomalies(&self, name: &str) -> Vec<PerformanceAnomaly> {
        let stats = self.get_latency_stats(name).await;
        let measurements = self.measurements.read().await;
        let mut anomalies = Vec::new();

        let threshold = stats.mean_ms + (3.0 * stats.stddev_ms);

        for measurement in measurements.iter() {
            if measurement.name == name && measurement.value > threshold {
                anomalies.push(PerformanceAnomaly {
                    metric_name: name.to_string(),
                    value: measurement.value,
                    threshold,
                    anomaly_type: AnomalyType::HighLatency,
                    timestamp: measurement.timestamp,
                    severity: if measurement.value > threshold * 2.0 {
                        AnomalySeverity::Critical
                    } else {
                        AnomalySeverity::Warning
                    },
                });
            }
        }

        anomalies
    }
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, HistogramSnapshot>,
    pub resources: Option<ResourceMetrics>,
    pub collected_at: DateTime<Utc>,
}

/// Performance anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnomaly {
    pub metric_name: String,
    pub value: f64,
    pub threshold: f64,
    pub anomaly_type: AnomalyType,
    pub timestamp: DateTime<Utc>,
    pub severity: AnomalySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    HighLatency,
    HighMemory,
    HighCpu,
    HighErrorRate,
    LowThroughput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::collector::EventCollector;
    use crate::analytics::config::AnalyticsConfigManager;

    async fn create_collector() -> PerformanceCollector {
        let config = AnalyticsConfigManager::new();
        let collector = Arc::new(EventCollector::new(config));
        PerformanceCollector::new(collector)
    }

    #[tokio::test]
    async fn test_timer() {
        let timer = Timer::new("test_operation")
            .with_tag("type", "unit_test");

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));

        let measurement = timer.finish();

        assert_eq!(measurement.name, "test_operation");
        assert!(measurement.value >= 10.0);
        assert_eq!(measurement.tags.get("type"), Some(&"unit_test".to_string()));
    }

    #[tokio::test]
    async fn test_histogram() {
        let histogram = Histogram::new("latency", vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0]);

        histogram.observe(2.0);
        histogram.observe(8.0);
        histogram.observe(15.0);
        histogram.observe(75.0);
        histogram.observe(200.0);

        let snapshot = histogram.snapshot();

        assert_eq!(snapshot.count, 5);
        assert_eq!(snapshot.min, 2.0);
        assert_eq!(snapshot.max, 200.0);
    }

    #[tokio::test]
    async fn test_latency_stats() {
        let collector = create_collector().await;

        // Record various latencies
        for latency in [10.0, 20.0, 30.0, 40.0, 50.0, 100.0, 200.0] {
            collector
                .record_timing("test_latency", latency, HashMap::new())
                .await;
        }

        let stats = collector.get_latency_stats("test_latency").await;

        assert_eq!(stats.count, 7);
        assert_eq!(stats.min_ms, 10.0);
        assert_eq!(stats.max_ms, 200.0);
    }

    #[tokio::test]
    async fn test_counter_and_gauge() {
        let collector = create_collector().await;

        collector.increment("requests", 1).await;
        collector.increment("requests", 2).await;

        assert_eq!(collector.get_counter("requests").await, 3);

        collector.gauge("active_connections", 42.0).await;
        assert_eq!(collector.get_gauge("active_connections").await, 42.0);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let collector = create_collector().await;

        // Record normal latencies
        for _ in 0..20 {
            collector
                .record_timing("test_latency", 50.0, HashMap::new())
                .await;
        }

        // Record an anomalous latency
        collector
            .record_timing("test_latency", 500.0, HashMap::new())
            .await;

        let anomalies = collector.detect_anomalies("test_latency").await;

        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.value == 500.0));
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Timer accuracy
   - Histogram bucket distribution
   - Counter and gauge operations
   - Stats calculations

2. **Integration Tests**
   - Full metrics pipeline
   - Event emission
   - Resource tracking

3. **Performance Tests**
   - High-frequency measurements
   - Memory efficiency

---

## Related Specs

- Spec 406: Analytics Types
- Spec 408: Analytics Collector
- Spec 415: Backend Analytics
- Spec 419: Error Tracking
