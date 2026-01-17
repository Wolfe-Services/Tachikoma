//! Git status implementation.

use crate::{
    status::{FileStatus, RepoStatus, StatusEntry, StatusOptions, StatusSummary},
    GitOid, GitRepository, GitResult,
};
use git2::{Status, StatusOptions as Git2StatusOpts};
use std::path::PathBuf;

impl GitRepository {
    /// Get repository status.
    pub fn status(&self, options: StatusOptions) -> GitResult<RepoStatus> {
        self.with_repo(|repo| {
            // Get branch info
            let (branch, head, upstream, ahead, behind) = get_branch_info(repo)?;

            // Get status entries
            let entries = get_status_entries(repo, &options)?;

            // Check in-progress operations
            let state = repo.state();

            Ok(RepoStatus {
                branch,
                head,
                upstream,
                ahead,
                behind,
                entries,
                is_merging: state == git2::RepositoryState::Merge,
                is_rebasing: matches!(
                    state,
                    git2::RepositoryState::Rebase
                        | git2::RepositoryState::RebaseInteractive
                        | git2::RepositoryState::RebaseMerge
                ),
                is_cherry_picking: state == git2::RepositoryState::CherryPick
                    || state == git2::RepositoryState::CherryPickSequence,
                is_reverting: state == git2::RepositoryState::Revert
                    || state == git2::RepositoryState::RevertSequence,
                is_bisecting: state == git2::RepositoryState::Bisect,
            })
        })
    }

    /// Get a quick status summary (faster than full status).
    pub fn status_quick(&self) -> GitResult<StatusSummary> {
        let status = self.status(StatusOptions::standard())?;
        Ok(status.summary())
    }

    /// Check if working directory is clean.
    pub fn is_clean(&self) -> GitResult<bool> {
        let status = self.status(StatusOptions {
            include_untracked: false,
            include_ignored: false,
            include_submodules: false,
            detect_renames: false,
            pathspecs: Vec::new(),
        })?;
        Ok(status.is_clean())
    }
}

fn get_branch_info(
    repo: &git2::Repository,
) -> GitResult<(Option<String>, Option<GitOid>, Option<String>, u32, u32)> {
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => {
            return Ok((None, None, None, 0, 0));
        }
    };

    let branch = head.shorthand().map(String::from);
    let head_oid = head.target().map(GitOid::from_git2);

    // Get upstream info
    let (upstream, ahead, behind) = if let Some(branch_name) = head.shorthand() {
        if let Ok(local_branch) = repo.find_branch(branch_name, git2::BranchType::Local) {
            if let Ok(upstream_branch) = local_branch.upstream() {
                let upstream_name = upstream_branch.name()?.map(String::from);

                let (ahead, behind) = if let (Some(local_oid), Ok(upstream_ref)) =
                    (head.target(), upstream_branch.into_reference().target())
                {
                    repo.graph_ahead_behind(local_oid, upstream_ref)
                        .unwrap_or((0, 0))
                } else {
                    (0, 0)
                };

                (upstream_name, ahead as u32, behind as u32)
            } else {
                (None, 0, 0)
            }
        } else {
            (None, 0, 0)
        }
    } else {
        (None, 0, 0)
    };

    Ok((branch, head_oid, upstream, ahead, behind))
}

fn get_status_entries(
    repo: &git2::Repository,
    options: &StatusOptions,
) -> GitResult<Vec<StatusEntry>> {
    let mut git_opts = Git2StatusOpts::new();

    if options.include_untracked {
        git_opts.include_untracked(true);
    }
    if options.include_ignored {
        git_opts.include_ignored(true);
    }
    if options.detect_renames {
        git_opts.renames_head_to_index(true);
        git_opts.renames_index_to_workdir(true);
    }

    for pathspec in &options.pathspecs {
        git_opts.pathspec(pathspec);
    }

    let statuses = repo.statuses(Some(&mut git_opts))?;
    let mut entries = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().map(PathBuf::from);
        if path.is_none() {
            continue;
        }
        let path = path.unwrap();

        let status = entry.status();
        let (index_status, worktree_status) = parse_status_flags(status);

        let orig_path = entry
            .head_to_index()
            .and_then(|d| d.old_file().path())
            .or_else(|| entry.index_to_workdir().and_then(|d| d.old_file().path()))
            .map(PathBuf::from);

        entries.push(StatusEntry {
            path,
            orig_path,
            index_status,
            worktree_status,
            is_binary: false, // Would need to check file content
        });
    }

    Ok(entries)
}

fn parse_status_flags(status: Status) -> (Option<FileStatus>, Option<FileStatus>) {
    let index_status = if status.is_index_new() {
        Some(FileStatus::New)
    } else if status.is_index_modified() {
        Some(FileStatus::Modified)
    } else if status.is_index_deleted() {
        Some(FileStatus::Deleted)
    } else if status.is_index_renamed() {
        Some(FileStatus::Renamed)
    } else if status.is_index_typechange() {
        Some(FileStatus::TypeChange)
    } else {
        None
    };

    let worktree_status = if status.is_wt_new() {
        Some(FileStatus::Untracked)
    } else if status.is_wt_modified() {
        Some(FileStatus::Modified)
    } else if status.is_wt_deleted() {
        Some(FileStatus::Deleted)
    } else if status.is_wt_renamed() {
        Some(FileStatus::Renamed)
    } else if status.is_wt_typechange() {
        Some(FileStatus::TypeChange)
    } else if status.is_ignored() {
        Some(FileStatus::Ignored)
    } else if status.is_conflicted() {
        Some(FileStatus::Conflicted)
    } else {
        None
    };

    (index_status, worktree_status)
}