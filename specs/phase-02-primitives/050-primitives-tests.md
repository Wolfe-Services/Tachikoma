# 050 - Primitives Integration Tests

**Phase:** 2 - Five Primitives
**Spec ID:** 050
**Status:** Planned
**Dependencies:** 031-050 (all primitives specs)
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement comprehensive integration tests for all primitives, testing real-world scenarios, edge cases, and cross-primitive interactions.

---

## Acceptance Criteria

- [x] Integration tests for each primitive
- [x] Cross-primitive workflow tests
- [x] Error scenario tests
- [x] Performance benchmarks
- [x] Security test cases
- [x] Test fixtures and helpers

---

## Implementation Details

### 1. Test Fixtures (tests/fixtures/mod.rs)

```rust
//! Test fixtures and helpers for primitive testing.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

use tachikoma_primitives::{PrimitiveConfig, PrimitiveContext};

/// A test project structure.
pub struct TestProject {
    pub dir: TempDir,
    pub ctx: PrimitiveContext,
}

impl TestProject {
    /// Create a new test project.
    pub fn new() -> Self {
        let dir = tempdir().expect("Failed to create temp dir");
        let ctx = PrimitiveContext::new(dir.path().to_path_buf());

        Self { dir, ctx }
    }

    /// Create with custom config.
    pub fn with_config(config: PrimitiveConfig) -> Self {
        let dir = tempdir().expect("Failed to create temp dir");
        let ctx = PrimitiveContext::with_config(dir.path().to_path_buf(), config);

        Self { dir, ctx }
    }

    /// Get project path.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Create a file with content.
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        let mut file = File::create(&path).expect("Failed to create file");
        file.write_all(content.as_bytes()).expect("Failed to write file");
        path
    }

    /// Create a directory.
    pub fn create_dir(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        fs::create_dir_all(&path).expect("Failed to create dir");
        path
    }

    /// Read a file.
    pub fn read_file(&self, name: &str) -> String {
        let path = self.dir.path().join(name);
        fs::read_to_string(path).expect("Failed to read file")
    }

    /// Check if file exists.
    pub fn file_exists(&self, name: &str) -> bool {
        self.dir.path().join(name).exists()
    }
}

/// Create a Rust project structure.
pub fn create_rust_project() -> TestProject {
    let project = TestProject::new();

    project.create_file("Cargo.toml", r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#);

    project.create_file("src/main.rs", r#"
fn main() {
    println!("Hello, world!");
}
"#);

    project.create_file("src/lib.rs", r#"
/// A greeting function.
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
"#);

    project.create_file("src/utils.rs", r#"
/// Utility functions.

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#);

    project
}

/// Create a project with many files.
pub fn create_large_project() -> TestProject {
    let project = TestProject::new();

    // Create directory structure
    for dir in ["src", "tests", "docs", "config"] {
        project.create_dir(dir);
    }

    // Create multiple source files
    for i in 0..20 {
        project.create_file(
            &format!("src/module_{}.rs", i),
            &format!("pub fn func_{}() {{ }}", i),
        );
    }

    // Create test files
    for i in 0..5 {
        project.create_file(
            &format!("tests/test_{}.rs", i),
            &format!("mod test_{} {{}}", i),
        );
    }

    project
}
```

### 2. Read File Integration Tests (tests/read_file_integration.rs)

```rust
//! Integration tests for read_file primitive.

mod fixtures;

use fixtures::{create_rust_project, TestProject};
use tachikoma_primitives::{read_file, ReadFileOptions, PrimitiveError};

#[tokio::test]
async fn test_read_entire_file() {
    let project = create_rust_project();

    let result = read_file(&project.ctx, "src/main.rs", None).await.unwrap();

    assert!(result.content.contains("fn main()"));
    assert!(result.content.contains("Hello, world!"));
    assert!(!result.truncated);
}

#[tokio::test]
async fn test_read_file_with_line_range() {
    let project = create_rust_project();

    let opts = ReadFileOptions::new().lines(2, 4);
    let result = read_file(&project.ctx, "src/lib.rs", Some(opts)).await.unwrap();

    // Should only contain lines 2-4
    assert!(result.content.contains("greet"));
    assert!(!result.content.contains("cfg(test)"));
}

#[tokio::test]
async fn test_read_nonexistent_file() {
    let project = TestProject::new();

    let result = read_file(&project.ctx, "nonexistent.rs", None).await;

    assert!(matches!(result, Err(PrimitiveError::FileNotFound { .. })));
}

#[tokio::test]
async fn test_read_binary_file() {
    let project = TestProject::new();

    // Create a binary file
    let mut content = vec![0u8; 1000];
    content[0] = 0x00; // NULL byte
    std::fs::write(project.path().join("binary.bin"), &content).unwrap();

    let result = read_file(&project.ctx, "binary.bin", None).await.unwrap();

    // Should detect and handle binary
    assert!(result.content.contains("Binary"));
}

#[tokio::test]
async fn test_read_large_file_truncation() {
    let project = TestProject::new();

    // Create a file larger than default max
    let content = "x".repeat(20 * 1024 * 1024); // 20 MB
    project.create_file("large.txt", &content);

    let opts = ReadFileOptions::new().max_size(1024);
    let result = read_file(&project.ctx, "large.txt", Some(opts)).await.unwrap();

    assert!(result.truncated);
    assert!(result.content.len() <= 1024);
}

#[tokio::test]
async fn test_read_utf8_with_bom() {
    let project = TestProject::new();

    // Create file with BOM
    let mut content = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    content.extend_from_slice(b"Hello");
    std::fs::write(project.path().join("bom.txt"), &content).unwrap();

    let result = read_file(&project.ctx, "bom.txt", None).await.unwrap();

    assert!(result.content.contains("Hello"));
}
```

### 3. Edit File Integration Tests (tests/edit_file_integration.rs)

```rust
//! Integration tests for edit_file primitive.

mod fixtures;

use fixtures::{create_rust_project, TestProject};
use tachikoma_primitives::{edit_file, EditFileOptions, PrimitiveError};

#[tokio::test]
async fn test_simple_replacement() {
    let project = create_rust_project();

    let result = edit_file(
        &project.ctx,
        "src/main.rs",
        "Hello, world!",
        "Hello, Tachikoma!",
        None,
    ).await.unwrap();

    assert!(result.success);
    assert_eq!(result.replacements, 1);

    let content = project.read_file("src/main.rs");
    assert!(content.contains("Hello, Tachikoma!"));
    assert!(!content.contains("Hello, world!"));
}

#[tokio::test]
async fn test_multiline_replacement() {
    let project = create_rust_project();

    let old = r#"fn main() {
    println!("Hello, world!");
}"#;

    let new = r#"fn main() {
    greet("World");
}"#;

    let result = edit_file(&project.ctx, "src/main.rs", old, new, None).await.unwrap();

    assert!(result.success);

    let content = project.read_file("src/main.rs");
    assert!(content.contains("greet(\"World\")"));
}

#[tokio::test]
async fn test_not_unique_error() {
    let project = TestProject::new();
    project.create_file("test.txt", "foo bar foo baz foo");

    let result = edit_file(&project.ctx, "test.txt", "foo", "qux", None).await;

    assert!(matches!(result, Err(PrimitiveError::NotUnique { count: 3 })));
}

#[tokio::test]
async fn test_replace_all() {
    let project = TestProject::new();
    project.create_file("test.txt", "foo bar foo baz foo");

    let opts = EditFileOptions::new().replace_all();
    let result = edit_file(&project.ctx, "test.txt", "foo", "qux", Some(opts)).await.unwrap();

    assert_eq!(result.replacements, 3);

    let content = project.read_file("test.txt");
    assert_eq!(content, "qux bar qux baz qux");
}

#[tokio::test]
async fn test_target_not_found() {
    let project = create_rust_project();

    let result = edit_file(
        &project.ctx,
        "src/main.rs",
        "NOTFOUND",
        "replacement",
        None,
    ).await;

    assert!(matches!(result, Err(PrimitiveError::TargetNotFound)));
}

#[tokio::test]
async fn test_dry_run() {
    let project = TestProject::new();
    project.create_file("test.txt", "original content");

    let opts = EditFileOptions::new().dry_run();
    let result = edit_file(&project.ctx, "test.txt", "original", "modified", Some(opts)).await.unwrap();

    assert!(result.success);
    assert_eq!(result.replacements, 1);

    // File should be unchanged
    let content = project.read_file("test.txt");
    assert_eq!(content, "original content");
}

#[tokio::test]
async fn test_backup_creation() {
    let project = TestProject::new();
    project.create_file("test.txt", "original content");

    let opts = EditFileOptions::new().with_backup();
    edit_file(&project.ctx, "test.txt", "original", "modified", Some(opts)).await.unwrap();

    // Backup should exist
    assert!(project.file_exists("test.bak"));
    assert_eq!(project.read_file("test.bak"), "original content");
}
```

### 4. Bash Integration Tests (tests/bash_integration.rs)

```rust
//! Integration tests for bash primitive.

mod fixtures;

use fixtures::TestProject;
use tachikoma_primitives::{bash, BashOptions, PrimitiveError};
use std::time::Duration;

#[tokio::test]
async fn test_simple_command() {
    let project = TestProject::new();

    let result = bash(&project.ctx, "echo 'hello'", None).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout.trim(), "hello");
    assert!(result.stderr.is_empty());
}

#[tokio::test]
async fn test_exit_code() {
    let project = TestProject::new();

    let result = bash(&project.ctx, "exit 42", None).await.unwrap();

    assert_eq!(result.exit_code, 42);
}

#[tokio::test]
async fn test_stderr_capture() {
    let project = TestProject::new();

    let result = bash(&project.ctx, "echo 'error' >&2", None).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
    assert_eq!(result.stderr.trim(), "error");
}

#[tokio::test]
async fn test_working_directory() {
    let project = TestProject::new();
    project.create_dir("subdir");
    project.create_file("subdir/test.txt", "content");

    let opts = BashOptions::new().working_dir("subdir");
    let result = bash(&project.ctx, "ls", Some(opts)).await.unwrap();

    assert!(result.stdout.contains("test.txt"));
}

#[tokio::test]
async fn test_environment_variables() {
    let project = TestProject::new();

    let opts = BashOptions::new()
        .env("MY_VAR", "my_value");
    let result = bash(&project.ctx, "echo $MY_VAR", Some(opts)).await.unwrap();

    assert_eq!(result.stdout.trim(), "my_value");
}

#[tokio::test]
async fn test_timeout() {
    let project = TestProject::new();

    let opts = BashOptions::new().timeout(Duration::from_millis(100));
    let result = bash(&project.ctx, "sleep 10", Some(opts)).await.unwrap();

    assert!(result.timed_out);
    assert_eq!(result.exit_code, -1);
}

#[tokio::test]
async fn test_piped_commands() {
    let project = TestProject::new();
    project.create_file("numbers.txt", "3\n1\n2");

    let result = bash(&project.ctx, "cat numbers.txt | sort", None).await.unwrap();

    assert_eq!(result.stdout.trim(), "1\n2\n3");
}

#[tokio::test]
async fn test_blocked_command() {
    let project = TestProject::new();

    let result = bash(&project.ctx, "rm -rf /", None).await;

    assert!(matches!(result, Err(PrimitiveError::Validation { .. })));
}
```

### 5. Code Search Integration Tests (tests/code_search_integration.rs)

```rust
//! Integration tests for code_search primitive.

mod fixtures;

use fixtures::{create_rust_project, create_large_project};
use tachikoma_primitives::{code_search, CodeSearchOptions};

#[tokio::test]
async fn test_simple_search() {
    let project = create_rust_project();

    let result = code_search(&project.ctx, "fn main", "src", None).await.unwrap();

    assert!(result.total_count >= 1);
    assert!(result.matches.iter().any(|m| m.line_content.contains("fn main")));
}

#[tokio::test]
async fn test_regex_search() {
    let project = create_rust_project();

    let result = code_search(&project.ctx, r"fn \w+\(", "src", None).await.unwrap();

    // Should find multiple functions
    assert!(result.total_count >= 2);
}

#[tokio::test]
async fn test_file_type_filter() {
    let project = create_rust_project();
    project.create_file("README.md", "fn fake_rust");

    let opts = CodeSearchOptions::new().file_type("rust");
    let result = code_search(&project.ctx, "fn", ".", Some(opts)).await.unwrap();

    // Should not include markdown file
    assert!(result.matches.iter().all(|m| {
        m.path.extension().map_or(false, |e| e == "rs")
    }));
}

#[tokio::test]
async fn test_context_lines() {
    let project = create_rust_project();

    let opts = CodeSearchOptions::new().context(2);
    let result = code_search(&project.ctx, "greet", "src", Some(opts)).await.unwrap();

    // Should have context
    let first_match = &result.matches[0];
    assert!(!first_match.context_before.is_empty() || !first_match.context_after.is_empty());
}

#[tokio::test]
async fn test_no_matches() {
    let project = create_rust_project();

    let result = code_search(&project.ctx, "NOTFOUND_PATTERN", "src", None).await.unwrap();

    assert_eq!(result.total_count, 0);
    assert!(result.matches.is_empty());
}

#[tokio::test]
async fn test_max_matches() {
    let project = create_large_project();

    let opts = CodeSearchOptions::new().max_matches(5);
    let result = code_search(&project.ctx, "pub fn", "src", Some(opts)).await.unwrap();

    assert!(result.matches.len() <= 5);
    if result.total_count > 5 {
        assert!(result.truncated);
    }
}

#[tokio::test]
async fn test_case_insensitive() {
    let project = create_rust_project();

    let opts = CodeSearchOptions::new().case_insensitive();
    let result = code_search(&project.ctx, "HELLO", "src", Some(opts)).await.unwrap();

    assert!(result.total_count >= 1);
}
```

### 6. Cross-Primitive Workflow Tests (tests/workflow_integration.rs)

```rust
//! Integration tests for cross-primitive workflows.

mod fixtures;

use fixtures::TestProject;
use tachikoma_primitives::{
    read_file, list_files, bash, edit_file, code_search,
    ListFilesOptions, CodeSearchOptions,
};

#[tokio::test]
async fn test_find_and_read_workflow() {
    let project = TestProject::new();
    project.create_file("src/config.rs", "pub const VERSION: &str = \"1.0.0\";");
    project.create_file("src/main.rs", "fn main() {}");

    // List files
    let opts = ListFilesOptions::new().extension("rs");
    let files = list_files(&project.ctx, "src", Some(opts)).await.unwrap();

    assert_eq!(files.entries.len(), 2);

    // Read each file
    for entry in &files.entries {
        let relative = entry.path.strip_prefix(project.path()).unwrap();
        let content = read_file(&project.ctx, relative.to_str().unwrap(), None).await.unwrap();
        assert!(!content.content.is_empty());
    }
}

#[tokio::test]
async fn test_search_and_edit_workflow() {
    let project = TestProject::new();
    project.create_file("src/version.rs", "const VERSION: &str = \"1.0.0\";");

    // Search for version
    let search = code_search(&project.ctx, r#"VERSION.*"1\.0\.0""#, "src", None).await.unwrap();

    assert_eq!(search.total_count, 1);
    let match_file = &search.matches[0].path;

    // Edit the file
    let relative = match_file.strip_prefix(project.path()).unwrap();
    edit_file(
        &project.ctx,
        relative.to_str().unwrap(),
        "\"1.0.0\"",
        "\"2.0.0\"",
        None,
    ).await.unwrap();

    // Verify
    let content = project.read_file("src/version.rs");
    assert!(content.contains("\"2.0.0\""));
}

#[tokio::test]
async fn test_bash_then_read_workflow() {
    let project = TestProject::new();

    // Create file with bash
    bash(&project.ctx, "echo 'created by bash' > output.txt", None).await.unwrap();

    // Read the created file
    let content = read_file(&project.ctx, "output.txt", None).await.unwrap();

    assert!(content.content.contains("created by bash"));
}

#[tokio::test]
async fn test_build_project_workflow() {
    let project = TestProject::new();

    // Create a simple Rust project
    project.create_file("Cargo.toml", r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#);

    project.create_file("src/main.rs", r#"
fn main() {
    println!("Build test");
}
"#);

    // Build
    let result = bash(&project.ctx, "cargo build 2>&1 || true", None).await.unwrap();

    // This test just ensures the workflow runs; actual build may fail
    // depending on environment
    assert!(result.stdout.len() > 0 || result.stderr.len() > 0);
}
```

### 7. Performance Benchmarks (benches/primitives_bench.rs)

```rust
//! Benchmarks for primitive operations.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tempfile::tempdir;
use std::fs::write;

fn create_benchmark_project() -> (tempfile::TempDir, tachikoma_primitives::PrimitiveContext) {
    let dir = tempdir().unwrap();

    // Create files
    for i in 0..100 {
        let content = format!("line {}\n", i).repeat(100);
        write(dir.path().join(format!("file_{}.txt", i)), &content).unwrap();
    }

    let ctx = tachikoma_primitives::PrimitiveContext::new(dir.path().to_path_buf());
    (dir, ctx)
}

fn bench_read_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, ctx) = create_benchmark_project();

    c.bench_function("read_file_small", |b| {
        b.iter(|| {
            rt.block_on(async {
                tachikoma_primitives::read_file(&ctx, "file_0.txt", None).await.unwrap()
            })
        })
    });
}

fn bench_list_files(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, ctx) = create_benchmark_project();

    c.bench_function("list_files", |b| {
        b.iter(|| {
            rt.block_on(async {
                tachikoma_primitives::list_files(&ctx, ".", None).await.unwrap()
            })
        })
    });
}

fn bench_code_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, ctx) = create_benchmark_project();

    c.bench_function("code_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                tachikoma_primitives::code_search(&ctx, "line", ".", None).await.unwrap()
            })
        })
    });
}

criterion_group!(benches, bench_read_file, bench_list_files, bench_code_search);
criterion_main!(benches);
```

---

## Testing Requirements

1. All primitives have integration tests
2. Error scenarios are covered
3. Edge cases are tested
4. Cross-primitive workflows work
5. Performance benchmarks run
6. Test fixtures are reusable
7. Tests run in CI

---

## Related Specs

- Depends on: All primitives specs (031-049)
- Used by: CI/CD pipeline
- Related: [phase-22-testing](../phase-22-testing/)
