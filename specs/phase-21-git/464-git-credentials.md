# 464 - Git Credentials

**Phase:** 21 - Git Integration
**Spec ID:** 464
**Status:** Planned
**Dependencies:** 452-git-detect
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement Git credential management, supporting various authentication methods for remote operations.

---

## Acceptance Criteria

- [ ] SSH key authentication
- [ ] HTTPS with credentials
- [ ] Credential caching
- [ ] Platform keychain integration
- [ ] Token-based authentication

---

## Implementation Details

### 1. Credential Types (src/credentials.rs)

```rust
//! Git credential management.

use crate::{GitResult, GitError};
use git2::{Cred, CredentialType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Credential type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GitCredential {
    /// SSH key authentication.
    SshKey {
        /// Username for SSH.
        username: String,
        /// Path to private key.
        private_key: PathBuf,
        /// Path to public key (optional).
        public_key: Option<PathBuf>,
        /// Passphrase (optional).
        passphrase: Option<String>,
    },
    /// Username and password.
    UserPassword {
        username: String,
        password: String,
    },
    /// Personal access token.
    Token {
        /// Username (usually "oauth2" or "x-access-token").
        username: String,
        /// The token.
        token: String,
    },
    /// SSH agent.
    SshAgent {
        username: String,
    },
    /// Default system credentials.
    Default,
}

impl GitCredential {
    /// Create SSH key credential.
    pub fn ssh_key(
        username: impl Into<String>,
        private_key: impl Into<PathBuf>,
    ) -> Self {
        Self::SshKey {
            username: username.into(),
            private_key: private_key.into(),
            public_key: None,
            passphrase: None,
        }
    }

    /// Create SSH agent credential.
    pub fn ssh_agent(username: impl Into<String>) -> Self {
        Self::SshAgent {
            username: username.into(),
        }
    }

    /// Create token credential.
    pub fn token(token: impl Into<String>) -> Self {
        Self::Token {
            username: "x-access-token".to_string(),
            token: token.into(),
        }
    }

    /// Create username/password credential.
    pub fn user_password(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::UserPassword {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Convert to git2 Cred.
    pub fn to_git2(&self) -> Result<Cred, git2::Error> {
        match self {
            Self::SshKey {
                username,
                private_key,
                public_key,
                passphrase,
            } => Cred::ssh_key(
                username,
                public_key.as_ref().map(|p| p.as_path()),
                private_key,
                passphrase.as_deref(),
            ),
            Self::UserPassword { username, password } => {
                Cred::userpass_plaintext(username, password)
            }
            Self::Token { username, token } => {
                Cred::userpass_plaintext(username, token)
            }
            Self::SshAgent { username } => {
                Cred::ssh_key_from_agent(username)
            }
            Self::Default => Cred::default(),
        }
    }
}

/// Credential store for caching credentials.
pub struct CredentialStore {
    credentials: std::collections::HashMap<String, GitCredential>,
}

impl CredentialStore {
    /// Create a new credential store.
    pub fn new() -> Self {
        Self {
            credentials: std::collections::HashMap::new(),
        }
    }

    /// Store a credential for a URL pattern.
    pub fn store(&mut self, pattern: impl Into<String>, credential: GitCredential) {
        self.credentials.insert(pattern.into(), credential);
    }

    /// Get a credential for a URL.
    pub fn get(&self, url: &str) -> Option<&GitCredential> {
        // Try exact match first
        if let Some(cred) = self.credentials.get(url) {
            return Some(cred);
        }

        // Try pattern matching
        for (pattern, cred) in &self.credentials {
            if url_matches_pattern(url, pattern) {
                return Some(cred);
            }
        }

        None
    }

    /// Remove a credential.
    pub fn remove(&mut self, pattern: &str) -> Option<GitCredential> {
        self.credentials.remove(pattern)
    }

    /// Clear all credentials.
    pub fn clear(&mut self) {
        self.credentials.clear();
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    // Simple pattern matching: pattern can be a prefix
    url.starts_with(pattern) || url.contains(pattern)
}

/// Credential callback builder.
pub struct CredentialCallback {
    credentials: Vec<GitCredential>,
    store: Option<std::sync::Arc<std::sync::Mutex<CredentialStore>>>,
}

impl CredentialCallback {
    /// Create a new credential callback.
    pub fn new() -> Self {
        Self {
            credentials: Vec::new(),
            store: None,
        }
    }

    /// Add a credential to try.
    pub fn with_credential(mut self, credential: GitCredential) -> Self {
        self.credentials.push(credential);
        self
    }

    /// Use a credential store.
    pub fn with_store(mut self, store: std::sync::Arc<std::sync::Mutex<CredentialStore>>) -> Self {
        self.store = Some(store);
        self
    }

    /// Build the callback function.
    pub fn build(self) -> impl Fn(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error> {
        move |url, username_from_url, allowed_types| {
            // Try store first
            if let Some(ref store) = self.store {
                if let Ok(store) = store.lock() {
                    if let Some(cred) = store.get(url) {
                        if let Ok(git_cred) = cred.to_git2() {
                            return Ok(git_cred);
                        }
                    }
                }
            }

            // Try provided credentials
            for cred in &self.credentials {
                if let Ok(git_cred) = cred.to_git2() {
                    return Ok(git_cred);
                }
            }

            // Try SSH agent
            if allowed_types.contains(CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                        return Ok(cred);
                    }
                }
            }

            // Try default
            if allowed_types.contains(CredentialType::DEFAULT) {
                return Cred::default();
            }

            Err(git2::Error::from_str("No suitable credentials found"))
        }
    }
}

impl Default for CredentialCallback {
    fn default() -> Self {
        Self::new()
    }
}

/// SSH key utilities.
pub mod ssh {
    use std::path::PathBuf;

    /// Get the default SSH directory.
    pub fn ssh_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".ssh"))
    }

    /// Get default SSH key paths.
    pub fn default_keys() -> Vec<PathBuf> {
        let mut keys = Vec::new();

        if let Some(ssh_dir) = ssh_dir() {
            for name in &["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"] {
                let path = ssh_dir.join(name);
                if path.exists() {
                    keys.push(path);
                }
            }
        }

        keys
    }

    /// Check if SSH agent is available.
    pub fn agent_available() -> bool {
        std::env::var("SSH_AUTH_SOCK").is_ok()
    }
}
```

### 2. Platform Keychain (src/keychain.rs)

```rust
//! Platform keychain integration.

use crate::{GitCredential, GitResult, GitError};

/// Keychain service name.
const SERVICE_NAME: &str = "tachikoma-git";

/// Store credential in system keychain.
#[cfg(target_os = "macos")]
pub fn store_in_keychain(account: &str, credential: &GitCredential) -> GitResult<()> {
    use security_framework::passwords::set_generic_password;

    let password = match credential {
        GitCredential::UserPassword { password, .. } => password.clone(),
        GitCredential::Token { token, .. } => token.clone(),
        _ => return Err(GitError::InvalidOperation {
            message: "Only password/token credentials can be stored in keychain".to_string(),
        }),
    };

    set_generic_password(SERVICE_NAME, account, password.as_bytes())
        .map_err(|e| GitError::InvalidOperation {
            message: format!("Failed to store in keychain: {}", e),
        })?;

    Ok(())
}

/// Get credential from system keychain.
#[cfg(target_os = "macos")]
pub fn get_from_keychain(account: &str) -> GitResult<Option<String>> {
    use security_framework::passwords::get_generic_password;

    match get_generic_password(SERVICE_NAME, account) {
        Ok(password) => {
            let password = String::from_utf8(password)
                .map_err(|_| GitError::InvalidOperation {
                    message: "Invalid UTF-8 in stored password".to_string(),
                })?;
            Ok(Some(password))
        }
        Err(_) => Ok(None),
    }
}

/// Delete credential from system keychain.
#[cfg(target_os = "macos")]
pub fn delete_from_keychain(account: &str) -> GitResult<()> {
    use security_framework::passwords::delete_generic_password;

    let _ = delete_generic_password(SERVICE_NAME, account);
    Ok(())
}

// Stubs for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn store_in_keychain(_account: &str, _credential: &GitCredential) -> GitResult<()> {
    Err(GitError::InvalidOperation {
        message: "Keychain not supported on this platform".to_string(),
    })
}

#[cfg(not(target_os = "macos"))]
pub fn get_from_keychain(_account: &str) -> GitResult<Option<String>> {
    Ok(None)
}

#[cfg(not(target_os = "macos"))]
pub fn delete_from_keychain(_account: &str) -> GitResult<()> {
    Ok(())
}
```

---

## Testing Requirements

1. SSH key authentication works
2. Token authentication works
3. Credential caching works
4. URL pattern matching works
5. Fallback chain works

---

## Related Specs

- Depends on: [452-git-detect.md](452-git-detect.md)
- Next: [465-git-ssh.md](465-git-ssh.md)
