# 462 - Git Blame

**Phase:** 21 - Git Integration
**Spec ID:** 462
**Status:** Planned
**Dependencies:** 461-git-history
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement Git blame functionality, showing the commit that last modified each line of a file.

---

## Acceptance Criteria

- [x] Blame for entire file
- [x] Blame for line range
- [x] Include commit details
- [x] Handle renames
- [x] Incremental blame

---

## Implementation Details

### 1. Blame Types (src/blame.rs)

```rust
//! Git blame functionality.

use crate::{GitCommit, GitOid, GitRepository, GitResult, GitSignature};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Blame result for a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameResult {
    /// Path that was blamed.
    pub path: String,
    /// Blame entries.
    pub entries: Vec<BlameEntry>,
    /// Total lines.
    pub total_lines: u32,
}

/// A single blame entry (hunk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameEntry {
    /// Commit OID that introduced these lines.
    pub commit_oid: GitOid,
    /// Commit summary.
    pub summary: String,
    /// Author signature.
    pub author: GitSignature,
    /// Original file path (for renames).
    pub orig_path: Option<String>,
    /// Starting line in final file.
    pub final_start_line: u32,
    /// Number of lines.
    pub lines_count: u32,
    /// Starting line in original commit.
    pub orig_start_line: u32,
    /// Is this line from a boundary commit.
    pub is_boundary: bool,
}

/// Blame options.
#[derive(Debug, Clone, Default)]
pub struct BlameOptions {
    /// Start line (1-indexed).
    pub start_line: Option<u32>,
    /// End line (1-indexed).
    pub end_line: Option<u32>,
    /// Follow renames.
    pub follow_renames: bool,
    /// Ignore whitespace changes.
    pub ignore_whitespace: bool,
    /// Commit to start from (default: HEAD).
    pub from: Option<GitOid>,
}

impl BlameOptions {
    /// Blame specific line range.
    pub fn lines(start: u32, end: u32) -> Self {
        Self {
            start_line: Some(start),
            end_line: Some(end),
            follow_renames: true,
            ..Default::default()
        }
    }

    /// Enable rename following.
    pub fn follow_renames(mut self) -> Self {
        self.follow_renames = true;
        self
    }

    /// Ignore whitespace.
    pub fn ignore_whitespace(mut self) -> Self {
        self.ignore_whitespace = true;
        self
    }

    /// Start from specific commit.
    pub fn from_commit(mut self, oid: GitOid) -> Self {
        self.from = Some(oid);
        self
    }
}

impl GitRepository {
    /// Get blame for a file.
    pub fn blame(&self, path: impl AsRef<Path>, options: BlameOptions) -> GitResult<BlameResult> {
        let path = path.as_ref();

        self.with_repo(|repo| {
            let mut blame_opts = git2::BlameOptions::new();

            if let Some(start) = options.start_line {
                blame_opts.min_line(start as usize);
            }
            if let Some(end) = options.end_line {
                blame_opts.max_line(end as usize);
            }
            if options.follow_renames {
                blame_opts.track_copies_same_file(true);
                blame_opts.track_copies_same_commit_moves(true);
                blame_opts.track_copies_same_commit_copies(true);
            }
            if options.ignore_whitespace {
                blame_opts.ignore_whitespace(true);
            }
            if let Some(ref from) = options.from {
                blame_opts.newest_commit(from.as_git2());
            }

            let blame = repo.blame_file(path, Some(&mut blame_opts))?;
            let mut entries = Vec::new();
            let mut total_lines = 0u32;

            for hunk in blame.iter() {
                let commit = repo.find_commit(hunk.final_commit_id())?;

                entries.push(BlameEntry {
                    commit_oid: GitOid::from_git2(hunk.final_commit_id()),
                    summary: commit.summary().unwrap_or("").to_string(),
                    author: GitSignature::from_git2(&commit.author()),
                    orig_path: hunk.path().map(|p| p.to_string_lossy().to_string()),
                    final_start_line: hunk.final_start_line() as u32,
                    lines_count: hunk.lines_in_hunk() as u32,
                    orig_start_line: hunk.orig_start_line() as u32,
                    is_boundary: hunk.is_boundary(),
                });

                total_lines += hunk.lines_in_hunk() as u32;
            }

            Ok(BlameResult {
                path: path.to_string_lossy().to_string(),
                entries,
                total_lines,
            })
        })
    }

    /// Get blame for a specific line.
    pub fn blame_line(&self, path: impl AsRef<Path>, line: u32) -> GitResult<Option<BlameEntry>> {
        let result = self.blame(path, BlameOptions::lines(line, line))?;
        Ok(result.entries.into_iter().next())
    }

    /// Get blame with full commit information.
    pub fn blame_detailed(
        &self,
        path: impl AsRef<Path>,
        options: BlameOptions,
    ) -> GitResult<Vec<(BlameEntry, GitCommit)>> {
        let result = self.blame(path, options)?;
        let mut detailed = Vec::new();

        for entry in result.entries {
            let commit = self.get_commit(&entry.commit_oid)?;
            detailed.push((entry, commit));
        }

        Ok(detailed)
    }
}

/// Per-line blame information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineBLame {
    /// Line number (1-indexed).
    pub line_number: u32,
    /// Line content.
    pub content: String,
    /// Commit that introduced this line.
    pub commit_oid: GitOid,
    /// Commit summary.
    pub summary: String,
    /// Author name.
    pub author: String,
    /// Commit date.
    pub date: chrono::DateTime<chrono::Utc>,
}

impl GitRepository {
    /// Get per-line blame (combines blame with file content).
    pub fn blame_lines(&self, path: impl AsRef<Path>) -> GitResult<Vec<LineBLame>> {
        let path = path.as_ref();
        let blame_result = self.blame(path, BlameOptions::default())?;

        // Read file content
        let content = self.with_repo(|repo| {
            let head = repo.head()?.peel_to_tree()?;
            let entry = head.get_path(path)?;
            let blob = repo.find_blob(entry.id())?;
            Ok(String::from_utf8_lossy(blob.content()).to_string())
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let line_num = (i + 1) as u32;

            // Find the blame entry for this line
            let entry = blame_result.entries.iter().find(|e| {
                line_num >= e.final_start_line
                    && line_num < e.final_start_line + e.lines_count
            });

            if let Some(entry) = entry {
                result.push(LineBLame {
                    line_number: line_num,
                    content: line.to_string(),
                    commit_oid: entry.commit_oid,
                    summary: entry.summary.clone(),
                    author: entry.author.name.clone(),
                    date: entry.author.when,
                });
            }
        }

        Ok(result)
    }
}
```

---

## Testing Requirements

1. Blame returns correct commits
2. Line range filtering works
3. Rename following works
4. Boundary commits are marked
5. Per-line blame is accurate

---

## Related Specs

- Depends on: [461-git-history.md](461-git-history.md)
- Next: [463-git-hooks.md](463-git-hooks.md)
