# 470 - Git Tests

**Phase:** 21 - Git Integration
**Spec ID:** 470
**Status:** Planned
**Dependencies:** 451-469 (all Git specs)
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Implement comprehensive tests for all Git integration functionality, ensuring reliability and correctness.

---

## Acceptance Criteria

- [ ] Unit tests for all modules
- [ ] Integration tests with real repositories
- [ ] Property-based tests for edge cases
- [ ] Performance benchmarks
- [ ] Mock repository helpers

---

## Implementation Details

### 1. Test Utilities (src/tests/helpers.rs)

```rust
//! Git test utilities and helpers.

use crate::{GitOid, GitRepository, GitResult};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test repository builder.
pub struct TestRepo {
    /// Temporary directory.
    pub dir: TempDir,
    /// Repository instance.
    pub repo: GitRepository,
}

impl TestRepo {
    /// Create a new test repository.
    pub fn new() -> GitResult<Self> {
        let dir = TempDir::new()?;
        let repo = GitRepository::init(dir.path())?;

        // Configure test user
        repo.set_config("user.name", "Test User")?;
        repo.set_config("user.email", "test@example.com")?;

        Ok(Self { dir, repo })
    }

    /// Create a bare test repository.
    pub fn bare() -> GitResult<Self> {
        let dir = TempDir::new()?;
        let repo = GitRepository::init_bare(dir.path())?;
        Ok(Self { dir, repo })
    }

    /// Get the repository path.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Create a file with content.
    pub fn write_file(&self, name: &str, content: &str) -> GitResult<PathBuf> {
        let path = self.dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Create multiple files.
    pub fn write_files(&self, files: &[(&str, &str)]) -> GitResult<Vec<PathBuf>> {
        files.iter()
            .map(|(name, content)| self.write_file(name, content))
            .collect()
    }

    /// Stage a file.
    pub fn stage(&self, path: &str) -> GitResult<()> {
        let path = Path::new(path);
        self.repo.stage_paths(&[path])
    }

    /// Stage all files.
    pub fn stage_all(&self) -> GitResult<()> {
        self.repo.stage_all()
    }

    /// Create a commit.
    pub fn commit(&self, message: &str) -> GitResult<GitOid> {
        let commit = self.repo.commit(message, Default::default())?;
        Ok(commit.oid)
    }

    /// Create file and commit in one step.
    pub fn commit_file(&self, name: &str, content: &str, message: &str) -> GitResult<GitOid> {
        self.write_file(name, content)?;
        self.stage(name)?;
        self.commit(message)
    }

    /// Create multiple commits.
    pub fn commit_files(&self, commits: &[(&str, &str, &str)]) -> GitResult<Vec<GitOid>> {
        commits.iter()
            .map(|(name, content, message)| self.commit_file(name, content, message))
            .collect()
    }

    /// Create a branch.
    pub fn branch(&self, name: &str) -> GitResult<()> {
        self.repo.create_branch(name, None, false)?;
        Ok(())
    }

    /// Checkout a branch.
    pub fn checkout(&self, name: &str) -> GitResult<()> {
        self.repo.checkout(name)
    }

    /// Create and checkout a branch.
    pub fn checkout_new(&self, name: &str) -> GitResult<()> {
        self.branch(name)?;
        self.checkout(name)
    }

    /// Get current HEAD OID.
    pub fn head(&self) -> GitResult<GitOid> {
        self.repo.head_oid()
    }

    /// Add a remote.
    pub fn add_remote(&self, name: &str, url: &str) -> GitResult<()> {
        self.repo.add_remote(name, url)?;
        Ok(())
    }
}

/// Create a test repository with some initial commits.
pub fn repo_with_history() -> GitResult<TestRepo> {
    let repo = TestRepo::new()?;

    repo.commit_files(&[
        ("README.md", "# Test Project\n", "Initial commit"),
        ("src/main.rs", "fn main() {}\n", "Add main"),
        ("src/lib.rs", "pub fn hello() {}\n", "Add lib"),
    ])?;

    Ok(repo)
}

/// Create a test repository with branches.
pub fn repo_with_branches() -> GitResult<TestRepo> {
    let repo = repo_with_history()?;

    // Create feature branch
    repo.checkout_new("feature")?;
    repo.commit_file("feature.rs", "// feature\n", "Add feature")?;

    // Go back to main
    repo.checkout("main")?;

    // Create another branch
    repo.checkout_new("bugfix")?;
    repo.commit_file("fix.rs", "// fix\n", "Add fix")?;

    // Go back to main
    repo.checkout("main")?;

    Ok(repo)
}

/// Create a test repository with merge conflict.
pub fn repo_with_conflict() -> GitResult<TestRepo> {
    let repo = repo_with_history()?;

    // Create conflicting changes on feature branch
    repo.checkout_new("feature")?;
    repo.write_file("conflict.txt", "feature content\n")?;
    repo.stage_all()?;
    repo.commit("Feature changes")?;

    // Create conflicting changes on main
    repo.checkout("main")?;
    repo.write_file("conflict.txt", "main content\n")?;
    repo.stage_all()?;
    repo.commit("Main changes")?;

    Ok(repo)
}

/// Create a pair of repositories (origin and clone).
pub fn repo_pair() -> GitResult<(TestRepo, TestRepo)> {
    // Create "origin" as bare
    let origin = TestRepo::bare()?;

    // Create "local" and set up remote
    let local = TestRepo::new()?;
    local.add_remote("origin", origin.path().to_str().unwrap())?;

    // Create initial commit
    local.commit_file("README.md", "# Test\n", "Initial commit")?;

    Ok((origin, local))
}

/// Assert macro for Git operations.
#[macro_export]
macro_rules! assert_git_ok {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => panic!("Git operation failed: {:?}", e),
        }
    };
}

/// Assert macro for Git errors.
#[macro_export]
macro_rules! assert_git_err {
    ($result:expr) => {
        match $result {
            Ok(_) => panic!("Expected Git error, got Ok"),
            Err(e) => e,
        }
    };
}
```

### 2. Core Type Tests (src/tests/core_tests.rs)

```rust
//! Tests for core Git types.

use super::helpers::*;
use crate::{GitOid, GitRef, GitSignature};

#[test]
fn test_git_oid_from_hex() {
    let hex = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    let oid = GitOid::from_hex(hex).unwrap();
    assert_eq!(oid.to_hex(), hex);
}

#[test]
fn test_git_oid_invalid_hex() {
    let result = GitOid::from_hex("invalid");
    assert!(result.is_err());
}

#[test]
fn test_git_oid_short_hex() {
    let hex = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    let oid = GitOid::from_hex(hex).unwrap();
    assert_eq!(oid.short(), &hex[..7]);
}

#[test]
fn test_git_ref_branch() {
    let ref_name = GitRef::branch("main");
    assert_eq!(ref_name.full_name(), "refs/heads/main");
    assert!(ref_name.is_branch());
}

#[test]
fn test_git_ref_tag() {
    let ref_name = GitRef::tag("v1.0.0");
    assert_eq!(ref_name.full_name(), "refs/tags/v1.0.0");
    assert!(ref_name.is_tag());
}

#[test]
fn test_git_signature_creation() {
    let sig = GitSignature::new("Test User", "test@example.com");
    assert_eq!(sig.name, "Test User");
    assert_eq!(sig.email, "test@example.com");
}
```

### 3. Repository Tests (src/tests/repo_tests.rs)

```rust
//! Tests for repository operations.

use super::helpers::*;
use crate::GitRepository;

#[test]
fn test_repo_init() {
    let repo = TestRepo::new().unwrap();
    assert!(repo.path().join(".git").exists());
}

#[test]
fn test_repo_init_bare() {
    let repo = TestRepo::bare().unwrap();
    assert!(repo.repo.is_bare().unwrap());
}

#[test]
fn test_repo_discover() {
    let repo = TestRepo::new().unwrap();
    let subdir = repo.path().join("subdir");
    std::fs::create_dir_all(&subdir).unwrap();

    let discovered = GitRepository::discover(&subdir).unwrap();
    assert_eq!(discovered.root_path(), repo.path());
}

#[test]
fn test_repo_head() {
    let repo = repo_with_history().unwrap();
    let head = repo.repo.head_ref().unwrap();
    assert!(head.is_some());
}

#[test]
fn test_repo_is_empty() {
    let repo = TestRepo::new().unwrap();
    assert!(repo.repo.is_empty().unwrap());

    repo.commit_file("test.txt", "content", "Initial").unwrap();
    assert!(!repo.repo.is_empty().unwrap());
}
```

### 4. Status Tests (src/tests/status_tests.rs)

```rust
//! Tests for status operations.

use super::helpers::*;
use crate::status::StatusOptions;

#[test]
fn test_status_clean() {
    let repo = repo_with_history().unwrap();
    let status = repo.repo.status(StatusOptions::default()).unwrap();
    assert!(status.is_clean());
}

#[test]
fn test_status_untracked() {
    let repo = TestRepo::new().unwrap();
    repo.write_file("untracked.txt", "content").unwrap();

    let status = repo.repo.status(StatusOptions::default()).unwrap();
    assert_eq!(status.untracked.len(), 1);
}

#[test]
fn test_status_staged() {
    let repo = TestRepo::new().unwrap();
    repo.write_file("staged.txt", "content").unwrap();
    repo.stage("staged.txt").unwrap();

    let status = repo.repo.status(StatusOptions::default()).unwrap();
    assert_eq!(status.staged.len(), 1);
}

#[test]
fn test_status_modified() {
    let repo = repo_with_history().unwrap();
    repo.write_file("README.md", "modified content").unwrap();

    let status = repo.repo.status(StatusOptions::default()).unwrap();
    assert_eq!(status.modified.len(), 1);
}

#[test]
fn test_status_exclude_untracked() {
    let repo = TestRepo::new().unwrap();
    repo.write_file("untracked.txt", "content").unwrap();

    let status = repo.repo.status(StatusOptions::default().exclude_untracked()).unwrap();
    assert!(status.untracked.is_empty());
}
```

### 5. Branch Tests (src/tests/branch_tests.rs)

```rust
//! Tests for branch operations.

use super::helpers::*;

#[test]
fn test_branch_create() {
    let repo = repo_with_history().unwrap();
    repo.branch("feature").unwrap();

    let branches = repo.repo.list_branches(None).unwrap();
    assert!(branches.iter().any(|b| b.name == "feature"));
}

#[test]
fn test_branch_delete() {
    let repo = repo_with_history().unwrap();
    repo.branch("to-delete").unwrap();
    repo.repo.delete_branch("to-delete", false).unwrap();

    let branches = repo.repo.list_branches(None).unwrap();
    assert!(!branches.iter().any(|b| b.name == "to-delete"));
}

#[test]
fn test_branch_rename() {
    let repo = repo_with_history().unwrap();
    repo.branch("old-name").unwrap();
    repo.repo.rename_branch("old-name", "new-name").unwrap();

    let branches = repo.repo.list_branches(None).unwrap();
    assert!(!branches.iter().any(|b| b.name == "old-name"));
    assert!(branches.iter().any(|b| b.name == "new-name"));
}

#[test]
fn test_branch_checkout() {
    let repo = repo_with_branches().unwrap();
    repo.checkout("feature").unwrap();

    let current = repo.repo.current_branch().unwrap().unwrap();
    assert_eq!(current.name, "feature");
}

#[test]
fn test_branch_list_local() {
    let repo = repo_with_branches().unwrap();
    let branches = repo.repo.list_branches(Some(crate::branch::BranchType::Local)).unwrap();
    assert!(branches.len() >= 3); // main, feature, bugfix
}
```

### 6. Commit Tests (src/tests/commit_tests.rs)

```rust
//! Tests for commit operations.

use super::helpers::*;
use crate::commit::CommitOptions;

#[test]
fn test_commit_basic() {
    let repo = TestRepo::new().unwrap();
    repo.write_file("test.txt", "content").unwrap();
    repo.stage_all().unwrap();

    let oid = repo.commit("Test commit").unwrap();
    let commit = repo.repo.get_commit(&oid).unwrap();

    assert_eq!(commit.summary, "Test commit");
}

#[test]
fn test_commit_amend() {
    let repo = TestRepo::new().unwrap();
    repo.commit_file("test.txt", "content", "Original message").unwrap();

    let commit = repo.repo.amend(Some("Amended message")).unwrap();
    assert_eq!(commit.summary, "Amended message");
}

#[test]
fn test_commit_empty_fails() {
    let repo = repo_with_history().unwrap();
    let result = repo.repo.commit("Empty commit", CommitOptions::default());
    assert!(result.is_err());
}

#[test]
fn test_commit_allow_empty() {
    let repo = repo_with_history().unwrap();
    let commit = repo.repo.commit("Empty commit", CommitOptions::default().allow_empty()).unwrap();
    assert!(commit.oid.to_hex().len() > 0);
}

#[test]
fn test_commit_parents() {
    let repo = repo_with_history().unwrap();
    let oid = repo.commit_file("new.txt", "content", "New commit").unwrap();
    let commit = repo.repo.get_commit(&oid).unwrap();

    assert_eq!(commit.parents.len(), 1);
}
```

### 7. Diff Tests (src/tests/diff_tests.rs)

```rust
//! Tests for diff operations.

use super::helpers::*;
use crate::diff::DiffOptions;

#[test]
fn test_diff_workdir() {
    let repo = repo_with_history().unwrap();
    repo.write_file("README.md", "modified content\n").unwrap();

    let diff = repo.repo.diff_workdir(DiffOptions::default()).unwrap();
    assert_eq!(diff.files_changed, 1);
}

#[test]
fn test_diff_staged() {
    let repo = repo_with_history().unwrap();
    repo.write_file("new.txt", "new content\n").unwrap();
    repo.stage("new.txt").unwrap();

    let diff = repo.repo.diff_staged(DiffOptions::default()).unwrap();
    assert_eq!(diff.files_changed, 1);
}

#[test]
fn test_diff_commits() {
    let repo = repo_with_history().unwrap();
    let commits: Vec<_> = repo.repo.log(Default::default()).unwrap();

    assert!(commits.len() >= 2);
    let diff = repo.repo.diff_commits(
        &commits[1].oid,
        &commits[0].oid,
        DiffOptions::default()
    ).unwrap();

    assert!(diff.files_changed > 0);
}

#[test]
fn test_diff_context_lines() {
    let repo = repo_with_history().unwrap();

    // Create file with multiple lines
    let content = (0..20).map(|i| format!("line {}\n", i)).collect::<String>();
    repo.write_file("lines.txt", &content).unwrap();
    repo.stage_all().unwrap();
    repo.commit("Add lines").unwrap();

    // Modify middle line
    let new_content = content.replace("line 10\n", "modified line 10\n");
    repo.write_file("lines.txt", &new_content).unwrap();

    let diff = repo.repo.diff_workdir(DiffOptions::default().context_lines(3)).unwrap();
    assert!(diff.files_changed > 0);
}
```

### 8. Merge Tests (src/tests/merge_tests.rs)

```rust
//! Tests for merge operations.

use super::helpers::*;
use crate::merge::MergeOptions;

#[test]
fn test_merge_fast_forward() {
    let repo = repo_with_history().unwrap();

    // Create feature branch with new commit
    repo.checkout_new("feature").unwrap();
    repo.commit_file("feature.txt", "content", "Feature commit").unwrap();

    // Go back to main and merge
    repo.checkout("main").unwrap();
    let result = repo.repo.merge("feature", MergeOptions::default()).unwrap();

    assert!(result.fast_forward);
}

#[test]
fn test_merge_conflict() {
    let repo = repo_with_conflict().unwrap();
    let result = repo.repo.merge("feature", MergeOptions::default()).unwrap();

    assert!(result.conflicts.len() > 0);
}

#[test]
fn test_merge_abort() {
    let repo = repo_with_conflict().unwrap();
    let _ = repo.repo.merge("feature", MergeOptions::default()).unwrap();
    repo.repo.merge_abort().unwrap();

    let status = repo.repo.status(Default::default()).unwrap();
    assert!(status.conflicted.is_empty());
}
```

### 9. History Tests (src/tests/history_tests.rs)

```rust
//! Tests for history operations.

use super::helpers::*;
use crate::history::HistoryOptions;

#[test]
fn test_log_all() {
    let repo = repo_with_history().unwrap();
    let log = repo.repo.log(HistoryOptions::default()).unwrap();

    assert!(log.len() >= 3);
}

#[test]
fn test_log_limit() {
    let repo = repo_with_history().unwrap();
    let log = repo.repo.log(HistoryOptions::default().limit(2)).unwrap();

    assert_eq!(log.len(), 2);
}

#[test]
fn test_log_path() {
    let repo = repo_with_history().unwrap();
    let log = repo.repo.log(HistoryOptions::default().path("src/main.rs")).unwrap();

    assert!(log.len() >= 1);
}

#[test]
fn test_log_since() {
    let repo = repo_with_history().unwrap();
    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    let log = repo.repo.log(HistoryOptions::default().since(since)).unwrap();

    assert!(log.len() >= 3); // All commits should be within the last hour
}
```

### 10. Blame Tests (src/tests/blame_tests.rs)

```rust
//! Tests for blame operations.

use super::helpers::*;
use crate::blame::BlameOptions;

#[test]
fn test_blame_file() {
    let repo = repo_with_history().unwrap();
    let blame = repo.repo.blame("README.md", BlameOptions::default()).unwrap();

    assert!(blame.entries.len() > 0);
}

#[test]
fn test_blame_line_range() {
    let repo = repo_with_history().unwrap();

    // Create file with multiple lines
    let content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    repo.write_file("lines.txt", content).unwrap();
    repo.stage_all().unwrap();
    repo.commit("Add lines").unwrap();

    let blame = repo.repo.blame("lines.txt", BlameOptions::lines(2, 4)).unwrap();
    assert!(blame.entries.len() > 0);
}

#[test]
fn test_blame_line() {
    let repo = repo_with_history().unwrap();
    let blame = repo.repo.blame_line("README.md", 1).unwrap();

    assert!(blame.is_some());
}
```

### 11. Performance Benchmarks (benches/git_benchmarks.rs)

```rust
//! Git operation benchmarks.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tachikoma_git::tests::helpers::*;

fn bench_status(c: &mut Criterion) {
    let repo = repo_with_history().unwrap();

    c.bench_function("status_clean", |b| {
        b.iter(|| repo.repo.status(Default::default()).unwrap())
    });
}

fn bench_log(c: &mut Criterion) {
    let repo = repo_with_history().unwrap();

    // Add more commits for meaningful benchmark
    for i in 0..100 {
        repo.commit_file(
            &format!("file{}.txt", i),
            "content",
            &format!("Commit {}", i)
        ).unwrap();
    }

    let mut group = c.benchmark_group("log");
    for limit in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(limit),
            limit,
            |b, &limit| {
                b.iter(|| {
                    repo.repo.log(
                        crate::history::HistoryOptions::default().limit(limit)
                    ).unwrap()
                })
            },
        );
    }
    group.finish();
}

fn bench_diff(c: &mut Criterion) {
    let repo = repo_with_history().unwrap();

    // Make some changes
    for i in 0..10 {
        repo.write_file(&format!("changed{}.txt", i), "new content").unwrap();
    }

    c.bench_function("diff_workdir", |b| {
        b.iter(|| repo.repo.diff_workdir(Default::default()).unwrap())
    });
}

fn bench_branch_list(c: &mut Criterion) {
    let repo = repo_with_history().unwrap();

    // Create many branches
    for i in 0..50 {
        repo.branch(&format!("branch-{}", i)).unwrap();
    }

    c.bench_function("list_branches", |b| {
        b.iter(|| repo.repo.list_branches(None).unwrap())
    });
}

criterion_group!(
    benches,
    bench_status,
    bench_log,
    bench_diff,
    bench_branch_list,
);
criterion_main!(benches);
```

### 12. Property-Based Tests (src/tests/prop_tests.rs)

```rust
//! Property-based tests for Git operations.

use proptest::prelude::*;
use super::helpers::*;

proptest! {
    #[test]
    fn prop_commit_message_preserved(message in "[a-zA-Z0-9 ]{1,100}") {
        let repo = TestRepo::new().unwrap();
        repo.write_file("test.txt", "content").unwrap();
        repo.stage_all().unwrap();

        let oid = repo.commit(&message).unwrap();
        let commit = repo.repo.get_commit(&oid).unwrap();

        prop_assert_eq!(commit.message.trim(), message.trim());
    }

    #[test]
    fn prop_branch_names_valid(name in "[a-zA-Z][a-zA-Z0-9-]{0,50}") {
        let repo = repo_with_history().unwrap();

        if let Ok(_) = repo.branch(&name) {
            let branches = repo.repo.list_branches(None).unwrap();
            prop_assert!(branches.iter().any(|b| b.name == name));
        }
    }

    #[test]
    fn prop_file_content_preserved(content in "[a-zA-Z0-9\n ]{0,1000}") {
        let repo = TestRepo::new().unwrap();
        repo.write_file("test.txt", &content).unwrap();
        repo.stage_all().unwrap();
        repo.commit("Test commit").unwrap();

        let read_content = std::fs::read_to_string(repo.path().join("test.txt")).unwrap();
        prop_assert_eq!(read_content, content);
    }

    #[test]
    fn prop_oid_roundtrip(bytes in prop::array::uniform32(0u8..)) {
        // Create a valid hex string from bytes
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

        if let Ok(oid) = crate::GitOid::from_hex(&hex) {
            let roundtrip = oid.to_hex();
            prop_assert_eq!(roundtrip, hex);
        }
    }
}

#[test]
fn test_commit_order_preserved() {
    let repo = TestRepo::new().unwrap();

    let mut oids = Vec::new();
    for i in 0..10 {
        repo.write_file(&format!("file{}.txt", i), &format!("content {}", i)).unwrap();
        repo.stage_all().unwrap();
        oids.push(repo.commit(&format!("Commit {}", i)).unwrap());
    }

    let log = repo.repo.log(Default::default()).unwrap();

    // Most recent should be first
    assert_eq!(log[0].oid, oids[9]);
    assert_eq!(log[9].oid, oids[0]);
}

#[test]
fn test_concurrent_reads() {
    use std::thread;

    let repo = repo_with_history().unwrap();
    let repo_path = repo.path().to_path_buf();

    let handles: Vec<_> = (0..4).map(|_| {
        let path = repo_path.clone();
        thread::spawn(move || {
            let repo = crate::GitRepository::open(&path).unwrap();
            for _ in 0..100 {
                repo.status(Default::default()).unwrap();
                repo.log(Default::default()).unwrap();
            }
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
```

---

## Testing Requirements

1. All unit tests pass
2. Integration tests work with real Git
3. Property tests cover edge cases
4. Benchmarks show acceptable performance
5. Concurrent access is safe

---

## Related Specs

- Depends on: All Git specs (451-469)
- Phase completion: This completes Phase 21 - Git Integration
