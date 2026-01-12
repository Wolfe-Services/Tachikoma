# 471j - jj Conflict Introspection

**Phase:** 21 - VCS Integration
**Spec ID:** 471j
**Status:** Planned
**Dependencies:** 471e-jj-conflicts
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Prefer jj's structured conflict API over parsing conflict markers in files. jj can provide conflict data programmatically, which is more reliable than regex parsing.

---

## Why This Matters

The 471e spec parses conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`), but:
- Markers can appear in code (strings, comments)
- Marker formats vary across tools
- jj already knows the conflict structure internally

**jj's structured output is the source of truth** - only fall back to marker parsing when needed.

---

## Acceptance Criteria

- [ ] Query conflicts via `jj resolve --list` JSON output
- [ ] Get conflict sides via `jj diff` structured output
- [ ] Prefer structured API, fall back to marker parsing
- [ ] Detect "real" conflicts vs marker-like content

---

## Implementation Details

### src/jj/conflict_introspection.rs

```rust
//! jj conflict introspection - prefer structured API over marker parsing.

use crate::{Conflict, ConflictSide};
use super::repo::{JjRepo, JjError};
use serde::Deserialize;

/// Structured conflict from jj.
#[derive(Debug, Deserialize)]
struct JjConflict {
    path: String,
    #[serde(default)]
    num_sides: usize,
}

/// Conflict side from jj diff.
#[derive(Debug, Deserialize)]
struct JjConflictSide {
    description: String,
    content: String,
}

impl JjRepo {
    /// Get conflicts using jj's structured output (preferred).
    /// Falls back to marker parsing if structured output unavailable.
    pub fn conflicts_structured(&self) -> Result<Vec<Conflict>, JjError> {
        // Try structured JSON output first
        match self.conflicts_from_jj_api() {
            Ok(conflicts) => Ok(conflicts),
            Err(_) => {
                // Fall back to marker parsing
                tracing::debug!("Falling back to marker parsing for conflicts");
                self.conflicts_from_markers()
            }
        }
    }

    /// Query conflicts via jj's internal API.
    fn conflicts_from_jj_api(&self) -> Result<Vec<Conflict>, JjError> {
        // jj resolve --list with template for structured output
        let output = std::process::Command::new("jj")
            .args([
                "resolve",
                "--list",
                "-T",
                r#"concat(path, "\n")"#,
            ])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        if !output.status.success() {
            return Err(JjError::Command("jj resolve --list failed".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let conflicts: Vec<Conflict> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|path| {
                // Get detailed info for each conflict
                let sides = self.get_conflict_sides(path).unwrap_or_default();
                Conflict {
                    path: path.into(),
                    conflict_count: sides.len().saturating_sub(1) / 2,
                    sides,
                }
            })
            .collect();

        Ok(conflicts)
    }

    /// Get the sides of a conflict using jj's diff.
    fn get_conflict_sides(&self, path: &str) -> Result<Vec<ConflictSide>, JjError> {
        // Use jj diff to get conflict content
        // jj diff --from @- --to @ <path> shows the conflicted state
        let output = std::process::Command::new("jj")
            .args(["file", "show", path])
            .current_dir(&self.root)
            .output()
            .map_err(|e| JjError::Command(e.to_string()))?;

        let content = String::from_utf8_lossy(&output.stdout);

        // If the file contains conflict markers, parse them
        if content.contains("<<<<<<<") && content.contains(">>>>>>>") {
            Ok(parse_conflict_markers_internal(&content))
        } else {
            // No markers = no conflict or already resolved
            Ok(vec![])
        }
    }

    /// Fall back to reading files and parsing markers.
    fn conflicts_from_markers(&self) -> Result<Vec<Conflict>, JjError> {
        // Get list of files that might have conflicts
        let status = self.status()?;

        let mut conflicts = vec![];
        for change in status {
            if change.has_conflict {
                let file_path = self.root.join(&change.path);
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    if has_real_conflict_markers(&content) {
                        let sides = parse_conflict_markers_internal(&content);
                        conflicts.push(Conflict {
                            path: change.path,
                            conflict_count: count_conflict_regions(&content),
                            sides,
                        });
                    }
                }
            }
        }
        Ok(conflicts)
    }

    /// Resolve using jj's API (preferred over manual file editing).
    pub fn resolve_structured(
        &self,
        path: &str,
        resolution: ConflictResolution,
    ) -> Result<(), JjError> {
        match resolution {
            ConflictResolution::TakeSide(side) => {
                let tool = format!(":{}", side);
                std::process::Command::new("jj")
                    .args(["resolve", "--tool", &tool, path])
                    .current_dir(&self.root)
                    .output()
                    .map_err(|e| JjError::Command(e.to_string()))?;
            }
            ConflictResolution::Custom(content) => {
                // Write content and mark resolved
                let file_path = self.root.join(path);
                std::fs::write(&file_path, content)
                    .map_err(|e| JjError::Command(e.to_string()))?;
            }
        }
        Ok(())
    }
}

/// How to resolve a conflict.
pub enum ConflictResolution {
    /// Take one side: "left", "right", "base"
    TakeSide(String),
    /// Custom merged content
    Custom(String),
}

/// Check if content has REAL conflict markers (not just in strings/comments).
fn has_real_conflict_markers(content: &str) -> bool {
    // Simple heuristic: markers at start of line are likely real
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("<<<<<<<")
            || trimmed.starts_with("=======")
            || trimmed.starts_with(">>>>>>>")
            || trimmed.starts_with("|||||||")
    })
}

/// Count conflict regions in content.
fn count_conflict_regions(content: &str) -> usize {
    content.matches("<<<<<<<").count()
}

/// Parse conflict markers from content.
fn parse_conflict_markers_internal(content: &str) -> Vec<ConflictSide> {
    let mut sides = vec![];
    let mut current_side: Option<(String, Vec<String>)> = None;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            current_side = Some(("left".to_string(), vec![]));
        } else if line.starts_with("|||||||") {
            if let Some((desc, lines)) = current_side.take() {
                sides.push(ConflictSide {
                    description: desc,
                    content: lines.join("\n"),
                });
            }
            current_side = Some(("base".to_string(), vec![]));
        } else if line.starts_with("=======") {
            if let Some((desc, lines)) = current_side.take() {
                sides.push(ConflictSide {
                    description: desc,
                    content: lines.join("\n"),
                });
            }
            current_side = Some(("right".to_string(), vec![]));
        } else if line.starts_with(">>>>>>>") {
            if let Some((desc, lines)) = current_side.take() {
                sides.push(ConflictSide {
                    description: desc,
                    content: lines.join("\n"),
                });
            }
        } else if let Some((_, ref mut lines)) = current_side {
            lines.push(line.to_string());
        }
    }

    sides
}
```

---

## Testing Requirements

1. Prefer structured output over marker parsing
2. Fall back gracefully when API unavailable
3. Detect false positive markers (in strings)
4. Parse real conflicts correctly

---

## Related Specs

- Depends on: [471e-jj-conflicts.md](471e-jj-conflicts.md)
