# 471f - jj Undo/Redo Operations

**Phase:** 21 - VCS Integration
**Spec ID:** 471f
**Status:** Planned
**Dependencies:** 471e-jj-conflicts
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Implement jj's operation log for undo/redo. Every jj operation is recorded and reversible - essential for agentic coding where we need to roll back agent mistakes.

---

## Why This Matters

When an agent makes a mistake:
- Git: Complex reflog archaeology, scary force operations
- jj: `jj undo` - done. Every operation is reversible.

---

## Acceptance Criteria

- [ ] Get operation log (recent operations)
- [ ] Undo last operation
- [ ] Undo specific operation by ID
- [ ] Redo (undo the undo)
- [ ] Get operation details

---

## Implementation Details

### src/jj/undo.rs

```rust
//! jj operation log and undo/redo.

use crate::{Operation, VcsResult};
use tachikoma_common_core::Timestamp;
use super::repo::{JjRepo, JjError};

impl JjRepo {
    /// Get recent operations.
    pub fn operation_log(&self, limit: usize) -> Result<Vec<Operation>, JjError> {
        let output = std::process::Command::new("jj")
            .args(["op", "log", "-n", &limit.to_string()])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_op_log(&stdout))
    }

    /// Undo the last operation.
    pub fn undo(&self) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["undo"])
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

    /// Undo a specific operation.
    pub fn undo_operation(&self, op_id: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["op", "undo", op_id])
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

    /// Restore to a specific operation state.
    pub fn restore_operation(&self, op_id: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["op", "restore", op_id])
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

    /// Get current operation ID.
    pub fn current_operation(&self) -> Result<String, JjError> {
        let output = std::process::Command::new("jj")
            .args(["op", "log", "-n", "1", "-T", "self.id()"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn parse_op_log(output: &str) -> Vec<Operation> {
    // Simplified parsing - real impl would parse jj's output format
    output
        .lines()
        .filter(|line| line.contains('@'))
        .map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            Operation {
                id: parts.get(0).unwrap_or(&"").trim_start_matches('@').to_string(),
                description: parts.get(1).unwrap_or(&"").to_string(),
                timestamp: Timestamp::now(),
                undoable: true,
            }
        })
        .collect()
}
```

---

## Testing Requirements

1. Operation log returns recent ops
2. Undo reverts last operation
3. Undo specific operation works
4. Restore to operation state works

---

## Related Specs

- Depends on: [471e-jj-conflicts.md](471e-jj-conflicts.md)
- Next: [471g-jj-branches.md](471g-jj-branches.md)
