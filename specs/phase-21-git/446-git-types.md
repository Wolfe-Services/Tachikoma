# Spec 446: Git Core Types

## Phase
21 - Git Integration

## Spec ID
446

## Status
Planned

## Dependencies
- Spec 001: Core Types (foundation types)
- Spec 010: Error Handling (error types)

## Estimated Context
~10%

---

## Objective

Define comprehensive type definitions for Git operations in Tachikoma, providing type-safe abstractions over git2 primitives. These types will serve as the foundation for all Git functionality, ensuring consistent representation of commits, branches, remotes, and other Git objects throughout the codebase.

---

## Acceptance Criteria

- [ ] Define `GitOid` wrapper type for object identifiers
- [ ] Define `GitCommit` struct with full commit metadata
- [ ] Define `GitBranch` struct for branch representation
- [ ] Define `GitRemote` struct for remote configuration
- [ ] Define `GitSignature` struct for author/committer info
- [ ] Define `GitReference` enum for refs (branch, tag, symbolic)
- [ ] Define `GitFileStatus` enum for file states
- [ ] Define `GitDiffDelta` struct for diff entries
- [ ] Define `GitMergeStatus` enum for merge states
- [ ] Implement conversions from git2 types
- [ ] Implement serialization for all types
- [ ] Add comprehensive documentation

---

## Implementation Details

### Core Type Definitions

```rust
// src/git/types.rs

use chrono::{DateTime, TimeZone, Utc};
use git2::{self, Oid, Time};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Wrapper around git2::Oid for type safety and serialization
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GitOid([u8; 20]);

impl GitOid {
    /// Create a new GitOid from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, GitError> {
        if bytes.len() != 20 {
            return Err(GitError::InvalidOid("OID must be 20 bytes".into()));
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }

    /// Create from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, GitError> {
        let oid = Oid::from_str(hex).map_err(|e| GitError::InvalidOid(e.to_string()))?;
        Ok(Self::from(oid))
    }

    /// Get the short form (first 7 characters)
    pub fn short(&self) -> String {
        self.to_string()[..7].to_string()
    }

    /// Check if this is a zero OID
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    /// Convert to git2::Oid
    pub fn to_git2_oid(&self) -> Oid {
        Oid::from_bytes(&self.0).expect("Valid OID bytes")
    }
}

impl From<Oid> for GitOid {
    fn from(oid: Oid) -> Self {
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(oid.as_bytes());
        Self(bytes)
    }
}

impl fmt::Display for GitOid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GitOid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GitOid({})", self.short())
    }
}

impl Serialize for GitOid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GitOid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// Git signature (author or committer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSignature {
    pub name: String,
    pub email: String,
    pub time: DateTime<Utc>,
}

impl GitSignature {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            time: Utc::now(),
        }
    }

    pub fn with_time(mut self, time: DateTime<Utc>) -> Self {
        self.time = time;
        self
    }
}

impl<'a> From<git2::Signature<'a>> for GitSignature {
    fn from(sig: git2::Signature<'a>) -> Self {
        let time = sig.when();
        let datetime = Utc.timestamp_opt(time.seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        Self {
            name: sig.name().unwrap_or("Unknown").to_string(),
            email: sig.email().unwrap_or("unknown@example.com").to_string(),
            time: datetime,
        }
    }
}

/// Represents a Git commit with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub oid: GitOid,
    pub tree_oid: GitOid,
    pub parent_oids: Vec<GitOid>,
    pub author: GitSignature,
    pub committer: GitSignature,
    pub message: String,
    pub summary: String,
}

impl GitCommit {
    /// Check if this is a merge commit
    pub fn is_merge(&self) -> bool {
        self.parent_oids.len() > 1
    }

    /// Check if this is a root commit
    pub fn is_root(&self) -> bool {
        self.parent_oids.is_empty()
    }

    /// Get the first parent OID (if any)
    pub fn first_parent(&self) -> Option<&GitOid> {
        self.parent_oids.first()
    }
}

impl<'a> TryFrom<git2::Commit<'a>> for GitCommit {
    type Error = GitError;

    fn try_from(commit: git2::Commit<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            oid: GitOid::from(commit.id()),
            tree_oid: GitOid::from(commit.tree_id()),
            parent_oids: commit.parent_ids().map(GitOid::from).collect(),
            author: GitSignature::from(commit.author()),
            committer: GitSignature::from(commit.committer()),
            message: commit.message().unwrap_or("").to_string(),
            summary: commit.summary().unwrap_or("").to_string(),
        })
    }
}

/// Git reference types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GitReferenceKind {
    Branch,
    RemoteBranch,
    Tag,
    Note,
    Symbolic,
    Other,
}

/// Represents a Git reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitReference {
    pub name: String,
    pub shorthand: String,
    pub kind: GitReferenceKind,
    pub target: Option<GitOid>,
    pub symbolic_target: Option<String>,
    pub is_head: bool,
}

impl GitReference {
    pub fn is_branch(&self) -> bool {
        matches!(self.kind, GitReferenceKind::Branch)
    }

    pub fn is_remote(&self) -> bool {
        matches!(self.kind, GitReferenceKind::RemoteBranch)
    }

    pub fn is_tag(&self) -> bool {
        matches!(self.kind, GitReferenceKind::Tag)
    }
}

/// Represents a Git branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub full_name: String,
    pub oid: GitOid,
    pub is_head: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
}

impl GitBranch {
    /// Check if branch is up to date with upstream
    pub fn is_up_to_date(&self) -> bool {
        self.ahead == Some(0) && self.behind == Some(0)
    }

    /// Check if branch has diverged from upstream
    pub fn has_diverged(&self) -> bool {
        matches!((self.ahead, self.behind), (Some(a), Some(b)) if a > 0 && b > 0)
    }
}

/// Represents a Git remote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRemote {
    pub name: String,
    pub url: Option<String>,
    pub push_url: Option<String>,
    pub fetch_refspecs: Vec<String>,
    pub push_refspecs: Vec<String>,
}

impl GitRemote {
    pub fn is_github(&self) -> bool {
        self.url.as_ref().map_or(false, |u| u.contains("github.com"))
    }

    pub fn is_gitlab(&self) -> bool {
        self.url.as_ref().map_or(false, |u| u.contains("gitlab.com"))
    }
}

/// File status in the working directory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitFileStatus {
    Current,
    IndexNew,
    IndexModified,
    IndexDeleted,
    IndexRenamed,
    IndexTypechange,
    WorktreeNew,
    WorktreeModified,
    WorktreeDeleted,
    WorktreeTypechange,
    WorktreeRenamed,
    Ignored,
    Conflicted,
}

impl GitFileStatus {
    pub fn is_staged(&self) -> bool {
        matches!(
            self,
            Self::IndexNew
                | Self::IndexModified
                | Self::IndexDeleted
                | Self::IndexRenamed
                | Self::IndexTypechange
        )
    }

    pub fn is_unstaged(&self) -> bool {
        matches!(
            self,
            Self::WorktreeNew
                | Self::WorktreeModified
                | Self::WorktreeDeleted
                | Self::WorktreeTypechange
                | Self::WorktreeRenamed
        )
    }

    pub fn is_conflicted(&self) -> bool {
        matches!(self, Self::Conflicted)
    }
}

impl From<git2::Status> for GitFileStatus {
    fn from(status: git2::Status) -> Self {
        if status.is_conflicted() {
            return Self::Conflicted;
        }
        if status.is_ignored() {
            return Self::Ignored;
        }
        if status.is_index_new() {
            return Self::IndexNew;
        }
        if status.is_index_modified() {
            return Self::IndexModified;
        }
        if status.is_index_deleted() {
            return Self::IndexDeleted;
        }
        if status.is_index_renamed() {
            return Self::IndexRenamed;
        }
        if status.is_index_typechange() {
            return Self::IndexTypechange;
        }
        if status.is_wt_new() {
            return Self::WorktreeNew;
        }
        if status.is_wt_modified() {
            return Self::WorktreeModified;
        }
        if status.is_wt_deleted() {
            return Self::WorktreeDeleted;
        }
        if status.is_wt_renamed() {
            return Self::WorktreeRenamed;
        }
        if status.is_wt_typechange() {
            return Self::WorktreeTypechange;
        }
        Self::Current
    }
}

/// Entry in a status listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusEntry {
    pub path: PathBuf,
    pub status: GitFileStatus,
    pub head_to_index: Option<GitDiffDelta>,
    pub index_to_workdir: Option<GitDiffDelta>,
}

/// Diff delta type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitDeltaKind {
    Unmodified,
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Ignored,
    Untracked,
    Typechange,
    Unreadable,
    Conflicted,
}

impl From<git2::Delta> for GitDeltaKind {
    fn from(delta: git2::Delta) -> Self {
        match delta {
            git2::Delta::Unmodified => Self::Unmodified,
            git2::Delta::Added => Self::Added,
            git2::Delta::Deleted => Self::Deleted,
            git2::Delta::Modified => Self::Modified,
            git2::Delta::Renamed => Self::Renamed,
            git2::Delta::Copied => Self::Copied,
            git2::Delta::Ignored => Self::Ignored,
            git2::Delta::Untracked => Self::Untracked,
            git2::Delta::Typechange => Self::Typechange,
            git2::Delta::Unreadable => Self::Unreadable,
            git2::Delta::Conflicted => Self::Conflicted,
        }
    }
}

/// Diff delta entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffDelta {
    pub kind: GitDeltaKind,
    pub old_file: Option<PathBuf>,
    pub new_file: Option<PathBuf>,
    pub old_oid: Option<GitOid>,
    pub new_oid: Option<GitOid>,
    pub similarity: Option<u32>,
}

/// Merge analysis result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitMergeStatus {
    /// Already up-to-date
    UpToDate,
    /// Can fast-forward
    FastForward,
    /// Normal merge required
    Normal,
    /// Unborn branch (no commits)
    Unborn,
}

impl From<git2::MergeAnalysis> for GitMergeStatus {
    fn from(analysis: git2::MergeAnalysis) -> Self {
        if analysis.is_up_to_date() {
            Self::UpToDate
        } else if analysis.is_fast_forward() {
            Self::FastForward
        } else if analysis.is_unborn() {
            Self::Unborn
        } else {
            Self::Normal
        }
    }
}

/// Conflict entry during merge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConflict {
    pub ancestor: Option<PathBuf>,
    pub ours: Option<PathBuf>,
    pub theirs: Option<PathBuf>,
}

/// Stash entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStashEntry {
    pub index: usize,
    pub oid: GitOid,
    pub message: String,
    pub committer: GitSignature,
}

/// Git error types
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git2 error: {0}")]
    Git2(#[from] git2::Error),

    #[error("Invalid OID: {0}")]
    InvalidOid(String),

    #[error("Repository not found: {0}")]
    RepoNotFound(PathBuf),

    #[error("Reference not found: {0}")]
    RefNotFound(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Remote not found: {0}")]
    RemoteNotFound(String),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("Merge conflict in {0} files")]
    MergeConflict(usize),

    #[error("Working directory not clean")]
    DirtyWorkingDirectory,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Hook failed: {0}")]
    HookFailed(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}

/// Result type for Git operations
pub type GitResult<T> = Result<T, GitError>;
```

### Type Builder Pattern

```rust
// src/git/types/builders.rs

use super::*;

/// Builder for creating GitCommit instances (for testing)
#[derive(Default)]
pub struct GitCommitBuilder {
    oid: Option<GitOid>,
    tree_oid: Option<GitOid>,
    parent_oids: Vec<GitOid>,
    author: Option<GitSignature>,
    committer: Option<GitSignature>,
    message: String,
}

impl GitCommitBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn oid(mut self, oid: GitOid) -> Self {
        self.oid = Some(oid);
        self
    }

    pub fn tree_oid(mut self, oid: GitOid) -> Self {
        self.tree_oid = Some(oid);
        self
    }

    pub fn parent(mut self, oid: GitOid) -> Self {
        self.parent_oids.push(oid);
        self
    }

    pub fn author(mut self, sig: GitSignature) -> Self {
        self.author = Some(sig);
        self
    }

    pub fn committer(mut self, sig: GitSignature) -> Self {
        self.committer = Some(sig);
        self
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    pub fn build(self) -> GitCommit {
        let summary = self.message.lines().next().unwrap_or("").to_string();
        GitCommit {
            oid: self.oid.unwrap_or_else(|| GitOid([0; 20])),
            tree_oid: self.tree_oid.unwrap_or_else(|| GitOid([0; 20])),
            parent_oids: self.parent_oids,
            author: self.author.unwrap_or_else(|| GitSignature::new("Test", "test@example.com")),
            committer: self.committer.unwrap_or_else(|| GitSignature::new("Test", "test@example.com")),
            summary,
            message: self.message,
        }
    }
}

/// Builder for GitBranch
#[derive(Default)]
pub struct GitBranchBuilder {
    name: String,
    oid: Option<GitOid>,
    is_head: bool,
    is_remote: bool,
    upstream: Option<String>,
}

impl GitBranchBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn oid(mut self, oid: GitOid) -> Self {
        self.oid = Some(oid);
        self
    }

    pub fn is_head(mut self, is_head: bool) -> Self {
        self.is_head = is_head;
        self
    }

    pub fn is_remote(mut self, is_remote: bool) -> Self {
        self.is_remote = is_remote;
        self
    }

    pub fn upstream(mut self, upstream: impl Into<String>) -> Self {
        self.upstream = Some(upstream.into());
        self
    }

    pub fn build(self) -> GitBranch {
        let full_name = if self.is_remote {
            format!("refs/remotes/{}", self.name)
        } else {
            format!("refs/heads/{}", self.name)
        };

        GitBranch {
            name: self.name,
            full_name,
            oid: self.oid.unwrap_or_else(|| GitOid([0; 20])),
            is_head: self.is_head,
            is_remote: self.is_remote,
            upstream: self.upstream,
            ahead: None,
            behind: None,
        }
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

    #[test]
    fn test_git_oid_from_hex() {
        let hex = "abcdef1234567890abcdef1234567890abcdef12";
        let oid = GitOid::from_hex(hex).unwrap();
        assert_eq!(oid.to_string(), hex);
    }

    #[test]
    fn test_git_oid_short() {
        let hex = "abcdef1234567890abcdef1234567890abcdef12";
        let oid = GitOid::from_hex(hex).unwrap();
        assert_eq!(oid.short(), "abcdef1");
    }

    #[test]
    fn test_git_oid_invalid() {
        let result = GitOid::from_hex("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_signature_new() {
        let sig = GitSignature::new("Alice", "alice@example.com");
        assert_eq!(sig.name, "Alice");
        assert_eq!(sig.email, "alice@example.com");
    }

    #[test]
    fn test_git_commit_is_merge() {
        let commit = GitCommitBuilder::new()
            .parent(GitOid([1; 20]))
            .parent(GitOid([2; 20]))
            .build();
        assert!(commit.is_merge());
    }

    #[test]
    fn test_git_commit_is_root() {
        let commit = GitCommitBuilder::new().build();
        assert!(commit.is_root());
    }

    #[test]
    fn test_git_file_status_staged() {
        assert!(GitFileStatus::IndexNew.is_staged());
        assert!(GitFileStatus::IndexModified.is_staged());
        assert!(!GitFileStatus::WorktreeModified.is_staged());
    }

    #[test]
    fn test_git_branch_diverged() {
        let mut branch = GitBranchBuilder::new("main").build();
        branch.ahead = Some(2);
        branch.behind = Some(1);
        assert!(branch.has_diverged());
    }

    #[test]
    fn test_git_oid_serialization() {
        let hex = "abcdef1234567890abcdef1234567890abcdef12";
        let oid = GitOid::from_hex(hex).unwrap();
        let json = serde_json::to_string(&oid).unwrap();
        assert_eq!(json, format!("\"{}\"", hex));

        let deserialized: GitOid = serde_json::from_str(&json).unwrap();
        assert_eq!(oid, deserialized);
    }

    #[test]
    fn test_git_remote_detection() {
        let remote = GitRemote {
            name: "origin".to_string(),
            url: Some("https://github.com/user/repo.git".to_string()),
            push_url: None,
            fetch_refspecs: vec![],
            push_refspecs: vec![],
        };
        assert!(remote.is_github());
        assert!(!remote.is_gitlab());
    }
}
```

---

## Related Specs

- Spec 447: Git Configuration
- Spec 448: Repository Operations
- Spec 449: Status Checking
- Spec 450: Diff Generation
