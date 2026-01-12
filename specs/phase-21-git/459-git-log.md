# Spec 459: Commit History (Log)

## Phase
21 - Git Integration

## Spec ID
459

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)

## Estimated Context
~9%

---

## Objective

Implement Git log functionality for Tachikoma, providing comprehensive commit history traversal, filtering, and formatting. This module supports various log formats, revision ranges, path filtering, and search capabilities essential for understanding project history.

---

## Acceptance Criteria

- [ ] Implement `GitLogger` for history traversal
- [ ] Support revision range specifications
- [ ] Support path filtering
- [ ] Implement commit search (grep in messages)
- [ ] Support author/committer filtering
- [ ] Support date range filtering
- [ ] Implement log formatting options
- [ ] Support graph visualization data
- [ ] Implement pagination for large histories
- [ ] Support follow renames

---

## Implementation Details

### Log Manager Implementation

```rust
// src/git/log.rs

use git2::{Commit, Oid, Revwalk, Sort, Time};
use chrono::{DateTime, TimeZone, Utc, Duration as ChronoDuration};
use regex::Regex;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::repo::GitRepository;
use super::types::*;

/// Options for log operations
#[derive(Debug, Clone, Default)]
pub struct LogOptions {
    /// Starting revision (default: HEAD)
    pub revision: Option<String>,
    /// Ending revision for range
    pub until: Option<String>,
    /// Maximum number of commits
    pub max_count: Option<usize>,
    /// Skip first N commits
    pub skip: usize,
    /// Filter by paths
    pub paths: Vec<PathBuf>,
    /// Filter by author name/email
    pub author: Option<String>,
    /// Filter by committer name/email
    pub committer: Option<String>,
    /// Filter by message content (grep)
    pub grep: Option<String>,
    /// Case insensitive grep
    pub grep_ignore_case: bool,
    /// After date
    pub after: Option<DateTime<Utc>>,
    /// Before date
    pub before: Option<DateTime<Utc>>,
    /// Only merge commits
    pub merges_only: bool,
    /// No merge commits
    pub no_merges: bool,
    /// First parent only (linear history)
    pub first_parent: bool,
    /// Topological sort
    pub topo_order: bool,
    /// Reverse order
    pub reverse: bool,
    /// Follow renames
    pub follow: bool,
    /// Include graph data
    pub graph: bool,
}

impl LogOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn revision(mut self, rev: impl Into<String>) -> Self {
        self.revision = Some(rev.into());
        self
    }

    pub fn range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.until = Some(from.into());
        self.revision = Some(to.into());
        self
    }

    pub fn max_count(mut self, count: usize) -> Self {
        self.max_count = Some(count);
        self
    }

    pub fn skip(mut self, count: usize) -> Self {
        self.skip = count;
        self
    }

    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.paths.push(path.into());
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn grep(mut self, pattern: impl Into<String>) -> Self {
        self.grep = Some(pattern.into());
        self
    }

    pub fn after(mut self, date: DateTime<Utc>) -> Self {
        self.after = Some(date);
        self
    }

    pub fn before(mut self, date: DateTime<Utc>) -> Self {
        self.before = Some(date);
        self
    }

    pub fn no_merges(mut self) -> Self {
        self.no_merges = true;
        self
    }

    pub fn first_parent(mut self) -> Self {
        self.first_parent = true;
        self
    }

    pub fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }
}

/// Graph node for visualization
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// The commit
    pub commit: GitCommit,
    /// Column index in graph
    pub column: usize,
    /// Parent connections
    pub parent_columns: Vec<usize>,
    /// Is this a merge point
    pub is_merge: bool,
    /// Is this a branch point
    pub is_branch_point: bool,
}

/// Log entry with optional graph data
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The commit
    pub commit: GitCommit,
    /// Refs pointing to this commit
    pub refs: Vec<String>,
    /// Graph node (if graph mode)
    pub graph: Option<GraphNode>,
}

/// Log iterator result
pub struct LogIterator<'a> {
    repo: &'a GitRepository,
    revwalk: Revwalk<'a>,
    options: LogOptions,
    grep_regex: Option<Regex>,
    count: usize,
    skipped: usize,
}

impl<'a> Iterator for LogIterator<'a> {
    type Item = GitResult<LogEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Check max count
            if let Some(max) = self.options.max_count {
                if self.count >= max {
                    return None;
                }
            }

            let oid = match self.revwalk.next() {
                Some(Ok(oid)) => oid,
                Some(Err(e)) => return Some(Err(GitError::Git2(e))),
                None => return None,
            };

            let commit = match self.repo.raw().find_commit(oid) {
                Ok(c) => c,
                Err(e) => return Some(Err(GitError::Git2(e))),
            };

            // Apply filters
            if !self.matches_filters(&commit) {
                continue;
            }

            // Handle skip
            if self.skipped < self.options.skip {
                self.skipped += 1;
                continue;
            }

            self.count += 1;

            // Convert commit
            let git_commit = match GitCommit::try_from(commit) {
                Ok(c) => c,
                Err(e) => return Some(Err(e)),
            };

            // Get refs
            let refs = self.get_refs_for_commit(&git_commit.oid);

            return Some(Ok(LogEntry {
                commit: git_commit,
                refs,
                graph: None, // Graph computation is separate
            }));
        }
    }
}

impl<'a> LogIterator<'a> {
    fn matches_filters(&self, commit: &Commit) -> bool {
        // Merge filter
        if self.options.no_merges && commit.parent_count() > 1 {
            return false;
        }
        if self.options.merges_only && commit.parent_count() <= 1 {
            return false;
        }

        // Author filter
        if let Some(ref author_pattern) = self.options.author {
            let author = commit.author();
            let author_str = format!(
                "{} <{}>",
                author.name().unwrap_or(""),
                author.email().unwrap_or("")
            );
            if !author_str.to_lowercase().contains(&author_pattern.to_lowercase()) {
                return false;
            }
        }

        // Committer filter
        if let Some(ref committer_pattern) = self.options.committer {
            let committer = commit.committer();
            let committer_str = format!(
                "{} <{}>",
                committer.name().unwrap_or(""),
                committer.email().unwrap_or("")
            );
            if !committer_str.to_lowercase().contains(&committer_pattern.to_lowercase()) {
                return false;
            }
        }

        // Date filters
        let commit_time = Utc.timestamp_opt(commit.time().seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        if let Some(ref after) = self.options.after {
            if commit_time < *after {
                return false;
            }
        }

        if let Some(ref before) = self.options.before {
            if commit_time > *before {
                return false;
            }
        }

        // Grep filter
        if let Some(ref regex) = self.grep_regex {
            let message = commit.message().unwrap_or("");
            if !regex.is_match(message) {
                return false;
            }
        }

        true
    }

    fn get_refs_for_commit(&self, oid: &GitOid) -> Vec<String> {
        let mut refs = Vec::new();

        if let Ok(references) = self.repo.raw().references() {
            for reference in references.flatten() {
                if let Some(target) = reference.target() {
                    if target == oid.to_git2_oid() {
                        if let Some(name) = reference.shorthand() {
                            refs.push(name.to_string());
                        }
                    }
                }
            }
        }

        refs
    }
}

/// Git log manager
pub struct GitLogger<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitLogger<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Get log iterator
    pub fn log(&'a self, options: LogOptions) -> GitResult<LogIterator<'a>> {
        let raw_repo = self.repo.raw();
        let mut revwalk = raw_repo.revwalk()?;

        // Configure sorting
        if options.topo_order {
            revwalk.set_sorting(Sort::TOPOLOGICAL)?;
        } else {
            revwalk.set_sorting(Sort::TIME)?;
        }

        if options.reverse {
            revwalk.set_sorting(Sort::REVERSE | revwalk.sorting())?;
        }

        // Set starting point
        let start = options.revision.as_deref().unwrap_or("HEAD");
        let start_oid = raw_repo.revparse_single(start)?.id();
        revwalk.push(start_oid)?;

        // Hide until revision if range
        if let Some(ref until) = options.until {
            let until_oid = raw_repo.revparse_single(until)?.id();
            revwalk.hide(until_oid)?;
        }

        // First parent only
        if options.first_parent {
            revwalk.simplify_first_parent()?;
        }

        // Compile grep regex
        let grep_regex = if let Some(ref pattern) = options.grep {
            let pattern = if options.grep_ignore_case {
                format!("(?i){}", pattern)
            } else {
                pattern.clone()
            };
            Some(Regex::new(&pattern).map_err(|e| GitError::Other(e.to_string()))?)
        } else {
            None
        };

        Ok(LogIterator {
            repo: self.repo,
            revwalk,
            options,
            grep_regex,
            count: 0,
            skipped: 0,
        })
    }

    /// Get commits as a vector (for small result sets)
    pub fn commits(&self, options: LogOptions) -> GitResult<Vec<GitCommit>> {
        let iter = self.log(options)?;
        iter.map(|r| r.map(|e| e.commit)).collect()
    }

    /// Get single commit by revision
    pub fn get(&self, revision: &str) -> GitResult<GitCommit> {
        self.repo.find_commit(&self.repo.revparse_single(revision)?)
    }

    /// Get commits affecting a specific file
    pub fn file_history(&self, path: &Path, options: LogOptions) -> GitResult<Vec<LogEntry>> {
        let mut opts = options;
        opts.paths.push(path.to_path_buf());

        // Note: For proper follow, we'd need custom logic to track renames
        // git2 doesn't directly support --follow

        let iter = self.log(opts)?;
        iter.collect()
    }

    /// Search commits by message
    pub fn search(&self, query: &str, options: LogOptions) -> GitResult<Vec<LogEntry>> {
        let opts = LogOptions {
            grep: Some(query.to_string()),
            ..options
        };
        let iter = self.log(opts)?;
        iter.collect()
    }

    /// Get commits by author
    pub fn by_author(&self, author: &str, options: LogOptions) -> GitResult<Vec<LogEntry>> {
        let opts = LogOptions {
            author: Some(author.to_string()),
            ..options
        };
        let iter = self.log(opts)?;
        iter.collect()
    }

    /// Get commit count
    pub fn count(&self, options: LogOptions) -> GitResult<usize> {
        let iter = self.log(options)?;
        Ok(iter.count())
    }

    /// Check if commit is ancestor of another
    pub fn is_ancestor(&self, ancestor: &str, descendant: &str) -> GitResult<bool> {
        let raw_repo = self.repo.raw();
        let ancestor_oid = raw_repo.revparse_single(ancestor)?.id();
        let descendant_oid = raw_repo.revparse_single(descendant)?.id();
        Ok(raw_repo.graph_descendant_of(descendant_oid, ancestor_oid)?)
    }

    /// Get merge base of two commits
    pub fn merge_base(&self, one: &str, two: &str) -> GitResult<GitOid> {
        let raw_repo = self.repo.raw();
        let one_oid = raw_repo.revparse_single(one)?.id();
        let two_oid = raw_repo.revparse_single(two)?.id();
        Ok(GitOid::from(raw_repo.merge_base(one_oid, two_oid)?))
    }
}

/// Format log entries
pub struct LogFormatter;

impl LogFormatter {
    /// Format as oneline
    pub fn oneline(entry: &LogEntry) -> String {
        let refs = if entry.refs.is_empty() {
            String::new()
        } else {
            format!(" ({})", entry.refs.join(", "))
        };
        format!("{}{} {}", entry.commit.oid.short(), refs, entry.commit.summary)
    }

    /// Format as short format
    pub fn short(entry: &LogEntry) -> String {
        format!(
            "commit {}\nAuthor: {} <{}>\n\n    {}\n",
            entry.commit.oid,
            entry.commit.author.name,
            entry.commit.author.email,
            entry.commit.summary
        )
    }

    /// Format as full format
    pub fn full(entry: &LogEntry) -> String {
        let refs = if entry.refs.is_empty() {
            String::new()
        } else {
            format!(" ({})", entry.refs.join(", "))
        };

        format!(
            "commit {}{}\nAuthor: {} <{}>\nDate:   {}\n\n    {}\n",
            entry.commit.oid,
            refs,
            entry.commit.author.name,
            entry.commit.author.email,
            entry.commit.author.time.format("%a %b %d %H:%M:%S %Y %z"),
            entry.commit.message.lines().collect::<Vec<_>>().join("\n    ")
        )
    }

    /// Format log as list
    pub fn format_list(entries: &[LogEntry], format: LogFormat) -> String {
        entries
            .iter()
            .map(|e| match format {
                LogFormat::Oneline => Self::oneline(e),
                LogFormat::Short => Self::short(e),
                LogFormat::Full => Self::full(e),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Oneline,
    Short,
    Full,
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

    fn setup_test_repo_with_commits() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();

        let mut config = repo.config().unwrap();
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

        // Create multiple commits
        for i in 1..=5 {
            let file = format!("file{}.txt", i);
            std::fs::write(dir.path().join(&file), format!("Content {}", i)).unwrap();
            repo.stage_file(std::path::Path::new(&file)).unwrap();

            let raw = repo.raw();
            let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
            let tree_id = raw.index().unwrap().write_tree().unwrap();
            let tree = raw.find_tree(tree_id).unwrap();

            let parents: Vec<_> = raw.head().ok()
                .and_then(|h| h.peel_to_commit().ok())
                .into_iter()
                .collect();
            let parent_refs: Vec<_> = parents.iter().collect();

            raw.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("Commit {}", i),
                &tree,
                &parent_refs,
            ).unwrap();
        }

        (dir, repo)
    }

    #[test]
    fn test_log_options_builder() {
        let opts = LogOptions::new()
            .revision("main")
            .max_count(10)
            .skip(5)
            .author("John")
            .no_merges();

        assert_eq!(opts.revision, Some("main".to_string()));
        assert_eq!(opts.max_count, Some(10));
        assert_eq!(opts.skip, 5);
        assert!(opts.no_merges);
    }

    #[test]
    fn test_log_range() {
        let opts = LogOptions::new().range("HEAD~3", "HEAD");
        assert_eq!(opts.until, Some("HEAD~3".to_string()));
        assert_eq!(opts.revision, Some("HEAD".to_string()));
    }

    #[test]
    fn test_log_basic() {
        let (_dir, repo) = setup_test_repo_with_commits();
        let logger = GitLogger::new(&repo);

        let commits = logger.commits(LogOptions::new()).unwrap();

        assert_eq!(commits.len(), 5);
    }

    #[test]
    fn test_log_max_count() {
        let (_dir, repo) = setup_test_repo_with_commits();
        let logger = GitLogger::new(&repo);

        let commits = logger.commits(LogOptions::new().max_count(3)).unwrap();

        assert_eq!(commits.len(), 3);
    }

    #[test]
    fn test_log_skip() {
        let (_dir, repo) = setup_test_repo_with_commits();
        let logger = GitLogger::new(&repo);

        let all_commits = logger.commits(LogOptions::new()).unwrap();
        let skipped = logger.commits(LogOptions::new().skip(2)).unwrap();

        assert_eq!(skipped.len(), all_commits.len() - 2);
    }

    #[test]
    fn test_log_search() {
        let (_dir, repo) = setup_test_repo_with_commits();
        let logger = GitLogger::new(&repo);

        let results = logger.search("Commit 3", LogOptions::new()).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].commit.message.contains("Commit 3"));
    }

    #[test]
    fn test_log_count() {
        let (_dir, repo) = setup_test_repo_with_commits();
        let logger = GitLogger::new(&repo);

        let count = logger.count(LogOptions::new()).unwrap();

        assert_eq!(count, 5);
    }

    #[test]
    fn test_format_oneline() {
        let entry = LogEntry {
            commit: GitCommit {
                oid: GitOid([0xab; 20]),
                tree_oid: GitOid([0; 20]),
                parent_oids: Vec::new(),
                author: GitSignature::new("Test", "test@example.com"),
                committer: GitSignature::new("Test", "test@example.com"),
                message: "Test commit".to_string(),
                summary: "Test commit".to_string(),
            },
            refs: vec!["main".to_string()],
            graph: None,
        };

        let formatted = LogFormatter::oneline(&entry);
        assert!(formatted.contains("abababa"));
        assert!(formatted.contains("(main)"));
        assert!(formatted.contains("Test commit"));
    }

    #[test]
    fn test_merge_base() {
        let (_dir, repo) = setup_test_repo_with_commits();
        let logger = GitLogger::new(&repo);

        // HEAD and HEAD~2 should have a merge base
        let base = logger.merge_base("HEAD", "HEAD~2").unwrap();
        assert!(!base.is_zero());
    }

    #[test]
    fn test_is_ancestor() {
        let (_dir, repo) = setup_test_repo_with_commits();
        let logger = GitLogger::new(&repo);

        assert!(logger.is_ancestor("HEAD~2", "HEAD").unwrap());
        assert!(!logger.is_ancestor("HEAD", "HEAD~2").unwrap());
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 460: Blame Information
- Spec 450: Diff Generation
