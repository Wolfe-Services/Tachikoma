# 458 - Git Pull

**Phase:** 21 - Git Integration
**Spec ID:** 458
**Status:** Planned
**Dependencies:** 457-git-push, 459-git-merge
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement Git pull operations, combining fetch and merge/rebase to update the local repository.

---

## Acceptance Criteria

- [x] Pull with merge (default)
- [x] Pull with rebase
- [x] Fetch only
- [x] Handle conflicts
- [x] Progress callbacks

---

## Implementation Details

### 1. Fetch Operations (src/fetch.rs)

```rust
//! Git fetch operations.

use crate::{GitOid, GitRepository, GitResult, GitError};
use crate::push::CredentialProvider;
use git2::{FetchOptions, RemoteCallbacks};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{debug, info};

/// Fetch progress information.
#[derive(Debug, Clone)]
pub struct FetchProgress {
    /// Objects received.
    pub received_objects: u32,
    /// Total objects.
    pub total_objects: u32,
    /// Indexed objects.
    pub indexed_objects: u32,
    /// Bytes received.
    pub received_bytes: u64,
}

/// Fetch result.
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Updated refs.
    pub updated_refs: Vec<UpdatedRef>,
    /// Statistics.
    pub stats: FetchProgress,
}

/// An updated reference.
#[derive(Debug, Clone)]
pub struct UpdatedRef {
    /// Reference name.
    pub name: String,
    /// Old OID (None if new).
    pub old_oid: Option<GitOid>,
    /// New OID.
    pub new_oid: GitOid,
}

impl GitRepository {
    /// Fetch from a remote.
    pub fn fetch(&self, remote: Option<&str>) -> GitResult<FetchResult> {
        self.fetch_with_credentials(remote, None)
    }

    /// Fetch with credential callback.
    pub fn fetch_with_credentials(
        &self,
        remote: Option<&str>,
        credentials: Option<Arc<dyn CredentialProvider>>,
    ) -> GitResult<FetchResult> {
        self.with_repo_mut(|repo| {
            let remote_name = remote.unwrap_or("origin");
            let mut remote = repo.find_remote(remote_name)?;

            // Setup callbacks
            let mut callbacks = RemoteCallbacks::new();

            // Progress callback
            let progress = Arc::new(Mutex::new(FetchProgress {
                received_objects: 0,
                total_objects: 0,
                indexed_objects: 0,
                received_bytes: 0,
            }));

            let progress_clone = progress.clone();
            callbacks.transfer_progress(move |stats| {
                let mut p = progress_clone.lock();
                p.received_objects = stats.received_objects() as u32;
                p.total_objects = stats.total_objects() as u32;
                p.indexed_objects = stats.indexed_objects() as u32;
                p.received_bytes = stats.received_bytes() as u64;
                debug!(
                    "Fetch progress: {}/{} objects, {} bytes",
                    stats.received_objects(),
                    stats.total_objects(),
                    stats.received_bytes()
                );
                true
            });

            // Track updated refs
            let updated = Arc::new(Mutex::new(Vec::new()));

            let updated_clone = updated.clone();
            callbacks.update_tips(move |refname, old_oid, new_oid| {
                updated_clone.lock().push(UpdatedRef {
                    name: refname.to_string(),
                    old_oid: if old_oid.is_zero() {
                        None
                    } else {
                        Some(GitOid::from_git2(old_oid))
                    },
                    new_oid: GitOid::from_git2(new_oid),
                });
                true
            });

            // Credentials callback
            if let Some(provider) = credentials {
                callbacks.credentials(move |url, username_from_url, allowed_types| {
                    provider.get_credentials(url, username_from_url, allowed_types)
                });
            }

            // Setup fetch options
            let mut fetch_opts = FetchOptions::new();
            fetch_opts.remote_callbacks(callbacks);

            // Fetch all branches
            let refspecs: Vec<String> = remote.fetch_refspecs()?
                .iter()
                .filter_map(|r| r.map(String::from))
                .collect();
            let refspec_strs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();

            remote.fetch(&refspec_strs, Some(&mut fetch_opts), None)?;

            // Collect results
            let stats = Arc::try_unwrap(progress).unwrap().into_inner();
            let updated_refs = Arc::try_unwrap(updated).unwrap().into_inner();

            Ok(FetchResult {
                updated_refs,
                stats,
            })
        })
    }

    /// Fetch all remotes.
    pub fn fetch_all(&self) -> GitResult<Vec<(String, FetchResult)>> {
        let remotes = self.list_remotes()?;
        let mut results = Vec::new();

        for remote in remotes {
            match self.fetch(Some(&remote)) {
                Ok(result) => results.push((remote, result)),
                Err(e) => {
                    debug!("Failed to fetch {}: {}", remote, e);
                }
            }
        }

        Ok(results)
    }
}
```

### 2. Pull Operations (src/pull.rs)

```rust
//! Git pull operations.

use crate::{GitOid, GitRepository, GitResult, GitError};
use crate::fetch::FetchResult;
use crate::merge::MergeResult;
use crate::push::CredentialProvider;
use std::sync::Arc;

/// Pull strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullStrategy {
    /// Merge (default).
    Merge,
    /// Rebase.
    Rebase,
    /// Fast-forward only.
    FastForwardOnly,
}

impl Default for PullStrategy {
    fn default() -> Self {
        Self::Merge
    }
}

/// Pull options.
#[derive(Debug, Clone, Default)]
pub struct PullOpts {
    /// Pull strategy.
    pub strategy: PullStrategy,
    /// Remote name.
    pub remote: Option<String>,
    /// Branch name.
    pub branch: Option<String>,
    /// Autostash before pull.
    pub autostash: bool,
}

impl PullOpts {
    /// Create with merge strategy.
    pub fn merge() -> Self {
        Self {
            strategy: PullStrategy::Merge,
            ..Default::default()
        }
    }

    /// Create with rebase strategy.
    pub fn rebase() -> Self {
        Self {
            strategy: PullStrategy::Rebase,
            ..Default::default()
        }
    }

    /// Create with fast-forward only.
    pub fn ff_only() -> Self {
        Self {
            strategy: PullStrategy::FastForwardOnly,
            ..Default::default()
        }
    }

    /// Set remote name.
    pub fn from_remote(mut self, name: impl Into<String>) -> Self {
        self.remote = Some(name.into());
        self
    }

    /// Set branch name.
    pub fn branch(mut self, name: impl Into<String>) -> Self {
        self.branch = Some(name.into());
        self
    }

    /// Enable autostash.
    pub fn autostash(mut self) -> Self {
        self.autostash = true;
        self
    }
}

/// Pull result.
#[derive(Debug)]
pub struct PullResult {
    /// Fetch result.
    pub fetch: FetchResult,
    /// Merge/rebase result.
    pub integration: IntegrationResult,
    /// Was stash applied.
    pub stash_applied: bool,
}

/// Integration result (merge or rebase).
#[derive(Debug)]
pub enum IntegrationResult {
    /// Already up to date.
    UpToDate,
    /// Fast-forward was performed.
    FastForward { new_head: GitOid },
    /// Merge was performed.
    Merged(MergeResult),
    /// Rebase was performed.
    Rebased { commits_replayed: u32 },
    /// Has conflicts.
    Conflict { files: Vec<String> },
}

impl GitRepository {
    /// Pull from remote.
    pub fn pull(&self, options: PullOpts) -> GitResult<PullResult> {
        self.pull_with_credentials(options, None)
    }

    /// Pull with credential callback.
    pub fn pull_with_credentials(
        &self,
        options: PullOpts,
        credentials: Option<Arc<dyn CredentialProvider>>,
    ) -> GitResult<PullResult> {
        // Stash if needed
        let stashed = if options.autostash && !self.is_clean()? {
            self.stash_save("autostash before pull")?;
            true
        } else {
            false
        };

        // Fetch
        let remote_name = options.remote.as_deref().unwrap_or("origin");
        let fetch_result = self.fetch_with_credentials(Some(remote_name), credentials)?;

        // Determine upstream ref
        let upstream_ref = if let Some(branch) = &options.branch {
            format!("{}/{}", remote_name, branch)
        } else {
            self.with_repo(|repo| {
                let head = repo.head()?;
                let branch_name = head.shorthand().ok_or_else(|| GitError::InvalidOperation {
                    message: "Cannot pull in detached HEAD state without explicit branch".to_string(),
                })?;

                let branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
                let upstream = branch.upstream()?;
                let upstream_name = upstream.name()?.ok_or_else(|| GitError::RefNotFound {
                    name: "upstream".to_string(),
                })?;

                Ok(upstream_name.to_string())
            })?
        };

        // Get upstream commit
        let upstream_oid = self.with_repo(|repo| {
            let reference = repo.find_reference(&format!("refs/remotes/{}", upstream_ref))?;
            let oid = reference.target().ok_or_else(|| GitError::RefNotFound {
                name: upstream_ref.clone(),
            })?;
            Ok(GitOid::from_git2(oid))
        })?;

        // Perform integration
        let integration = match options.strategy {
            PullStrategy::Merge => {
                self.integrate_merge(&upstream_oid)?
            }
            PullStrategy::Rebase => {
                self.integrate_rebase(&upstream_oid)?
            }
            PullStrategy::FastForwardOnly => {
                self.integrate_ff_only(&upstream_oid)?
            }
        };

        // Pop stash if we stashed
        let stash_applied = if stashed {
            match self.stash_pop() {
                Ok(()) => true,
                Err(e) => {
                    debug!("Failed to pop stash: {}", e);
                    false
                }
            }
        } else {
            false
        };

        Ok(PullResult {
            fetch: fetch_result,
            integration,
            stash_applied,
        })
    }

    fn integrate_merge(&self, upstream: &GitOid) -> GitResult<IntegrationResult> {
        let head = self.head_oid()?;

        // Check if already up to date
        let merge_base = self.merge_base(&head, upstream)?;
        if merge_base == *upstream {
            return Ok(IntegrationResult::UpToDate);
        }

        // Check if fast-forward is possible
        if merge_base == head {
            // Fast-forward
            self.checkout_commit(upstream)?;
            return Ok(IntegrationResult::FastForward { new_head: *upstream });
        }

        // Perform merge
        let result = self.merge(upstream, None)?;
        Ok(IntegrationResult::Merged(result))
    }

    fn integrate_rebase(&self, upstream: &GitOid) -> GitResult<IntegrationResult> {
        // Check if already up to date
        let head = self.head_oid()?;
        let merge_base = self.merge_base(&head, upstream)?;
        if merge_base == *upstream {
            return Ok(IntegrationResult::UpToDate);
        }

        // Perform rebase
        // Note: Full rebase implementation would be more complex
        let commits = self.commits_range(Some(&merge_base), &head, None)?;
        let count = commits.len();

        self.with_repo_mut(|repo| {
            // Reset to upstream
            let upstream_commit = repo.find_commit(upstream.as_git2())?;
            repo.reset(upstream_commit.as_object(), git2::ResetType::Hard, None)?;

            // Cherry-pick each commit
            for commit in &commits {
                let original = repo.find_commit(commit.oid.as_git2())?;
                repo.cherrypick(&original, None)?;

                // Check for conflicts
                let index = repo.index()?;
                if index.has_conflicts() {
                    let files: Vec<String> = index.conflicts()?
                        .filter_map(|c| c.ok())
                        .filter_map(|c| c.our.or(c.their))
                        .filter_map(|e| String::from_utf8(e.path).ok())
                        .collect();

                    return Err(GitError::MergeConflict { files });
                }

                // Commit
                let tree_oid = repo.index()?.write_tree()?;
                let tree = repo.find_tree(tree_oid)?;
                let sig = repo.signature()?;
                let parent = repo.head()?.peel_to_commit()?;

                repo.commit(
                    Some("HEAD"),
                    &original.author(),
                    &sig,
                    original.message().unwrap_or(""),
                    &tree,
                    &[&parent],
                )?;

                repo.cleanup_state()?;
            }

            Ok(())
        })?;

        Ok(IntegrationResult::Rebased { commits_replayed: count as u32 })
    }

    fn integrate_ff_only(&self, upstream: &GitOid) -> GitResult<IntegrationResult> {
        let head = self.head_oid()?;
        let merge_base = self.merge_base(&head, upstream)?;

        if merge_base == *upstream {
            return Ok(IntegrationResult::UpToDate);
        }

        if merge_base != head {
            return Err(GitError::InvalidOperation {
                message: "Fast-forward not possible, would create a merge".to_string(),
            });
        }

        self.checkout_commit(upstream)?;
        Ok(IntegrationResult::FastForward { new_head: *upstream })
    }
}

use tracing::debug;
```

---

## Testing Requirements

1. Fetch updates remote tracking branches
2. Merge pull creates merge commit
3. Rebase pull replays commits
4. FF-only fails when not possible
5. Autostash works correctly

---

## Related Specs

- Depends on: [457-git-push.md](457-git-push.md), [459-git-merge.md](459-git-merge.md)
- Next: [459-git-merge.md](459-git-merge.md)
