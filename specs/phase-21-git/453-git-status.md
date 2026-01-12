# 453 - Git Status

**Phase:** 21 - Git Integration
**Spec ID:** 453
**Status:** Planned
**Dependencies:** 452-git-detect
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement Git status functionality, providing detailed information about the working directory and staging area state.

---

## Acceptance Criteria

- [ ] Working directory status
- [ ] Staging area status
- [ ] File status enumeration
- [ ] Submodule status
- [ ] Status filtering options

---

## Implementation Details

### 1. Status Types (src/status.rs)

```rust
//! Git status types and implementation.

use crate::{GitOid, GitRepository, GitResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File status in the working directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    /// File is untracked.
    Untracked,
    /// File is ignored.
    Ignored,
    /// File is new in index.
    New,
    /// File is modified.
    Modified,
    /// File is deleted.
    Deleted,
    /// File is renamed.
    Renamed,
    /// File is copied.
    Copied,
    /// File has type change.
    TypeChange,
    /// File is conflicted.
    Conflicted,
}

/// Detailed file status entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEntry {
    /// File path (relative to repo root).
    pub path: PathBuf,
    /// Original path (for renames).
    pub orig_path: Option<PathBuf>,
    /// Status in the index (staging area).
    pub index_status: Option<FileStatus>,
    /// Status in the working directory.
    pub worktree_status: Option<FileStatus>,
    /// Is binary file.
    pub is_binary: bool,
}

impl StatusEntry {
    /// Check if file has staged changes.
    pub fn is_staged(&self) -> bool {
        self.index_status.is_some()
            && !matches!(self.index_status, Some(FileStatus::Untracked | FileStatus::Ignored))
    }

    /// Check if file has unstaged changes.
    pub fn is_unstaged(&self) -> bool {
        matches!(
            self.worktree_status,
            Some(FileStatus::Modified | FileStatus::Deleted | FileStatus::TypeChange)
        )
    }

    /// Check if file is conflicted.
    pub fn is_conflicted(&self) -> bool {
        matches!(
            self.index_status,
            Some(FileStatus::Conflicted)
        ) || matches!(
            self.worktree_status,
            Some(FileStatus::Conflicted)
        )
    }
}

/// Repository status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    /// Current branch.
    pub branch: Option<String>,
    /// Head commit OID.
    pub head: Option<GitOid>,
    /// Upstream branch.
    pub upstream: Option<String>,
    /// Commits ahead of upstream.
    pub ahead: u32,
    /// Commits behind upstream.
    pub behind: u32,
    /// Status entries.
    pub entries: Vec<StatusEntry>,
    /// Is in the middle of a merge.
    pub is_merging: bool,
    /// Is in the middle of a rebase.
    pub is_rebasing: bool,
    /// Is in the middle of a cherry-pick.
    pub is_cherry_picking: bool,
    /// Is in the middle of a revert.
    pub is_reverting: bool,
    /// Is in the middle of a bisect.
    pub is_bisecting: bool,
}

impl RepoStatus {
    /// Check if working directory is clean.
    pub fn is_clean(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get only staged entries.
    pub fn staged(&self) -> Vec<&StatusEntry> {
        self.entries.iter().filter(|e| e.is_staged()).collect()
    }

    /// Get only unstaged entries.
    pub fn unstaged(&self) -> Vec<&StatusEntry> {
        self.entries.iter().filter(|e| e.is_unstaged()).collect()
    }

    /// Get only untracked entries.
    pub fn untracked(&self) -> Vec<&StatusEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.worktree_status, Some(FileStatus::Untracked)))
            .collect()
    }

    /// Get only conflicted entries.
    pub fn conflicted(&self) -> Vec<&StatusEntry> {
        self.entries.iter().filter(|e| e.is_conflicted()).collect()
    }

    /// Check if there are merge conflicts.
    pub fn has_conflicts(&self) -> bool {
        self.entries.iter().any(|e| e.is_conflicted())
    }

    /// Get summary counts.
    pub fn summary(&self) -> StatusSummary {
        StatusSummary {
            staged: self.staged().len(),
            unstaged: self.unstaged().len(),
            untracked: self.untracked().len(),
            conflicted: self.conflicted().len(),
        }
    }
}

/// Summary counts for status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatusSummary {
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicted: usize,
}

/// Status query options.
#[derive(Debug, Clone, Default)]
pub struct StatusOptions {
    /// Include untracked files.
    pub include_untracked: bool,
    /// Include ignored files.
    pub include_ignored: bool,
    /// Include submodules.
    pub include_submodules: bool,
    /// Detect renames.
    pub detect_renames: bool,
    /// Path patterns to filter.
    pub pathspecs: Vec<String>,
}

impl StatusOptions {
    /// Include all file types.
    pub fn all() -> Self {
        Self {
            include_untracked: true,
            include_ignored: true,
            include_submodules: true,
            detect_renames: true,
            pathspecs: Vec::new(),
        }
    }

    /// Standard status (untracked but not ignored).
    pub fn standard() -> Self {
        Self {
            include_untracked: true,
            include_ignored: false,
            include_submodules: true,
            detect_renames: true,
            pathspecs: Vec::new(),
        }
    }
}
```

### 2. Status Implementation (src/status_impl.rs)

```rust
//! Git status implementation.

use crate::{
    status::{FileStatus, RepoStatus, StatusEntry, StatusOptions},
    GitOid, GitRepository, GitResult,
};
use git2::{Status, StatusOptions as Git2StatusOpts};
use std::path::PathBuf;

impl GitRepository {
    /// Get repository status.
    pub fn status(&self, options: StatusOptions) -> GitResult<RepoStatus> {
        self.with_repo(|repo| {
            // Get branch info
            let (branch, head, upstream, ahead, behind) = get_branch_info(repo)?;

            // Get status entries
            let entries = get_status_entries(repo, &options)?;

            // Check in-progress operations
            let state = repo.state();

            Ok(RepoStatus {
                branch,
                head,
                upstream,
                ahead,
                behind,
                entries,
                is_merging: state == git2::RepositoryState::Merge,
                is_rebasing: matches!(
                    state,
                    git2::RepositoryState::Rebase
                        | git2::RepositoryState::RebaseInteractive
                        | git2::RepositoryState::RebaseMerge
                ),
                is_cherry_picking: state == git2::RepositoryState::CherryPick
                    || state == git2::RepositoryState::CherryPickSequence,
                is_reverting: state == git2::RepositoryState::Revert
                    || state == git2::RepositoryState::RevertSequence,
                is_bisecting: state == git2::RepositoryState::Bisect,
            })
        })
    }

    /// Get a quick status summary (faster than full status).
    pub fn status_quick(&self) -> GitResult<StatusSummary> {
        let status = self.status(StatusOptions::standard())?;
        Ok(status.summary())
    }

    /// Check if working directory is clean.
    pub fn is_clean(&self) -> GitResult<bool> {
        let status = self.status(StatusOptions {
            include_untracked: false,
            include_ignored: false,
            include_submodules: false,
            detect_renames: false,
            pathspecs: Vec::new(),
        })?;
        Ok(status.is_clean())
    }
}

fn get_branch_info(
    repo: &git2::Repository,
) -> GitResult<(Option<String>, Option<GitOid>, Option<String>, u32, u32)> {
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => {
            return Ok((None, None, None, 0, 0));
        }
    };

    let branch = head.shorthand().map(String::from);
    let head_oid = head.target().map(GitOid::from_git2);

    // Get upstream info
    let (upstream, ahead, behind) = if let Some(branch_name) = head.shorthand() {
        if let Ok(local_branch) = repo.find_branch(branch_name, git2::BranchType::Local) {
            if let Ok(upstream_branch) = local_branch.upstream() {
                let upstream_name = upstream_branch.name()?.map(String::from);

                let (ahead, behind) = if let (Some(local_oid), Ok(upstream_ref)) =
                    (head.target(), upstream_branch.into_reference().target())
                {
                    repo.graph_ahead_behind(local_oid, upstream_ref)
                        .unwrap_or((0, 0))
                } else {
                    (0, 0)
                };

                (upstream_name, ahead as u32, behind as u32)
            } else {
                (None, 0, 0)
            }
        } else {
            (None, 0, 0)
        }
    } else {
        (None, 0, 0)
    };

    Ok((branch, head_oid, upstream, ahead, behind))
}

fn get_status_entries(
    repo: &git2::Repository,
    options: &StatusOptions,
) -> GitResult<Vec<StatusEntry>> {
    let mut git_opts = Git2StatusOpts::new();

    if options.include_untracked {
        git_opts.include_untracked(true);
    }
    if options.include_ignored {
        git_opts.include_ignored(true);
    }
    if options.detect_renames {
        git_opts.renames_head_to_index(true);
        git_opts.renames_index_to_workdir(true);
    }

    for pathspec in &options.pathspecs {
        git_opts.pathspec(pathspec);
    }

    let statuses = repo.statuses(Some(&mut git_opts))?;
    let mut entries = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().map(PathBuf::from);
        if path.is_none() {
            continue;
        }
        let path = path.unwrap();

        let status = entry.status();
        let (index_status, worktree_status) = parse_status_flags(status);

        let orig_path = entry
            .head_to_index()
            .and_then(|d| d.old_file().path())
            .or_else(|| entry.index_to_workdir().and_then(|d| d.old_file().path()))
            .map(PathBuf::from);

        entries.push(StatusEntry {
            path,
            orig_path,
            index_status,
            worktree_status,
            is_binary: false, // Would need to check file content
        });
    }

    Ok(entries)
}

fn parse_status_flags(status: Status) -> (Option<FileStatus>, Option<FileStatus>) {
    let index_status = if status.is_index_new() {
        Some(FileStatus::New)
    } else if status.is_index_modified() {
        Some(FileStatus::Modified)
    } else if status.is_index_deleted() {
        Some(FileStatus::Deleted)
    } else if status.is_index_renamed() {
        Some(FileStatus::Renamed)
    } else if status.is_index_typechange() {
        Some(FileStatus::TypeChange)
    } else {
        None
    };

    let worktree_status = if status.is_wt_new() {
        Some(FileStatus::Untracked)
    } else if status.is_wt_modified() {
        Some(FileStatus::Modified)
    } else if status.is_wt_deleted() {
        Some(FileStatus::Deleted)
    } else if status.is_wt_renamed() {
        Some(FileStatus::Renamed)
    } else if status.is_wt_typechange() {
        Some(FileStatus::TypeChange)
    } else if status.is_ignored() {
        Some(FileStatus::Ignored)
    } else if status.is_conflicted() {
        Some(FileStatus::Conflicted)
    } else {
        None
    };

    (index_status, worktree_status)
}

use crate::status::StatusSummary;
```

---

## Testing Requirements

1. Status correctly identifies file states
2. Staging area vs working directory distinction
3. Rename detection works
4. Conflict detection works
5. Branch tracking info is accurate

---

## Related Specs

- Depends on: [452-git-detect.md](452-git-detect.md)
- Next: [454-git-diff.md](454-git-diff.md)
