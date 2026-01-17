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