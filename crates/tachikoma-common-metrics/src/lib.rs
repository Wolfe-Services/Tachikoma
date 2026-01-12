//! Metrics collection for Tachikoma.
//!
//! This crate provides a metrics collection system for tracking counters, gauges,
//! and histograms with support for labels and Prometheus-compatible export.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use tachikoma_common_metrics as metrics;
//!
//! // Initialize the global registry
//! metrics::init();
//!
//! // Create metrics
//! let counter = metrics::counter("requests_total");
//! let gauge = metrics::gauge("active_connections");
//! let histogram = metrics::histogram("response_time");
//!
//! // Use the metrics
//! counter.inc();
//! gauge.set(42);
//! histogram.observe(0.1);
//! ```
//!
//! ## Using Labels
//!
//! ```rust
//! use tachikoma_common_metrics::labeled::{labels, LabeledCounter};
//!
//! let labels = labels([("method", "GET"), ("status", "200")]);
//! let counter = LabeledCounter::new("http_requests", labels);
//! counter.inc();
//! ```
//!
//! ## Prometheus Export
//!
//! ```rust
//! use tachikoma_common_metrics::MetricsRegistry;
//!
//! let registry = MetricsRegistry::new();
//! let counter = registry.counter("test_counter");
//! counter.inc_by(5);
//!
//! let prometheus_output = registry.export_prometheus();
//! println!("{}", prometheus_output);
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Mutex};

/// Global metrics registry.
static REGISTRY: RwLock<Option<MetricsRegistry>> = RwLock::new(None);

/// Initialize the metrics registry.
pub fn init() {
    let mut guard = REGISTRY.write().unwrap();
    *guard = Some(MetricsRegistry::new());
}

/// Get or create a counter.
pub fn counter(name: &'static str) -> Counter {
    let guard = REGISTRY.read().unwrap();
    if let Some(ref registry) = *guard {
        return registry.counter(name);
    }
    Counter::noop()
}

/// Get or create a gauge.
pub fn gauge(name: &'static str) -> Gauge {
    let guard = REGISTRY.read().unwrap();
    if let Some(ref registry) = *guard {
        return registry.gauge(name);
    }
    Gauge::noop()
}

/// Get or create a histogram.
pub fn histogram(name: &'static str) -> Histogram {
    let guard = REGISTRY.read().unwrap();
    if let Some(ref registry) = *guard {
        return registry.histogram(name);
    }
    Histogram::noop()
}

/// Metrics registry.
pub struct MetricsRegistry {
    counters: RwLock<HashMap<&'static str, Counter>>,
    gauges: RwLock<HashMap<&'static str, Gauge>>,
    histograms: RwLock<HashMap<&'static str, Histogram>>,
}

impl MetricsRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a counter.
    pub fn counter(&self, name: &'static str) -> Counter {
        let mut counters = self.counters.write().unwrap();
        counters
            .entry(name)
            .or_insert_with(|| Counter::new(name))
            .clone()
    }

    /// Get or create a gauge.
    pub fn gauge(&self, name: &'static str) -> Gauge {
        let mut gauges = self.gauges.write().unwrap();
        gauges
            .entry(name)
            .or_insert_with(|| Gauge::new(name))
            .clone()
    }

    /// Get or create a histogram.
    pub fn histogram(&self, name: &'static str) -> Histogram {
        let mut histograms = self.histograms.write().unwrap();
        histograms
            .entry(name)
            .or_insert_with(|| Histogram::new(name))
            .clone()
    }

    /// Export metrics in Prometheus format.
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        for (name, counter) in self.counters.read().unwrap().iter() {
            output.push_str(&format!(
                "# TYPE {} counter\n{} {}\n",
                name,
                name,
                counter.get()
            ));
        }

        for (name, gauge) in self.gauges.read().unwrap().iter() {
            output.push_str(&format!(
                "# TYPE {} gauge\n{} {}\n",
                name,
                name,
                gauge.get()
            ));
        }

        for (name, histogram) in self.histograms.read().unwrap().iter() {
            let (count, sum) = histogram.get_count_and_sum();
            output.push_str(&format!(
                "# TYPE {} histogram\n{}_count {}\n{}_sum {}\n",
                name, name, count, name, sum
            ));
            
            for (le, count) in histogram.get_buckets() {
                output.push_str(&format!("{}{{le=\"{:.1}\"}} {}\n", name, le, count));
            }
            output.push_str(&format!("{}{{le=\"+Inf\"}} {}\n", name, count));
        }

        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A counter metric (monotonically increasing).
#[derive(Clone)]
pub struct Counter {
    name: &'static str,
    value: std::sync::Arc<AtomicU64>,
}

impl Counter {
    /// Create a new counter.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            value: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a no-op counter.
    pub fn noop() -> Self {
        Self::new("noop")
    }

    /// Increment by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment by a value.
    pub fn inc_by(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Get the current value.
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// A gauge metric (can go up and down).
#[derive(Clone)]
pub struct Gauge {
    name: &'static str,
    value: std::sync::Arc<AtomicU64>,
}

impl Gauge {
    /// Create a new gauge.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            value: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a no-op gauge.
    pub fn noop() -> Self {
        Self::new("noop")
    }

    /// Set the value.
    pub fn set(&self, v: u64) {
        self.value.store(v, Ordering::Relaxed);
    }

    /// Increment by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement by 1.
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get the current value.
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// A histogram metric (tracks distribution of values).
#[derive(Clone)]
pub struct Histogram {
    name: &'static str,
    count: Arc<AtomicU64>,
    sum: Arc<AtomicU64>,
    buckets: Arc<Mutex<Vec<(f64, AtomicU64)>>>,
}

impl Histogram {
    /// Default bucket boundaries for histograms.
    const DEFAULT_BUCKETS: &'static [f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

    /// Create a new histogram.
    pub fn new(name: &'static str) -> Self {
        let buckets = Self::DEFAULT_BUCKETS
            .iter()
            .map(|&upper_bound| (upper_bound, AtomicU64::new(0)))
            .collect();
        
        Self {
            name,
            count: Arc::new(AtomicU64::new(0)),
            sum: Arc::new(AtomicU64::new(0)),
            buckets: Arc::new(Mutex::new(buckets)),
        }
    }

    /// Create a no-op histogram.
    pub fn noop() -> Self {
        Self::new("noop")
    }

    /// Observe a value.
    pub fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        // For simplicity, convert to u64 - in production might use f64 with atomic
        self.sum.fetch_add((value * 1000.0) as u64, Ordering::Relaxed); // Store as milliseconds
        
        let buckets = self.buckets.lock().unwrap();
        for (upper_bound, count) in buckets.iter() {
            if value <= *upper_bound {
                count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get count and sum.
    pub fn get_count_and_sum(&self) -> (u64, f64) {
        let count = self.count.load(Ordering::Relaxed);
        let sum = self.sum.load(Ordering::Relaxed) as f64 / 1000.0; // Convert back from milliseconds
        (count, sum)
    }

    /// Get bucket counts.
    pub fn get_buckets(&self) -> Vec<(f64, u64)> {
        let buckets = self.buckets.lock().unwrap();
        buckets.iter()
            .map(|(upper_bound, count)| (*upper_bound, count.load(Ordering::Relaxed)))
            .collect()
    }
}

/// Common metric names.
pub mod names {
    pub const MISSIONS_STARTED: &str = "tachikoma_missions_started_total";
    pub const MISSIONS_COMPLETED: &str = "tachikoma_missions_completed_total";
    pub const MISSIONS_FAILED: &str = "tachikoma_missions_failed_total";
    pub const ACTIVE_MISSIONS: &str = "tachikoma_active_missions";
    pub const TOKENS_USED: &str = "tachikoma_tokens_used_total";
    pub const API_CALLS: &str = "tachikoma_api_calls_total";
    pub const CONTEXT_USAGE: &str = "tachikoma_context_usage_percent";
}

/// Labeled metrics support.
pub mod labeled {
    use super::*;
    use std::collections::BTreeMap;

    /// A label set for metrics.
    pub type Labels = BTreeMap<String, String>;

    /// Create labels from key-value pairs.
    pub fn labels<const N: usize>(pairs: [(&str, &str); N]) -> Labels {
        pairs.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// A counter with labels.
    #[derive(Clone)]
    pub struct LabeledCounter {
        name: String,
        labels: Labels,
        counter: Counter,
    }

    impl LabeledCounter {
        /// Create a new labeled counter.
        pub fn new(name: &str, labels: Labels) -> Self {
            let metric_name = format!("{}:{}", name, format_labels(&labels));
            Self {
                name: name.to_string(),
                labels,
                counter: Counter::new(Box::leak(metric_name.into_boxed_str())),
            }
        }

        /// Increment by 1.
        pub fn inc(&self) {
            self.counter.inc();
        }

        /// Increment by a value.
        pub fn inc_by(&self, n: u64) {
            self.counter.inc_by(n);
        }

        /// Get the current value.
        pub fn get(&self) -> u64 {
            self.counter.get()
        }

        /// Get the labels.
        pub fn labels(&self) -> &Labels {
            &self.labels
        }

        /// Get the metric name.
        pub fn name(&self) -> &str {
            &self.name
        }
    }

    /// A gauge with labels.
    #[derive(Clone)]
    pub struct LabeledGauge {
        name: String,
        labels: Labels,
        gauge: Gauge,
    }

    impl LabeledGauge {
        /// Create a new labeled gauge.
        pub fn new(name: &str, labels: Labels) -> Self {
            let metric_name = format!("{}:{}", name, format_labels(&labels));
            Self {
                name: name.to_string(),
                labels,
                gauge: Gauge::new(Box::leak(metric_name.into_boxed_str())),
            }
        }

        /// Set the value.
        pub fn set(&self, v: u64) {
            self.gauge.set(v);
        }

        /// Increment by 1.
        pub fn inc(&self) {
            self.gauge.inc();
        }

        /// Decrement by 1.
        pub fn dec(&self) {
            self.gauge.dec();
        }

        /// Get the current value.
        pub fn get(&self) -> u64 {
            self.gauge.get()
        }

        /// Get the labels.
        pub fn labels(&self) -> &Labels {
            &self.labels
        }

        /// Get the metric name.
        pub fn name(&self) -> &str {
            &self.name
        }
    }

    /// Format labels for metric name.
    fn format_labels(labels: &Labels) -> String {
        if labels.is_empty() {
            return String::new();
        }
        
        let mut formatted = labels.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");
        
        formatted.insert(0, '{');
        formatted.push('}');
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::labeled::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test");
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        counter.inc_by(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test");
        gauge.set(10);
        assert_eq!(gauge.get(), 10);
        gauge.inc();
        assert_eq!(gauge.get(), 11);
        gauge.dec();
        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new("test_histogram");
        
        // Observe some values
        histogram.observe(0.5);
        histogram.observe(1.5);
        histogram.observe(2.0);
        
        let (count, sum) = histogram.get_count_and_sum();
        assert_eq!(count, 3);
        assert!((sum - 4.0).abs() < 0.1); // Allow for floating point precision
        
        let buckets = histogram.get_buckets();
        assert!(!buckets.is_empty());
    }

    #[test]
    fn test_labeled_counter() {
        let labels = labels([("method", "GET"), ("status", "200")]);
        let counter = LabeledCounter::new("http_requests", labels);
        
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        assert_eq!(counter.name(), "http_requests");
        assert_eq!(counter.labels().get("method"), Some(&"GET".to_string()));
    }

    #[test]
    fn test_labeled_gauge() {
        let labels = labels([("service", "api")]);
        let gauge = LabeledGauge::new("active_connections", labels);
        
        gauge.set(5);
        assert_eq!(gauge.get(), 5);
        gauge.inc();
        assert_eq!(gauge.get(), 6);
        gauge.dec();
        assert_eq!(gauge.get(), 5);
    }

    #[test]
    fn test_registry_prometheus_export() {
        let registry = MetricsRegistry::new();
        
        // Create some metrics
        let counter = registry.counter("test_counter");
        let gauge = registry.gauge("test_gauge");
        let histogram = registry.histogram("test_histogram");
        
        counter.inc_by(5);
        gauge.set(10);
        histogram.observe(1.0);
        
        let export = registry.export_prometheus();
        
        // Check that all metric types are present
        assert!(export.contains("# TYPE test_counter counter"));
        assert!(export.contains("test_counter 5"));
        assert!(export.contains("# TYPE test_gauge gauge"));
        assert!(export.contains("test_gauge 10"));
        assert!(export.contains("# TYPE test_histogram histogram"));
        assert!(export.contains("test_histogram_count 1"));
    }

    #[test]
    fn test_prometheus_format() {
        let registry = MetricsRegistry::new();
        
        let counter = registry.counter("http_requests_total");
        let gauge = registry.gauge("memory_usage");
        let histogram = registry.histogram("response_time");
        
        counter.inc_by(100);
        gauge.set(1024);
        histogram.observe(0.1);
        histogram.observe(0.5);
        
        let export = registry.export_prometheus();
        
        // Verify Prometheus format compliance
        let lines: Vec<&str> = export.lines().collect();
        
        // Check counter format
        assert!(lines.iter().any(|line| line.starts_with("# TYPE http_requests_total counter")));
        assert!(lines.iter().any(|line| *line == "http_requests_total 100"));
        
        // Check gauge format
        assert!(lines.iter().any(|line| line.starts_with("# TYPE memory_usage gauge")));
        assert!(lines.iter().any(|line| *line == "memory_usage 1024"));
        
        // Check histogram format
        assert!(lines.iter().any(|line| line.starts_with("# TYPE response_time histogram")));
        assert!(lines.iter().any(|line| *line == "response_time_count 2"));
        assert!(lines.iter().any(|line| line.starts_with("response_time_sum")));
        assert!(lines.iter().any(|line| line.contains("le=\"")));
    }

    #[test] 
    fn test_global_registry() {
        init();
        
        let counter = counter("global_test");
        let gauge = gauge("global_gauge");
        let histogram = histogram("global_histogram");
        
        counter.inc();
        gauge.set(42);
        histogram.observe(0.1);
        
        assert_eq!(counter.get(), 1);
        assert_eq!(gauge.get(), 42);
        
        let (count, _) = histogram.get_count_and_sum();
        assert_eq!(count, 1);
    }
}