# Spec 334: Distributed Tracing

## Phase
15 - Server/API Layer

## Spec ID
334

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 314: Middleware

## Estimated Context
~10%

---

## Objective

Implement distributed tracing for the Tachikoma server using OpenTelemetry, enabling request correlation across services, performance analysis, and debugging of complex request flows.

---

## Acceptance Criteria

- [ ] OpenTelemetry integration
- [ ] Request tracing with spans
- [ ] Trace context propagation
- [ ] Custom span attributes
- [ ] Configurable trace sampling
- [ ] Export to multiple backends (Jaeger, Zipkin, OTLP)
- [ ] Correlation with logs and metrics

---

## Implementation Details

### Tracing Configuration

```rust
// src/server/tracing/config.rs
use serde::{Deserialize, Serialize};

/// Tracing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,

    /// Service name for traces
    pub service_name: String,

    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,

    /// Export destination
    pub exporter: TracingExporter,

    /// OTLP endpoint
    pub otlp_endpoint: Option<String>,

    /// Jaeger agent endpoint
    pub jaeger_endpoint: Option<String>,

    /// Include span events
    pub include_events: bool,

    /// Include span links
    pub include_links: bool,

    /// Maximum attributes per span
    pub max_attributes: u32,

    /// Maximum events per span
    pub max_events: u32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TracingExporter {
    #[default]
    None,
    Jaeger,
    Zipkin,
    Otlp,
    Stdout,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "tachikoma".to_string(),
            sampling_rate: 1.0,
            exporter: TracingExporter::None,
            otlp_endpoint: None,
            jaeger_endpoint: None,
            include_events: true,
            include_links: false,
            max_attributes: 128,
            max_events: 128,
        }
    }
}
```

### Tracing Setup

```rust
// src/server/tracing/setup.rs
use opentelemetry::{
    global,
    sdk::{
        propagation::TraceContextPropagator,
        trace::{self, Sampler, Tracer},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

use super::config::{TracingConfig, TracingExporter};

/// Initialize tracing with the given configuration
pub fn init_tracing(config: &TracingConfig) -> Result<(), TracingError> {
    // Set up global propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Create resource with service information
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    // Create sampler
    let sampler = if config.sampling_rate >= 1.0 {
        Sampler::AlwaysOn
    } else if config.sampling_rate <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.sampling_rate)
    };

    // Create tracer provider based on exporter
    let tracer = match config.exporter {
        TracingExporter::None => {
            // No-op tracer
            None
        }
        TracingExporter::Stdout => {
            let exporter = opentelemetry_stdout::SpanExporter::default();
            let provider = trace::TracerProvider::builder()
                .with_resource(resource)
                .with_sampler(sampler)
                .with_simple_exporter(exporter)
                .build();
            global::set_tracer_provider(provider.clone());
            Some(provider.tracer("tachikoma"))
        }
        TracingExporter::Jaeger => {
            let endpoint = config.jaeger_endpoint.as_deref()
                .unwrap_or("http://localhost:14268/api/traces");

            let tracer = opentelemetry_jaeger::new_collector_pipeline()
                .with_service_name(&config.service_name)
                .with_endpoint(endpoint)
                .with_trace_config(
                    trace::config()
                        .with_resource(resource)
                        .with_sampler(sampler)
                )
                .install_batch(opentelemetry::runtime::Tokio)?;

            Some(tracer)
        }
        TracingExporter::Otlp => {
            let endpoint = config.otlp_endpoint.as_deref()
                .unwrap_or("http://localhost:4317");

            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(endpoint)
                )
                .with_trace_config(
                    trace::config()
                        .with_resource(resource)
                        .with_sampler(sampler)
                )
                .install_batch(opentelemetry::runtime::Tokio)?;

            Some(tracer)
        }
        TracingExporter::Zipkin => {
            let tracer = opentelemetry_zipkin::new_pipeline()
                .with_service_name(&config.service_name)
                .with_trace_config(
                    trace::config()
                        .with_resource(resource)
                        .with_sampler(sampler)
                )
                .install_batch(opentelemetry::runtime::Tokio)?;

            Some(tracer)
        }
    };

    // Set up tracing subscriber
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tachikoma=debug"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if let Some(tracer) = tracer {
        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        subscriber.with(telemetry_layer).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

/// Shutdown tracing providers
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Failed to initialize tracer: {0}")]
    Init(String),

    #[error("OpenTelemetry error: {0}")]
    OpenTelemetry(#[from] opentelemetry::trace::TraceError),
}
```

### Tracing Middleware

```rust
// src/server/middleware/tracing.rs
use axum::{
    extract::Request,
    http::{header, HeaderMap},
    middleware::Next,
    response::Response,
};
use opentelemetry::{
    global,
    propagation::Extractor,
    trace::{SpanKind, Status, TraceContextExt, Tracer},
    Context,
};
use tracing::{instrument, Span};

/// Header extractor for trace context propagation
struct HeaderExtractor<'a>(&'a HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// Tracing middleware that creates spans for requests
pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    let tracer = global::tracer("tachikoma");

    // Extract trace context from headers
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });

    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let path = request.uri().path().to_string();

    // Create span
    let mut span = tracer
        .span_builder(format!("{} {}", method, path))
        .with_kind(SpanKind::Server)
        .with_attributes(vec![
            opentelemetry::KeyValue::new("http.method", method.clone()),
            opentelemetry::KeyValue::new("http.url", uri.clone()),
            opentelemetry::KeyValue::new("http.route", path.clone()),
        ])
        .start_with_context(&tracer, &parent_context);

    let cx = Context::current_with_span(span);
    let _guard = cx.attach();

    // Add trace ID to request extensions
    let trace_id = cx.span().span_context().trace_id().to_string();

    // Run the request
    let response = next.run(request).await;

    // Update span with response info
    let status_code = response.status().as_u16();
    let current_span = cx.span();

    current_span.set_attribute(opentelemetry::KeyValue::new(
        "http.status_code",
        status_code as i64,
    ));

    if status_code >= 500 {
        current_span.set_status(Status::error("Server error"));
    } else if status_code >= 400 {
        current_span.set_status(Status::error("Client error"));
    } else {
        current_span.set_status(Status::Ok);
    }

    current_span.end();

    response
}

/// Extension trait for adding tracing to handlers
pub trait TracingExt {
    fn trace_span(&self, name: &str) -> tracing::Span;
}

impl TracingExt for Request {
    fn trace_span(&self, name: &str) -> tracing::Span {
        tracing::info_span!(
            name,
            method = %self.method(),
            uri = %self.uri(),
        )
    }
}
```

### Span Helpers

```rust
// src/server/tracing/spans.rs
use opentelemetry::{
    global,
    trace::{SpanKind, Status, Tracer},
    Context, KeyValue,
};
use std::future::Future;

/// Create a new span for a specific operation
pub fn create_span(name: &str, kind: SpanKind) -> impl Drop {
    let tracer = global::tracer("tachikoma");
    let span = tracer
        .span_builder(name)
        .with_kind(kind)
        .start(&tracer);

    SpanGuard {
        cx: Context::current_with_span(span),
    }
}

struct SpanGuard {
    cx: Context,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        self.cx.span().end();
    }
}

/// Instrument a future with a span
pub async fn with_span<F, T>(name: &str, kind: SpanKind, f: F) -> T
where
    F: Future<Output = T>,
{
    let tracer = global::tracer("tachikoma");
    let span = tracer
        .span_builder(name)
        .with_kind(kind)
        .start(&tracer);

    let cx = Context::current_with_span(span);
    let _guard = cx.attach();

    let result = f.await;

    cx.span().end();
    result
}

/// Add an event to the current span
pub fn span_event(name: &str, attributes: Vec<KeyValue>) {
    let cx = Context::current();
    cx.span().add_event(name, attributes);
}

/// Set an attribute on the current span
pub fn span_attribute(key: &str, value: impl Into<opentelemetry::Value>) {
    let cx = Context::current();
    cx.span().set_attribute(KeyValue::new(key.to_string(), value.into()));
}

/// Set the span status
pub fn span_status(status: Status) {
    let cx = Context::current();
    cx.span().set_status(status);
}

/// Record an error on the current span
pub fn span_error(error: &dyn std::error::Error) {
    let cx = Context::current();
    cx.span().record_error(error);
    cx.span().set_status(Status::error(error.to_string()));
}

/// Get the current trace ID
pub fn current_trace_id() -> Option<String> {
    let cx = Context::current();
    let span_context = cx.span().span_context();
    if span_context.is_valid() {
        Some(span_context.trace_id().to_string())
    } else {
        None
    }
}

/// Get the current span ID
pub fn current_span_id() -> Option<String> {
    let cx = Context::current();
    let span_context = cx.span().span_context();
    if span_context.is_valid() {
        Some(span_context.span_id().to_string())
    } else {
        None
    }
}
```

### Instrumented Operations

```rust
// src/server/tracing/instrumented.rs
use opentelemetry::{
    global,
    trace::{SpanKind, Tracer},
    KeyValue,
};

/// Instrumented database query
pub async fn traced_query<F, T, E>(
    operation: &str,
    table: &str,
    query_fn: F,
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let tracer = global::tracer("tachikoma");
    let span = tracer
        .span_builder(format!("db.{}", operation))
        .with_kind(SpanKind::Client)
        .with_attributes(vec![
            KeyValue::new("db.system", "sqlite"),
            KeyValue::new("db.operation", operation.to_string()),
            KeyValue::new("db.table", table.to_string()),
        ])
        .start(&tracer);

    let cx = opentelemetry::Context::current_with_span(span);
    let _guard = cx.attach();

    let result = query_fn.await;

    if let Err(ref e) = result {
        cx.span().set_status(opentelemetry::trace::Status::error(e.to_string()));
    }

    cx.span().end();
    result
}

/// Instrumented backend call
pub async fn traced_backend_call<F, T, E>(
    backend: &str,
    operation: &str,
    call_fn: F,
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let tracer = global::tracer("tachikoma");
    let span = tracer
        .span_builder(format!("backend.{}.{}", backend, operation))
        .with_kind(SpanKind::Client)
        .with_attributes(vec![
            KeyValue::new("backend.name", backend.to_string()),
            KeyValue::new("backend.operation", operation.to_string()),
        ])
        .start(&tracer);

    let cx = opentelemetry::Context::current_with_span(span);
    let _guard = cx.attach();

    let result = call_fn.await;

    if let Err(ref e) = result {
        cx.span().set_status(opentelemetry::trace::Status::error(e.to_string()));
    }

    cx.span().end();
    result
}

/// Instrumented LLM execution
pub async fn traced_execution<F, T, E>(
    spec_id: &str,
    backend: &str,
    model: &str,
    execution_fn: F,
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let tracer = global::tracer("tachikoma");
    let span = tracer
        .span_builder("llm.execution")
        .with_kind(SpanKind::Client)
        .with_attributes(vec![
            KeyValue::new("spec.id", spec_id.to_string()),
            KeyValue::new("llm.backend", backend.to_string()),
            KeyValue::new("llm.model", model.to_string()),
        ])
        .start(&tracer);

    let cx = opentelemetry::Context::current_with_span(span);
    let _guard = cx.attach();

    let start = std::time::Instant::now();
    let result = execution_fn.await;
    let duration = start.elapsed();

    cx.span().set_attribute(KeyValue::new(
        "llm.duration_ms",
        duration.as_millis() as i64,
    ));

    if let Err(ref e) = result {
        cx.span().set_status(opentelemetry::trace::Status::error(e.to_string()));
    }

    cx.span().end();
    result
}
```

### Log-Trace Correlation

```rust
// src/server/tracing/correlation.rs
use tracing_subscriber::fmt::format::FmtSpan;

/// Custom log formatter that includes trace context
pub struct TracingFormatter;

impl TracingFormatter {
    /// Create a tracing layer with trace ID correlation
    pub fn layer() -> impl tracing_subscriber::Layer<tracing_subscriber::Registry> {
        tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .fmt_fields(TracingFieldFormatter)
    }
}

struct TracingFieldFormatter;

impl<'writer> tracing_subscriber::fmt::FormatFields<'writer> for TracingFieldFormatter {
    fn format_fields<R: tracing_subscriber::field::RecordFields>(
        &self,
        writer: tracing_subscriber::fmt::format::Writer<'writer>,
        fields: R,
    ) -> std::fmt::Result {
        // Include trace_id if available
        if let Some(trace_id) = super::spans::current_trace_id() {
            write!(writer, "trace_id={} ", trace_id)?;
        }

        // Default field formatting
        tracing_subscriber::fmt::format::DefaultFields::new().format_fields(writer, fields)
    }
}

/// Macro for logging with trace context
#[macro_export]
macro_rules! trace_log {
    ($level:ident, $($arg:tt)*) => {
        {
            let trace_id = $crate::server::tracing::spans::current_trace_id()
                .unwrap_or_else(|| "none".to_string());
            tracing::$level!(trace_id = %trace_id, $($arg)*);
        }
    };
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
    fn test_tracing_config_defaults() {
        let config = TracingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.sampling_rate, 1.0);
    }

    #[tokio::test]
    async fn test_span_creation() {
        // Initialize stdout tracer for testing
        let config = TracingConfig {
            exporter: TracingExporter::Stdout,
            ..Default::default()
        };
        init_tracing(&config).unwrap();

        // Create a test span
        {
            let _guard = create_span("test_operation", SpanKind::Internal);
            span_attribute("test.key", "test_value");
            span_event("test_event", vec![]);
        }

        shutdown_tracing();
    }

    #[test]
    fn test_header_extractor() {
        let mut headers = HeaderMap::new();
        headers.insert("traceparent", "00-abc123-def456-01".parse().unwrap());

        let extractor = HeaderExtractor(&headers);
        assert!(extractor.get("traceparent").is_some());
    }
}
```

---

## Related Specs

- **Spec 314**: Middleware Stack
- **Spec 333**: Prometheus Metrics
- **Spec 312**: Server Configuration
