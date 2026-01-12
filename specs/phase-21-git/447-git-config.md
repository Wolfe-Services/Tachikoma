# Spec 447: Git Configuration

## Phase
21 - Git Integration

## Spec ID
447

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 010: Error Handling (error types)

## Estimated Context
~9%

---

## Objective

Implement Git configuration management for Tachikoma, providing functionality to read, write, and manage Git configuration at system, global, and local repository levels. This includes handling user identity, remote URLs, merge preferences, and custom Tachikoma-specific settings.

---

## Acceptance Criteria

- [ ] Implement `GitConfig` struct for configuration access
- [ ] Support reading from system, global, and local configs
- [ ] Support writing configuration values
- [ ] Implement user identity management (name, email)
- [ ] Implement remote URL configuration
- [ ] Support custom Tachikoma configuration namespace
- [ ] Handle configuration inheritance correctly
- [ ] Support multi-value configuration entries
- [ ] Implement configuration validation
- [ ] Add configuration snapshot/restore functionality

---

## Implementation Details

### Configuration Manager

```rust
// src/git/config.rs

use git2::{Config, ConfigLevel};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use super::types::{GitError, GitResult};

/// Configuration level priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GitConfigLevel {
    /// System-wide configuration (/etc/gitconfig)
    System,
    /// Per-user configuration (~/.gitconfig)
    Global,
    /// Per-repository configuration (.git/config)
    Local,
    /// Worktree-specific configuration
    Worktree,
    /// Command-line configuration (highest priority)
    Command,
}

impl From<GitConfigLevel> for ConfigLevel {
    fn from(level: GitConfigLevel) -> Self {
        match level {
            GitConfigLevel::System => ConfigLevel::System,
            GitConfigLevel::Global => ConfigLevel::Global,
            GitConfigLevel::Local => ConfigLevel::Local,
            GitConfigLevel::Worktree => ConfigLevel::Worktree,
            GitConfigLevel::Command => ConfigLevel::App,
        }
    }
}

/// Git user identity
#[derive(Debug, Clone, Default)]
pub struct GitIdentity {
    pub name: Option<String>,
    pub email: Option<String>,
    pub signing_key: Option<String>,
}

impl GitIdentity {
    pub fn is_complete(&self) -> bool {
        self.name.is_some() && self.email.is_some()
    }
}

/// Git configuration manager
pub struct GitConfig {
    config: Config,
    repo_path: Option<PathBuf>,
}

impl GitConfig {
    /// Open the default Git configuration
    pub fn open_default() -> GitResult<Self> {
        let config = Config::open_default()?;
        Ok(Self {
            config,
            repo_path: None,
        })
    }

    /// Open configuration for a specific repository
    pub fn open_repo(repo_path: impl AsRef<Path>) -> GitResult<Self> {
        let repo = git2::Repository::open(repo_path.as_ref())?;
        let config = repo.config()?;
        Ok(Self {
            config,
            repo_path: Some(repo_path.as_ref().to_path_buf()),
        })
    }

    /// Open configuration at a specific level
    pub fn open_level(level: GitConfigLevel) -> GitResult<Self> {
        let path = match level {
            GitConfigLevel::Global => {
                let home = dirs::home_dir()
                    .ok_or_else(|| GitError::InvalidConfig("Cannot find home directory".into()))?;
                home.join(".gitconfig")
            }
            GitConfigLevel::System => PathBuf::from("/etc/gitconfig"),
            _ => return Err(GitError::InvalidConfig("Level requires repository context".into())),
        };

        let config = Config::open(&path)?;
        Ok(Self {
            config,
            repo_path: None,
        })
    }

    /// Get a string value
    pub fn get_string(&self, name: &str) -> GitResult<Option<String>> {
        match self.config.get_string(name) {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(GitError::Git2(e)),
        }
    }

    /// Get a boolean value
    pub fn get_bool(&self, name: &str) -> GitResult<Option<bool>> {
        match self.config.get_bool(name) {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(GitError::Git2(e)),
        }
    }

    /// Get an integer value
    pub fn get_i32(&self, name: &str) -> GitResult<Option<i32>> {
        match self.config.get_i32(name) {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(GitError::Git2(e)),
        }
    }

    /// Get an i64 value
    pub fn get_i64(&self, name: &str) -> GitResult<Option<i64>> {
        match self.config.get_i64(name) {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(GitError::Git2(e)),
        }
    }

    /// Get all values for a multi-valued key
    pub fn get_multivar(&self, name: &str, regexp: Option<&str>) -> GitResult<Vec<String>> {
        let mut values = Vec::new();
        let entries = self.config.multivar(name, regexp)?;
        entries.for_each(|entry| {
            if let Some(value) = entry.value() {
                values.push(value.to_string());
            }
        })?;
        Ok(values)
    }

    /// Set a string value
    pub fn set_string(&mut self, name: &str, value: &str) -> GitResult<()> {
        self.config.set_str(name, value)?;
        Ok(())
    }

    /// Set a boolean value
    pub fn set_bool(&mut self, name: &str, value: bool) -> GitResult<()> {
        self.config.set_bool(name, value)?;
        Ok(())
    }

    /// Set an integer value
    pub fn set_i32(&mut self, name: &str, value: i32) -> GitResult<()> {
        self.config.set_i32(name, value)?;
        Ok(())
    }

    /// Set an i64 value
    pub fn set_i64(&mut self, name: &str, value: i64) -> GitResult<()> {
        self.config.set_i64(name, value)?;
        Ok(())
    }

    /// Remove a configuration entry
    pub fn remove(&mut self, name: &str) -> GitResult<()> {
        self.config.remove(name)?;
        Ok(())
    }

    /// Remove all matching entries for a multi-valued key
    pub fn remove_multivar(&mut self, name: &str, regexp: &str) -> GitResult<()> {
        self.config.remove_multivar(name, regexp)?;
        Ok(())
    }

    /// Get user identity from configuration
    pub fn get_identity(&self) -> GitResult<GitIdentity> {
        Ok(GitIdentity {
            name: self.get_string("user.name")?,
            email: self.get_string("user.email")?,
            signing_key: self.get_string("user.signingkey")?,
        })
    }

    /// Set user identity
    pub fn set_identity(&mut self, identity: &GitIdentity) -> GitResult<()> {
        if let Some(ref name) = identity.name {
            self.set_string("user.name", name)?;
        }
        if let Some(ref email) = identity.email {
            self.set_string("user.email", email)?;
        }
        if let Some(ref key) = identity.signing_key {
            self.set_string("user.signingkey", key)?;
        }
        Ok(())
    }

    /// Get all entries with a given prefix
    pub fn get_entries_with_prefix(&self, prefix: &str) -> GitResult<HashMap<String, String>> {
        let mut entries = HashMap::new();
        let iterator = self.config.entries(Some(&format!("{}.*", prefix)))?;

        iterator.for_each(|entry| {
            if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
                entries.insert(name.to_string(), value.to_string());
            }
        })?;

        Ok(entries)
    }

    /// Create a snapshot for later comparison
    pub fn snapshot(&self) -> GitResult<ConfigSnapshot> {
        let mut values = HashMap::new();
        let iterator = self.config.entries(None)?;

        iterator.for_each(|entry| {
            if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
                values.insert(name.to_string(), value.to_string());
            }
        })?;

        Ok(ConfigSnapshot { values })
    }
}

/// Configuration snapshot for comparison
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    values: HashMap<String, String>,
}

impl ConfigSnapshot {
    /// Get differences from another snapshot
    pub fn diff(&self, other: &ConfigSnapshot) -> ConfigDiff {
        let mut added = HashMap::new();
        let mut removed = HashMap::new();
        let mut changed = HashMap::new();

        // Find added and changed
        for (key, value) in &other.values {
            match self.values.get(key) {
                None => {
                    added.insert(key.clone(), value.clone());
                }
                Some(old_value) if old_value != value => {
                    changed.insert(key.clone(), (old_value.clone(), value.clone()));
                }
                _ => {}
            }
        }

        // Find removed
        for (key, value) in &self.values {
            if !other.values.contains_key(key) {
                removed.insert(key.clone(), value.clone());
            }
        }

        ConfigDiff {
            added,
            removed,
            changed,
        }
    }
}

/// Differences between two configuration snapshots
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    pub added: HashMap<String, String>,
    pub removed: HashMap<String, String>,
    pub changed: HashMap<String, (String, String)>,
}

impl ConfigDiff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }
}

/// Tachikoma-specific configuration
pub struct TachikomaConfig {
    config: GitConfig,
}

impl TachikomaConfig {
    const PREFIX: &'static str = "tachikoma";

    pub fn new(config: GitConfig) -> Self {
        Self { config }
    }

    /// Get AI model preference
    pub fn get_ai_model(&self) -> GitResult<Option<String>> {
        self.config.get_string(&format!("{}.ai.model", Self::PREFIX))
    }

    /// Set AI model preference
    pub fn set_ai_model(&mut self, model: &str) -> GitResult<()> {
        self.config.set_string(&format!("{}.ai.model", Self::PREFIX), model)
    }

    /// Get auto-commit message generation setting
    pub fn get_auto_message(&self) -> GitResult<bool> {
        Ok(self.config
            .get_bool(&format!("{}.commit.automessage", Self::PREFIX))?
            .unwrap_or(false))
    }

    /// Set auto-commit message generation
    pub fn set_auto_message(&mut self, enabled: bool) -> GitResult<()> {
        self.config.set_bool(&format!("{}.commit.automessage", Self::PREFIX), enabled)
    }

    /// Get code review on commit setting
    pub fn get_review_on_commit(&self) -> GitResult<bool> {
        Ok(self.config
            .get_bool(&format!("{}.commit.review", Self::PREFIX))?
            .unwrap_or(false))
    }

    /// Get all Tachikoma settings
    pub fn get_all(&self) -> GitResult<HashMap<String, String>> {
        self.config.get_entries_with_prefix(Self::PREFIX)
    }
}

/// Configuration validation
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate email format
    pub fn validate_email(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        !parts[0].is_empty() && !parts[1].is_empty() && parts[1].contains('.')
    }

    /// Validate remote URL
    pub fn validate_remote_url(url: &str) -> bool {
        // SSH URL
        if url.starts_with("git@") && url.contains(':') {
            return true;
        }
        // HTTPS URL
        if url.starts_with("https://") || url.starts_with("http://") {
            return true;
        }
        // Git protocol
        if url.starts_with("git://") {
            return true;
        }
        // Local path
        if url.starts_with('/') || url.starts_with("file://") {
            return true;
        }
        false
    }

    /// Validate branch name
    pub fn validate_branch_name(name: &str) -> bool {
        if name.is_empty() || name.starts_with('-') || name.ends_with('.') {
            return false;
        }
        // Check for invalid characters and sequences
        !name.contains("..")
            && !name.contains("//")
            && !name.contains("@{")
            && !name.contains('\\')
            && !name.contains('\x00')
            && !name.ends_with(".lock")
    }
}
```

### Scoped Configuration Writer

```rust
// src/git/config/scoped.rs

use super::*;

/// Scoped configuration writer that restores original values on drop
pub struct ScopedConfig<'a> {
    config: &'a mut GitConfig,
    original_values: HashMap<String, Option<String>>,
}

impl<'a> ScopedConfig<'a> {
    pub fn new(config: &'a mut GitConfig) -> Self {
        Self {
            config,
            original_values: HashMap::new(),
        }
    }

    /// Set a value temporarily
    pub fn set(&mut self, name: &str, value: &str) -> GitResult<()> {
        // Store original value if not already stored
        if !self.original_values.contains_key(name) {
            let original = self.config.get_string(name)?;
            self.original_values.insert(name.to_string(), original);
        }
        self.config.set_string(name, value)
    }

    /// Commit changes (don't restore on drop)
    pub fn commit(mut self) {
        self.original_values.clear();
    }
}

impl Drop for ScopedConfig<'_> {
    fn drop(&mut self) {
        for (name, original) in &self.original_values {
            match original {
                Some(value) => {
                    let _ = self.config.set_string(name, value);
                }
                None => {
                    let _ = self.config.remove(name);
                }
            }
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

    fn setup_test_repo() -> (TempDir, git2::Repository) {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_config_string_operations() {
        let (dir, _repo) = setup_test_repo();
        let mut config = GitConfig::open_repo(dir.path()).unwrap();

        // Set and get string
        config.set_string("test.key", "value").unwrap();
        assert_eq!(config.get_string("test.key").unwrap(), Some("value".to_string()));

        // Remove
        config.remove("test.key").unwrap();
        assert_eq!(config.get_string("test.key").unwrap(), None);
    }

    #[test]
    fn test_config_bool_operations() {
        let (dir, _repo) = setup_test_repo();
        let mut config = GitConfig::open_repo(dir.path()).unwrap();

        config.set_bool("test.enabled", true).unwrap();
        assert_eq!(config.get_bool("test.enabled").unwrap(), Some(true));

        config.set_bool("test.enabled", false).unwrap();
        assert_eq!(config.get_bool("test.enabled").unwrap(), Some(false));
    }

    #[test]
    fn test_identity_management() {
        let (dir, _repo) = setup_test_repo();
        let mut config = GitConfig::open_repo(dir.path()).unwrap();

        let identity = GitIdentity {
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            signing_key: None,
        };

        config.set_identity(&identity).unwrap();

        let retrieved = config.get_identity().unwrap();
        assert_eq!(retrieved.name, Some("Test User".to_string()));
        assert_eq!(retrieved.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_config_snapshot_diff() {
        let (dir, _repo) = setup_test_repo();
        let mut config = GitConfig::open_repo(dir.path()).unwrap();

        config.set_string("test.a", "1").unwrap();
        config.set_string("test.b", "2").unwrap();

        let snapshot1 = config.snapshot().unwrap();

        config.set_string("test.b", "changed").unwrap();
        config.set_string("test.c", "3").unwrap();
        config.remove("test.a").unwrap();

        let snapshot2 = config.snapshot().unwrap();

        let diff = snapshot1.diff(&snapshot2);
        assert!(diff.added.contains_key("test.c"));
        assert!(diff.removed.contains_key("test.a"));
        assert!(diff.changed.contains_key("test.b"));
    }

    #[test]
    fn test_email_validation() {
        assert!(ConfigValidator::validate_email("user@example.com"));
        assert!(ConfigValidator::validate_email("user.name@sub.domain.com"));
        assert!(!ConfigValidator::validate_email("invalid"));
        assert!(!ConfigValidator::validate_email("@example.com"));
        assert!(!ConfigValidator::validate_email("user@"));
    }

    #[test]
    fn test_remote_url_validation() {
        assert!(ConfigValidator::validate_remote_url("https://github.com/user/repo.git"));
        assert!(ConfigValidator::validate_remote_url("git@github.com:user/repo.git"));
        assert!(ConfigValidator::validate_remote_url("git://github.com/user/repo.git"));
        assert!(ConfigValidator::validate_remote_url("/local/path/to/repo"));
        assert!(!ConfigValidator::validate_remote_url("invalid-url"));
    }

    #[test]
    fn test_branch_name_validation() {
        assert!(ConfigValidator::validate_branch_name("main"));
        assert!(ConfigValidator::validate_branch_name("feature/new-feature"));
        assert!(ConfigValidator::validate_branch_name("release-1.0"));
        assert!(!ConfigValidator::validate_branch_name("-invalid"));
        assert!(!ConfigValidator::validate_branch_name("invalid..name"));
        assert!(!ConfigValidator::validate_branch_name("name.lock"));
    }

    #[test]
    fn test_scoped_config() {
        let (dir, _repo) = setup_test_repo();
        let mut config = GitConfig::open_repo(dir.path()).unwrap();

        config.set_string("test.original", "original_value").unwrap();

        {
            let mut scoped = ScopedConfig::new(&mut config);
            scoped.set("test.original", "temporary_value").unwrap();
            scoped.set("test.new", "new_value").unwrap();

            // Values are temporary
            assert_eq!(config.get_string("test.original").unwrap(), Some("temporary_value".to_string()));
        }
        // ScopedConfig dropped - values restored

        assert_eq!(config.get_string("test.original").unwrap(), Some("original_value".to_string()));
        assert_eq!(config.get_string("test.new").unwrap(), None);
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 463: Credential Management
