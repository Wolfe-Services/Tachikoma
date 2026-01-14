//! API benchmarks for the Tachikoma server.

use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn benchmark_health_check(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("health_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                // TODO: Implement actual health check benchmark
                // when handlers are available
                tokio::time::sleep(std::time::Duration::from_nanos(1)).await;
            })
        })
    });
}

criterion_group!(benches, benchmark_health_check);
criterion_main!(benches);