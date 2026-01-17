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