# 452 - Git Detection

**Phase:** 21 - Git Integration
**Spec ID:** 452
**Status:** Planned
**Dependencies:** 451-git-core-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement Git repository detection, enabling automatic discovery of Git repositories in project directories.

---

## Acceptance Criteria

- [ ] Repository detection from any path
- [ ] Worktree detection
- [ ] Bare repository detection
- [ ] Repository metadata extraction
- [ ] Safe initialization

---

## Implementation Details

### 1. Repository Detection (src/detect.rs)

```rust
//! Git repository detection and discovery.

use crate::{GitError, GitResult};
use git2::{Repository, RepositoryOpenFlags};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Repository information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    /// Path to the repository root.
    pub root_path: PathBuf,
    /// Path to the .git directory.
    pub git_dir: PathBuf,
    /// Is this a bare repository.
    pub is_bare: bool,
    /// Is this a worktree.
    pub is_worktree: bool,
    /// Is this a shallow clone.
    pub is_shallow: bool,
    /// Current branch name (if any).
    pub current_branch: Option<String>,
    /// Number of remotes.
    pub remote_count: usize,
}

/// Detect a Git repository from a path.
pub fn detect_repo(path: impl AsRef<Path>) -> GitResult<Option<RepoInfo>> {
    let path = path.as_ref();

    match Repository::discover(path) {
        Ok(repo) => Ok(Some(extract_repo_info(&repo)?)),
        Err(e) => {
            if e.code() == git2::ErrorCode::NotFound {
                Ok(None)
            } else {
                Err(GitError::Git2(e))
            }
        }
    }
}

/// Open a repository at an exact path.
pub fn open_repo(path: impl AsRef<Path>) -> GitResult<Repository> {
    let path = path.as_ref();

    Repository::open(path).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            GitError::RepoNotFound {
                path: path.display().to_string(),
            }
        } else {
            GitError::Git2(e)
        }
    })
}

/// Open a repository with specific flags.
pub fn open_repo_with_flags(
    path: impl AsRef<Path>,
    flags: RepositoryOpenFlags,
) -> GitResult<Repository> {
    let path = path.as_ref();

    Repository::open_ext(path, flags, &[] as &[&str]).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            GitError::RepoNotFound {
                path: path.display().to_string(),
            }
        } else {
            GitError::Git2(e)
        }
    })
}

/// Extract repository information.
fn extract_repo_info(repo: &Repository) -> GitResult<RepoInfo> {
    let root_path = if repo.is_bare() {
        repo.path().to_path_buf()
    } else {
        repo.workdir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo.path().to_path_buf())
    };

    let current_branch = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().map(String::from));

    let remote_count = repo.remotes().map(|r| r.len()).unwrap_or(0);

    Ok(RepoInfo {
        root_path,
        git_dir: repo.path().to_path_buf(),
        is_bare: repo.is_bare(),
        is_worktree: repo.is_worktree(),
        is_shallow: repo.is_shallow(),
        current_branch,
        remote_count,
    })
}

/// Check if a path is inside a Git repository.
pub fn is_inside_repo(path: impl AsRef<Path>) -> bool {
    Repository::discover(path.as_ref()).is_ok()
}

/// Find the repository root from any path.
pub fn find_repo_root(path: impl AsRef<Path>) -> GitResult<Option<PathBuf>> {
    match Repository::discover(path.as_ref()) {
        Ok(repo) => {
            if repo.is_bare() {
                Ok(Some(repo.path().to_path_buf()))
            } else {
                Ok(repo.workdir().map(|p| p.to_path_buf()))
            }
        }
        Err(e) => {
            if e.code() == git2::ErrorCode::NotFound {
                Ok(None)
            } else {
                Err(GitError::Git2(e))
            }
        }
    }
}

/// Find all Git repositories under a directory.
pub fn find_repos(root: impl AsRef<Path>, max_depth: usize) -> Vec<PathBuf> {
    let root = root.as_ref();
    let mut repos = Vec::new();

    find_repos_recursive(root, 0, max_depth, &mut repos);

    repos
}

fn find_repos_recursive(
    path: &Path,
    depth: usize,
    max_depth: usize,
    repos: &mut Vec<PathBuf>,
) {
    if depth > max_depth {
        return;
    }

    // Check if this is a repo
    if path.join(".git").exists() || path.join("HEAD").exists() {
        repos.push(path.to_path_buf());
        return; // Don't descend into repos
    }

    // Descend into subdirectories
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // Skip hidden directories (except .git which we check above)
                let name = entry.file_name();
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
                find_repos_recursive(&entry_path, depth + 1, max_depth, repos);
            }
        }
    }
}

/// Repository detection options.
#[derive(Debug, Clone, Default)]
pub struct DetectOptions {
    /// Search across filesystem boundaries.
    pub cross_fs: bool,
    /// Search in parent directories.
    pub search_parents: bool,
    /// Maximum parent directories to search.
    pub ceiling: Option<PathBuf>,
}

impl DetectOptions {
    /// Create with parent search enabled.
    pub fn search_parents() -> Self {
        Self {
            search_parents: true,
            ..Default::default()
        }
    }

    /// Set a ceiling directory.
    pub fn with_ceiling(mut self, ceiling: impl Into<PathBuf>) -> Self {
        self.ceiling = Some(ceiling.into());
        self
    }

    /// Convert to git2 open flags.
    pub fn to_flags(&self) -> RepositoryOpenFlags {
        let mut flags = RepositoryOpenFlags::empty();

        if !self.cross_fs {
            flags |= RepositoryOpenFlags::NO_SEARCH;
        }

        flags
    }
}
```

### 2. Repository Wrapper (src/repository.rs)

```rust
//! Repository wrapper for safe Git operations.

use crate::{detect::RepoInfo, GitError, GitResult, GitOid, GitCommit, GitRef};
use git2::Repository as Git2Repo;
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
```

---

## Testing Requirements

1. Repository detection works from subdirectories
2. Worktree detection is accurate
3. Bare repositories are handled
4. Multiple repo discovery works
5. Thread-safe access works

---

## Related Specs

- Depends on: [451-git-core-types.md](451-git-core-types.md)
- Next: [453-git-status.md](453-git-status.md)
