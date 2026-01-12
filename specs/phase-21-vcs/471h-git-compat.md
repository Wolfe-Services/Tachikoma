# 471h - Git Compatibility Layer

**Phase:** 21 - VCS Integration
**Spec ID:** 471h
**Status:** Planned
**Dependencies:** 471g-jj-branches
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Implement git compatibility for jj. jj can colocate with git repos and push/pull to git remotes natively.

---

## Why Git Compatibility

- Most remotes are git (GitHub, GitLab, etc.)
- Teams may have git-only users
- CI/CD often expects git
- Colocated mode: jj + git in same repo

---

## Acceptance Criteria

- [ ] Push to git remote
- [ ] Pull/fetch from git remote
- [ ] Export to git refs
- [ ] Import git refs
- [ ] Handle colocated repos

---

## Implementation Details

### src/jj/git_compat.rs

```rust
//! Git compatibility for jj.

use crate::VcsResult;
use super::repo::{JjRepo, JjError};

impl JjRepo {
    /// Fetch from a git remote.
    pub fn git_fetch(&self, remote: Option<&str>) -> Result<VcsResult, JjError> {
        let mut args = vec!["git", "fetch"];
        if let Some(r) = remote {
            args.push(r);
        }

        let output = std::process::Command::new("jj")
            .args(&args)
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Push to a git remote.
    pub fn git_push(&self, branch: &str, remote: Option<&str>) -> Result<VcsResult, JjError> {
        let mut args = vec!["git", "push", "-b", branch];
        if let Some(r) = remote {
            args.extend(["--remote", r]);
        }

        let output = std::process::Command::new("jj")
            .args(&args)
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Export jj changes to git refs.
    pub fn git_export(&self) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["git", "export"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Import git refs to jj.
    pub fn git_import(&self) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["git", "import"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Clone a git repository with jj.
    pub fn git_clone(url: &str, path: &str) -> Result<JjRepo, JjError> {
        let output = std::process::Command::new("jj")
            .args(["git", "clone", url, path])
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        if !output.status.success() {
            return Err(JjError::Command(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(JjRepo { root: path.into() })
    }

    /// Check if this is a colocated jj+git repo.
    pub fn is_colocated(&self) -> bool {
        self.root.join(".git").exists() && self.root.join(".jj").exists()
    }
}
```

---

## Testing Requirements

1. Fetch from git remote
2. Push to git remote
3. Export/import git refs
4. Clone git repo with jj

---

## Related Specs

- Depends on: [471g-jj-branches.md](471g-jj-branches.md)
- Next: [471i-vcs-tests.md](471i-vcs-tests.md)
