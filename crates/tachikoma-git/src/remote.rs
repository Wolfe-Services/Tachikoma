//! Git remote management.

use crate::{GitOid, GitRef, GitRepository, GitResult, GitError};
use serde::{Deserialize, Serialize};

/// Remote repository information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRemote {
    /// Remote name.
    pub name: String,
    /// Fetch URL.
    pub fetch_url: Option<String>,
    /// Push URL (if different from fetch).
    pub push_url: Option<String>,
    /// Fetch refspecs.
    pub fetch_refspecs: Vec<String>,
    /// Push refspecs.
    pub push_refspecs: Vec<String>,
}

/// Remote branch information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBranch {
    /// Full reference name.
    pub ref_name: String,
    /// Short name.
    pub name: String,
    /// Remote name.
    pub remote: String,
    /// Current OID.
    pub oid: GitOid,
}

impl GitRepository {
    /// List all remotes.
    pub fn list_remotes(&self) -> GitResult<Vec<String>> {
        self.with_repo(|repo| {
            let remotes = repo.remotes()?;
            Ok(remotes.iter().filter_map(|r| r.map(String::from)).collect())
        })
    }

    /// Get remote information.
    pub fn get_remote(&self, name: &str) -> GitResult<GitRemote> {
        self.with_repo(|repo| {
            let remote = repo.find_remote(name)?;

            let fetch_url = remote.url().map(String::from);
            let push_url = remote.pushurl().map(String::from);

            let fetch_refspecs = remote.fetch_refspecs()?
                .iter()
                .filter_map(|r| r.map(String::from))
                .collect();

            let push_refspecs = remote.push_refspecs()?
                .iter()
                .filter_map(|r| r.map(String::from))
                .collect();

            Ok(GitRemote {
                name: name.to_string(),
                fetch_url,
                push_url,
                fetch_refspecs,
                push_refspecs,
            })
        })
    }

    /// Add a new remote.
    pub fn add_remote(&self, name: &str, url: &str) -> GitResult<GitRemote> {
        self.with_repo_mut(|repo| {
            repo.remote(name, url)?;
            Ok(())
        })?;

        self.get_remote(name)
    }

    /// Remove a remote.
    pub fn remove_remote(&self, name: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            repo.remote_delete(name)?;
            Ok(())
        })
    }

    /// Rename a remote.
    pub fn rename_remote(&self, old_name: &str, new_name: &str) -> GitResult<Vec<String>> {
        self.with_repo_mut(|repo| {
            let problems = repo.remote_rename(old_name, new_name)?;
            Ok(problems.iter().filter_map(|p| p.map(String::from)).collect())
        })
    }

    /// Set remote URL.
    pub fn set_remote_url(&self, name: &str, url: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            repo.remote_set_url(name, url)?;
            Ok(())
        })
    }

    /// Set remote push URL.
    pub fn set_remote_push_url(&self, name: &str, url: &str) -> GitResult<()> {
        self.with_repo_mut(|repo| {
            repo.remote_set_pushurl(name, Some(url))?;
            Ok(())
        })
    }

    /// Get remote branches.
    pub fn remote_branches(&self, remote: Option<&str>) -> GitResult<Vec<RemoteBranch>> {
        self.with_repo(|repo| {
            let mut branches = Vec::new();

            for reference in repo.references()? {
                let reference = reference?;

                if !reference.is_remote() {
                    continue;
                }

                let ref_name = match reference.name() {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                // Filter by remote if specified
                if let Some(remote_name) = remote {
                    if !ref_name.contains(&format!("/{}/", remote_name)) {
                        continue;
                    }
                }

                let name = reference.shorthand().unwrap_or("").to_string();
                let remote_name = ref_name
                    .strip_prefix("refs/remotes/")
                    .and_then(|s| s.split('/').next())
                    .unwrap_or("")
                    .to_string();

                let oid = reference.target().map(GitOid::from_git2).ok_or_else(|| {
                    GitError::RefNotFound { name: ref_name.clone() }
                })?;

                branches.push(RemoteBranch {
                    ref_name,
                    name,
                    remote: remote_name,
                    oid,
                });
            }

            Ok(branches)
        })
    }

    /// Get the default remote (usually "origin").
    pub fn default_remote(&self) -> GitResult<Option<String>> {
        let remotes = self.list_remotes()?;

        // Prefer "origin"
        if remotes.contains(&"origin".to_string()) {
            return Ok(Some("origin".to_string()));
        }

        // Return first remote
        Ok(remotes.into_iter().next())
    }

    /// Prune stale remote branches.
    pub fn prune_remote(&self, name: &str) -> GitResult<Vec<String>> {
        self.with_repo_mut(|repo| {
            let mut remote = repo.find_remote(name)?;
            let mut pruned = Vec::new();

            // Get list of remote refs before prune
            let refs_before: Vec<String> = repo.references()?
                .filter_map(|r| r.ok())
                .filter(|r| r.is_remote())
                .filter_map(|r| r.name().map(String::from))
                .filter(|n| n.contains(&format!("/{}/", name)))
                .collect();

            // Connect and prune
            remote.connect(git2::Direction::Fetch)?;
            let callbacks = git2::RemoteCallbacks::new();
            remote.prune(Some(callbacks))?;

            // Get list after prune
            let refs_after: Vec<String> = repo.references()?
                .filter_map(|r| r.ok())
                .filter(|r| r.is_remote())
                .filter_map(|r| r.name().map(String::from))
                .filter(|n| n.contains(&format!("/{}/", name)))
                .collect();

            // Find what was pruned
            for ref_name in refs_before {
                if !refs_after.contains(&ref_name) {
                    pruned.push(ref_name);
                }
            }

            Ok(pruned)
        })
    }
}