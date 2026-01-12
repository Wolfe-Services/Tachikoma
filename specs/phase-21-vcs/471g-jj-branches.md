# 471g - jj Branch Operations

**Phase:** 21 - VCS Integration
**Spec ID:** 471g
**Status:** Planned
**Dependencies:** 471f-jj-undo
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Implement jj branch operations. In jj, branches are just labels on commits - not required for work like in git.

---

## Acceptance Criteria

- [ ] List branches
- [ ] Create branch at current change
- [ ] Move branch to different change
- [ ] Delete branch
- [ ] Track remote branches

---

## Implementation Details

### src/jj/branches.rs

```rust
//! jj branch operations.

use crate::VcsResult;
use super::repo::{JjRepo, JjError};

/// Branch information.
#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub change_id: String,
    pub is_remote: bool,
    pub tracking: Option<String>,
}

impl JjRepo {
    /// List all branches.
    pub fn branches(&self) -> Result<Vec<Branch>, JjError> {
        let output = std::process::Command::new("jj")
            .args(["branch", "list", "--all"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_branches(&stdout))
    }

    /// Create a branch at the current change.
    pub fn create_branch(&self, name: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["branch", "create", name])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: format!("Created branch: {}", name),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Move a branch to a different change.
    pub fn move_branch(&self, name: &str, to: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["branch", "set", name, "-r", to])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: format!("Moved branch {} to {}", name, to),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Delete a branch.
    pub fn delete_branch(&self, name: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["branch", "delete", name])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: format!("Deleted branch: {}", name),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Track a remote branch.
    pub fn track_branch(&self, branch: &str, remote: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["branch", "track", &format!("{}@{}", branch, remote)])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: format!("Tracking {}/{}", remote, branch),
            affected_files: vec![],
            conflicts: vec![],
        })
    }
}

fn parse_branches(output: &str) -> Vec<Branch> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let is_remote = line.contains('@');
            let parts: Vec<&str> = line.split(':').collect();
            Branch {
                name: parts.get(0).unwrap_or(&"").trim().to_string(),
                change_id: parts.get(1).unwrap_or(&"").trim().to_string(),
                is_remote,
                tracking: None,
            }
        })
        .collect()
}
```

---

## Testing Requirements

1. List branches correctly
2. Create branch at current change
3. Move branch works
4. Delete branch works

---

## Related Specs

- Depends on: [471f-jj-undo.md](471f-jj-undo.md)
- Next: [471h-git-compat.md](471h-git-compat.md)
