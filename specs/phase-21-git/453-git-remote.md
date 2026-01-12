# Spec 453: Remote Operations

## Phase
21 - Git Integration

## Spec ID
453

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)
- Spec 463: Credential Management (authentication)

## Estimated Context
~9%

---

## Objective

Implement Git remote management for Tachikoma, providing functionality to configure, list, and interact with remote repositories. This module handles remote URLs, refspecs, and provides the foundation for push/pull operations.

---

## Acceptance Criteria

- [ ] Implement `GitRemoteManager` for remote operations
- [ ] Support adding, removing, and renaming remotes
- [ ] Support listing remotes with details
- [ ] Implement URL management (fetch/push URLs)
- [ ] Support refspec configuration
- [ ] Implement remote reference listing (ls-remote)
- [ ] Support prune operations
- [ ] Detect remote type (GitHub, GitLab, etc.)
- [ ] Support remote head detection
- [ ] Implement remote validation

---

## Implementation Details

### Remote Manager Implementation

```rust
// src/git/remote.rs

use git2::{Cred, Direction, FetchOptions, RemoteCallbacks, Repository};
use std::collections::HashMap;
use url::Url;

use super::repo::GitRepository;
use super::types::*;

/// Remote hosting provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteProvider {
    GitHub,
    GitLab,
    Bitbucket,
    AzureDevOps,
    Custom,
    Unknown,
}

impl RemoteProvider {
    pub fn from_url(url: &str) -> Self {
        let url_lower = url.to_lowercase();
        if url_lower.contains("github.com") {
            Self::GitHub
        } else if url_lower.contains("gitlab.com") || url_lower.contains("gitlab") {
            Self::GitLab
        } else if url_lower.contains("bitbucket.org") || url_lower.contains("bitbucket") {
            Self::Bitbucket
        } else if url_lower.contains("dev.azure.com") || url_lower.contains("visualstudio.com") {
            Self::AzureDevOps
        } else {
            Self::Unknown
        }
    }
}

/// Remote URL parsing result
#[derive(Debug, Clone)]
pub struct ParsedRemoteUrl {
    pub protocol: RemoteProtocol,
    pub host: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub provider: RemoteProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteProtocol {
    Https,
    Ssh,
    Git,
    File,
    Unknown,
}

impl ParsedRemoteUrl {
    pub fn parse(url: &str) -> Option<Self> {
        // SSH format: git@github.com:owner/repo.git
        if url.starts_with("git@") {
            let parts: Vec<&str> = url.trim_start_matches("git@").splitn(2, ':').collect();
            if parts.len() == 2 {
                let host = parts[0].to_string();
                let path = parts[1].trim_end_matches(".git");
                let path_parts: Vec<&str> = path.split('/').collect();

                return Some(Self {
                    protocol: RemoteProtocol::Ssh,
                    host: host.clone(),
                    owner: path_parts.first().map(|s| s.to_string()),
                    repo: path_parts.get(1).map(|s| s.to_string()),
                    provider: RemoteProvider::from_url(url),
                });
            }
        }

        // HTTPS/Git URL format
        if let Ok(parsed) = Url::parse(url) {
            let protocol = match parsed.scheme() {
                "https" => RemoteProtocol::Https,
                "http" => RemoteProtocol::Https,
                "git" => RemoteProtocol::Git,
                "file" => RemoteProtocol::File,
                "ssh" => RemoteProtocol::Ssh,
                _ => RemoteProtocol::Unknown,
            };

            let host = parsed.host_str().unwrap_or("").to_string();
            let path = parsed.path().trim_start_matches('/').trim_end_matches(".git");
            let path_parts: Vec<&str> = path.split('/').collect();

            return Some(Self {
                protocol,
                host,
                owner: path_parts.first().map(|s| s.to_string()),
                repo: path_parts.get(1).map(|s| s.to_string()),
                provider: RemoteProvider::from_url(url),
            });
        }

        None
    }

    /// Generate HTTPS URL from parsed info
    pub fn to_https_url(&self) -> Option<String> {
        match (&self.owner, &self.repo) {
            (Some(owner), Some(repo)) => {
                Some(format!("https://{}/{}/{}.git", self.host, owner, repo))
            }
            _ => None,
        }
    }

    /// Generate SSH URL from parsed info
    pub fn to_ssh_url(&self) -> Option<String> {
        match (&self.owner, &self.repo) {
            (Some(owner), Some(repo)) => {
                Some(format!("git@{}:{}/{}.git", self.host, owner, repo))
            }
            _ => None,
        }
    }
}

/// Remote reference (from ls-remote)
#[derive(Debug, Clone)]
pub struct RemoteRef {
    pub name: String,
    pub oid: GitOid,
    pub is_head: bool,
    pub is_branch: bool,
    pub is_tag: bool,
}

/// Options for ls-remote
#[derive(Debug, Clone, Default)]
pub struct LsRemoteOptions {
    /// Only show heads (branches)
    pub heads: bool,
    /// Only show tags
    pub tags: bool,
    /// Pattern to filter refs
    pub pattern: Option<String>,
}

impl LsRemoteOptions {
    pub fn heads() -> Self {
        Self {
            heads: true,
            tags: false,
            pattern: None,
        }
    }

    pub fn tags() -> Self {
        Self {
            heads: false,
            tags: true,
            pattern: None,
        }
    }
}

/// Git remote manager
pub struct GitRemoteManager<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitRemoteManager<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Add a new remote
    pub fn add(&self, name: &str, url: &str) -> GitResult<GitRemote> {
        let raw_repo = self.repo.raw();

        // Validate URL
        if ParsedRemoteUrl::parse(url).is_none() {
            return Err(GitError::InvalidConfig(format!("Invalid remote URL: {}", url)));
        }

        let remote = raw_repo.remote(name, url)?;
        self.remote_to_git_remote(&remote)
    }

    /// Add remote with separate fetch and push URLs
    pub fn add_with_push_url(&self, name: &str, fetch_url: &str, push_url: &str) -> GitResult<GitRemote> {
        let raw_repo = self.repo.raw();

        raw_repo.remote(name, fetch_url)?;
        raw_repo.remote_set_pushurl(name, Some(push_url))?;

        self.get(name)
    }

    /// Remove a remote
    pub fn remove(&self, name: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        raw_repo.remote_delete(name)?;
        Ok(())
    }

    /// Rename a remote
    pub fn rename(&self, old_name: &str, new_name: &str) -> GitResult<Vec<String>> {
        let raw_repo = self.repo.raw();
        let problems = raw_repo.remote_rename(old_name, new_name)?;
        Ok(problems.iter().filter_map(|s| s.map(String::from)).collect())
    }

    /// Get a remote by name
    pub fn get(&self, name: &str) -> GitResult<GitRemote> {
        let raw_repo = self.repo.raw();
        let remote = raw_repo.find_remote(name)?;
        self.remote_to_git_remote(&remote)
    }

    /// List all remotes
    pub fn list(&self) -> GitResult<Vec<GitRemote>> {
        let raw_repo = self.repo.raw();
        let remote_names = raw_repo.remotes()?;

        let mut remotes = Vec::new();
        for name in remote_names.iter().flatten() {
            if let Ok(remote) = self.get(name) {
                remotes.push(remote);
            }
        }

        Ok(remotes)
    }

    /// Set the URL for a remote
    pub fn set_url(&self, name: &str, url: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        raw_repo.remote_set_url(name, url)?;
        Ok(())
    }

    /// Set the push URL for a remote
    pub fn set_push_url(&self, name: &str, url: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        raw_repo.remote_set_pushurl(name, Some(url))?;
        Ok(())
    }

    /// Add a fetch refspec
    pub fn add_fetch_refspec(&self, name: &str, refspec: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        raw_repo.remote_add_fetch(name, refspec)?;
        Ok(())
    }

    /// Add a push refspec
    pub fn add_push_refspec(&self, name: &str, refspec: &str) -> GitResult<()> {
        let raw_repo = self.repo.raw();
        raw_repo.remote_add_push(name, refspec)?;
        Ok(())
    }

    /// List remote references (ls-remote)
    pub fn ls_remote(&self, name: &str, options: &LsRemoteOptions) -> GitResult<Vec<RemoteRef>> {
        let raw_repo = self.repo.raw();
        let mut remote = raw_repo.find_remote(name)?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username, allowed| {
            // Try SSH key from agent
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                if let Some(username) = username {
                    return Cred::ssh_key_from_agent(username);
                }
            }
            Cred::default()
        });

        // Connect to remote
        remote.connect_auth(Direction::Fetch, Some(callbacks), None)?;

        let refs: Vec<RemoteRef> = remote
            .list()?
            .iter()
            .filter_map(|head| {
                let name = head.name().to_string();
                let oid = GitOid::from(head.oid());

                let is_head = name == "HEAD";
                let is_branch = name.starts_with("refs/heads/");
                let is_tag = name.starts_with("refs/tags/");

                // Apply filters
                if options.heads && !is_branch && !is_head {
                    return None;
                }
                if options.tags && !is_tag {
                    return None;
                }
                if let Some(ref pattern) = options.pattern {
                    if !name.contains(pattern) {
                        return None;
                    }
                }

                Some(RemoteRef {
                    name,
                    oid,
                    is_head,
                    is_branch,
                    is_tag,
                })
            })
            .collect();

        remote.disconnect()?;
        Ok(refs)
    }

    /// Get the default branch of a remote
    pub fn default_branch(&self, name: &str) -> GitResult<Option<String>> {
        let raw_repo = self.repo.raw();
        let mut remote = raw_repo.find_remote(name)?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username, allowed| {
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                if let Some(username) = username {
                    return Cred::ssh_key_from_agent(username);
                }
            }
            Cred::default()
        });

        remote.connect_auth(Direction::Fetch, Some(callbacks), None)?;

        let default = remote.default_branch()?;
        let branch_name = default.as_str().map(|s| {
            s.strip_prefix("refs/heads/").unwrap_or(s).to_string()
        });

        remote.disconnect()?;
        Ok(branch_name)
    }

    /// Prune stale remote-tracking branches
    pub fn prune(&self, name: &str) -> GitResult<Vec<String>> {
        let raw_repo = self.repo.raw();
        let mut remote = raw_repo.find_remote(name)?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username, allowed| {
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                if let Some(username) = username {
                    return Cred::ssh_key_from_agent(username);
                }
            }
            Cred::default()
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        fetch_opts.prune(git2::FetchPrune::On);

        // Fetch with prune to get the list of pruned refs
        // Note: This actually does the prune, not just lists
        remote.fetch(&[] as &[&str], Some(&mut fetch_opts), None)?;

        // Return list of pruned refs (would need to track these during fetch)
        Ok(Vec::new())
    }

    /// Check if remote exists
    pub fn exists(&self, name: &str) -> bool {
        self.repo.raw().find_remote(name).is_ok()
    }

    /// Get parsed URL info for a remote
    pub fn parse_url(&self, name: &str) -> GitResult<Option<ParsedRemoteUrl>> {
        let remote = self.get(name)?;
        Ok(remote.url.and_then(|u| ParsedRemoteUrl::parse(&u)))
    }

    fn remote_to_git_remote(&self, remote: &git2::Remote) -> GitResult<GitRemote> {
        Ok(GitRemote {
            name: remote.name().unwrap_or("").to_string(),
            url: remote.url().map(String::from),
            push_url: remote.pushurl().map(String::from),
            fetch_refspecs: remote
                .fetch_refspecs()?
                .iter()
                .filter_map(|s| s.map(String::from))
                .collect(),
            push_refspecs: remote
                .push_refspecs()?
                .iter()
                .filter_map(|s| s.map(String::from))
                .collect(),
        })
    }
}

/// Remote URL converter utilities
pub struct RemoteUrlConverter;

impl RemoteUrlConverter {
    /// Convert SSH URL to HTTPS
    pub fn ssh_to_https(url: &str) -> Option<String> {
        ParsedRemoteUrl::parse(url).and_then(|p| p.to_https_url())
    }

    /// Convert HTTPS URL to SSH
    pub fn https_to_ssh(url: &str) -> Option<String> {
        ParsedRemoteUrl::parse(url).and_then(|p| p.to_ssh_url())
    }

    /// Normalize URL (ensure .git suffix)
    pub fn normalize(url: &str) -> String {
        if url.ends_with(".git") {
            url.to_string()
        } else {
            format!("{}.git", url.trim_end_matches('/'))
        }
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

    fn setup_test_repo() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_parse_ssh_url() {
        let parsed = ParsedRemoteUrl::parse("git@github.com:owner/repo.git").unwrap();

        assert_eq!(parsed.protocol, RemoteProtocol::Ssh);
        assert_eq!(parsed.host, "github.com");
        assert_eq!(parsed.owner, Some("owner".to_string()));
        assert_eq!(parsed.repo, Some("repo".to_string()));
        assert_eq!(parsed.provider, RemoteProvider::GitHub);
    }

    #[test]
    fn test_parse_https_url() {
        let parsed = ParsedRemoteUrl::parse("https://github.com/owner/repo.git").unwrap();

        assert_eq!(parsed.protocol, RemoteProtocol::Https);
        assert_eq!(parsed.host, "github.com");
        assert_eq!(parsed.owner, Some("owner".to_string()));
        assert_eq!(parsed.repo, Some("repo".to_string()));
    }

    #[test]
    fn test_parse_gitlab_url() {
        let parsed = ParsedRemoteUrl::parse("git@gitlab.com:group/project.git").unwrap();

        assert_eq!(parsed.provider, RemoteProvider::GitLab);
    }

    #[test]
    fn test_url_conversion() {
        let ssh = "git@github.com:owner/repo.git";
        let https = RemoteUrlConverter::ssh_to_https(ssh).unwrap();
        assert_eq!(https, "https://github.com/owner/repo.git");

        let back_to_ssh = RemoteUrlConverter::https_to_ssh(&https).unwrap();
        assert_eq!(back_to_ssh, ssh);
    }

    #[test]
    fn test_add_remote() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitRemoteManager::new(&repo);

        let remote = manager.add("origin", "https://github.com/owner/repo.git").unwrap();

        assert_eq!(remote.name, "origin");
        assert_eq!(remote.url, Some("https://github.com/owner/repo.git".to_string()));
    }

    #[test]
    fn test_list_remotes() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitRemoteManager::new(&repo);

        manager.add("origin", "https://github.com/owner/repo.git").unwrap();
        manager.add("upstream", "https://github.com/upstream/repo.git").unwrap();

        let remotes = manager.list().unwrap();

        assert_eq!(remotes.len(), 2);
    }

    #[test]
    fn test_remove_remote() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitRemoteManager::new(&repo);

        manager.add("origin", "https://github.com/owner/repo.git").unwrap();
        manager.remove("origin").unwrap();

        assert!(!manager.exists("origin"));
    }

    #[test]
    fn test_rename_remote() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitRemoteManager::new(&repo);

        manager.add("origin", "https://github.com/owner/repo.git").unwrap();
        manager.rename("origin", "upstream").unwrap();

        assert!(!manager.exists("origin"));
        assert!(manager.exists("upstream"));
    }

    #[test]
    fn test_set_urls() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitRemoteManager::new(&repo);

        manager.add("origin", "https://github.com/owner/repo.git").unwrap();
        manager.set_push_url("origin", "git@github.com:owner/repo.git").unwrap();

        let remote = manager.get("origin").unwrap();
        assert_eq!(remote.push_url, Some("git@github.com:owner/repo.git".to_string()));
    }

    #[test]
    fn test_url_normalize() {
        assert_eq!(
            RemoteUrlConverter::normalize("https://github.com/owner/repo"),
            "https://github.com/owner/repo.git"
        );
        assert_eq!(
            RemoteUrlConverter::normalize("https://github.com/owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
    }

    #[test]
    fn test_remote_provider_detection() {
        assert_eq!(
            RemoteProvider::from_url("https://github.com/owner/repo.git"),
            RemoteProvider::GitHub
        );
        assert_eq!(
            RemoteProvider::from_url("git@gitlab.com:owner/repo.git"),
            RemoteProvider::GitLab
        );
        assert_eq!(
            RemoteProvider::from_url("https://bitbucket.org/owner/repo.git"),
            RemoteProvider::Bitbucket
        );
    }

    #[test]
    fn test_invalid_url() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitRemoteManager::new(&repo);

        let result = manager.add("bad", "not-a-valid-url");
        assert!(result.is_err());
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 454: Push Operations
- Spec 455: Pull/Fetch Operations
- Spec 463: Credential Management
