//! Git diff implementation.

use crate::{
    diff::{DiffFile, DiffHunk, DiffLine, DiffOptions, DiffStats, DiffStatus, GitDiff, LineOrigin},
    GitOid, GitRepository, GitResult, GitError,
};
use git2::{Diff, DiffOptions as Git2DiffOpts, DiffDelta, DiffHunk as Git2DiffHunk, DiffLine as Git2DiffLine};
use std::path::PathBuf;

impl GitRepository {
    /// Diff between working directory and index.
    pub fn diff_index_to_workdir(&self, options: DiffOptions) -> GitResult<GitDiff> {
        self.with_repo(|repo| {
            let mut opts = build_diff_options(&options);
            let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;
            parse_diff(&diff, &options)
        })
    }

    /// Diff between HEAD and index (staged changes).
    pub fn diff_head_to_index(&self, options: DiffOptions) -> GitResult<GitDiff> {
        self.with_repo(|repo| {
            let head = repo.head()?.peel_to_tree()?;
            let mut opts = build_diff_options(&options);
            let diff = repo.diff_tree_to_index(Some(&head), None, Some(&mut opts))?;
            parse_diff(&diff, &options)
        })
    }

    /// Diff between two commits.
    pub fn diff_commits(
        &self,
        old: &GitOid,
        new: &GitOid,
        options: DiffOptions,
    ) -> GitResult<GitDiff> {
        self.with_repo(|repo| {
            let old_commit = repo.find_commit(old.as_git2())?;
            let new_commit = repo.find_commit(new.as_git2())?;
            let old_tree = old_commit.tree()?;
            let new_tree = new_commit.tree()?;

            let mut opts = build_diff_options(&options);
            let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut opts))?;
            parse_diff(&diff, &options)
        })
    }

    /// Diff between HEAD and working directory (all changes).
    pub fn diff_head_to_workdir(&self, options: DiffOptions) -> GitResult<GitDiff> {
        self.with_repo(|repo| {
            let head = repo.head()?.peel_to_tree()?;
            let mut opts = build_diff_options(&options);
            let diff = repo.diff_tree_to_workdir_with_index(Some(&head), Some(&mut opts))?;
            parse_diff(&diff, &options)
        })
    }

    /// Get diff statistics only (faster than full diff).
    pub fn diff_stats(&self, old: Option<&GitOid>, new: Option<&GitOid>) -> GitResult<DiffStats> {
        self.with_repo(|repo| {
            let diff = match (old, new) {
                (Some(old), Some(new)) => {
                    let old_commit = repo.find_commit(old.as_git2())?;
                    let new_commit = repo.find_commit(new.as_git2())?;
                    repo.diff_tree_to_tree(
                        Some(&old_commit.tree()?),
                        Some(&new_commit.tree()?),
                        None,
                    )?
                }
                (None, None) => {
                    repo.diff_index_to_workdir(None, None)?
                }
                _ => {
                    return Err(GitError::InvalidOperation {
                        message: "Must specify both or neither commit".to_string(),
                    });
                }
            };

            let stats = diff.stats()?;
            Ok(DiffStats {
                files_changed: stats.files_changed(),
                insertions: stats.insertions(),
                deletions: stats.deletions(),
            })
        })
    }
}

fn build_diff_options(options: &DiffOptions) -> Git2DiffOpts {
    let mut opts = Git2DiffOpts::new();

    opts.context_lines(options.context_lines);

    if options.ignore_whitespace {
        opts.ignore_whitespace(true);
    }
    if options.ignore_whitespace_eol {
        opts.ignore_whitespace_eol(true);
    }

    for pathspec in &options.pathspecs {
        opts.pathspec(pathspec);
    }

    opts
}

fn parse_diff(diff: &Diff, options: &DiffOptions) -> GitResult<GitDiff> {
    let mut files = Vec::new();
    let mut total_stats = DiffStats::default();

    // Find renames if enabled
    let mut diff = diff.clone();
    if options.detect_renames {
        let mut find_opts = git2::DiffFindOptions::new();
        find_opts.renames(true);
        if options.detect_copies {
            find_opts.copies(true);
        }
        diff.find_similar(Some(&mut find_opts))?;
    }

    // Parse deltas
    for delta_idx in 0..diff.deltas().len() {
        let delta = diff.get_delta(delta_idx).unwrap();
        let mut file = parse_delta(&delta);

        // Parse hunks for this file
        diff.foreach(
            &mut |_, _| true,
            None,
            Some(&mut |d, h| {
                if d.new_file().path() == delta.new_file().path() {
                    if let Some(hunk) = parse_hunk(h) {
                        file.hunks.push(hunk);
                    }
                }
                true
            }),
            Some(&mut |d, h, l| {
                if d.new_file().path() == delta.new_file().path() {
                    if let Some(hunk) = file.hunks.last_mut() {
                        if let Some(line) = parse_line(l) {
                            // Update stats
                            match line.origin {
                                LineOrigin::Addition => file.stats.insertions += 1,
                                LineOrigin::Deletion => file.stats.deletions += 1,
                                _ => {}
                            }
                            hunk.lines.push(line);
                        }
                    }
                }
                true
            }),
        )?;

        total_stats.files_changed += 1;
        total_stats.insertions += file.stats.insertions;
        total_stats.deletions += file.stats.deletions;

        files.push(file);
    }

    Ok(GitDiff {
        files,
        stats: total_stats,
    })
}

fn parse_delta(delta: &DiffDelta) -> DiffFile {
    let status = match delta.status() {
        git2::Delta::Added => DiffStatus::Added,
        git2::Delta::Deleted => DiffStatus::Deleted,
        git2::Delta::Modified => DiffStatus::Modified,
        git2::Delta::Renamed => DiffStatus::Renamed,
        git2::Delta::Copied => DiffStatus::Copied,
        git2::Delta::TypeChange => DiffStatus::TypeChange,
        git2::Delta::Untracked => DiffStatus::Untracked,
        git2::Delta::Ignored => DiffStatus::Ignored,
        git2::Delta::Conflicted => DiffStatus::Conflicted,
        _ => DiffStatus::Modified,
    };

    let old_file = delta.old_file();
    let new_file = delta.new_file();

    DiffFile {
        old_path: old_file.path().map(PathBuf::from),
        new_path: new_file.path().map(PathBuf::from).unwrap_or_default(),
        status,
        old_oid: if old_file.id().is_zero() { None } else { Some(GitOid::from_git2(old_file.id())) },
        new_oid: if new_file.id().is_zero() { None } else { Some(GitOid::from_git2(new_file.id())) },
        is_binary: old_file.is_binary() || new_file.is_binary(),
        mode_changed: old_file.mode() != new_file.mode(),
        old_mode: Some(old_file.mode()),
        new_mode: Some(new_file.mode()),
        hunks: Vec::new(),
        stats: DiffStats::default(),
    }
}

fn parse_hunk(hunk: &Git2DiffHunk) -> Option<DiffHunk> {
    Some(DiffHunk {
        old_start: hunk.old_start(),
        old_lines: hunk.old_lines(),
        new_start: hunk.new_start(),
        new_lines: hunk.new_lines(),
        header: String::from_utf8_lossy(hunk.header()).to_string(),
        lines: Vec::new(),
    })
}

fn parse_line(line: &Git2DiffLine) -> Option<DiffLine> {
    let origin = match line.origin() {
        ' ' => LineOrigin::Context,
        '+' => LineOrigin::Addition,
        '-' => LineOrigin::Deletion,
        '=' => LineOrigin::ContextEofnl,
        '>' => LineOrigin::AddEofnl,
        '<' => LineOrigin::DelEofnl,
        'F' => LineOrigin::FileHeader,
        'H' => LineOrigin::HunkHeader,
        'B' => LineOrigin::Binary,
        _ => return None,
    };

    Some(DiffLine {
        origin,
        content: String::from_utf8_lossy(line.content()).to_string(),
        old_lineno: line.old_lineno(),
        new_lineno: line.new_lineno(),
    })
}