use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

mod criterion_config;
use criterion_config::bench_config;

/// Benchmark string operations
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // Test different string sizes
    for size in [100, 1000, 10000, 100000].iter() {
        let input = "a".repeat(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("clone", size),
            &input,
            |b, input| {
                b.iter(|| black_box(input.clone()))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("to_lowercase", size),
            &input,
            |b, input| {
                b.iter(|| black_box(input.to_lowercase()))
            },
        );
    }

    group.finish();
}

/// Benchmark JSON serialization
fn bench_json_serialization(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize)]
    struct TestData {
        id: String,
        name: String,
        values: Vec<i64>,
        metadata: HashMap<String, String>,
    }

    let test_data = TestData {
        id: "test-123".into(),
        name: "Benchmark Test".into(),
        values: (0..100).collect(),
        metadata: (0..10)
            .map(|i| (format!("key_{}", i), format!("value_{}", i)))
            .collect(),
    };

    let json_string = serde_json::to_string(&test_data).unwrap();

    let mut group = c.benchmark_group("json");

    group.bench_function("serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&test_data).unwrap()))
    });

    group.bench_function("deserialize", |b| {
        b.iter(|| black_box(serde_json::from_str::<TestData>(&json_string).unwrap()))
    });

    group.bench_function("serialize_pretty", |b| {
        b.iter(|| black_box(serde_json::to_string_pretty(&test_data).unwrap()))
    });

    group.finish();
}

/// Benchmark collection operations
fn bench_collections(c: &mut Criterion) {
    use std::collections::{HashMap, BTreeMap};

    let mut group = c.benchmark_group("collections");

    for size in [100, 1000, 10000].iter() {
        let items: Vec<(String, i64)> = (0..*size)
            .map(|i| (format!("key_{}", i), i))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("hashmap_insert", size),
            &items,
            |b, items| {
                b.iter(|| {
                    let mut map = HashMap::new();
                    for (k, v) in items {
                        map.insert(k.clone(), *v);
                    }
                    black_box(map)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("btreemap_insert", size),
            &items,
            |b, items| {
                b.iter(|| {
                    let mut map = BTreeMap::new();
                    for (k, v) in items {
                        map.insert(k.clone(), *v);
                    }
                    black_box(map)
                })
            },
        );

        let hashmap: HashMap<_, _> = items.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("hashmap_lookup", size),
            &hashmap,
            |b, map| {
                b.iter(|| {
                    for i in 0..*size {
                        black_box(map.get(&format!("key_{}", i)));
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = bench_config();
    targets = bench_string_operations, bench_json_serialization, bench_collections
);

criterion_main!(benches);