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

/// SSH configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfigEntry {
    /// Host pattern.
    pub host: String,
    /// Hostname to connect to.
    pub hostname: Option<String>,
    /// Port to connect to.
    pub port: Option<u16>,
    /// User to connect as.
    pub user: Option<String>,
    /// Identity file paths.
    pub identity_files: Vec<PathBuf>,
    /// Other configuration options.
    pub options: std::collections::HashMap<String, String>,
}

/// Parse SSH config file.
pub fn parse_ssh_config(config_path: impl AsRef<Path>) -> GitResult<Vec<SshConfigEntry>> {
    use std::fs;
    
    let content = fs::read_to_string(config_path)?;
    let mut entries = Vec::new();
    let mut current_entry: Option<SshConfigEntry> = None;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        let key = parts[0].to_lowercase();
        let value = parts[1..].join(" ");
        
        match key.as_str() {
            "host" => {
                // Save previous entry
                if let Some(entry) = current_entry.take() {
                    entries.push(entry);
                }
                
                // Start new entry
                current_entry = Some(SshConfigEntry {
                    host: value,
                    hostname: None,
                    port: None,
                    user: None,
                    identity_files: Vec::new(),
                    options: std::collections::HashMap::new(),
                });
            }
            "hostname" => {
                if let Some(ref mut entry) = current_entry {
                    entry.hostname = Some(value);
                }
            }
            "port" => {
                if let Some(ref mut entry) = current_entry {
                    entry.port = value.parse().ok();
                }
            }
            "user" => {
                if let Some(ref mut entry) = current_entry {
                    entry.user = Some(value);
                }
            }
            "identityfile" => {
                if let Some(ref mut entry) = current_entry {
                    entry.identity_files.push(PathBuf::from(value));
                }
            }
            _ => {
                if let Some(ref mut entry) = current_entry {
                    entry.options.insert(key, value);
                }
            }
        }
    }
    
    // Save last entry
    if let Some(entry) = current_entry {
        entries.push(entry);
    }
    
    Ok(entries)
}

/// Get SSH config for a specific host.
pub fn get_ssh_config(host: &str) -> GitResult<Option<SshConfigEntry>> {
    let ssh_dir = crate::credentials::ssh::ssh_dir()
        .ok_or_else(|| GitError::InvalidOperation {
            message: "Could not find SSH directory".to_string(),
        })?;
    
    let config_path = ssh_dir.join("config");
    if !config_path.exists() {
        return Ok(None);
    }
    
    let entries = parse_ssh_config(config_path)?;
    
    // Find matching entry (simple pattern matching for now)
    for entry in entries {
        if entry.host == host || entry.host == "*" || host_matches_pattern(host, &entry.host) {
            return Ok(Some(entry));
        }
    }
    
    Ok(None)
}

/// Check if host matches SSH config pattern.
fn host_matches_pattern(host: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    
    if pattern.contains('*') {
        // Simple wildcard matching
        if pattern.starts_with('*') {
            return host.ends_with(&pattern[1..]);
        }
        if pattern.ends_with('*') {
            return host.starts_with(&pattern[..pattern.len() - 1]);
        }
        // Middle wildcards would need more complex regex
        return false;
    }
    
    host == pattern
}

/// Known host entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownHost {
    /// Host pattern (can include port).
    pub host: String,
    /// Key type (ssh-rsa, ssh-ed25519, etc.).
    pub key_type: String,
    /// Public key data.
    pub key_data: String,
}

/// Parse known_hosts file.
pub fn parse_known_hosts(known_hosts_path: impl AsRef<Path>) -> GitResult<Vec<KnownHost>> {
    use std::fs;
    
    let content = fs::read_to_string(known_hosts_path)?;
    let mut hosts = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        
        let host = parts[0].to_string();
        let key_type = parts[1].to_string();
        let key_data = parts[2].to_string();
        
        hosts.push(KnownHost {
            host,
            key_type,
            key_data,
        });
    }
    
    Ok(hosts)
}

/// Check if a host is in known_hosts.
pub fn is_known_host(host: &str, port: Option<u16>) -> GitResult<bool> {
    let ssh_dir = crate::credentials::ssh::ssh_dir()
        .ok_or_else(|| GitError::InvalidOperation {
            message: "Could not find SSH directory".to_string(),
        })?;
    
    let known_hosts_path = ssh_dir.join("known_hosts");
    if !known_hosts_path.exists() {
        return Ok(false);
    }
    
    let hosts = parse_known_hosts(known_hosts_path)?;
    
    let host_pattern = if let Some(port) = port {
        format!("[{}]:{}", host, port)
    } else {
        host.to_string()
    };
    
    for known_host in hosts {
        if known_host.host == host_pattern || known_host.host == host {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Add a host to known_hosts.
pub fn add_known_host(host: &str, port: Option<u16>, key_type: &str, key_data: &str) -> GitResult<()> {
    let ssh_dir = crate::credentials::ssh::ssh_dir()
        .ok_or_else(|| GitError::InvalidOperation {
            message: "Could not find SSH directory".to_string(),
        })?;
    
    let known_hosts_path = ssh_dir.join("known_hosts");
    
    let host_pattern = if let Some(port) = port {
        format!("[{}]:{}", host, port)
    } else {
        host.to_string()
    };
    
    let entry = format!("{} {} {}\n", host_pattern, key_type, key_data);
    
    use std::fs::OpenOptions;
    use std::io::Write;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(known_hosts_path)?;
    
    file.write_all(entry.as_bytes())?;
    
    Ok(())
}

/// Get host key using ssh-keyscan.
pub fn get_host_key(host: &str, port: Option<u16>) -> GitResult<(String, String)> {
    use std::process::Command;
    
    let mut cmd = Command::new("ssh-keyscan");
    
    if let Some(port) = port {
        cmd.arg("-p").arg(port.to_string());
    }
    
    cmd.arg(host);
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        return Err(GitError::InvalidOperation {
            message: format!(
                "ssh-keyscan failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let key_type = parts[1].to_string();
            let key_data = parts[2].to_string();
            return Ok((key_type, key_data));
        }
    }
    
    Err(GitError::InvalidOperation {
        message: "No host key found".to_string(),
    })
}