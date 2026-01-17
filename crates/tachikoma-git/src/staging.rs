//! Git staging (index) operations.

use crate::{GitRepository, GitResult, GitError};
use std::path::Path;

impl GitRepository {
    /// Stage a file for commit.
    pub fn stage_file(&self, path: impl AsRef<Path>) -> GitResult<()> {
        let path = path.as_ref();

        self.with_repo_mut(|repo| {
            let mut index = repo.index()?;

            // Check if file exists
            let workdir = repo.workdir().ok_or_else(|| GitError::InvalidOperation {
                message: "Cannot stage in bare repository".to_string(),
            })?;

            let full_path = workdir.join(path);

            if full_path.exists() {
                index.add_path(path)?;
            } else {
                // File was deleted
                index.remove_path(path)?;
            }

            index.write()?;
            Ok(())
        })
    }

    /// Stage multiple files.
    pub fn stage_files(&self, paths: &[impl AsRef<Path>]) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let mut index = repo.index()?;
            let workdir = repo.workdir().ok_or_else(|| GitError::InvalidOperation {
                message: "Cannot stage in bare repository".to_string(),
            })?;

            for path in paths {
                let path = path.as_ref();
                let full_path = workdir.join(path);

                if full_path.exists() {
                    index.add_path(path)?;
                } else {
                    index.remove_path(path)?;
                }
            }

            index.write()?;
            Ok(())
        })
    }

    /// Stage all changes.
    pub fn stage_all(&self) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let mut index = repo.index()?;
            index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
            index.write()?;
            Ok(())
        })
    }

    /// Stage files matching a pattern.
    pub fn stage_pattern(&self, pattern: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let mut index = repo.index()?;
            index.add_all([pattern].iter(), git2::IndexAddOption::DEFAULT, None)?;
            index.write()?;
            Ok(())
        })
    }

    /// Unstage a file.
    pub fn unstage_file(&self, path: impl AsRef<Path>) -> GitResult<()> {
        let path = path.as_ref();

        self.with_repo_mut(|repo| {
            let head = repo.head()?.peel_to_commit()?;
            let head_tree = head.tree()?;

            let mut index = repo.index()?;

            // If file exists in HEAD, restore it to HEAD state
            // Otherwise, remove it from index
            if let Ok(entry) = head_tree.get_path(path) {
                let blob = repo.find_blob(entry.id())?;
                index.add_frombuffer(
                    &git2::IndexEntry {
                        ctime: git2::IndexTime::new(0, 0),
                        mtime: git2::IndexTime::new(0, 0),
                        dev: 0,
                        ino: 0,
                        mode: entry.filemode() as u32,
                        uid: 0,
                        gid: 0,
                        file_size: blob.content().len() as u32,
                        id: entry.id(),
                        flags: 0,
                        flags_extended: 0,
                        path: path.to_string_lossy().as_bytes().to_vec(),
                    },
                    blob.content(),
                )?;
            } else {
                index.remove_path(path)?;
            }

            index.write()?;
            Ok(())
        })
    }

    /// Unstage all files.
    pub fn unstage_all(&self) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            let head = repo.head()?.peel_to_commit()?;
            repo.reset(head.as_object(), git2::ResetType::Mixed, None)?;
            Ok(())
        })
    }

    /// Check if there are staged changes.
    pub fn has_staged(&self) -> GitResult<bool> {
        self.with_repo(|repo| {
            let head = repo.head()?.peel_to_tree()?;
            let diff = repo.diff_tree_to_index(Some(&head), None, None)?;
            Ok(diff.deltas().len() > 0)
        })
    }
}