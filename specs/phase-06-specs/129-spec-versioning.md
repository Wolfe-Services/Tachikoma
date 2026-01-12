# Spec 129: Spec Versioning

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 129
- **Status**: Planned
- **Dependencies**: 120-spec-parsing, 121-spec-metadata
- **Estimated Context**: ~9%

## Objective

Implement versioning for specifications to track changes over time, support rollback, and maintain history. Versioning integrates with git when available but also provides standalone versioning capabilities for non-git environments.

## Acceptance Criteria

- [ ] Spec versions are tracked with semantic versioning
- [ ] Change history is maintained
- [ ] Rollback to previous versions works
- [ ] Git integration extracts version history
- [ ] Version comparison (diff) is supported
- [ ] Version metadata is embedded in specs
- [ ] Breaking changes are flagged
- [ ] Version migration is supported

## Implementation Details

### Versioning System

```rust
// src/spec/versioning.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::fs;

/// Semantic version for specs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SpecVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl SpecVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('v');
        let parts: Vec<&str> = s.split('-').collect();
        let version_parts: Vec<&str> = parts[0].split('.').collect();

        if version_parts.len() < 2 {
            return None;
        }

        Some(Self {
            major: version_parts.get(0)?.parse().ok()?,
            minor: version_parts.get(1)?.parse().ok()?,
            patch: version_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            pre_release: parts.get(1).map(|s| s.to_string()),
        })
    }

    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }

    pub fn is_breaking_change(&self, other: &Self) -> bool {
        self.major != other.major
    }
}

impl std::fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

impl Default for SpecVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

/// A version entry in the history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Version number
    pub version: SpecVersion,
    /// Timestamp of version
    pub timestamp: DateTime<Utc>,
    /// Author/committer
    pub author: Option<String>,
    /// Change description
    pub description: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Git commit hash (if available)
    pub commit_hash: Option<String>,
    /// Snapshot of spec content (optional)
    pub content_hash: Option<String>,
}

/// Type of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// New spec created
    Created,
    /// Minor content update
    Updated,
    /// Breaking structural change
    Breaking,
    /// Status change only
    StatusChange,
    /// Acceptance criteria change
    CriteriaChange,
    /// Implementation change
    ImplementationChange,
    /// Deprecated
    Deprecated,
}

/// Version history for a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    /// Spec ID
    pub spec_id: u32,
    /// Current version
    pub current: SpecVersion,
    /// Version entries
    pub entries: Vec<VersionEntry>,
    /// Deprecated versions
    pub deprecated_versions: Vec<SpecVersion>,
}

impl VersionHistory {
    pub fn new(spec_id: u32) -> Self {
        Self {
            spec_id,
            current: SpecVersion::default(),
            entries: Vec::new(),
            deprecated_versions: Vec::new(),
        }
    }

    /// Add a new version entry
    pub fn add_entry(&mut self, entry: VersionEntry) {
        self.current = entry.version.clone();
        self.entries.push(entry);
    }

    /// Get entry by version
    pub fn get_entry(&self, version: &SpecVersion) -> Option<&VersionEntry> {
        self.entries.iter().find(|e| &e.version == version)
    }

    /// Get latest N entries
    pub fn recent(&self, n: usize) -> &[VersionEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    /// Check if version exists
    pub fn has_version(&self, version: &SpecVersion) -> bool {
        self.entries.iter().any(|e| &e.version == version)
    }
}

/// Version manager
pub struct VersionManager {
    /// Version histories by spec ID
    histories: HashMap<u32, VersionHistory>,
    /// Storage path for version data
    storage_path: PathBuf,
    /// Git integration
    git: Option<GitVersioning>,
}

impl VersionManager {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            histories: HashMap::new(),
            storage_path,
            git: None,
        }
    }

    /// Enable git integration
    pub fn with_git(mut self, repo_path: PathBuf) -> Self {
        self.git = Some(GitVersioning::new(repo_path));
        self
    }

    /// Load version history for a spec
    pub async fn load_history(&mut self, spec_id: u32) -> Result<&VersionHistory, VersionError> {
        if !self.histories.contains_key(&spec_id) {
            let history = self.load_from_storage(spec_id).await
                .unwrap_or_else(|_| VersionHistory::new(spec_id));
            self.histories.insert(spec_id, history);
        }

        Ok(self.histories.get(&spec_id).unwrap())
    }

    /// Load from storage
    async fn load_from_storage(&self, spec_id: u32) -> Result<VersionHistory, VersionError> {
        let path = self.storage_path.join(format!("{:03}.version.json", spec_id));
        let content = fs::read_to_string(&path).await?;
        let history: VersionHistory = serde_json::from_str(&content)?;
        Ok(history)
    }

    /// Save to storage
    async fn save_to_storage(&self, history: &VersionHistory) -> Result<(), VersionError> {
        let path = self.storage_path.join(format!("{:03}.version.json", history.spec_id));
        let content = serde_json::to_string_pretty(history)?;
        fs::create_dir_all(&self.storage_path).await?;
        fs::write(&path, content).await?;
        Ok(())
    }

    /// Record a new version
    pub async fn record_version(
        &mut self,
        spec_id: u32,
        description: &str,
        change_type: ChangeType,
        author: Option<String>,
    ) -> Result<SpecVersion, VersionError> {
        let history = self.histories.entry(spec_id)
            .or_insert_with(|| VersionHistory::new(spec_id));

        // Determine new version based on change type
        let new_version = match change_type {
            ChangeType::Breaking => history.current.bump_major(),
            ChangeType::Created => SpecVersion::default(),
            ChangeType::StatusChange | ChangeType::CriteriaChange => history.current.bump_patch(),
            _ => history.current.bump_minor(),
        };

        let entry = VersionEntry {
            version: new_version.clone(),
            timestamp: Utc::now(),
            author,
            description: description.to_string(),
            change_type,
            commit_hash: self.get_current_commit().await,
            content_hash: None,
        };

        history.add_entry(entry);
        self.save_to_storage(history).await?;

        Ok(new_version)
    }

    /// Get current git commit
    async fn get_current_commit(&self) -> Option<String> {
        self.git.as_ref()?.get_head_commit().await.ok()
    }

    /// Get version history
    pub fn get_history(&self, spec_id: u32) -> Option<&VersionHistory> {
        self.histories.get(&spec_id)
    }

    /// Sync with git history
    pub async fn sync_from_git(&mut self, spec_id: u32, spec_path: &Path) -> Result<(), VersionError> {
        let git = self.git.as_ref()
            .ok_or(VersionError::GitNotAvailable)?;

        let commits = git.get_file_history(spec_path).await?;

        let history = self.histories.entry(spec_id)
            .or_insert_with(|| VersionHistory::new(spec_id));

        for (i, commit) in commits.iter().enumerate() {
            let version = SpecVersion::new(1, i as u32, 0);

            if !history.has_version(&version) {
                history.add_entry(VersionEntry {
                    version,
                    timestamp: commit.timestamp,
                    author: Some(commit.author.clone()),
                    description: commit.message.clone(),
                    change_type: if i == 0 { ChangeType::Created } else { ChangeType::Updated },
                    commit_hash: Some(commit.hash.clone()),
                    content_hash: None,
                });
            }
        }

        self.save_to_storage(history).await?;
        Ok(())
    }
}

/// Git versioning integration
pub struct GitVersioning {
    repo_path: PathBuf,
}

/// Git commit info
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl GitVersioning {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    /// Get HEAD commit hash
    pub async fn get_head_commit(&self) -> Result<String, VersionError> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get file history from git
    pub async fn get_file_history(&self, file_path: &Path) -> Result<Vec<GitCommit>, VersionError> {
        let output = tokio::process::Command::new("git")
            .args([
                "log",
                "--format=%H|%an|%s|%aI",
                "--follow",
                "--",
                file_path.to_str().unwrap(),
            ])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                commits.push(GitCommit {
                    hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    message: parts[2].to_string(),
                    timestamp: DateTime::parse_from_rfc3339(parts[3])
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                });
            }
        }

        // Reverse to get oldest first
        commits.reverse();
        Ok(commits)
    }

    /// Get file content at specific commit
    pub async fn get_file_at_commit(
        &self,
        file_path: &Path,
        commit: &str,
    ) -> Result<String, VersionError> {
        let output = tokio::process::Command::new("git")
            .args(["show", &format!("{}:{}", commit, file_path.to_str().unwrap())])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(VersionError::GitError("Failed to get file content".to_string()))
        }
    }
}

/// Version comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparison {
    pub from_version: SpecVersion,
    pub to_version: SpecVersion,
    pub changes: Vec<VersionChange>,
    pub is_breaking: bool,
}

/// A specific change between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub section: String,
    pub change_type: DiffType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
}

/// Version errors
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Git not available")]
    GitNotAvailable,

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Version not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = SpecVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        let v = SpecVersion::parse("v2.0.0-beta").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.pre_release, Some("beta".to_string()));
    }

    #[test]
    fn test_version_bumping() {
        let v = SpecVersion::new(1, 2, 3);

        assert_eq!(v.bump_major(), SpecVersion::new(2, 0, 0));
        assert_eq!(v.bump_minor(), SpecVersion::new(1, 3, 0));
        assert_eq!(v.bump_patch(), SpecVersion::new(1, 2, 4));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = SpecVersion::new(1, 0, 0);
        let v2 = SpecVersion::new(2, 0, 0);

        assert!(v2 > v1);
        assert!(v1.is_breaking_change(&v2));
    }

    #[test]
    fn test_version_history() {
        let mut history = VersionHistory::new(116);

        history.add_entry(VersionEntry {
            version: SpecVersion::new(1, 0, 0),
            timestamp: Utc::now(),
            author: Some("test".to_string()),
            description: "Initial version".to_string(),
            change_type: ChangeType::Created,
            commit_hash: None,
            content_hash: None,
        });

        assert_eq!(history.current, SpecVersion::new(1, 0, 0));
        assert!(history.has_version(&SpecVersion::new(1, 0, 0)));
    }
}
```

## Testing Requirements

- [ ] Unit tests for version parsing
- [ ] Tests for version bumping
- [ ] Tests for version history
- [ ] Tests for version comparison
- [ ] Tests for git integration
- [ ] Tests for storage persistence
- [ ] Integration tests with real specs
- [ ] Tests for breaking change detection

## Related Specs

- **130-spec-diff.md**: Diff generation between versions
- **120-spec-parsing.md**: Parsing for version comparison
- **121-spec-metadata.md**: Version in metadata
