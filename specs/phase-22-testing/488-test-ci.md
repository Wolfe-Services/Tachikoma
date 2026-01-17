# 488 - Test CI Integration

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 488
**Status:** Planned
**Dependencies:** 471-test-harness, 009-ci-pipeline
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Configure comprehensive CI/CD integration for all test types, ensuring automated test execution, reporting, and quality gates across pull requests and main branch commits.

---

## Acceptance Criteria

- [x] All test types run on pull requests
- [x] Parallel test execution optimized
- [x] Test results and coverage reported
- [x] Quality gates block failing PRs
- [x] Caching reduces CI time
- [x] Matrix builds cover all platforms

---

## Implementation Details

### 1. Main Test Workflow

Create `.github/workflows/tests.yml`:

```yaml
name: Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Fast checks first
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: web/package-lock.json

      - name: Install web dependencies
        run: cd web && npm ci

      - name: Lint TypeScript
        run: cd web && npm run lint

      - name: Type check
        run: cd web && npm run check

  # Rust tests
  rust-tests:
    name: Rust Tests
    runs-on: ${{ matrix.os }}
    needs: lint
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
      fail-fast: false

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.os }}

      - name: Run unit tests
        run: cargo test --workspace --lib

      - name: Run integration tests
        run: cargo test --workspace --test '*'

      - name: Run doc tests
        run: cargo test --workspace --doc

  # Rust coverage (Linux only)
  rust-coverage:
    name: Rust Coverage
    runs-on: ubuntu-latest
    needs: rust-tests
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Generate coverage
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info

      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          flags: rust
          fail_ci_if_error: true

  # TypeScript tests
  typescript-tests:
    name: TypeScript Tests
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: web/package-lock.json

      - name: Install dependencies
        run: cd web && npm ci

      - name: Run tests
        run: cd web && npm test -- --run --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          directory: web/coverage
          flags: typescript
          fail_ci_if_error: true

  # E2E tests
  e2e-tests:
    name: E2E Tests
    runs-on: ${{ matrix.os }}
    needs: [rust-tests, typescript-tests]
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
      fail-fast: false

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Install dependencies
        run: npm ci

      - name: Build Electron app
        run: npm run build:electron

      - name: Install Playwright
        run: npx playwright install --with-deps

      - name: Run E2E tests
        run: npm run test:e2e
        env:
          CI: true

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-${{ matrix.os }}
          path: |
            e2e/playwright-report/
            e2e/test-results/

  # Quality gates
  quality-gate:
    name: Quality Gate
    runs-on: ubuntu-latest
    needs: [rust-coverage, typescript-tests, e2e-tests]
    steps:
      - name: Check all tests passed
        run: echo "All tests passed!"
```

### 2. Property and Snapshot Tests

Create `.github/workflows/property-tests.yml`:

```yaml
name: Property & Snapshot Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  property-tests:
    name: Property Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Run proptest (extended)
        run: PROPTEST_CASES=ci cargo test --workspace --features proptest
        env:
          PROPTEST_CASES: '1024'

      - name: Run fast-check
        working-directory: web
        run: npm run test:property

  snapshot-tests:
    name: Snapshot Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Check Rust snapshots
        run: cargo insta test --check

      - name: Install web dependencies
        run: cd web && npm ci

      - name: Check TypeScript snapshots
        run: cd web && npm test -- --run
```

### 3. Scheduled Tests

Create `.github/workflows/scheduled-tests.yml`:

```yaml
name: Scheduled Tests

on:
  schedule:
    # Run nightly at 2 AM UTC
    - cron: '0 2 * * *'
  workflow_dispatch:

jobs:
  full-test-suite:
    name: Full Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: llvm-tools-preview

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install all dependencies
        run: |
          npm ci
          cd web && npm ci
          cargo install cargo-llvm-cov cargo-nextest

      - name: Run all Rust tests with coverage
        run: cargo llvm-cov nextest --workspace --all-features

      - name: Run all TypeScript tests
        run: cd web && npm test -- --run --coverage

      - name: Run E2E tests
        run: npm run test:e2e

      - name: Run property tests (extended)
        run: PROPTEST_CASES=10000 cargo test --workspace proptest

  load-tests:
    name: Load Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install k6
        run: |
          sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6

      - name: Run load tests
        run: ./scripts/load-test.sh load all

  benchmarks:
    name: Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Run benchmarks
        run: cargo bench --workspace -- --save-baseline nightly

      - name: Compare with previous
        run: |
          cargo bench --workspace -- --baseline nightly --load-baseline previous || true

      - name: Upload benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: benchmarks
          path: target/criterion/
```

### 4. Test Caching Configuration

Create `.github/actions/test-cache/action.yml`:

```yaml
name: Test Cache
description: Cache test dependencies and artifacts

inputs:
  key-prefix:
    description: Cache key prefix
    default: 'test'

runs:
  using: composite
  steps:
    - name: Cache Rust dependencies
      uses: Swatinem/rust-cache@v2
      with:
        prefix-key: ${{ inputs.key-prefix }}-rust

    - name: Cache Node modules
      uses: actions/cache@v4
      with:
        path: |
          ~/.npm
          web/node_modules
        key: ${{ inputs.key-prefix }}-node-${{ hashFiles('**/package-lock.json') }}
        restore-keys: |
          ${{ inputs.key-prefix }}-node-

    - name: Cache Playwright browsers
      uses: actions/cache@v4
      with:
        path: ~/.cache/ms-playwright
        key: ${{ inputs.key-prefix }}-playwright-${{ hashFiles('**/package-lock.json') }}

    - name: Cache test fixtures
      uses: actions/cache@v4
      with:
        path: tests/fixtures/.cache
        key: ${{ inputs.key-prefix }}-fixtures-${{ hashFiles('tests/fixtures/**') }}
```

### 5. Test Status Badge Configuration

Add to `README.md`:

```markdown
## Test Status

[![Tests](https://github.com/tachikoma/tachikoma/actions/workflows/tests.yml/badge.svg)](https://github.com/tachikoma/tachikoma/actions/workflows/tests.yml)
[![Coverage](https://codecov.io/gh/tachikoma/tachikoma/branch/main/graph/badge.svg)](https://codecov.io/gh/tachikoma/tachikoma)
[![E2E](https://github.com/tachikoma/tachikoma/actions/workflows/tests.yml/badge.svg?branch=main&event=push)](https://github.com/tachikoma/tachikoma/actions/workflows/tests.yml)
```

---

## Testing Requirements

1. All tests run on every PR
2. CI completes in under 15 minutes for PRs
3. Coverage reports upload to Codecov
4. Quality gates block merging on failures
5. Caching significantly reduces build times

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [009-ci-pipeline.md](../phase-00-setup/009-ci-pipeline.md)
- Next: [489-flaky-tests.md](489-flaky-tests.md)
- Related: [481-test-coverage.md](481-test-coverage.md)
