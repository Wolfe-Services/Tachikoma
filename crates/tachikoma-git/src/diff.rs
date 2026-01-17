//! Git diff types.

use crate::GitOid;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Diff between two trees/commits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    /// Files changed.
    pub files: Vec<DiffFile>,
    /// Total statistics.
    pub stats: DiffStats,
}

impl GitDiff {
    /// Check if diff is empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get only added files.
    pub fn added(&self) -> Vec<&DiffFile> {
        self.files.iter().filter(|f| f.status == DiffStatus::Added).collect()
    }

    /// Get only deleted files.
    pub fn deleted(&self) -> Vec<&DiffFile> {
        self.files.iter().filter(|f| f.status == DiffStatus::Deleted).collect()
    }

    /// Get only modified files.
    pub fn modified(&self) -> Vec<&DiffFile> {
        self.files.iter().filter(|f| f.status == DiffStatus::Modified).collect()
    }
}

/// Diff statistics.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DiffStats {
    /// Files changed.
    pub files_changed: u32,
    /// Insertions.
    pub insertions: u32,
    /// Deletions.
    pub deletions: u32,
}

/// Status of a file in the diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    TypeChange,
    Untracked,
    Ignored,
    Conflicted,
}

/// A file in the diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    /// Old path (for renames).
    pub old_path: Option<PathBuf>,
    /// New path.
    pub new_path: PathBuf,
    /// File status.
    pub status: DiffStatus,
    /// Old OID.
    pub old_oid: Option<GitOid>,
    /// New OID.
    pub new_oid: Option<GitOid>,
    /// Is binary file.
    pub is_binary: bool,
    /// File mode changed.
    pub mode_changed: bool,
    /// Old file mode.
    pub old_mode: Option<u32>,
    /// New file mode.
    pub new_mode: Option<u32>,
    /// Hunks in this file.
    pub hunks: Vec<DiffHunk>,
    /// File-level statistics.
    pub stats: DiffStats,
}

impl DiffFile {
    /// Get the primary path for this file.
    pub fn path(&self) -> &PathBuf {
        &self.new_path
    }

    /// Check if this is a rename.
    pub fn is_rename(&self) -> bool {
        self.status == DiffStatus::Renamed
    }
}

/// A hunk in the diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Old file start line.
    pub old_start: u32,
    /// Old file line count.
    pub old_lines: u32,
    /// New file start line.
    pub new_start: u32,
    /// New file line count.
    pub new_lines: u32,
    /// Hunk header.
    pub header: String,
    /// Lines in this hunk.
    pub lines: Vec<DiffLine>,
}

/// A line in the diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line origin.
    pub origin: LineOrigin,
    /// Line content.
    pub content: String,
    /// Old line number (if applicable).
    pub old_lineno: Option<u32>,
    /// New line number (if applicable).
    pub new_lineno: Option<u32>,
}

/// Line origin in diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineOrigin {
    Context,
    Addition,
    Deletion,
    ContextEofnl,
    AddEofnl,
    DelEofnl,
    FileHeader,
    HunkHeader,
    Binary,
}

/// Diff options.
#[derive(Debug, Clone, Default)]
pub struct DiffOptions {
    /// Include context lines.
    pub context_lines: u32,
    /// Ignore whitespace changes.
    pub ignore_whitespace: bool,
    /// Ignore whitespace at end of line.
    pub ignore_whitespace_eol: bool,
    /// Detect renames.
    pub detect_renames: bool,
    /// Detect copies.
    pub detect_copies: bool,
    /// Path patterns to include.
    pub pathspecs: Vec<String>,
}

impl DiffOptions {
    /// Standard diff options.
    pub fn standard() -> Self {
        Self {
            context_lines: 3,
            ignore_whitespace: false,
            ignore_whitespace_eol: false,
            detect_renames: true,
            detect_copies: false,
            pathspecs: Vec::new(),
        }
    }

    /// Diff with no context.
    pub fn no_context() -> Self {
        Self {
            context_lines: 0,
            ..Self::standard()
        }
    }
}