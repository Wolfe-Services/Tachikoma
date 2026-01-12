//! Git Operations - Auto-commit functionality for Ralph loop
//!
//! Commits changes after each successful spec implementation.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Check if directory is a git repository
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Initialize a git repository if not already one
pub fn init_repo(path: &Path) -> Result<()> {
    if is_git_repo(path) {
        return Ok(());
    }

    let output = Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .context("Failed to run git init")?;

    if !output.status.success() {
        anyhow::bail!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Get current git status (short format)
pub fn status(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(path)
        .output()
        .context("Failed to run git status")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Check if there are uncommitted changes
pub fn has_changes(path: &Path) -> Result<bool> {
    let status = status(path)?;
    Ok(!status.trim().is_empty())
}

/// Stage all changes
pub fn add_all(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(path)
        .output()
        .context("Failed to run git add")?;

    if !output.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Commit staged changes with a message
pub fn commit(path: &Path, message: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .context("Failed to run git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "nothing to commit" is not really an error
        if stderr.contains("nothing to commit") {
            return Ok("Nothing to commit".to_string());
        }
        anyhow::bail!("git commit failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get the current commit hash (short)
pub fn current_commit_short(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .context("Failed to get current commit")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get commit hash");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the current branch name
pub fn current_branch(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        // Might be in detached HEAD state
        return Ok("(detached)".to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Auto-commit changes for a completed spec
pub fn auto_commit_spec(path: &Path, spec_id: u32, spec_name: &str) -> Result<Option<String>> {
    if !has_changes(path)? {
        tracing::info!("No changes to commit for spec {}", spec_id);
        return Ok(None);
    }

    // Stage all changes
    add_all(path)?;

    // Create commit message
    let message = format!(
        "spec({}): implement {}\n\nAutomated commit by Ralph loop.\nSpec: {:03}-{}",
        spec_id, spec_name, spec_id, spec_name.to_lowercase().replace(' ', "-")
    );

    let result = commit(path, &message)?;

    // Get commit hash for confirmation
    let hash = current_commit_short(path).unwrap_or_else(|_| "unknown".to_string());

    tracing::info!("Committed spec {} as {}", spec_id, hash);

    Ok(Some(hash))
}

/// Get recent commits (for context in prompts)
pub fn recent_commits(path: &Path, count: usize) -> Result<Vec<CommitInfo>> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-{}", count),
            "--pretty=format:%H|%h|%s|%an|%ar",
        ])
        .current_dir(path)
        .output()
        .context("Failed to get recent commits")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<_> = line.split('|').collect();
            if parts.len() >= 5 {
                Some(CommitInfo {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    subject: parts[2].to_string(),
                    author: parts[3].to_string(),
                    relative_time: parts[4].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(commits)
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub author: String,
    pub relative_time: String,
}

impl std::fmt::Display for CommitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({}) - {}",
            self.short_hash, self.subject, self.relative_time, self.author
        )
    }
}

/// Create a branch for the current spec work
pub fn create_spec_branch(path: &Path, spec_id: u32, spec_name: &str) -> Result<String> {
    let branch_name = format!(
        "spec/{:03}-{}",
        spec_id,
        spec_name.to_lowercase().replace(' ', "-").replace('_', "-")
    );

    // Check if branch exists
    let check = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch_name)])
        .current_dir(path)
        .output()?;

    if check.status.success() {
        // Branch exists, check it out
        let output = Command::new("git")
            .args(["checkout", &branch_name])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to checkout existing branch {}", branch_name);
        }
    } else {
        // Create and checkout new branch
        let output = Command::new("git")
            .args(["checkout", "-b", &branch_name])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to create branch {}: {}",
                branch_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(branch_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_git_repo() {
        let temp = TempDir::new().unwrap();
        assert!(!is_git_repo(temp.path()));

        fs::create_dir(temp.path().join(".git")).unwrap();
        assert!(is_git_repo(temp.path()));
    }
}
