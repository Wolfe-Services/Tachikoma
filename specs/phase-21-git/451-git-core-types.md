# 451 - Git Core Types

**Phase:** 21 - Git Integration
**Spec ID:** 451
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Define the core types for Git integration using the git2 crate, providing safe Rust wrappers for Git objects and operations.

---

## Acceptance Criteria

- [x] `tachikoma-git` crate created
- [x] Git object ID wrapper type
- [x] Repository wrapper type
- [x] Reference types
- [x] Error types for Git operations

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-git/Cargo.toml)

```toml
[package]
name = "tachikoma-git"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Git integration for Tachikoma"

[dependencies]
tachikoma-common-core.workspace = true
git2 = { version = "0.18", features = ["vendored-libgit2"] }
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
chrono = { version = "0.4", features = ["serde"] }
tokio = { workspace = true, features = ["fs", "process"] }
tracing.workspace = true
parking_lot.workspace = true

[dev-dependencies]
tempfile = "3.10"
proptest.workspace = true
```

### 2. Object ID Type (src/oid.rs)

```rust
//! Git object ID wrapper.

use git2::Oid as Git2Oid;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::fmt;
use std::str::FromStr;

/// Git object identifier (SHA-1 hash).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GitOid(Git2Oid);

impl GitOid {
    /// Create from git2 Oid.
    pub fn from_git2(oid: Git2Oid) -> Self {
        Self(oid)
    }

    /// Get the underlying git2 Oid.
    pub fn as_git2(&self) -> Git2Oid {
        self.0
    }

    /// Parse from hex string.
    pub fn from_hex(hex: &str) -> Result<Self, GitOidError> {
        Git2Oid::from_str(hex)
            .map(Self)
            .map_err(|_| GitOidError::InvalidHex(hex.to_string()))
    }

    /// Get as hex string.
    pub fn to_hex(&self) -> String {
        self.0.to_string()
    }

    /// Get short form (first 7 characters).
    pub fn short(&self) -> String {
        self.to_hex()[..7].to_string()
    }

    /// Check if this is a zero OID.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Create a zero OID.
    pub fn zero() -> Self {
        Self(Git2Oid::zero())
    }
}

impl fmt::Display for GitOid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for GitOid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GitOid({})", self.short())
    }
}

impl FromStr for GitOid {
    type Err = GitOidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Serialize for GitOid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for GitOid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// Git OID error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GitOidError {
    #[error("invalid hex string: {0}")]
    InvalidHex(String),
}
```

### 3. Reference Types (src/reference.rs)

```rust
//! Git reference types.

use crate::GitOid;
use serde::{Deserialize, Serialize};

/// Type of Git reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefType {
    /// Local branch.
    Branch,
    /// Remote tracking branch.
    RemoteBranch,
    /// Tag.
    Tag,
    /// Other reference.
    Other,
}

/// A Git reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRef {
    /// Full reference name (e.g., "refs/heads/main").
    pub name: String,
    /// Short name (e.g., "main").
    pub shorthand: String,
    /// Reference type.
    pub ref_type: RefType,
    /// Target OID (if direct reference).
    pub target: Option<GitOid>,
    /// Symbolic target (if symbolic reference).
    pub symbolic_target: Option<String>,
    /// Is this the HEAD reference.
    pub is_head: bool,
}

impl GitRef {
    /// Create from git2 reference.
    pub fn from_git2(reference: &git2::Reference, is_head: bool) -> Option<Self> {
        let name = reference.name()?.to_string();
        let shorthand = reference.shorthand()?.to_string();

        let ref_type = if reference.is_branch() {
            RefType::Branch
        } else if reference.is_remote() {
            RefType::RemoteBranch
        } else if reference.is_tag() {
            RefType::Tag
        } else {
            RefType::Other
        };

        let target = reference.target().map(GitOid::from_git2);
        let symbolic_target = reference.symbolic_target().map(String::from);

        Some(Self {
            name,
            shorthand,
            ref_type,
            target,
            symbolic_target,
            is_head,
        })
    }

    /// Check if this is a local branch.
    pub fn is_branch(&self) -> bool {
        matches!(self.ref_type, RefType::Branch)
    }

    /// Check if this is a remote branch.
    pub fn is_remote(&self) -> bool {
        matches!(self.ref_type, RefType::RemoteBranch)
    }
}

/// Branch information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    /// Branch name.
    pub name: String,
    /// Is this the current branch.
    pub is_current: bool,
    /// Upstream branch (if tracking).
    pub upstream: Option<String>,
    /// Latest commit OID.
    pub commit: GitOid,
    /// Commits ahead of upstream.
    pub ahead: Option<u32>,
    /// Commits behind upstream.
    pub behind: Option<u32>,
}

/// Tag information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitTag {
    /// Tag name.
    pub name: String,
    /// Tag OID.
    pub oid: GitOid,
    /// Target commit OID (for annotated tags).
    pub target: GitOid,
    /// Tag message (for annotated tags).
    pub message: Option<String>,
    /// Tagger information (for annotated tags).
    pub tagger: Option<GitSignature>,
}

/// Git signature (author/committer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSignature {
    /// Name.
    pub name: String,
    /// Email.
    pub email: String,
    /// Timestamp.
    pub when: chrono::DateTime<chrono::Utc>,
}

impl GitSignature {
    /// Create from git2 signature.
    pub fn from_git2(sig: &git2::Signature) -> Self {
        let when = chrono::DateTime::from_timestamp(
            sig.when().seconds(),
            0,
        ).unwrap_or_else(chrono::Utc::now);

        Self {
            name: sig.name().unwrap_or("Unknown").to_string(),
            email: sig.email().unwrap_or("unknown@example.com").to_string(),
            when,
        }
    }
}
```

### 4. Error Types (src/error.rs)

```rust
//! Git error types.

use thiserror::Error;

/// Git operation error.
#[derive(Debug, Error)]
pub enum GitError {
    /// Repository not found.
    #[error("repository not found at {path}")]
    RepoNotFound { path: String },

    /// Not a git repository.
    #[error("not a git repository: {path}")]
    NotARepo { path: String },

    /// Reference not found.
    #[error("reference not found: {name}")]
    RefNotFound { name: String },

    /// Branch not found.
    #[error("branch not found: {name}")]
    BranchNotFound { name: String },

    /// Remote not found.
    #[error("remote not found: {name}")]
    RemoteNotFound { name: String },

    /// Commit not found.
    #[error("commit not found: {oid}")]
    CommitNotFound { oid: String },

    /// Merge conflict.
    #[error("merge conflict in {files:?}")]
    MergeConflict { files: Vec<String> },

    /// Dirty working directory.
    #[error("working directory has uncommitted changes")]
    DirtyWorkDir,

    /// Authentication failed.
    #[error("authentication failed: {reason}")]
    AuthFailed { reason: String },

    /// Network error.
    #[error("network error: {message}")]
    Network { message: String },

    /// Invalid operation.
    #[error("invalid operation: {message}")]
    InvalidOperation { message: String },

    /// Git2 library error.
    #[error("git error: {0}")]
    Git2(#[from] git2::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for Git operations.
pub type GitResult<T> = Result<T, GitError>;

impl GitError {
    /// Check if this is a network-related error.
    pub fn is_network_error(&self) -> bool {
        match self {
            Self::Network { .. } | Self::AuthFailed { .. } => true,
            Self::Git2(e) => {
                matches!(e.class(), git2::ErrorClass::Net | git2::ErrorClass::Http)
            }
            _ => false,
        }
    }

    /// Check if this is a conflict error.
    pub fn is_conflict(&self) -> bool {
        matches!(self, Self::MergeConflict { .. })
    }
}
```

### 5. Commit Type (src/commit.rs)

```rust
//! Git commit types.

use crate::{GitOid, GitSignature};
use serde::{Deserialize, Serialize};

/// Git commit information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    /// Commit OID.
    pub oid: GitOid,
    /// Commit message (first line).
    pub summary: String,
    /// Full commit message.
    pub message: String,
    /// Author signature.
    pub author: GitSignature,
    /// Committer signature.
    pub committer: GitSignature,
    /// Parent commit OIDs.
    pub parents: Vec<GitOid>,
    /// Tree OID.
    pub tree: GitOid,
}

impl GitCommit {
    /// Create from git2 commit.
    pub fn from_git2(commit: &git2::Commit) -> Self {
        Self {
            oid: GitOid::from_git2(commit.id()),
            summary: commit.summary().unwrap_or("").to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: GitSignature::from_git2(&commit.author()),
            committer: GitSignature::from_git2(&commit.committer()),
            parents: commit.parent_ids().map(GitOid::from_git2).collect(),
            tree: GitOid::from_git2(commit.tree_id()),
        }
    }

    /// Check if this is a merge commit.
    pub fn is_merge(&self) -> bool {
        self.parents.len() > 1
    }

    /// Get the first parent OID.
    pub fn first_parent(&self) -> Option<&GitOid> {
        self.parents.first()
    }
}

/// Commit creation options.
#[derive(Debug, Clone, Default)]
pub struct CommitOptions {
    /// Commit message.
    pub message: String,
    /// Author (defaults to config).
    pub author: Option<(String, String)>,
    /// Committer (defaults to author).
    pub committer: Option<(String, String)>,
    /// Allow empty commits.
    pub allow_empty: bool,
    /// Amend the last commit.
    pub amend: bool,
    /// Sign the commit with GPG.
    pub sign: bool,
}

impl CommitOptions {
    /// Create with just a message.
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Default::default()
        }
    }

    /// Set author.
    pub fn author(mut self, name: impl Into<String>, email: impl Into<String>) -> Self {
        self.author = Some((name.into(), email.into()));
        self
    }

    /// Enable amend mode.
    pub fn amend(mut self) -> Self {
        self.amend = true;
        self
    }
}
```

### 6. Library Root (src/lib.rs)

```rust
//! Git integration for Tachikoma.
//!
//! This crate provides safe Rust wrappers around the git2 library.

#![warn(missing_docs)]

pub mod commit;
pub mod error;
pub mod oid;
pub mod reference;

pub use commit::{CommitOptions, GitCommit};
pub use error::{GitError, GitResult};
pub use oid::{GitOid, GitOidError};
pub use reference::{GitBranch, GitRef, GitSignature, GitTag, RefType};

// Re-export git2 for advanced usage
pub use git2;
```

---

## Testing Requirements

1. OID parsing and formatting works
2. Reference types serialize correctly
3. Commit creation captures all fields
4. Error types are informative
5. git2 interop is seamless

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [452-git-detect.md](452-git-detect.md)
- Used by: All Git integration components
