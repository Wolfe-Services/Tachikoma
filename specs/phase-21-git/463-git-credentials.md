# Spec 463: Credential Management

## Phase
21 - Git Integration

## Spec ID
463

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 447: Git Configuration (configuration access)

## Estimated Context
~9%

---

## Objective

Implement Git credential management for Tachikoma, providing secure handling of authentication credentials for Git operations. This module supports multiple credential sources including SSH keys, credential helpers, and system keychains with secure memory handling.

---

## Acceptance Criteria

- [ ] Implement `GitCredentialManager` for credential operations
- [ ] Support SSH key authentication
- [ ] Support HTTPS authentication
- [ ] Integrate with system credential helpers
- [ ] Support credential caching
- [ ] Implement secure credential storage
- [ ] Support multiple credential sources
- [ ] Handle authentication prompts
- [ ] Support SSH agent forwarding
- [ ] Validate credentials before use

---

## Implementation Details

### Credential Manager Implementation

```rust
// src/git/credentials.rs

use git2::{Cred, CredentialType, RemoteCallbacks};
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::types::*;

/// Credential type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitCredentialKind {
    /// Username/password for HTTPS
    UserPass,
    /// SSH key
    SshKey,
    /// SSH key from agent
    SshAgent,
    /// Default system credentials
    Default,
}

/// Stored credential
#[derive(Clone)]
pub struct StoredCredential {
    pub kind: GitCredentialKind,
    pub host: String,
    pub username: Option<String>,
    pub password: Option<SecretString>,
    pub ssh_key_path: Option<PathBuf>,
    pub ssh_passphrase: Option<SecretString>,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
}

impl StoredCredential {
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Instant::now() > e).unwrap_or(false)
    }
}

impl std::fmt::Debug for StoredCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredCredential")
            .field("kind", &self.kind)
            .field("host", &self.host)
            .field("username", &self.username)
            .field("ssh_key_path", &self.ssh_key_path)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

/// Credential request context
#[derive(Debug, Clone)]
pub struct CredentialRequest {
    pub url: String,
    pub host: String,
    pub username: Option<String>,
    pub allowed_types: CredentialTypeFlags,
}

#[derive(Debug, Clone, Copy)]
pub struct CredentialTypeFlags {
    pub userpass_plaintext: bool,
    pub ssh_key: bool,
    pub ssh_agent: bool,
    pub default: bool,
}

impl From<CredentialType> for CredentialTypeFlags {
    fn from(types: CredentialType) -> Self {
        Self {
            userpass_plaintext: types.contains(CredentialType::USER_PASS_PLAINTEXT),
            ssh_key: types.contains(CredentialType::SSH_KEY),
            ssh_agent: types.contains(CredentialType::SSH_MEMORY),
            default: types.contains(CredentialType::DEFAULT),
        }
    }
}

/// Credential provider trait
pub trait CredentialProvider: Send + Sync {
    fn provide(&self, request: &CredentialRequest) -> GitResult<Option<StoredCredential>>;
    fn name(&self) -> &'static str;
}

/// SSH agent provider
pub struct SshAgentProvider;

impl CredentialProvider for SshAgentProvider {
    fn provide(&self, request: &CredentialRequest) -> GitResult<Option<StoredCredential>> {
        if !request.allowed_types.ssh_agent && !request.allowed_types.ssh_key {
            return Ok(None);
        }

        Ok(Some(StoredCredential {
            kind: GitCredentialKind::SshAgent,
            host: request.host.clone(),
            username: request.username.clone(),
            password: None,
            ssh_key_path: None,
            ssh_passphrase: None,
            created_at: Instant::now(),
            expires_at: None,
        }))
    }

    fn name(&self) -> &'static str {
        "ssh-agent"
    }
}

/// SSH key file provider
pub struct SshKeyProvider {
    key_paths: Vec<PathBuf>,
}

impl SshKeyProvider {
    pub fn new() -> Self {
        let mut key_paths = Vec::new();

        // Default SSH key locations
        if let Some(home) = dirs::home_dir() {
            let ssh_dir = home.join(".ssh");
            for key_name in &["id_ed25519", "id_rsa", "id_ecdsa"] {
                let key_path = ssh_dir.join(key_name);
                if key_path.exists() {
                    key_paths.push(key_path);
                }
            }
        }

        Self { key_paths }
    }

    pub fn with_key(mut self, path: impl Into<PathBuf>) -> Self {
        self.key_paths.push(path.into());
        self
    }
}

impl Default for SshKeyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialProvider for SshKeyProvider {
    fn provide(&self, request: &CredentialRequest) -> GitResult<Option<StoredCredential>> {
        if !request.allowed_types.ssh_key {
            return Ok(None);
        }

        for key_path in &self.key_paths {
            if key_path.exists() {
                return Ok(Some(StoredCredential {
                    kind: GitCredentialKind::SshKey,
                    host: request.host.clone(),
                    username: request.username.clone(),
                    password: None,
                    ssh_key_path: Some(key_path.clone()),
                    ssh_passphrase: None,
                    created_at: Instant::now(),
                    expires_at: None,
                }));
            }
        }

        Ok(None)
    }

    fn name(&self) -> &'static str {
        "ssh-key"
    }
}

/// Credential cache
pub struct CredentialCache {
    credentials: Arc<Mutex<HashMap<String, StoredCredential>>>,
    default_ttl: Duration,
}

impl CredentialCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            credentials: Arc::new(Mutex::new(HashMap::new())),
            default_ttl,
        }
    }

    pub fn get(&self, host: &str) -> Option<StoredCredential> {
        let creds = self.credentials.lock().unwrap();
        creds.get(host).and_then(|c| {
            if c.is_expired() {
                None
            } else {
                Some(c.clone())
            }
        })
    }

    pub fn store(&self, credential: StoredCredential) {
        let mut creds = self.credentials.lock().unwrap();
        creds.insert(credential.host.clone(), credential);
    }

    pub fn remove(&self, host: &str) {
        let mut creds = self.credentials.lock().unwrap();
        creds.remove(host);
    }

    pub fn clear(&self) {
        let mut creds = self.credentials.lock().unwrap();
        creds.clear();
    }

    pub fn clear_expired(&self) {
        let mut creds = self.credentials.lock().unwrap();
        creds.retain(|_, c| !c.is_expired());
    }
}

impl Default for CredentialCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(3600)) // 1 hour default
    }
}

/// Git credential manager
pub struct GitCredentialManager {
    providers: Vec<Box<dyn CredentialProvider>>,
    cache: CredentialCache,
    prompt_callback: Option<Box<dyn Fn(&CredentialRequest) -> Option<StoredCredential> + Send + Sync>>,
}

impl GitCredentialManager {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            cache: CredentialCache::default(),
            prompt_callback: None,
        }
    }

    /// Create with default providers
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.add_provider(Box::new(SshAgentProvider));
        manager.add_provider(Box::new(SshKeyProvider::new()));
        manager
    }

    /// Add a credential provider
    pub fn add_provider(&mut self, provider: Box<dyn CredentialProvider>) {
        self.providers.push(provider);
    }

    /// Set a prompt callback for interactive credential requests
    pub fn set_prompt_callback<F>(&mut self, callback: F)
    where
        F: Fn(&CredentialRequest) -> Option<StoredCredential> + Send + Sync + 'static,
    {
        self.prompt_callback = Some(Box::new(callback));
    }

    /// Get credentials for a URL
    pub fn get_credentials(&self, url: &str, username: Option<&str>, allowed: CredentialType) -> GitResult<Cred> {
        let request = self.create_request(url, username, allowed);

        // Check cache first
        if let Some(cached) = self.cache.get(&request.host) {
            return self.credential_to_cred(&cached, username);
        }

        // Try providers
        for provider in &self.providers {
            if let Ok(Some(cred)) = provider.provide(&request) {
                // Cache the credential
                self.cache.store(cred.clone());
                return self.credential_to_cred(&cred, username);
            }
        }

        // Try prompt callback
        if let Some(ref callback) = self.prompt_callback {
            if let Some(cred) = callback(&request) {
                self.cache.store(cred.clone());
                return self.credential_to_cred(&cred, username);
            }
        }

        // Fall back to default
        Cred::default().map_err(GitError::Git2)
    }

    /// Create remote callbacks with credential handling
    pub fn create_callbacks(&self) -> RemoteCallbacks<'_> {
        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|url, username, allowed| {
            self.get_credentials(url, username, allowed)
                .map_err(|e| git2::Error::from_str(&e.to_string()))
        });

        callbacks
    }

    /// Store a credential manually
    pub fn store(&self, credential: StoredCredential) {
        self.cache.store(credential);
    }

    /// Remove cached credential
    pub fn remove(&self, host: &str) {
        self.cache.remove(host);
    }

    /// Clear all cached credentials
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    fn create_request(&self, url: &str, username: Option<&str>, allowed: CredentialType) -> CredentialRequest {
        let host = url::Url::parse(url)
            .map(|u| u.host_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| {
                // Try to extract host from SSH URL
                if url.contains('@') {
                    url.split('@').nth(1)
                        .and_then(|s| s.split(':').next())
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                }
            });

        CredentialRequest {
            url: url.to_string(),
            host,
            username: username.map(String::from),
            allowed_types: CredentialTypeFlags::from(allowed),
        }
    }

    fn credential_to_cred(&self, stored: &StoredCredential, username: Option<&str>) -> GitResult<Cred> {
        match stored.kind {
            GitCredentialKind::SshAgent => {
                let username = username.or(stored.username.as_deref()).unwrap_or("git");
                Cred::ssh_key_from_agent(username).map_err(GitError::Git2)
            }
            GitCredentialKind::SshKey => {
                let username = username.or(stored.username.as_deref()).unwrap_or("git");
                let key_path = stored.ssh_key_path.as_ref()
                    .ok_or_else(|| GitError::AuthenticationFailed("No SSH key path".into()))?;
                let pub_key_path = PathBuf::from(format!("{}.pub", key_path.display()));
                let pub_key = if pub_key_path.exists() { Some(pub_key_path.as_path()) } else { None };

                Cred::ssh_key(
                    username,
                    pub_key,
                    key_path,
                    stored.ssh_passphrase.as_ref().map(|p| p.expose_secret().as_str()),
                ).map_err(GitError::Git2)
            }
            GitCredentialKind::UserPass => {
                let username = username.or(stored.username.as_deref())
                    .ok_or_else(|| GitError::AuthenticationFailed("No username".into()))?;
                let password = stored.password.as_ref()
                    .ok_or_else(|| GitError::AuthenticationFailed("No password".into()))?;

                Cred::userpass_plaintext(username, password.expose_secret())
                    .map_err(GitError::Git2)
            }
            GitCredentialKind::Default => {
                Cred::default().map_err(GitError::Git2)
            }
        }
    }
}

impl Default for GitCredentialManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Helper to check SSH key
pub fn check_ssh_key(path: &Path) -> GitResult<SshKeyInfo> {
    if !path.exists() {
        return Err(GitError::AuthenticationFailed(format!(
            "SSH key not found: {}",
            path.display()
        )));
    }

    let content = std::fs::read_to_string(path)?;

    let key_type = if content.contains("BEGIN OPENSSH PRIVATE KEY") {
        "openssh"
    } else if content.contains("BEGIN RSA PRIVATE KEY") {
        "rsa"
    } else if content.contains("BEGIN EC PRIVATE KEY") {
        "ecdsa"
    } else if content.contains("BEGIN DSA PRIVATE KEY") {
        "dsa"
    } else {
        "unknown"
    };

    let encrypted = content.contains("ENCRYPTED");

    Ok(SshKeyInfo {
        path: path.to_path_buf(),
        key_type: key_type.to_string(),
        encrypted,
    })
}

#[derive(Debug, Clone)]
pub struct SshKeyInfo {
    pub path: PathBuf,
    pub key_type: String,
    pub encrypted: bool,
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

    #[test]
    fn test_credential_type_flags_from() {
        let flags = CredentialTypeFlags::from(
            CredentialType::SSH_KEY | CredentialType::USER_PASS_PLAINTEXT
        );

        assert!(flags.ssh_key);
        assert!(flags.userpass_plaintext);
        assert!(!flags.ssh_agent);
    }

    #[test]
    fn test_stored_credential_expired() {
        let cred = StoredCredential {
            kind: GitCredentialKind::SshAgent,
            host: "github.com".to_string(),
            username: Some("git".to_string()),
            password: None,
            ssh_key_path: None,
            ssh_passphrase: None,
            created_at: Instant::now(),
            expires_at: Some(Instant::now() - Duration::from_secs(1)),
        };

        assert!(cred.is_expired());
    }

    #[test]
    fn test_stored_credential_not_expired() {
        let cred = StoredCredential {
            kind: GitCredentialKind::SshAgent,
            host: "github.com".to_string(),
            username: Some("git".to_string()),
            password: None,
            ssh_key_path: None,
            ssh_passphrase: None,
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + Duration::from_secs(3600)),
        };

        assert!(!cred.is_expired());
    }

    #[test]
    fn test_credential_cache_store_get() {
        let cache = CredentialCache::default();

        let cred = StoredCredential {
            kind: GitCredentialKind::SshAgent,
            host: "github.com".to_string(),
            username: Some("git".to_string()),
            password: None,
            ssh_key_path: None,
            ssh_passphrase: None,
            created_at: Instant::now(),
            expires_at: None,
        };

        cache.store(cred.clone());

        let retrieved = cache.get("github.com").unwrap();
        assert_eq!(retrieved.host, "github.com");
    }

    #[test]
    fn test_credential_cache_remove() {
        let cache = CredentialCache::default();

        let cred = StoredCredential {
            kind: GitCredentialKind::SshAgent,
            host: "github.com".to_string(),
            username: None,
            password: None,
            ssh_key_path: None,
            ssh_passphrase: None,
            created_at: Instant::now(),
            expires_at: None,
        };

        cache.store(cred);
        assert!(cache.get("github.com").is_some());

        cache.remove("github.com");
        assert!(cache.get("github.com").is_none());
    }

    #[test]
    fn test_credential_manager_default() {
        let manager = GitCredentialManager::with_defaults();
        // Just ensure it doesn't panic
        assert!(!manager.providers.is_empty());
    }

    #[test]
    fn test_ssh_key_provider_new() {
        let provider = SshKeyProvider::new();
        assert_eq!(provider.name(), "ssh-key");
    }

    #[test]
    fn test_ssh_agent_provider() {
        let provider = SshAgentProvider;
        assert_eq!(provider.name(), "ssh-agent");

        let request = CredentialRequest {
            url: "git@github.com:user/repo.git".to_string(),
            host: "github.com".to_string(),
            username: Some("git".to_string()),
            allowed_types: CredentialTypeFlags {
                userpass_plaintext: false,
                ssh_key: true,
                ssh_agent: true,
                default: false,
            },
        };

        let result = provider.provide(&request).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_check_ssh_key_not_found() {
        let result = check_ssh_key(Path::new("/nonexistent/key"));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_ssh_key_types() {
        let dir = TempDir::new().unwrap();

        // RSA key
        let rsa_path = dir.path().join("id_rsa");
        std::fs::write(&rsa_path, "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----").unwrap();
        let info = check_ssh_key(&rsa_path).unwrap();
        assert_eq!(info.key_type, "rsa");

        // OpenSSH key
        let openssh_path = dir.path().join("id_ed25519");
        std::fs::write(&openssh_path, "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----").unwrap();
        let info = check_ssh_key(&openssh_path).unwrap();
        assert_eq!(info.key_type, "openssh");
    }

    #[test]
    fn test_credential_kind_debug() {
        let cred = StoredCredential {
            kind: GitCredentialKind::UserPass,
            host: "github.com".to_string(),
            username: Some("user".to_string()),
            password: Some(SecretString::new("secret".to_string())),
            ssh_key_path: None,
            ssh_passphrase: None,
            created_at: Instant::now(),
            expires_at: None,
        };

        let debug = format!("{:?}", cred);
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("secret"));
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 447: Git Configuration
- Spec 453: Remote Operations
- Spec 454: Push Operations
- Spec 455: Pull/Fetch Operations
