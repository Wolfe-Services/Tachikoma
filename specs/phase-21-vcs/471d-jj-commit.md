# 471d - jj Commit Operations

**Phase:** 21 - VCS Integration
**Spec ID:** 471d
**Status:** Planned
**Dependencies:** 471c-jj-status
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Implement jj commit (describe) and new change operations. In jj, the working copy is always a commit - you "describe" it to add a message.

---

## Acceptance Criteria

- [ ] Describe current change (add commit message)
- [ ] Create new empty change
- [ ] Squash changes together
- [ ] Split a change

---

## Implementation Details

### src/jj/commit.rs

```rust
//! jj commit operations.

use crate::{ChangeId, VcsResult};
use super::repo::{JjRepo, JjError};

impl JjRepo {
    /// Describe the current working copy change.
    /// This is jj's equivalent of "git commit -m".
    pub fn describe(&self, message: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["describe", "-m", message])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let success = output.status.success();
        let change_id = if success {
            Some(self.working_copy_id()?)
        } else {
            None
        };

        Ok(VcsResult {
            success,
            change_id,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Create a new empty change on top of current.
    /// The current change becomes "committed" and we get a fresh working copy.
    pub fn new_change(&self) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["new"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let success = output.status.success();
        let change_id = if success {
            Some(self.working_copy_id()?)
        } else {
            None
        };

        Ok(VcsResult {
            success,
            change_id,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![],
            conflicts: vec![],
        })
    }

    /// Squash current change into parent.
    pub fn squash(&self) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["squash"])
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

    /// Squash with a message.
    pub fn squash_with_message(&self, message: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["squash", "-m", message])
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

    /// Commit workflow: describe + new (equivalent to git commit).
    pub fn commit(&self, message: &str) -> Result<VcsResult, JjError> {
        // First describe the current change
        self.describe(message)?;
        // Then create a new change
        self.new_change()
    }
}
```

---

## Testing Requirements

1. Describe adds message to working copy
2. New creates fresh change
3. Squash combines changes
4. Commit workflow works end-to-end

---

## Related Specs

- Depends on: [471c-jj-status.md](471c-jj-status.md)
- Next: [471e-jj-conflicts.md](471e-jj-conflicts.md)
