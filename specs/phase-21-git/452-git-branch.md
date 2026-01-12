# Spec 452: Branch Management

## Phase
21 - Git Integration

## Spec ID
452

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)

## Estimated Context
~10%

---

## Objective

Implement comprehensive Git branch management for Tachikoma, providing functionality to create, delete, rename, and switch branches. This module supports branch listing, upstream tracking, and comparison operations essential for managing development workflows.

---

## Acceptance Criteria

- [ ] Implement `GitBranchManager` for branch operations
- [ ] Support creating branches (with optional start point)
- [ ] Support deleting branches (with force option)
- [ ] Support renaming branches
- [ ] Implement branch checkout (switch)
- [ ] Support listing local and remote branches
- [ ] Implement upstream tracking configuration
- [ ] Calculate ahead/behind counts
- [ ] Support branch comparison
- [ ] Implement branch cleanup (merged branches)

---

## Implementation Details

### Branch Manager Implementation

```rust
// src/git/branch.rs

use git2::{Branch, BranchType, Repository};
use std::collections::HashMap;

use super::repo::GitRepository;
use super::types::*;

/// Options for creating a branch
#[derive(Debug, Clone, Default)]
pub struct CreateBranchOptions {
    /// Start point (commit, branch, or tag). Defaults to HEAD.
    pub start_point: Option<String>,
    /// Force creation (overwrite if exists)
    pub force: bool,
    /// Set up tracking for remote branch
    pub track: Option<String>,
    /// Checkout after creation
    pub checkout: bool,
}

impl CreateBranchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_point(mut self, start: impl Into<String>) -> Self {
        self.start_point = Some(start.into());
        self
    }

    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn track(mut self, remote_branch: impl Into<String>) -> Self {
        self.track = Some(remote_branch.into());
        self
    }

    pub fn checkout(mut self, checkout: bool) -> Self {
        self.checkout = checkout;
        self
    }
}

/// Options for deleting a branch
#[derive(Debug, Clone, Default)]
pub struct DeleteBranchOptions {
    /// Force delete (even if not merged)
    pub force: bool,
    /// Delete remote tracking branch too
    pub delete_remote: bool,
}

impl DeleteBranchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
}

/// Options for listing branches
#[derive(Debug, Clone, Default)]
pub struct ListBranchOptions {
    /// Include local branches
    pub local: bool,
    /// Include remote branches
    pub remote: bool,
    /// Filter by pattern
    pub pattern: Option<String>,
    /// Only show merged branches
    pub merged_into: Option<String>,
    /// Only show unmerged branches
    pub no_merged_into: Option<String>,
    /// Sort by (name, committerdate, authordate)
    pub sort: BranchSort,
}

impl ListBranchOptions {
    pub fn new() -> Self {
        Self {
            local: true,
            remote: false,
            ..Default::default()
        }
    }

    pub fn all() -> Self {
        Self {
            local: true,
            remote: true,
            ..Default::default()
        }
    }

    pub fn remote_only() -> Self {
        Self {
            local: false,
            remote: true,
            ..Default::default()
        }
    }

    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    pub fn merged_into(mut self, branch: impl Into<String>) -> Self {
        self.merged_into = Some(branch.into());
        self
    }
}

/// Branch sorting options
#[derive(Debug, Clone, Copy, Default)]
pub enum BranchSort {
    #[default]
    Name,
    CommitterDate,
    AuthorDate,
}

/// Options for checkout operation
#[derive(Debug, Clone, Default)]
pub struct CheckoutOptions {
    /// Force checkout (discard local changes)
    pub force: bool,
    /// Create branch if it doesn't exist
    pub create: bool,
    /// Detach HEAD
    pub detach: bool,
    /// Merge local modifications
    pub merge: bool,
}

impl CheckoutOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn create(mut self) -> Self {
        self.create = true;
        self
    }

    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }
}

/// Branch comparison result
#[derive(Debug, Clone)]
pub struct BranchComparison {
    pub base: String,
    pub compare: String,
    pub ahead: usize,
    pub behind: usize,
    pub common_ancestor: Option<GitOid>,
}

impl BranchComparison {
    pub fn is_up_to_date(&self) -> bool {
        self.ahead == 0 && self.behind == 0
    }

    pub fn can_fast_forward(&self) -> bool {
        self.behind == 0 && self.ahead > 0
    }

    pub fn has_diverged(&self) -> bool {
        self.ahead > 0 && self.behind > 0
    }
}

/// Git branch manager
pub struct GitBranchManager<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitBranchManager<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Create a new branch
    pub fn create(&self, name: &str, options: CreateBranchOptions) -> GitResult<GitBranch> {
        let raw_repo = self.repo.raw();

        // Get start point commit
        let start_commit = match &options.start_point {
            Some(spec) => {
                let obj = raw_repo.revparse_single(spec)?;
                obj.peel_to_commit()?
            }
            None => raw_repo.head()?.peel_to_commit()?,
        };

        // Create branch
        let branch = raw_repo.branch(name, &start_commit, options.force)?;

        // Set up tracking if requested
        if let Some(ref upstream) = options.track {
            self.set_upstream(name, upstream)?;
        }

        // Checkout if requested
        if options.checkout {
            self.checkout(name, CheckoutOptions::default())?;
        }

        self.branch_to_git_branch(&branch, false)
    }

    /// Delete a branch
    pub fn delete(&self, name: &str, options: DeleteBranchOptions) -> GitResult<()> {
        let raw_repo = self.repo.raw();

        let mut branch = raw_repo.find_branch(name, BranchType::Local)?;

        // Check if branch is merged (unless force)
        if !options.force {
            if !self.is_merged(name)? {
                return Err(GitError::Other(format!(
                    "Branch '{}' is not fully merged. Use force to delete anyway.",
                    name
                )));
            }
        }

        // Check if this is the current branch
        if branch.is_head() {
            return Err(GitError::Other(format!(
                "Cannot delete branch '{}' because it is currently checked out",
                name
            )));
        }

        branch.delete()?;
        Ok(())
    }

    /// Rename a branch
    pub fn rename(&self, old_name: &str, new_name: &str, force: bool) -> GitResult<GitBranch> {
        let raw_repo = self.repo.raw();

        let mut branch = raw_repo.find_branch(old_name, BranchType::Local)?;
        let new_branch = branch.rename(new_name, force)?;

        self.branch_to_git_branch(&new_branch, new_branch.is_head())
    }

    /// Checkout a branch
    pub fn checkout(&self, name: &str, options: CheckoutOptions) -> GitResult<()> {
        let raw_repo = self.repo.raw();

        // Handle branch creation
        if options.create {
            if raw_repo.find_branch(name, BranchType::Local).is_err() {
                self.create(name, CreateBranchOptions::default())?;
            }
        }

        // Get reference
        let ref_name = if options.detach {
            // For detached HEAD, resolve to commit
            let obj = raw_repo.revparse_single(name)?;
            raw_repo.set_head_detached(obj.id())?;
            return Ok(());
        } else {
            format!("refs/heads/{}", name)
        };

        // Build checkout options
        let mut checkout_opts = git2::build::CheckoutBuilder::new();

        if options.force {
            checkout_opts.force();
        } else {
            checkout_opts.safe();
        }

        // Perform checkout
        let obj = raw_repo.revparse_single(&ref_name)?;
        raw_repo.checkout_tree(&obj, Some(&mut checkout_opts))?;
        raw_repo.set_head(&ref_name)?;

        Ok(())
    }

    /// Get current branch
    pub fn current(&self) -> GitResult<Option<GitBranch>> {
        let raw_repo = self.repo.raw();

        let head = match raw_repo.head() {
            Ok(h) => h,
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => return Ok(None),
            Err(e) => return Err(GitError::Git2(e)),
        };

        if !head.is_branch() {
            return Ok(None); // Detached HEAD
        }

        let branch_name = head.shorthand().ok_or_else(|| {
            GitError::Other("Cannot get branch name".into())
        })?;

        let branch = raw_repo.find_branch(branch_name, BranchType::Local)?;
        let git_branch = self.branch_to_git_branch(&branch, true)?;

        Ok(Some(git_branch))
    }

    /// List branches
    pub fn list(&self, options: ListBranchOptions) -> GitResult<Vec<GitBranch>> {
        let raw_repo = self.repo.raw();
        let mut branches = Vec::new();

        // Get current branch for is_head comparison
        let head_name = raw_repo.head().ok().and_then(|h| h.shorthand().map(String::from));

        // List local branches
        if options.local {
            let local_branches = raw_repo.branches(Some(BranchType::Local))?;
            for branch_result in local_branches {
                let (branch, _) = branch_result?;
                let is_head = branch.name()?.map(|n| Some(n.to_string()) == head_name).unwrap_or(false);

                if let Ok(git_branch) = self.branch_to_git_branch(&branch, is_head) {
                    // Apply filters
                    if self.branch_matches_filters(&git_branch, &options)? {
                        branches.push(git_branch);
                    }
                }
            }
        }

        // List remote branches
        if options.remote {
            let remote_branches = raw_repo.branches(Some(BranchType::Remote))?;
            for branch_result in remote_branches {
                let (branch, _) = branch_result?;

                if let Ok(mut git_branch) = self.branch_to_git_branch(&branch, false) {
                    git_branch.is_remote = true;
                    if self.branch_matches_filters(&git_branch, &options)? {
                        branches.push(git_branch);
                    }
                }
            }
        }

        // Sort
        match options.sort {
            BranchSort::Name => branches.sort_by(|a, b| a.name.cmp(&b.name)),
            BranchSort::CommitterDate | BranchSort::AuthorDate => {
                // Would need to look up commits for proper sorting
                branches.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }

        Ok(branches)
    }

    /// Set upstream tracking branch
    pub fn set_upstream(&self, branch: &str, upstream: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        let mut local_branch = raw_repo.find_branch(branch, BranchType::Local)?;
        local_branch.set_upstream(Some(upstream))?;
        Ok(())
    }

    /// Unset upstream tracking branch
    pub fn unset_upstream(&self, branch: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        let mut local_branch = raw_repo.find_branch(branch, BranchType::Local)?;
        local_branch.set_upstream(None)?;
        Ok(())
    }

    /// Compare two branches
    pub fn compare(&self, base: &str, compare: &str) -> GitResult<BranchComparison> {
        let raw_repo = self.repo.raw();

        let base_oid = raw_repo.revparse_single(base)?.id();
        let compare_oid = raw_repo.revparse_single(compare)?.id();

        let (ahead, behind) = raw_repo.graph_ahead_behind(compare_oid, base_oid)?;

        let common_ancestor = raw_repo
            .merge_base(base_oid, compare_oid)
            .ok()
            .map(GitOid::from);

        Ok(BranchComparison {
            base: base.to_string(),
            compare: compare.to_string(),
            ahead,
            behind,
            common_ancestor,
        })
    }

    /// Check if a branch is merged into current branch
    pub fn is_merged(&self, branch: &str) -> GitResult<bool> {
        let raw_repo = self.repo.raw();

        let head_oid = raw_repo.head()?.target()
            .ok_or_else(|| GitError::Other("HEAD has no target".into()))?;

        let branch_oid = raw_repo.revparse_single(branch)?.id();

        // A branch is merged if it's an ancestor of HEAD
        Ok(raw_repo.graph_descendant_of(head_oid, branch_oid)?)
    }

    /// List merged branches
    pub fn list_merged(&self, into: Option<&str>) -> GitResult<Vec<GitBranch>> {
        let target = into.unwrap_or("HEAD");

        self.list(ListBranchOptions::new().merged_into(target))
    }

    /// Delete all merged branches
    pub fn cleanup_merged(&self, protect: &[&str]) -> GitResult<Vec<String>> {
        let merged = self.list_merged(None)?;
        let current = self.current()?.map(|b| b.name);
        let mut deleted = Vec::new();

        for branch in merged {
            // Skip protected branches
            if protect.contains(&branch.name.as_str()) {
                continue;
            }

            // Skip current branch
            if Some(&branch.name) == current.as_ref() {
                continue;
            }

            // Skip remote branches
            if branch.is_remote {
                continue;
            }

            if self.delete(&branch.name, DeleteBranchOptions::default()).is_ok() {
                deleted.push(branch.name);
            }
        }

        Ok(deleted)
    }

    fn branch_to_git_branch(&self, branch: &Branch, is_head: bool) -> GitResult<GitBranch> {
        let raw_repo = self.repo.raw();

        let name = branch.name()?.unwrap_or("").to_string();
        let full_name = branch.get().name().unwrap_or("").to_string();
        let oid = branch.get().target().map(GitOid::from).unwrap_or_else(|| GitOid([0; 20]));

        // Get upstream info
        let (upstream, ahead, behind) = match branch.upstream() {
            Ok(upstream) => {
                let upstream_name = upstream.name()?.map(String::from);
                let local_oid = branch.get().target();
                let upstream_oid = upstream.get().target();

                let (ahead, behind) = match (local_oid, upstream_oid) {
                    (Some(l), Some(u)) => raw_repo.graph_ahead_behind(l, u).unwrap_or((0, 0)),
                    _ => (0, 0),
                };

                (upstream_name, Some(ahead), Some(behind))
            }
            Err(_) => (None, None, None),
        };

        Ok(GitBranch {
            name,
            full_name,
            oid,
            is_head,
            is_remote: branch.get().is_remote(),
            upstream,
            ahead,
            behind,
        })
    }

    fn branch_matches_filters(&self, branch: &GitBranch, options: &ListBranchOptions) -> GitResult<bool> {
        // Pattern filter
        if let Some(ref pattern) = options.pattern {
            if !branch.name.contains(pattern) {
                return Ok(false);
            }
        }

        // Merged filter
        if let Some(ref target) = options.merged_into {
            let raw_repo = self.repo.raw();
            let target_oid = raw_repo.revparse_single(target)?.id();

            if !raw_repo.graph_descendant_of(target_oid, branch.oid.to_git2_oid())? {
                return Ok(false);
            }
        }

        // Not merged filter
        if let Some(ref target) = options.no_merged_into {
            let raw_repo = self.repo.raw();
            let target_oid = raw_repo.revparse_single(target)?.id();

            if raw_repo.graph_descendant_of(target_oid, branch.oid.to_git2_oid())? {
                return Ok(false);
            }
        }

        Ok(true)
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
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();

        // Configure user
        let mut config = repo.config().unwrap();
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

        // Create initial commit
        std::fs::write(dir.path().join("README.md"), "# Test").unwrap();
        repo.stage_file(std::path::Path::new("README.md")).unwrap();

        let raw = repo.raw();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();

        (dir, repo)
    }

    #[test]
    fn test_create_branch() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        let branch = manager.create("feature", CreateBranchOptions::default()).unwrap();

        assert_eq!(branch.name, "feature");
        assert!(!branch.is_head);
    }

    #[test]
    fn test_create_and_checkout() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        manager.create("feature", CreateBranchOptions::new().checkout(true)).unwrap();

        let current = manager.current().unwrap().unwrap();
        assert_eq!(current.name, "feature");
    }

    #[test]
    fn test_list_branches() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        manager.create("feature-1", CreateBranchOptions::default()).unwrap();
        manager.create("feature-2", CreateBranchOptions::default()).unwrap();

        let branches = manager.list(ListBranchOptions::new()).unwrap();

        assert!(branches.len() >= 3); // master/main + 2 features
    }

    #[test]
    fn test_delete_branch() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        manager.create("to-delete", CreateBranchOptions::default()).unwrap();
        manager.delete("to-delete", DeleteBranchOptions::new().force()).unwrap();

        let branches = manager.list(ListBranchOptions::new()).unwrap();
        assert!(!branches.iter().any(|b| b.name == "to-delete"));
    }

    #[test]
    fn test_cannot_delete_current_branch() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        let current = manager.current().unwrap().unwrap();
        let result = manager.delete(&current.name, DeleteBranchOptions::default());

        assert!(result.is_err());
    }

    #[test]
    fn test_rename_branch() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        manager.create("old-name", CreateBranchOptions::default()).unwrap();
        manager.rename("old-name", "new-name", false).unwrap();

        let branches = manager.list(ListBranchOptions::new()).unwrap();
        assert!(branches.iter().any(|b| b.name == "new-name"));
        assert!(!branches.iter().any(|b| b.name == "old-name"));
    }

    #[test]
    fn test_branch_comparison() {
        let (dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        // Create a branch and add a commit
        manager.create("feature", CreateBranchOptions::new().checkout(true)).unwrap();

        std::fs::write(dir.path().join("feature.txt"), "feature").unwrap();
        repo.stage_file(std::path::Path::new("feature.txt")).unwrap();

        let raw = repo.raw();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        let head = raw.head().unwrap().peel_to_commit().unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Feature commit", &tree, &[&head]).unwrap();

        // Compare
        let comparison = manager.compare("master", "feature").unwrap();

        assert_eq!(comparison.ahead, 1);
        assert_eq!(comparison.behind, 0);
        assert!(comparison.can_fast_forward());
    }

    #[test]
    fn test_checkout_options() {
        let opts = CheckoutOptions::new().force().create();
        assert!(opts.force);
        assert!(opts.create);
    }

    #[test]
    fn test_list_with_pattern() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitBranchManager::new(&repo);

        manager.create("feature-auth", CreateBranchOptions::default()).unwrap();
        manager.create("feature-ui", CreateBranchOptions::default()).unwrap();
        manager.create("bugfix-123", CreateBranchOptions::default()).unwrap();

        let branches = manager.list(ListBranchOptions::new().pattern("feature")).unwrap();

        assert_eq!(branches.len(), 2);
        assert!(branches.iter().all(|b| b.name.contains("feature")));
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 453: Remote Operations
- Spec 456: Merge Operations
