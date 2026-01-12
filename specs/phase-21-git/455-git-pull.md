# Spec 455: Pull and Fetch Operations

## Phase
21 - Git Integration

## Spec ID
455

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)
- Spec 453: Remote Operations (remote management)
- Spec 456: Merge Operations (for pull merge)

## Estimated Context
~10%

---

## Objective

Implement Git fetch and pull operations for Tachikoma with progress reporting and multiple merge strategies. This module handles retrieving changes from remote repositories and integrating them into the local branch with proper conflict handling.

---

## Acceptance Criteria

- [ ] Implement `GitFetcher` for fetch operations
- [ ] Implement `GitPuller` for pull operations
- [ ] Support progress reporting during fetch/pull
- [ ] Support multiple merge strategies (merge, rebase, fast-forward only)
- [ ] Handle authentication for remote access
- [ ] Support fetching specific refs
- [ ] Implement fetch prune
- [ ] Support fetch tags options
- [ ] Handle pull conflicts properly
- [ ] Support dry-run for pull preview

---

## Implementation Details

### Fetch and Pull Implementation

```rust
// src/git/fetch.rs

use git2::{AnnotatedCommit, AutotagOption, Cred, FetchOptions, RemoteCallbacks};
use std::sync::Arc;

use super::repo::GitRepository;
use super::types::*;

/// Fetch progress information
#[derive(Debug, Clone)]
pub struct FetchProgress {
    /// Current stage
    pub stage: FetchStage,
    /// Total objects to download
    pub total_objects: usize,
    /// Objects received
    pub received_objects: usize,
    /// Objects indexed
    pub indexed_objects: usize,
    /// Bytes received
    pub received_bytes: usize,
    /// Local objects
    pub local_objects: usize,
    /// Total deltas
    pub total_deltas: usize,
    /// Indexed deltas
    pub indexed_deltas: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchStage {
    Connecting,
    Counting,
    Compressing,
    Receiving,
    Resolving,
    Done,
}

/// Fetch result
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Remote name
    pub remote: String,
    /// Updated references
    pub updated_refs: Vec<FetchedRef>,
    /// New tags
    pub new_tags: Vec<String>,
    /// Pruned refs
    pub pruned_refs: Vec<String>,
    /// Total bytes received
    pub bytes_received: usize,
}

#[derive(Debug, Clone)]
pub struct FetchedRef {
    pub name: String,
    pub old_oid: Option<GitOid>,
    pub new_oid: GitOid,
    pub is_new: bool,
}

/// Options for fetch operations
#[derive(Debug, Clone)]
pub struct FetchOperationOptions {
    /// Remote name (default: "origin")
    pub remote: String,
    /// Prune deleted refs
    pub prune: bool,
    /// Fetch tags
    pub tags: TagFetchOption,
    /// Depth for shallow fetch (None for full)
    pub depth: Option<i32>,
    /// Specific refspecs to fetch
    pub refspecs: Vec<String>,
    /// Force fetch
    pub force: bool,
    /// Fetch all remotes
    pub all: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TagFetchOption {
    #[default]
    Auto,
    All,
    None,
}

impl Default for FetchOperationOptions {
    fn default() -> Self {
        Self {
            remote: "origin".to_string(),
            prune: false,
            tags: TagFetchOption::Auto,
            depth: None,
            refspecs: Vec::new(),
            force: false,
            all: false,
        }
    }
}

impl FetchOperationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = remote.into();
        self
    }

    pub fn prune(mut self) -> Self {
        self.prune = true;
        self
    }

    pub fn tags(mut self, tags: TagFetchOption) -> Self {
        self.tags = tags;
        self
    }

    pub fn depth(mut self, depth: i32) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn refspec(mut self, refspec: impl Into<String>) -> Self {
        self.refspecs.push(refspec.into());
        self
    }

    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }
}

/// Git fetch manager
pub struct GitFetcher<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitFetcher<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Fetch from default remote
    pub fn fetch(&self, options: FetchOperationOptions) -> GitResult<FetchResult> {
        self.fetch_with_progress(options, None)
    }

    /// Fetch with progress callback
    pub fn fetch_with_progress(
        &self,
        options: FetchOperationOptions,
        progress_callback: Option<Box<dyn Fn(FetchProgress) + Send>>,
    ) -> GitResult<FetchResult> {
        let raw_repo = self.repo.raw();

        if options.all {
            return self.fetch_all_remotes(&options, progress_callback);
        }

        let mut remote = raw_repo.find_remote(&options.remote)?;

        // Build callbacks
        let mut callbacks = RemoteCallbacks::new();

        // Credential callback
        callbacks.credentials(|_url, username, _allowed| {
            if let Some(username) = username {
                return Cred::ssh_key_from_agent(username);
            }
            Cred::default()
        });

        // Progress callback
        let bytes_received = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let bytes_clone = bytes_received.clone();

        if let Some(callback) = progress_callback {
            let callback = Arc::new(callback);

            callbacks.transfer_progress(move |progress| {
                bytes_clone.store(
                    progress.received_bytes(),
                    std::sync::atomic::Ordering::Relaxed,
                );

                callback(FetchProgress {
                    stage: if progress.received_objects() < progress.total_objects() {
                        FetchStage::Receiving
                    } else {
                        FetchStage::Resolving
                    },
                    total_objects: progress.total_objects(),
                    received_objects: progress.received_objects(),
                    indexed_objects: progress.indexed_objects(),
                    received_bytes: progress.received_bytes(),
                    local_objects: progress.local_objects(),
                    total_deltas: progress.total_deltas(),
                    indexed_deltas: progress.indexed_deltas(),
                });
                true
            });
        }

        // Build fetch options
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        if options.prune {
            fetch_opts.prune(git2::FetchPrune::On);
        }

        match options.tags {
            TagFetchOption::Auto => fetch_opts.download_tags(AutotagOption::Auto),
            TagFetchOption::All => fetch_opts.download_tags(AutotagOption::All),
            TagFetchOption::None => fetch_opts.download_tags(AutotagOption::None),
        };

        // Build refspecs
        let refspecs: Vec<String> = if options.refspecs.is_empty() {
            remote
                .fetch_refspecs()?
                .iter()
                .filter_map(|s| s.map(String::from))
                .collect()
        } else {
            options.refspecs
        };

        let refspec_strs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();

        // Perform fetch
        remote.fetch(&refspec_strs, Some(&mut fetch_opts), None)?;

        // Collect updated refs
        let updated_refs = self.get_updated_refs(&options.remote)?;

        Ok(FetchResult {
            remote: options.remote,
            updated_refs,
            new_tags: Vec::new(), // Would need to track during fetch
            pruned_refs: Vec::new(),
            bytes_received: bytes_received.load(std::sync::atomic::Ordering::Relaxed),
        })
    }

    /// Fetch from all configured remotes
    fn fetch_all_remotes(
        &self,
        options: &FetchOperationOptions,
        progress_callback: Option<Box<dyn Fn(FetchProgress) + Send>>,
    ) -> GitResult<FetchResult> {
        let raw_repo = self.repo.raw();
        let remotes = raw_repo.remotes()?;

        let mut all_updated_refs = Vec::new();
        let mut total_bytes = 0;

        for remote_name in remotes.iter().flatten() {
            let mut opts = options.clone();
            opts.remote = remote_name.to_string();
            opts.all = false;

            // For now, only first remote gets progress callback
            let result = if remote_name == remotes.iter().next().flatten().unwrap_or("") {
                self.fetch_with_progress(opts, progress_callback.take().map(|c| c))?
            } else {
                self.fetch_with_progress(opts, None)?
            };

            all_updated_refs.extend(result.updated_refs);
            total_bytes += result.bytes_received;
        }

        Ok(FetchResult {
            remote: "all".to_string(),
            updated_refs: all_updated_refs,
            new_tags: Vec::new(),
            pruned_refs: Vec::new(),
            bytes_received: total_bytes,
        })
    }

    fn get_updated_refs(&self, remote: &str) -> GitResult<Vec<FetchedRef>> {
        // This would need to track refs before/after fetch for accurate results
        // For now, return remote tracking refs
        let raw_repo = self.repo.raw();
        let mut refs = Vec::new();

        let ref_prefix = format!("refs/remotes/{}/", remote);

        for reference in raw_repo.references()? {
            let reference = reference?;
            if let Some(name) = reference.name() {
                if name.starts_with(&ref_prefix) {
                    if let Some(oid) = reference.target() {
                        refs.push(FetchedRef {
                            name: name.to_string(),
                            old_oid: None, // Would need tracking
                            new_oid: GitOid::from(oid),
                            is_new: false,
                        });
                    }
                }
            }
        }

        Ok(refs)
    }
}

// src/git/pull.rs

use super::fetch::{FetchOperationOptions, FetchProgress, GitFetcher};
use super::merge::GitMerger;
use super::rebase::GitRebaser;
use super::repo::GitRepository;
use super::types::*;

/// Pull strategy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PullStrategy {
    #[default]
    Merge,
    Rebase,
    FastForwardOnly,
}

/// Options for pull operations
#[derive(Debug, Clone)]
pub struct PullOperationOptions {
    /// Remote name
    pub remote: String,
    /// Remote branch (defaults to upstream)
    pub branch: Option<String>,
    /// Pull strategy
    pub strategy: PullStrategy,
    /// Allow unrelated histories (for merge)
    pub allow_unrelated: bool,
    /// Autostash before operation
    pub autostash: bool,
    /// Fetch options
    pub fetch_options: FetchOperationOptions,
}

impl Default for PullOperationOptions {
    fn default() -> Self {
        Self {
            remote: "origin".to_string(),
            branch: None,
            strategy: PullStrategy::Merge,
            allow_unrelated: false,
            autostash: false,
            fetch_options: FetchOperationOptions::default(),
        }
    }
}

impl PullOperationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = remote.into();
        self.fetch_options.remote = self.remote.clone();
        self
    }

    pub fn branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    pub fn strategy(mut self, strategy: PullStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn rebase(self) -> Self {
        self.strategy(PullStrategy::Rebase)
    }

    pub fn ff_only(self) -> Self {
        self.strategy(PullStrategy::FastForwardOnly)
    }

    pub fn autostash(mut self) -> Self {
        self.autostash = true;
        self
    }
}

/// Pull result
#[derive(Debug, Clone)]
pub struct PullResult {
    /// Was the pull successful
    pub success: bool,
    /// Strategy used
    pub strategy: PullStrategy,
    /// New HEAD after pull
    pub new_head: Option<GitOid>,
    /// Number of commits pulled
    pub commits_pulled: usize,
    /// Files updated
    pub files_updated: usize,
    /// Conflicts (if merge conflicts)
    pub conflicts: Vec<String>,
    /// Was fast-forward
    pub was_fast_forward: bool,
}

/// Git pull manager
pub struct GitPuller<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitPuller<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Pull from remote
    pub fn pull(&self, options: PullOperationOptions) -> GitResult<PullResult> {
        self.pull_with_progress(options, None)
    }

    /// Pull with progress reporting
    pub fn pull_with_progress(
        &self,
        options: PullOperationOptions,
        progress_callback: Option<Box<dyn Fn(FetchProgress) + Send>>,
    ) -> GitResult<PullResult> {
        let raw_repo = self.repo.raw();

        // Get current branch
        let current_branch = self.repo.current_branch()?
            .ok_or_else(|| GitError::Other("Not on a branch".into()))?;

        // Determine remote branch
        let remote_branch = options.branch.clone().unwrap_or_else(|| current_branch.clone());

        // Fetch first
        let fetcher = GitFetcher::new(self.repo);
        let mut fetch_opts = options.fetch_options.clone();
        fetch_opts.refspec(format!("refs/heads/{}:refs/remotes/{}/{}",
            remote_branch, options.remote, remote_branch));

        fetcher.fetch_with_progress(fetch_opts, progress_callback)?;

        // Get the fetch head
        let fetch_head = self.get_fetch_head(&options.remote, &remote_branch)?;

        // Check merge status
        let (analysis, _preference) = raw_repo.merge_analysis(&[&fetch_head])?;

        // Handle based on analysis and strategy
        if analysis.is_up_to_date() {
            return Ok(PullResult {
                success: true,
                strategy: options.strategy,
                new_head: self.repo.head()?.target,
                commits_pulled: 0,
                files_updated: 0,
                conflicts: Vec::new(),
                was_fast_forward: false,
            });
        }

        match options.strategy {
            PullStrategy::FastForwardOnly => {
                if !analysis.is_fast_forward() {
                    return Err(GitError::Other(
                        "Cannot fast-forward, local branch has diverged".into()
                    ));
                }
                self.fast_forward(&fetch_head)
            }
            PullStrategy::Merge => {
                if analysis.is_fast_forward() {
                    self.fast_forward(&fetch_head)
                } else {
                    self.merge(&fetch_head, &options)
                }
            }
            PullStrategy::Rebase => {
                self.rebase_onto(&fetch_head)
            }
        }
    }

    fn get_fetch_head(
        &self,
        remote: &str,
        branch: &str,
    ) -> GitResult<AnnotatedCommit<'a>> {
        let raw_repo = self.repo.raw();
        let ref_name = format!("refs/remotes/{}/{}", remote, branch);
        let reference = raw_repo.find_reference(&ref_name)?;
        let annotated = raw_repo.reference_to_annotated_commit(&reference)?;

        // Safety: This is safe because we're within the same repo lifetime
        Ok(unsafe { std::mem::transmute(annotated) })
    }

    fn fast_forward(&self, target: &AnnotatedCommit) -> GitResult<PullResult> {
        let raw_repo = self.repo.raw();

        // Get reference to update
        let mut head_ref = raw_repo.head()?;
        let refname = head_ref.name()
            .ok_or_else(|| GitError::Other("Cannot get HEAD name".into()))?
            .to_string();

        // Fast-forward
        head_ref.set_target(target.id(), "pull: Fast-forward")?;

        // Checkout
        raw_repo.checkout_head(Some(
            git2::build::CheckoutBuilder::new().force()
        ))?;

        Ok(PullResult {
            success: true,
            strategy: PullStrategy::FastForwardOnly,
            new_head: Some(GitOid::from(target.id())),
            commits_pulled: 1, // Would need to calculate
            files_updated: 0,
            conflicts: Vec::new(),
            was_fast_forward: true,
        })
    }

    fn merge(&self, target: &AnnotatedCommit, options: &PullOperationOptions) -> GitResult<PullResult> {
        let raw_repo = self.repo.raw();

        // Perform merge
        raw_repo.merge(&[target], None, None)?;

        // Check for conflicts
        let index = raw_repo.index()?;
        if index.has_conflicts() {
            let conflicts: Vec<String> = index
                .conflicts()?
                .filter_map(|c| c.ok())
                .filter_map(|c| {
                    c.our.or(c.their).or(c.ancestor)
                        .and_then(|e| String::from_utf8(e.path.clone()).ok())
                })
                .collect();

            return Ok(PullResult {
                success: false,
                strategy: PullStrategy::Merge,
                new_head: None,
                commits_pulled: 0,
                files_updated: 0,
                conflicts,
                was_fast_forward: false,
            });
        }

        // Create merge commit
        let sig = raw_repo.signature()?;
        let head = raw_repo.head()?.peel_to_commit()?;
        let remote_commit = raw_repo.find_commit(target.id())?;

        let mut index = raw_repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = raw_repo.find_tree(tree_id)?;

        let message = format!(
            "Merge branch '{}' of {} into {}",
            target.refname().unwrap_or("unknown"),
            options.remote,
            self.repo.current_branch()?.unwrap_or_default()
        );

        let commit_oid = raw_repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &message,
            &tree,
            &[&head, &remote_commit],
        )?;

        // Cleanup merge state
        raw_repo.cleanup_state()?;

        Ok(PullResult {
            success: true,
            strategy: PullStrategy::Merge,
            new_head: Some(GitOid::from(commit_oid)),
            commits_pulled: 1,
            files_updated: 0,
            conflicts: Vec::new(),
            was_fast_forward: false,
        })
    }

    fn rebase_onto(&self, target: &AnnotatedCommit) -> GitResult<PullResult> {
        // Rebase implementation would delegate to GitRebaser
        // For now, return a placeholder
        Err(GitError::Other("Rebase pull not yet implemented".into()))
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

        let mut config = repo.config().unwrap();
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

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
    fn test_fetch_options_builder() {
        let opts = FetchOperationOptions::new()
            .remote("upstream")
            .prune()
            .tags(TagFetchOption::All)
            .depth(1);

        assert_eq!(opts.remote, "upstream");
        assert!(opts.prune);
        assert!(matches!(opts.tags, TagFetchOption::All));
        assert_eq!(opts.depth, Some(1));
    }

    #[test]
    fn test_pull_options_builder() {
        let opts = PullOperationOptions::new()
            .remote("origin")
            .branch("main")
            .rebase()
            .autostash();

        assert_eq!(opts.remote, "origin");
        assert_eq!(opts.branch, Some("main".to_string()));
        assert_eq!(opts.strategy, PullStrategy::Rebase);
        assert!(opts.autostash);
    }

    #[test]
    fn test_pull_strategy_enum() {
        assert_eq!(PullStrategy::default(), PullStrategy::Merge);
        assert_ne!(PullStrategy::Rebase, PullStrategy::Merge);
    }

    #[test]
    fn test_fetch_progress_struct() {
        let progress = FetchProgress {
            stage: FetchStage::Receiving,
            total_objects: 100,
            received_objects: 50,
            indexed_objects: 45,
            received_bytes: 4096,
            local_objects: 10,
            total_deltas: 20,
            indexed_deltas: 15,
        };

        assert_eq!(progress.stage, FetchStage::Receiving);
        assert_eq!(progress.total_objects, 100);
    }

    #[test]
    fn test_fetch_result() {
        let result = FetchResult {
            remote: "origin".to_string(),
            updated_refs: vec![
                FetchedRef {
                    name: "refs/remotes/origin/main".to_string(),
                    old_oid: None,
                    new_oid: GitOid([1; 20]),
                    is_new: true,
                }
            ],
            new_tags: vec!["v1.0".to_string()],
            pruned_refs: Vec::new(),
            bytes_received: 1024,
        };

        assert_eq!(result.updated_refs.len(), 1);
        assert!(result.updated_refs[0].is_new);
    }

    #[test]
    fn test_pull_result_with_conflicts() {
        let result = PullResult {
            success: false,
            strategy: PullStrategy::Merge,
            new_head: None,
            commits_pulled: 0,
            files_updated: 0,
            conflicts: vec!["file.txt".to_string(), "other.txt".to_string()],
            was_fast_forward: false,
        };

        assert!(!result.success);
        assert_eq!(result.conflicts.len(), 2);
    }

    #[test]
    fn test_ff_only_strategy() {
        let opts = PullOperationOptions::new().ff_only();
        assert_eq!(opts.strategy, PullStrategy::FastForwardOnly);
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 453: Remote Operations
- Spec 454: Push Operations
- Spec 456: Merge Operations
- Spec 457: Rebase Support
