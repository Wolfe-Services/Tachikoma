//! Detached HEAD operations.

use crate::{GitOid, GitRepository, GitResult, GitError};

impl GitRepository {
    /// Checkout a specific commit (detached HEAD).
    pub fn checkout_commit(&self, oid: &GitOid) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let commit = repo.find_commit(oid.as_git2())?;
            let tree = commit.tree()?;

            repo.checkout_tree(tree.as_object(), None)?;
            repo.set_head_detached(oid.as_git2())?;

            Ok(())
        })
    }

    /// Check if HEAD is detached.
    pub fn is_head_detached(&self) -> GitResult<bool> {
        self.with_repo(|repo| {
            Ok(repo.head_detached()?)
        })
    }

    /// Get current HEAD as OID (works for both attached and detached).
    pub fn head_oid(&self) -> GitResult<GitOid> {
        self.with_repo(|repo| {
            let head = repo.head()?;
            let oid = head.target().ok_or_else(|| GitError::RefNotFound {
                name: "HEAD".to_string(),
            })?;
            Ok(GitOid::from_git2(oid))
        })
    }
}