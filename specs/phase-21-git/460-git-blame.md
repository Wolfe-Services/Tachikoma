# Spec 460: Blame Information

## Phase
21 - Git Integration

## Spec ID
460

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)

## Estimated Context
~8%

---

## Objective

Implement Git blame functionality for Tachikoma, providing line-by-line authorship information for files. This module enables tracking when and by whom each line was last modified, with support for following renames and detecting code movement.

---

## Acceptance Criteria

- [ ] Implement `GitBlamer` for blame operations
- [ ] Support basic file blame
- [ ] Support blame at specific revisions
- [ ] Support line range blame
- [ ] Implement follow renames (-C)
- [ ] Implement move detection (-M)
- [ ] Support ignore whitespace changes
- [ ] Provide blame statistics
- [ ] Support incremental blame loading
- [ ] Format blame output

---

## Implementation Details

### Blame Manager Implementation

```rust
// src/git/blame.rs

use git2::{Blame, BlameOptions, BlameHunk, Oid, Repository};
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::path::Path;

use super::repo::GitRepository;
use super::types::*;

/// Blame line information
#[derive(Debug, Clone)]
pub struct BlameLine {
    /// Line number in current file (1-indexed)
    pub line_number: usize,
    /// Line content
    pub content: String,
    /// Commit that introduced this line
    pub commit_oid: GitOid,
    /// Original line number in introducing commit
    pub original_line: usize,
    /// Author who introduced this line
    pub author: GitSignature,
    /// Whether this is a boundary commit
    pub is_boundary: bool,
}

/// Blame hunk (contiguous lines from same commit)
#[derive(Debug, Clone)]
pub struct BlameHunkInfo {
    /// Commit that introduced these lines
    pub commit_oid: GitOid,
    /// Original path (if renamed)
    pub original_path: Option<String>,
    /// Starting line in current file
    pub start_line: usize,
    /// Number of lines
    pub line_count: usize,
    /// Starting line in original file
    pub original_start: usize,
    /// Author
    pub author: GitSignature,
    /// Commit message summary
    pub summary: Option<String>,
    /// Is this a boundary commit
    pub is_boundary: bool,
}

/// Complete blame result
#[derive(Debug, Clone)]
pub struct BlameResult {
    /// Path of the blamed file
    pub path: String,
    /// Revision blamed at
    pub revision: Option<String>,
    /// Line-by-line blame
    pub lines: Vec<BlameLine>,
    /// Hunks for summary view
    pub hunks: Vec<BlameHunkInfo>,
}

impl BlameResult {
    /// Get blame for a specific line
    pub fn line(&self, line_number: usize) -> Option<&BlameLine> {
        self.lines.get(line_number.saturating_sub(1))
    }

    /// Get all unique commits
    pub fn unique_commits(&self) -> Vec<GitOid> {
        let mut commits: Vec<_> = self.hunks.iter()
            .map(|h| h.commit_oid)
            .collect();
        commits.sort();
        commits.dedup();
        commits
    }

    /// Get blame statistics by author
    pub fn by_author(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for line in &self.lines {
            *stats.entry(line.author.name.clone()).or_insert(0) += 1;
        }
        stats
    }

    /// Get percentage by author
    pub fn author_percentages(&self) -> HashMap<String, f64> {
        let stats = self.by_author();
        let total = self.lines.len() as f64;
        stats.into_iter()
            .map(|(author, count)| (author, (count as f64 / total) * 100.0))
            .collect()
    }
}

/// Options for blame operation
#[derive(Debug, Clone, Default)]
pub struct BlameOperationOptions {
    /// Revision to blame at (default: HEAD)
    pub revision: Option<String>,
    /// Starting line (1-indexed)
    pub start_line: Option<usize>,
    /// Ending line (1-indexed)
    pub end_line: Option<usize>,
    /// Follow renames
    pub follow_renames: bool,
    /// Move detection threshold
    pub move_threshold: Option<u32>,
    /// Copy detection threshold
    pub copy_threshold: Option<u32>,
    /// Ignore whitespace changes
    pub ignore_whitespace: bool,
    /// First parent only
    pub first_parent: bool,
}

impl BlameOperationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn revision(mut self, rev: impl Into<String>) -> Self {
        self.revision = Some(rev.into());
        self
    }

    pub fn lines(mut self, start: usize, end: usize) -> Self {
        self.start_line = Some(start);
        self.end_line = Some(end);
        self
    }

    pub fn follow_renames(mut self) -> Self {
        self.follow_renames = true;
        self
    }

    pub fn ignore_whitespace(mut self) -> Self {
        self.ignore_whitespace = true;
        self
    }

    pub fn first_parent(mut self) -> Self {
        self.first_parent = true;
        self
    }
}

/// Git blame manager
pub struct GitBlamer<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitBlamer<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Blame a file
    pub fn blame(&self, path: &Path, options: BlameOperationOptions) -> GitResult<BlameResult> {
        let raw_repo = self.repo.raw();

        // Build blame options
        let mut blame_opts = BlameOptions::new();

        if options.follow_renames {
            blame_opts.track_copies_same_file(true);
            blame_opts.track_copies_same_commit_moves(true);
        }

        if options.ignore_whitespace {
            // git2 doesn't have direct whitespace ignore
            // Would need custom handling
        }

        if options.first_parent {
            blame_opts.first_parent(true);
        }

        if let Some(start) = options.start_line {
            blame_opts.min_line(start);
        }

        if let Some(end) = options.end_line {
            blame_opts.max_line(end);
        }

        if let Some(ref rev) = options.revision {
            let commit = raw_repo.revparse_single(rev)?.peel_to_commit()?;
            blame_opts.newest_commit(commit.id());
        }

        // Run blame
        let blame = raw_repo.blame_file(path, Some(&mut blame_opts))?;

        // Read file content for line matching
        let content = self.read_file_at_revision(path, options.revision.as_deref())?;
        let lines: Vec<&str> = content.lines().collect();

        // Process hunks
        let mut result_lines = Vec::new();
        let mut hunks = Vec::new();

        for hunk_idx in 0..blame.len() {
            let hunk = blame.get_index(hunk_idx)
                .ok_or_else(|| GitError::Other("Invalid hunk index".into()))?;

            let hunk_info = self.process_hunk(&hunk, raw_repo)?;
            hunks.push(hunk_info.clone());

            // Add lines from this hunk
            let start = hunk.final_start_line();
            let count = hunk.lines_in_hunk();

            for i in 0..count {
                let line_num = start + i;
                let content = lines.get(line_num.saturating_sub(1))
                    .unwrap_or(&"")
                    .to_string();

                result_lines.push(BlameLine {
                    line_number: line_num,
                    content,
                    commit_oid: GitOid::from(hunk.final_commit_id()),
                    original_line: hunk.orig_start_line() + i,
                    author: GitSignature {
                        name: hunk.final_signature().name().unwrap_or("").to_string(),
                        email: hunk.final_signature().email().unwrap_or("").to_string(),
                        time: Utc.timestamp_opt(
                            hunk.final_signature().when().seconds(),
                            0
                        ).single().unwrap_or_else(Utc::now),
                    },
                    is_boundary: hunk.is_boundary(),
                });
            }
        }

        Ok(BlameResult {
            path: path.to_string_lossy().to_string(),
            revision: options.revision,
            lines: result_lines,
            hunks,
        })
    }

    /// Quick blame for a single line
    pub fn blame_line(&self, path: &Path, line: usize) -> GitResult<BlameLine> {
        let result = self.blame(
            path,
            BlameOperationOptions::new().lines(line, line)
        )?;

        result.lines.into_iter().next()
            .ok_or_else(|| GitError::Other("Line not found".into()))
    }

    /// Get blame summary (hunks only, no line content)
    pub fn blame_summary(&self, path: &Path, options: BlameOperationOptions) -> GitResult<Vec<BlameHunkInfo>> {
        let result = self.blame(path, options)?;
        Ok(result.hunks)
    }

    fn process_hunk(&self, hunk: &BlameHunk, repo: &Repository) -> GitResult<BlameHunkInfo> {
        let commit_id = hunk.final_commit_id();

        // Get commit message
        let summary = repo.find_commit(commit_id).ok()
            .and_then(|c| c.summary().map(String::from));

        // Get original path
        let orig_path = hunk.path().map(|p| p.to_string_lossy().to_string());

        let sig = hunk.final_signature();
        let time = Utc.timestamp_opt(sig.when().seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        Ok(BlameHunkInfo {
            commit_oid: GitOid::from(commit_id),
            original_path: orig_path,
            start_line: hunk.final_start_line(),
            line_count: hunk.lines_in_hunk(),
            original_start: hunk.orig_start_line(),
            author: GitSignature {
                name: sig.name().unwrap_or("").to_string(),
                email: sig.email().unwrap_or("").to_string(),
                time,
            },
            summary,
            is_boundary: hunk.is_boundary(),
        })
    }

    fn read_file_at_revision(&self, path: &Path, revision: Option<&str>) -> GitResult<String> {
        let raw_repo = self.repo.raw();

        let tree = if let Some(rev) = revision {
            let commit = raw_repo.revparse_single(rev)?.peel_to_commit()?;
            commit.tree()?
        } else {
            let head = raw_repo.head()?.peel_to_commit()?;
            head.tree()?
        };

        let entry = tree.get_path(path)?;
        let blob = raw_repo.find_blob(entry.id())?;

        Ok(String::from_utf8_lossy(blob.content()).to_string())
    }
}

/// Format blame output
pub struct BlameFormatter;

impl BlameFormatter {
    /// Format as standard blame output
    pub fn standard(result: &BlameResult) -> String {
        result.lines.iter()
            .map(|line| {
                format!(
                    "{} ({} {} {:4}) {}",
                    line.commit_oid.short(),
                    truncate(&line.author.name, 15),
                    line.author.time.format("%Y-%m-%d"),
                    line.line_number,
                    line.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format as porcelain output
    pub fn porcelain(result: &BlameResult) -> String {
        let mut output = String::new();

        for hunk in &result.hunks {
            output.push_str(&format!(
                "{} {} {} {}\n",
                hunk.commit_oid,
                hunk.original_start,
                hunk.start_line,
                hunk.line_count
            ));
            output.push_str(&format!("author {}\n", hunk.author.name));
            output.push_str(&format!("author-mail <{}>\n", hunk.author.email));
            output.push_str(&format!("author-time {}\n", hunk.author.time.timestamp()));

            if let Some(ref summary) = hunk.summary {
                output.push_str(&format!("summary {}\n", summary));
            }

            if hunk.is_boundary {
                output.push_str("boundary\n");
            }

            output.push('\n');
        }

        output
    }

    /// Format as simple line listing
    pub fn simple(result: &BlameResult) -> String {
        result.lines.iter()
            .map(|line| format!("{} {}", line.commit_oid.short(), line.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo_with_file() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();

        let mut config = repo.config().unwrap();
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

        // Create a file with multiple commits
        let file_path = dir.path().join("test.txt");

        // First commit
        std::fs::write(&file_path, "Line 1\n").unwrap();
        repo.stage_file(std::path::Path::new("test.txt")).unwrap();
        let raw = repo.raw();
        let sig = git2::Signature::now("Author 1", "author1@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Add line 1", &tree, &[]).unwrap();

        // Second commit
        std::fs::write(&file_path, "Line 1\nLine 2\n").unwrap();
        repo.stage_file(std::path::Path::new("test.txt")).unwrap();
        let sig = git2::Signature::now("Author 2", "author2@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        let parent = raw.head().unwrap().peel_to_commit().unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Add line 2", &tree, &[&parent]).unwrap();

        (dir, repo)
    }

    #[test]
    fn test_blame_options_builder() {
        let opts = BlameOperationOptions::new()
            .revision("HEAD~1")
            .lines(10, 20)
            .follow_renames()
            .ignore_whitespace();

        assert_eq!(opts.revision, Some("HEAD~1".to_string()));
        assert_eq!(opts.start_line, Some(10));
        assert_eq!(opts.end_line, Some(20));
        assert!(opts.follow_renames);
        assert!(opts.ignore_whitespace);
    }

    #[test]
    fn test_blame_basic() {
        let (dir, repo) = setup_test_repo_with_file();
        let blamer = GitBlamer::new(&repo);

        let result = blamer.blame(
            Path::new("test.txt"),
            BlameOperationOptions::new()
        ).unwrap();

        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.hunks.len(), 2);
    }

    #[test]
    fn test_blame_different_authors() {
        let (dir, repo) = setup_test_repo_with_file();
        let blamer = GitBlamer::new(&repo);

        let result = blamer.blame(
            Path::new("test.txt"),
            BlameOperationOptions::new()
        ).unwrap();

        let authors = result.by_author();
        assert!(authors.contains_key("Author 1"));
        assert!(authors.contains_key("Author 2"));
    }

    #[test]
    fn test_blame_unique_commits() {
        let (dir, repo) = setup_test_repo_with_file();
        let blamer = GitBlamer::new(&repo);

        let result = blamer.blame(
            Path::new("test.txt"),
            BlameOperationOptions::new()
        ).unwrap();

        let commits = result.unique_commits();
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn test_blame_line() {
        let (dir, repo) = setup_test_repo_with_file();
        let blamer = GitBlamer::new(&repo);

        let line = blamer.blame_line(Path::new("test.txt"), 1).unwrap();

        assert_eq!(line.line_number, 1);
        assert_eq!(line.author.name, "Author 1");
    }

    #[test]
    fn test_blame_result_get_line() {
        let (dir, repo) = setup_test_repo_with_file();
        let blamer = GitBlamer::new(&repo);

        let result = blamer.blame(
            Path::new("test.txt"),
            BlameOperationOptions::new()
        ).unwrap();

        let line = result.line(2).unwrap();
        assert_eq!(line.line_number, 2);
    }

    #[test]
    fn test_author_percentages() {
        let (dir, repo) = setup_test_repo_with_file();
        let blamer = GitBlamer::new(&repo);

        let result = blamer.blame(
            Path::new("test.txt"),
            BlameOperationOptions::new()
        ).unwrap();

        let percentages = result.author_percentages();
        assert_eq!(percentages.get("Author 1"), Some(&50.0));
        assert_eq!(percentages.get("Author 2"), Some(&50.0));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short     ");
        assert_eq!(truncate("a very long string", 10), "a very...");
    }

    #[test]
    fn test_format_simple() {
        let result = BlameResult {
            path: "test.txt".to_string(),
            revision: None,
            lines: vec![
                BlameLine {
                    line_number: 1,
                    content: "Hello".to_string(),
                    commit_oid: GitOid([0xab; 20]),
                    original_line: 1,
                    author: GitSignature::new("Test", "test@example.com"),
                    is_boundary: false,
                }
            ],
            hunks: Vec::new(),
        };

        let formatted = BlameFormatter::simple(&result);
        assert!(formatted.contains("abababa"));
        assert!(formatted.contains("Hello"));
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 459: Commit History
- Spec 450: Diff Generation
