# 471e - jj Conflict Handling (Key Differentiator)

**Phase:** 21 - VCS Integration
**Spec ID:** 471e
**Status:** Planned
**Dependencies:** 471d-jj-commit
**Estimated Context:** ~4% of Sonnet window

---

## Objective

Implement jj's superior conflict handling. Unlike git, jj allows committing with conflicts and resolving them later - critical for agentic coding where agents may create conflicting changes.

---

## Why This Matters for Agentic Coding

In traditional git workflows, merge conflicts **block all progress**. The agent must stop, resolve, then continue.

In jj:
1. Conflicts can be committed as-is
2. Multiple agents can work concurrently
3. Conflicts are data, not errors
4. Resolution can happen asynchronously
5. Easy to see all conflicts across the tree

---

## Acceptance Criteria

- [ ] List all conflicts in working copy
- [ ] Get conflict details (all sides)
- [ ] Resolve conflict with chosen content
- [ ] Auto-resolve simple conflicts
- [ ] Commit with unresolved conflicts (jj feature!)

---

## Implementation Details

### src/jj/conflicts.rs

```rust
//! jj conflict handling - the killer feature for agentic coding.

use std::path::Path;
use crate::{Conflict, ConflictSide, VcsResult};
use super::repo::{JjRepo, JjError};

impl JjRepo {
    /// List all files with conflicts.
    pub fn list_conflicts(&self) -> Result<Vec<Conflict>, JjError> {
        let output = std::process::Command::new("jj")
            .args(["resolve", "--list"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_conflicts(&stdout))
    }

    /// Get detailed conflict info for a file.
    pub fn conflict_details(&self, path: &str) -> Result<Conflict, JjError> {
        // Read the file with conflict markers
        let file_path = self.root.join(path);
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| JjError::Command(e.to_string()))?;

        let sides = parse_conflict_markers(&content);

        Ok(Conflict {
            path: path.into(),
            conflict_count: sides.len().saturating_sub(1) / 2,
            sides,
        })
    }

    /// Resolve a conflict by choosing one side.
    pub fn resolve_with_side(&self, path: &str, side: &str) -> Result<VcsResult, JjError> {
        // jj resolve --tool=:$side $path
        let tool = format!(":{}", side); // :left, :right, :base
        let output = std::process::Command::new("jj")
            .args(["resolve", "--tool", &tool, path])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![path.into()],
            conflicts: vec![],
        })
    }

    /// Resolve a conflict with custom content.
    pub fn resolve_with_content(&self, path: &str, content: &str) -> Result<VcsResult, JjError> {
        // Write resolved content directly
        let file_path = self.root.join(path);
        std::fs::write(&file_path, content)
            .map_err(|e| JjError::Command(e.to_string()))?;

        // Mark as resolved
        let output = std::process::Command::new("jj")
            .args(["resolve", "--mark-resolved", path])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: "Conflict resolved".to_string(),
            affected_files: vec![path.into()],
            conflicts: vec![],
        })
    }

    /// Check if there are any unresolved conflicts.
    pub fn has_conflicts(&self) -> Result<bool, JjError> {
        let conflicts = self.list_conflicts()?;
        Ok(!conflicts.is_empty())
    }

    /// Restore conflict markers (un-resolve).
    pub fn restore_conflict(&self, path: &str) -> Result<VcsResult, JjError> {
        let output = std::process::Command::new("jj")
            .args(["restore", "--from", "@-", path])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(VcsResult {
            success: output.status.success(),
            change_id: None,
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            affected_files: vec![path.into()],
            conflicts: vec![],
        })
    }
}

fn parse_conflicts(output: &str) -> Vec<Conflict> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| Conflict {
            path: line.trim().into(),
            conflict_count: 1,
            sides: vec![],
        })
        .collect()
}

fn parse_conflict_markers(content: &str) -> Vec<ConflictSide> {
    let mut sides = vec![];
    let mut current_side: Option<(String, Vec<String>)> = None;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            current_side = Some(("left".to_string(), vec![]));
        } else if line.starts_with("|||||||") {
            if let Some((desc, content)) = current_side.take() {
                sides.push(ConflictSide {
                    description: desc,
                    content: content.join("\n"),
                });
            }
            current_side = Some(("base".to_string(), vec![]));
        } else if line.starts_with("=======") {
            if let Some((desc, content)) = current_side.take() {
                sides.push(ConflictSide {
                    description: desc,
                    content: content.join("\n"),
                });
            }
            current_side = Some(("right".to_string(), vec![]));
        } else if line.starts_with(">>>>>>>") {
            if let Some((desc, content)) = current_side.take() {
                sides.push(ConflictSide {
                    description: desc,
                    content: content.join("\n"),
                });
            }
        } else if let Some((_, ref mut content)) = current_side {
            content.push(line.to_string());
        }
    }

    sides
}
```

---

## Testing Requirements

1. List conflicts correctly
2. Parse conflict markers
3. Resolve with side selection
4. Resolve with custom content
5. Un-resolve (restore) works

---

## Related Specs

- Depends on: [471d-jj-commit.md](471d-jj-commit.md)
- Next: [471f-jj-undo.md](471f-jj-undo.md)
