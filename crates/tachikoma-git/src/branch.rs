//! Git branch operations.

use crate::{GitBranch, GitOid, GitRepository, GitResult, GitError, GitRef, RefType};
use git2::BranchType;

impl GitRepository {
    /// List all local branches.
    pub fn branches_local(&self) -> GitResult<Vec<GitBranch>> {
        self.list_branches(BranchType::Local)
    }

    /// List all remote branches.
    pub fn branches_remote(&self) -> GitResult<Vec<GitBranch>> {
        self.list_branches(BranchType::Remote)
    }

    /// List all branches.
    pub fn branches_all(&self) -> GitResult<Vec<GitBranch>> {
        let mut branches = self.branches_local()?;
        branches.extend(self.branches_remote()?);
        Ok(branches)
    }

    fn list_branches(&self, branch_type: BranchType) -> GitResult<Vec<GitBranch>> {
        self.with_repo(|repo| {
            let branches = repo.branches(Some(branch_type))?;
            let head = repo.head().ok();
            let head_name = head.as_ref().and_then(|h| h.shorthand()).map(String::from);

            let mut result = Vec::new();
            for branch in branches {
                let (branch, _) = branch?;
                if let Some(git_branch) = self.parse_branch(&branch, head_name.as_deref())? {
                    result.push(git_branch);
                }
            }

            Ok(result)
        })
    }

    fn parse_branch(
        &self,
        branch: &git2::Branch,
        head_name: Option<&str>,
    ) -> GitResult<Option<GitBranch>> {
        let name = match branch.name()? {
            Some(n) => n.to_string(),
            None => return Ok(None),
        };

        let is_current = head_name.map(|h| h == name).unwrap_or(false);

        let commit = branch
            .get()
            .target()
            .map(GitOid::from_git2)
            .ok_or_else(|| GitError::RefNotFound { name: name.clone() })?;

        let (upstream, ahead, behind) = if let Ok(upstream_branch) = branch.upstream() {
            let upstream_name = upstream_branch.name()?.map(String::from);

            let (ahead, behind) = self.with_repo(|repo| {
                if let (Some(local_oid), Some(upstream_oid)) = (
                    branch.get().target(),
                    upstream_branch.get().target(),
                ) {
                    Ok(repo.graph_ahead_behind(local_oid, upstream_oid)?)
                } else {
                    Ok((0, 0))
                }
            })?;

            (upstream_name, Some(ahead as u32), Some(behind as u32))
        } else {
            (None, None, None)
        };

        Ok(Some(GitBranch {
            name,
            is_current,
            upstream,
            commit,
            ahead,
            behind,
        }))
    }

    /// Create a new branch.
    pub fn create_branch(&self, name: &str, target: Option<&GitOid>) -> GitResult<GitBranch> {
        self.with_repo_mut(|repo| {
            let commit = match target {
                Some(oid) => repo.find_commit(oid.as_git2())?,
                None => repo.head()?.peel_to_commit()?,
            };

            let branch = repo.branch(name, &commit, false)?;
            let branch_ref = self.parse_branch(&branch, None)?;

            branch_ref.ok_or_else(|| GitError::BranchNotFound {
                name: name.to_string(),
            })
        })
    }

    /// Create a branch from a reference (commit, tag, branch).
    pub fn create_branch_from_ref(&self, name: &str, reference: &str) -> GitResult<GitBranch> {
        self.with_repo_mut(|repo| {
            let obj = repo.revparse_single(reference)?;
            let commit = obj.peel_to_commit()?;

            let branch = repo.branch(name, &commit, false)?;
            let branch_ref = self.parse_branch(&branch, None)?;

            branch_ref.ok_or_else(|| GitError::BranchNotFound {
                name: name.to_string(),
            })
        })
    }

    /// Delete a branch.
    pub fn delete_branch(&self, name: &str, force: bool) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let mut branch = repo.find_branch(name, BranchType::Local)?;

            // Check if it's the current branch
            if branch.is_head() {
                return Err(GitError::InvalidOperation {
                    message: "Cannot delete the current branch".to_string(),
                });
            }

            // Check if branch is fully merged (unless force)
            if !force {
                let head = repo.head()?.peel_to_commit()?;
                let branch_commit = branch.get().peel_to_commit()?;

                let merge_base = repo.merge_base(head.id(), branch_commit.id())?;
                if merge_base != branch_commit.id() {
                    return Err(GitError::InvalidOperation {
                        message: format!(
                            "Branch '{}' is not fully merged. Use force to delete anyway.",
                            name
                        ),
                    });
                }
            }

            branch.delete()?;
            Ok(())
        })
    }

    /// Rename a branch.
    pub fn rename_branch(&self, old_name: &str, new_name: &str, force: bool) -> GitResult<GitBranch> {
        self.with_repo_mut(|repo| {
            let mut branch = repo.find_branch(old_name, BranchType::Local)?;
            let new_branch = branch.rename(new_name, force)?;
            let branch_ref = self.parse_branch(&new_branch, None)?;

            branch_ref.ok_or_else(|| GitError::BranchNotFound {
                name: new_name.to_string(),
            })
        })
    }

    /// Checkout a branch.
    pub fn checkout_branch(&self, name: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            // Find the branch
            let branch = repo.find_branch(name, BranchType::Local)?;
            let reference = branch.into_reference();
            let commit = reference.peel_to_commit()?;

            // Checkout the tree
            let tree = commit.tree()?;
            repo.checkout_tree(tree.as_object(), None)?;

            // Update HEAD
            repo.set_head(reference.name().ok_or_else(|| GitError::RefNotFound {
                name: name.to_string(),
            })?)?;

            Ok(())
        })
    }

    /// Checkout a branch, creating it if it doesn't exist.
    pub fn checkout_branch_create(&self, name: &str, start_point: Option<&str>) -> GitResult<()> {
        // Try to checkout existing branch
        if self.branch_exists(name)? {
            return self.checkout_branch(name);
        }

        // Create from start point or HEAD
        if let Some(ref_name) = start_point {
            self.create_branch_from_ref(name, ref_name)?;
        } else {
            self.create_branch(name, None)?;
        }

        self.checkout_branch(name)
    }

    /// Check if a branch exists.
    pub fn branch_exists(&self, name: &str) -> GitResult<bool> {
        self.with_repo(|repo| {
            Ok(repo.find_branch(name, BranchType::Local).is_ok())
        })
    }

    /// Get the current branch name.
    pub fn current_branch(&self) -> GitResult<Option<String>> {
        self.with_repo(|repo| {
            let head = repo.head()?;
            if head.is_branch() {
                Ok(head.shorthand().map(String::from))
            } else {
                Ok(None) // Detached HEAD
            }
        })
    }

    /// Set upstream for a branch.
    pub fn set_upstream(&self, branch_name: &str, upstream: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
            branch.set_upstream(Some(upstream))?;
            Ok(())
        })
    }

    /// Unset upstream for a branch.
    pub fn unset_upstream(&self, branch_name: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
            branch.set_upstream(None)?;
            Ok(())
        })
    }
}