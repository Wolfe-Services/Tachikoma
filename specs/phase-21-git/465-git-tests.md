# Spec 465: Git Integration Tests

## Phase
21 - Git Integration

## Spec ID
465

## Status
Planned

## Dependencies
- All previous Git specs (446-464)

## Estimated Context
~12%

---

## Objective

Implement comprehensive integration tests for the Git module in Tachikoma, ensuring all Git operations work correctly together and handle edge cases properly. This includes tests for complex workflows, error scenarios, and performance benchmarks.

---

## Acceptance Criteria

- [ ] Integration tests for repository lifecycle
- [ ] Tests for branching and merging workflows
- [ ] Tests for remote operations (mocked)
- [ ] Tests for conflict resolution
- [ ] Tests for stash and worktree operations
- [ ] Performance benchmarks for large repositories
- [ ] Error handling tests
- [ ] Edge case tests
- [ ] Test utilities and fixtures
- [ ] CI/CD test configuration

---

## Implementation Details

### Test Utilities and Fixtures

```rust
// src/git/tests/mod.rs

pub mod fixtures;
pub mod helpers;
pub mod integration;
pub mod benchmarks;

/// Re-export test utilities
pub use fixtures::*;
pub use helpers::*;
```

```rust
// src/git/tests/fixtures.rs

use tempfile::TempDir;
use std::path::{Path, PathBuf};
use std::fs;
use git2::{Repository, Signature};

use crate::git::repo::GitRepository;
use crate::git::types::*;

/// Test fixture for Git repository
pub struct GitTestFixture {
    pub temp_dir: TempDir,
    pub repo: GitRepository,
}

impl GitTestFixture {
    /// Create a new empty repository
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo = GitRepository::init(temp_dir.path(), false).expect("Failed to init repo");

        // Configure git
        let mut config = repo.config().expect("Failed to get config");
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

        Self { temp_dir, repo }
    }

    /// Create repository with initial commit
    pub fn with_initial_commit() -> Self {
        let fixture = Self::new();
        fixture.create_file("README.md", "# Test Repository\n");
        fixture.stage_file("README.md");
        fixture.commit("Initial commit");
        fixture
    }

    /// Create repository with multiple commits
    pub fn with_history(num_commits: usize) -> Self {
        let fixture = Self::with_initial_commit();

        for i in 1..num_commits {
            let filename = format!("file{}.txt", i);
            fixture.create_file(&filename, &format!("Content {}\n", i));
            fixture.stage_file(&filename);
            fixture.commit(&format!("Add {}", filename));
        }

        fixture
    }

    /// Create repository with branches
    pub fn with_branches(branch_names: &[&str]) -> Self {
        let fixture = Self::with_initial_commit();

        for name in branch_names {
            fixture.create_branch(name);
        }

        fixture
    }

    /// Get repository path
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a file in the repository
    pub fn create_file(&self, name: &str, content: &str) {
        let path = self.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    /// Modify a file
    pub fn modify_file(&self, name: &str, content: &str) {
        self.create_file(name, content);
    }

    /// Delete a file
    pub fn delete_file(&self, name: &str) {
        let path = self.path().join(name);
        fs::remove_file(path).unwrap();
    }

    /// Stage a file
    pub fn stage_file(&self, name: &str) {
        self.repo.stage_file(Path::new(name)).unwrap();
    }

    /// Stage all changes
    pub fn stage_all(&self) {
        self.repo.stage_all().unwrap();
    }

    /// Create a commit
    pub fn commit(&self, message: &str) -> GitOid {
        let raw = self.repo.raw();
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();

        let parents: Vec<_> = raw.head().ok()
            .and_then(|h| h.peel_to_commit().ok())
            .into_iter()
            .collect();
        let parent_refs: Vec<_> = parents.iter().collect();

        let oid = raw.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parent_refs,
        ).unwrap();

        GitOid::from(oid)
    }

    /// Create a branch
    pub fn create_branch(&self, name: &str) {
        let raw = self.repo.raw();
        let head = raw.head().unwrap().peel_to_commit().unwrap();
        raw.branch(name, &head, false).unwrap();
    }

    /// Checkout a branch
    pub fn checkout(&self, name: &str) {
        let raw = self.repo.raw();
        let refname = format!("refs/heads/{}", name);
        let obj = raw.revparse_single(&refname).unwrap();
        raw.checkout_tree(&obj, None).unwrap();
        raw.set_head(&refname).unwrap();
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Option<String> {
        self.repo.current_branch().ok().flatten()
    }

    /// Create a merge conflict
    pub fn create_conflict(&self, branch_name: &str, file: &str) {
        // Create a change on current branch
        self.modify_file(file, "Main branch content\n");
        self.stage_file(file);
        self.commit("Change on main");

        // Create conflicting change on other branch
        self.checkout(branch_name);
        self.modify_file(file, "Branch content\n");
        self.stage_file(file);
        self.commit("Change on branch");
    }
}

impl Default for GitTestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a bare repository fixture
pub fn create_bare_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo = Repository::init_bare(temp_dir.path()).expect("Failed to init bare repo");
    (temp_dir, repo)
}

/// Create a pair of repos for remote testing (origin + clone)
pub fn create_repo_pair() -> (GitTestFixture, GitTestFixture) {
    let origin = GitTestFixture::with_initial_commit();
    let origin_path = origin.path().to_string_lossy().to_string();

    let clone_dir = TempDir::new().expect("Failed to create temp dir");
    let clone_repo = Repository::clone(&origin_path, clone_dir.path())
        .expect("Failed to clone");

    let clone = GitTestFixture {
        temp_dir: clone_dir,
        repo: GitRepository::open(clone_repo.workdir().unwrap()).unwrap(),
    };

    (origin, clone)
}
```

### Integration Tests

```rust
// src/git/tests/integration.rs

use super::fixtures::*;
use crate::git::*;

#[cfg(test)]
mod repository_tests {
    use super::*;

    #[test]
    fn test_repository_lifecycle() {
        let fixture = GitTestFixture::new();

        assert!(fixture.repo.is_empty().unwrap());
        assert_eq!(fixture.repo.state(), RepoState::Clean);
    }

    #[test]
    fn test_repository_with_commits() {
        let fixture = GitTestFixture::with_history(5);

        assert!(!fixture.repo.is_empty().unwrap());

        let logger = GitLogger::new(&fixture.repo);
        let commits = logger.commits(LogOptions::new()).unwrap();
        assert_eq!(commits.len(), 5);
    }

    #[test]
    fn test_repository_discover() {
        let fixture = GitTestFixture::with_initial_commit();

        // Create subdirectory
        let subdir = fixture.path().join("src").join("module");
        std::fs::create_dir_all(&subdir).unwrap();

        // Discover from subdirectory
        let discovered = GitRepository::discover(&subdir).unwrap();
        assert_eq!(discovered.path(), fixture.path());
    }
}

#[cfg(test)]
mod branch_workflow_tests {
    use super::*;

    #[test]
    fn test_feature_branch_workflow() {
        let fixture = GitTestFixture::with_initial_commit();
        let manager = GitBranchManager::new(&fixture.repo);

        // Create feature branch
        manager.create("feature", CreateBranchOptions::new().checkout(true)).unwrap();
        assert_eq!(fixture.current_branch(), Some("feature".to_string()));

        // Make changes on feature
        fixture.create_file("feature.txt", "Feature content");
        fixture.stage_file("feature.txt");
        fixture.commit("Add feature");

        // Switch back to main
        manager.checkout("master", CheckoutOptions::new()).unwrap();

        // Check file doesn't exist on main
        assert!(!fixture.path().join("feature.txt").exists());

        // Merge feature
        let merger = GitMerger::new(&fixture.repo);
        let result = merger.merge("feature", MergeOperationOptions::new()).unwrap();
        assert!(result.success);

        // File should now exist
        assert!(fixture.path().join("feature.txt").exists());
    }

    #[test]
    fn test_multiple_branches() {
        let fixture = GitTestFixture::with_branches(&["develop", "feature-1", "feature-2"]);
        let manager = GitBranchManager::new(&fixture.repo);

        let branches = manager.list(ListBranchOptions::new()).unwrap();

        // Should have main + 3 branches
        assert!(branches.len() >= 4);
    }

    #[test]
    fn test_branch_ahead_behind() {
        let fixture = GitTestFixture::with_initial_commit();
        let manager = GitBranchManager::new(&fixture.repo);

        // Create and checkout feature branch
        manager.create("feature", CreateBranchOptions::new().checkout(true)).unwrap();

        // Add commits on feature
        fixture.create_file("f1.txt", "1");
        fixture.stage_all();
        fixture.commit("Feature 1");

        fixture.create_file("f2.txt", "2");
        fixture.stage_all();
        fixture.commit("Feature 2");

        // Compare with master
        let comparison = manager.compare("master", "feature").unwrap();
        assert_eq!(comparison.ahead, 2);
        assert_eq!(comparison.behind, 0);
    }
}

#[cfg(test)]
mod merge_tests {
    use super::*;

    #[test]
    fn test_fast_forward_merge() {
        let fixture = GitTestFixture::with_initial_commit();
        let manager = GitBranchManager::new(&fixture.repo);

        // Create and checkout feature
        manager.create("feature", CreateBranchOptions::new().checkout(true)).unwrap();
        fixture.create_file("new.txt", "content");
        fixture.stage_all();
        fixture.commit("Add file");

        // Back to master
        manager.checkout("master", CheckoutOptions::new()).unwrap();

        // Merge
        let merger = GitMerger::new(&fixture.repo);
        let result = merger.merge("feature", MergeOperationOptions::new()).unwrap();

        assert!(result.success);
        assert!(result.fast_forward);
    }

    #[test]
    fn test_three_way_merge() {
        let fixture = GitTestFixture::with_initial_commit();
        let manager = GitBranchManager::new(&fixture.repo);

        // Create feature branch
        manager.create("feature", CreateBranchOptions::default()).unwrap();

        // Add commit on master
        fixture.create_file("master.txt", "master");
        fixture.stage_all();
        fixture.commit("Master change");

        // Add commit on feature
        manager.checkout("feature", CheckoutOptions::new()).unwrap();
        fixture.create_file("feature.txt", "feature");
        fixture.stage_all();
        fixture.commit("Feature change");

        // Back to master and merge
        manager.checkout("master", CheckoutOptions::new()).unwrap();
        let merger = GitMerger::new(&fixture.repo);
        let result = merger.merge("feature", MergeOperationOptions::new()).unwrap();

        assert!(result.success);
        assert!(!result.fast_forward);
    }

    #[test]
    fn test_merge_conflict_detection() {
        let fixture = GitTestFixture::with_initial_commit();
        let manager = GitBranchManager::new(&fixture.repo);

        // Create feature branch
        manager.create("feature", CreateBranchOptions::default()).unwrap();

        // Modify same file on master
        fixture.modify_file("README.md", "Master version\n");
        fixture.stage_all();
        fixture.commit("Master change");

        // Modify same file on feature
        manager.checkout("feature", CheckoutOptions::new()).unwrap();
        fixture.modify_file("README.md", "Feature version\n");
        fixture.stage_all();
        fixture.commit("Feature change");

        // Back to master and try merge
        manager.checkout("master", CheckoutOptions::new()).unwrap();
        let merger = GitMerger::new(&fixture.repo);
        let result = merger.merge("feature", MergeOperationOptions::new()).unwrap();

        assert!(!result.success);
        assert!(result.has_conflicts());
    }
}

#[cfg(test)]
mod diff_tests {
    use super::*;

    #[test]
    fn test_staged_diff() {
        let fixture = GitTestFixture::with_initial_commit();

        fixture.create_file("new.txt", "new content\n");
        fixture.stage_file("new.txt");

        let generator = GitDiffGenerator::new(&fixture.repo);
        let diff = generator.staged_diff(&DiffGenerationOptions::default()).unwrap();

        assert!(!diff.is_empty());
        assert!(diff.added_files().len() >= 1);
    }

    #[test]
    fn test_unstaged_diff() {
        let fixture = GitTestFixture::with_initial_commit();

        fixture.modify_file("README.md", "Modified content\n");

        let generator = GitDiffGenerator::new(&fixture.repo);
        let diff = generator.unstaged_diff(&DiffGenerationOptions::default()).unwrap();

        assert!(!diff.is_empty());
        assert!(diff.modified_files().len() >= 1);
    }

    #[test]
    fn test_diff_between_commits() {
        let fixture = GitTestFixture::with_history(3);

        let generator = GitDiffGenerator::new(&fixture.repo);
        let diff = generator.diff(
            DiffTarget::Commit(fixture.repo.revparse_single("HEAD~2").unwrap()),
            DiffTarget::Commit(fixture.repo.revparse_single("HEAD").unwrap()),
            &DiffGenerationOptions::default(),
        ).unwrap();

        assert!(!diff.is_empty());
    }
}

#[cfg(test)]
mod stash_tests {
    use super::*;

    #[test]
    fn test_stash_save_pop() {
        let fixture = GitTestFixture::with_initial_commit();

        // Make changes
        fixture.modify_file("README.md", "Modified\n");

        // Stash
        let stasher = GitStasher::new(&fixture.repo);
        let result = stasher.save(StashSaveOptions::new().message("WIP")).unwrap();
        assert!(result.success);

        // File should be reverted
        let content = std::fs::read_to_string(fixture.path().join("README.md")).unwrap();
        assert_eq!(content, "# Test Repository\n");

        // Pop stash
        let pop_result = stasher.pop(StashApplyOptions::new()).unwrap();
        assert!(pop_result.success);

        // File should be modified again
        let content = std::fs::read_to_string(fixture.path().join("README.md")).unwrap();
        assert_eq!(content, "Modified\n");
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn test_clean_status() {
        let fixture = GitTestFixture::with_initial_commit();

        let mut checker = GitStatusChecker::new(&fixture.repo);
        let status = checker.check(&StatusCheckOptions::default()).unwrap();

        assert!(status.summary.is_clean());
    }

    #[test]
    fn test_status_with_changes() {
        let fixture = GitTestFixture::with_initial_commit();

        // Create untracked file
        fixture.create_file("untracked.txt", "content");

        // Modify tracked file
        fixture.modify_file("README.md", "Modified\n");

        // Stage a file
        fixture.create_file("staged.txt", "staged");
        fixture.stage_file("staged.txt");

        let mut checker = GitStatusChecker::new(&fixture.repo);
        let status = checker.check(&StatusCheckOptions::default()).unwrap();

        assert!(!status.summary.is_clean());
        assert_eq!(status.summary.untracked, 1);
        assert_eq!(status.summary.modified, 1);
        assert!(status.summary.staged >= 1);
    }
}
```

### Benchmark Tests

```rust
// src/git/tests/benchmarks.rs

#[cfg(test)]
mod benchmarks {
    use super::super::fixtures::*;
    use crate::git::*;
    use std::time::Instant;

    fn measure<F: FnOnce() -> R, R>(name: &str, f: F) -> R {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        println!("{}: {:?}", name, duration);
        result
    }

    #[test]
    #[ignore] // Run with --ignored for benchmarks
    fn bench_log_large_history() {
        let fixture = GitTestFixture::with_history(1000);

        let logger = GitLogger::new(&fixture.repo);

        measure("Log 1000 commits", || {
            logger.commits(LogOptions::new()).unwrap()
        });
    }

    #[test]
    #[ignore]
    fn bench_status_many_files() {
        let fixture = GitTestFixture::with_initial_commit();

        // Create many files
        for i in 0..1000 {
            fixture.create_file(&format!("file{}.txt", i), &format!("Content {}", i));
        }

        let mut checker = GitStatusChecker::new(&fixture.repo);

        measure("Status with 1000 untracked files", || {
            checker.check(&StatusCheckOptions::default()).unwrap()
        });
    }

    #[test]
    #[ignore]
    fn bench_diff_large_file() {
        let fixture = GitTestFixture::with_initial_commit();

        // Create a large file
        let large_content: String = (0..10000).map(|i| format!("Line {}\n", i)).collect();
        fixture.create_file("large.txt", &large_content);
        fixture.stage_file("large.txt");
        fixture.commit("Add large file");

        // Modify the file
        let modified_content = large_content.replace("Line 5000", "Modified line 5000");
        fixture.modify_file("large.txt", &modified_content);

        let generator = GitDiffGenerator::new(&fixture.repo);

        measure("Diff large file", || {
            generator.unstaged_diff(&DiffGenerationOptions::default()).unwrap()
        });
    }
}
```

### Test Configuration

```rust
// src/git/tests/helpers.rs

use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init_test_env() {
    INIT.call_once(|| {
        // Set up any global test configuration
        env::set_var("GIT_AUTHOR_NAME", "Test User");
        env::set_var("GIT_AUTHOR_EMAIL", "test@example.com");
        env::set_var("GIT_COMMITTER_NAME", "Test User");
        env::set_var("GIT_COMMITTER_EMAIL", "test@example.com");
    });
}

/// Assert that a path exists
#[macro_export]
macro_rules! assert_path_exists {
    ($path:expr) => {
        assert!($path.exists(), "Path should exist: {:?}", $path);
    };
}

/// Assert that a path does not exist
#[macro_export]
macro_rules! assert_path_not_exists {
    ($path:expr) => {
        assert!(!$path.exists(), "Path should not exist: {:?}", $path);
    };
}

/// Run a test with timeout
pub fn with_timeout<F, R>(duration: std::time::Duration, f: F) -> Option<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = f();
        let _ = tx.send(result);
    });

    rx.recv_timeout(duration).ok()
}
```

---

## Testing Requirements

### Running Tests

```bash
# Run all Git tests
cargo test git::tests

# Run integration tests only
cargo test git::tests::integration

# Run benchmarks
cargo test git::tests::benchmarks -- --ignored

# Run with verbose output
cargo test git::tests -- --nocapture
```

### CI Configuration

```yaml
# .github/workflows/git-tests.yml
name: Git Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Configure Git
        run: |
          git config --global user.name "Test User"
          git config --global user.email "test@example.com"

      - name: Run tests
        run: cargo test git::tests

      - name: Run benchmarks
        run: cargo test git::tests::benchmarks -- --ignored
```

---

## Related Specs

- All specs 446-464 (dependencies)
- Spec 001: Core Types
- Spec 010: Error Handling
