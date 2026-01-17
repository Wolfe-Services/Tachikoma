//! Git push operations.

use crate::{GitRepository, GitResult, GitError};
use git2::{PushOptions, RemoteCallbacks, Cred, CredentialType};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{debug, info};

/// Push progress information.
#[derive(Debug, Clone)]
pub struct PushProgress {
    /// Current operation.
    pub operation: String,
    /// Objects transferred.
    pub current: u32,
    /// Total objects.
    pub total: u32,
    /// Bytes transferred.
    pub bytes: u64,
}

/// Push options.
#[derive(Debug, Clone, Default)]
pub struct PushOpts {
    /// Force push.
    pub force: bool,
    /// Set upstream.
    pub set_upstream: bool,
    /// Push tags.
    pub tags: bool,
    /// Remote name.
    pub remote: Option<String>,
    /// Refspecs to push.
    pub refspecs: Vec<String>,
}

impl PushOpts {
    /// Create default push options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable force push.
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Enable set-upstream.
    pub fn set_upstream(mut self) -> Self {
        self.set_upstream = true;
        self
    }

    /// Include tags.
    pub fn with_tags(mut self) -> Self {
        self.tags = true;
        self
    }

    /// Set remote name.
    pub fn to_remote(mut self, name: impl Into<String>) -> Self {
        self.remote = Some(name.into());
        self
    }

    /// Add a refspec.
    pub fn refspec(mut self, refspec: impl Into<String>) -> Self {
        self.refspecs.push(refspec.into());
        self
    }
}

/// Push result.
#[derive(Debug, Clone)]
pub struct PushResult {
    /// Successfully pushed refs.
    pub pushed: Vec<String>,
    /// Refs that failed to push.
    pub failed: Vec<(String, String)>, // (ref, error)
    /// Was anything pushed.
    pub success: bool,
}

impl GitRepository {
    /// Push current branch to its upstream.
    pub fn push(&self, options: PushOpts) -> GitResult<PushResult> {
        self.push_with_credentials(options, None)
    }

    /// Push with credential callback.
    pub fn push_with_credentials(
        &self,
        options: PushOpts,
        credentials: Option<Box<dyn CredentialProvider>>,
    ) -> GitResult<PushResult> {
        self.with_repo_mut(|repo| {
            // Determine remote
            let remote_name = options.remote.as_deref().unwrap_or("origin");
            let mut remote = repo.find_remote(remote_name).map_err(|_| GitError::RemoteNotFound {
                name: remote_name.to_string(),
            })?;

            // Determine refspecs
            let refspecs = if options.refspecs.is_empty() {
                // Push current branch
                let head = repo.head().map_err(|_| GitError::InvalidOperation {
                    message: "Cannot push: no commits found".to_string(),
                })?;

                let branch = head.shorthand().ok_or_else(|| GitError::InvalidOperation {
                    message: "Cannot push detached HEAD without explicit refspec".to_string(),
                })?;

                let refspec = if options.force {
                    format!("+refs/heads/{}:refs/heads/{}", branch, branch)
                } else {
                    format!("refs/heads/{}:refs/heads/{}", branch, branch)
                };

                vec![refspec]
            } else {
                options.refspecs.iter().map(|r| {
                    if options.force && !r.starts_with('+') {
                        format!("+{}", r)
                    } else {
                        r.clone()
                    }
                }).collect()
            };

            // Setup callbacks
            let mut callbacks = RemoteCallbacks::new();

            // Progress callback
            let progress = Arc::new(Mutex::new(PushProgress {
                operation: "Starting".to_string(),
                current: 0,
                total: 0,
                bytes: 0,
            }));

            let progress_clone = progress.clone();
            callbacks.push_transfer_progress(move |current, total, bytes| {
                let mut p = progress_clone.lock();
                p.operation = "Transferring".to_string();
                p.current = current as u32;
                p.total = total as u32;
                p.bytes = bytes as u64;
                debug!("Push progress: {}/{} objects, {} bytes", current, total, bytes);
                true // Continue
            });

            // Track pushed refs
            let pushed = Arc::new(Mutex::new(Vec::new()));
            let failed = Arc::new(Mutex::new(Vec::new()));

            let pushed_clone = pushed.clone();
            let failed_clone = failed.clone();
            callbacks.push_update_reference(move |refname, status| {
                match status {
                    None => {
                        pushed_clone.lock().push(refname.to_string());
                        info!("Pushed: {}", refname);
                        Ok(())
                    }
                    Some(msg) => {
                        failed_clone.lock().push((refname.to_string(), msg.to_string()));
                        Ok(())
                    }
                }
            });

            // Credentials callback
            if let Some(provider) = credentials {
                callbacks.credentials(move |url, username_from_url, allowed_types| {
                    provider.get_credentials(url, username_from_url, allowed_types)
                });
            } else {
                // Default credential handling
                callbacks.credentials(|_url, username_from_url, allowed_types| {
                    // Try SSH key first
                    if allowed_types.contains(CredentialType::SSH_KEY) {
                        if let Some(username) = username_from_url {
                            if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                                return Ok(cred);
                            }
                        }
                    }

                    // Fall back to default credentials
                    if allowed_types.contains(CredentialType::DEFAULT) {
                        return Cred::default();
                    }

                    Err(git2::Error::from_str("No suitable credentials found"))
                });
            }

            // Setup push options
            let mut push_opts = PushOptions::new();
            push_opts.remote_callbacks(callbacks);

            // Execute push
            let refspec_strs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();
            remote.push(&refspec_strs, Some(&mut push_opts)).map_err(|e| {
                if e.message().contains("authentication") || e.message().contains("credentials") {
                    GitError::AuthFailed {
                        reason: e.message().to_string(),
                    }
                } else if e.message().contains("network") || e.message().contains("connection") {
                    GitError::Network {
                        message: e.message().to_string(),
                    }
                } else {
                    GitError::Git2(e)
                }
            })?;

            // Collect results
            let pushed = Arc::try_unwrap(pushed).unwrap().into_inner();
            let failed = Arc::try_unwrap(failed).unwrap().into_inner();

            // Set upstream if requested
            if options.set_upstream && !pushed.is_empty() {
                if let Ok(head) = repo.head() {
                    if let Some(branch_name) = head.shorthand() {
                        if let Ok(mut branch) = repo.find_branch(branch_name, git2::BranchType::Local) {
                            let upstream = format!("{}/{}", remote_name, branch_name);
                            let _ = branch.set_upstream(Some(&upstream));
                        }
                    }
                }
            }

            Ok(PushResult {
                success: !pushed.is_empty() && failed.is_empty(),
                pushed,
                failed,
            })
        })
    }

    /// Push all tags.
    pub fn push_tags(&self, remote: Option<&str>) -> GitResult<PushResult> {
        let opts = PushOpts::new()
            .to_remote(remote.unwrap_or("origin"))
            .refspec("refs/tags/*:refs/tags/*");

        self.push(opts)
    }

    /// Delete a remote branch.
    pub fn delete_remote_branch(&self, remote: &str, branch: &str) -> GitResult<PushResult> {
        let opts = PushOpts::new()
            .to_remote(remote)
            .refspec(format!(":refs/heads/{}", branch));

        self.push(opts)
    }
}

/// Credential provider trait.
pub trait CredentialProvider: Send + Sync {
    /// Get credentials for authentication.
    fn get_credentials(
        &self,
        url: &str,
        username_from_url: Option<&str>,
        allowed_types: CredentialType,
    ) -> Result<Cred, git2::Error>;
}

/// Default credential provider using git config.
pub struct DefaultCredentialProvider;

impl CredentialProvider for DefaultCredentialProvider {
    fn get_credentials(
        &self,
        url: &str,
        username_from_url: Option<&str>,
        allowed_types: CredentialType,
    ) -> Result<Cred, git2::Error> {
        // Try SSH key first
        if allowed_types.contains(CredentialType::SSH_KEY) {
            if let Some(username) = username_from_url {
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }
            }
        }

        // Fall back to default credentials
        if allowed_types.contains(CredentialType::DEFAULT) {
            return Cred::default();
        }

        Err(git2::Error::from_str("No suitable credentials found"))
    }
}