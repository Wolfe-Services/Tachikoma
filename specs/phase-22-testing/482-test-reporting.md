# 482 - Test Reporting

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 482
**Status:** Planned
**Dependencies:** 471-test-harness, 481-test-coverage
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive test reporting that generates JUnit XML, HTML, and JSON reports for both Rust and TypeScript tests, enabling integration with CI systems and providing detailed test result visualization.

---

## Acceptance Criteria

- [x] JUnit XML reports for CI integration
- [x] HTML reports for human-readable results
- [x] JSON reports for programmatic processing
- [x] Test timing and performance metrics
- [x] Failure details with stack traces
- [x] Historical trend tracking

---

## Implementation Details

### 1. Rust Test Reporting

Create `crates/tachikoma-test-harness/src/reporting/mod.rs`:

```rust
//! Test reporting utilities.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Test result status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Ignored,
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub classname: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub stack_trace: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Test suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub tests: u32,
    pub failures: u32,
    pub errors: u32,
    pub skipped: u32,
    pub time_ms: u64,
    pub timestamp: String,
    pub test_cases: Vec<TestResult>,
}

/// Full test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub name: String,
    pub tests: u32,
    pub failures: u32,
    pub errors: u32,
    pub skipped: u32,
    pub time_ms: u64,
    pub suites: Vec<TestSuite>,
}

impl TestReport {
    /// Create a new empty report
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tests: 0,
            failures: 0,
            errors: 0,
            skipped: 0,
            time_ms: 0,
            suites: Vec::new(),
        }
    }

    /// Add a test suite
    pub fn add_suite(&mut self, suite: TestSuite) {
        self.tests += suite.tests;
        self.failures += suite.failures;
        self.errors += suite.errors;
        self.skipped += suite.skipped;
        self.time_ms += suite.time_ms;
        self.suites.push(suite);
    }

    /// Export to JUnit XML format
    pub fn to_junit_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<testsuites name="{}" tests="{}" failures="{}" errors="{}" skipped="{}" time="{:.3}">"#,
            escape_xml(&self.name),
            self.tests,
            self.failures,
            self.errors,
            self.skipped,
            self.time_ms as f64 / 1000.0
        ));
        xml.push('\n');

        for suite in &self.suites {
            xml.push_str(&format!(
                r#"  <testsuite name="{}" tests="{}" failures="{}" errors="{}" skipped="{}" time="{:.3}" timestamp="{}">"#,
                escape_xml(&suite.name),
                suite.tests,
                suite.failures,
                suite.errors,
                suite.skipped,
                suite.time_ms as f64 / 1000.0,
                suite.timestamp
            ));
            xml.push('\n');

            for test in &suite.test_cases {
                xml.push_str(&format!(
                    r#"    <testcase name="{}" classname="{}" time="{:.3}">"#,
                    escape_xml(&test.name),
                    escape_xml(&test.classname),
                    test.duration_ms as f64 / 1000.0
                ));
                xml.push('\n');

                match test.status {
                    TestStatus::Failed => {
                        xml.push_str(&format!(
                            r#"      <failure message="{}">{}</failure>"#,
                            escape_xml(test.message.as_deref().unwrap_or("")),
                            escape_xml(test.stack_trace.as_deref().unwrap_or(""))
                        ));
                        xml.push('\n');
                    }
                    TestStatus::Skipped | TestStatus::Ignored => {
                        xml.push_str("      <skipped/>\n");
                    }
                    TestStatus::Passed => {}
                }

                if let Some(stdout) = &test.stdout {
                    xml.push_str(&format!(
                        "      <system-out>{}</system-out>\n",
                        escape_xml(stdout)
                    ));
                }
                if let Some(stderr) = &test.stderr {
                    xml.push_str(&format!(
                        "      <system-err>{}</system-err>\n",
                        escape_xml(stderr)
                    ));
                }

                xml.push_str("    </testcase>\n");
            }

            xml.push_str("  </testsuite>\n");
        }

        xml.push_str("</testsuites>\n");
        xml
    }

    /// Export to JSON format
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    /// Export to HTML format
    pub fn to_html(&self) -> String {
        let pass_rate = if self.tests > 0 {
            ((self.tests - self.failures - self.errors) as f64 / self.tests as f64) * 100.0
        } else {
            100.0
        };

        let status_class = if self.failures > 0 || self.errors > 0 {
            "failed"
        } else {
            "passed"
        };

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Report - {}</title>
    <style>
        body {{ font-family: -apple-system, sans-serif; margin: 40px; }}
        .summary {{ background: #f5f5f5; padding: 20px; border-radius: 8px; margin-bottom: 20px; }}
        .summary.passed {{ border-left: 4px solid #4caf50; }}
        .summary.failed {{ border-left: 4px solid #f44336; }}
        .stat {{ display: inline-block; margin-right: 30px; }}
        .stat-value {{ font-size: 24px; font-weight: bold; }}
        .stat-label {{ color: #666; font-size: 12px; }}
        .suite {{ margin: 20px 0; }}
        .suite-header {{ font-size: 18px; font-weight: bold; margin-bottom: 10px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }}
        .passed {{ color: #4caf50; }}
        .failed {{ color: #f44336; }}
        .skipped {{ color: #ff9800; }}
        .duration {{ color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <h1>Test Report</h1>
    <div class="summary {}">
        <div class="stat">
            <div class="stat-value">{}</div>
            <div class="stat-label">Total Tests</div>
        </div>
        <div class="stat">
            <div class="stat-value passed">{}</div>
            <div class="stat-label">Passed</div>
        </div>
        <div class="stat">
            <div class="stat-value failed">{}</div>
            <div class="stat-label">Failed</div>
        </div>
        <div class="stat">
            <div class="stat-value skipped">{}</div>
            <div class="stat-label">Skipped</div>
        </div>
        <div class="stat">
            <div class="stat-value">{:.1}%</div>
            <div class="stat-label">Pass Rate</div>
        </div>
        <div class="stat">
            <div class="stat-value">{:.2}s</div>
            <div class="stat-label">Duration</div>
        </div>
    </div>
    {}
</body>
</html>"#,
            self.name,
            status_class,
            self.tests,
            self.tests - self.failures - self.errors - self.skipped,
            self.failures + self.errors,
            self.skipped,
            pass_rate,
            self.time_ms as f64 / 1000.0,
            self.suites
                .iter()
                .map(|s| suite_to_html(s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

fn suite_to_html(suite: &TestSuite) -> String {
    let tests_html: String = suite
        .test_cases
        .iter()
        .map(|t| {
            let status_class = match t.status {
                TestStatus::Passed => "passed",
                TestStatus::Failed => "failed",
                TestStatus::Skipped | TestStatus::Ignored => "skipped",
            };
            format!(
                r#"<tr>
                    <td>{}</td>
                    <td class="{}">{:?}</td>
                    <td class="duration">{}ms</td>
                </tr>"#,
                t.name, status_class, t.status, t.duration_ms
            )
        })
        .collect();

    format!(
        r#"<div class="suite">
            <div class="suite-header">{} ({} tests)</div>
            <table>
                <thead><tr><th>Test</th><th>Status</th><th>Duration</th></tr></thead>
                <tbody>{}</tbody>
            </table>
        </div>"#,
        suite.name, suite.tests, tests_html
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
```

### 2. Rust Test Reporter Integration

Add to `Cargo.toml`:

```toml
[workspace.metadata.nextest]
# Use nextest for better test reporting
```

Create `.config/nextest.toml`:

```toml
[profile.default]
retries = 0
slow-timeout = { period = "60s", terminate-after = 2 }
fail-fast = false

[profile.ci]
retries = 2
fail-fast = true

[[profile.ci.overrides]]
filter = 'test(flaky)'
retries = 3
```

### 3. TypeScript Test Reporting

Update `web/vitest.config.ts`:

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    // ... other config

    // Reporter configuration
    reporters: [
      'default',
      'json',
      'junit',
      'html',
    ],

    outputFile: {
      json: './test-results/results.json',
      junit: './test-results/junit.xml',
      html: './test-results/index.html',
    },

    // Include timing information
    benchmark: {
      include: ['**/*.bench.{js,ts}'],
      reporters: ['default', 'json'],
      outputFile: './test-results/benchmark.json',
    },
  },
});
```

### 4. Combined Reporting Script

Create `scripts/test-report.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

OUTPUT_DIR="test-results"
mkdir -p "$OUTPUT_DIR"

echo "Running Rust tests with reporting..."
cargo nextest run --profile ci \
    --message-format json \
    > "$OUTPUT_DIR/rust-results.json" 2>&1 || true

# Convert to JUnit
cargo nextest run --profile ci \
    --message-format junit \
    > "$OUTPUT_DIR/rust-junit.xml" 2>&1 || true

echo "Running TypeScript tests with reporting..."
cd web && npm test -- --run --reporter json --reporter junit
cd ..
mv web/test-results/* "$OUTPUT_DIR/"

echo "Test reports generated in $OUTPUT_DIR"
```

---

## Testing Requirements

1. JUnit XML validates against schema
2. HTML reports render correctly in browsers
3. JSON reports parse correctly
4. Test timing is accurate
5. Failure details include useful context

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [481-test-coverage.md](481-test-coverage.md)
- Next: [483-e2e-framework.md](483-e2e-framework.md)
- Related: [488-test-ci.md](488-test-ci.md)
