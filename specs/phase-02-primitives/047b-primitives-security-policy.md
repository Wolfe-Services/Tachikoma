# 047b - Primitives Security Policy

**Phase:** 2 - The Five Primitives
**Spec ID:** 047b
**Status:** Planned
**Dependencies:** 047-primitives-validation, 036-bash-exec-core
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define and enforce security constraints for primitive execution, especially the `bash` primitive which has system-level access. This spec establishes the enforcement points, not just documentation.

---

## Security Principles

1. **Defense in Depth**: Multiple enforcement layers, not just prompts
2. **Principle of Least Privilege**: Primitives only access what's needed
3. **Fail Secure**: Deny by default, explicit allow
4. **Audit Everything**: All primitive executions are logged
5. **No Security by Obscurity**: Constraints are explicit and documented

---

## Acceptance Criteria

- [ ] `SecurityPolicy` struct with all constraint types
- [ ] Path allowlist/blocklist enforcement
- [ ] Command blocklist for dangerous operations
- [ ] Environment variable filtering
- [ ] Output sanitizer for secrets redaction
- [ ] Policy loading from config
- [ ] Policy validation before primitive execution
- [ ] Audit logging of policy violations

---

## Implementation Details

### 1. Security Policy Types (src/security.rs)

```rust
//! Security policy enforcement for primitives.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use regex::Regex;

/// Security policy for primitive execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Paths that primitives can read from.
    pub read_allowlist: Vec<PathBuf>,
    /// Paths that primitives can write to.
    pub write_allowlist: Vec<PathBuf>,
    /// Paths that are never accessible (overrides allowlist).
    pub path_blocklist: Vec<PathBuf>,
    /// Commands that cannot be executed.
    pub command_blocklist: Vec<String>,
    /// Command patterns that cannot be executed (regex).
    pub command_pattern_blocklist: Vec<String>,
    /// Environment variables to filter from bash execution.
    pub env_blocklist: Vec<String>,
    /// Patterns to redact from output (secrets).
    pub redaction_patterns: Vec<String>,
    /// Maximum file size for read operations (bytes).
    pub max_read_size: u64,
    /// Maximum output size for bash (bytes).
    pub max_output_size: u64,
    /// Whether to allow network access in bash.
    pub allow_network: bool,
    /// Workspace root (all paths must be within).
    pub workspace_root: PathBuf,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            read_allowlist: vec![],  // Empty = workspace only
            write_allowlist: vec![], // Empty = workspace only
            path_blocklist: vec![
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/etc/shadow"),
                PathBuf::from("/etc/sudoers"),
                PathBuf::from("~/.ssh"),
                PathBuf::from("~/.gnupg"),
                PathBuf::from("~/.aws/credentials"),
                PathBuf::from(".env"),
                PathBuf::from(".env.local"),
                PathBuf::from("*.pem"),
                PathBuf::from("*.key"),
            ],
            command_blocklist: vec![
                // Destructive commands
                "rm -rf /".to_string(),
                "rm -rf /*".to_string(),
                "mkfs".to_string(),
                "dd if=/dev/zero".to_string(),
                ":(){ :|:& };:".to_string(), // Fork bomb
                // Privilege escalation
                "sudo".to_string(),
                "su ".to_string(),
                "chmod 777".to_string(),
                "chown root".to_string(),
                // Network exfiltration
                "curl | sh".to_string(),
                "wget | sh".to_string(),
                "nc -e".to_string(),
                "netcat -e".to_string(),
                // Crypto mining / malware
                "xmrig".to_string(),
                "minerd".to_string(),
            ],
            command_pattern_blocklist: vec![
                r"rm\s+-rf\s+/".to_string(),
                r">\s*/dev/sd[a-z]".to_string(),
                r"curl.*\|\s*(ba)?sh".to_string(),
                r"wget.*\|\s*(ba)?sh".to_string(),
            ],
            env_blocklist: vec![
                "AWS_SECRET_ACCESS_KEY".to_string(),
                "AWS_SESSION_TOKEN".to_string(),
                "ANTHROPIC_API_KEY".to_string(),
                "OPENAI_API_KEY".to_string(),
                "GITHUB_TOKEN".to_string(),
                "NPM_TOKEN".to_string(),
                "DATABASE_URL".to_string(),
            ],
            redaction_patterns: vec![
                // API keys
                r"sk-[a-zA-Z0-9]{32,}".to_string(),
                r"api[_-]?key['\"]?\s*[:=]\s*['\"]?[a-zA-Z0-9]{20,}".to_string(),
                // AWS
                r"AKIA[0-9A-Z]{16}".to_string(),
                // Private keys
                r"-----BEGIN [A-Z]+ PRIVATE KEY-----".to_string(),
                // Passwords in URLs
                r"://[^:]+:[^@]+@".to_string(),
            ],
            max_read_size: 10 * 1024 * 1024, // 10MB
            max_output_size: 1 * 1024 * 1024, // 1MB
            allow_network: true,
            workspace_root: PathBuf::from("."),
        }
    }
}

impl SecurityPolicy {
    /// Create a strict policy for untrusted execution.
    pub fn strict(workspace: PathBuf) -> Self {
        Self {
            workspace_root: workspace.clone(),
            read_allowlist: vec![workspace.clone()],
            write_allowlist: vec![workspace],
            allow_network: false,
            ..Default::default()
        }
    }

    /// Check if a path is allowed for reading.
    pub fn can_read(&self, path: &Path) -> Result<(), SecurityViolation> {
        self.check_path_allowed(path, &self.read_allowlist, "read")
    }

    /// Check if a path is allowed for writing.
    pub fn can_write(&self, path: &Path) -> Result<(), SecurityViolation> {
        self.check_path_allowed(path, &self.write_allowlist, "write")
    }

    /// Check if a command is allowed.
    pub fn can_execute(&self, command: &str) -> Result<(), SecurityViolation> {
        // Check exact blocklist
        for blocked in &self.command_blocklist {
            if command.contains(blocked) {
                return Err(SecurityViolation::BlockedCommand {
                    command: command.to_string(),
                    reason: format!("Contains blocked pattern: {}", blocked),
                });
            }
        }

        // Check regex patterns
        for pattern in &self.command_pattern_blocklist {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(command) {
                    return Err(SecurityViolation::BlockedCommand {
                        command: command.to_string(),
                        reason: format!("Matches blocked pattern: {}", pattern),
                    });
                }
            }
        }

        Ok(())
    }

    /// Filter environment variables for bash execution.
    pub fn filter_env(&self, env: &[(String, String)]) -> Vec<(String, String)> {
        env.iter()
            .filter(|(k, _)| !self.env_blocklist.contains(k))
            .cloned()
            .collect()
    }

    /// Redact secrets from output.
    pub fn redact_output(&self, output: &str) -> String {
        let mut result = output.to_string();
        for pattern in &self.redaction_patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[REDACTED]").to_string();
            }
        }
        result
    }

    fn check_path_allowed(
        &self,
        path: &Path,
        allowlist: &[PathBuf],
        operation: &str,
    ) -> Result<(), SecurityViolation> {
        // Canonicalize to prevent traversal attacks
        let canonical = path.canonicalize().map_err(|e| SecurityViolation::PathError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        // Check blocklist first (always denied)
        for blocked in &self.path_blocklist {
            if canonical.starts_with(blocked) || path_matches_glob(&canonical, blocked) {
                return Err(SecurityViolation::BlockedPath {
                    path: canonical,
                    reason: "Path is in blocklist".to_string(),
                });
            }
        }

        // Must be within workspace
        if !canonical.starts_with(&self.workspace_root) {
            return Err(SecurityViolation::OutsideWorkspace {
                path: canonical,
                workspace: self.workspace_root.clone(),
            });
        }

        // If allowlist is empty, workspace is the allowlist
        if allowlist.is_empty() {
            return Ok(());
        }

        // Check allowlist
        for allowed in allowlist {
            if canonical.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(SecurityViolation::NotInAllowlist {
            path: canonical,
            operation: operation.to_string(),
        })
    }
}

/// Security violation error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SecurityViolation {
    #[error("Blocked path: {path:?} - {reason}")]
    BlockedPath { path: PathBuf, reason: String },

    #[error("Path outside workspace: {path:?} (workspace: {workspace:?})")]
    OutsideWorkspace { path: PathBuf, workspace: PathBuf },

    #[error("Path not in {operation} allowlist: {path:?}")]
    NotInAllowlist { path: PathBuf, operation: String },

    #[error("Blocked command: {command} - {reason}")]
    BlockedCommand { command: String, reason: String },

    #[error("Path error for {path:?}: {reason}")]
    PathError { path: PathBuf, reason: String },
}

fn path_matches_glob(path: &Path, pattern: &Path) -> bool {
    // Simple glob matching for *.ext patterns
    if let Some(pattern_str) = pattern.to_str() {
        if pattern_str.starts_with("*.") {
            if let Some(ext) = path.extension() {
                return pattern_str.trim_start_matches("*.") == ext;
            }
        }
    }
    false
}
```

### 2. Policy Enforcement in Context (update src/context.rs)

```rust
use crate::security::{SecurityPolicy, SecurityViolation};

impl PrimitiveContext {
    /// Validate read operation against security policy.
    pub fn validate_read(&self, path: &Path) -> Result<(), SecurityViolation> {
        self.security_policy.can_read(path)
    }

    /// Validate write operation against security policy.
    pub fn validate_write(&self, path: &Path) -> Result<(), SecurityViolation> {
        self.security_policy.can_write(path)
    }

    /// Validate command execution against security policy.
    pub fn validate_command(&self, command: &str) -> Result<(), SecurityViolation> {
        self.security_policy.can_execute(command)
    }
}
```

---

## Testing Requirements

1. Path blocklist prevents access to sensitive files
2. Command blocklist catches dangerous commands
3. Regex patterns catch obfuscated dangerous commands
4. Environment filtering removes sensitive variables
5. Output redaction catches API keys and secrets
6. Workspace boundary is enforced
7. Allowlist correctly restricts to specified paths

---

## Audit Logging

All security violations MUST be logged with:
- Timestamp
- Primitive type
- Attempted operation
- Violation type
- Full context (sanitized)

---

## Related Specs

- Depends on: [047-primitives-validation.md](047-primitives-validation.md)
- Depends on: [036-bash-exec-core.md](036-bash-exec-core.md)
- Related: [048-primitives-audit.md](048-primitives-audit.md)
