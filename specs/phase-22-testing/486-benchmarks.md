# 486 - Performance Benchmarks

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 486
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement performance benchmarking infrastructure using Criterion for Rust and Vitest bench for TypeScript to measure and track performance of critical code paths.

---

## Acceptance Criteria

- [ ] Criterion benchmarks for Rust hot paths
- [ ] Vitest benchmarks for TypeScript utilities
- [ ] Baseline tracking for regression detection
- [ ] CI integration with threshold alerts
- [ ] Historical performance trend visualization
- [ ] Memory usage benchmarks included

---

## Implementation Details

### 1. Criterion Configuration for Rust

Create `crates/tachikoma-test-harness/benches/criterion_config.rs`:

```rust
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
```

### 2. Rust Benchmark Examples

Create `crates/tachikoma-common-core/benches/common_benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

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

    #[derive(Serialize, Deserialize)]
    struct TestData {
        id: String,
        name: String,
        values: Vec<i64>,
        metadata: std::collections::HashMap<String, String>,
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
    benches,
    bench_string_operations,
    bench_json_serialization,
    bench_collections
);

criterion_main!(benches);
```

### 3. Primitives Benchmarks

Create `crates/tachikoma-primitives/benches/primitive_benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tempfile::TempDir;
use std::fs;

/// Benchmark file reading operations
fn bench_read_file(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut group = c.benchmark_group("read_file");

    // Create test files of different sizes
    for size in [1024, 10240, 102400, 1024000].iter() {
        let file_path = temp_dir.path().join(format!("test_{}.txt", size));
        let content: String = (0..*size).map(|_| 'a').collect();
        fs::write(&file_path, &content).unwrap();

        group.bench_with_input(
            BenchmarkId::new("std_read", size),
            &file_path,
            |b, path| {
                b.iter(|| black_box(fs::read_to_string(path).unwrap()))
            },
        );

        // Benchmark with mmap (if implemented)
        // group.bench_with_input(
        //     BenchmarkId::new("mmap_read", size),
        //     &file_path,
        //     |b, path| {
        //         b.iter(|| black_box(mmap_read_file(path).unwrap()))
        //     },
        // );
    }

    group.finish();
}

/// Benchmark file writing operations
fn bench_write_file(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut group = c.benchmark_group("write_file");

    for size in [1024, 10240, 102400].iter() {
        let content: String = (0..*size).map(|_| 'a').collect();
        let counter = std::sync::atomic::AtomicUsize::new(0);

        group.bench_with_input(
            BenchmarkId::new("std_write", size),
            &(&temp_dir, &content),
            |b, (dir, content)| {
                b.iter(|| {
                    let n = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let path = dir.path().join(format!("write_test_{}.txt", n));
                    black_box(fs::write(&path, content).unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Benchmark code search (ripgrep-style)
fn bench_code_search(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    for i in 0..100 {
        let content = format!(
            "fn function_{}() {{\n    let value = {};\n    println!(\"test\");\n}}\n",
            i, i
        );
        fs::write(temp_dir.path().join(format!("file_{}.rs", i)), content).unwrap();
    }

    let mut group = c.benchmark_group("code_search");

    group.bench_function("simple_pattern", |b| {
        b.iter(|| {
            // Simple grep-style search
            let pattern = "function";
            let mut matches = Vec::new();
            for entry in fs::read_dir(temp_dir.path()).unwrap() {
                let path = entry.unwrap().path();
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains(pattern) {
                        matches.push(path);
                    }
                }
            }
            black_box(matches)
        })
    });

    group.bench_function("regex_pattern", |b| {
        let re = regex::Regex::new(r"fn \w+\(\)").unwrap();
        b.iter(|| {
            let mut matches = Vec::new();
            for entry in fs::read_dir(temp_dir.path()).unwrap() {
                let path = entry.unwrap().path();
                if let Ok(content) = fs::read_to_string(&path) {
                    for m in re.find_iter(&content) {
                        matches.push((path.clone(), m.as_str().to_string()));
                    }
                }
            }
            black_box(matches)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_read_file,
    bench_write_file,
    bench_code_search
);

criterion_main!(benches);
```

### 4. TypeScript Benchmarks

Create `web/src/test/bench/benchmarks.bench.ts`:

```typescript
import { bench, describe } from 'vitest';

describe('String Operations', () => {
  const smallString = 'a'.repeat(100);
  const mediumString = 'a'.repeat(10000);
  const largeString = 'a'.repeat(100000);

  bench('small string clone', () => {
    const copy = smallString.slice();
  });

  bench('medium string clone', () => {
    const copy = mediumString.slice();
  });

  bench('large string clone', () => {
    const copy = largeString.slice();
  });

  bench('string concatenation', () => {
    let result = '';
    for (let i = 0; i < 1000; i++) {
      result += 'a';
    }
  });

  bench('array join', () => {
    const parts: string[] = [];
    for (let i = 0; i < 1000; i++) {
      parts.push('a');
    }
    parts.join('');
  });
});

describe('JSON Operations', () => {
  const testData = {
    id: 'test-123',
    name: 'Benchmark Test',
    values: Array.from({ length: 100 }, (_, i) => i),
    metadata: Object.fromEntries(
      Array.from({ length: 10 }, (_, i) => [`key_${i}`, `value_${i}`])
    ),
  };

  const jsonString = JSON.stringify(testData);

  bench('JSON.stringify', () => {
    JSON.stringify(testData);
  });

  bench('JSON.parse', () => {
    JSON.parse(jsonString);
  });

  bench('JSON roundtrip', () => {
    JSON.parse(JSON.stringify(testData));
  });
});

describe('Array Operations', () => {
  const smallArray = Array.from({ length: 100 }, (_, i) => i);
  const largeArray = Array.from({ length: 10000 }, (_, i) => i);

  bench('small array map', () => {
    smallArray.map(x => x * 2);
  });

  bench('large array map', () => {
    largeArray.map(x => x * 2);
  });

  bench('small array filter', () => {
    smallArray.filter(x => x % 2 === 0);
  });

  bench('large array filter', () => {
    largeArray.filter(x => x % 2 === 0);
  });

  bench('small array reduce', () => {
    smallArray.reduce((acc, x) => acc + x, 0);
  });

  bench('large array reduce', () => {
    largeArray.reduce((acc, x) => acc + x, 0);
  });
});

describe('Object Operations', () => {
  const testObj = Object.fromEntries(
    Array.from({ length: 1000 }, (_, i) => [`key_${i}`, i])
  );

  bench('Object.keys', () => {
    Object.keys(testObj);
  });

  bench('Object.values', () => {
    Object.values(testObj);
  });

  bench('Object.entries', () => {
    Object.entries(testObj);
  });

  bench('Object spread', () => {
    const copy = { ...testObj };
  });
});
```

### 5. CI Benchmark Integration

Create `.github/workflows/benchmarks.yml`:

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  rust-benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Run benchmarks
        run: cargo bench --workspace -- --save-baseline pr

      - name: Compare against main
        if: github.event_name == 'pull_request'
        run: |
          git fetch origin main
          git checkout origin/main
          cargo bench --workspace -- --save-baseline main
          git checkout -
          cargo bench --workspace -- --baseline main --load-baseline pr

      - name: Upload benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: rust-benchmarks
          path: target/criterion/

  typescript-benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: cd web && npm ci

      - name: Run benchmarks
        run: cd web && npm run bench

      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: typescript-benchmarks
          path: web/bench-results/
```

---

## Testing Requirements

1. Benchmarks run without errors
2. Results are reproducible within noise threshold
3. CI detects significant regressions
4. Historical data is preserved
5. Memory benchmarks capture allocations

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Next: [487-load-testing.md](487-load-testing.md)
- Related: [488-test-ci.md](488-test-ci.md)
