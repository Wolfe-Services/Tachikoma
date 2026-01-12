# 471c - jj Status Operations

**Phase:** 21 - VCS Integration
**Spec ID:** 471c
**Status:** Planned
**Dependencies:** 471b-jj-repository
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Implement jj status and diff operations for tracking working copy changes.

---

## Acceptance Criteria

- [ ] Get list of changed files
- [ ] Detect file change types (added, modified, deleted)
- [ ] Detect conflicts in working copy
- [ ] Get diff for specific files

---

## Implementation Details

### src/jj/status.rs

```rust
//! jj status and diff operations.

use std::path::PathBuf;
use crate::{FileChange, ChangeType, Conflict};
use super::repo::{JjRepo, JjError};

impl JjRepo {
    /// Get status of working copy changes.
    pub fn status(&self) -> Result<Vec<FileChange>, JjError> {
        let output = std::process::Command::new("jj")
            .args(["diff", "--summary"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let changes = parse_diff_summary(&stdout);
        Ok(changes)
    }

    /// Get files with conflicts.
    pub fn conflicts(&self) -> Result<Vec<Conflict>, JjError> {
        let output = std::process::Command::new("jj")
            .args(["resolve", "--list"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let conflicts = parse_conflict_list(&stdout);
        Ok(conflicts)
    }

    /// Get diff for a specific file.
    pub fn diff_file(&self, path: &str) -> Result<String, JjError> {
        let output = std::process::Command::new("jj")
            .args(["diff", path])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get full diff of working copy.
    pub fn diff(&self) -> Result<String, JjError> {
        let output = std::process::Command::new("jj")
            .args(["diff"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

fn parse_diff_summary(output: &str) -> Vec<FileChange> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                return None;
            }
            let change_type = match parts[0] {
                "A" => ChangeType::Added,
                "M" => ChangeType::Modified,
                "D" => ChangeType::Deleted,
                "R" => ChangeType::Renamed,
                "C" => ChangeType::Conflicted,
                _ => return None,
            };
            Some(FileChange {
                path: PathBuf::from(parts[1].trim()),
                change_type,
                has_conflict: change_type == ChangeType::Conflicted,
            })
        })
        .collect()
}

fn parse_conflict_list(output: &str) -> Vec<Conflict> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| Conflict {
            path: PathBuf::from(line.trim()),
            conflict_count: 1, // Would need more parsing for exact count
            sides: vec![],
        })
        .collect()
}
```

---

## Testing Requirements

1. Parse diff summary correctly
2. Detect all change types
3. List conflicts accurately

---

## Related Specs

- Depends on: [471b-jj-repository.md](471b-jj-repository.md)
- Next: [471d-jj-commit.md](471d-jj-commit.md)
