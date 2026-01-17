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