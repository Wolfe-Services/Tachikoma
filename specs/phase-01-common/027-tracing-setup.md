# 027 - Tracing Setup

**Phase:** 1 - Core Common Crates
**Spec ID:** 027
**Status:** Planned
**Dependencies:** 026-logging-infrastructure
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure distributed tracing with spans, instrumentation macros, and context propagation for debugging complex operations.

---

## Acceptance Criteria

- [x] Span creation and nesting
- [x] Instrumentation attributes
- [x] Context propagation across async
- [x] Span field recording
- [x] Performance-aware tracing

---

## Implementation Details

### 1. Tracing Module (crates/tachikoma-common-log/src/tracing.rs)

```rust
//! Distributed tracing utilities.

use std::future::Future;
use tracing::{info_span, instrument, Instrument, Span};

/// Create a span for a mission operation.
pub fn mission_span(mission_id: &str) -> Span {
    info_span!("mission", id = %mission_id)
}

/// Create a span for a backend operation.
pub fn backend_span(backend: &str, operation: &str) -> Span {
    info_span!("backend", name = %backend, op = %operation)
}

/// Create a span for a file operation.
pub fn file_span(operation: &str, path: &str) -> Span {
    info_span!("file", op = %operation, path = %path)
}

/// Instrument a future with a span.
pub fn instrument_future<F: Future>(future: F, span: Span) -> impl Future<Output = F::Output> {
    future.instrument(span)
}

/// Record an error on the current span.
pub fn record_error(error: &dyn std::error::Error) {
    Span::current().record("error", tracing::field::display(error));
}

/// Timing utility for operations.
pub struct Timer {
    start: std::time::Instant,
    operation: &'static str,
}

impl Timer {
    /// Start a new timer.
    pub fn start(operation: &'static str) -> Self {
        Self {
            start: std::time::Instant::now(),
            operation,
        }
    }

    /// Complete the timer and record duration.
    pub fn finish(self) {
        let duration = self.start.elapsed();
        tracing::debug!(
            operation = %self.operation,
            duration_ms = %duration.as_millis(),
            "operation completed"
        );
    }
}

/// Macro for timing a block of code.
#[macro_export]
macro_rules! timed {
    ($name:expr, $body:expr) => {{
        let _timer = $crate::tracing::Timer::start($name);
        let result = $body;
        _timer.finish();
        result
    }};
}

/// Attribute macro for instrumenting functions.
///
/// Re-export of tracing::instrument for convenience.
pub use tracing::instrument;

/// Example instrumented function.
#[instrument(skip(data), fields(size = data.len()))]
pub async fn example_instrumented_fn(id: &str, data: &[u8]) -> Result<(), String> {
    tracing::info!("processing data");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_span() {
        let span = mission_span("test-123");
        let _guard = span.enter();
        // Span should be active
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.finish();
    }
}
```

### 2. Common Instrumentation Patterns

```rust
// In any crate, use the instrument attribute:

use tachikoma_common_log::tracing::instrument;

#[instrument(
    name = "create_mission",
    skip(config),           // Don't log large/sensitive args
    fields(spec = %spec_path),
    err                     // Log errors automatically
)]
pub async fn create_mission(spec_path: &str, config: &Config) -> Result<Mission, Error> {
    tracing::debug!("creating mission from spec");
    // ...
}
```

---

## Testing Requirements

1. Spans are created with correct fields
2. Nested spans maintain hierarchy
3. Async context is propagated
4. Timer records accurate duration

---

## Related Specs

- Depends on: [026-logging-infrastructure.md](026-logging-infrastructure.md)
- Next: [028-metrics-foundation.md](028-metrics-foundation.md)
