# 459 - Git Merge

**Phase:** 21 - Git Integration
**Spec ID:** 459
**Status:** Planned
**Dependencies:** 456-git-branch
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement Git merge operations, enabling merging branches and handling merge conflicts.

---

## Acceptance Criteria

- [ ] Merge branches
- [ ] Fast-forward merges
- [ ] Merge with custom message
- [ ] Conflict detection
- [ ] Merge abort

---

## Implementation Details

### 1. Merge Types (src/merge.rs)

```rust
//! Git merge operations.

use crate::{GitCommit, GitOid, GitRepository, GitResult, GitError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Merge result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Result type.
    pub result_type: MergeResultType,
    /// New commit OID (if merge completed).
    pub commit: Option<GitOid>,
    /// Conflicted files.
    pub conflicts: Vec<ConflictFile>,
}

/// Type of merge result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeResultType {
    /// Already up to date.
    UpToDate,
    /// Fast-forward merge.
    FastForward,
    /// Normal merge (created merge commit).
    Merged,
    /// Has conflicts that need resolution.
    Conflict,
}

/// A file with merge conflicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFile {
    /// File path.
    pub path: PathBuf,
    /// Ancestor blob (common base).
    pub ancestor: Option<ConflictBlob>,
    /// Our version.
    pub ours: Option<ConflictBlob>,
    /// Their version.
    pub theirs: Option<ConflictBlob>,
}

/// Blob information for conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictBlob {
    /// Blob OID.
    pub oid: GitOid,
    /// File mode.
    pub mode: u32,
}

/// Merge options.
#[derive(Debug, Clone, Default)]
pub struct MergeOptions {
    /// Custom commit message.
    pub message: Option<String>,
    /// Allow fast-forward.
    pub allow_ff: bool,
    /// Only allow fast-forward.
    pub ff_only: bool,
    /// No commit (leave in merging state).
    pub no_commit: bool,
    /// Squash merge.
    pub squash: bool,
}

impl MergeOptions {
    /// Standard merge options.
    pub fn standard() -> Self {
        Self {
            allow_ff: true,
            ..Default::default()
        }
    }

    /// No fast-forward merge.
    pub fn no_ff() -> Self {
        Self {
            allow_ff: false,
            ..Default::default()
        }
    }

    /// Fast-forward only.
    pub fn ff_only() -> Self {
        Self {
            allow_ff: true,
            ff_only: true,
            ..Default::default()
        }
    }

    /// Set custom message.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

impl GitRepository {
    /// Merge a branch into HEAD.
    pub fn merge_branch(&self, branch: &str, options: Option<MergeOptions>) -> GitResult<MergeResult> {
        let oid = self.with_repo(|repo| {
            let branch = repo.find_branch(branch, git2::BranchType::Local)?;
            let oid = branch.get().target().ok_or_else(|| GitError::BranchNotFound {
                name: branch.to_string(),
            })?;
            Ok(GitOid::from_git2(oid))
        })?;

        self.merge(&oid, options)
    }

    /// Merge a commit into HEAD.
    pub fn merge(&self, commit_oid: &GitOid, options: Option<MergeOptions>) -> GitResult<MergeResult> {
        let options = options.unwrap_or_default();

        self.with_repo_mut(|repo| {
            let head = repo.head()?.peel_to_commit()?;
            let their_commit = repo.find_commit(commit_oid.as_git2())?;

            // Check merge base
            let merge_base = repo.merge_base(head.id(), their_commit.id())?;

            // Already up to date?
            if merge_base == their_commit.id() {
                return Ok(MergeResult {
                    result_type: MergeResultType::UpToDate,
                    commit: None,
                    conflicts: Vec::new(),
                });
            }

            // Can fast-forward?
            if merge_base == head.id() && options.allow_ff {
                // Fast-forward
                repo.checkout_tree(their_commit.tree()?.as_object(), None)?;
                repo.set_head_detached(their_commit.id())?;
                // Re-attach to branch
                if let Ok(head_ref) = repo.head() {
                    if let Some(name) = head_ref.shorthand() {
                        let branch_ref = format!("refs/heads/{}", name);
                        repo.reference(&branch_ref, their_commit.id(), true, "fast-forward")?;
                        repo.set_head(&branch_ref)?;
                    }
                }

                return Ok(MergeResult {
                    result_type: MergeResultType::FastForward,
                    commit: Some(GitOid::from_git2(their_commit.id())),
                    conflicts: Vec::new(),
                });
            }

            // FF-only requested but not possible
            if options.ff_only {
                return Err(GitError::InvalidOperation {
                    message: "Fast-forward not possible".to_string(),
                });
            }

            // Perform merge
            let their_annotated = repo.find_annotated_commit(commit_oid.as_git2())?;
            repo.merge(&[&their_annotated], None, None)?;

            // Check for conflicts
            let index = repo.index()?;
            if index.has_conflicts() {
                let conflicts = collect_conflicts(&index)?;
                return Ok(MergeResult {
                    result_type: MergeResultType::Conflict,
                    commit: None,
                    conflicts,
                });
            }

            // No commit requested
            if options.no_commit {
                return Ok(MergeResult {
                    result_type: MergeResultType::Merged,
                    commit: None,
                    conflicts: Vec::new(),
                });
            }

            // Create merge commit
            let tree_oid = repo.index()?.write_tree()?;
            let tree = repo.find_tree(tree_oid)?;
            let sig = repo.signature()?;

            let message = options.message.unwrap_or_else(|| {
                format!(
                    "Merge commit '{}' into {}",
                    their_commit.id().to_string()[..7].to_string(),
                    head.summary().unwrap_or("HEAD")
                )
            });

            let new_oid = repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &message,
                &tree,
                &[&head, &their_commit],
            )?;

            // Cleanup merge state
            repo.cleanup_state()?;

            Ok(MergeResult {
                result_type: MergeResultType::Merged,
                commit: Some(GitOid::from_git2(new_oid)),
                conflicts: Vec::new(),
            })
        })
    }

    /// Abort an in-progress merge.
    pub fn merge_abort(&self) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            if repo.state() != git2::RepositoryState::Merge {
                return Err(GitError::InvalidOperation {
                    message: "No merge in progress".to_string(),
                });
            }

            // Reset to HEAD
            let head = repo.head()?.peel_to_commit()?;
            repo.reset(head.as_object(), git2::ResetType::Hard, None)?;
            repo.cleanup_state()?;

            Ok(())
        })
    }

    /// Continue a merge after resolving conflicts.
    pub fn merge_continue(&self, message: Option<&str>) -> GitResult<MergeResult> {
        self.with_repo_mut(|repo| {
            if repo.state() != git2::RepositoryState::Merge {
                return Err(GitError::InvalidOperation {
                    message: "No merge in progress".to_string(),
                });
            }

            let index = repo.index()?;
            if index.has_conflicts() {
                let conflicts = collect_conflicts(&index)?;
                return Ok(MergeResult {
                    result_type: MergeResultType::Conflict,
                    commit: None,
                    conflicts,
                });
            }

            // Get merge heads
            let head = repo.head()?.peel_to_commit()?;
            let merge_heads: Vec<git2::Oid> = repo.mergehead_foreach(|oid| {
                true
            }).map(|_| Vec::new()).unwrap_or_default();

            // For simplicity, we'll just get MERGE_HEAD directly
            let merge_head_path = repo.path().join("MERGE_HEAD");
            let merge_head_content = std::fs::read_to_string(&merge_head_path)?;
            let merge_head_oid = git2::Oid::from_str(merge_head_content.trim())?;
            let merge_commit = repo.find_commit(merge_head_oid)?;

            // Create merge commit
            let tree_oid = repo.index()?.write_tree()?;
            let tree = repo.find_tree(tree_oid)?;
            let sig = repo.signature()?;

            let default_message = format!(
                "Merge commit '{}' into {}",
                merge_commit.id().to_string()[..7].to_string(),
                head.summary().unwrap_or("HEAD")
            );

            let new_oid = repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                message.unwrap_or(&default_message),
                &tree,
                &[&head, &merge_commit],
            )?;

            repo.cleanup_state()?;

            Ok(MergeResult {
                result_type: MergeResultType::Merged,
                commit: Some(GitOid::from_git2(new_oid)),
                conflicts: Vec::new(),
            })
        })
    }

    /// Get merge base between two commits.
    pub fn merge_base(&self, oid1: &GitOid, oid2: &GitOid) -> GitResult<GitOid> {
        self.with_repo(|repo| {
            let base = repo.merge_base(oid1.as_git2(), oid2.as_git2())?;
            Ok(GitOid::from_git2(base))
        })
    }

    /// Check if a merge would have conflicts.
    pub fn merge_check(&self, commit_oid: &GitOid) -> GitResult<Vec<ConflictFile>> {
        self.with_repo(|repo| {
            let head = repo.head()?.peel_to_commit()?;
            let their_commit = repo.find_commit(commit_oid.as_git2())?;

            let head_tree = head.tree()?;
            let their_tree = their_commit.tree()?;
            let ancestor_oid = repo.merge_base(head.id(), their_commit.id())?;
            let ancestor = repo.find_commit(ancestor_oid)?;
            let ancestor_tree = ancestor.tree()?;

            let mut index = repo.merge_trees(&ancestor_tree, &head_tree, &their_tree, None)?;

            if index.has_conflicts() {
                collect_conflicts(&index)
            } else {
                Ok(Vec::new())
            }
        })
    }
}

fn collect_conflicts(index: &git2::Index) -> GitResult<Vec<ConflictFile>> {
    let mut conflicts = Vec::new();

    for conflict in index.conflicts()? {
        let conflict = conflict?;

        let path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .and_then(|e| String::from_utf8(e.path.clone()).ok())
            .map(PathBuf::from)
            .unwrap_or_default();

        conflicts.push(ConflictFile {
            path,
            ancestor: conflict.ancestor.as_ref().map(|e| ConflictBlob {
                oid: GitOid::from_git2(e.id),
                mode: e.mode,
            }),
            ours: conflict.our.as_ref().map(|e| ConflictBlob {
                oid: GitOid::from_git2(e.id),
                mode: e.mode,
            }),
            theirs: conflict.their.as_ref().map(|e| ConflictBlob {
                oid: GitOid::from_git2(e.id),
                mode: e.mode,
            }),
        });
    }

    Ok(conflicts)
}
```

---

## Testing Requirements

1. Fast-forward merge works
2. Normal merge creates commit
3. Conflicts are detected
4. Merge abort restores state
5. Merge continue completes merge

---

## Related Specs

- Depends on: [456-git-branch.md](456-git-branch.md)
- Next: [460-git-conflict.md](460-git-conflict.md)
