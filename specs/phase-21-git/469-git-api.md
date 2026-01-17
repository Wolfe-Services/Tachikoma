# 469 - Git API

**Phase:** 21 - Git Integration
**Spec ID:** 469
**Status:** Planned
**Dependencies:** 451-468 (all Git specs)
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement the unified Git API layer, providing a consistent interface for all Git operations exposed to the Tachikoma system.

---

## Acceptance Criteria

- [x] Repository operations API
- [x] Branch operations API
- [x] Commit operations API
- [x] Remote operations API
- [x] Status and diff API
- [x] History and blame API
- [x] Async operation support

---

## Implementation Details

### 1. Git API Types (src/api.rs)

```rust
//! Unified Git API.

use crate::{
    blame::{BlameOptions, BlameResult},
    branch::{BranchInfo, BranchType},
    commit::{CommitOptions, GitCommit},
    credentials::GitCredential,
    diff::{DiffOptions, GitDiff},
    history::{HistoryOptions, LogEntry},
    hooks::{HookInfo, HookResult, HookType},
    lfs::{LfsManager, LfsStatus},
    merge::{MergeOptions, MergeResult},
    remote::{GitRemote, RemoteBranch},
    status::{GitStatus, StatusOptions},
    worktree::{GitWorktree, WorktreeAddOptions},
    GitOid, GitRef, GitRepository, GitResult, GitError,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Git API providing unified access to all Git operations.
pub struct GitApi {
    repo: Arc<RwLock<GitRepository>>,
    credentials: Arc<RwLock<Option<GitCredential>>>,
}

impl GitApi {
    /// Create a new Git API instance.
    pub fn new(repo: GitRepository) -> Self {
        Self {
            repo: Arc::new(RwLock::new(repo)),
            credentials: Arc::new(RwLock::new(None)),
        }
    }

    /// Open a repository at the given path.
    pub fn open(path: impl AsRef<Path>) -> GitResult<Self> {
        let repo = GitRepository::open(path)?;
        Ok(Self::new(repo))
    }

    /// Discover and open a repository.
    pub fn discover(path: impl AsRef<Path>) -> GitResult<Self> {
        let repo = GitRepository::discover(path)?;
        Ok(Self::new(repo))
    }

    /// Set credentials for remote operations.
    pub async fn set_credentials(&self, credentials: GitCredential) {
        let mut creds = self.credentials.write().await;
        *creds = Some(credentials);
    }

    /// Clear credentials.
    pub async fn clear_credentials(&self) {
        let mut creds = self.credentials.write().await;
        *creds = None;
    }

    // === Repository Operations ===

    /// Get repository information.
    pub async fn info(&self) -> GitResult<RepositoryInfo> {
        let repo = self.repo.read().await;
        Ok(RepositoryInfo {
            root_path: repo.root_path().to_path_buf(),
            git_dir: repo.git_dir().to_path_buf(),
            is_bare: repo.is_bare()?,
            is_worktree: repo.is_worktree()?,
            head: repo.head_ref()?,
            default_branch: repo.default_branch()?,
        })
    }

    /// Initialize a new repository.
    pub fn init(path: impl AsRef<Path>, bare: bool) -> GitResult<Self> {
        let repo = if bare {
            GitRepository::init_bare(path)?
        } else {
            GitRepository::init(path)?
        };
        Ok(Self::new(repo))
    }

    /// Clone a repository.
    pub async fn clone(
        url: &str,
        path: impl AsRef<Path>,
        options: CloneOptions,
    ) -> GitResult<Self> {
        let repo = GitRepository::clone(url, path, options.into())?;
        Ok(Self::new(repo))
    }

    // === Status Operations ===

    /// Get repository status.
    pub async fn status(&self, options: StatusOptions) -> GitResult<GitStatus> {
        let repo = self.repo.read().await;
        repo.status(options)
    }

    /// Get quick status (staged, modified, untracked counts).
    pub async fn quick_status(&self) -> GitResult<QuickStatus> {
        let status = self.status(StatusOptions::default()).await?;
        Ok(QuickStatus {
            staged: status.staged.len(),
            modified: status.modified.len(),
            untracked: status.untracked.len(),
            conflicted: status.conflicted.len(),
            clean: status.is_clean(),
        })
    }

    // === Branch Operations ===

    /// List branches.
    pub async fn branches(&self, branch_type: Option<BranchType>) -> GitResult<Vec<BranchInfo>> {
        let repo = self.repo.read().await;
        repo.list_branches(branch_type)
    }

    /// Get current branch.
    pub async fn current_branch(&self) -> GitResult<Option<BranchInfo>> {
        let repo = self.repo.read().await;
        repo.current_branch()
    }

    /// Create a branch.
    pub async fn create_branch(
        &self,
        name: &str,
        target: Option<&str>,
        force: bool,
    ) -> GitResult<BranchInfo> {
        let repo = self.repo.write().await;
        repo.create_branch(name, target, force)
    }

    /// Delete a branch.
    pub async fn delete_branch(&self, name: &str, force: bool) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.delete_branch(name, force)
    }

    /// Rename a branch.
    pub async fn rename_branch(&self, old_name: &str, new_name: &str) -> GitResult<BranchInfo> {
        let repo = self.repo.write().await;
        repo.rename_branch(old_name, new_name)
    }

    /// Checkout a branch or commit.
    pub async fn checkout(&self, target: &str, create: bool) -> GitResult<()> {
        let repo = self.repo.write().await;
        if create {
            repo.create_branch(target, None, false)?;
        }
        repo.checkout(target)
    }

    // === Commit Operations ===

    /// Stage files.
    pub async fn stage(&self, paths: &[&Path]) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.stage_paths(paths)
    }

    /// Stage all changes.
    pub async fn stage_all(&self) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.stage_all()
    }

    /// Unstage files.
    pub async fn unstage(&self, paths: &[&Path]) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.unstage_paths(paths)
    }

    /// Create a commit.
    pub async fn commit(&self, message: &str, options: CommitOptions) -> GitResult<GitCommit> {
        let repo = self.repo.write().await;
        repo.commit(message, options)
    }

    /// Amend the last commit.
    pub async fn amend(&self, message: Option<&str>) -> GitResult<GitCommit> {
        let repo = self.repo.write().await;
        repo.amend(message)
    }

    /// Get a commit by OID.
    pub async fn get_commit(&self, oid: &GitOid) -> GitResult<GitCommit> {
        let repo = self.repo.read().await;
        repo.get_commit(oid)
    }

    // === Diff Operations ===

    /// Get diff between working directory and index.
    pub async fn diff_workdir(&self, options: DiffOptions) -> GitResult<GitDiff> {
        let repo = self.repo.read().await;
        repo.diff_workdir(options)
    }

    /// Get diff between index and HEAD.
    pub async fn diff_staged(&self, options: DiffOptions) -> GitResult<GitDiff> {
        let repo = self.repo.read().await;
        repo.diff_staged(options)
    }

    /// Get diff between two commits.
    pub async fn diff_commits(
        &self,
        from: &GitOid,
        to: &GitOid,
        options: DiffOptions,
    ) -> GitResult<GitDiff> {
        let repo = self.repo.read().await;
        repo.diff_commits(from, to, options)
    }

    // === History Operations ===

    /// Get commit history.
    pub async fn log(&self, options: HistoryOptions) -> GitResult<Vec<LogEntry>> {
        let repo = self.repo.read().await;
        repo.log(options)
    }

    /// Get blame for a file.
    pub async fn blame(
        &self,
        path: impl AsRef<Path>,
        options: BlameOptions,
    ) -> GitResult<BlameResult> {
        let repo = self.repo.read().await;
        repo.blame(path, options)
    }

    // === Remote Operations ===

    /// List remotes.
    pub async fn remotes(&self) -> GitResult<Vec<String>> {
        let repo = self.repo.read().await;
        repo.list_remotes()
    }

    /// Get remote information.
    pub async fn get_remote(&self, name: &str) -> GitResult<GitRemote> {
        let repo = self.repo.read().await;
        repo.get_remote(name)
    }

    /// Add a remote.
    pub async fn add_remote(&self, name: &str, url: &str) -> GitResult<GitRemote> {
        let repo = self.repo.write().await;
        repo.add_remote(name, url)
    }

    /// Remove a remote.
    pub async fn remove_remote(&self, name: &str) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.remove_remote(name)
    }

    /// Get remote branches.
    pub async fn remote_branches(&self, remote: Option<&str>) -> GitResult<Vec<RemoteBranch>> {
        let repo = self.repo.read().await;
        repo.remote_branches(remote)
    }

    /// Fetch from remote.
    pub async fn fetch(&self, remote: &str, refspecs: Option<&[&str]>) -> GitResult<FetchSummary> {
        let repo = self.repo.write().await;
        let creds = self.credentials.read().await;
        repo.fetch(remote, refspecs, creds.as_ref())
    }

    /// Push to remote.
    pub async fn push(
        &self,
        remote: &str,
        refspecs: Option<&[&str]>,
        force: bool,
    ) -> GitResult<PushSummary> {
        let repo = self.repo.write().await;
        let creds = self.credentials.read().await;
        repo.push(remote, refspecs, force, creds.as_ref())
    }

    /// Pull from remote.
    pub async fn pull(&self, remote: &str, branch: &str) -> GitResult<PullSummary> {
        let repo = self.repo.write().await;
        let creds = self.credentials.read().await;
        repo.pull(remote, branch, creds.as_ref())
    }

    // === Merge Operations ===

    /// Merge a branch.
    pub async fn merge(&self, branch: &str, options: MergeOptions) -> GitResult<MergeResult> {
        let repo = self.repo.write().await;
        repo.merge(branch, options)
    }

    /// Abort a merge.
    pub async fn merge_abort(&self) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.merge_abort()
    }

    // === Stash Operations ===

    /// Stash changes.
    pub async fn stash(&self, message: Option<&str>, include_untracked: bool) -> GitResult<GitOid> {
        let repo = self.repo.write().await;
        repo.stash(message, include_untracked)
    }

    /// Pop stash.
    pub async fn stash_pop(&self, index: Option<usize>) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.stash_pop(index)
    }

    /// List stashes.
    pub async fn stash_list(&self) -> GitResult<Vec<StashEntry>> {
        let repo = self.repo.read().await;
        repo.stash_list()
    }

    // === Tag Operations ===

    /// List tags.
    pub async fn tags(&self) -> GitResult<Vec<TagInfo>> {
        let repo = self.repo.read().await;
        repo.list_tags()
    }

    /// Create a tag.
    pub async fn create_tag(
        &self,
        name: &str,
        target: Option<&str>,
        message: Option<&str>,
    ) -> GitResult<TagInfo> {
        let repo = self.repo.write().await;
        repo.create_tag(name, target, message)
    }

    /// Delete a tag.
    pub async fn delete_tag(&self, name: &str) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.delete_tag(name)
    }

    // === Hook Operations ===

    /// List hooks.
    pub async fn hooks(&self) -> GitResult<Vec<HookInfo>> {
        let repo = self.repo.read().await;
        repo.list_hooks()
    }

    /// Run a hook.
    pub async fn run_hook(&self, hook_type: HookType, args: &[&str]) -> GitResult<HookResult> {
        let repo = self.repo.read().await;
        repo.run_hook(hook_type, args)
    }

    // === Worktree Operations ===

    /// List worktrees.
    pub async fn worktrees(&self) -> GitResult<Vec<GitWorktree>> {
        let repo = self.repo.read().await;
        repo.list_worktrees()
    }

    /// Add a worktree.
    pub async fn add_worktree(
        &self,
        path: impl AsRef<Path>,
        reference: &str,
        options: WorktreeAddOptions,
    ) -> GitResult<GitWorktree> {
        let repo = self.repo.write().await;
        repo.add_worktree(path, reference, options)
    }

    /// Remove a worktree.
    pub async fn remove_worktree(&self, name: &str, force: bool) -> GitResult<()> {
        let repo = self.repo.write().await;
        repo.remove_worktree(name, force)
    }

    // === LFS Operations ===

    /// Get LFS status.
    pub async fn lfs_status(&self) -> GitResult<LfsStatus> {
        let repo = self.repo.read().await;
        let lfs = LfsManager::new((*repo).clone());
        lfs.status()
    }

    /// Track pattern with LFS.
    pub async fn lfs_track(&self, pattern: &str) -> GitResult<()> {
        let repo = self.repo.read().await;
        let lfs = LfsManager::new((*repo).clone());
        lfs.track(pattern)
    }
}

/// Repository information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Root path.
    pub root_path: PathBuf,
    /// Git directory.
    pub git_dir: PathBuf,
    /// Is bare repository.
    pub is_bare: bool,
    /// Is worktree.
    pub is_worktree: bool,
    /// Current HEAD reference.
    pub head: Option<GitRef>,
    /// Default branch name.
    pub default_branch: Option<String>,
}

/// Quick status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickStatus {
    /// Number of staged files.
    pub staged: usize,
    /// Number of modified files.
    pub modified: usize,
    /// Number of untracked files.
    pub untracked: usize,
    /// Number of conflicted files.
    pub conflicted: usize,
    /// Working directory is clean.
    pub clean: bool,
}

/// Clone options.
#[derive(Debug, Clone, Default)]
pub struct CloneOptions {
    /// Branch to checkout.
    pub branch: Option<String>,
    /// Depth for shallow clone.
    pub depth: Option<u32>,
    /// Clone bare repository.
    pub bare: bool,
    /// Credentials for authentication.
    pub credentials: Option<GitCredential>,
}

/// Fetch summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSummary {
    /// Number of refs updated.
    pub refs_updated: usize,
    /// New branches.
    pub new_branches: Vec<String>,
    /// Deleted branches.
    pub deleted_branches: Vec<String>,
}

/// Push summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSummary {
    /// Refs pushed.
    pub refs_pushed: Vec<String>,
    /// Was force push.
    pub forced: bool,
}

/// Pull summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullSummary {
    /// Commits pulled.
    pub commits: usize,
    /// Files changed.
    pub files_changed: usize,
    /// Merge result.
    pub merge_result: MergeResult,
}

/// Stash entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEntry {
    /// Stash index.
    pub index: usize,
    /// Stash message.
    pub message: String,
    /// Stash OID.
    pub oid: GitOid,
    /// Branch at stash time.
    pub branch: Option<String>,
}

/// Tag information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// Tag name.
    pub name: String,
    /// Target OID.
    pub target: GitOid,
    /// Is annotated tag.
    pub annotated: bool,
    /// Tag message (for annotated tags).
    pub message: Option<String>,
    /// Tagger signature.
    pub tagger: Option<crate::GitSignature>,
}

/// Batch operation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum GitOperation {
    Status { options: StatusOptions },
    Log { options: HistoryOptions },
    Diff { from: Option<GitOid>, to: Option<GitOid> },
    Branches { branch_type: Option<BranchType> },
    Commit { message: String, options: CommitOptions },
    Checkout { target: String, create: bool },
    Fetch { remote: String },
    Push { remote: String, force: bool },
    Pull { remote: String, branch: String },
}

/// Batch operation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum GitOperationResult {
    Status(GitStatus),
    Log(Vec<LogEntry>),
    Diff(GitDiff),
    Branches(Vec<BranchInfo>),
    Commit(GitCommit),
    Checkout,
    Fetch(FetchSummary),
    Push(PushSummary),
    Pull(PullSummary),
    Error { message: String },
}

impl GitApi {
    /// Execute a batch of operations.
    pub async fn batch(&self, operations: Vec<GitOperation>) -> Vec<GitOperationResult> {
        let mut results = Vec::new();

        for op in operations {
            let result = match op {
                GitOperation::Status { options } => {
                    match self.status(options).await {
                        Ok(status) => GitOperationResult::Status(status),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Log { options } => {
                    match self.log(options).await {
                        Ok(log) => GitOperationResult::Log(log),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Diff { from, to } => {
                    let result = match (from, to) {
                        (Some(f), Some(t)) => self.diff_commits(&f, &t, DiffOptions::default()).await,
                        _ => self.diff_workdir(DiffOptions::default()).await,
                    };
                    match result {
                        Ok(diff) => GitOperationResult::Diff(diff),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Branches { branch_type } => {
                    match self.branches(branch_type).await {
                        Ok(branches) => GitOperationResult::Branches(branches),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Commit { message, options } => {
                    match self.commit(&message, options).await {
                        Ok(commit) => GitOperationResult::Commit(commit),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Checkout { target, create } => {
                    match self.checkout(&target, create).await {
                        Ok(()) => GitOperationResult::Checkout,
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Fetch { remote } => {
                    match self.fetch(&remote, None).await {
                        Ok(summary) => GitOperationResult::Fetch(summary),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Push { remote, force } => {
                    match self.push(&remote, None, force).await {
                        Ok(summary) => GitOperationResult::Push(summary),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
                GitOperation::Pull { remote, branch } => {
                    match self.pull(&remote, &branch).await {
                        Ok(summary) => GitOperationResult::Pull(summary),
                        Err(e) => GitOperationResult::Error { message: e.to_string() },
                    }
                }
            };
            results.push(result);
        }

        results
    }
}
```

---

## Testing Requirements

1. All API operations work correctly
2. Async operations are properly synchronized
3. Credentials are handled securely
4. Batch operations execute in order
5. Error handling is consistent

---

## Related Specs

- Depends on: All Git specs (451-468)
- Next: [470-git-tests.md](470-git-tests.md)
