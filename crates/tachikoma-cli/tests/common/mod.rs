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