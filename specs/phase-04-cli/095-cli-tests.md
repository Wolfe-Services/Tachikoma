# Spec 095: CLI Tests

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 095
- **Status**: Planned
- **Dependencies**: 076-cli-crate through 094-cli-man
- **Estimated Context**: ~12%

## Objective

Implement comprehensive testing infrastructure for the CLI, including unit tests, integration tests, snapshot tests, and test utilities for CLI-specific scenarios.

## Acceptance Criteria

- [ ] Unit tests for all CLI components
- [ ] Integration tests for all commands
- [ ] Snapshot tests for output formatting
- [ ] Test utilities for CLI testing
- [ ] Mock infrastructure for backends/tools
- [ ] CI test configuration
- [ ] Test coverage reporting
- [ ] Performance benchmarks

## Implementation Details

### tests/common/mod.rs

```rust
//! Common test utilities for CLI testing.

use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

/// Test context with temporary directory
pub struct TestContext {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("tachikoma.toml");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Create a minimal config file
    pub fn with_config(self, config: &str) -> Self {
        std::fs::write(&self.config_path, config).expect("Failed to write config");
        self
    }

    /// Create a default config
    pub fn with_default_config(self) -> Self {
        let config = r#"
[project]
name = "test-project"
version = "0.1.0"

[agent]
model = "test-model"
"#;
        self.with_config(config)
    }

    /// Get path to temp directory
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a command configured for this context
    pub fn command(&self) -> Command {
        let mut cmd = Command::cargo_bin("tachikoma").expect("Binary not found");
        cmd.current_dir(self.path())
            .env("TACHIKOMA_CONFIG", &self.config_path)
            .env("NO_COLOR", "1"); // Disable colors for predictable output
        cmd
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Assert helpers for CLI output
pub trait OutputAssertions {
    fn assert_success(&self);
    fn assert_failure(&self);
    fn assert_stdout_contains(&self, text: &str);
    fn assert_stderr_contains(&self, text: &str);
    fn assert_exit_code(&self, code: i32);
}

impl OutputAssertions for Output {
    fn assert_success(&self) {
        assert!(
            self.status.success(),
            "Command failed with status: {}\nstderr: {}",
            self.status,
            String::from_utf8_lossy(&self.stderr)
        );
    }

    fn assert_failure(&self) {
        assert!(
            !self.status.success(),
            "Command succeeded unexpectedly\nstdout: {}",
            String::from_utf8_lossy(&self.stdout)
        );
    }

    fn assert_stdout_contains(&self, text: &str) {
        let stdout = String::from_utf8_lossy(&self.stdout);
        assert!(
            stdout.contains(text),
            "stdout did not contain '{}'\nstdout: {}",
            text,
            stdout
        );
    }

    fn assert_stderr_contains(&self, text: &str) {
        let stderr = String::from_utf8_lossy(&self.stderr);
        assert!(
            stderr.contains(text),
            "stderr did not contain '{}'\nstderr: {}",
            text,
            stderr
        );
    }

    fn assert_exit_code(&self, code: i32) {
        assert_eq!(
            self.status.code(),
            Some(code),
            "Expected exit code {}, got {:?}",
            code,
            self.status.code()
        );
    }
}

/// Mock backend server for testing
pub struct MockBackend {
    server: mockito::ServerGuard,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            server: mockito::Server::new(),
        }
    }

    pub fn url(&self) -> String {
        self.server.url()
    }

    /// Mock a successful completion response
    pub fn mock_completion(&mut self, response: &str) -> mockito::Mock {
        self.server
            .mock("POST", "/v1/messages")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"content": [{{"type": "text", "text": "{response}"}}]}}"#
            ))
            .create()
    }

    /// Mock an error response
    pub fn mock_error(&mut self, status: usize, message: &str) -> mockito::Mock {
        self.server
            .mock("POST", "/v1/messages")
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(format!(r#"{{"error": {{"message": "{message}"}}}}"#))
            .create()
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON output assertions
pub mod json {
    use serde_json::Value;

    pub fn parse_output(output: &[u8]) -> Value {
        serde_json::from_slice(output).expect("Failed to parse JSON output")
    }

    pub fn assert_success(json: &Value) {
        assert_eq!(
            json.get("success").and_then(|v| v.as_bool()),
            Some(true),
            "Expected success: true in JSON output"
        );
    }

    pub fn assert_error(json: &Value, code: &str) {
        assert_eq!(
            json.get("success").and_then(|v| v.as_bool()),
            Some(false),
            "Expected success: false in JSON output"
        );
        assert_eq!(
            json.get("error")
                .and_then(|e| e.get("code"))
                .and_then(|c| c.as_str()),
            Some(code),
            "Expected error code: {code}"
        );
    }
}

/// Snapshot testing utilities
pub mod snapshot {
    use std::path::Path;

    /// Update snapshots if TACHIKOMA_UPDATE_SNAPSHOTS is set
    pub fn should_update() -> bool {
        std::env::var("TACHIKOMA_UPDATE_SNAPSHOTS").is_ok()
    }

    /// Compare output with snapshot
    pub fn assert_snapshot(name: &str, actual: &str) {
        let snapshot_path = Path::new("tests/snapshots").join(format!("{name}.snap"));

        if should_update() {
            std::fs::create_dir_all(snapshot_path.parent().unwrap()).ok();
            std::fs::write(&snapshot_path, actual).expect("Failed to write snapshot");
            return;
        }

        if !snapshot_path.exists() {
            panic!(
                "Snapshot not found: {}\nRun with TACHIKOMA_UPDATE_SNAPSHOTS=1 to create",
                snapshot_path.display()
            );
        }

        let expected = std::fs::read_to_string(&snapshot_path).expect("Failed to read snapshot");

        if actual != expected {
            // Show diff
            println!("Snapshot mismatch for {name}:");
            for diff in diff::lines(&expected, actual) {
                match diff {
                    diff::Result::Left(l) => println!("-{l}"),
                    diff::Result::Right(r) => println!("+{r}"),
                    diff::Result::Both(b, _) => println!(" {b}"),
                }
            }
            panic!("Snapshot mismatch. Run with TACHIKOMA_UPDATE_SNAPSHOTS=1 to update");
        }
    }
}
```

### tests/integration/mod.rs

```rust
//! Integration test organization.

mod backends;
mod config;
mod doctor;
mod help;
mod init;
mod tools;
```

### tests/integration/init.rs

```rust
//! Integration tests for init command.

use crate::common::*;

#[test]
fn test_init_creates_project() {
    let ctx = TestContext::new();

    ctx.command()
        .args(["init", "test-project", "--no-prompt"])
        .assert()
        .success();

    let project_dir = ctx.path().join("test-project");
    assert!(project_dir.exists());
    assert!(project_dir.join("tachikoma.toml").exists());
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src/main.rs").exists());
}

#[test]
fn test_init_with_template() {
    let ctx = TestContext::new();

    ctx.command()
        .args(["init", "tools-project", "--template", "tools", "--no-prompt"])
        .assert()
        .success();

    let project_dir = ctx.path().join("tools-project");
    assert!(project_dir.join("config/tools.toml").exists());
}

#[test]
fn test_init_refuses_existing_directory() {
    let ctx = TestContext::new();

    // Create project first
    ctx.command()
        .args(["init", "existing", "--no-prompt"])
        .assert()
        .success();

    // Try to create again
    ctx.command()
        .args(["init", "existing", "--no-prompt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not empty"));
}

#[test]
fn test_init_force_overwrites() {
    let ctx = TestContext::new();

    // Create project first
    ctx.command()
        .args(["init", "force-test", "--no-prompt"])
        .assert()
        .success();

    // Force overwrite
    ctx.command()
        .args(["init", "force-test", "--no-prompt", "--force"])
        .assert()
        .success();
}

#[test]
fn test_init_invalid_name() {
    let ctx = TestContext::new();

    ctx.command()
        .args(["init", "123invalid", "--no-prompt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must start with a letter"));
}

#[test]
fn test_init_json_output() {
    let ctx = TestContext::new();

    let output = ctx
        .command()
        .args(["--format", "json", "init", "json-test", "--no-prompt"])
        .output()
        .expect("Command failed");

    output.assert_success();

    let json = json::parse_output(&output.stdout);
    json::assert_success(&json);
}
```

### tests/integration/config.rs

```rust
//! Integration tests for config command.

use crate::common::*;

#[test]
fn test_config_list() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("project.name"));
}

#[test]
fn test_config_get() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["config", "get", "project.name"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-project"));
}

#[test]
fn test_config_get_nonexistent() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["config", "get", "nonexistent.key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_config_set() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["config", "set", "agent.temperature", "0.5"])
        .assert()
        .success();

    // Verify the change
    ctx.command()
        .args(["config", "get", "agent.temperature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.5"));
}

#[test]
fn test_config_path() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".toml"));
}

#[test]
fn test_config_validate_valid() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["config", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_config_validate_invalid() {
    let ctx = TestContext::new().with_config("invalid toml [[[");

    ctx.command()
        .args(["config", "validate"])
        .assert()
        .failure();
}

#[test]
fn test_config_json_output() {
    let ctx = TestContext::new().with_default_config();

    let output = ctx
        .command()
        .args(["--format", "json", "config", "list"])
        .output()
        .expect("Command failed");

    output.assert_success();

    let json = json::parse_output(&output.stdout);
    assert!(json.is_object() || json.is_array());
}
```

### tests/integration/doctor.rs

```rust
//! Integration tests for doctor command.

use crate::common::*;

#[test]
fn test_doctor_runs() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["doctor"])
        .assert()
        .success();
}

#[test]
fn test_doctor_category_filter() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["doctor", "--category", "system"])
        .assert()
        .success()
        .stdout(predicate::str::contains("System"));
}

#[test]
fn test_doctor_json_output() {
    let ctx = TestContext::new().with_default_config();

    let output = ctx
        .command()
        .args(["--format", "json", "doctor"])
        .output()
        .expect("Command failed");

    let json = json::parse_output(&output.stdout);
    assert!(json.get("checks").is_some());
    assert!(json.get("summary").is_some());
}

#[test]
fn test_doctor_verbose() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["doctor", "--verbose"])
        .assert()
        .success();
}
```

### tests/integration/tools.rs

```rust
//! Integration tests for tools command.

use crate::common::*;

#[test]
fn test_tools_list() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["tools", "list"])
        .assert()
        .success();
}

#[test]
fn test_tools_search() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["tools", "search", "filesystem"])
        .assert()
        .success();
}

#[test]
fn test_tools_show_nonexistent() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["tools", "show", "nonexistent-tool"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_tools_validate() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["tools", "validate"])
        .assert()
        .success();
}
```

### tests/integration/backends.rs

```rust
//! Integration tests for backends command.

use crate::common::*;

#[test]
fn test_backends_list_empty() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["backends", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No backends"));
}

#[test]
fn test_backends_add_and_remove() {
    let ctx = TestContext::new().with_default_config();

    // Add backend
    ctx.command()
        .args([
            "backends", "add", "test-backend",
            "--backend-type", "ollama",
            "--base-url", "http://localhost:11434",
        ])
        .assert()
        .success();

    // List should show it
    ctx.command()
        .args(["backends", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-backend"));

    // Remove backend
    ctx.command()
        .args(["backends", "remove", "test-backend", "--yes"])
        .assert()
        .success();

    // Should be gone
    ctx.command()
        .args(["backends", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No backends"));
}

#[test]
fn test_backends_show_nonexistent() {
    let ctx = TestContext::new().with_default_config();

    ctx.command()
        .args(["backends", "show", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_backends_default() {
    let ctx = TestContext::new().with_default_config();

    // Add two backends
    ctx.command()
        .args([
            "backends", "add", "backend1",
            "--backend-type", "ollama",
        ])
        .assert()
        .success();

    ctx.command()
        .args([
            "backends", "add", "backend2",
            "--backend-type", "ollama",
        ])
        .assert()
        .success();

    // Set backend2 as default
    ctx.command()
        .args(["backends", "default", "backend2"])
        .assert()
        .success();

    // List should show backend2 as default
    ctx.command()
        .args(["backends", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backend2").and(predicate::str::contains("default")));
}
```

### tests/integration/help.rs

```rust
//! Integration tests for help system.

use crate::common::*;

#[test]
fn test_help_flag() {
    let ctx = TestContext::new();

    ctx.command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tachikoma"))
        .stdout(predicate::str::contains("USAGE"));
}

#[test]
fn test_subcommand_help() {
    let ctx = TestContext::new();

    ctx.command()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn test_version_flag() {
    let ctx = TestContext::new();

    ctx.command()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_unknown_command() {
    let ctx = TestContext::new();

    ctx.command()
        .arg("unknown-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_typo_suggestion() {
    let ctx = TestContext::new();

    ctx.command()
        .arg("initt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Did you mean"));
}
```

### tests/snapshots/README.md

```markdown
# Snapshot Tests

This directory contains snapshot files for CLI output testing.

## Updating Snapshots

To update snapshots, run tests with:

```bash
TACHIKOMA_UPDATE_SNAPSHOTS=1 cargo test
```

## Adding New Snapshots

1. Write a test using `snapshot::assert_snapshot("name", output)`
2. Run with `TACHIKOMA_UPDATE_SNAPSHOTS=1` to create the snapshot
3. Review the generated `.snap` file
4. Commit the snapshot file

## Snapshot Format

Snapshots are plain text files with the exact expected output.
```

### Cargo.toml Test Dependencies

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.14"
mockito = "1.4"
diff = "0.1"
criterion = "0.5"

[[bench]]
name = "cli_benchmarks"
harness = false
```

### benches/cli_benchmarks.rs

```rust
//! Performance benchmarks for CLI operations.

use criterion::{criterion_group, criterion_main, Criterion};
use std::process::Command;

fn bench_help(c: &mut Criterion) {
    c.bench_function("help_flag", |b| {
        b.iter(|| {
            Command::new(env!("CARGO_BIN_EXE_tachikoma"))
                .arg("--help")
                .output()
                .expect("Failed to run command")
        })
    });
}

fn bench_version(c: &mut Criterion) {
    c.bench_function("version_flag", |b| {
        b.iter(|| {
            Command::new(env!("CARGO_BIN_EXE_tachikoma"))
                .arg("--version")
                .output()
                .expect("Failed to run command")
        })
    });
}

fn bench_config_list(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("tachikoma.toml");
    std::fs::write(&config_path, "[project]\nname = \"test\"").unwrap();

    c.bench_function("config_list", |b| {
        b.iter(|| {
            Command::new(env!("CARGO_BIN_EXE_tachikoma"))
                .env("TACHIKOMA_CONFIG", &config_path)
                .args(["config", "list"])
                .output()
                .expect("Failed to run command")
        })
    });
}

criterion_group!(benches, bench_help, bench_version, bench_config_list);
criterion_main!(benches);
```

## Testing Requirements

### CI Configuration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          toolchain: ${{ matrix.rust }}

      - name: Run tests
        run: cargo test --all-features

      - name: Run integration tests
        run: cargo test --test '*'

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out xml

      - name: Upload coverage
        uses: codecov/codecov-action@v4
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **091-cli-errors.md**: Error handling tests
- **092-cli-logging.md**: Logging tests
