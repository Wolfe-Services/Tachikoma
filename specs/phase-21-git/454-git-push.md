# Spec 454: Push Operations

## Phase
21 - Git Integration

## Spec ID
454

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)
- Spec 453: Remote Operations (remote management)
- Spec 463: Credential Management (authentication)

## Estimated Context
~10%

---

## Objective

Implement Git push operations for Tachikoma with progress reporting and comprehensive options support. This module handles pushing commits, tags, and branches to remote repositories with proper authentication and error handling.

---

## Acceptance Criteria

- [ ] Implement `GitPusher` for push operations
- [ ] Support pushing branches with progress reporting
- [ ] Support pushing tags
- [ ] Implement force push (with and without lease)
- [ ] Support pushing to specific remotes
- [ ] Implement refspec-based pushing
- [ ] Handle authentication (SSH, HTTPS)
- [ ] Support dry-run mode
- [ ] Provide detailed push results
- [ ] Handle push rejection scenarios

---

## Implementation Details

### Push Manager Implementation

```rust
// src/git/push.rs

use git2::{Cred, Direction, PushOptions, RemoteCallbacks};
use std::sync::{Arc, Mutex};

use super::repo::GitRepository;
use super::types::*;

/// Push progress information
#[derive(Debug, Clone)]
pub struct PushProgress {
    /// Current stage
    pub stage: PushStage,
    /// Objects written
    pub objects_written: usize,
    /// Total objects to write
    pub total_objects: usize,
    /// Bytes written
    pub bytes_written: usize,
    /// Current reference being pushed
    pub current_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushStage {
    Preparing,
    Counting,
    Compressing,
    Writing,
    Updating,
    Done,
}

/// Push result for a single reference
#[derive(Debug, Clone)]
pub struct PushRefResult {
    /// The reference name
    pub refname: String,
    /// Whether push succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Was this an update or create
    pub update_type: PushUpdateType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushUpdateType {
    Create,
    Update,
    Delete,
    ForceUpdate,
    Rejected,
}

/// Complete push operation result
#[derive(Debug, Clone)]
pub struct PushResult {
    /// Remote name
    pub remote: String,
    /// Results per reference
    pub ref_results: Vec<PushRefResult>,
    /// Overall success
    pub success: bool,
    /// Total objects pushed
    pub objects_pushed: usize,
    /// Total bytes transferred
    pub bytes_transferred: usize,
}

impl PushResult {
    pub fn failed_refs(&self) -> Vec<&PushRefResult> {
        self.ref_results.iter().filter(|r| !r.success).collect()
    }

    pub fn successful_refs(&self) -> Vec<&PushRefResult> {
        self.ref_results.iter().filter(|r| r.success).collect()
    }
}

/// Options for push operations
#[derive(Debug, Clone)]
pub struct PushOperationOptions {
    /// Remote name (default: "origin")
    pub remote: String,
    /// Force push
    pub force: bool,
    /// Force with lease (safer force push)
    pub force_with_lease: bool,
    /// Expected OID for force-with-lease
    pub lease_oid: Option<GitOid>,
    /// Dry run (don't actually push)
    pub dry_run: bool,
    /// Push all branches
    pub all: bool,
    /// Push tags
    pub tags: bool,
    /// Set upstream
    pub set_upstream: bool,
    /// Delete remote ref
    pub delete: bool,
    /// Atomic push (all or nothing)
    pub atomic: bool,
    /// Specific refspecs to push
    pub refspecs: Vec<String>,
}

impl Default for PushOperationOptions {
    fn default() -> Self {
        Self {
            remote: "origin".to_string(),
            force: false,
            force_with_lease: false,
            lease_oid: None,
            dry_run: false,
            all: false,
            tags: false,
            set_upstream: false,
            delete: false,
            atomic: false,
            refspecs: Vec::new(),
        }
    }
}

impl PushOperationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = remote.into();
        self
    }

    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn force_with_lease(mut self, expected: Option<GitOid>) -> Self {
        self.force_with_lease = true;
        self.lease_oid = expected;
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    pub fn tags(mut self) -> Self {
        self.tags = true;
        self
    }

    pub fn set_upstream(mut self) -> Self {
        self.set_upstream = true;
        self
    }

    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    pub fn refspec(mut self, refspec: impl Into<String>) -> Self {
        self.refspecs.push(refspec.into());
        self
    }
}

/// Credential provider trait
pub trait CredentialProvider: Send + Sync {
    fn provide(&self, url: &str, username: Option<&str>) -> GitResult<git2::Cred>;
}

/// Default credential provider using SSH agent and git credentials
pub struct DefaultCredentialProvider;

impl CredentialProvider for DefaultCredentialProvider {
    fn provide(&self, _url: &str, username: Option<&str>) -> GitResult<git2::Cred> {
        // Try SSH agent first
        if let Some(username) = username {
            if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                return Ok(cred);
            }
        }

        // Fall back to default
        Cred::default().map_err(GitError::Git2)
    }
}

/// Git push manager
pub struct GitPusher<'a> {
    repo: &'a GitRepository,
    credential_provider: Box<dyn CredentialProvider>,
}

impl<'a> GitPusher<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self {
            repo,
            credential_provider: Box::new(DefaultCredentialProvider),
        }
    }

    pub fn with_credential_provider(mut self, provider: impl CredentialProvider + 'static) -> Self {
        self.credential_provider = Box::new(provider);
        self
    }

    /// Push current branch to remote
    pub fn push_current(&self, options: PushOperationOptions) -> GitResult<PushResult> {
        let branch = self.repo.current_branch()?
            .ok_or_else(|| GitError::Other("Not on a branch (detached HEAD)".into()))?;

        self.push_branch(&branch, options)
    }

    /// Push a specific branch
    pub fn push_branch(&self, branch: &str, options: PushOperationOptions) -> GitResult<PushResult> {
        let refspec = if options.delete {
            format!(":refs/heads/{}", branch)
        } else if options.force || options.force_with_lease {
            format!("+refs/heads/{}:refs/heads/{}", branch, branch)
        } else {
            format!("refs/heads/{}:refs/heads/{}", branch, branch)
        };

        let mut opts = options.clone();
        opts.refspecs = vec![refspec];
        self.push(opts)
    }

    /// Push tags
    pub fn push_tags(&self, options: PushOperationOptions) -> GitResult<PushResult> {
        let mut opts = options.clone();
        opts.refspecs = vec!["refs/tags/*:refs/tags/*".to_string()];
        self.push(opts)
    }

    /// Push a specific tag
    pub fn push_tag(&self, tag: &str, options: PushOperationOptions) -> GitResult<PushResult> {
        let mut opts = options.clone();
        opts.refspecs = vec![format!("refs/tags/{}:refs/tags/{}", tag, tag)];
        self.push(opts)
    }

    /// Push with full options
    pub fn push(&self, options: PushOperationOptions) -> GitResult<PushResult> {
        self.push_with_progress(options, None)
    }

    /// Push with progress callback
    pub fn push_with_progress(
        &self,
        options: PushOperationOptions,
        progress_callback: Option<Box<dyn Fn(PushProgress) + Send>>,
    ) -> GitResult<PushResult> {
        let raw_repo = self.repo.raw();
        let mut remote = raw_repo.find_remote(&options.remote)?;

        // Build refspecs
        let refspecs = self.build_refspecs(&options)?;

        if refspecs.is_empty() {
            return Err(GitError::Other("No refspecs to push".into()));
        }

        // Track results
        let ref_results = Arc::new(Mutex::new(Vec::new()));
        let ref_results_clone = ref_results.clone();

        let progress = Arc::new(Mutex::new(PushProgress {
            stage: PushStage::Preparing,
            objects_written: 0,
            total_objects: 0,
            bytes_written: 0,
            current_ref: None,
        }));
        let progress_clone = progress.clone();

        // Set up callbacks
        let mut callbacks = RemoteCallbacks::new();

        // Credential callback
        callbacks.credentials(|url, username, _allowed| {
            // Use SSH key from agent
            if let Some(username) = username {
                return Cred::ssh_key_from_agent(username);
            }
            Cred::default()
        });

        // Progress callback
        if let Some(callback) = progress_callback {
            let callback = Arc::new(callback);
            let progress_for_cb = progress_clone.clone();

            callbacks.push_transfer_progress(move |current, total, bytes| {
                let mut p = progress_for_cb.lock().unwrap();
                p.stage = PushStage::Writing;
                p.objects_written = current;
                p.total_objects = total;
                p.bytes_written = bytes;
                callback(p.clone());
            });
        }

        // Update status callback
        callbacks.push_update_reference(move |refname, status| {
            let mut results = ref_results_clone.lock().unwrap();
            results.push(PushRefResult {
                refname: refname.to_string(),
                success: status.is_none(),
                error: status.map(String::from),
                update_type: PushUpdateType::Update, // Would need more context for accurate type
            });
            Ok(())
        });

        // Perform push
        let mut push_opts = PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        // Convert refspecs to &str slice
        let refspec_strs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();

        if !options.dry_run {
            remote.push(&refspec_strs, Some(&mut push_opts))?;
        }

        let results = ref_results.lock().unwrap().clone();
        let p = progress.lock().unwrap();

        let success = results.iter().all(|r| r.success);

        // Set upstream if requested
        if options.set_upstream && success {
            if let Some(branch) = self.repo.current_branch()? {
                if let Err(e) = self.set_upstream_tracking(&branch, &options.remote) {
                    // Log but don't fail
                    eprintln!("Warning: Could not set upstream: {}", e);
                }
            }
        }

        Ok(PushResult {
            remote: options.remote,
            ref_results: results,
            success,
            objects_pushed: p.objects_written,
            bytes_transferred: p.bytes_written,
        })
    }

    /// Delete a remote branch
    pub fn delete_remote_branch(&self, branch: &str, remote: &str) -> GitResult<PushResult> {
        self.push(
            PushOperationOptions::new()
                .remote(remote)
                .delete()
                .refspec(format!(":refs/heads/{}", branch)),
        )
    }

    /// Delete a remote tag
    pub fn delete_remote_tag(&self, tag: &str, remote: &str) -> GitResult<PushResult> {
        self.push(
            PushOperationOptions::new()
                .remote(remote)
                .delete()
                .refspec(format!(":refs/tags/{}", tag)),
        )
    }

    fn build_refspecs(&self, options: &PushOperationOptions) -> GitResult<Vec<String>> {
        let mut refspecs = Vec::new();

        // Explicit refspecs take priority
        if !options.refspecs.is_empty() {
            return Ok(options.refspecs.clone());
        }

        // Push all branches
        if options.all {
            let force_prefix = if options.force { "+" } else { "" };
            refspecs.push(format!("{}refs/heads/*:refs/heads/*", force_prefix));
        }

        // Push tags
        if options.tags {
            refspecs.push("refs/tags/*:refs/tags/*".to_string());
        }

        // Default: push current branch
        if refspecs.is_empty() {
            if let Some(branch) = self.repo.current_branch()? {
                let force_prefix = if options.force || options.force_with_lease { "+" } else { "" };
                refspecs.push(format!(
                    "{}refs/heads/{}:refs/heads/{}",
                    force_prefix, branch, branch
                ));
            } else {
                return Err(GitError::Other("No branch to push and no refspecs specified".into()));
            }
        }

        Ok(refspecs)
    }

    fn set_upstream_tracking(&self, branch: &str, remote: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        let mut local_branch = raw_repo.find_branch(branch, git2::BranchType::Local)?;
        let upstream_name = format!("{}/{}", remote, branch);
        local_branch.set_upstream(Some(&upstream_name))?;
        Ok(())
    }
}

/// Helper for checking if push would be rejected
pub fn would_reject_push(repo: &GitRepository, remote: &str, branch: &str) -> GitResult<bool> {
    let raw_repo = repo.raw();

    // Get local commit
    let local_ref = format!("refs/heads/{}", branch);
    let local_oid = raw_repo.refname_to_id(&local_ref)?;

    // Get remote commit
    let remote_ref = format!("refs/remotes/{}/{}", remote, branch);
    let remote_oid = match raw_repo.refname_to_id(&remote_ref) {
        Ok(oid) => oid,
        Err(_) => return Ok(false), // Remote branch doesn't exist, push will create
    };

    // Check if local is descendant of remote
    let is_descendant = raw_repo.graph_descendant_of(local_oid, remote_oid)?;

    Ok(!is_descendant)
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
    fn test_push_options_builder() {
        let opts = PushOperationOptions::new()
            .remote("upstream")
            .force()
            .set_upstream()
            .refspec("refs/heads/main:refs/heads/main");

        assert_eq!(opts.remote, "upstream");
        assert!(opts.force);
        assert!(opts.set_upstream);
        assert_eq!(opts.refspecs.len(), 1);
    }

    #[test]
    fn test_force_with_lease() {
        let oid = GitOid([1; 20]);
        let opts = PushOperationOptions::new()
            .force_with_lease(Some(oid));

        assert!(opts.force_with_lease);
        assert!(opts.lease_oid.is_some());
    }

    #[test]
    fn test_push_result_filtering() {
        let result = PushResult {
            remote: "origin".to_string(),
            ref_results: vec![
                PushRefResult {
                    refname: "refs/heads/main".to_string(),
                    success: true,
                    error: None,
                    update_type: PushUpdateType::Update,
                },
                PushRefResult {
                    refname: "refs/heads/feature".to_string(),
                    success: false,
                    error: Some("rejected".to_string()),
                    update_type: PushUpdateType::Rejected,
                },
            ],
            success: false,
            objects_pushed: 10,
            bytes_transferred: 1024,
        };

        assert_eq!(result.successful_refs().len(), 1);
        assert_eq!(result.failed_refs().len(), 1);
    }

    #[test]
    fn test_push_stage_enum() {
        assert_ne!(PushStage::Preparing, PushStage::Done);
        assert_eq!(PushStage::Writing, PushStage::Writing);
    }

    #[test]
    fn test_build_refspecs_current_branch() {
        let (_dir, repo) = setup_test_repo();
        let pusher = GitPusher::new(&repo);

        let options = PushOperationOptions::default();
        let refspecs = pusher.build_refspecs(&options).unwrap();

        assert!(!refspecs.is_empty());
        // Should include the current branch
    }

    #[test]
    fn test_build_refspecs_force() {
        let (_dir, repo) = setup_test_repo();
        let pusher = GitPusher::new(&repo);

        let options = PushOperationOptions::new().force();
        let refspecs = pusher.build_refspecs(&options).unwrap();

        assert!(refspecs[0].starts_with('+'));
    }

    #[test]
    fn test_build_refspecs_all() {
        let (_dir, repo) = setup_test_repo();
        let pusher = GitPusher::new(&repo);

        let options = PushOperationOptions::new().all();
        let refspecs = pusher.build_refspecs(&options).unwrap();

        assert!(refspecs[0].contains("refs/heads/*"));
    }

    #[test]
    fn test_build_refspecs_tags() {
        let (_dir, repo) = setup_test_repo();
        let pusher = GitPusher::new(&repo);

        let options = PushOperationOptions::new().tags();
        let refspecs = pusher.build_refspecs(&options).unwrap();

        assert!(refspecs[0].contains("refs/tags/*"));
    }

    #[test]
    fn test_build_refspecs_explicit() {
        let (_dir, repo) = setup_test_repo();
        let pusher = GitPusher::new(&repo);

        let options = PushOperationOptions::new()
            .refspec("refs/heads/main:refs/heads/main")
            .refspec("refs/tags/v1.0:refs/tags/v1.0");

        let refspecs = pusher.build_refspecs(&options).unwrap();

        assert_eq!(refspecs.len(), 2);
    }

    #[test]
    fn test_push_progress_struct() {
        let progress = PushProgress {
            stage: PushStage::Writing,
            objects_written: 50,
            total_objects: 100,
            bytes_written: 4096,
            current_ref: Some("refs/heads/main".to_string()),
        };

        assert_eq!(progress.stage, PushStage::Writing);
        assert_eq!(progress.objects_written, 50);
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 453: Remote Operations
- Spec 455: Pull/Fetch Operations
- Spec 463: Credential Management
