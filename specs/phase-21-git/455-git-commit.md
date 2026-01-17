# 455 - Git Commit

**Phase:** 21 - Git Integration
**Spec ID:** 455
**Status:** Planned
**Dependencies:** 454-git-diff
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement Git commit operations, including staging files, creating commits, and amending commits.

---

## Acceptance Criteria

- [x] Stage individual files
- [x] Stage all changes
- [x] Unstage files
- [x] Create commits
- [x] Amend commits

---

## Implementation Details

### 1. Staging Operations (src/staging.rs)

```rust
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
```

### 2. Commit Operations (src/commit_ops.rs)

```rust
//! Git commit operations.

use crate::{CommitOptions, GitCommit, GitOid, GitRepository, GitResult, GitError, GitSignature};
use git2::Signature;

impl GitRepository {
    /// Create a commit with staged changes.
    pub fn commit(&self, options: CommitOptions) -> GitResult<GitOid> {
        self.with_repo_mut(|repo| {
            // Get the index
            let mut index = repo.index()?;

            // Check for staged changes
            if !options.allow_empty {
                let head = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
                let diff = repo.diff_tree_to_index(head.as_ref(), Some(&index), None)?;
                if diff.deltas().len() == 0 {
                    return Err(GitError::InvalidOperation {
                        message: "No changes staged for commit".to_string(),
                    });
                }
            }

            // Create tree from index
            let tree_oid = index.write_tree()?;
            let tree = repo.find_tree(tree_oid)?;

            // Get signatures
            let author = match options.author {
                Some((name, email)) => Signature::now(&name, &email)?,
                None => repo.signature()?,
            };
            let committer = match options.committer {
                Some((name, email)) => Signature::now(&name, &email)?,
                None => author.clone(),
            };

            // Get parent commits
            let parents = if options.amend {
                let head = repo.head()?.peel_to_commit()?;
                head.parents().collect::<Vec<_>>()
            } else if let Ok(head) = repo.head() {
                vec![head.peel_to_commit()?]
            } else {
                vec![] // Initial commit
            };

            let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

            // Create the commit
            let oid = if options.amend {
                let head = repo.head()?.peel_to_commit()?;
                head.amend(
                    Some("HEAD"),
                    Some(&author),
                    Some(&committer),
                    None,
                    Some(&options.message),
                    Some(&tree),
                )?
            } else {
                repo.commit(
                    Some("HEAD"),
                    &author,
                    &committer,
                    &options.message,
                    &tree,
                    &parent_refs,
                )?
            };

            Ok(GitOid::from_git2(oid))
        })
    }

    /// Get a commit by OID.
    pub fn get_commit(&self, oid: &GitOid) -> GitResult<GitCommit> {
        self.with_repo(|repo| {
            let commit = repo.find_commit(oid.as_git2())?;
            Ok(GitCommit::from_git2(&commit))
        })
    }

    /// Get the HEAD commit.
    pub fn head_commit(&self) -> GitResult<GitCommit> {
        self.with_repo(|repo| {
            let head = repo.head()?.peel_to_commit()?;
            Ok(GitCommit::from_git2(&head))
        })
    }

    /// Get commits in a range.
    pub fn commits_range(
        &self,
        from: Option<&GitOid>,
        to: &GitOid,
        limit: Option<usize>,
    ) -> GitResult<Vec<GitCommit>> {
        self.with_repo(|repo| {
            let mut revwalk = repo.revwalk()?;
            revwalk.push(to.as_git2())?;

            if let Some(from_oid) = from {
                revwalk.hide(from_oid.as_git2())?;
            }

            revwalk.set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL)?;

            let mut commits = Vec::new();
            for oid in revwalk {
                let oid = oid?;
                let commit = repo.find_commit(oid)?;
                commits.push(GitCommit::from_git2(&commit));

                if let Some(max) = limit {
                    if commits.len() >= max {
                        break;
                    }
                }
            }

            Ok(commits)
        })
    }

    /// Get recent commits from HEAD.
    pub fn recent_commits(&self, count: usize) -> GitResult<Vec<GitCommit>> {
        let head = self.with_repo(|repo| {
            Ok(GitOid::from_git2(repo.head()?.target().ok_or_else(|| {
                GitError::RefNotFound {
                    name: "HEAD".to_string(),
                }
            })?))
        })?;

        self.commits_range(None, &head, Some(count))
    }

    /// Revert a commit.
    pub fn revert_commit(&self, oid: &GitOid) -> GitResult<GitOid> {
        self.with_repo_mut(|repo| {
            let commit = repo.find_commit(oid.as_git2())?;
            let head = repo.head()?.peel_to_commit()?;

            // Perform the revert
            repo.revert(&commit, None)?;

            // Check if there are conflicts
            let index = repo.index()?;
            if index.has_conflicts() {
                return Err(GitError::MergeConflict {
                    files: index
                        .conflicts()?
                        .filter_map(|c| c.ok())
                        .filter_map(|c| c.our.or(c.their).or(c.ancestor))
                        .filter_map(|e| String::from_utf8(e.path).ok())
                        .collect(),
                });
            }

            // Create revert commit
            let message = format!("Revert \"{}\"", commit.summary().unwrap_or(""));
            let tree_oid = index.write_tree()?;
            let tree = repo.find_tree(tree_oid)?;
            let sig = repo.signature()?;

            let new_oid = repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &message,
                &tree,
                &[&head],
            )?;

            Ok(GitOid::from_git2(new_oid))
        })
    }
}

/// Commit builder for fluent API.
pub struct CommitBuilder {
    repo: GitRepository,
    options: CommitOptions,
}

impl CommitBuilder {
    /// Create a new commit builder.
    pub fn new(repo: GitRepository, message: impl Into<String>) -> Self {
        Self {
            repo,
            options: CommitOptions::with_message(message),
        }
    }

    /// Set author.
    pub fn author(mut self, name: impl Into<String>, email: impl Into<String>) -> Self {
        self.options.author = Some((name.into(), email.into()));
        self
    }

    /// Set committer.
    pub fn committer(mut self, name: impl Into<String>, email: impl Into<String>) -> Self {
        self.options.committer = Some((name.into(), email.into()));
        self
    }

    /// Allow empty commit.
    pub fn allow_empty(mut self) -> Self {
        self.options.allow_empty = true;
        self
    }

    /// Amend the last commit.
    pub fn amend(mut self) -> Self {
        self.options.amend = true;
        self
    }

    /// Execute the commit.
    pub fn execute(self) -> GitResult<GitOid> {
        self.repo.commit(self.options)
    }
}
```

---

## Testing Requirements

1. Staging adds files to index
2. Unstaging removes from index
3. Commits create valid objects
4. Amend modifies last commit
5. Empty commit handling works

---

## Related Specs

- Depends on: [454-git-diff.md](454-git-diff.md)
- Next: [456-git-branch.md](456-git-branch.md)
