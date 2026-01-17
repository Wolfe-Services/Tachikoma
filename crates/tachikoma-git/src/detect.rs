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
pub(crate) fn extract_repo_info(repo: &Repository) -> GitResult<RepoInfo> {
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