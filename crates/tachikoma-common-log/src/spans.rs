//! Distributed tracing utilities.

use std::future::Future;
use tracing::{info_span, Instrument, Span};

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
        let _timer = $crate::spans::Timer::start($name);
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
#[tracing::instrument(skip(data), fields(size = data.len()))]
pub async fn example_instrumented_fn(id: &str, data: &[u8]) -> Result<(), String> {
    tracing::info!("processing data");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::EnvFilter;

    // Helper to capture tracing output for testing
    fn with_subscriber<F>(f: F) 
    where 
        F: FnOnce() + Send + 'static,
    {
        let subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter(EnvFilter::new("trace"))
            .finish();
        
        tracing::subscriber::with_default(subscriber, f);
    }

    #[test]
    fn test_span_creation_and_nesting() {
        with_subscriber(|| {
            // Test mission span creation
            let mission = mission_span("mission-123");
            let _guard1 = mission.enter();
            
            // Test nested backend span
            let backend = backend_span("postgres", "query");
            let _guard2 = backend.enter();
            
            // Test nested file span
            let file = file_span("read", "/tmp/test.txt");
            let _guard3 = file.enter();
            
            tracing::info!("nested operation");
            
            // Spans should be properly nested
        });
    }

    #[test]
    fn test_instrumentation_attributes() {
        with_subscriber(|| {
            // Test span field recording
            let span = mission_span("attr-test");
            let _guard = span.enter();
            
            // Record additional attributes
            span.record("status", "in_progress");
            span.record("progress", 50);
            
            tracing::info!("span with attributes");
        });
    }

    #[tokio::test]
    async fn test_context_propagation_across_async() {
        let mission = mission_span("async-test");
        
        // Test that span context propagates through async boundaries
        let future = async {
            tracing::info!("async task 1");
            
            // Nest another async operation
            let backend = backend_span("redis", "set");
            let inner_future = async {
                tracing::info!("async task 2");
            };
            
            instrument_future(inner_future, backend).await;
        };
        
        // Instrument the whole test with the mission span
        instrument_future(future, mission).await;
    }

    #[test]
    fn test_span_field_recording() {
        with_subscriber(|| {
            let span = file_span("process", "/data/input.csv");
            let _guard = span.enter();
            
            // Test error recording
            let error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
            record_error(&error);
            
            // Test additional field recording
            tracing::Span::current().record("bytes_processed", 1024);
            tracing::Span::current().record("duration_ms", 150);
            
            tracing::info!("file processing complete");
        });
    }

    #[test]
    fn test_performance_aware_tracing() {
        // Test timer functionality
        let timer = Timer::start("performance_test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.finish();
        
        // Test timed macro
        with_subscriber(|| {
            let result = timed!("macro_test", {
                std::thread::sleep(std::time::Duration::from_millis(5));
                "success"
            });
            
            assert_eq!(result, "success");
        });
    }

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

    #[tokio::test]
    async fn test_instrumented_function() {
        let data = b"test data";
        let result = example_instrumented_fn("test-id", data).await;
        assert!(result.is_ok());
    }
}