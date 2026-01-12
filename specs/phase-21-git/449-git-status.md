# Spec 449: Status Checking

## Phase
21 - Git Integration

## Spec ID
449

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)

## Estimated Context
~9%

---

## Objective

Implement comprehensive Git status checking functionality for Tachikoma, providing detailed information about the working directory state, staged changes, untracked files, and branch tracking status. This module enables users and other components to understand the current state of a repository.

---

## Acceptance Criteria

- [ ] Implement `GitStatusChecker` for status operations
- [ ] Detect staged, unstaged, and untracked files
- [ ] Detect renamed and copied files
- [ ] Support status filtering by path
- [ ] Track branch ahead/behind counts
- [ ] Detect merge conflicts
- [ ] Support ignore pattern handling
- [ ] Implement efficient status caching
- [ ] Provide summary statistics
- [ ] Support async status checking for large repos

---

## Implementation Details

### Status Checker Implementation

```rust
// src/git/status.rs

use git2::{Repository, Status, StatusOptions, StatusShow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::repo::GitRepository;
use super::types::*;

/// Options for status checking
#[derive(Debug, Clone)]
pub struct StatusCheckOptions {
    /// Include untracked files
    pub include_untracked: bool,
    /// Include ignored files
    pub include_ignored: bool,
    /// Recurse into untracked directories
    pub recurse_untracked_dirs: bool,
    /// Only show changes in these paths
    pub pathspecs: Vec<PathBuf>,
    /// Rename detection threshold (0-100, None to disable)
    pub rename_threshold: Option<u16>,
    /// Rename detection limit (number of files to compare)
    pub rename_limit: Option<usize>,
    /// Sort results by path
    pub sort: bool,
    /// Use cached status if available and fresh
    pub use_cache: bool,
    /// Cache TTL
    pub cache_ttl: Duration,
}

impl Default for StatusCheckOptions {
    fn default() -> Self {
        Self {
            include_untracked: true,
            include_ignored: false,
            recurse_untracked_dirs: true,
            pathspecs: Vec::new(),
            rename_threshold: Some(50),
            rename_limit: Some(200),
            sort: true,
            use_cache: true,
            cache_ttl: Duration::from_secs(5),
        }
    }
}

impl StatusCheckOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include_untracked(mut self, include: bool) -> Self {
        self.include_untracked = include;
        self
    }

    pub fn include_ignored(mut self, include: bool) -> Self {
        self.include_ignored = include;
        self
    }

    pub fn pathspec(mut self, path: impl Into<PathBuf>) -> Self {
        self.pathspecs.push(path.into());
        self
    }

    pub fn pathspecs(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.pathspecs.extend(paths);
        self
    }
}

/// Repository status summary
#[derive(Debug, Clone, Default)]
pub struct StatusSummary {
    /// Number of staged files
    pub staged: usize,
    /// Number of modified (unstaged) files
    pub modified: usize,
    /// Number of untracked files
    pub untracked: usize,
    /// Number of ignored files
    pub ignored: usize,
    /// Number of conflicted files
    pub conflicted: usize,
    /// Number of deleted files (staged)
    pub staged_deleted: usize,
    /// Number of deleted files (unstaged)
    pub deleted: usize,
    /// Number of renamed files
    pub renamed: usize,
}

impl StatusSummary {
    /// Check if working directory is clean
    pub fn is_clean(&self) -> bool {
        self.staged == 0
            && self.modified == 0
            && self.untracked == 0
            && self.conflicted == 0
            && self.deleted == 0
    }

    /// Check if there are staged changes ready to commit
    pub fn has_staged_changes(&self) -> bool {
        self.staged > 0 || self.staged_deleted > 0 || self.renamed > 0
    }

    /// Check if there are unstaged changes
    pub fn has_unstaged_changes(&self) -> bool {
        self.modified > 0 || self.deleted > 0
    }

    /// Check if there are conflicts
    pub fn has_conflicts(&self) -> bool {
        self.conflicted > 0
    }

    /// Total number of changed files
    pub fn total_changes(&self) -> usize {
        self.staged + self.modified + self.untracked + self.conflicted + self.deleted
    }
}

/// Complete repository status
#[derive(Debug, Clone)]
pub struct RepositoryStatus {
    /// Current branch name (None if detached HEAD)
    pub branch: Option<String>,
    /// Current HEAD commit
    pub head_oid: Option<GitOid>,
    /// Upstream branch (if tracking)
    pub upstream: Option<String>,
    /// Commits ahead of upstream
    pub ahead: usize,
    /// Commits behind upstream
    pub behind: usize,
    /// Repository state (merging, rebasing, etc.)
    pub state: RepoState,
    /// Status entries
    pub entries: Vec<GitStatusEntry>,
    /// Summary statistics
    pub summary: StatusSummary,
    /// Time when status was computed
    pub computed_at: Instant,
}

impl RepositoryStatus {
    /// Get entries filtered by status
    pub fn entries_with_status(&self, status: GitFileStatus) -> Vec<&GitStatusEntry> {
        self.entries.iter().filter(|e| e.status == status).collect()
    }

    /// Get staged entries
    pub fn staged_entries(&self) -> Vec<&GitStatusEntry> {
        self.entries.iter().filter(|e| e.status.is_staged()).collect()
    }

    /// Get unstaged entries
    pub fn unstaged_entries(&self) -> Vec<&GitStatusEntry> {
        self.entries.iter().filter(|e| e.status.is_unstaged()).collect()
    }

    /// Get conflicted entries
    pub fn conflicted_entries(&self) -> Vec<&GitStatusEntry> {
        self.entries.iter().filter(|e| e.status.is_conflicted()).collect()
    }

    /// Check if a specific file has changes
    pub fn has_changes(&self, path: &Path) -> bool {
        self.entries.iter().any(|e| e.path == path)
    }

    /// Get entry for a specific file
    pub fn get_entry(&self, path: &Path) -> Option<&GitStatusEntry> {
        self.entries.iter().find(|e| e.path == path)
    }
}

/// Git status checker
pub struct GitStatusChecker<'a> {
    repo: &'a GitRepository,
    cache: Option<CachedStatus>,
}

struct CachedStatus {
    status: RepositoryStatus,
    computed_at: Instant,
}

impl<'a> GitStatusChecker<'a> {
    /// Create a new status checker
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo, cache: None }
    }

    /// Check repository status
    pub fn check(&mut self, options: &StatusCheckOptions) -> GitResult<RepositoryStatus> {
        // Check cache
        if options.use_cache {
            if let Some(ref cached) = self.cache {
                if cached.computed_at.elapsed() < options.cache_ttl {
                    return Ok(cached.status.clone());
                }
            }
        }

        let status = self.compute_status(options)?;

        // Update cache
        self.cache = Some(CachedStatus {
            status: status.clone(),
            computed_at: Instant::now(),
        });

        Ok(status)
    }

    /// Force refresh status (bypass cache)
    pub fn refresh(&mut self, options: &StatusCheckOptions) -> GitResult<RepositoryStatus> {
        self.cache = None;
        self.check(options)
    }

    /// Invalidate cache
    pub fn invalidate_cache(&mut self) {
        self.cache = None;
    }

    fn compute_status(&self, options: &StatusCheckOptions) -> GitResult<RepositoryStatus> {
        let raw_repo = self.repo.raw();

        // Build status options
        let mut git_opts = StatusOptions::new();
        git_opts.include_untracked(options.include_untracked);
        git_opts.include_ignored(options.include_ignored);
        git_opts.recurse_untracked_dirs(options.recurse_untracked_dirs);
        git_opts.show(StatusShow::IndexAndWorkdir);

        if let Some(threshold) = options.rename_threshold {
            git_opts.renames_head_to_index(true);
            git_opts.renames_index_to_workdir(true);
            // Note: git2 doesn't expose rename threshold directly
        }

        for path in &options.pathspecs {
            git_opts.pathspec(path);
        }

        // Get statuses
        let statuses = raw_repo.statuses(Some(&mut git_opts))?;

        // Convert to our types
        let mut entries = Vec::new();
        let mut summary = StatusSummary::default();

        for entry in statuses.iter() {
            let path = PathBuf::from(entry.path().unwrap_or(""));
            let status = GitFileStatus::from(entry.status());

            // Update summary
            match status {
                GitFileStatus::IndexNew | GitFileStatus::IndexModified => summary.staged += 1,
                GitFileStatus::IndexDeleted => summary.staged_deleted += 1,
                GitFileStatus::IndexRenamed => summary.renamed += 1,
                GitFileStatus::WorktreeNew => summary.untracked += 1,
                GitFileStatus::WorktreeModified => summary.modified += 1,
                GitFileStatus::WorktreeDeleted => summary.deleted += 1,
                GitFileStatus::Ignored => summary.ignored += 1,
                GitFileStatus::Conflicted => summary.conflicted += 1,
                _ => {}
            }

            entries.push(GitStatusEntry {
                path,
                status,
                head_to_index: None, // Would require diff computation
                index_to_workdir: None,
            });
        }

        if options.sort {
            entries.sort_by(|a, b| a.path.cmp(&b.path));
        }

        // Get branch info
        let (branch, head_oid, upstream, ahead, behind) = self.get_branch_info()?;

        Ok(RepositoryStatus {
            branch,
            head_oid,
            upstream,
            ahead,
            behind,
            state: self.repo.state(),
            entries,
            summary,
            computed_at: Instant::now(),
        })
    }

    fn get_branch_info(&self) -> GitResult<(Option<String>, Option<GitOid>, Option<String>, usize, usize)> {
        let raw_repo = self.repo.raw();

        let head = match raw_repo.head() {
            Ok(h) => h,
            Err(_) => return Ok((None, None, None, 0, 0)),
        };

        let head_oid = head.target().map(GitOid::from);
        let branch = head.shorthand().map(String::from);

        // Get upstream info if this is a branch
        if head.is_branch() {
            if let Some(branch_name) = head.shorthand() {
                if let Ok(local_branch) = raw_repo.find_branch(branch_name, git2::BranchType::Local) {
                    if let Ok(upstream_branch) = local_branch.upstream() {
                        let upstream_name = upstream_branch.name()?.map(String::from);

                        // Calculate ahead/behind
                        if let (Some(local_oid), Some(upstream_oid)) = (
                            head.target(),
                            upstream_branch.get().target(),
                        ) {
                            let (ahead, behind) = raw_repo.graph_ahead_behind(local_oid, upstream_oid)?;
                            return Ok((branch, head_oid, upstream_name, ahead, behind));
                        }

                        return Ok((branch, head_oid, upstream_name, 0, 0));
                    }
                }
            }
        }

        Ok((branch, head_oid, None, 0, 0))
    }
}

/// Quick status check functions
pub fn is_clean(repo: &GitRepository) -> GitResult<bool> {
    let mut checker = GitStatusChecker::new(repo);
    let status = checker.check(&StatusCheckOptions::default())?;
    Ok(status.summary.is_clean())
}

pub fn has_staged_changes(repo: &GitRepository) -> GitResult<bool> {
    let mut checker = GitStatusChecker::new(repo);
    let status = checker.check(&StatusCheckOptions::default())?;
    Ok(status.summary.has_staged_changes())
}

pub fn has_conflicts(repo: &GitRepository) -> GitResult<bool> {
    let mut checker = GitStatusChecker::new(repo);
    let status = checker.check(&StatusCheckOptions::default())?;
    Ok(status.summary.has_conflicts())
}

/// Status watcher for file system changes
pub struct StatusWatcher {
    repo_path: PathBuf,
    last_check: Option<Instant>,
    debounce: Duration,
}

impl StatusWatcher {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            last_check: None,
            debounce: Duration::from_millis(100),
        }
    }

    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    /// Check if enough time has passed for a new status check
    pub fn should_check(&self) -> bool {
        match self.last_check {
            None => true,
            Some(last) => last.elapsed() >= self.debounce,
        }
    }

    /// Mark that a check was performed
    pub fn mark_checked(&mut self) {
        self.last_check = Some(Instant::now());
    }
}

/// Format status for display
pub struct StatusFormatter;

impl StatusFormatter {
    /// Format status as short string (like git status -s)
    pub fn format_short(entry: &GitStatusEntry) -> String {
        let index_char = match entry.status {
            GitFileStatus::IndexNew => 'A',
            GitFileStatus::IndexModified => 'M',
            GitFileStatus::IndexDeleted => 'D',
            GitFileStatus::IndexRenamed => 'R',
            GitFileStatus::IndexTypechange => 'T',
            _ => ' ',
        };

        let worktree_char = match entry.status {
            GitFileStatus::WorktreeNew => '?',
            GitFileStatus::WorktreeModified => 'M',
            GitFileStatus::WorktreeDeleted => 'D',
            GitFileStatus::WorktreeRenamed => 'R',
            GitFileStatus::WorktreeTypechange => 'T',
            GitFileStatus::Conflicted => 'U',
            GitFileStatus::Ignored => '!',
            _ if entry.status.is_staged() => ' ',
            _ => ' ',
        };

        format!("{}{} {}", index_char, worktree_char, entry.path.display())
    }

    /// Format complete status summary
    pub fn format_summary(status: &RepositoryStatus) -> String {
        let mut lines = Vec::new();

        // Branch line
        if let Some(ref branch) = status.branch {
            let mut branch_line = format!("On branch {}", branch);
            if let Some(ref upstream) = status.upstream {
                branch_line.push_str(&format!(" (tracking {})", upstream));
            }
            lines.push(branch_line);

            if status.ahead > 0 || status.behind > 0 {
                lines.push(format!(
                    "Your branch is {} ahead, {} behind",
                    status.ahead, status.behind
                ));
            }
        } else {
            lines.push("HEAD detached".to_string());
        }

        // State
        if !status.state.is_clean() {
            lines.push(format!("Currently {}", status.state.description()));
        }

        lines.push(String::new());

        // Changes
        if status.summary.is_clean() {
            lines.push("Nothing to commit, working tree clean".to_string());
        } else {
            if status.summary.has_staged_changes() {
                lines.push("Changes to be committed:".to_string());
                for entry in status.staged_entries() {
                    lines.push(format!("  {}", Self::format_short(entry)));
                }
                lines.push(String::new());
            }

            if status.summary.has_unstaged_changes() {
                lines.push("Changes not staged for commit:".to_string());
                for entry in status.unstaged_entries() {
                    lines.push(format!("  {}", Self::format_short(entry)));
                }
                lines.push(String::new());
            }

            if status.summary.untracked > 0 {
                lines.push("Untracked files:".to_string());
                for entry in status.entries_with_status(GitFileStatus::WorktreeNew) {
                    lines.push(format!("  {}", entry.path.display()));
                }
            }
        }

        lines.join("\n")
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();
        (dir, repo)
    }

    fn create_initial_commit(dir: &TempDir, repo: &GitRepository) {
        // Create and stage a file
        std::fs::write(dir.path().join("README.md"), "# Test").unwrap();
        repo.stage_file(Path::new("README.md")).unwrap();

        // Create commit using raw repo
        let raw = repo.raw();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
    }

    #[test]
    fn test_status_clean_repo() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        let mut checker = GitStatusChecker::new(&repo);
        let status = checker.check(&StatusCheckOptions::default()).unwrap();

        assert!(status.summary.is_clean());
        assert!(!status.summary.has_conflicts());
    }

    #[test]
    fn test_status_untracked_files() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        // Create untracked file
        std::fs::write(dir.path().join("new_file.txt"), "content").unwrap();

        let mut checker = GitStatusChecker::new(&repo);
        let status = checker.check(&StatusCheckOptions::default()).unwrap();

        assert_eq!(status.summary.untracked, 1);
        assert!(!status.summary.is_clean());
    }

    #[test]
    fn test_status_staged_changes() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        // Create and stage new file
        std::fs::write(dir.path().join("staged.txt"), "staged content").unwrap();
        repo.stage_file(Path::new("staged.txt")).unwrap();

        let mut checker = GitStatusChecker::new(&repo);
        let status = checker.check(&StatusCheckOptions::default()).unwrap();

        assert!(status.summary.has_staged_changes());
        assert_eq!(status.summary.staged, 1);
    }

    #[test]
    fn test_status_modified_files() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        // Modify existing file
        std::fs::write(dir.path().join("README.md"), "# Modified").unwrap();

        let mut checker = GitStatusChecker::new(&repo);
        let status = checker.check(&StatusCheckOptions::default()).unwrap();

        assert!(status.summary.has_unstaged_changes());
        assert_eq!(status.summary.modified, 1);
    }

    #[test]
    fn test_status_caching() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        let mut checker = GitStatusChecker::new(&repo);

        let status1 = checker.check(&StatusCheckOptions::default()).unwrap();
        let status2 = checker.check(&StatusCheckOptions::default()).unwrap();

        // Same computed_at means cache was used
        assert_eq!(status1.computed_at, status2.computed_at);
    }

    #[test]
    fn test_status_cache_invalidation() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        let mut checker = GitStatusChecker::new(&repo);

        let status1 = checker.check(&StatusCheckOptions::default()).unwrap();
        checker.invalidate_cache();
        let status2 = checker.check(&StatusCheckOptions::default()).unwrap();

        // Different computed_at means cache was invalidated
        assert_ne!(status1.computed_at, status2.computed_at);
    }

    #[test]
    fn test_status_summary() {
        let summary = StatusSummary {
            staged: 2,
            modified: 1,
            untracked: 3,
            ignored: 0,
            conflicted: 0,
            staged_deleted: 1,
            deleted: 0,
            renamed: 1,
        };

        assert!(!summary.is_clean());
        assert!(summary.has_staged_changes());
        assert!(summary.has_unstaged_changes());
        assert_eq!(summary.total_changes(), 6);
    }

    #[test]
    fn test_format_short() {
        let entry = GitStatusEntry {
            path: PathBuf::from("test.txt"),
            status: GitFileStatus::IndexNew,
            head_to_index: None,
            index_to_workdir: None,
        };

        let formatted = StatusFormatter::format_short(&entry);
        assert!(formatted.starts_with("A "));
        assert!(formatted.contains("test.txt"));
    }

    #[test]
    fn test_status_watcher() {
        let watcher = StatusWatcher::new("/tmp/repo");
        assert!(watcher.should_check());

        let mut watcher = watcher;
        watcher.mark_checked();
        assert!(!watcher.should_check()); // Within debounce period
    }

    #[test]
    fn test_is_clean_helper() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        assert!(is_clean(&repo).unwrap());
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 450: Diff Generation
- Spec 451: Commit Operations
