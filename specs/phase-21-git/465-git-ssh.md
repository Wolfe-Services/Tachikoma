# 465 - Git SSH

**Phase:** 21 - Git Integration
**Spec ID:** 465
**Status:** Planned
**Dependencies:** 464-git-credentials
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement SSH-specific Git operations, including SSH key management and SSH URL handling.

---

## Acceptance Criteria

- [ ] SSH URL parsing
- [ ] SSH key generation
- [ ] SSH config parsing
- [ ] Known hosts management
- [ ] SSH agent interaction

---

## Implementation Details

### 1. SSH Types (src/ssh.rs)

```rust
//! SSH support for Git operations.

use crate::{GitResult, GitError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// SSH URL components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshUrl {
    /// Username (usually "git").
    pub user: String,
    /// Host.
    pub host: String,
    /// Port (default 22).
    pub port: u16,
    /// Path on the remote.
    pub path: String,
}

impl SshUrl {
    /// Parse an SSH URL.
    pub fn parse(url: &str) -> Option<Self> {
        // Handle both formats:
        // git@github.com:user/repo.git
        // ssh://git@github.com/user/repo.git

        if url.starts_with("ssh://") {
            Self::parse_ssh_scheme(url)
        } else if url.contains('@') && url.contains(':') {
            Self::parse_scp_style(url)
        } else {
            None
        }
    }

    fn parse_ssh_scheme(url: &str) -> Option<Self> {
        let url = url.strip_prefix("ssh://")?;
        let (user_host, path) = url.split_once('/')?;

        let (user, host_port) = if user_host.contains('@') {
            let (u, h) = user_host.split_once('@')?;
            (u.to_string(), h)
        } else {
            ("git".to_string(), user_host)
        };

        let (host, port) = if host_port.contains(':') {
            let (h, p) = host_port.split_once(':')?;
            (h.to_string(), p.parse().ok()?)
        } else {
            (host_port.to_string(), 22)
        };

        Some(Self {
            user,
            host,
            port,
            path: path.to_string(),
        })
    }

    fn parse_scp_style(url: &str) -> Option<Self> {
        let (user_host, path) = url.split_once(':')?;
        let (user, host) = user_host.split_once('@')?;

        Some(Self {
            user: user.to_string(),
            host: host.to_string(),
            port: 22,
            path: path.to_string(),
        })
    }

    /// Convert to SSH URL string.
    pub fn to_url(&self) -> String {
        if self.port == 22 {
            format!("{}@{}:{}", self.user, self.host, self.path)
        } else {
            format!("ssh://{}@{}:{}/{}", self.user, self.host, self.port, self.path)
        }
    }

    /// Convert to HTTPS URL (for GitHub/GitLab).
    pub fn to_https(&self) -> Option<String> {
        // Only works for well-known hosts
        if self.host.contains("github.com") || self.host.contains("gitlab.com") {
            Some(format!("https://{}/{}", self.host, self.path))
        } else {
            None
        }
    }
}

/// SSH key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SshKeyType {
    Rsa,
    Ed25519,
    Ecdsa,
    Dsa,
}

impl SshKeyType {
    /// Get key file name prefix.
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Rsa => "id_rsa",
            Self::Ed25519 => "id_ed25519",
            Self::Ecdsa => "id_ecdsa",
            Self::Dsa => "id_dsa",
        }
    }

    /// Get ssh-keygen algorithm name.
    pub fn algorithm(&self) -> &'static str {
        match self {
            Self::Rsa => "rsa",
            Self::Ed25519 => "ed25519",
            Self::Ecdsa => "ecdsa",
            Self::Dsa => "dsa",
        }
    }
}

impl Default for SshKeyType {
    fn default() -> Self {
        Self::Ed25519
    }
}

/// SSH key pair information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyPair {
    /// Key type.
    pub key_type: SshKeyType,
    /// Private key path.
    pub private_key: PathBuf,
    /// Public key path.
    pub public_key: PathBuf,
    /// Comment/label.
    pub comment: Option<String>,
    /// Public key fingerprint.
    pub fingerprint: Option<String>,
}

/// Generate a new SSH key pair.
pub fn generate_ssh_key(
    key_type: SshKeyType,
    output_path: impl AsRef<Path>,
    comment: Option<&str>,
    passphrase: Option<&str>,
) -> GitResult<SshKeyPair> {
    use std::process::Command;

    let output_path = output_path.as_ref();
    let private_key = output_path.to_path_buf();
    let public_key = output_path.with_extension("pub");

    // Build ssh-keygen command
    let mut cmd = Command::new("ssh-keygen");
    cmd.arg("-t").arg(key_type.algorithm());
    cmd.arg("-f").arg(&private_key);
    cmd.arg("-N").arg(passphrase.unwrap_or(""));

    if let Some(comment) = comment {
        cmd.arg("-C").arg(comment);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(GitError::InvalidOperation {
            message: format!(
                "ssh-keygen failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    // Get fingerprint
    let fingerprint = get_key_fingerprint(&public_key).ok();

    Ok(SshKeyPair {
        key_type,
        private_key,
        public_key,
        comment: comment.map(String::from),
        fingerprint,
    })
}

/// Get SSH key fingerprint.
pub fn get_key_fingerprint(public_key: impl AsRef<Path>) -> GitResult<String> {
    use std::process::Command;

    let output = Command::new("ssh-keygen")
        .arg("-lf")
        .arg(public_key.as_ref())
        .output()?;

    if !output.status.success() {
        return Err(GitError::InvalidOperation {
            message: "Failed to get key fingerprint".to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let fingerprint = stdout
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .to_string();

    Ok(fingerprint)
}

/// List SSH keys in the default SSH directory.
pub fn list_ssh_keys() -> GitResult<Vec<SshKeyPair>> {
    let ssh_dir = crate::credentials::ssh::ssh_dir()
        .ok_or_else(|| GitError::InvalidOperation {
            message: "Could not find SSH directory".to_string(),
        })?;

    let mut keys = Vec::new();

    for key_type in &[SshKeyType::Ed25519, SshKeyType::Rsa, SshKeyType::Ecdsa] {
        let private_key = ssh_dir.join(key_type.filename());
        let public_key = private_key.with_extension("pub");

        if private_key.exists() {
            let fingerprint = if public_key.exists() {
                get_key_fingerprint(&public_key).ok()
            } else {
                None
            };

            keys.push(SshKeyPair {
                key_type: *key_type,
                private_key,
                public_key,
                comment: None,
                fingerprint,
            });
        }
    }

    Ok(keys)
}

/// Add key to SSH agent.
pub fn add_key_to_agent(key_path: impl AsRef<Path>) -> GitResult<()> {
    use std::process::Command;

    let output = Command::new("ssh-add")
        .arg(key_path.as_ref())
        .output()?;

    if !output.status.success() {
        return Err(GitError::InvalidOperation {
            message: format!(
                "ssh-add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    Ok(())
}

/// List keys in SSH agent.
pub fn list_agent_keys() -> GitResult<Vec<String>> {
    use std::process::Command;

    let output = Command::new("ssh-add")
        .arg("-l")
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new()); // Agent might be empty or not running
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let keys: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    Ok(keys)
}
```

---

## Testing Requirements

1. SSH URL parsing works for both formats
2. Key generation creates valid keys
3. Fingerprint extraction works
4. Agent interaction works
5. Key listing is complete

---

## Related Specs

- Depends on: [464-git-credentials.md](464-git-credentials.md)
- Next: [466-git-remote.md](466-git-remote.md)
