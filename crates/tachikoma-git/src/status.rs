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