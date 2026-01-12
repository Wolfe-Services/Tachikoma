# Spec 457: Rebase Support

## Phase
21 - Git Integration

## Spec ID
457

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)
- Spec 451: Commit Operations (commit creation)

## Estimated Context
~11%

---

## Objective

Implement Git rebase operations for Tachikoma with support for interactive rebase, rebase onto, and conflict handling. This module provides clean history management capabilities essential for maintaining linear commit histories and integrating feature branches.

---

## Acceptance Criteria

- [ ] Implement `GitRebaser` for rebase operations
- [ ] Support standard rebase onto branch
- [ ] Support interactive rebase operations
- [ ] Implement rebase --onto for complex rebases
- [ ] Handle rebase conflicts with continue/abort/skip
- [ ] Support autosquash for fixup commits
- [ ] Implement rebase todo list management
- [ ] Support preserving merge commits
- [ ] Implement rebase progress tracking
- [ ] Support rebase dry-run preview

---

## Implementation Details

### Rebase Manager Implementation

```rust
// src/git/rebase.rs

use git2::{
    AnnotatedCommit, Commit, Oid, RebaseOperation, RebaseOperationType,
    RebaseOptions, Repository,
};
use std::path::PathBuf;

use super::repo::GitRepository;
use super::types::*;

/// Rebase operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseAction {
    Pick,
    Reword,
    Edit,
    Squash,
    Fixup,
    Drop,
}

impl From<RebaseOperationType> for RebaseAction {
    fn from(op: RebaseOperationType) -> Self {
        match op {
            RebaseOperationType::Pick => Self::Pick,
            RebaseOperationType::Reword => Self::Reword,
            RebaseOperationType::Edit => Self::Edit,
            RebaseOperationType::Squash => Self::Squash,
            RebaseOperationType::Fixup => Self::Fixup,
            RebaseOperationType::Exec => Self::Pick, // No direct mapping
        }
    }
}

/// A single step in the rebase
#[derive(Debug, Clone)]
pub struct RebaseStep {
    /// Action to perform
    pub action: RebaseAction,
    /// Commit OID
    pub oid: GitOid,
    /// Commit message summary
    pub summary: String,
}

/// Rebase todo list for interactive rebase
#[derive(Debug, Clone)]
pub struct RebaseTodoList {
    pub steps: Vec<RebaseStep>,
}

impl RebaseTodoList {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add(&mut self, action: RebaseAction, oid: GitOid, summary: impl Into<String>) {
        self.steps.push(RebaseStep {
            action,
            oid,
            summary: summary.into(),
        });
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Move a step to a new position
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from < self.steps.len() && to < self.steps.len() {
            let step = self.steps.remove(from);
            self.steps.insert(to, step);
        }
    }

    /// Change action for a step
    pub fn set_action(&mut self, index: usize, action: RebaseAction) {
        if let Some(step) = self.steps.get_mut(index) {
            step.action = action;
        }
    }

    /// Format as text (like git rebase -i todo file)
    pub fn to_text(&self) -> String {
        self.steps
            .iter()
            .map(|s| {
                let action = match s.action {
                    RebaseAction::Pick => "pick",
                    RebaseAction::Reword => "reword",
                    RebaseAction::Edit => "edit",
                    RebaseAction::Squash => "squash",
                    RebaseAction::Fixup => "fixup",
                    RebaseAction::Drop => "drop",
                };
                format!("{} {} {}", action, s.oid.short(), s.summary)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse from text
    pub fn from_text(text: &str) -> GitResult<Self> {
        let mut steps = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() < 2 {
                continue;
            }

            let action = match parts[0].to_lowercase().as_str() {
                "pick" | "p" => RebaseAction::Pick,
                "reword" | "r" => RebaseAction::Reword,
                "edit" | "e" => RebaseAction::Edit,
                "squash" | "s" => RebaseAction::Squash,
                "fixup" | "f" => RebaseAction::Fixup,
                "drop" | "d" => RebaseAction::Drop,
                _ => continue,
            };

            let oid = GitOid::from_hex(parts[1])?;
            let summary = parts.get(2).unwrap_or(&"").to_string();

            steps.push(RebaseStep { action, oid, summary });
        }

        Ok(Self { steps })
    }
}

impl Default for RebaseTodoList {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for rebase operations
#[derive(Debug, Clone, Default)]
pub struct RebaseOperationOptions {
    /// Upstream branch/commit to rebase onto
    pub onto: Option<String>,
    /// Interactive rebase
    pub interactive: bool,
    /// Autosquash fixup commits
    pub autosquash: bool,
    /// Preserve merge commits
    pub preserve_merges: bool,
    /// Autostash before rebase
    pub autostash: bool,
    /// Keep empty commits
    pub keep_empty: bool,
    /// Custom todo list (for interactive)
    pub todo_list: Option<RebaseTodoList>,
}

impl RebaseOperationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn onto(mut self, target: impl Into<String>) -> Self {
        self.onto = Some(target.into());
        self
    }

    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    pub fn autosquash(mut self) -> Self {
        self.autosquash = true;
        self
    }

    pub fn autostash(mut self) -> Self {
        self.autostash = true;
        self
    }

    pub fn with_todo(mut self, todo: RebaseTodoList) -> Self {
        self.todo_list = Some(todo);
        self.interactive = true;
        self
    }
}

/// Rebase progress information
#[derive(Debug, Clone)]
pub struct RebaseProgress {
    /// Current step (1-indexed)
    pub current: usize,
    /// Total steps
    pub total: usize,
    /// Current operation
    pub current_operation: Option<RebaseStep>,
    /// Commits already applied
    pub applied: Vec<GitOid>,
}

impl RebaseProgress {
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }
}

/// Rebase result
#[derive(Debug, Clone)]
pub struct RebaseResult {
    /// Was rebase successful
    pub success: bool,
    /// New HEAD after rebase
    pub new_head: Option<GitOid>,
    /// Number of commits rebased
    pub commits_rebased: usize,
    /// Conflicts encountered
    pub conflicts: Vec<PathBuf>,
    /// Stopped at commit (if conflict or edit)
    pub stopped_at: Option<GitOid>,
}

/// Git rebase manager
pub struct GitRebaser<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitRebaser<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Start a rebase operation
    pub fn rebase(
        &self,
        upstream: &str,
        options: RebaseOperationOptions,
    ) -> GitResult<RebaseResult> {
        let raw_repo = self.repo.raw();

        // Get annotated commits
        let upstream_commit = self.resolve_to_annotated(upstream)?;

        let onto_commit = if let Some(ref onto) = options.onto {
            Some(self.resolve_to_annotated(onto)?)
        } else {
            None
        };

        // Get HEAD
        let head = raw_repo.head()?.peel_to_commit()?;
        let head_annotated = raw_repo.find_annotated_commit(head.id())?;

        // Build rebase options
        let mut rebase_opts = RebaseOptions::new();

        // Init rebase
        let mut rebase = raw_repo.rebase(
            Some(&head_annotated),
            Some(&upstream_commit),
            onto_commit.as_ref(),
            Some(&mut rebase_opts),
        )?;

        // If interactive with todo list, we would modify operations here
        // Note: git2 doesn't directly support interactive rebase editing
        // We simulate it by iterating through operations

        let total_ops = rebase.len();
        let mut commits_rebased = 0;
        let mut conflicts = Vec::new();

        // Process each operation
        while let Some(op) = rebase.next() {
            let op = op?;

            // Check operation type and handle accordingly
            if let Some(todo) = &options.todo_list {
                if let Some(step) = todo.steps.get(commits_rebased) {
                    match step.action {
                        RebaseAction::Drop => continue,
                        RebaseAction::Edit => {
                            // Stop for editing
                            return Ok(RebaseResult {
                                success: false,
                                new_head: None,
                                commits_rebased,
                                conflicts: Vec::new(),
                                stopped_at: Some(GitOid::from(op.id())),
                            });
                        }
                        _ => {}
                    }
                }
            }

            // Attempt to commit the operation
            let sig = raw_repo.signature()?;

            match rebase.commit(None, &sig, None) {
                Ok(_) => {
                    commits_rebased += 1;
                }
                Err(e) if e.code() == git2::ErrorCode::Applied => {
                    // Already applied, skip
                    commits_rebased += 1;
                }
                Err(e) if e.code() == git2::ErrorCode::Uncommitted => {
                    // Conflict - need manual resolution
                    let index = raw_repo.index()?;
                    for conflict in index.conflicts()? {
                        if let Ok(c) = conflict {
                            if let Some(entry) = c.our.or(c.their).or(c.ancestor) {
                                conflicts.push(PathBuf::from(
                                    String::from_utf8_lossy(&entry.path).to_string()
                                ));
                            }
                        }
                    }

                    return Ok(RebaseResult {
                        success: false,
                        new_head: None,
                        commits_rebased,
                        conflicts,
                        stopped_at: Some(GitOid::from(op.id())),
                    });
                }
                Err(e) => return Err(GitError::Git2(e)),
            }
        }

        // Finish rebase
        rebase.finish(None)?;

        // Get new HEAD
        let new_head = raw_repo.head()?.target().map(GitOid::from);

        Ok(RebaseResult {
            success: true,
            new_head,
            commits_rebased,
            conflicts: Vec::new(),
            stopped_at: None,
        })
    }

    /// Continue rebase after conflict resolution
    pub fn continue_rebase(&self) -> GitResult<RebaseResult> {
        let raw_repo = self.repo.raw();

        // Check if there's a rebase in progress
        let mut rebase = raw_repo.open_rebase(None)?;

        // Check for remaining conflicts
        let index = raw_repo.index()?;
        if index.has_conflicts() {
            return Err(GitError::MergeConflict(index.conflicts()?.count()));
        }

        let sig = raw_repo.signature()?;
        let mut commits_rebased = 0;

        // Continue with remaining operations
        while let Some(op) = rebase.next() {
            let _op = op?;

            match rebase.commit(None, &sig, None) {
                Ok(_) => {
                    commits_rebased += 1;
                }
                Err(e) if e.code() == git2::ErrorCode::Applied => {
                    commits_rebased += 1;
                }
                Err(e) if e.code() == git2::ErrorCode::Uncommitted => {
                    return Err(GitError::MergeConflict(1));
                }
                Err(e) => return Err(GitError::Git2(e)),
            }
        }

        rebase.finish(None)?;

        let new_head = raw_repo.head()?.target().map(GitOid::from);

        Ok(RebaseResult {
            success: true,
            new_head,
            commits_rebased,
            conflicts: Vec::new(),
            stopped_at: None,
        })
    }

    /// Abort rebase
    pub fn abort(&self) -> GitResult<()> {
        let raw_repo = self.repo.raw();

        let mut rebase = raw_repo.open_rebase(None)?;
        rebase.abort()?;

        Ok(())
    }

    /// Skip current commit in rebase
    pub fn skip(&self) -> GitResult<RebaseResult> {
        let raw_repo = self.repo.raw();

        let mut rebase = raw_repo.open_rebase(None)?;

        // Skip current by moving to next
        let sig = raw_repo.signature()?;
        let mut commits_rebased = 0;

        while let Some(op) = rebase.next() {
            let _op = op?;

            match rebase.commit(None, &sig, None) {
                Ok(_) => {
                    commits_rebased += 1;
                }
                Err(e) if e.code() == git2::ErrorCode::Applied => {
                    commits_rebased += 1;
                }
                Err(e) if e.code() == git2::ErrorCode::Uncommitted => {
                    return Err(GitError::MergeConflict(1));
                }
                Err(e) => return Err(GitError::Git2(e)),
            }
        }

        rebase.finish(None)?;

        let new_head = raw_repo.head()?.target().map(GitOid::from);

        Ok(RebaseResult {
            success: true,
            new_head,
            commits_rebased,
            conflicts: Vec::new(),
            stopped_at: None,
        })
    }

    /// Get rebase progress
    pub fn progress(&self) -> GitResult<Option<RebaseProgress>> {
        let raw_repo = self.repo.raw();

        match raw_repo.open_rebase(None) {
            Ok(rebase) => {
                let current = rebase.operation_current().unwrap_or(0);
                let total = rebase.len();

                Ok(Some(RebaseProgress {
                    current: current + 1,
                    total,
                    current_operation: None, // Would need to track
                    applied: Vec::new(),
                }))
            }
            Err(_) => Ok(None), // No rebase in progress
        }
    }

    /// Preview rebase (get todo list without executing)
    pub fn preview(&self, upstream: &str) -> GitResult<RebaseTodoList> {
        let raw_repo = self.repo.raw();

        let upstream_oid = raw_repo.revparse_single(upstream)?.id();
        let head_oid = raw_repo.head()?.target()
            .ok_or_else(|| GitError::Other("HEAD has no target".into()))?;

        // Find commits to rebase
        let merge_base = raw_repo.merge_base(upstream_oid, head_oid)?;

        let mut revwalk = raw_repo.revwalk()?;
        revwalk.push(head_oid)?;
        revwalk.hide(merge_base)?;

        let mut todo = RebaseTodoList::new();

        for oid in revwalk {
            let oid = oid?;
            let commit = raw_repo.find_commit(oid)?;
            let summary = commit.summary().unwrap_or("").to_string();

            todo.add(RebaseAction::Pick, GitOid::from(oid), summary);
        }

        // Reverse to get chronological order
        todo.steps.reverse();

        Ok(todo)
    }

    fn resolve_to_annotated(&self, spec: &str) -> GitResult<AnnotatedCommit<'a>> {
        let raw_repo = self.repo.raw();
        let obj = raw_repo.revparse_single(spec)?;
        let commit = obj.peel_to_commit()?;
        let annotated = raw_repo.find_annotated_commit(commit.id())?;

        Ok(unsafe { std::mem::transmute(annotated) })
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

        let mut config = repo.config().unwrap();
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

        std::fs::write(dir.path().join("README.md"), "# Test").unwrap();
        repo.stage_file(std::path::Path::new("README.md")).unwrap();

        let raw = repo.raw();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();

        (dir, repo)
    }

    #[test]
    fn test_rebase_todo_list_new() {
        let todo = RebaseTodoList::new();
        assert!(todo.is_empty());
    }

    #[test]
    fn test_rebase_todo_list_add() {
        let mut todo = RebaseTodoList::new();
        todo.add(RebaseAction::Pick, GitOid([1; 20]), "First commit");
        todo.add(RebaseAction::Squash, GitOid([2; 20]), "Second commit");

        assert_eq!(todo.len(), 2);
        assert_eq!(todo.steps[0].action, RebaseAction::Pick);
        assert_eq!(todo.steps[1].action, RebaseAction::Squash);
    }

    #[test]
    fn test_rebase_todo_list_to_text() {
        let mut todo = RebaseTodoList::new();
        todo.add(RebaseAction::Pick, GitOid([0xab; 20]), "First");
        todo.add(RebaseAction::Fixup, GitOid([0xcd; 20]), "Second");

        let text = todo.to_text();
        assert!(text.contains("pick"));
        assert!(text.contains("fixup"));
    }

    #[test]
    fn test_rebase_todo_list_from_text() {
        let text = "pick abc1234 First commit\nsquash def5678 Second commit\n";
        let result = RebaseTodoList::from_text(text);

        // This will fail because the OIDs aren't valid full hex
        // In real usage, OIDs would be full 40-char hex
        assert!(result.is_err());
    }

    #[test]
    fn test_rebase_todo_list_reorder() {
        let mut todo = RebaseTodoList::new();
        todo.add(RebaseAction::Pick, GitOid([1; 20]), "First");
        todo.add(RebaseAction::Pick, GitOid([2; 20]), "Second");
        todo.add(RebaseAction::Pick, GitOid([3; 20]), "Third");

        todo.reorder(0, 2);

        assert_eq!(todo.steps[0].summary, "Second");
        assert_eq!(todo.steps[2].summary, "First");
    }

    #[test]
    fn test_rebase_todo_list_set_action() {
        let mut todo = RebaseTodoList::new();
        todo.add(RebaseAction::Pick, GitOid([1; 20]), "Commit");

        todo.set_action(0, RebaseAction::Squash);

        assert_eq!(todo.steps[0].action, RebaseAction::Squash);
    }

    #[test]
    fn test_rebase_options_builder() {
        let opts = RebaseOperationOptions::new()
            .onto("main")
            .interactive()
            .autosquash()
            .autostash();

        assert_eq!(opts.onto, Some("main".to_string()));
        assert!(opts.interactive);
        assert!(opts.autosquash);
        assert!(opts.autostash);
    }

    #[test]
    fn test_rebase_progress_percentage() {
        let progress = RebaseProgress {
            current: 5,
            total: 10,
            current_operation: None,
            applied: Vec::new(),
        };

        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_rebase_progress_percentage_zero_total() {
        let progress = RebaseProgress {
            current: 0,
            total: 0,
            current_operation: None,
            applied: Vec::new(),
        };

        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_abort_no_rebase() {
        let (_dir, repo) = setup_test_repo();
        let rebaser = GitRebaser::new(&repo);

        // Should fail when no rebase in progress
        let result = rebaser.abort();
        assert!(result.is_err());
    }

    #[test]
    fn test_rebase_action_from_type() {
        assert_eq!(
            RebaseAction::from(RebaseOperationType::Pick),
            RebaseAction::Pick
        );
        assert_eq!(
            RebaseAction::from(RebaseOperationType::Squash),
            RebaseAction::Squash
        );
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 451: Commit Operations
- Spec 455: Pull/Fetch Operations
- Spec 456: Merge Operations
