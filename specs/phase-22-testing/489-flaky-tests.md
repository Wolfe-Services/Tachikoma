# 489 - Flaky Test Handling

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 489
**Status:** Planned
**Dependencies:** 471-test-harness, 488-test-ci
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement infrastructure to detect, track, quarantine, and fix flaky tests to maintain CI reliability and developer confidence in the test suite.

---

## Acceptance Criteria

- [x] Automatic flaky test detection via retries
- [x] Quarantine mechanism for known flaky tests
- [x] Flaky test tracking and reporting
- [x] Alerts when flaky tests exceed threshold
- [x] Documentation for fixing flaky tests
- [x] Regular flaky test review process

---

## Implementation Details

### 1. Rust Flaky Test Configuration

Create `.config/nextest.toml`:

```toml
[profile.default]
retries = 0
slow-timeout = { period = "60s", terminate-after = 2 }
fail-fast = false
test-threads = "num-cpus"

[profile.ci]
retries = 2
fail-fast = false
slow-timeout = { period = "120s", terminate-after = 2 }

# Flaky test profile with more retries
[profile.flaky-detection]
retries = 5
fail-fast = false

# Override for known flaky tests
[[profile.ci.overrides]]
filter = 'test(flaky::)'
retries = 3
slow-timeout = { period = "180s", terminate-after = 3 }

[[profile.ci.overrides]]
filter = 'test(::network::)'
retries = 2

[[profile.ci.overrides]]
filter = 'test(::timeout::)'
retries = 2
slow-timeout = { period = "300s" }
```

### 2. Flaky Test Tracking System

Create `crates/tachikoma-test-harness/src/flaky/mod.rs`:

```rust
//! Flaky test detection and tracking.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Record of a flaky test occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestRecord {
    pub test_name: String,
    pub module: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: u32,
    pub pass_count: u32,
    pub fail_count: u32,
    pub failure_messages: Vec<String>,
    pub status: FlakyTestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlakyTestStatus {
    Active,
    Quarantined,
    Fixed,
    Investigating,
}

impl FlakyTestRecord {
    pub fn new(test_name: String, module: String) -> Self {
        let now = Utc::now();
        Self {
            test_name,
            module,
            first_seen: now,
            last_seen: now,
            occurrence_count: 1,
            pass_count: 0,
            fail_count: 0,
            failure_messages: Vec::new(),
            status: FlakyTestStatus::Active,
        }
    }

    pub fn record_pass(&mut self) {
        self.pass_count += 1;
        self.occurrence_count += 1;
        self.last_seen = Utc::now();
    }

    pub fn record_fail(&mut self, message: Option<String>) {
        self.fail_count += 1;
        self.occurrence_count += 1;
        self.last_seen = Utc::now();
        if let Some(msg) = message {
            if self.failure_messages.len() < 10 {
                self.failure_messages.push(msg);
            }
        }
    }

    pub fn flaky_rate(&self) -> f64 {
        if self.occurrence_count == 0 {
            return 0.0;
        }
        let flaky = self.occurrence_count as f64 - self.pass_count.max(self.fail_count) as f64;
        flaky / self.occurrence_count as f64
    }

    pub fn is_flaky(&self) -> bool {
        self.occurrence_count >= 3 && self.pass_count > 0 && self.fail_count > 0
    }
}

/// Flaky test database
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FlakyTestDb {
    pub tests: HashMap<String, FlakyTestRecord>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl FlakyTestDb {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    pub fn record_test_result(&mut self, test_name: &str, module: &str, passed: bool, message: Option<String>) {
        let key = format!("{}::{}", module, test_name);

        let record = self.tests
            .entry(key)
            .or_insert_with(|| FlakyTestRecord::new(test_name.to_string(), module.to_string()));

        if passed {
            record.record_pass();
        } else {
            record.record_fail(message);
        }

        self.last_updated = Some(Utc::now());
    }

    pub fn get_flaky_tests(&self) -> Vec<&FlakyTestRecord> {
        self.tests.values().filter(|t| t.is_flaky()).collect()
    }

    pub fn get_quarantined(&self) -> Vec<&FlakyTestRecord> {
        self.tests.values()
            .filter(|t| t.status == FlakyTestStatus::Quarantined)
            .collect()
    }

    pub fn quarantine(&mut self, test_key: &str) {
        if let Some(record) = self.tests.get_mut(test_key) {
            record.status = FlakyTestStatus::Quarantined;
        }
    }

    pub fn mark_fixed(&mut self, test_key: &str) {
        if let Some(record) = self.tests.get_mut(test_key) {
            record.status = FlakyTestStatus::Fixed;
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# Flaky Test Report\n\n");

        let flaky = self.get_flaky_tests();
        if flaky.is_empty() {
            report.push_str("No flaky tests detected.\n");
            return report;
        }

        report.push_str(&format!("## Summary\n\nTotal flaky tests: {}\n\n", flaky.len()));

        report.push_str("## Active Flaky Tests\n\n");
        report.push_str("| Test | Module | Flaky Rate | Pass/Fail | Status |\n");
        report.push_str("|------|--------|------------|-----------|--------|\n");

        for test in &flaky {
            report.push_str(&format!(
                "| {} | {} | {:.1}% | {}/{} | {:?} |\n",
                test.test_name,
                test.module,
                test.flaky_rate() * 100.0,
                test.pass_count,
                test.fail_count,
                test.status
            ));
        }

        report
    }
}

/// Macro to mark a test as potentially flaky
#[macro_export]
macro_rules! flaky_test {
    ($name:ident, $body:expr) => {
        #[test]
        #[allow(non_snake_case)]
        fn $name() {
            // Tag for nextest filtering
            let _flaky_marker = "flaky";
            $body
        }
    };
}

/// Macro to skip quarantined tests
#[macro_export]
macro_rules! quarantined_test {
    ($name:ident, $reason:expr, $body:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            $body
        }
    };
}
```

### 3. TypeScript Flaky Test Configuration

Create `web/vitest.flaky.config.ts`:

```typescript
import { defineConfig } from 'vitest/config';
import baseConfig from './vitest.config';

export default defineConfig({
  ...baseConfig,
  test: {
    ...baseConfig.test,
    retry: 3, // Retry failed tests

    // Hook to track flaky tests
    onConsoleLog(log) {
      // Could send to tracking service
      return false;
    },

    reporters: [
      'default',
      ['json', { outputFile: 'test-results/flaky-report.json' }],
    ],
  },
});
```

Create `web/src/test/flaky/tracker.ts`:

```typescript
/**
 * Flaky test tracking for TypeScript tests.
 */

interface FlakyTestRecord {
  testName: string;
  file: string;
  passCount: number;
  failCount: number;
  lastSeen: string;
  status: 'active' | 'quarantined' | 'fixed';
}

class FlakyTestTracker {
  private records: Map<string, FlakyTestRecord> = new Map();

  record(testName: string, file: string, passed: boolean): void {
    const key = `${file}::${testName}`;
    const existing = this.records.get(key) || {
      testName,
      file,
      passCount: 0,
      failCount: 0,
      lastSeen: new Date().toISOString(),
      status: 'active' as const,
    };

    if (passed) {
      existing.passCount++;
    } else {
      existing.failCount++;
    }
    existing.lastSeen = new Date().toISOString();

    this.records.set(key, existing);
  }

  isFlaky(key: string): boolean {
    const record = this.records.get(key);
    if (!record) return false;
    return record.passCount > 0 && record.failCount > 0;
  }

  getFlakyTests(): FlakyTestRecord[] {
    return Array.from(this.records.values()).filter(r =>
      r.passCount > 0 && r.failCount > 0
    );
  }

  toJSON(): string {
    return JSON.stringify(Array.from(this.records.values()), null, 2);
  }
}

export const flakyTracker = new FlakyTestTracker();

/**
 * Mark a test as known flaky with automatic retry
 */
export function flakyTest(
  name: string,
  fn: () => void | Promise<void>,
  options: { maxRetries?: number; reason?: string } = {}
): void {
  const { maxRetries = 3, reason } = options;

  // vitest will handle retries based on config
  // This is mainly for documentation
  return (globalThis as any).it(
    `${name}${reason ? ` (flaky: ${reason})` : ''}`,
    fn
  );
}

/**
 * Mark a test as quarantined (skipped)
 */
export function quarantinedTest(
  name: string,
  reason: string,
  fn: () => void | Promise<void>
): void {
  return (globalThis as any).it.skip(`${name} (quarantined: ${reason})`, fn);
}
```

### 4. CI Flaky Test Workflow

Create `.github/workflows/flaky-tests.yml`:

```yaml
name: Flaky Test Detection

on:
  schedule:
    - cron: '0 3 * * *'  # Daily at 3 AM
  workflow_dispatch:

jobs:
  detect-flaky:
    name: Detect Flaky Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install nextest
        uses: taiki-e/install-action@nextest

      - name: Run tests multiple times
        run: |
          for i in {1..10}; do
            echo "Run $i of 10"
            cargo nextest run --profile flaky-detection \
              --message-format json \
              >> flaky-results.jsonl 2>&1 || true
          done

      - name: Analyze results
        run: |
          python scripts/analyze-flaky.py flaky-results.jsonl > flaky-report.md

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: flaky-test-report
          path: flaky-report.md

      - name: Create issue if new flaky tests
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('flaky-report.md', 'utf8');

            if (report.includes('New flaky tests detected')) {
              github.rest.issues.create({
                owner: context.repo.owner,
                repo: context.repo.repo,
                title: 'New Flaky Tests Detected',
                body: report,
                labels: ['flaky-test', 'automated']
              });
            }
```

### 5. Flaky Test Analysis Script

Create `scripts/analyze-flaky.py`:

```python
#!/usr/bin/env python3
"""Analyze test results to detect flaky tests."""

import json
import sys
from collections import defaultdict
from datetime import datetime

def analyze_flaky_tests(results_file: str) -> str:
    """Analyze test results and generate a flaky test report."""
    test_results = defaultdict(lambda: {"pass": 0, "fail": 0, "messages": []})

    with open(results_file) as f:
        for line in f:
            try:
                result = json.loads(line)
                if result.get("type") == "test":
                    name = result["name"]
                    status = result["status"]

                    if status == "passed":
                        test_results[name]["pass"] += 1
                    elif status == "failed":
                        test_results[name]["fail"] += 1
                        if "message" in result:
                            test_results[name]["messages"].append(result["message"])
            except json.JSONDecodeError:
                continue

    # Find flaky tests (both passed and failed)
    flaky_tests = {
        name: data for name, data in test_results.items()
        if data["pass"] > 0 and data["fail"] > 0
    }

    # Generate report
    report = ["# Flaky Test Analysis Report", ""]
    report.append(f"Generated: {datetime.now().isoformat()}")
    report.append(f"Total tests analyzed: {len(test_results)}")
    report.append(f"Flaky tests found: {len(flaky_tests)}")
    report.append("")

    if flaky_tests:
        report.append("## New flaky tests detected")
        report.append("")
        report.append("| Test | Pass Count | Fail Count | Flaky Rate |")
        report.append("|------|------------|------------|------------|")

        for name, data in sorted(flaky_tests.items(),
                                  key=lambda x: x[1]["fail"],
                                  reverse=True):
            total = data["pass"] + data["fail"]
            flaky_rate = min(data["pass"], data["fail"]) / total * 100
            report.append(f"| {name} | {data['pass']} | {data['fail']} | {flaky_rate:.1f}% |")

        report.append("")
        report.append("## Failure Messages")
        for name, data in flaky_tests.items():
            if data["messages"]:
                report.append(f"\n### {name}")
                for msg in data["messages"][:3]:  # First 3 messages
                    report.append(f"```\n{msg[:500]}\n```")
    else:
        report.append("No flaky tests detected in this run.")

    return "\n".join(report)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: analyze-flaky.py <results-file>")
        sys.exit(1)

    print(analyze_flaky_tests(sys.argv[1]))
```

---

## Testing Requirements

1. Flaky detection runs reliably in CI
2. Quarantined tests don't block merges
3. Reports accurately identify flaky tests
4. Alert threshold configurable
5. Historical tracking enables trend analysis

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [488-test-ci.md](488-test-ci.md)
- Next: [490-test-docs.md](490-test-docs.md)
- Related: [482-test-reporting.md](482-test-reporting.md)
