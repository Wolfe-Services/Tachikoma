//! Repository wrapper for safe Git operations.

use crate::{detect::RepoInfo, GitError, GitResult, GitOid, GitCommit, GitRef, CommitOptions};
use git2::{Repository as Git2Repo, Signature};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Thread-safe repository wrapper.
pub struct GitRepository {
    inner: Arc<RwLock<Git2Repo>>,
    info: RepoInfo,
}

impl GitRepository {
    /// Open a repository at the given path.
    pub fn open(path: impl AsRef<Path>) -> GitResult<Self> {
        let repo = crate::detect::open_repo(path)?;
        let info = crate::detect::extract_repo_info(&repo)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(repo)),
            info,
        })
    }

    /// Discover and open a repository from the given path.
    pub fn discover(path: impl AsRef<Path>) -> GitResult<Self> {
        let repo = git2::Repository::discover(path.as_ref())?;
        let info = crate::detect::extract_repo_info(&repo)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(repo)),
            info,
        })
    }

    /// Initialize a new repository.
    pub fn init(path: impl AsRef<Path>, bare: bool) -> GitResult<Self> {
        let repo = if bare {
            git2::Repository::init_bare(path.as_ref())?
        } else {
            git2::Repository::init(path.as_ref())?
        };
        let info = crate::detect::extract_repo_info(&repo)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(repo)),
            info,
        })
    }

    /// Clone a repository.
    pub fn clone(url: &str, path: impl AsRef<Path>) -> GitResult<Self> {
        let repo = git2::Repository::clone(url, path.as_ref())?;
        let info = crate::detect::extract_repo_info(&repo)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(repo)),
            info,
        })
    }

    /// Get repository information.
    pub fn info(&self) -> &RepoInfo {
        &self.info
    }

    /// Get the repository root path.
    pub fn root_path(&self) -> &Path {
        &self.info.root_path
    }

    /// Get the .git directory path.
    pub fn git_dir(&self) -> &Path {
        &self.info.git_dir
    }

    /// Check if repository is bare.
    pub fn is_bare(&self) -> bool {
        self.info.is_bare
    }

    /// Access the underlying git2 repository (read).
    pub fn with_repo<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Git2Repo) -> R,
    {
        let repo = self.inner.read();
        f(&repo)
    }

    /// Access the underlying git2 repository (write).
    pub fn with_repo_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Git2Repo) -> R,
    {
        let mut repo = self.inner.write();
        f(&mut repo)
    }

    /// Refresh repository information.
    pub fn refresh(&mut self) -> GitResult<()> {
        let info = self.with_repo(|repo| crate::detect::extract_repo_info(repo))?;
        self.info = info;
        Ok(())
    }

    /// Get a specific commit by OID.
    pub fn get_commit(&self, oid: &GitOid) -> GitResult<GitCommit> {
        self.with_repo(|repo| {
            let commit = repo.find_commit(oid.as_git2())?;
            Ok(GitCommit::from_git2(&commit))
        })
    }
}

impl Clone for GitRepository {
    fn clone(&self) -> Self {
        // Re-open the repository for the clone
        Self::open(&self.info.root_path).expect("Failed to re-open repository")
    }
}

// Safety: GitRepository is Send + Sync because it uses Arc<RwLock<_>>
unsafe impl Send for GitRepository {}
unsafe impl Sync for GitRepository {}

/// Options for repository operations.
#[derive(Debug, Clone, Default)]
pub struct GitRepositoryOptions {
    /// Open flags.
    pub flags: Option<git2::RepositoryOpenFlags>,
    /// Search ceiling directories.
    pub ceiling_dirs: Vec<PathBuf>,
}

impl GitRepositoryOptions {
    /// Create new options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set open flags.
    pub fn with_flags(mut self, flags: git2::RepositoryOpenFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Add ceiling directory.
    pub fn with_ceiling(mut self, ceiling: impl Into<PathBuf>) -> Self {
        self.ceiling_dirs.push(ceiling.into());
        self
    }
}