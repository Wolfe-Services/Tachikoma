# 471i - VCS Integration Tests

**Phase:** 21 - VCS Integration
**Spec ID:** 471i
**Status:** Planned
**Dependencies:** 471h-git-compat
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Comprehensive tests for VCS integration ensuring jj operations work correctly.

---

## Acceptance Criteria

- [ ] Unit tests for all jj operations
- [ ] Integration tests with real jj repos
- [ ] Git compatibility tests
- [ ] Conflict handling tests
- [ ] Undo/redo tests

---

## Implementation Details

### tests/vcs_integration.rs

```rust
//! VCS integration tests.

use tachikoma_vcs::*;
use tempfile::TempDir;

#[test]
fn test_jj_init() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();
    assert!(dir.path().join(".jj").exists());
}

#[test]
fn test_jj_colocated_init() {
    let dir = TempDir::new().unwrap();
    // First create a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Then init jj colocated
    let repo = JjRepo::init(dir.path(), true).unwrap();
    assert!(repo.is_colocated());
}

#[test]
fn test_jj_status_empty() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();
    let status = repo.status().unwrap();
    assert!(status.is_empty());
}

#[test]
fn test_jj_status_with_changes() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();

    // Create a file
    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

    let status = repo.status().unwrap();
    assert!(!status.is_empty());
    assert!(status.iter().any(|c| c.path.to_str() == Some("test.txt")));
}

#[test]
fn test_jj_describe() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();

    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

    let result = repo.describe("Test commit").unwrap();
    assert!(result.success);
}

#[test]
fn test_jj_commit_workflow() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();

    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

    // Commit (describe + new)
    let result = repo.commit("Test commit").unwrap();
    assert!(result.success);

    // Working copy should be clean now (empty change)
    let status = repo.status().unwrap();
    assert!(status.is_empty());
}

#[test]
fn test_jj_undo() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();

    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
    repo.describe("Test").unwrap();

    // Undo should work
    let result = repo.undo().unwrap();
    assert!(result.success);
}

#[test]
fn test_jj_branches() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();

    // Create branch
    repo.create_branch("feature").unwrap();

    let branches = repo.branches().unwrap();
    assert!(branches.iter().any(|b| b.name == "feature"));
}

#[test]
fn test_jj_operation_log() {
    let dir = TempDir::new().unwrap();
    let repo = JjRepo::init(dir.path(), false).unwrap();

    // Do some operations
    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
    repo.describe("Test").unwrap();

    let ops = repo.operation_log(10).unwrap();
    assert!(!ops.is_empty());
}
```

---

## Testing Requirements

All tests should pass with jj installed.

---

## Related Specs

- Depends on: [471h-git-compat.md](471h-git-compat.md)
