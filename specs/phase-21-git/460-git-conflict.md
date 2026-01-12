# 460 - Git Conflict

**Phase:** 21 - Git Integration
**Spec ID:** 460
**Status:** Planned
**Dependencies:** 459-git-merge
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement Git conflict detection and resolution tools, enabling users to view and resolve merge conflicts.

---

## Acceptance Criteria

- [ ] Detect conflict state
- [ ] List conflicted files
- [ ] Read conflict markers
- [ ] Mark file as resolved
- [ ] Support different resolution strategies

---

## Implementation Details

### 1. Conflict Types (src/conflict.rs)

```rust
//! Git conflict handling.

use crate::{GitOid, GitRepository, GitResult, GitError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Conflict region in a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRegion {
    /// Start line number.
    pub start_line: u32,
    /// End line number.
    pub end_line: u32,
    /// Our (local) content.
    pub ours: String,
    /// Their (remote) content.
    pub theirs: String,
    /// Base (common ancestor) content.
    pub base: Option<String>,
}

/// Detailed conflict information for a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConflict {
    /// File path.
    pub path: PathBuf,
    /// Type of conflict.
    pub conflict_type: ConflictType,
    /// Conflict regions.
    pub regions: Vec<ConflictRegion>,
    /// Full content with conflict markers.
    pub content: String,
}

/// Type of conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Both modified.
    BothModified,
    /// Added on both sides differently.
    BothAdded,
    /// Modified vs deleted.
    ModifyDelete,
    /// Deleted vs modified.
    DeleteModify,
    /// Binary file conflict.
    Binary,
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Keep our version.
    Ours,
    /// Keep their version.
    Theirs,
    /// Keep base version.
    Base,
    /// Union (include both).
    Union,
}

impl GitRepository {
    /// Check if repository has conflicts.
    pub fn has_conflicts(&self) -> GitResult<bool> {
        self.with_repo(|repo| {
            let index = repo.index()?;
            Ok(index.has_conflicts())
        })
    }

    /// List all conflicted files.
    pub fn list_conflicts(&self) -> GitResult<Vec<PathBuf>> {
        self.with_repo(|repo| {
            let index = repo.index()?;
            let mut paths = Vec::new();

            for conflict in index.conflicts()? {
                let conflict = conflict?;
                if let Some(entry) = conflict.our.or(conflict.their).or(conflict.ancestor) {
                    if let Ok(path) = String::from_utf8(entry.path) {
                        paths.push(PathBuf::from(path));
                    }
                }
            }

            Ok(paths)
        })
    }

    /// Get detailed conflict information for a file.
    pub fn get_conflict(&self, path: impl AsRef<Path>) -> GitResult<FileConflict> {
        let path = path.as_ref();

        self.with_repo(|repo| {
            let workdir = repo.workdir().ok_or_else(|| GitError::InvalidOperation {
                message: "Cannot get conflict in bare repository".to_string(),
            })?;

            let full_path = workdir.join(path);
            let content = std::fs::read_to_string(&full_path)?;

            let regions = parse_conflict_markers(&content);
            let conflict_type = determine_conflict_type(repo, path)?;

            Ok(FileConflict {
                path: path.to_path_buf(),
                conflict_type,
                regions,
                content,
            })
        })
    }

    /// Resolve a conflict by choosing a side.
    pub fn resolve_conflict(
        &self,
        path: impl AsRef<Path>,
        strategy: ResolutionStrategy,
    ) -> GitResult<()> {
        let path = path.as_ref();

        self.with_repo_mut(|repo| {
            let index = repo.index()?;

            // Find conflict entries
            let conflict = index.conflicts()?.find(|c| {
                c.as_ref()
                    .ok()
                    .and_then(|c| c.our.as_ref().or(c.their.as_ref()))
                    .and_then(|e| String::from_utf8(e.path.clone()).ok())
                    .map(|p| p == path.to_string_lossy())
                    .unwrap_or(false)
            });

            let conflict = conflict.ok_or_else(|| GitError::InvalidOperation {
                message: format!("No conflict found for {}", path.display()),
            })??;

            let (content, mode) = match strategy {
                ResolutionStrategy::Ours => {
                    let entry = conflict.our.ok_or_else(|| GitError::InvalidOperation {
                        message: "No 'our' version".to_string(),
                    })?;
                    let blob = repo.find_blob(entry.id)?;
                    (blob.content().to_vec(), entry.mode)
                }
                ResolutionStrategy::Theirs => {
                    let entry = conflict.their.ok_or_else(|| GitError::InvalidOperation {
                        message: "No 'their' version".to_string(),
                    })?;
                    let blob = repo.find_blob(entry.id)?;
                    (blob.content().to_vec(), entry.mode)
                }
                ResolutionStrategy::Base => {
                    let entry = conflict.ancestor.ok_or_else(|| GitError::InvalidOperation {
                        message: "No base version".to_string(),
                    })?;
                    let blob = repo.find_blob(entry.id)?;
                    (blob.content().to_vec(), entry.mode)
                }
                ResolutionStrategy::Union => {
                    // Union merge: include both changes
                    let workdir = repo.workdir().unwrap();
                    let full_path = workdir.join(path);
                    let content = std::fs::read(&full_path)?;
                    let cleaned = remove_conflict_markers(&content);
                    (cleaned, conflict.our.or(conflict.their).map(|e| e.mode).unwrap_or(0o100644))
                }
            };

            // Write resolved content
            let workdir = repo.workdir().unwrap();
            let full_path = workdir.join(path);
            std::fs::write(&full_path, &content)?;

            // Stage the resolved file
            drop(index);
            let mut index = repo.index()?;
            index.add_path(path)?;
            index.write()?;

            Ok(())
        })
    }

    /// Resolve conflict with custom content.
    pub fn resolve_conflict_with_content(
        &self,
        path: impl AsRef<Path>,
        content: &[u8],
    ) -> GitResult<()> {
        let path = path.as_ref();

        self.with_repo_mut(|repo| {
            let workdir = repo.workdir().ok_or_else(|| GitError::InvalidOperation {
                message: "Cannot resolve conflict in bare repository".to_string(),
            })?;

            let full_path = workdir.join(path);
            std::fs::write(&full_path, content)?;

            let mut index = repo.index()?;
            index.add_path(path)?;
            index.write()?;

            Ok(())
        })
    }

    /// Mark all files as resolved (after manual resolution).
    pub fn resolve_all(&self) -> GitResult<()> {
        let conflicts = self.list_conflicts()?;

        for path in conflicts {
            self.with_repo_mut(|repo| {
                let mut index = repo.index()?;
                index.add_path(&path)?;
                index.write()?;
                Ok::<_, GitError>(())
            })?;
        }

        Ok(())
    }
}

fn parse_conflict_markers(content: &str) -> Vec<ConflictRegion> {
    let mut regions = Vec::new();
    let mut lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with("<<<<<<<") {
            let start_line = i as u32;
            let mut ours = String::new();
            let mut base = None;
            let mut theirs = String::new();
            let mut in_ours = true;
            let mut in_base = false;

            i += 1;
            while i < lines.len() {
                if lines[i].starts_with("|||||||") {
                    in_ours = false;
                    in_base = true;
                    base = Some(String::new());
                } else if lines[i].starts_with("=======") {
                    in_ours = false;
                    in_base = false;
                } else if lines[i].starts_with(">>>>>>>") {
                    break;
                } else if in_ours {
                    ours.push_str(lines[i]);
                    ours.push('\n');
                } else if in_base {
                    if let Some(ref mut b) = base {
                        b.push_str(lines[i]);
                        b.push('\n');
                    }
                } else {
                    theirs.push_str(lines[i]);
                    theirs.push('\n');
                }
                i += 1;
            }

            regions.push(ConflictRegion {
                start_line,
                end_line: i as u32,
                ours,
                theirs,
                base,
            });
        }
        i += 1;
    }

    regions
}

fn remove_conflict_markers(content: &[u8]) -> Vec<u8> {
    let content = String::from_utf8_lossy(content);
    let mut result = String::new();
    let mut in_conflict = false;
    let mut in_ours = false;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            in_ours = true;
        } else if line.starts_with("|||||||") {
            // Skip base section
            in_ours = false;
        } else if line.starts_with("=======") {
            in_ours = false;
        } else if line.starts_with(">>>>>>>") {
            in_conflict = false;
        } else if in_conflict {
            // Include both ours and theirs for union
            result.push_str(line);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result.into_bytes()
}

fn determine_conflict_type(repo: &git2::Repository, path: &Path) -> GitResult<ConflictType> {
    let index = repo.index()?;

    for conflict in index.conflicts()? {
        let conflict = conflict?;
        let conflict_path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .and_then(|e| String::from_utf8(e.path.clone()).ok());

        if conflict_path.as_deref() == Some(&path.to_string_lossy().to_string()) {
            return Ok(match (conflict.our.is_some(), conflict.their.is_some(), conflict.ancestor.is_some()) {
                (true, true, true) => ConflictType::BothModified,
                (true, true, false) => ConflictType::BothAdded,
                (true, false, true) => ConflictType::DeleteModify,
                (false, true, true) => ConflictType::ModifyDelete,
                _ => ConflictType::BothModified,
            });
        }
    }

    Ok(ConflictType::BothModified)
}
```

---

## Testing Requirements

1. Conflict detection works
2. Conflict markers are parsed correctly
3. Resolution strategies apply correctly
4. Custom content resolution works
5. resolve_all marks all resolved

---

## Related Specs

- Depends on: [459-git-merge.md](459-git-merge.md)
- Next: [461-git-history.md](461-git-history.md)
