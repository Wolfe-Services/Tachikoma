# 028 - Metrics Foundation

**Phase:** 1 - Core Common Crates
**Spec ID:** 028
**Status:** Planned
**Dependencies:** 011-common-core-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Set up a metrics collection system for tracking counters, gauges, and histograms for operational visibility.

---

## Acceptance Criteria

- [ ] Counter metrics
- [ ] Gauge metrics
- [ ] Histogram metrics
- [ ] Labels/tags support
- [ ] Prometheus-compatible export

---

## Implementation Details

### 1. Metrics Module (crates/tachikoma-common-metrics/src/lib.rs)

```rust
//! Metrics collection for Tachikoma.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

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

/// Metrics registry.
pub struct MetricsRegistry {
    counters: RwLock<HashMap<&'static str, Counter>>,
    gauges: RwLock<HashMap<&'static str, Gauge>>,
}

impl MetricsRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
```

### 2. Crate Setup

```toml
[package]
name = "tachikoma-common-metrics"
version.workspace = true
edition.workspace = true

[dependencies]
# Minimal dependencies
```

---

## Testing Requirements

1. Counters increment correctly
2. Gauges can increase and decrease
3. Prometheus export is valid format
4. Thread-safe access works

---

## Related Specs

- Depends on: [011-common-core-types.md](011-common-core-types.md)
- Next: [029-file-system-utilities.md](029-file-system-utilities.md)
