# 328 - Metrics Endpoints

**Phase:** 15 - Server
**Spec ID:** 328
**Status:** Planned
**Dependencies:** 317-axum-router
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement Prometheus-compatible metrics endpoints with request metrics, custom application metrics, and histogram support.

---

## Acceptance Criteria

- [ ] Prometheus metrics endpoint (/metrics)
- [ ] Request duration histograms
- [ ] Request count by status/method/path
- [ ] Active connections gauge
- [ ] Custom application metrics
- [ ] Metric labels support
- [ ] Metric middleware integration

---

## Implementation Details

### 1. Metrics Types (crates/tachikoma-server/src/metrics/types.rs)

```rust
//! Metrics types and collectors.

use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};
use std::sync::Arc;

/// Application metrics collection.
#[derive(Clone)]
pub struct AppMetrics {
    pub registry: Arc<Registry>,

    // HTTP metrics
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_requests_in_flight: IntGauge,
    pub http_response_size_bytes: HistogramVec,

    // Application metrics
    pub missions_total: IntCounterVec,
    pub missions_active: IntGauge,
    pub forge_sessions_total: IntCounterVec,
    pub forge_sessions_active: IntGauge,
    pub tokens_used_total: IntCounterVec,
    pub tokens_cost_dollars: CounterVec,

    // Database metrics
    pub db_connections_active: IntGauge,
    pub db_connections_idle: IntGauge,
    pub db_query_duration_seconds: HistogramVec,

    // Cache metrics
    pub cache_hits_total: IntCounter,
    pub cache_misses_total: IntCounter,
}

impl AppMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        // HTTP metrics
        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "path", "status"],
        )
        .unwrap();

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "path"],
        )
        .unwrap();

        let http_requests_in_flight = IntGauge::new(
            "http_requests_in_flight",
            "Current number of HTTP requests being processed",
        )
        .unwrap();

        let http_response_size_bytes = HistogramVec::new(
            HistogramOpts::new(
                "http_response_size_bytes",
                "HTTP response size in bytes",
            )
            .buckets(prometheus::exponential_buckets(100.0, 10.0, 8).unwrap()),
            &["method", "path"],
        )
        .unwrap();

        // Application metrics
        let missions_total = IntCounterVec::new(
            Opts::new("tachikoma_missions_total", "Total missions created"),
            &["status"],
        )
        .unwrap();

        let missions_active = IntGauge::new(
            "tachikoma_missions_active",
            "Currently active missions",
        )
        .unwrap();

        let forge_sessions_total = IntCounterVec::new(
            Opts::new("tachikoma_forge_sessions_total", "Total forge sessions"),
            &["status"],
        )
        .unwrap();

        let forge_sessions_active = IntGauge::new(
            "tachikoma_forge_sessions_active",
            "Currently active forge sessions",
        )
        .unwrap();

        let tokens_used_total = IntCounterVec::new(
            Opts::new("tachikoma_tokens_used_total", "Total tokens used"),
            &["model", "type"],
        )
        .unwrap();

        let tokens_cost_dollars = CounterVec::new(
            Opts::new("tachikoma_tokens_cost_dollars", "Total cost in dollars"),
            &["model"],
        )
        .unwrap();

        // Database metrics
        let db_connections_active = IntGauge::new(
            "db_connections_active",
            "Active database connections",
        )
        .unwrap();

        let db_connections_idle = IntGauge::new(
            "db_connections_idle",
            "Idle database connections",
        )
        .unwrap();

        let db_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "db_query_duration_seconds",
                "Database query duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["query_type"],
        )
        .unwrap();

        // Cache metrics
        let cache_hits_total = IntCounter::new(
            "cache_hits_total",
            "Total cache hits",
        )
        .unwrap();

        let cache_misses_total = IntCounter::new(
            "cache_misses_total",
            "Total cache misses",
        )
        .unwrap();

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone())).unwrap();
        registry.register(Box::new(http_request_duration_seconds.clone())).unwrap();
        registry.register(Box::new(http_requests_in_flight.clone())).unwrap();
        registry.register(Box::new(http_response_size_bytes.clone())).unwrap();
        registry.register(Box::new(missions_total.clone())).unwrap();
        registry.register(Box::new(missions_active.clone())).unwrap();
        registry.register(Box::new(forge_sessions_total.clone())).unwrap();
        registry.register(Box::new(forge_sessions_active.clone())).unwrap();
        registry.register(Box::new(tokens_used_total.clone())).unwrap();
        registry.register(Box::new(tokens_cost_dollars.clone())).unwrap();
        registry.register(Box::new(db_connections_active.clone())).unwrap();
        registry.register(Box::new(db_connections_idle.clone())).unwrap();
        registry.register(Box::new(db_query_duration_seconds.clone())).unwrap();
        registry.register(Box::new(cache_hits_total.clone())).unwrap();
        registry.register(Box::new(cache_misses_total.clone())).unwrap();

        Self {
            registry: Arc::new(registry),
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
            http_response_size_bytes,
            missions_total,
            missions_active,
            forge_sessions_total,
            forge_sessions_active,
            tokens_used_total,
            tokens_cost_dollars,
            db_connections_active,
            db_connections_idle,
            db_query_duration_seconds,
            cache_hits_total,
            cache_misses_total,
        }
    }

    /// Record an HTTP request.
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        self.http_requests_total
            .with_label_values(&[method, path, &status.to_string()])
            .inc();

        self.http_request_duration_seconds
            .with_label_values(&[method, path])
            .observe(duration_secs);
    }

    /// Record token usage.
    pub fn record_tokens(&self, model: &str, input_tokens: u64, output_tokens: u64, cost: f64) {
        self.tokens_used_total
            .with_label_values(&[model, "input"])
            .inc_by(input_tokens);
        self.tokens_used_total
            .with_label_values(&[model, "output"])
            .inc_by(output_tokens);
        self.tokens_cost_dollars
            .with_label_values(&[model])
            .inc_by(cost);
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Metrics Middleware (crates/tachikoma-server/src/metrics/middleware.rs)

```rust
//! Metrics collection middleware.

use super::types::AppMetrics;
use axum::{
    body::Body,
    http::Request,
    response::Response,
};
use std::time::Instant;
use tower::{Layer, Service};

/// Metrics collection layer.
#[derive(Clone)]
pub struct MetricsLayer {
    metrics: AppMetrics,
}

impl MetricsLayer {
    pub fn new(metrics: AppMetrics) -> Self {
        Self { metrics }
    }
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware {
            inner,
            metrics: self.metrics.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MetricsMiddleware<S> {
    inner: S,
    metrics: AppMetrics,
}

impl<S> Service<Request<Body>> for MetricsMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let metrics = self.metrics.clone();
        let mut inner = self.inner.clone();

        let method = req.method().to_string();
        let path = normalize_path(req.uri().path());

        // Increment in-flight counter
        metrics.http_requests_in_flight.inc();

        let start = Instant::now();

        Box::pin(async move {
            let response = inner.call(req).await?;

            // Decrement in-flight counter
            metrics.http_requests_in_flight.dec();

            // Record metrics
            let duration = start.elapsed().as_secs_f64();
            let status = response.status().as_u16();

            metrics.record_http_request(&method, &path, status, duration);

            Ok(response)
        })
    }
}

/// Normalize path for metrics (replace IDs with placeholders).
fn normalize_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let normalized: Vec<String> = parts
        .iter()
        .map(|part| {
            // Check if part looks like a UUID or numeric ID
            if is_uuid(part) || is_numeric_id(part) {
                ":id".to_string()
            } else {
                part.to_string()
            }
        })
        .collect();

    normalized.join("/")
}

fn is_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}

fn is_numeric_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}
```

### 3. Metrics Handler (crates/tachikoma-server/src/metrics/handler.rs)

```rust
//! Metrics endpoint handler.

use super::types::AppMetrics;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use prometheus::Encoder;
use std::sync::Arc;

/// Prometheus metrics endpoint handler.
pub async fn metrics_handler(State(metrics): State<Arc<AppMetrics>>) -> Response {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = metrics.registry.gather();

    let mut buffer = Vec::new();
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, prometheus::TEXT_FORMAT)],
                buffer,
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode metrics: {}", e),
            )
                .into_response()
        }
    }
}

/// JSON metrics endpoint (simplified view).
pub async fn metrics_json_handler(State(metrics): State<Arc<AppMetrics>>) -> Response {
    let metric_families = metrics.registry.gather();

    let mut output = serde_json::Map::new();

    for mf in metric_families {
        let name = mf.get_name();
        let metrics: Vec<serde_json::Value> = mf
            .get_metric()
            .iter()
            .map(|m| {
                let labels: serde_json::Map<String, serde_json::Value> = m
                    .get_label()
                    .iter()
                    .map(|l| (l.get_name().to_string(), serde_json::json!(l.get_value())))
                    .collect();

                let value = if m.has_counter() {
                    m.get_counter().get_value()
                } else if m.has_gauge() {
                    m.get_gauge().get_value()
                } else if m.has_histogram() {
                    m.get_histogram().get_sample_sum()
                } else {
                    0.0
                };

                serde_json::json!({
                    "labels": labels,
                    "value": value,
                })
            })
            .collect();

        output.insert(name.to_string(), serde_json::json!(metrics));
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&output).unwrap(),
    )
        .into_response()
}
```

### 4. Metrics Router (crates/tachikoma-server/src/metrics/router.rs)

```rust
//! Metrics routes configuration.

use super::{
    handler::{metrics_handler, metrics_json_handler},
    types::AppMetrics,
};
use axum::{routing::get, Router};
use std::sync::Arc;

/// Create metrics routes.
pub fn metrics_routes(metrics: Arc<AppMetrics>) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/metrics/json", get(metrics_json_handler))
        .with_state(metrics)
}
```

---

## Testing Requirements

1. Prometheus format correct
2. Request metrics recorded
3. Path normalization works
4. Histogram buckets correct
5. Counter increments properly
6. Gauge reflects current state
7. Custom metrics work

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [329-websocket-setup.md](329-websocket-setup.md)
- Used by: Prometheus, Grafana
