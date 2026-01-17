//! Git worktree management.

use crate::{GitOid, GitRepository, GitResult, GitError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Worktree information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitWorktree {
    /// Worktree name.
    pub name: String,
    /// Path to the worktree.
    pub path: PathBuf,
    /// Whether this is the main worktree.
    pub is_main: bool,
    /// Whether the worktree is locked.
    pub is_locked: bool,
    /// Lock reason (if locked).
    pub lock_reason: Option<String>,
    /// Current HEAD commit.
    pub head: Option<GitOid>,
    /// Current branch name.
    pub branch: Option<String>,
    /// Whether the worktree is valid (path exists).
    pub is_valid: bool,
}

/// Worktree add options.
#[derive(Debug, Clone, Default)]
pub struct WorktreeAddOptions {
    /// Create a new branch.
    pub new_branch: Option<String>,
    /// Force creation even if branch exists elsewhere.
    pub force: bool,
    /// Checkout after creating.
    pub checkout: bool,
    /// Lock the worktree after creation.
    pub lock: bool,
    /// Detach HEAD (don't checkout branch).
    pub detach: bool,
}

impl WorktreeAddOptions {
    /// Create with a new branch.
    pub fn with_branch(branch: impl Into<String>) -> Self {
        Self {
            new_branch: Some(branch.into()),
            checkout: true,
            ..Default::default()
        }
    }

    /// Set force flag.
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Set lock flag.
    pub fn lock(mut self) -> Self {
        self.lock = true;
        self
    }

    /// Set detach flag.
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }
}

impl GitRepository {
    /// List all worktrees.
    pub fn list_worktrees(&self) -> GitResult<Vec<GitWorktree>> {
        self.with_repo(|repo| {
            let mut worktrees = Vec::new();

            // Add main worktree
            let main_path = repo.workdir()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| repo.path().to_path_buf());

            let head_ref = repo.head().ok();
            let head_oid = head_ref.as_ref()
                .and_then(|r| r.target())
                .map(GitOid::from_git2);
            let branch = head_ref.as_ref()
                .filter(|r| r.is_branch())
                .and_then(|r| r.shorthand())
                .map(String::from);

            worktrees.push(GitWorktree {
                name: String::from("main"),
                path: main_path.clone(),
                is_main: true,
                is_locked: false,
                lock_reason: None,
                head: head_oid,
                branch,
                is_valid: main_path.exists(),
            });

            // List linked worktrees
            let worktree_names = repo.worktrees()?;
            for name in worktree_names.iter() {
                if let Some(name) = name {
                    if let Ok(wt) = repo.find_worktree(name) {
                        let wt_path = wt.path().to_path_buf();
                        let is_locked = wt.is_locked();
                        let lock_reason = if is_locked {
                            // git2 doesn't expose lock reason directly
                            None
                        } else {
                            None
                        };

                        // Get worktree HEAD
                        let (wt_head, wt_branch) = self.get_worktree_head(&wt_path);

                        worktrees.push(GitWorktree {
                            name: name.to_string(),
                            path: wt_path.clone(),
                            is_main: false,
                            is_locked,
                            lock_reason,
                            head: wt_head,
                            branch: wt_branch,
                            is_valid: wt.validate().is_ok(),
                        });
                    }
                }
            }

            Ok(worktrees)
        })
    }

    /// Get worktree HEAD information.
    fn get_worktree_head(&self, path: &Path) -> (Option<GitOid>, Option<String>) {
        // Try to open worktree as a repository
        if let Ok(wt_repo) = git2::Repository::open(path) {
            let head_ref = wt_repo.head().ok();
            let head_oid = head_ref.as_ref()
                .and_then(|r| r.target())
                .map(GitOid::from_git2);
            let branch = head_ref.as_ref()
                .filter(|r| r.is_branch())
                .and_then(|r| r.shorthand())
                .map(String::from);
            (head_oid, branch)
        } else {
            (None, None)
        }
    }

    /// Add a new worktree.
    pub fn add_worktree(
        &self,
        path: impl AsRef<Path>,
        reference: &str,
        options: WorktreeAddOptions,
    ) -> GitResult<GitWorktree> {
        let path = path.as_ref();

        self.with_repo_mut(|repo| {
            let mut wt_opts = git2::WorktreeAddOptions::new();

            if let Some(ref branch) = options.new_branch {
                // Create new branch from reference
                let target = repo.revparse_single(reference)?;
                let commit = target.peel_to_commit()?;
                let branch_ref = repo.branch(branch, &commit, options.force)?;
                wt_opts.reference(Some(branch_ref.get()));
            } else {
                // Use existing reference
                let reference = repo.find_reference(reference)
                    .or_else(|_| repo.find_branch(reference, git2::BranchType::Local)
                        .map(|b| b.into_reference()))?;
                wt_opts.reference(Some(&reference));
            }

            if options.lock {
                wt_opts.lock(true);
            }

            // Generate worktree name from path
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("worktree")
                .to_string();

            repo.worktree(&name, path, Some(&wt_opts))?;

            Ok(())
        })?;

        // Return information about the created worktree
        let worktrees = self.list_worktrees()?;
        worktrees.into_iter()
            .find(|wt| wt.path == path)
            .ok_or_else(|| GitError::InvalidOperation {
                message: "Failed to find created worktree".to_string(),
            })
    }

    /// Remove a worktree.
    pub fn remove_worktree(&self, name: &str, force: bool) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let wt = repo.find_worktree(name)?;

            // Check if locked
            if wt.is_locked() && !force {
                return Err(GitError::InvalidOperation {
                    message: format!("Worktree '{}' is locked. Use force to remove.", name),
                });
            }

            // Validate worktree
            if wt.validate().is_err() && !force {
                return Err(GitError::InvalidOperation {
                    message: format!("Worktree '{}' is invalid. Use force to remove.", name),
                });
            }

            // Prune the worktree (removes the administrative files)
            wt.prune(Some(&mut git2::WorktreePruneOptions::new().working_tree(true)))?;

            Ok(())
        })
    }

    /// Lock a worktree.
    pub fn lock_worktree(&self, name: &str, reason: Option<&str>) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let wt = repo.find_worktree(name)?;

            if wt.is_locked() {
                return Err(GitError::InvalidOperation {
                    message: format!("Worktree '{}' is already locked", name),
                });
            }

            wt.lock(reason)?;
            Ok(())
        })
    }

    /// Unlock a worktree.
    pub fn unlock_worktree(&self, name: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let wt = repo.find_worktree(name)?;

            if !wt.is_locked() {
                return Err(GitError::InvalidOperation {
                    message: format!("Worktree '{}' is not locked", name),
                });
            }

            wt.unlock()?;
            Ok(())
        })
    }

    /// Prune stale worktrees.
    pub fn prune_worktrees(&self) -> GitResult<Vec<String>> {
        let mut pruned = Vec::new();

        self.with_repo_mut(|repo| {
            let worktree_names = repo.worktrees()?;

            for name in worktree_names.iter() {
                if let Some(name) = name {
                    if let Ok(wt) = repo.find_worktree(name) {
                        // Check if worktree is stale (path doesn't exist)
                        if wt.validate().is_err() {
                            let mut prune_opts = git2::WorktreePruneOptions::new();
                            prune_opts.valid(true);

                            if wt.prune(Some(&mut prune_opts)).is_ok() {
                                pruned.push(name.to_string());
                            }
                        }
                    }
                }
            }

            Ok(())
        })?;

        Ok(pruned)
    }

    /// Get a specific worktree.
    pub fn get_worktree(&self, name: &str) -> GitResult<GitWorktree> {
        let worktrees = self.list_worktrees()?;
        worktrees.into_iter()
            .find(|wt| wt.name == name)
            .ok_or_else(|| GitError::InvalidOperation {
                message: format!("Worktree '{}' not found", name),
            })
    }

    /// Check if a path is in a worktree.
    pub fn is_worktree(&self) -> GitResult<bool> {
        self.with_repo(|repo| {
            Ok(repo.is_worktree())
        })
    }

    /// Get the main repository path from a worktree.
    pub fn main_repo_path(&self) -> GitResult<PathBuf> {
        self.with_repo(|repo| {
            if repo.is_worktree() {
                // Find the main repo via commondir
                let commondir = repo.path().join("commondir");
                if commondir.exists() {
                    let common = std::fs::read_to_string(&commondir)?;
                    let common_path = repo.path().join(common.trim());
                    return Ok(common_path.parent()
                        .unwrap_or(&common_path)
                        .to_path_buf());
                }
            }
            Ok(repo.workdir()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| repo.path().to_path_buf()))
        })
    }

    /// Check if a branch is merged into another branch.
    pub fn is_branch_merged(&self, branch: &str, target: &str) -> GitResult<bool> {
        self.with_repo(|repo| {
            let branch_ref = repo.find_branch(branch, git2::BranchType::Local)?;
            let target_ref = repo.find_branch(target, git2::BranchType::Local)?;

            let branch_commit = branch_ref.get().peel_to_commit()?;
            let target_commit = target_ref.get().peel_to_commit()?;

            // Check if branch commit is ancestor of target commit
            let merge_base = repo.merge_base(branch_commit.id(), target_commit.id())?;
            Ok(merge_base == branch_commit.id())
        })
    }
}

/// Worktree operations for parallel development.
pub struct WorktreeManager {
    main_repo: GitRepository,
}

impl WorktreeManager {
    /// Create a new worktree manager.
    pub fn new(repo: GitRepository) -> Self {
        Self { main_repo: repo }
    }

    /// Create a worktree for a feature branch.
    pub fn create_feature_worktree(
        &self,
        feature_name: &str,
        base_branch: &str,
    ) -> GitResult<GitWorktree> {
        let worktrees_dir = self.main_repo.root_path().join("..").join("worktrees");
        std::fs::create_dir_all(&worktrees_dir)?;

        let wt_path = worktrees_dir.join(feature_name);
        let branch_name = format!("feature/{}", feature_name);

        self.main_repo.add_worktree(
            &wt_path,
            base_branch,
            WorktreeAddOptions::with_branch(&branch_name),
        )
    }

    /// Create a worktree for a bugfix.
    pub fn create_bugfix_worktree(
        &self,
        issue_id: &str,
        base_branch: &str,
    ) -> GitResult<GitWorktree> {
        let worktrees_dir = self.main_repo.root_path().join("..").join("worktrees");
        std::fs::create_dir_all(&worktrees_dir)?;

        let wt_path = worktrees_dir.join(format!("bugfix-{}", issue_id));
        let branch_name = format!("bugfix/{}", issue_id);

        self.main_repo.add_worktree(
            &wt_path,
            base_branch,
            WorktreeAddOptions::with_branch(&branch_name),
        )
    }

    /// Get all feature worktrees.
    pub fn feature_worktrees(&self) -> GitResult<Vec<GitWorktree>> {
        let worktrees = self.main_repo.list_worktrees()?;
        Ok(worktrees.into_iter()
            .filter(|wt| wt.branch.as_ref()
                .map(|b| b.starts_with("feature/"))
                .unwrap_or(false))
            .collect())
    }

    /// Clean up merged worktrees.
    pub fn cleanup_merged(&self, target_branch: &str) -> GitResult<Vec<String>> {
        let mut cleaned = Vec::new();
        let worktrees = self.main_repo.list_worktrees()?;

        for wt in worktrees {
            if wt.is_main {
                continue;
            }

            if let Some(ref branch) = wt.branch {
                // Check if branch is merged into target
                if self.main_repo.is_branch_merged(branch, target_branch)? {
                    self.main_repo.remove_worktree(&wt.name, false)?;
                    cleaned.push(wt.name);
                }
            }
        }

        Ok(cleaned)
    }
}