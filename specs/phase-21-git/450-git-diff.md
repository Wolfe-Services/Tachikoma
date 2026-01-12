# Spec 450: Diff Generation

## Phase
21 - Git Integration

## Spec ID
450

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)

## Estimated Context
~11%

---

## Objective

Implement comprehensive Git diff generation for Tachikoma, providing the ability to compare commits, trees, the working directory, and the index. This module supports various diff formats including unified diff, side-by-side, and structured representations suitable for AI-assisted code review and analysis.

---

## Acceptance Criteria

- [ ] Implement `GitDiff` struct for diff representation
- [ ] Support diff between commits, trees, index, and workdir
- [ ] Generate unified diff format output
- [ ] Support diff statistics (insertions, deletions)
- [ ] Implement diff filtering by path and file type
- [ ] Support rename and copy detection
- [ ] Implement word-level diff (for fine-grained changes)
- [ ] Support binary file detection
- [ ] Implement diff patch application
- [ ] Provide structured diff output for AI analysis

---

## Implementation Details

### Diff Manager Implementation

```rust
// src/git/diff.rs

use git2::{
    Diff, DiffDelta, DiffFile, DiffFormat, DiffHunk, DiffLine, DiffOptions,
    DiffFindOptions, Repository,
};
use std::path::{Path, PathBuf};
use std::fmt;

use super::repo::GitRepository;
use super::types::*;

/// Diff comparison targets
#[derive(Debug, Clone)]
pub enum DiffTarget {
    /// The index (staging area)
    Index,
    /// The working directory
    Workdir,
    /// A specific commit
    Commit(GitOid),
    /// HEAD commit
    Head,
    /// A tree object
    Tree(GitOid),
}

/// Options for diff generation
#[derive(Debug, Clone)]
pub struct DiffGenerationOptions {
    /// Number of context lines
    pub context_lines: u32,
    /// Detect renames
    pub detect_renames: bool,
    /// Rename threshold (0-100)
    pub rename_threshold: u16,
    /// Detect copies
    pub detect_copies: bool,
    /// Include untracked files
    pub include_untracked: bool,
    /// Ignore whitespace changes
    pub ignore_whitespace: bool,
    /// Ignore whitespace at end of line
    pub ignore_whitespace_eol: bool,
    /// Treat all files as text
    pub force_text: bool,
    /// Path filters
    pub pathspecs: Vec<PathBuf>,
    /// Include function context in hunks
    pub show_function_context: bool,
}

impl Default for DiffGenerationOptions {
    fn default() -> Self {
        Self {
            context_lines: 3,
            detect_renames: true,
            rename_threshold: 50,
            detect_copies: false,
            include_untracked: false,
            ignore_whitespace: false,
            ignore_whitespace_eol: false,
            force_text: false,
            pathspecs: Vec::new(),
            show_function_context: true,
        }
    }
}

impl DiffGenerationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn context_lines(mut self, lines: u32) -> Self {
        self.context_lines = lines;
        self
    }

    pub fn detect_renames(mut self, detect: bool) -> Self {
        self.detect_renames = detect;
        self
    }

    pub fn ignore_whitespace(mut self, ignore: bool) -> Self {
        self.ignore_whitespace = ignore;
        self
    }

    pub fn pathspec(mut self, path: impl Into<PathBuf>) -> Self {
        self.pathspecs.push(path.into());
        self
    }
}

/// Statistics for a diff
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    /// Number of files changed
    pub files_changed: usize,
    /// Number of insertions
    pub insertions: usize,
    /// Number of deletions
    pub deletions: usize,
}

impl DiffStats {
    /// Total number of lines changed
    pub fn total_lines(&self) -> usize {
        self.insertions + self.deletions
    }
}

impl fmt::Display for DiffStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} file(s) changed, {} insertion(s)(+), {} deletion(s)(-)",
            self.files_changed, self.insertions, self.deletions
        )
    }
}

/// A single line in a diff hunk
#[derive(Debug, Clone)]
pub struct GitDiffLine {
    /// Line content (without newline)
    pub content: String,
    /// Type of line
    pub origin: DiffLineOrigin,
    /// Old line number (if applicable)
    pub old_lineno: Option<u32>,
    /// New line number (if applicable)
    pub new_lineno: Option<u32>,
}

/// Line origin type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineOrigin {
    Context,
    Addition,
    Deletion,
    ContextEOFNL,
    AddEOFNL,
    DeleteEOFNL,
    FileHeader,
    HunkHeader,
    Binary,
}

impl From<char> for DiffLineOrigin {
    fn from(c: char) -> Self {
        match c {
            ' ' => Self::Context,
            '+' => Self::Addition,
            '-' => Self::Deletion,
            '=' => Self::ContextEOFNL,
            '>' => Self::AddEOFNL,
            '<' => Self::DeleteEOFNL,
            'F' => Self::FileHeader,
            'H' => Self::HunkHeader,
            'B' => Self::Binary,
            _ => Self::Context,
        }
    }
}

/// A hunk in a diff (a contiguous region of changes)
#[derive(Debug, Clone)]
pub struct GitDiffHunk {
    /// Header line (e.g., @@ -1,5 +1,6 @@)
    pub header: String,
    /// Old start line
    pub old_start: u32,
    /// Old line count
    pub old_lines: u32,
    /// New start line
    pub new_start: u32,
    /// New line count
    pub new_lines: u32,
    /// Lines in this hunk
    pub lines: Vec<GitDiffLine>,
}

impl GitDiffHunk {
    /// Get only the changed lines (additions and deletions)
    pub fn changed_lines(&self) -> Vec<&GitDiffLine> {
        self.lines
            .iter()
            .filter(|l| matches!(l.origin, DiffLineOrigin::Addition | DiffLineOrigin::Deletion))
            .collect()
    }

    /// Count additions in this hunk
    pub fn additions(&self) -> usize {
        self.lines.iter().filter(|l| l.origin == DiffLineOrigin::Addition).count()
    }

    /// Count deletions in this hunk
    pub fn deletions(&self) -> usize {
        self.lines.iter().filter(|l| l.origin == DiffLineOrigin::Deletion).count()
    }
}

/// A file entry in a diff
#[derive(Debug, Clone)]
pub struct GitDiffFile {
    /// Old path (before rename/copy)
    pub old_path: Option<PathBuf>,
    /// New path
    pub new_path: Option<PathBuf>,
    /// Delta type
    pub delta: GitDeltaKind,
    /// Old file OID
    pub old_oid: Option<GitOid>,
    /// New file OID
    pub new_oid: Option<GitOid>,
    /// Similarity (for renames/copies, 0-100)
    pub similarity: Option<u32>,
    /// Is binary file
    pub is_binary: bool,
    /// Hunks in this file
    pub hunks: Vec<GitDiffHunk>,
}

impl GitDiffFile {
    /// Get the primary path (new_path or old_path)
    pub fn path(&self) -> Option<&Path> {
        self.new_path.as_deref().or(self.old_path.as_deref())
    }

    /// Count total additions
    pub fn additions(&self) -> usize {
        self.hunks.iter().map(|h| h.additions()).sum()
    }

    /// Count total deletions
    pub fn deletions(&self) -> usize {
        self.hunks.iter().map(|h| h.deletions()).sum()
    }
}

/// Complete diff result
#[derive(Debug, Clone)]
pub struct GitDiff {
    /// Files in the diff
    pub files: Vec<GitDiffFile>,
    /// Statistics
    pub stats: DiffStats,
}

impl GitDiff {
    /// Check if diff is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get files matching a pattern
    pub fn files_matching(&self, pattern: &str) -> Vec<&GitDiffFile> {
        self.files
            .iter()
            .filter(|f| {
                f.path()
                    .map(|p| p.to_string_lossy().contains(pattern))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get only modified files
    pub fn modified_files(&self) -> Vec<&GitDiffFile> {
        self.files.iter().filter(|f| f.delta == GitDeltaKind::Modified).collect()
    }

    /// Get only added files
    pub fn added_files(&self) -> Vec<&GitDiffFile> {
        self.files.iter().filter(|f| f.delta == GitDeltaKind::Added).collect()
    }

    /// Get only deleted files
    pub fn deleted_files(&self) -> Vec<&GitDiffFile> {
        self.files.iter().filter(|f| f.delta == GitDeltaKind::Deleted).collect()
    }

    /// Format as unified diff string
    pub fn to_unified_diff(&self) -> String {
        let mut output = String::new();

        for file in &self.files {
            // File header
            let old_path = file.old_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "/dev/null".to_string());
            let new_path = file.new_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "/dev/null".to_string());

            output.push_str(&format!("diff --git a/{} b/{}\n", old_path, new_path));

            if file.is_binary {
                output.push_str("Binary files differ\n");
                continue;
            }

            output.push_str(&format!("--- a/{}\n", old_path));
            output.push_str(&format!("+++ b/{}\n", new_path));

            for hunk in &file.hunks {
                output.push_str(&hunk.header);
                if !hunk.header.ends_with('\n') {
                    output.push('\n');
                }

                for line in &hunk.lines {
                    let prefix = match line.origin {
                        DiffLineOrigin::Context => ' ',
                        DiffLineOrigin::Addition => '+',
                        DiffLineOrigin::Deletion => '-',
                        _ => ' ',
                    };
                    output.push(prefix);
                    output.push_str(&line.content);
                    if !line.content.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
        }

        output
    }
}

/// Git diff generator
pub struct GitDiffGenerator<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitDiffGenerator<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Generate diff between two targets
    pub fn diff(
        &self,
        old: DiffTarget,
        new: DiffTarget,
        options: &DiffGenerationOptions,
    ) -> GitResult<GitDiff> {
        let raw_repo = self.repo.raw();

        let mut diff_opts = self.build_diff_options(options);

        let diff = match (&old, &new) {
            (DiffTarget::Head, DiffTarget::Index) => {
                let head_tree = self.get_head_tree(raw_repo)?;
                raw_repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut diff_opts))?
            }
            (DiffTarget::Index, DiffTarget::Workdir) => {
                raw_repo.diff_index_to_workdir(None, Some(&mut diff_opts))?
            }
            (DiffTarget::Head, DiffTarget::Workdir) => {
                let head_tree = self.get_head_tree(raw_repo)?;
                raw_repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut diff_opts))?
            }
            (DiffTarget::Commit(old_oid), DiffTarget::Commit(new_oid)) => {
                let old_tree = self.get_commit_tree(raw_repo, old_oid)?;
                let new_tree = self.get_commit_tree(raw_repo, new_oid)?;
                raw_repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut diff_opts))?
            }
            (DiffTarget::Commit(oid), DiffTarget::Index) => {
                let old_tree = self.get_commit_tree(raw_repo, oid)?;
                raw_repo.diff_tree_to_index(Some(&old_tree), None, Some(&mut diff_opts))?
            }
            _ => return Err(GitError::Other("Unsupported diff target combination".into())),
        };

        // Apply rename/copy detection if enabled
        let diff = if options.detect_renames || options.detect_copies {
            let mut find_opts = DiffFindOptions::new();
            find_opts.renames(options.detect_renames);
            find_opts.copies(options.detect_copies);
            find_opts.rename_threshold(options.rename_threshold);

            let mut diff = diff;
            diff.find_similar(Some(&mut find_opts))?;
            diff
        } else {
            diff
        };

        self.convert_diff(diff)
    }

    /// Generate diff for a single file
    pub fn diff_file(
        &self,
        path: &Path,
        old: DiffTarget,
        new: DiffTarget,
    ) -> GitResult<Option<GitDiffFile>> {
        let options = DiffGenerationOptions::default().pathspec(path);
        let diff = self.diff(old, new, &options)?;
        Ok(diff.files.into_iter().next())
    }

    /// Generate diff for staged changes (HEAD to index)
    pub fn staged_diff(&self, options: &DiffGenerationOptions) -> GitResult<GitDiff> {
        self.diff(DiffTarget::Head, DiffTarget::Index, options)
    }

    /// Generate diff for unstaged changes (index to workdir)
    pub fn unstaged_diff(&self, options: &DiffGenerationOptions) -> GitResult<GitDiff> {
        self.diff(DiffTarget::Index, DiffTarget::Workdir, options)
    }

    /// Generate diff for all uncommitted changes
    pub fn uncommitted_diff(&self, options: &DiffGenerationOptions) -> GitResult<GitDiff> {
        self.diff(DiffTarget::Head, DiffTarget::Workdir, options)
    }

    fn build_diff_options(&self, options: &DiffGenerationOptions) -> DiffOptions {
        let mut diff_opts = DiffOptions::new();
        diff_opts.context_lines(options.context_lines);
        diff_opts.include_untracked(options.include_untracked);
        diff_opts.ignore_whitespace(options.ignore_whitespace);
        diff_opts.ignore_whitespace_eol(options.ignore_whitespace_eol);
        diff_opts.force_text(options.force_text);

        for path in &options.pathspecs {
            diff_opts.pathspec(path);
        }

        diff_opts
    }

    fn get_head_tree(&self, repo: &Repository) -> GitResult<Option<git2::Tree>> {
        match repo.head() {
            Ok(head) => {
                let tree = head.peel_to_tree()?;
                Ok(Some(tree))
            }
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => Ok(None),
            Err(e) => Err(GitError::Git2(e)),
        }
    }

    fn get_commit_tree(&self, repo: &Repository, oid: &GitOid) -> GitResult<git2::Tree> {
        let commit = repo.find_commit(oid.to_git2_oid())?;
        Ok(commit.tree()?)
    }

    fn convert_diff(&self, diff: Diff) -> GitResult<GitDiff> {
        let stats = diff.stats()?;
        let diff_stats = DiffStats {
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        };

        let mut files = Vec::new();
        let num_deltas = diff.deltas().len();

        for delta_idx in 0..num_deltas {
            let delta = diff.get_delta(delta_idx).unwrap();

            let old_file = delta.old_file();
            let new_file = delta.new_file();

            let mut diff_file = GitDiffFile {
                old_path: old_file.path().map(PathBuf::from),
                new_path: new_file.path().map(PathBuf::from),
                delta: GitDeltaKind::from(delta.status()),
                old_oid: if old_file.id().is_zero() { None } else { Some(GitOid::from(old_file.id())) },
                new_oid: if new_file.id().is_zero() { None } else { Some(GitOid::from(new_file.id())) },
                similarity: if delta.status() == git2::Delta::Renamed || delta.status() == git2::Delta::Copied {
                    Some(delta.similarity() as u32)
                } else {
                    None
                },
                is_binary: delta.flags().is_binary(),
                hunks: Vec::new(),
            };

            // Collect hunks using the print callback
            let mut current_hunk: Option<GitDiffHunk> = None;

            diff.print(DiffFormat::Patch, |delta_inner, hunk, line| {
                // Only process for current delta
                if delta_inner.old_file().path() != delta.old_file().path() {
                    return true;
                }

                if let Some(hunk_data) = hunk {
                    // Save previous hunk
                    if let Some(h) = current_hunk.take() {
                        diff_file.hunks.push(h);
                    }

                    // Start new hunk
                    current_hunk = Some(GitDiffHunk {
                        header: String::from_utf8_lossy(hunk_data.header()).to_string(),
                        old_start: hunk_data.old_start(),
                        old_lines: hunk_data.old_lines(),
                        new_start: hunk_data.new_start(),
                        new_lines: hunk_data.new_lines(),
                        lines: Vec::new(),
                    });
                }

                if let Some(ref mut h) = current_hunk {
                    let content = String::from_utf8_lossy(line.content()).to_string();
                    h.lines.push(GitDiffLine {
                        content,
                        origin: DiffLineOrigin::from(line.origin()),
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                    });
                }

                true
            })?;

            // Don't forget the last hunk
            if let Some(h) = current_hunk {
                diff_file.hunks.push(h);
            }

            files.push(diff_file);
        }

        Ok(GitDiff {
            files,
            stats: diff_stats,
        })
    }
}

/// Helper function to get a quick diff summary
pub fn quick_diff_stats(repo: &GitRepository, old: DiffTarget, new: DiffTarget) -> GitResult<DiffStats> {
    let generator = GitDiffGenerator::new(repo);
    let diff = generator.diff(old, new, &DiffGenerationOptions::default())?;
    Ok(diff.stats)
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
        (dir, repo)
    }

    fn create_initial_commit(dir: &TempDir, repo: &GitRepository) {
        std::fs::write(dir.path().join("file.txt"), "line 1\nline 2\nline 3\n").unwrap();
        repo.stage_file(Path::new("file.txt")).unwrap();

        let raw = repo.raw();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = raw.index().unwrap().write_tree().unwrap();
        let tree = raw.find_tree(tree_id).unwrap();
        raw.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
    }

    #[test]
    fn test_diff_empty_repo() {
        let (_dir, repo) = setup_test_repo();
        let generator = GitDiffGenerator::new(&repo);

        let diff = generator.staged_diff(&DiffGenerationOptions::default()).unwrap();
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_staged_changes() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        // Create and stage a new file
        std::fs::write(dir.path().join("new.txt"), "new content").unwrap();
        repo.stage_file(Path::new("new.txt")).unwrap();

        let generator = GitDiffGenerator::new(&repo);
        let diff = generator.staged_diff(&DiffGenerationOptions::default()).unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].delta, GitDeltaKind::Added);
    }

    #[test]
    fn test_diff_modified_file() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        // Modify the file
        std::fs::write(dir.path().join("file.txt"), "line 1\nmodified\nline 3\n").unwrap();

        let generator = GitDiffGenerator::new(&repo);
        let diff = generator.unstaged_diff(&DiffGenerationOptions::default()).unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].delta, GitDeltaKind::Modified);
        assert!(diff.stats.insertions > 0 || diff.stats.deletions > 0);
    }

    #[test]
    fn test_diff_unified_output() {
        let (dir, repo) = setup_test_repo();
        create_initial_commit(&dir, &repo);

        std::fs::write(dir.path().join("file.txt"), "modified content\n").unwrap();

        let generator = GitDiffGenerator::new(&repo);
        let diff = generator.unstaged_diff(&DiffGenerationOptions::default()).unwrap();

        let unified = diff.to_unified_diff();
        assert!(unified.contains("diff --git"));
        assert!(unified.contains("---"));
        assert!(unified.contains("+++"));
    }

    #[test]
    fn test_diff_stats() {
        let stats = DiffStats {
            files_changed: 3,
            insertions: 10,
            deletions: 5,
        };

        assert_eq!(stats.total_lines(), 15);
        assert!(stats.to_string().contains("3 file"));
        assert!(stats.to_string().contains("10 insertion"));
    }

    #[test]
    fn test_diff_hunk_counts() {
        let hunk = GitDiffHunk {
            header: "@@ -1,3 +1,4 @@".to_string(),
            old_start: 1,
            old_lines: 3,
            new_start: 1,
            new_lines: 4,
            lines: vec![
                GitDiffLine {
                    content: "context".to_string(),
                    origin: DiffLineOrigin::Context,
                    old_lineno: Some(1),
                    new_lineno: Some(1),
                },
                GitDiffLine {
                    content: "deleted".to_string(),
                    origin: DiffLineOrigin::Deletion,
                    old_lineno: Some(2),
                    new_lineno: None,
                },
                GitDiffLine {
                    content: "added".to_string(),
                    origin: DiffLineOrigin::Addition,
                    old_lineno: None,
                    new_lineno: Some(2),
                },
            ],
        };

        assert_eq!(hunk.additions(), 1);
        assert_eq!(hunk.deletions(), 1);
        assert_eq!(hunk.changed_lines().len(), 2);
    }

    #[test]
    fn test_diff_generation_options() {
        let opts = DiffGenerationOptions::new()
            .context_lines(5)
            .ignore_whitespace(true)
            .pathspec("src/");

        assert_eq!(opts.context_lines, 5);
        assert!(opts.ignore_whitespace);
        assert_eq!(opts.pathspecs.len(), 1);
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 449: Status Checking
- Spec 451: Commit Operations
