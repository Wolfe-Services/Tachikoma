use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;

/// Standard benchmark configuration
pub fn bench_config() -> Criterion {
    Criterion::default()
        .significance_level(0.05)
        .sample_size(100)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2))
        .with_output_color(true)
}

/// Quick benchmark configuration for development
pub fn quick_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .measurement_time(Duration::from_secs(1))
        .warm_up_time(Duration::from_millis(500))
}

/// CI benchmark configuration (more rigorous)
pub fn ci_config() -> Criterion {
    Criterion::default()
        .significance_level(0.01)
        .sample_size(200)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3))
        .noise_threshold(0.03)
}