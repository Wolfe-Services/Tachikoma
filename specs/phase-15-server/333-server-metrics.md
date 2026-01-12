# Spec 333: Prometheus Metrics

## Phase
15 - Server/API Layer

## Spec ID
333

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 322: Health Checks

## Estimated Context
~9%

---

## Objective

Implement Prometheus metrics collection and exposition for the Tachikoma server, providing comprehensive observability into request performance, resource utilization, and business metrics.

---

## Acceptance Criteria

- [ ] Standard HTTP metrics (requests, latency, errors)
- [ ] Custom business metrics (executions, specs, etc.)
- [ ] Prometheus /metrics endpoint
- [ ] Configurable metric labels
- [ ] Histogram buckets for latency
- [ ] Metric middleware integration
- [ ] Metric documentation

---

## Implementation Details

### Metrics Configuration

```rust
// src/server/metrics/config.rs
use serde::{Deserialize, Serialize};

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics endpoint path
    pub path: String,

    /// Include default process metrics
    pub include_process_metrics: bool,

    /// Custom labels to add to all metrics
    pub global_labels: Vec<(String, String)>,

    /// Histogram buckets for request latency (in seconds)
    pub latency_buckets: Vec<f64>,

    /// Histogram buckets for request size (in bytes)
    pub size_buckets: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/metrics".to_string(),
            include_process_metrics: true,
            global_labels: vec![],
            latency_buckets: vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
            size_buckets: vec![
                100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
            ],
        }
    }
}
```

### Metrics Registry

```rust
// src/server/metrics/registry.rs
use prometheus::{
    self, Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    Opts, Registry, TextEncoder, Encoder,
};
use once_cell::sync::Lazy;

use super::config::MetricsConfig;

/// Global metrics registry
pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

/// All application metrics
pub struct Metrics {
    pub registry: Registry,

    // HTTP metrics
    pub http_requests_total: CounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_request_size_bytes: HistogramVec,
    pub http_response_size_bytes: HistogramVec,
    pub http_requests_in_flight: Gauge,

    // WebSocket metrics
    pub ws_connections_total: Counter,
    pub ws_connections_current: Gauge,
    pub ws_messages_received_total: CounterVec,
    pub ws_messages_sent_total: CounterVec,

    // Execution metrics
    pub executions_total: CounterVec,
    pub execution_duration_seconds: HistogramVec,
    pub execution_tokens_total: CounterVec,
    pub executions_in_progress: Gauge,

    // Storage metrics
    pub db_queries_total: CounterVec,
    pub db_query_duration_seconds: HistogramVec,
    pub db_connections_current: Gauge,

    // Cache metrics
    pub cache_hits_total: Counter,
    pub cache_misses_total: Counter,
    pub cache_size: Gauge,

    // Backend metrics
    pub backend_requests_total: CounterVec,
    pub backend_request_duration_seconds: HistogramVec,
    pub backend_errors_total: CounterVec,

    // Business metrics
    pub missions_total: Gauge,
    pub specs_total: GaugeVec,
    pub file_changes_total: CounterVec,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        // HTTP metrics
        let http_requests_total = CounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "path", "status"],
        ).unwrap();

        let http_request_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request latency in seconds",
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "path"],
        ).unwrap();

        let http_request_size_bytes = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "http_request_size_bytes",
                "HTTP request size in bytes",
            ).buckets(vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0]),
            &["method", "path"],
        ).unwrap();

        let http_response_size_bytes = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "http_response_size_bytes",
                "HTTP response size in bytes",
            ).buckets(vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0]),
            &["method", "path"],
        ).unwrap();

        let http_requests_in_flight = Gauge::new(
            "http_requests_in_flight",
            "Current number of HTTP requests being processed",
        ).unwrap();

        // WebSocket metrics
        let ws_connections_total = Counter::new(
            "ws_connections_total",
            "Total number of WebSocket connections",
        ).unwrap();

        let ws_connections_current = Gauge::new(
            "ws_connections_current",
            "Current number of active WebSocket connections",
        ).unwrap();

        let ws_messages_received_total = CounterVec::new(
            Opts::new("ws_messages_received_total", "Total WebSocket messages received"),
            &["message_type"],
        ).unwrap();

        let ws_messages_sent_total = CounterVec::new(
            Opts::new("ws_messages_sent_total", "Total WebSocket messages sent"),
            &["message_type"],
        ).unwrap();

        // Execution metrics
        let executions_total = CounterVec::new(
            Opts::new("executions_total", "Total number of LLM executions"),
            &["backend", "model", "status"],
        ).unwrap();

        let execution_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "execution_duration_seconds",
                "LLM execution duration in seconds",
            ).buckets(vec![0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0]),
            &["backend", "model"],
        ).unwrap();

        let execution_tokens_total = CounterVec::new(
            Opts::new("execution_tokens_total", "Total tokens used"),
            &["backend", "model", "type"], // type: prompt, completion
        ).unwrap();

        let executions_in_progress = Gauge::new(
            "executions_in_progress",
            "Current number of executions in progress",
        ).unwrap();

        // Storage metrics
        let db_queries_total = CounterVec::new(
            Opts::new("db_queries_total", "Total database queries"),
            &["operation", "table"],
        ).unwrap();

        let db_query_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "db_query_duration_seconds",
                "Database query duration in seconds",
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["operation"],
        ).unwrap();

        let db_connections_current = Gauge::new(
            "db_connections_current",
            "Current number of database connections",
        ).unwrap();

        // Cache metrics
        let cache_hits_total = Counter::new("cache_hits_total", "Total cache hits").unwrap();
        let cache_misses_total = Counter::new("cache_misses_total", "Total cache misses").unwrap();
        let cache_size = Gauge::new("cache_size", "Current cache size").unwrap();

        // Backend metrics
        let backend_requests_total = CounterVec::new(
            Opts::new("backend_requests_total", "Total backend API requests"),
            &["backend", "endpoint"],
        ).unwrap();

        let backend_request_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "backend_request_duration_seconds",
                "Backend API request duration",
            ).buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0]),
            &["backend"],
        ).unwrap();

        let backend_errors_total = CounterVec::new(
            Opts::new("backend_errors_total", "Total backend API errors"),
            &["backend", "error_type"],
        ).unwrap();

        // Business metrics
        let missions_total = Gauge::new("missions_total", "Total number of missions").unwrap();

        let specs_total = GaugeVec::new(
            Opts::new("specs_total", "Total number of specs"),
            &["status"],
        ).unwrap();

        let file_changes_total = CounterVec::new(
            Opts::new("file_changes_total", "Total file changes"),
            &["change_type", "status"],
        ).unwrap();

        // Register all metrics
        let metrics = Self {
            registry: registry.clone(),
            http_requests_total: http_requests_total.clone(),
            http_request_duration_seconds: http_request_duration_seconds.clone(),
            http_request_size_bytes: http_request_size_bytes.clone(),
            http_response_size_bytes: http_response_size_bytes.clone(),
            http_requests_in_flight: http_requests_in_flight.clone(),
            ws_connections_total: ws_connections_total.clone(),
            ws_connections_current: ws_connections_current.clone(),
            ws_messages_received_total: ws_messages_received_total.clone(),
            ws_messages_sent_total: ws_messages_sent_total.clone(),
            executions_total: executions_total.clone(),
            execution_duration_seconds: execution_duration_seconds.clone(),
            execution_tokens_total: execution_tokens_total.clone(),
            executions_in_progress: executions_in_progress.clone(),
            db_queries_total: db_queries_total.clone(),
            db_query_duration_seconds: db_query_duration_seconds.clone(),
            db_connections_current: db_connections_current.clone(),
            cache_hits_total: cache_hits_total.clone(),
            cache_misses_total: cache_misses_total.clone(),
            cache_size: cache_size.clone(),
            backend_requests_total: backend_requests_total.clone(),
            backend_request_duration_seconds: backend_request_duration_seconds.clone(),
            backend_errors_total: backend_errors_total.clone(),
            missions_total: missions_total.clone(),
            specs_total: specs_total.clone(),
            file_changes_total: file_changes_total.clone(),
        };

        // Register with registry
        registry.register(Box::new(http_requests_total)).unwrap();
        registry.register(Box::new(http_request_duration_seconds)).unwrap();
        registry.register(Box::new(http_request_size_bytes)).unwrap();
        registry.register(Box::new(http_response_size_bytes)).unwrap();
        registry.register(Box::new(http_requests_in_flight)).unwrap();
        registry.register(Box::new(ws_connections_total)).unwrap();
        registry.register(Box::new(ws_connections_current)).unwrap();
        registry.register(Box::new(ws_messages_received_total)).unwrap();
        registry.register(Box::new(ws_messages_sent_total)).unwrap();
        registry.register(Box::new(executions_total)).unwrap();
        registry.register(Box::new(execution_duration_seconds)).unwrap();
        registry.register(Box::new(execution_tokens_total)).unwrap();
        registry.register(Box::new(executions_in_progress)).unwrap();
        registry.register(Box::new(db_queries_total)).unwrap();
        registry.register(Box::new(db_query_duration_seconds)).unwrap();
        registry.register(Box::new(db_connections_current)).unwrap();
        registry.register(Box::new(cache_hits_total)).unwrap();
        registry.register(Box::new(cache_misses_total)).unwrap();
        registry.register(Box::new(cache_size)).unwrap();
        registry.register(Box::new(backend_requests_total)).unwrap();
        registry.register(Box::new(backend_request_duration_seconds)).unwrap();
        registry.register(Box::new(backend_errors_total)).unwrap();
        registry.register(Box::new(missions_total)).unwrap();
        registry.register(Box::new(specs_total)).unwrap();
        registry.register(Box::new(file_changes_total)).unwrap();

        metrics
    }

    /// Encode metrics to Prometheus format
    pub fn encode(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
```

### Metrics Middleware

```rust
// src/server/middleware/metrics.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

use crate::server::metrics::METRICS;

/// Middleware for collecting HTTP metrics
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = normalize_path(request.uri().path());

    // Track request size
    if let Some(content_length) = request.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
    {
        METRICS.http_request_size_bytes
            .with_label_values(&[&method, &path])
            .observe(content_length);
    }

    // Track in-flight requests
    METRICS.http_requests_in_flight.inc();

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    METRICS.http_requests_in_flight.dec();

    let status = response.status().as_u16().to_string();

    // Record metrics
    METRICS.http_requests_total
        .with_label_values(&[&method, &path, &status])
        .inc();

    METRICS.http_request_duration_seconds
        .with_label_values(&[&method, &path])
        .observe(duration.as_secs_f64());

    // Track response size
    if let Some(content_length) = response.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
    {
        METRICS.http_response_size_bytes
            .with_label_values(&[&method, &path])
            .observe(content_length);
    }

    response
}

/// Normalize path to reduce cardinality (replace UUIDs with placeholder)
fn normalize_path(path: &str) -> String {
    let uuid_regex = regex::Regex::new(
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
    ).unwrap();

    uuid_regex.replace_all(path, ":id").to_string()
}
```

### Metrics Handler

```rust
// src/server/handlers/metrics.rs
use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::server::metrics::METRICS;

/// Handler for /metrics endpoint
pub async fn metrics_handler() -> impl IntoResponse {
    let metrics = METRICS.encode();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/plain; version=0.0.4; charset=utf-8".parse().unwrap(),
    );

    (StatusCode::OK, headers, metrics)
}
```

### Metric Helpers

```rust
// src/server/metrics/helpers.rs
use std::time::Instant;

use super::METRICS;

/// Timer guard for measuring duration
pub struct Timer {
    start: Instant,
    metric_fn: Box<dyn FnOnce(f64) + Send>,
}

impl Timer {
    pub fn new<F>(metric_fn: F) -> Self
    where
        F: FnOnce(f64) + Send + 'static,
    {
        Self {
            start: Instant::now(),
            metric_fn: Box::new(metric_fn),
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        let duration = self.start.elapsed().as_secs_f64();
        (self.metric_fn)(duration);
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Timer auto-records on drop if stop() wasn't called
    }
}

/// Record execution metrics
pub fn record_execution(backend: &str, model: &str, status: &str, duration_secs: f64, tokens: (u32, u32)) {
    METRICS.executions_total
        .with_label_values(&[backend, model, status])
        .inc();

    METRICS.execution_duration_seconds
        .with_label_values(&[backend, model])
        .observe(duration_secs);

    METRICS.execution_tokens_total
        .with_label_values(&[backend, model, "prompt"])
        .inc_by(tokens.0 as f64);

    METRICS.execution_tokens_total
        .with_label_values(&[backend, model, "completion"])
        .inc_by(tokens.1 as f64);
}

/// Record database query metrics
pub fn record_db_query(operation: &str, table: &str, duration_secs: f64) {
    METRICS.db_queries_total
        .with_label_values(&[operation, table])
        .inc();

    METRICS.db_query_duration_seconds
        .with_label_values(&[operation])
        .observe(duration_secs);
}

/// Record cache metrics
pub fn record_cache_hit() {
    METRICS.cache_hits_total.inc();
}

pub fn record_cache_miss() {
    METRICS.cache_misses_total.inc();
}

/// Update business metrics
pub async fn update_business_metrics(state: &AppState) {
    let storage = state.storage();

    // Update mission count
    if let Ok(count) = storage.missions().count().await {
        METRICS.missions_total.set(count as f64);
    }

    // Update spec counts by status
    if let Ok(counts) = storage.specs().count_by_status().await {
        for (status, count) in counts {
            METRICS.specs_total
                .with_label_values(&[&status])
                .set(count as f64);
        }
    }
}
```

### Routes

```rust
// src/server/routes/metrics.rs
use axum::{Router, routing::get};

use crate::server::state::AppState;
use crate::server::handlers::metrics::metrics_handler;

pub fn metrics_routes() -> Router<AppState> {
    Router::new()
        .route("/metrics", get(metrics_handler))
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        let metrics = Metrics::new();
        assert!(metrics.encode().contains("http_requests_total"));
    }

    #[test]
    fn test_counter_increment() {
        METRICS.http_requests_total
            .with_label_values(&["GET", "/test", "200"])
            .inc();

        let output = METRICS.encode();
        assert!(output.contains("http_requests_total"));
    }

    #[test]
    fn test_histogram_observation() {
        METRICS.http_request_duration_seconds
            .with_label_values(&["GET", "/test"])
            .observe(0.5);

        let output = METRICS.encode();
        assert!(output.contains("http_request_duration_seconds"));
    }

    #[test]
    fn test_path_normalization() {
        let path = "/api/v1/missions/123e4567-e89b-12d3-a456-426614174000/specs";
        let normalized = normalize_path(path);
        assert_eq!(normalized, "/api/v1/missions/:id/specs");
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let response = metrics_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

---

## Related Specs

- **Spec 322**: Health Checks
- **Spec 334**: Distributed Tracing
- **Spec 314**: Middleware Stack
