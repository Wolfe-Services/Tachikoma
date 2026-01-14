//! Request monitoring and statistics.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Request statistics.
#[derive(Debug, Clone, Default)]
pub struct RequestStats {
    /// Total requests.
    pub total_requests: u64,
    /// Requests in the last minute.
    pub requests_per_minute: f64,
    /// Error count.
    pub error_count: u64,
    /// Error rate (percentage).
    pub error_rate: f64,
    /// Average latency (ms).
    pub avg_latency_ms: f64,
    /// P50 latency (ms).
    pub p50_latency_ms: f64,
    /// P95 latency (ms).
    pub p95_latency_ms: f64,
    /// P99 latency (ms).
    pub p99_latency_ms: f64,
    /// Max latency (ms).
    pub max_latency_ms: f64,
}

/// Request monitor tracking request metrics.
pub struct RequestMonitor {
    /// Total request counter.
    total_requests: AtomicU64,
    /// Error counter.
    error_count: AtomicU64,
    /// Recent request latencies (for percentile calculation).
    latencies: Arc<RwLock<VecDeque<RequestRecord>>>,
    /// Window duration.
    window: Duration,
}

struct RequestRecord {
    timestamp: Instant,
    latency_ms: f64,
    is_error: bool,
}

impl RequestMonitor {
    pub fn new(window: Duration) -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            latencies: Arc::new(RwLock::new(VecDeque::new())),
            window,
        }
    }

    /// Record a request.
    pub async fn record(&self, latency_ms: f64, is_error: bool) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        if is_error {
            self.error_count.fetch_add(1, Ordering::SeqCst);
        }

        let record = RequestRecord {
            timestamp: Instant::now(),
            latency_ms,
            is_error,
        };

        let mut latencies = self.latencies.write().await;
        latencies.push_back(record);

        // Clean old records
        let cutoff = Instant::now() - self.window;
        while let Some(front) = latencies.front() {
            if front.timestamp < cutoff {
                latencies.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get current statistics.
    pub async fn stats(&self) -> RequestStats {
        let latencies = self.latencies.read().await;
        let total = self.total_requests.load(Ordering::SeqCst);
        let errors = self.error_count.load(Ordering::SeqCst);

        if latencies.is_empty() {
            return RequestStats {
                total_requests: total,
                error_count: errors,
                ..Default::default()
            };
        }

        // Collect latency values
        let mut values: Vec<f64> = latencies.iter().map(|r| r.latency_ms).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let window_errors = latencies.iter().filter(|r| r.is_error).count();
        let window_total = latencies.len();

        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let p50 = percentile(&values, 50.0);
        let p95 = percentile(&values, 95.0);
        let p99 = percentile(&values, 99.0);
        let max = values.last().copied().unwrap_or(0.0);

        // Calculate requests per minute
        let oldest = latencies.front().map(|r| r.timestamp);
        let newest = latencies.back().map(|r| r.timestamp);
        let rpm = if let (Some(old), Some(new)) = (oldest, newest) {
            let duration = new.duration_since(old).as_secs_f64();
            if duration > 0.0 {
                (window_total as f64 / duration) * 60.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        RequestStats {
            total_requests: total,
            requests_per_minute: rpm,
            error_count: errors,
            error_rate: if window_total > 0 {
                (window_errors as f64 / window_total as f64) * 100.0
            } else {
                0.0
            },
            avg_latency_ms: avg,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            max_latency_ms: max,
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (p / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values.get(index).copied().unwrap_or(0.0)
}