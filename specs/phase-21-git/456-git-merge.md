# Spec 456: Merge Operations

## Phase
21 - Git Integration

## Spec ID
456

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)
- Spec 450: Diff Generation (for merge preview)

## Estimated Context
~11%

---

## Objective

Implement Git merge operations for Tachikoma with support for multiple merge strategies, conflict detection and resolution, and merge preview capabilities. This module handles both fast-forward and three-way merges with comprehensive conflict management.

---

## Acceptance Criteria

- [ ] Implement `GitMerger` for merge operations
- [ ] Support fast-forward merges
- [ ] Support three-way merges
- [ ] Implement multiple merge strategies (recursive, ours, theirs)
- [ ] Detect and report merge conflicts
- [ ] Support conflict resolution helpers
- [ ] Implement merge abort/continue
- [ ] Support squash merge
- [ ] Implement merge preview (dry run)
- [ ] Support merge commit message customization

---

## Implementation Details

### Merge Manager Implementation

```rust
// src/git/merge.rs

use git2::{
    AnnotatedCommit, Commit, Index, MergeOptions, ObjectType,
    Repository, Signature,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::repo::GitRepository;
use super::types::*;

/// Merge strategy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MergeStrategy {
    #[default]
    Recursive,
    Ours,
    Theirs,
    Octopus,
}

/// Merge options
#[derive(Debug, Clone)]
pub struct MergeOperationOptions {
    /// Merge strategy
    pub strategy: MergeStrategy,
    /// Custom merge commit message
    pub message: Option<String>,
    /// No fast-forward (always create merge commit)
    pub no_ff: bool,
    /// Fast-forward only
    pub ff_only: bool,
    /// Squash merge
    pub squash: bool,
    /// Allow merging unrelated histories
    pub allow_unrelated: bool,
    /// Commit the merge automatically
    pub commit: bool,
    /// Conflict style (merge, diff3)
    pub conflict_style: ConflictStyle,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ConflictStyle {
    #[default]
    Merge,
    Diff3,
}

impl Default for MergeOperationOptions {
    fn default() -> Self {
        Self {
            strategy: MergeStrategy::default(),
            message: None,
            no_ff: false,
            ff_only: false,
            squash: false,
            allow_unrelated: false,
            commit: true,
            conflict_style: ConflictStyle::default(),
        }
    }
}

impl MergeOperationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn strategy(mut self, strategy: MergeStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn no_ff(mut self) -> Self {
        self.no_ff = true;
        self
    }

    pub fn ff_only(mut self) -> Self {
        self.ff_only = true;
        self
    }

    pub fn squash(mut self) -> Self {
        self.squash = true;
        self.commit = false;
        self
    }

    pub fn no_commit(mut self) -> Self {
        self.commit = false;
        self
    }
}

/// Merge analysis result
#[derive(Debug, Clone)]
pub struct MergeAnalysis {
    /// Is already up to date
    pub up_to_date: bool,
    /// Can fast-forward
    pub fast_forward: bool,
    /// Normal merge required
    pub normal: bool,
    /// Unborn branch
    pub unborn: bool,
    /// Base commit (merge base)
    pub base: Option<GitOid>,
    /// Commits to merge
    pub commits_to_merge: usize,
}

/// Merge conflict information
#[derive(Debug, Clone)]
pub struct MergeConflictInfo {
    /// Path of conflicted file
    pub path: PathBuf,
    /// Ancestor (base) content
    pub ancestor: Option<ConflictEntry>,
    /// Ours (current branch) content
    pub ours: Option<ConflictEntry>,
    /// Theirs (merge branch) content
    pub theirs: Option<ConflictEntry>,
    /// Conflict markers present in file
    pub has_markers: bool,
}

#[derive(Debug, Clone)]
pub struct ConflictEntry {
    pub oid: GitOid,
    pub mode: u32,
}

/// Merge result
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// Was merge successful
    pub success: bool,
    /// Resulting commit (if committed)
    pub commit: Option<GitOid>,
    /// Was fast-forward
    pub fast_forward: bool,
    /// Files merged
    pub files_merged: usize,
    /// Conflicts encountered
    pub conflicts: Vec<MergeConflictInfo>,
    /// Was squash merge
    pub squashed: bool,
}

impl MergeResult {
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
}

/// Git merge manager
pub struct GitMerger<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitMerger<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Analyze merge before performing it
    pub fn analyze(&self, source: &str) -> GitResult<MergeAnalysis> {
        let raw_repo = self.repo.raw();

        let annotated = self.resolve_to_annotated(source)?;
        let (analysis, _preference) = raw_repo.merge_analysis(&[&annotated])?;

        // Find merge base
        let head_oid = raw_repo.head()?.target()
            .ok_or_else(|| GitError::Other("HEAD has no target".into()))?;
        let source_oid = annotated.id();

        let base = raw_repo.merge_base(head_oid, source_oid).ok().map(GitOid::from);

        // Count commits to merge
        let commits_to_merge = if let Some(base_oid) = base.as_ref() {
            self.count_commits_between(&base_oid.to_git2_oid(), &source_oid)?
        } else {
            0
        };

        Ok(MergeAnalysis {
            up_to_date: analysis.is_up_to_date(),
            fast_forward: analysis.is_fast_forward(),
            normal: analysis.is_normal(),
            unborn: analysis.is_unborn(),
            base,
            commits_to_merge,
        })
    }

    /// Merge a branch or commit
    pub fn merge(&self, source: &str, options: MergeOperationOptions) -> GitResult<MergeResult> {
        let raw_repo = self.repo.raw();

        // Analyze first
        let analysis = self.analyze(source)?;

        // Check if up to date
        if analysis.up_to_date {
            return Ok(MergeResult {
                success: true,
                commit: self.repo.head()?.target,
                fast_forward: false,
                files_merged: 0,
                conflicts: Vec::new(),
                squashed: false,
            });
        }

        // Check ff-only constraint
        if options.ff_only && !analysis.fast_forward {
            return Err(GitError::Other(
                "Cannot fast-forward, branches have diverged".into()
            ));
        }

        // Perform appropriate merge type
        let annotated = self.resolve_to_annotated(source)?;

        if analysis.fast_forward && !options.no_ff && !options.squash {
            self.fast_forward_merge(&annotated)
        } else if options.squash {
            self.squash_merge(&annotated, &options)
        } else {
            self.three_way_merge(&annotated, &options)
        }
    }

    /// Abort an in-progress merge
    pub fn abort(&self) -> GitResult<()> {
        let raw_repo = self.repo.raw();

        // Check if there's a merge in progress
        if raw_repo.state() != git2::RepositoryState::Merge {
            return Err(GitError::Other("No merge in progress".into()));
        }

        // Reset to HEAD
        let head = raw_repo.head()?.peel_to_commit()?;
        raw_repo.reset(&head.into_object(), git2::ResetType::Hard, None)?;

        // Cleanup merge state
        raw_repo.cleanup_state()?;

        Ok(())
    }

    /// Continue merge after conflict resolution
    pub fn continue_merge(&self, message: Option<&str>) -> GitResult<MergeResult> {
        let raw_repo = self.repo.raw();

        // Check index for conflicts
        let index = raw_repo.index()?;
        if index.has_conflicts() {
            return Err(GitError::MergeConflict(
                index.conflicts()?.count()
            ));
        }

        // Read MERGE_HEAD
        let merge_head_path = raw_repo.path().join("MERGE_HEAD");
        if !merge_head_path.exists() {
            return Err(GitError::Other("No merge in progress".into()));
        }

        let merge_head_content = std::fs::read_to_string(&merge_head_path)?;
        let merge_oid = git2::Oid::from_str(merge_head_content.trim())?;
        let merge_commit = raw_repo.find_commit(merge_oid)?;

        // Create merge commit
        let head_commit = raw_repo.head()?.peel_to_commit()?;
        let sig = raw_repo.signature()?;

        let mut index = raw_repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = raw_repo.find_tree(tree_id)?;

        let default_msg = format!(
            "Merge commit '{}'",
            merge_oid.to_string().chars().take(7).collect::<String>()
        );
        let message = message.unwrap_or(&default_msg);

        let commit_oid = raw_repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&head_commit, &merge_commit],
        )?;

        // Cleanup
        raw_repo.cleanup_state()?;

        Ok(MergeResult {
            success: true,
            commit: Some(GitOid::from(commit_oid)),
            fast_forward: false,
            files_merged: 0,
            conflicts: Vec::new(),
            squashed: false,
        })
    }

    /// Get current conflicts
    pub fn conflicts(&self) -> GitResult<Vec<MergeConflictInfo>> {
        let raw_repo = self.repo.raw();
        let index = raw_repo.index()?;

        let mut conflicts = Vec::new();

        for conflict in index.conflicts()? {
            let conflict = conflict?;

            let path = conflict.our
                .as_ref()
                .or(conflict.their.as_ref())
                .or(conflict.ancestor.as_ref())
                .map(|e| PathBuf::from(String::from_utf8_lossy(&e.path).to_string()))
                .unwrap_or_default();

            conflicts.push(MergeConflictInfo {
                path,
                ancestor: conflict.ancestor.map(|e| ConflictEntry {
                    oid: GitOid::from(e.id),
                    mode: e.mode,
                }),
                ours: conflict.our.map(|e| ConflictEntry {
                    oid: GitOid::from(e.id),
                    mode: e.mode,
                }),
                theirs: conflict.their.map(|e| ConflictEntry {
                    oid: GitOid::from(e.id),
                    mode: e.mode,
                }),
                has_markers: true, // Would need to check file content
            });
        }

        Ok(conflicts)
    }

    /// Resolve a conflict by choosing a side
    pub fn resolve_conflict(&self, path: &Path, resolution: ConflictResolution) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        let mut index = raw_repo.index()?;

        match resolution {
            ConflictResolution::Ours => {
                // Get ours entry and add it
                let conflicts: Vec<_> = index.conflicts()?.collect();
                for conflict in conflicts {
                    let conflict = conflict?;
                    if let Some(entry) = conflict.our {
                        let entry_path = String::from_utf8_lossy(&entry.path);
                        if entry_path == path.to_string_lossy() {
                            index.remove_path(path)?;
                            // Re-add the file from our side
                            index.add_path(path)?;
                            break;
                        }
                    }
                }
            }
            ConflictResolution::Theirs => {
                // Similar but use theirs
                let workdir = raw_repo.workdir()
                    .ok_or_else(|| GitError::Other("No workdir".into()))?;

                // Get theirs content from the blob
                let conflicts: Vec<_> = index.conflicts()?.collect();
                for conflict in conflicts {
                    let conflict = conflict?;
                    if let Some(entry) = conflict.their {
                        let entry_path = String::from_utf8_lossy(&entry.path);
                        if entry_path == path.to_string_lossy() {
                            let blob = raw_repo.find_blob(entry.id)?;
                            std::fs::write(workdir.join(path), blob.content())?;
                            index.remove_path(path)?;
                            index.add_path(path)?;
                            break;
                        }
                    }
                }
            }
            ConflictResolution::Manual => {
                // Just mark as resolved by adding the current file content
                index.remove_path(path)?;
                index.add_path(path)?;
            }
        }

        index.write()?;
        Ok(())
    }

    fn fast_forward_merge(&self, target: &AnnotatedCommit) -> GitResult<MergeResult> {
        let raw_repo = self.repo.raw();

        let mut reference = raw_repo.head()?;
        let refname = reference.name()
            .ok_or_else(|| GitError::Other("Cannot get HEAD name".into()))?
            .to_string();

        reference.set_target(target.id(), "Fast-forward merge")?;

        raw_repo.checkout_head(Some(
            git2::build::CheckoutBuilder::new().force()
        ))?;

        Ok(MergeResult {
            success: true,
            commit: Some(GitOid::from(target.id())),
            fast_forward: true,
            files_merged: 0,
            conflicts: Vec::new(),
            squashed: false,
        })
    }

    fn squash_merge(&self, target: &AnnotatedCommit, options: &MergeOperationOptions) -> GitResult<MergeResult> {
        let raw_repo = self.repo.raw();

        // Perform merge without committing
        let mut merge_opts = MergeOptions::new();
        raw_repo.merge(&[target], Some(&mut merge_opts), None)?;

        // Check for conflicts
        let index = raw_repo.index()?;
        if index.has_conflicts() {
            let conflicts = self.conflicts()?;
            return Ok(MergeResult {
                success: false,
                commit: None,
                fast_forward: false,
                files_merged: 0,
                conflicts,
                squashed: true,
            });
        }

        // Stage all changes but don't commit
        // The squash merge leaves changes staged for the user to commit

        // Cleanup merge state but keep index
        raw_repo.cleanup_state()?;

        Ok(MergeResult {
            success: true,
            commit: None, // Squash doesn't auto-commit
            fast_forward: false,
            files_merged: 0,
            conflicts: Vec::new(),
            squashed: true,
        })
    }

    fn three_way_merge(&self, target: &AnnotatedCommit, options: &MergeOperationOptions) -> GitResult<MergeResult> {
        let raw_repo = self.repo.raw();

        // Perform merge
        let mut merge_opts = MergeOptions::new();

        match options.strategy {
            MergeStrategy::Ours => {
                // For "ours" strategy, we don't actually merge, just create commit
            }
            MergeStrategy::Theirs => {
                merge_opts.file_favor(git2::FileFavor::Theirs);
            }
            _ => {}
        }

        raw_repo.merge(&[target], Some(&mut merge_opts), None)?;

        // Check for conflicts
        let index = raw_repo.index()?;
        if index.has_conflicts() {
            let conflicts = self.conflicts()?;
            return Ok(MergeResult {
                success: false,
                commit: None,
                fast_forward: false,
                files_merged: 0,
                conflicts,
                squashed: false,
            });
        }

        // Create merge commit if requested
        if options.commit {
            let sig = raw_repo.signature()?;
            let head_commit = raw_repo.head()?.peel_to_commit()?;
            let merge_commit = raw_repo.find_commit(target.id())?;

            let mut index = raw_repo.index()?;
            let tree_id = index.write_tree()?;
            let tree = raw_repo.find_tree(tree_id)?;

            let message = options.message.clone().unwrap_or_else(|| {
                format!("Merge '{}' into {}",
                    target.refname().unwrap_or("unknown"),
                    self.repo.current_branch().ok().flatten().unwrap_or_default()
                )
            });

            let commit_oid = raw_repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &message,
                &tree,
                &[&head_commit, &merge_commit],
            )?;

            raw_repo.cleanup_state()?;

            Ok(MergeResult {
                success: true,
                commit: Some(GitOid::from(commit_oid)),
                fast_forward: false,
                files_merged: 0,
                conflicts: Vec::new(),
                squashed: false,
            })
        } else {
            Ok(MergeResult {
                success: true,
                commit: None,
                fast_forward: false,
                files_merged: 0,
                conflicts: Vec::new(),
                squashed: false,
            })
        }
    }

    fn resolve_to_annotated(&self, spec: &str) -> GitResult<AnnotatedCommit<'a>> {
        let raw_repo = self.repo.raw();
        let obj = raw_repo.revparse_single(spec)?;
        let commit = obj.peel_to_commit()?;
        let annotated = raw_repo.find_annotated_commit(commit.id())?;

        // Safety: transmute to extend lifetime
        Ok(unsafe { std::mem::transmute(annotated) })
    }

    fn count_commits_between(&self, base: &git2::Oid, head: &git2::Oid) -> GitResult<usize> {
        let raw_repo = self.repo.raw();
        let mut revwalk = raw_repo.revwalk()?;
        revwalk.push(*head)?;
        revwalk.hide(*base)?;
        Ok(revwalk.count())
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy)]
pub enum ConflictResolution {
    Ours,
    Theirs,
    Manual,
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
    fn test_merge_options_builder() {
        let opts = MergeOperationOptions::new()
            .strategy(MergeStrategy::Recursive)
            .message("Custom merge message")
            .no_ff();

        assert!(opts.no_ff);
        assert_eq!(opts.message, Some("Custom merge message".to_string()));
    }

    #[test]
    fn test_squash_option() {
        let opts = MergeOperationOptions::new().squash();

        assert!(opts.squash);
        assert!(!opts.commit); // Squash implies no auto-commit
    }

    #[test]
    fn test_merge_analysis_up_to_date() {
        let (_dir, repo) = setup_test_repo();
        let merger = GitMerger::new(&repo);

        // Analyzing HEAD against itself should be up-to-date
        let analysis = merger.analyze("HEAD").unwrap();

        assert!(analysis.up_to_date);
    }

    #[test]
    fn test_abort_no_merge() {
        let (_dir, repo) = setup_test_repo();
        let merger = GitMerger::new(&repo);

        // Should fail when no merge is in progress
        let result = merger.abort();
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_result_has_conflicts() {
        let result = MergeResult {
            success: false,
            commit: None,
            fast_forward: false,
            files_merged: 0,
            conflicts: vec![
                MergeConflictInfo {
                    path: PathBuf::from("file.txt"),
                    ancestor: None,
                    ours: None,
                    theirs: None,
                    has_markers: true,
                }
            ],
            squashed: false,
        };

        assert!(result.has_conflicts());
    }

    #[test]
    fn test_conflict_resolution_enum() {
        let _ours = ConflictResolution::Ours;
        let _theirs = ConflictResolution::Theirs;
        let _manual = ConflictResolution::Manual;
        // Just ensure they compile
    }

    #[test]
    fn test_merge_strategy_default() {
        assert_eq!(MergeStrategy::default(), MergeStrategy::Recursive);
    }

    #[test]
    fn test_ff_only_option() {
        let opts = MergeOperationOptions::new().ff_only();
        assert!(opts.ff_only);
        assert!(!opts.no_ff);
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 450: Diff Generation
- Spec 455: Pull/Fetch Operations
- Spec 457: Rebase Support
