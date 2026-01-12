# 471b - jj Repository Detection

**Phase:** 21 - VCS Integration
**Spec ID:** 471b
**Status:** Planned
**Dependencies:** 471a-vcs-crate-setup
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Implement jj repository detection and initialization.

---

## Acceptance Criteria

- [ ] Detect existing jj repo (.jj directory)
- [ ] Detect git repo and offer colocated jj init
- [ ] `JjRepo` struct with workspace reference
- [ ] Repository info extraction

---

## Implementation Details

### src/jj/repo.rs

```rust
//! jj repository operations.

use std::path::{Path, PathBuf};
use crate::{RepoInfo, VcsType, ChangeId};

/// A jj repository.
pub struct JjRepo {
    root: PathBuf,
    // jj-lib workspace handle would go here
}

impl JjRepo {
    /// Detect jj repo at or above the given path.
    pub fn detect(path: &Path) -> Option<Self> {
        let mut current = path.to_path_buf();
        loop {
            let jj_dir = current.join(".jj");
            if jj_dir.is_dir() {
                return Some(Self { root: current });
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Check if a git repo exists (for colocated init).
    pub fn has_git_repo(path: &Path) -> bool {
        let mut current = path.to_path_buf();
        loop {
            if current.join(".git").exists() {
                return true;
            }
            if !current.pop() {
                break;
            }
        }
        false
    }

    /// Initialize a new jj repo (colocated with git if present).
    pub fn init(path: &Path, colocate_git: bool) -> Result<Self, JjError> {
        // jj init [--git-repo=.]
        let mut cmd = std::process::Command::new("jj");
        cmd.arg("init").current_dir(path);

        if colocate_git && Self::has_git_repo(path) {
            cmd.arg("--git-repo=.");
        }

        let output = cmd.output().map_err(|e| JjError::Command(e.to_string()))?;

        if !output.status.success() {
            return Err(JjError::Init(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(Self { root: path.to_path_buf() })
    }

    /// Get repository root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get repository info.
    pub fn info(&self) -> Result<RepoInfo, JjError> {
        let working_copy = self.working_copy_id()?;
        let branch = self.current_branch()?;
        let is_dirty = self.has_changes()?;

        Ok(RepoInfo {
            root: self.root.clone(),
            vcs_type: VcsType::Jj,
            working_copy,
            branch,
            is_dirty,
        })
    }

    fn working_copy_id(&self) -> Result<ChangeId, JjError> {
        let output = std::process::Command::new("jj")
            .args(["log", "-r", "@", "-T", "change_id", "--no-graph"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(ChangeId(String::from_utf8_lossy(&output.stdout).trim().to_string()))
    }

    fn current_branch(&self) -> Result<Option<String>, JjError> {
        let output = std::process::Command::new("jj")
            .args(["branch", "list", "-r", "@"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let branch = stdout.lines().next().map(|s| s.trim().to_string());
        Ok(branch.filter(|s| !s.is_empty()))
    }

    fn has_changes(&self) -> Result<bool, JjError> {
        let output = std::process::Command::new("jj")
            .args(["diff", "--stat"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(!output.stdout.is_empty())
    }
}

/// jj operation errors.
#[derive(Debug, thiserror::Error)]
pub enum JjError {
    #[error("jj command failed: {0}")]
    Command(String),
    #[error("jj init failed: {0}")]
    Init(String),
    #[error("Not a jj repository")]
    NotRepo,
}
```

---

## Testing Requirements

1. Detect jj repo from subdirectory
2. Detect git repo for colocated init
3. Init creates .jj directory

---

## Related Specs

- Depends on: [471a-vcs-crate-setup.md](471a-vcs-crate-setup.md)
- Next: [471c-jj-status.md](471c-jj-status.md)
