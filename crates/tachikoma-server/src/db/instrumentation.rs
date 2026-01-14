//! Query instrumentation for logging and metrics.

use std::time::{Duration, Instant};
use tracing::{debug, info, warn, Span};

/// Query execution timer.
pub struct QueryTimer {
    query: String,
    start: Instant,
    slow_threshold: Duration,
}

impl QueryTimer {
    pub fn new(query: impl Into<String>, slow_threshold: Duration) -> Self {
        Self {
            query: query.into(),
            start: Instant::now(),
            slow_threshold,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn finish(self) -> Duration {
        let elapsed = self.elapsed();
        let elapsed_ms = elapsed.as_millis();

        if elapsed > self.slow_threshold {
            warn!(
                query = %self.query,
                elapsed_ms = elapsed_ms,
                "Slow query detected"
            );
        } else {
            debug!(
                query = %self.query,
                elapsed_ms = elapsed_ms,
                "Query completed"
            );
        }

        elapsed
    }
}

/// Macro for timing queries.
#[macro_export]
macro_rules! timed_query {
    ($pool:expr, $query:expr, $slow_threshold:expr) => {{
        let timer = $crate::db::instrumentation::QueryTimer::new(
            stringify!($query),
            $slow_threshold,
        );
        let result = $query;
        timer.finish();
        result
    }};
}

/// Query logger for SQLx.
pub struct QueryLogger {
    slow_threshold: Duration,
}

impl QueryLogger {
    pub fn new(slow_threshold: Duration) -> Self {
        Self { slow_threshold }
    }
}

// Note: Would implement sqlx::QueryLogger trait here