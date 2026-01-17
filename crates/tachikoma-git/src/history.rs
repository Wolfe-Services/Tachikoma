//! Git history traversal.

use crate::{GitCommit, GitOid, GitRepository, GitResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// History query options.
#[derive(Debug, Clone, Default)]
pub struct HistoryOptions {
    /// Maximum commits to return.
    pub limit: Option<usize>,
    /// Skip this many commits.
    pub skip: usize,
    /// Start from this ref (default: HEAD).
    pub from: Option<String>,
    /// Stop at this commit.
    pub until: Option<GitOid>,
    /// Only commits affecting this path.
    pub path: Option<String>,
    /// Only commits by this author.
    pub author: Option<String>,
    /// Only commits matching this message.
    pub grep: Option<String>,
    /// Only commits after this date.
    pub after: Option<DateTime<Utc>>,
    /// Only commits before this date.
    pub before: Option<DateTime<Utc>>,
    /// Include merge commits.
    pub include_merges: bool,
    /// First parent only (simplified history).
    pub first_parent: bool,
}

impl HistoryOptions {
    /// Create with limit.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit: Some(limit),
            include_merges: true,
            ..Default::default()
        }
    }

    /// Set path filter.
    pub fn for_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set author filter.
    pub fn by_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set message grep.
    pub fn grep(mut self, pattern: impl Into<String>) -> Self {
        self.grep = Some(pattern.into());
        self
    }

    /// Set date range.
    pub fn between(mut self, after: DateTime<Utc>, before: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self.before = Some(before);
        self
    }

    /// Exclude merge commits.
    pub fn no_merges(mut self) -> Self {
        self.include_merges = false;
        self
    }

    /// Simplify history to first parent.
    pub fn first_parent(mut self) -> Self {
        self.first_parent = true;
        self
    }
}

/// Commit with graph information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The commit.
    pub commit: GitCommit,
    /// Graph column for this commit.
    pub graph_column: u32,
    /// Is this a branch point.
    pub is_branch: bool,
    /// Is this a merge point.
    pub is_merge: bool,
}

/// History result page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    /// Commits in this page.
    pub entries: Vec<HistoryEntry>,
    /// Total commits (if known).
    pub total: Option<usize>,
    /// Has more commits.
    pub has_more: bool,
    /// Cursor for next page.
    pub cursor: Option<String>,
}

impl GitRepository {
    /// Get commit history.
    pub fn history(&self, options: HistoryOptions) -> GitResult<HistoryPage> {
        self.with_repo(|repo| {
            let mut revwalk = repo.revwalk()?;

            // Set starting point
            if let Some(ref from) = options.from {
                let obj = repo.revparse_single(from)?;
                revwalk.push(obj.id())?;
            } else {
                revwalk.push_head()?;
            }

            // Set sorting
            let mut sort = git2::Sort::TIME;
            if options.first_parent {
                sort |= git2::Sort::TOPOLOGICAL;
                revwalk.simplify_first_parent()?;
            }
            revwalk.set_sorting(sort)?;

            // Collect commits
            let mut entries = Vec::new();
            let mut skipped = 0;
            let limit = options.limit.unwrap_or(usize::MAX);

            for oid_result in revwalk {
                let oid = oid_result?;

                // Stop at until commit
                if let Some(ref until) = options.until {
                    if oid == until.as_git2() {
                        break;
                    }
                }

                let commit = repo.find_commit(oid)?;

                // Apply filters
                if !self.commit_matches_filters(&commit, &options)? {
                    continue;
                }

                // Skip
                if skipped < options.skip {
                    skipped += 1;
                    continue;
                }

                // Check limit
                if entries.len() >= limit {
                    break;
                }

                let git_commit = GitCommit::from_git2(&commit);
                entries.push(HistoryEntry {
                    is_merge: git_commit.is_merge(),
                    is_branch: false, // Would need more analysis
                    graph_column: 0,  // Simplified
                    commit: git_commit,
                });
            }

            let has_more = entries.len() >= limit;
            let cursor = entries.last().map(|e| e.commit.oid.to_hex());

            Ok(HistoryPage {
                entries,
                total: None,
                has_more,
                cursor,
            })
        })
    }

    fn commit_matches_filters(
        &self,
        commit: &git2::Commit,
        options: &HistoryOptions,
    ) -> GitResult<bool> {
        // Merge filter
        if !options.include_merges && commit.parent_count() > 1 {
            return Ok(false);
        }

        // Author filter
        if let Some(ref author_filter) = options.author {
            let author = commit.author();
            let name = author.name().unwrap_or("");
            let email = author.email().unwrap_or("");
            if !name.contains(author_filter) && !email.contains(author_filter) {
                return Ok(false);
            }
        }

        // Message grep
        if let Some(ref grep) = options.grep {
            let message = commit.message().unwrap_or("");
            if !message.contains(grep) {
                return Ok(false);
            }
        }

        // Date filters
        let commit_time = DateTime::from_timestamp(commit.time().seconds(), 0)
            .unwrap_or_else(Utc::now);

        if let Some(after) = options.after {
            if commit_time < after {
                return Ok(false);
            }
        }

        if let Some(before) = options.before {
            if commit_time > before {
                return Ok(false);
            }
        }

        // Path filter - most complex
        if let Some(ref path) = options.path {
            if !self.commit_touches_path(commit, path)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn commit_touches_path(&self, commit: &git2::Commit, path: &str) -> GitResult<bool> {
        self.with_repo(|repo| {
            let tree = commit.tree()?;

            // Check if path exists in this commit
            if tree.get_path(Path::new(path)).is_err() {
                // Path doesn't exist, check if it was deleted
                if let Some(parent) = commit.parents().next() {
                    let parent_tree = parent.tree()?;
                    return Ok(parent_tree.get_path(Path::new(path)).is_ok());
                }
                return Ok(false);
            }

            // Check if path changed from parent
            if let Some(parent) = commit.parents().next() {
                let diff = repo.diff_tree_to_tree(
                    Some(&parent.tree()?),
                    Some(&tree),
                    None,
                )?;

                for delta in diff.deltas() {
                    if let Some(old_path) = delta.old_file().path() {
                        if old_path.to_string_lossy().contains(path) {
                            return Ok(true);
                        }
                    }
                    if let Some(new_path) = delta.new_file().path() {
                        if new_path.to_string_lossy().contains(path) {
                            return Ok(true);
                        }
                    }
                }

                Ok(false)
            } else {
                // Initial commit, path exists so it was added
                Ok(true)
            }
        })
    }

    /// Get file history.
    pub fn file_history(&self, path: &str, limit: Option<usize>) -> GitResult<Vec<GitCommit>> {
        let options = HistoryOptions {
            path: Some(path.to_string()),
            limit,
            ..Default::default()
        };

        let page = self.history(options)?;
        Ok(page.entries.into_iter().map(|e| e.commit).collect())
    }

    /// Search commits.
    pub fn search_commits(&self, query: &str, limit: usize) -> GitResult<Vec<GitCommit>> {
        let options = HistoryOptions {
            grep: Some(query.to_string()),
            limit: Some(limit),
            ..Default::default()
        };

        let page = self.history(options)?;
        Ok(page.entries.into_iter().map(|e| e.commit).collect())
    }
}