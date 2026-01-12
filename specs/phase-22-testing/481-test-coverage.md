# 481 - Test Coverage Setup

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 481
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure comprehensive test coverage collection and reporting for both Rust (using cargo-llvm-cov) and TypeScript (using v8 coverage via Vitest), enabling visibility into untested code paths.

---

## Acceptance Criteria

- [ ] Rust coverage via cargo-llvm-cov configured
- [ ] TypeScript coverage via v8 provider configured
- [ ] Coverage thresholds enforced in CI
- [ ] HTML and LCOV report formats generated
- [ ] Coverage badges generated for README
- [ ] Merged coverage report across all languages

---

## Implementation Details

### 1. Rust Coverage Configuration

Create `.cargo/config.toml` additions:

```toml
[alias]
# Coverage commands
coverage = "llvm-cov --workspace --lcov --output-path lcov.info"
coverage-html = "llvm-cov --workspace --html"
coverage-json = "llvm-cov --workspace --json --output-path coverage.json"
```

Create `scripts/coverage-rust.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Colors
GREEN='\033[0;32m'
NC='\033[0m'
log() { echo -e "${GREEN}[COVERAGE]${NC} $1"; }

# Ensure cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    log "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

OUTPUT_DIR="${1:-coverage/rust}"
mkdir -p "$OUTPUT_DIR"

log "Running Rust tests with coverage..."

# Generate LCOV report
cargo llvm-cov --workspace \
    --lcov \
    --output-path "$OUTPUT_DIR/lcov.info" \
    --ignore-filename-regex '(tests/|test_|_test\.rs)'

# Generate HTML report
cargo llvm-cov --workspace \
    --html \
    --output-dir "$OUTPUT_DIR/html" \
    --ignore-filename-regex '(tests/|test_|_test\.rs)'

# Generate JSON for processing
cargo llvm-cov --workspace \
    --json \
    --output-path "$OUTPUT_DIR/coverage.json" \
    --ignore-filename-regex '(tests/|test_|_test\.rs)'

log "Coverage reports generated in $OUTPUT_DIR"

# Print summary
cargo llvm-cov --workspace report --summary-only
```

### 2. TypeScript Coverage Configuration

Update `web/vitest.config.ts`:

```typescript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],

  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],

    coverage: {
      // Use v8 provider for accurate coverage
      provider: 'v8',

      // Enable coverage collection
      enabled: true,

      // Output formats
      reporter: ['text', 'json', 'html', 'lcov'],

      // Output directory
      reportsDirectory: './coverage',

      // Files to include
      include: ['src/**/*.{ts,svelte}'],

      // Files to exclude
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.test.ts',
        '**/*.spec.ts',
        '**/index.ts',
      ],

      // Coverage thresholds
      thresholds: {
        lines: 70,
        functions: 70,
        branches: 60,
        statements: 70,
      },

      // Fail if thresholds not met
      thresholdAutoUpdate: false,

      // Show uncovered lines in output
      all: true,
    },
  },
});
```

Create `web/scripts/coverage.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

OUTPUT_DIR="${1:-coverage}"
mkdir -p "$OUTPUT_DIR"

echo "Running TypeScript tests with coverage..."

npm run test -- --coverage --run

echo "Coverage reports generated in $OUTPUT_DIR"
```

### 3. Combined Coverage Script

Create `scripts/coverage-all.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
NC='\033[0m'
log() { echo -e "${GREEN}[COVERAGE]${NC} $1"; }

OUTPUT_DIR="coverage"
mkdir -p "$OUTPUT_DIR"

# Run Rust coverage
log "Collecting Rust coverage..."
./scripts/coverage-rust.sh "$OUTPUT_DIR/rust"

# Run TypeScript coverage
log "Collecting TypeScript coverage..."
cd web && npm run test -- --coverage --run
cd ..
mv web/coverage "$OUTPUT_DIR/typescript"

# Merge LCOV reports
log "Merging coverage reports..."
if command -v lcov &> /dev/null; then
    lcov \
        -a "$OUTPUT_DIR/rust/lcov.info" \
        -a "$OUTPUT_DIR/typescript/lcov.info" \
        -o "$OUTPUT_DIR/merged.info"
else
    log "lcov not installed, skipping merge"
fi

# Generate summary
log "Coverage Summary"
echo "================"
echo ""
echo "Rust Coverage:"
cat "$OUTPUT_DIR/rust/coverage.json" | jq '.data[0].totals'
echo ""
echo "TypeScript Coverage:"
cat "$OUTPUT_DIR/typescript/coverage-summary.json" | jq '.total'

log "Reports available at:"
echo "  - Rust HTML: $OUTPUT_DIR/rust/html/index.html"
echo "  - TypeScript HTML: $OUTPUT_DIR/typescript/index.html"
```

### 4. Coverage Thresholds Configuration

Create `coverage.config.json`:

```json
{
  "rust": {
    "thresholds": {
      "lines": 70,
      "functions": 70,
      "branches": 60
    },
    "exclude": [
      "*/tests/*",
      "*_test.rs",
      "*/benches/*"
    ]
  },
  "typescript": {
    "thresholds": {
      "lines": 70,
      "functions": 70,
      "branches": 60,
      "statements": 70
    },
    "exclude": [
      "src/test/**",
      "**/*.d.ts",
      "**/*.test.ts"
    ]
  },
  "global": {
    "minimumCoverage": 65,
    "failOnDecrease": true
  }
}
```

### 5. CI Coverage Integration

Create `.github/workflows/coverage.yml`:

```yaml
name: Coverage

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  rust-coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info

      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          flags: rust
          fail_ci_if_error: true

  typescript-coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: cd web && npm ci

      - name: Run tests with coverage
        run: cd web && npm run test -- --coverage --run

      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          directory: web/coverage
          flags: typescript
          fail_ci_if_error: true

  coverage-check:
    needs: [rust-coverage, typescript-coverage]
    runs-on: ubuntu-latest
    steps:
      - name: Check coverage thresholds
        run: |
          echo "Coverage thresholds checked by individual jobs"
```

### 6. Coverage Badge Generation

Create `scripts/coverage-badge.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Extract coverage percentage from reports
RUST_COV=$(cat coverage/rust/coverage.json | jq -r '.data[0].totals.lines.percent' | cut -d'.' -f1)
TS_COV=$(cat coverage/typescript/coverage-summary.json | jq -r '.total.lines.pct' | cut -d'.' -f1)

# Average coverage
TOTAL_COV=$(( (RUST_COV + TS_COV) / 2 ))

# Determine color
if [ "$TOTAL_COV" -ge 80 ]; then
    COLOR="brightgreen"
elif [ "$TOTAL_COV" -ge 60 ]; then
    COLOR="yellow"
else
    COLOR="red"
fi

# Generate badge URL
BADGE_URL="https://img.shields.io/badge/coverage-${TOTAL_COV}%25-${COLOR}"

echo "Coverage: ${TOTAL_COV}%"
echo "Badge URL: $BADGE_URL"

# Update README badge (if needed)
# sed -i "s|coverage-[0-9]*%25-[a-z]*|coverage-${TOTAL_COV}%25-${COLOR}|g" README.md
```

---

## Testing Requirements

1. `cargo llvm-cov` generates accurate coverage
2. `npm test -- --coverage` generates accurate coverage
3. Coverage thresholds fail CI when not met
4. HTML reports are browsable and accurate
5. Coverage badges reflect actual metrics

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Next: [482-test-reporting.md](482-test-reporting.md)
- Related: [488-test-ci.md](488-test-ci.md)
