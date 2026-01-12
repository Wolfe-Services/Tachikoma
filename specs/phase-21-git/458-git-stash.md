# Spec 458: Stash Operations

## Phase
21 - Git Integration

## Spec ID
458

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)
- Spec 449: Status Checking (status verification)

## Estimated Context
~8%

---

## Objective

Implement Git stash operations for Tachikoma, providing functionality to temporarily save uncommitted changes and restore them later. This module supports creating, listing, applying, and managing stash entries with options for including untracked files and keeping the index state.

---

## Acceptance Criteria

- [ ] Implement `GitStasher` for stash operations
- [ ] Support stash creation (save)
- [ ] Support stash listing
- [ ] Support stash application (apply)
- [ ] Support stash pop (apply and drop)
- [ ] Support stash drop
- [ ] Support stash clear
- [ ] Include untracked files option
- [ ] Keep index option
- [ ] Support stash show (diff)
- [ ] Support creating stash with custom message

---

## Implementation Details

### Stash Manager Implementation

```rust
// src/git/stash.rs

use git2::{Oid, Repository, Signature, Stash, StashFlags};
use chrono::{DateTime, TimeZone, Utc};

use super::diff::{DiffGenerationOptions, GitDiff, GitDiffGenerator};
use super::repo::GitRepository;
use super::types::*;

/// Options for stash creation
#[derive(Debug, Clone, Default)]
pub struct StashSaveOptions {
    /// Custom message for the stash
    pub message: Option<String>,
    /// Keep staged changes staged
    pub keep_index: bool,
    /// Include untracked files
    pub include_untracked: bool,
    /// Include ignored files
    pub include_ignored: bool,
}

impl StashSaveOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn keep_index(mut self) -> Self {
        self.keep_index = true;
        self
    }

    pub fn include_untracked(mut self) -> Self {
        self.include_untracked = true;
        self
    }

    pub fn include_ignored(mut self) -> Self {
        self.include_ignored = true;
        self
    }

    fn to_flags(&self) -> StashFlags {
        let mut flags = StashFlags::DEFAULT;

        if self.keep_index {
            flags |= StashFlags::KEEP_INDEX;
        }
        if self.include_untracked {
            flags |= StashFlags::INCLUDE_UNTRACKED;
        }
        if self.include_ignored {
            flags |= StashFlags::INCLUDE_IGNORED;
        }

        flags
    }
}

/// Options for stash application
#[derive(Debug, Clone, Default)]
pub struct StashApplyOptions {
    /// Stash index to apply (0 = most recent)
    pub index: usize,
    /// Reinstall index state
    pub reinstall_index: bool,
}

impl StashApplyOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn reinstall_index(mut self) -> Self {
        self.reinstall_index = true;
        self
    }
}

/// Stash entry with details
#[derive(Debug, Clone)]
pub struct StashEntryDetails {
    /// Stash index
    pub index: usize,
    /// Stash OID
    pub oid: GitOid,
    /// Stash message
    pub message: String,
    /// Branch stash was created on
    pub branch: Option<String>,
    /// When stash was created
    pub created_at: DateTime<Utc>,
    /// Files in stash
    pub files: Vec<String>,
}

/// Result of stash operation
#[derive(Debug, Clone)]
pub struct StashResult {
    /// Whether operation succeeded
    pub success: bool,
    /// Stash OID (for save operations)
    pub oid: Option<GitOid>,
    /// Message
    pub message: Option<String>,
    /// Conflicts (for apply operations)
    pub conflicts: Vec<String>,
}

/// Git stash manager
pub struct GitStasher<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitStasher<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Save changes to stash
    pub fn save(&self, options: StashSaveOptions) -> GitResult<StashResult> {
        let raw_repo = self.repo.raw();
        let sig = raw_repo.signature()?;

        let message = options.message.as_deref();
        let flags = options.to_flags();

        let oid = raw_repo.stash_save(&sig, message, Some(flags))?;

        Ok(StashResult {
            success: true,
            oid: Some(GitOid::from(oid)),
            message: options.message,
            conflicts: Vec::new(),
        })
    }

    /// List all stashes
    pub fn list(&self) -> GitResult<Vec<GitStashEntry>> {
        let raw_repo = self.repo.raw();
        let mut entries = Vec::new();

        raw_repo.stash_foreach(|index, message, oid| {
            entries.push(GitStashEntry {
                index,
                oid: GitOid::from(*oid),
                message: message.to_string(),
                committer: GitSignature::new("", ""), // Would need commit lookup for full info
            });
            true
        })?;

        Ok(entries)
    }

    /// Get detailed stash entry information
    pub fn get(&self, index: usize) -> GitResult<StashEntryDetails> {
        let raw_repo = self.repo.raw();
        let entries = self.list()?;

        let entry = entries
            .into_iter()
            .find(|e| e.index == index)
            .ok_or_else(|| GitError::Other(format!("Stash @{{{}}} not found", index)))?;

        // Get the stash commit for more details
        let commit = raw_repo.find_commit(entry.oid.to_git2_oid())?;
        let committer = commit.committer();

        let created_at = Utc
            .timestamp_opt(committer.when().seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        // Parse branch from message
        let branch = Self::parse_branch_from_message(&entry.message);

        // Get files changed in stash
        let files = self.get_stash_files(index)?;

        Ok(StashEntryDetails {
            index,
            oid: entry.oid,
            message: entry.message,
            branch,
            created_at,
            files,
        })
    }

    /// Apply stash without removing it
    pub fn apply(&self, options: StashApplyOptions) -> GitResult<StashResult> {
        let raw_repo = self.repo.raw();

        let mut apply_opts = git2::StashApplyOptions::new();
        if options.reinstall_index {
            apply_opts.reinstall_index();
        }

        match raw_repo.stash_apply(options.index, Some(&mut apply_opts)) {
            Ok(()) => Ok(StashResult {
                success: true,
                oid: None,
                message: None,
                conflicts: Vec::new(),
            }),
            Err(e) if e.code() == git2::ErrorCode::Conflict => {
                // Get conflicts
                let index = raw_repo.index()?;
                let conflicts: Vec<String> = index
                    .conflicts()?
                    .filter_map(|c| c.ok())
                    .filter_map(|c| {
                        c.our.or(c.their).or(c.ancestor)
                            .and_then(|e| String::from_utf8(e.path).ok())
                    })
                    .collect();

                Ok(StashResult {
                    success: false,
                    oid: None,
                    message: None,
                    conflicts,
                })
            }
            Err(e) => Err(GitError::Git2(e)),
        }
    }

    /// Pop stash (apply and drop)
    pub fn pop(&self, options: StashApplyOptions) -> GitResult<StashResult> {
        let index = options.index;
        let result = self.apply(options)?;

        if result.success {
            self.drop(index)?;
        }

        Ok(result)
    }

    /// Drop a stash entry
    pub fn drop(&self, index: usize) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        raw_repo.stash_drop(index)?;
        Ok(())
    }

    /// Clear all stashes
    pub fn clear(&self) -> GitResult<usize> {
        let entries = self.list()?;
        let count = entries.len();

        // Drop from highest to lowest index to avoid index shifting
        for i in (0..count).rev() {
            self.drop(i)?;
        }

        Ok(count)
    }

    /// Show diff of stash contents
    pub fn show(&self, index: usize) -> GitResult<GitDiff> {
        let raw_repo = self.repo.raw();

        // Get stash commit
        let entries = self.list()?;
        let entry = entries
            .into_iter()
            .find(|e| e.index == index)
            .ok_or_else(|| GitError::Other(format!("Stash @{{{}}} not found", index)))?;

        let stash_commit = raw_repo.find_commit(entry.oid.to_git2_oid())?;

        // Get parent (original state)
        let parent = stash_commit.parent(0)?;

        // Generate diff
        let parent_tree = parent.tree()?;
        let stash_tree = stash_commit.tree()?;

        let diff = raw_repo.diff_tree_to_tree(
            Some(&parent_tree),
            Some(&stash_tree),
            None,
        )?;

        // Convert to our diff type
        let generator = GitDiffGenerator::new(self.repo);
        generator.diff(
            super::diff::DiffTarget::Commit(GitOid::from(parent.id())),
            super::diff::DiffTarget::Commit(GitOid::from(stash_commit.id())),
            &DiffGenerationOptions::default(),
        )
    }

    /// Create a stash branch
    pub fn branch(&self, name: &str, index: usize) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        raw_repo.stash_pop(index, None)?;

        // Create and checkout branch
        let head = raw_repo.head()?.peel_to_commit()?;
        raw_repo.branch(name, &head, false)?;

        let refname = format!("refs/heads/{}", name);
        raw_repo.set_head(&refname)?;

        Ok(())
    }

    fn get_stash_files(&self, index: usize) -> GitResult<Vec<String>> {
        let diff = self.show(index)?;
        Ok(diff.files.iter().filter_map(|f| {
            f.new_path.as_ref().or(f.old_path.as_ref())
                .map(|p| p.to_string_lossy().to_string())
        }).collect())
    }

    fn parse_branch_from_message(message: &str) -> Option<String> {
        // Stash messages are typically "WIP on <branch>: <commit> <message>"
        // or "On <branch>: <user message>"
        if let Some(start) = message.find("on ").or_else(|| message.find("On ")) {
            let rest = &message[start + 3..];
            if let Some(end) = rest.find(':') {
                return Some(rest[..end].to_string());
            }
        }
        None
    }
}

/// Quick stash helper functions
pub fn has_stash(repo: &GitRepository) -> GitResult<bool> {
    let stasher = GitStasher::new(repo);
    let entries = stasher.list()?;
    Ok(!entries.is_empty())
}

pub fn stash_count(repo: &GitRepository) -> GitResult<usize> {
    let stasher = GitStasher::new(repo);
    let entries = stasher.list()?;
    Ok(entries.len())
}

/// Format stash entry for display
pub fn format_stash_entry(entry: &GitStashEntry) -> String {
    format!("stash@{{{}}}: {}", entry.index, entry.message)
}

pub fn format_stash_list(entries: &[GitStashEntry]) -> String {
    entries
        .iter()
        .map(format_stash_entry)
        .collect::<Vec<_>>()
        .join("\n")
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

    fn setup_test_repo() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();

        let mut config = repo.config().unwrap();
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

        std::fs::write(dir.path().join("README.md"), "# Test").unwrap();
        repo.stage_file(std::path::Path::new("README.md")).unwrap();

        let raw = repo.raw();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();

        (dir, repo)
    }

    #[test]
    fn test_stash_save_options_builder() {
        let opts = StashSaveOptions::new()
            .message("WIP: feature")
            .keep_index()
            .include_untracked();

        assert_eq!(opts.message, Some("WIP: feature".to_string()));
        assert!(opts.keep_index);
        assert!(opts.include_untracked);
    }

    #[test]
    fn test_stash_save_options_flags() {
        let opts = StashSaveOptions::new()
            .keep_index()
            .include_untracked();

        let flags = opts.to_flags();
        assert!(flags.contains(StashFlags::KEEP_INDEX));
        assert!(flags.contains(StashFlags::INCLUDE_UNTRACKED));
    }

    #[test]
    fn test_stash_apply_options_builder() {
        let opts = StashApplyOptions::new()
            .index(2)
            .reinstall_index();

        assert_eq!(opts.index, 2);
        assert!(opts.reinstall_index);
    }

    #[test]
    fn test_list_empty_stash() {
        let (_dir, repo) = setup_test_repo();
        let stasher = GitStasher::new(&repo);

        let entries = stasher.list().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_stash_save_with_changes() {
        let (dir, repo) = setup_test_repo();

        // Make changes
        std::fs::write(dir.path().join("README.md"), "# Modified").unwrap();
        repo.stage_file(std::path::Path::new("README.md")).unwrap();

        let stasher = GitStasher::new(&repo);
        let result = stasher.save(StashSaveOptions::new().message("WIP")).unwrap();

        assert!(result.success);
        assert!(result.oid.is_some());

        // Verify stash was created
        let entries = stasher.list().unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_stash_save_no_changes() {
        let (_dir, repo) = setup_test_repo();

        let stasher = GitStasher::new(&repo);
        let result = stasher.save(StashSaveOptions::new());

        // Should fail with no changes to stash
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_branch_from_message() {
        assert_eq!(
            GitStasher::parse_branch_from_message("WIP on main: abc123 message"),
            Some("main".to_string())
        );

        assert_eq!(
            GitStasher::parse_branch_from_message("On feature-branch: custom message"),
            Some("feature-branch".to_string())
        );

        assert_eq!(
            GitStasher::parse_branch_from_message("random message"),
            None
        );
    }

    #[test]
    fn test_format_stash_entry() {
        let entry = GitStashEntry {
            index: 0,
            oid: GitOid([0; 20]),
            message: "WIP on main: abc123".to_string(),
            committer: GitSignature::new("Test", "test@example.com"),
        };

        let formatted = format_stash_entry(&entry);
        assert_eq!(formatted, "stash@{0}: WIP on main: abc123");
    }

    #[test]
    fn test_has_stash_helper() {
        let (_dir, repo) = setup_test_repo();
        assert!(!has_stash(&repo).unwrap());
    }

    #[test]
    fn test_stash_count_helper() {
        let (_dir, repo) = setup_test_repo();
        assert_eq!(stash_count(&repo).unwrap(), 0);
    }

    #[test]
    fn test_drop_invalid_index() {
        let (_dir, repo) = setup_test_repo();
        let stasher = GitStasher::new(&repo);

        let result = stasher.drop(0);
        assert!(result.is_err());
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 449: Status Checking
- Spec 450: Diff Generation
