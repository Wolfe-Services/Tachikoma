# 471a - VCS Crate Setup (jj-first)

**Phase:** 21 - VCS Integration
**Spec ID:** 471a
**Status:** Planned
**Dependencies:** 011-common-core-types
**Estimated Context:** ~4% of Sonnet window

---

## Objective

Create the `tachikoma-vcs` crate with jj (Jujutsu) as the primary version control system. jj is superior for agentic coding due to its conflict-free concurrent editing model, operation log for undo/redo, and native git compatibility.

---

## Why jj for Agentic Coding

| Feature | jj Advantage | Git Limitation |
|---------|--------------|----------------|
| Concurrent edits | First-class support, automatic rebasing | Manual merge hell |
| Working copy | Not special, just another commit | Dirty state blocks operations |
| Undo/redo | Operation log, trivial to undo anything | Reflog is obscure |
| Conflicts | Can commit with conflicts, resolve later | Blocks all progress |
| Branching | Anonymous by default, names optional | Requires explicit branch |
| Agent-friendly | No staging area complexity | Index/staging confusion |

---

## Acceptance Criteria

- [ ] `tachikoma-vcs` crate created with workspace integration
- [ ] jj library integration (jj-lib)
- [ ] Git compatibility layer for remote operations
- [ ] VCS trait abstraction (future-proofs for other VCS)
- [ ] Feature flags for jj vs git-only mode

---

## Implementation Details

### Cargo.toml

```toml
[package]
name = "tachikoma-vcs"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "VCS integration for Tachikoma (jj-first, git-compatible)"

[features]
default = ["jj"]
jj = ["jj-lib"]
git-only = ["git2"]

[dependencies]
tachikoma-common-core.workspace = true
thiserror.workspace = true
serde = { workspace = true, features = ["derive"] }
tracing.workspace = true

# jj (Jujutsu) - primary VCS
jj-lib = { version = "0.24", optional = true }

# Git - for compatibility and fallback
git2 = { version = "0.19", optional = true }

# Async support
tokio = { workspace = true, features = ["process"] }

[dev-dependencies]
tempfile = "3"
```

### src/lib.rs

```rust
//! Tachikoma VCS Integration
//!
//! jj-first version control with git compatibility.
//!
//! # Why jj?
//!
//! jj (Jujutsu) is ideal for agentic coding because:
//! - Concurrent edits are first-class (agents can work in parallel)
//! - No staging area confusion
//! - Conflicts can be committed and resolved later
//! - Operation log makes any operation undoable
//! - Native git compatibility for remotes

#![warn(missing_docs)]

pub mod types;
pub mod traits;

#[cfg(feature = "jj")]
pub mod jj;

#[cfg(feature = "git-only")]
pub mod git;

pub mod compat;

pub use types::*;
pub use traits::*;
```

### src/types.rs

```rust
//! Core VCS types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tachikoma_common_core::Timestamp;

/// A change ID (jj) or commit hash (git).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeId(pub String);

/// A commit ID (full hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitId(pub String);

/// Repository information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    /// Root path of the repository.
    pub root: PathBuf,
    /// VCS type.
    pub vcs_type: VcsType,
    /// Current working copy change/commit.
    pub working_copy: ChangeId,
    /// Current branch (if any).
    pub branch: Option<String>,
    /// Whether there are uncommitted changes.
    pub is_dirty: bool,
}

/// VCS type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VcsType {
    /// jj (Jujutsu) - preferred
    Jj,
    /// Git - fallback/compatibility
    Git,
}

/// A file change in the working copy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path relative to repo root.
    pub path: PathBuf,
    /// Type of change.
    pub change_type: ChangeType,
    /// Whether the file has conflicts.
    pub has_conflict: bool,
}

/// Type of file change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Conflicted,
}

/// A conflict in a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Path to conflicted file.
    pub path: PathBuf,
    /// Number of conflict regions.
    pub conflict_count: usize,
    /// Sides of the conflict.
    pub sides: Vec<ConflictSide>,
}

/// One side of a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSide {
    /// Description of this side.
    pub description: String,
    /// Content of this side.
    pub content: String,
}

/// Operation in the operation log (jj feature).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation ID.
    pub id: String,
    /// Operation description.
    pub description: String,
    /// Timestamp.
    pub timestamp: Timestamp,
    /// Whether this can be undone.
    pub undoable: bool,
}

/// Result of a VCS operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsResult {
    /// Whether the operation succeeded.
    pub success: bool,
    /// New change/commit ID if applicable.
    pub change_id: Option<ChangeId>,
    /// Human-readable message.
    pub message: String,
    /// Files affected.
    pub affected_files: Vec<PathBuf>,
    /// Any conflicts created.
    pub conflicts: Vec<Conflict>,
}
```

---

## Testing Requirements

1. Crate compiles with jj feature
2. Crate compiles with git-only feature
3. Types serialize/deserialize correctly
4. VcsType detection works

---

## Related Specs

- Next: [471b-jj-repository.md](471b-jj-repository.md)
- Git compat: [471e-git-compat.md](471e-git-compat.md)
