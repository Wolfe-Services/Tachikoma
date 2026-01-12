# Spec 448: Repository Operations

## Phase
21 - Git Integration

## Spec ID
448

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 447: Git Configuration (configuration management)

## Estimated Context
~11%

---

## Objective

Implement comprehensive repository management operations for Tachikoma, including repository discovery, initialization, cloning, and state inspection. This module serves as the primary interface for interacting with Git repositories and provides the foundation for all other Git operations.

---

## Acceptance Criteria

- [ ] Implement `GitRepository` wrapper with lifecycle management
- [ ] Support repository discovery (find repo from subdirectory)
- [ ] Implement repository initialization (regular and bare)
- [ ] Implement repository cloning with progress reporting
- [ ] Support repository state inspection (normal, merging, rebasing, etc.)
- [ ] Implement index/staging area operations
- [ ] Support submodule detection and listing
- [ ] Implement object lookup (commits, trees, blobs)
- [ ] Add repository validation and health checks
- [ ] Support repository path resolution

---

## Implementation Details

### Repository Manager

```rust
// src/git/repo.rs

use git2::{
    build::RepoBuilder, Cred, FetchOptions, IndexAddOption, ObjectType,
    RemoteCallbacks, Repository, RepositoryInitOptions, RepositoryState,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::GitConfig;
use super::types::*;

/// Repository state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoState {
    Clean,
    Merge,
    Revert,
    RevertSequence,
    CherryPick,
    CherryPickSequence,
    Bisect,
    Rebase,
    RebaseInteractive,
    RebaseMerge,
    ApplyMailbox,
    ApplyMailboxOrRebase,
}

impl From<RepositoryState> for RepoState {
    fn from(state: RepositoryState) -> Self {
        match state {
            RepositoryState::Clean => Self::Clean,
            RepositoryState::Merge => Self::Merge,
            RepositoryState::Revert => Self::Revert,
            RepositoryState::RevertSequence => Self::RevertSequence,
            RepositoryState::CherryPick => Self::CherryPick,
            RepositoryState::CherryPickSequence => Self::CherryPickSequence,
            RepositoryState::Bisect => Self::Bisect,
            RepositoryState::Rebase => Self::Rebase,
            RepositoryState::RebaseInteractive => Self::RebaseInteractive,
            RepositoryState::RebaseMerge => Self::RebaseMerge,
            RepositoryState::ApplyMailbox => Self::ApplyMailbox,
            RepositoryState::ApplyMailboxOrRebase => Self::ApplyMailboxOrRebase,
        }
    }
}

impl RepoState {
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Clean)
    }

    pub fn is_in_progress(&self) -> bool {
        !self.is_clean()
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Merge => "merging",
            Self::Revert => "reverting",
            Self::RevertSequence => "reverting (sequence)",
            Self::CherryPick => "cherry-picking",
            Self::CherryPickSequence => "cherry-picking (sequence)",
            Self::Bisect => "bisecting",
            Self::Rebase => "rebasing",
            Self::RebaseInteractive => "rebasing (interactive)",
            Self::RebaseMerge => "rebasing (merge)",
            Self::ApplyMailbox => "applying mailbox",
            Self::ApplyMailboxOrRebase => "applying mailbox or rebasing",
        }
    }
}

/// Clone progress information
#[derive(Debug, Clone)]
pub struct CloneProgress {
    pub stage: CloneStage,
    pub received_objects: usize,
    pub total_objects: usize,
    pub indexed_objects: usize,
    pub received_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneStage {
    Counting,
    Compressing,
    Receiving,
    Resolving,
    CheckingOut,
    Done,
}

/// Git repository wrapper
pub struct GitRepository {
    inner: Repository,
    path: PathBuf,
}

impl GitRepository {
    /// Open an existing repository
    pub fn open(path: impl AsRef<Path>) -> GitResult<Self> {
        let path = path.as_ref();
        let repo = Repository::open(path)?;
        let workdir = repo.workdir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo.path().to_path_buf());

        Ok(Self {
            inner: repo,
            path: workdir,
        })
    }

    /// Discover repository from a path (searches parent directories)
    pub fn discover(start_path: impl AsRef<Path>) -> GitResult<Self> {
        let path = start_path.as_ref();
        let repo = Repository::discover(path)?;
        let workdir = repo.workdir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo.path().to_path_buf());

        Ok(Self {
            inner: repo,
            path: workdir,
        })
    }

    /// Initialize a new repository
    pub fn init(path: impl AsRef<Path>, bare: bool) -> GitResult<Self> {
        let path = path.as_ref();

        let repo = if bare {
            Repository::init_bare(path)?
        } else {
            Repository::init(path)?
        };

        let workdir = repo.workdir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo.path().to_path_buf());

        Ok(Self {
            inner: repo,
            path: workdir,
        })
    }

    /// Initialize with options
    pub fn init_opts(path: impl AsRef<Path>, opts: &RepoInitOptions) -> GitResult<Self> {
        let path = path.as_ref();

        let mut git_opts = RepositoryInitOptions::new();
        git_opts.bare(opts.bare);
        git_opts.no_reinit(opts.no_reinit);
        git_opts.mkdir(opts.mkdir);
        git_opts.mkpath(opts.mkpath);

        if let Some(ref branch) = opts.initial_head {
            git_opts.initial_head(branch);
        }

        if let Some(ref desc) = opts.description {
            git_opts.description(desc);
        }

        let repo = Repository::init_opts(path, &git_opts)?;
        let workdir = repo.workdir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo.path().to_path_buf());

        Ok(Self {
            inner: repo,
            path: workdir,
        })
    }

    /// Clone a repository
    pub fn clone(
        url: &str,
        path: impl AsRef<Path>,
        progress_callback: Option<Box<dyn Fn(CloneProgress) + Send>>,
    ) -> GitResult<Self> {
        let path = path.as_ref();

        let mut callbacks = RemoteCallbacks::new();

        // Set up progress reporting
        if let Some(callback) = progress_callback {
            let callback = Arc::new(callback);
            let callback_clone = callback.clone();

            callbacks.transfer_progress(move |progress| {
                callback_clone(CloneProgress {
                    stage: CloneStage::Receiving,
                    received_objects: progress.received_objects(),
                    total_objects: progress.total_objects(),
                    indexed_objects: progress.indexed_objects(),
                    received_bytes: progress.received_bytes(),
                });
                true
            });
        }

        // Set up credential handling
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            // Try SSH agent first
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    return Cred::ssh_key_from_agent(username);
                }
            }

            // Try default credentials
            if allowed_types.contains(git2::CredentialType::DEFAULT) {
                return Cred::default();
            }

            Err(git2::Error::from_str("No suitable credentials found"))
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let repo = RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(url, path)?;

        let workdir = repo.workdir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo.path().to_path_buf());

        Ok(Self {
            inner: repo,
            path: workdir,
        })
    }

    /// Get the repository working directory path
    pub fn workdir(&self) -> Option<&Path> {
        self.inner.workdir()
    }

    /// Get the repository .git path
    pub fn git_dir(&self) -> &Path {
        self.inner.path()
    }

    /// Get the repository root path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if repository is bare
    pub fn is_bare(&self) -> bool {
        self.inner.is_bare()
    }

    /// Check if repository is empty (no commits)
    pub fn is_empty(&self) -> GitResult<bool> {
        Ok(self.inner.is_empty()?)
    }

    /// Check if repository is shallow
    pub fn is_shallow(&self) -> bool {
        self.inner.is_shallow()
    }

    /// Get repository state
    pub fn state(&self) -> RepoState {
        RepoState::from(self.inner.state())
    }

    /// Get repository configuration
    pub fn config(&self) -> GitResult<GitConfig> {
        GitConfig::open_repo(&self.path)
    }

    /// Get HEAD reference
    pub fn head(&self) -> GitResult<GitReference> {
        let head = self.inner.head()?;
        self.reference_to_git_reference(&head, true)
    }

    /// Check if HEAD is detached
    pub fn is_head_detached(&self) -> GitResult<bool> {
        Ok(self.inner.head_detached()?)
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> GitResult<Option<String>> {
        if self.is_head_detached()? {
            return Ok(None);
        }

        let head = self.inner.head()?;
        Ok(head.shorthand().map(String::from))
    }

    /// Look up a commit by OID
    pub fn find_commit(&self, oid: &GitOid) -> GitResult<GitCommit> {
        let commit = self.inner.find_commit(oid.to_git2_oid())?;
        GitCommit::try_from(commit)
    }

    /// Look up a commit by revision string (e.g., "HEAD", "main", "abc123")
    pub fn revparse_single(&self, spec: &str) -> GitResult<GitOid> {
        let obj = self.inner.revparse_single(spec)?;
        Ok(GitOid::from(obj.id()))
    }

    /// Get the index (staging area)
    pub fn index(&self) -> GitResult<GitIndex> {
        let index = self.inner.index()?;
        Ok(GitIndex::new(index))
    }

    /// Stage a file
    pub fn stage_file(&self, path: impl AsRef<Path>) -> GitResult<()> {
        let mut index = self.inner.index()?;
        index.add_path(path.as_ref())?;
        index.write()?;
        Ok(())
    }

    /// Stage all changes
    pub fn stage_all(&self) -> GitResult<()> {
        let mut index = self.inner.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    /// Unstage a file
    pub fn unstage_file(&self, path: impl AsRef<Path>) -> GitResult<()> {
        let head = self.inner.head()?.peel_to_commit()?;
        self.inner.reset_default(Some(&head.into_object()), [path.as_ref()])?;
        Ok(())
    }

    /// Get list of submodules
    pub fn submodules(&self) -> GitResult<Vec<GitSubmodule>> {
        let submodules = self.inner.submodules()?;
        Ok(submodules
            .iter()
            .map(|sm| GitSubmodule {
                name: sm.name().unwrap_or("").to_string(),
                path: PathBuf::from(sm.path()),
                url: sm.url().map(String::from),
                head_oid: sm.head_id().map(GitOid::from),
            })
            .collect())
    }

    /// Check if path is ignored
    pub fn is_ignored(&self, path: impl AsRef<Path>) -> GitResult<bool> {
        Ok(self.inner.is_path_ignored(path.as_ref())?)
    }

    /// Get the raw git2 repository (for advanced operations)
    pub fn raw(&self) -> &Repository {
        &self.inner
    }

    /// Convert git2 Reference to GitReference
    fn reference_to_git_reference(
        &self,
        reference: &git2::Reference,
        is_head: bool,
    ) -> GitResult<GitReference> {
        let name = reference.name().unwrap_or("").to_string();
        let shorthand = reference.shorthand().unwrap_or("").to_string();

        let kind = if reference.is_branch() {
            GitReferenceKind::Branch
        } else if reference.is_remote() {
            GitReferenceKind::RemoteBranch
        } else if reference.is_tag() {
            GitReferenceKind::Tag
        } else if reference.is_note() {
            GitReferenceKind::Note
        } else if reference.symbolic_target().is_some() {
            GitReferenceKind::Symbolic
        } else {
            GitReferenceKind::Other
        };

        let target = reference.target().map(GitOid::from);
        let symbolic_target = reference.symbolic_target().map(String::from);

        Ok(GitReference {
            name,
            shorthand,
            kind,
            target,
            symbolic_target,
            is_head,
        })
    }
}

/// Repository initialization options
#[derive(Debug, Clone, Default)]
pub struct RepoInitOptions {
    pub bare: bool,
    pub no_reinit: bool,
    pub mkdir: bool,
    pub mkpath: bool,
    pub initial_head: Option<String>,
    pub description: Option<String>,
}

impl RepoInitOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bare(mut self, bare: bool) -> Self {
        self.bare = bare;
        self
    }

    pub fn initial_head(mut self, branch: impl Into<String>) -> Self {
        self.initial_head = Some(branch.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Git index wrapper
pub struct GitIndex {
    inner: git2::Index,
}

impl GitIndex {
    fn new(index: git2::Index) -> Self {
        Self { inner: index }
    }

    /// Get number of entries in the index
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if there are conflicts
    pub fn has_conflicts(&self) -> bool {
        self.inner.has_conflicts()
    }

    /// Get all entries
    pub fn entries(&self) -> Vec<GitIndexEntry> {
        self.inner
            .iter()
            .map(|entry| GitIndexEntry {
                path: PathBuf::from(String::from_utf8_lossy(&entry.path).to_string()),
                oid: GitOid::from(git2::Oid::from_bytes(&entry.id).unwrap()),
                mode: entry.mode,
                flags: entry.flags,
            })
            .collect()
    }
}

/// Index entry
#[derive(Debug, Clone)]
pub struct GitIndexEntry {
    pub path: PathBuf,
    pub oid: GitOid,
    pub mode: u32,
    pub flags: u16,
}

/// Submodule information
#[derive(Debug, Clone)]
pub struct GitSubmodule {
    pub name: String,
    pub path: PathBuf,
    pub url: Option<String>,
    pub head_oid: Option<GitOid>,
}

/// Thread-safe repository handle
pub struct SharedRepository {
    inner: Arc<RwLock<GitRepository>>,
}

impl SharedRepository {
    pub fn new(repo: GitRepository) -> Self {
        Self {
            inner: Arc::new(RwLock::new(repo)),
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, GitRepository> {
        self.inner.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, GitRepository> {
        self.inner.write().await
    }
}

impl Clone for SharedRepository {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
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
        (dir, repo)
    }

    #[test]
    fn test_repo_init() {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();

        assert!(!repo.is_bare());
        assert!(repo.is_empty().unwrap());
        assert_eq!(repo.state(), RepoState::Clean);
    }

    #[test]
    fn test_repo_init_bare() {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), true).unwrap();

        assert!(repo.is_bare());
    }

    #[test]
    fn test_repo_discover() {
        let dir = TempDir::new().unwrap();
        let _repo = GitRepository::init(dir.path(), false).unwrap();

        // Create a subdirectory
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // Should discover parent repo
        let discovered = GitRepository::discover(&subdir).unwrap();
        assert_eq!(discovered.path(), dir.path());
    }

    #[test]
    fn test_repo_open() {
        let (dir, _repo) = setup_test_repo();

        // Re-open the repository
        let reopened = GitRepository::open(dir.path()).unwrap();
        assert!(!reopened.is_bare());
    }

    #[test]
    fn test_repo_stage_file() {
        let (dir, repo) = setup_test_repo();

        // Create a test file
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello").unwrap();

        // Stage the file
        repo.stage_file(Path::new("test.txt")).unwrap();

        let index = repo.index().unwrap();
        assert!(!index.is_empty());
    }

    #[test]
    fn test_repo_state_descriptions() {
        assert_eq!(RepoState::Clean.description(), "clean");
        assert_eq!(RepoState::Merge.description(), "merging");
        assert_eq!(RepoState::Rebase.description(), "rebasing");

        assert!(RepoState::Clean.is_clean());
        assert!(RepoState::Merge.is_in_progress());
    }

    #[test]
    fn test_repo_init_options() {
        let opts = RepoInitOptions::new()
            .bare(false)
            .initial_head("main")
            .description("Test repository");

        assert_eq!(opts.initial_head, Some("main".to_string()));
        assert!(!opts.bare);
    }

    #[test]
    fn test_index_operations() {
        let (dir, repo) = setup_test_repo();

        // Create multiple files
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        std::fs::write(dir.path().join("b.txt"), "b").unwrap();

        // Stage all
        repo.stage_all().unwrap();

        let index = repo.index().unwrap();
        assert_eq!(index.len(), 2);
        assert!(!index.has_conflicts());
    }

    #[test]
    fn test_is_ignored() {
        let (dir, repo) = setup_test_repo();

        // Create .gitignore
        std::fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();

        assert!(repo.is_ignored("test.log").unwrap());
        assert!(!repo.is_ignored("test.txt").unwrap());
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 447: Git Configuration
- Spec 449: Status Checking
- Spec 464: Worktree Support
