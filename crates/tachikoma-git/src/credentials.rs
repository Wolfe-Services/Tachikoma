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

    /// Create SSH key credential with public key.
    pub fn ssh_key_with_public(
        username: impl Into<String>,
        private_key: impl Into<PathBuf>,
        public_key: impl Into<PathBuf>,
    ) -> Self {
        Self::SshKey {
            username: username.into(),
            private_key: private_key.into(),
            public_key: Some(public_key.into()),
            passphrase: None,
        }
    }

    /// Create SSH key credential with passphrase.
    pub fn ssh_key_with_passphrase(
        username: impl Into<String>,
        private_key: impl Into<PathBuf>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self::SshKey {
            username: username.into(),
            private_key: private_key.into(),
            public_key: None,
            passphrase: Some(passphrase.into()),
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

    /// Create token credential with custom username.
    pub fn token_with_username(username: impl Into<String>, token: impl Into<String>) -> Self {
        Self::Token {
            username: username.into(),
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

    /// Check if this credential type is compatible with the requested types.
    pub fn is_compatible_with(&self, allowed_types: CredentialType) -> bool {
        match self {
            Self::SshKey { .. } | Self::SshAgent { .. } => {
                allowed_types.contains(CredentialType::SSH_KEY)
            }
            Self::UserPassword { .. } | Self::Token { .. } => {
                allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT)
            }
            Self::Default => allowed_types.contains(CredentialType::DEFAULT),
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

    /// List all stored patterns.
    pub fn patterns(&self) -> Vec<String> {
        self.credentials.keys().cloned().collect()
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    // Simple pattern matching: pattern can be a prefix or contain wildcard
    if pattern.contains('*') {
        // Convert to regex-like matching
        let pattern_regex = pattern.replace('*', ".*");
        return url.contains(&pattern_regex.replace(".*", ""));
    }
    
    // Check if URL starts with pattern or contains the domain/host
    url.starts_with(pattern) || 
    url.contains(pattern) ||
    extract_host(url).map_or(false, |host| host == pattern || host.ends_with(&format!(".{}", pattern)))
}

fn extract_host(url: &str) -> Option<String> {
    if let Some(start) = url.find("://") {
        let after_scheme = &url[start + 3..];
        if let Some(end) = after_scheme.find('/') {
            Some(after_scheme[..end].to_string())
        } else if let Some(end) = after_scheme.find(':') {
            Some(after_scheme[..end].to_string())
        } else {
            Some(after_scheme.to_string())
        }
    } else {
        // Handle SSH format like git@github.com:user/repo.git
        if let Some(at_pos) = url.find('@') {
            let after_at = &url[at_pos + 1..];
            if let Some(colon_pos) = after_at.find(':') {
                Some(after_at[..colon_pos].to_string())
            } else {
                Some(after_at.to_string())
            }
        } else {
            None
        }
    }
}

/// Credential callback builder.
pub struct CredentialCallback {
    credentials: Vec<GitCredential>,
    store: Option<std::sync::Arc<parking_lot::Mutex<CredentialStore>>>,
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
    pub fn with_store(mut self, store: std::sync::Arc<parking_lot::Mutex<CredentialStore>>) -> Self {
        self.store = Some(store);
        self
    }

    /// Build the callback function.
    pub fn build(self) -> impl Fn(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error> + Send + Sync {
        move |url, username_from_url, allowed_types| {
            // Try store first
            if let Some(ref store) = self.store {
                if let Ok(store) = store.try_lock() {
                    if let Some(cred) = store.get(url) {
                        if cred.is_compatible_with(allowed_types) {
                            if let Ok(git_cred) = cred.to_git2() {
                                return Ok(git_cred);
                            }
                        }
                    }
                }
            }

            // Try provided credentials
            for cred in &self.credentials {
                if cred.is_compatible_with(allowed_types) {
                    if let Ok(git_cred) = cred.to_git2() {
                        return Ok(git_cred);
                    }
                }
            }

            // Try SSH agent
            if allowed_types.contains(CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                        return Ok(cred);
                    }
                }
                // Try common usernames for SSH
                for username in &["git", "root"] {
                    if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                        return Ok(cred);
                    }
                }
            }

            // Try default SSH keys
            if allowed_types.contains(CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url.or(Some("git")) {
                    for key_path in ssh::default_keys() {
                        let public_key = key_path.with_extension("pub");
                        let public_key = if public_key.exists() {
                            Some(public_key.as_path())
                        } else {
                            None
                        };
                        
                        if let Ok(cred) = Cred::ssh_key(username, public_key, &key_path, None) {
                            return Ok(cred);
                        }
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
            // Order by preference: ed25519, rsa, ecdsa, dsa
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

    /// List keys available in SSH agent.
    pub fn agent_keys() -> Vec<String> {
        // This would require SSH agent protocol implementation
        // For now, just check if agent is available
        if agent_available() {
            vec!["ssh-agent".to_string()]
        } else {
            vec![]
        }
    }

    /// Generate SSH key pair.
    pub fn generate_key_pair(
        path: &std::path::Path,
        key_type: SshKeyType,
        passphrase: Option<&str>,
    ) -> crate::GitResult<()> {
        use std::process::Command;

        let type_arg = match key_type {
            SshKeyType::Ed25519 => "ed25519",
            SshKeyType::Rsa => "rsa",
            SshKeyType::Ecdsa => "ecdsa",
        };

        let mut cmd = Command::new("ssh-keygen");
        cmd.args(&["-t", type_arg])
           .args(&["-f", &path.to_string_lossy()])
           .args(&["-N", passphrase.unwrap_or("")]);

        let output = cmd.output().map_err(|e| crate::GitError::InvalidOperation {
            message: format!("Failed to generate SSH key: {}", e),
        })?;

        if !output.status.success() {
            return Err(crate::GitError::InvalidOperation {
                message: format!(
                    "ssh-keygen failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }
}

/// SSH key types.
#[derive(Debug, Clone, Copy)]
pub enum SshKeyType {
    /// Ed25519 (recommended).
    Ed25519,
    /// RSA.
    Rsa,
    /// ECDSA.
    Ecdsa,
}