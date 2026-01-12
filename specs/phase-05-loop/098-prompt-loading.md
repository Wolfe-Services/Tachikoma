# 098 - Prompt Loading

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 098
**Status:** Planned
**Dependencies:** 096-loop-runner-core, 029-file-system-utilities
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement the prompt.md loading system for the Ralph Loop - reading, validating, and preparing prompts from the filesystem with support for includes, frontmatter metadata, and file watching.

---

## Acceptance Criteria

- [ ] Load prompt.md from configured path
- [ ] Parse YAML frontmatter metadata
- [ ] Support for include directives
- [ ] File watching for prompt changes
- [ ] Prompt validation
- [ ] Caching with invalidation
- [ ] Support for multiple prompt files
- [ ] Error handling with helpful messages

---

## Implementation Details

### 1. Prompt Types (src/prompt/types.rs)

```rust
//! Prompt type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A loaded prompt with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// The source file path.
    pub source_path: PathBuf,
    /// Parsed frontmatter metadata.
    pub metadata: PromptMetadata,
    /// The prompt content (after frontmatter).
    pub content: String,
    /// Resolved includes.
    pub includes: Vec<IncludedContent>,
    /// Last modified timestamp.
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Content hash for cache invalidation.
    pub content_hash: String,
}

/// Prompt frontmatter metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptMetadata {
    /// Prompt name/title.
    #[serde(default)]
    pub name: Option<String>,

    /// Prompt description.
    #[serde(default)]
    pub description: Option<String>,

    /// Version string.
    #[serde(default)]
    pub version: Option<String>,

    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Variables that must be provided.
    #[serde(default)]
    pub required_vars: Vec<String>,

    /// Default variable values.
    #[serde(default)]
    pub defaults: HashMap<String, String>,

    /// Iteration-specific settings.
    #[serde(default)]
    pub iteration: IterationSettings,

    /// Custom metadata.
    #[serde(flatten)]
    pub custom: HashMap<String, serde_yaml::Value>,
}

/// Iteration settings from frontmatter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IterationSettings {
    /// Maximum context usage before suggesting reboot.
    #[serde(default)]
    pub context_threshold: Option<u8>,

    /// Whether to run tests after changes.
    #[serde(default)]
    pub run_tests: Option<bool>,

    /// Custom stop conditions.
    #[serde(default)]
    pub stop_on: Vec<String>,
}

/// Content included from another file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludedContent {
    /// The include directive path.
    pub directive_path: String,
    /// Resolved absolute path.
    pub resolved_path: PathBuf,
    /// The included content.
    pub content: String,
    /// Position in the prompt where include was found.
    pub position: usize,
}

impl Prompt {
    /// Get the fully rendered content with includes.
    pub fn render(&self) -> String {
        let mut result = self.content.clone();

        // Replace includes in reverse order to maintain positions
        for include in self.includes.iter().rev() {
            let directive = format!("{{{{include:{}}}}}", include.directive_path);
            result = result.replace(&directive, &include.content);
        }

        result
    }

    /// Validate the prompt.
    pub fn validate(&self) -> Result<(), PromptValidationError> {
        // Check content is not empty
        if self.content.trim().is_empty() {
            return Err(PromptValidationError::EmptyContent);
        }

        // Check required vars are mentioned
        for var in &self.metadata.required_vars {
            let pattern = format!("{{{{{}}}}}", var);
            if !self.content.contains(&pattern) {
                return Err(PromptValidationError::UnusedRequiredVar {
                    var: var.clone(),
                });
            }
        }

        Ok(())
    }

    /// Check if prompt has unresolved variables.
    pub fn unresolved_vars(&self) -> Vec<String> {
        let re = regex::Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").unwrap();
        re.captures_iter(&self.content)
            .map(|cap| cap[1].to_string())
            .collect()
    }
}

/// Prompt validation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PromptValidationError {
    #[error("prompt content is empty")]
    EmptyContent,

    #[error("required variable '{var}' is declared but not used in prompt")]
    UnusedRequiredVar { var: String },

    #[error("unresolved variable: {var}")]
    UnresolvedVariable { var: String },

    #[error("invalid frontmatter: {message}")]
    InvalidFrontmatter { message: String },
}
```

### 2. Prompt Loader (src/prompt/loader.rs)

```rust
//! Prompt loading from filesystem.

use super::types::{IncludedContent, Prompt, PromptMetadata, PromptValidationError};
use crate::error::{LoopError, LoopResult};

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tracing::{debug, instrument, warn};

/// Configuration for the prompt loader.
#[derive(Debug, Clone)]
pub struct PromptLoaderConfig {
    /// Base directory for resolving relative paths.
    pub base_dir: PathBuf,
    /// Maximum include depth.
    pub max_include_depth: u32,
    /// File extensions to accept.
    pub extensions: Vec<String>,
    /// Enable strict validation.
    pub strict: bool,
}

impl Default for PromptLoaderConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("."),
            max_include_depth: 5,
            extensions: vec!["md".to_string(), "txt".to_string()],
            strict: false,
        }
    }
}

/// Loads prompts from the filesystem.
pub struct PromptLoader {
    config: PromptLoaderConfig,
}

impl PromptLoader {
    /// Create a new loader.
    pub fn new(config: PromptLoaderConfig) -> Self {
        Self { config }
    }

    /// Load a prompt from a path.
    #[instrument(skip(self), fields(path = %path.as_ref().display()))]
    pub async fn load(&self, path: impl AsRef<Path>) -> LoopResult<Prompt> {
        let path = self.resolve_path(path.as_ref())?;

        debug!("Loading prompt from {:?}", path);

        // Read file
        let raw_content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| LoopError::PromptLoadFailed {
                path: path.clone(),
                source: e,
            })?;

        // Get metadata
        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|e| LoopError::PromptLoadFailed {
                path: path.clone(),
                source: e,
            })?;

        let last_modified = metadata
            .modified()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(|_| chrono::Utc::now());

        // Parse frontmatter and content
        let (frontmatter, content) = self.parse_frontmatter(&raw_content)?;

        // Parse metadata
        let prompt_metadata: PromptMetadata = if let Some(fm) = frontmatter {
            serde_yaml::from_str(&fm).map_err(|e| LoopError::PromptParseFailed {
                path: path.clone(),
                message: format!("Invalid frontmatter: {}", e),
            })?
        } else {
            PromptMetadata::default()
        };

        // Resolve includes
        let includes = self.resolve_includes(&content, &path, 0).await?;

        // Calculate content hash
        let content_hash = self.hash_content(&raw_content);

        let prompt = Prompt {
            source_path: path.clone(),
            metadata: prompt_metadata,
            content,
            includes,
            last_modified,
            content_hash,
        };

        // Validate if strict mode
        if self.config.strict {
            prompt.validate().map_err(|e| LoopError::PromptParseFailed {
                path,
                message: e.to_string(),
            })?;
        }

        Ok(prompt)
    }

    /// Resolve a path relative to base directory.
    fn resolve_path(&self, path: &Path) -> LoopResult<PathBuf> {
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(self.config.base_dir.join(path))
        }
    }

    /// Parse YAML frontmatter from content.
    fn parse_frontmatter(&self, content: &str) -> LoopResult<(Option<String>, String)> {
        let trimmed = content.trim_start();

        // Check for frontmatter delimiter
        if !trimmed.starts_with("---") {
            return Ok((None, content.to_string()));
        }

        // Find end of frontmatter
        let after_start = &trimmed[3..];
        let end_pos = after_start.find("\n---");

        match end_pos {
            Some(pos) => {
                let frontmatter = after_start[..pos].trim().to_string();
                let content = after_start[pos + 4..].trim().to_string();
                Ok((Some(frontmatter), content))
            }
            None => {
                // Malformed frontmatter
                warn!("Frontmatter started but not closed");
                Ok((None, content.to_string()))
            }
        }
    }

    /// Resolve include directives in content.
    async fn resolve_includes(
        &self,
        content: &str,
        parent_path: &Path,
        depth: u32,
    ) -> LoopResult<Vec<IncludedContent>> {
        if depth > self.config.max_include_depth {
            return Err(LoopError::PromptParseFailed {
                path: parent_path.to_path_buf(),
                message: format!("Maximum include depth ({}) exceeded", self.config.max_include_depth),
            });
        }

        let mut includes = Vec::new();
        let include_pattern = regex::Regex::new(r"\{\{include:([^}]+)\}\}").unwrap();

        for cap in include_pattern.captures_iter(content) {
            let directive_path = cap[1].trim();
            let position = cap.get(0).unwrap().start();

            // Resolve relative to parent
            let parent_dir = parent_path.parent().unwrap_or(Path::new("."));
            let resolved_path = parent_dir.join(directive_path);

            // Load included content
            let included_content = tokio::fs::read_to_string(&resolved_path)
                .await
                .map_err(|e| LoopError::PromptLoadFailed {
                    path: resolved_path.clone(),
                    source: e,
                })?;

            // Recursively resolve includes in included content
            let nested_includes = Box::pin(
                self.resolve_includes(&included_content, &resolved_path, depth + 1)
            ).await?;

            includes.push(IncludedContent {
                directive_path: directive_path.to_string(),
                resolved_path: resolved_path.clone(),
                content: included_content,
                position,
            });

            // Add nested includes
            includes.extend(nested_includes);
        }

        Ok(includes)
    }

    /// Calculate SHA-256 hash of content.
    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if a file has valid extension.
    pub fn is_valid_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.config.extensions.iter().any(|e| e == ext))
            .unwrap_or(false)
    }
}

/// Watches for prompt file changes.
pub struct PromptWatcher {
    /// Paths being watched.
    watched_paths: Vec<PathBuf>,
    /// Notification sender.
    notify_tx: tokio::sync::mpsc::Sender<PromptChange>,
}

/// A change to a watched prompt.
#[derive(Debug, Clone)]
pub struct PromptChange {
    /// Path that changed.
    pub path: PathBuf,
    /// Type of change.
    pub change_type: PromptChangeType,
}

/// Type of prompt change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptChangeType {
    /// Content was modified.
    Modified,
    /// File was deleted.
    Deleted,
    /// File was created.
    Created,
}

impl PromptWatcher {
    /// Create a new watcher.
    pub fn new() -> (Self, tokio::sync::mpsc::Receiver<PromptChange>) {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        (
            Self {
                watched_paths: Vec::new(),
                notify_tx: tx,
            },
            rx,
        )
    }

    /// Add a path to watch.
    pub async fn watch(&mut self, path: impl AsRef<Path>) -> LoopResult<()> {
        let path = path.as_ref().to_path_buf();

        if !self.watched_paths.contains(&path) {
            self.watched_paths.push(path);
        }

        Ok(())
    }

    /// Remove a path from watching.
    pub fn unwatch(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.watched_paths.retain(|p| p != path);
    }

    /// Start the watch loop.
    pub async fn run(&self) -> LoopResult<()> {
        use notify::{RecommendedWatcher, RecursiveMode, Watcher};

        let tx = self.notify_tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let change_type = match event.kind {
                        notify::EventKind::Modify(_) => Some(PromptChangeType::Modified),
                        notify::EventKind::Remove(_) => Some(PromptChangeType::Deleted),
                        notify::EventKind::Create(_) => Some(PromptChangeType::Created),
                        _ => None,
                    };

                    if let Some(ct) = change_type {
                        for path in event.paths {
                            let _ = tx.blocking_send(PromptChange {
                                path,
                                change_type: ct,
                            });
                        }
                    }
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| LoopError::WatcherFailed {
            message: e.to_string(),
        })?;

        // Add all watched paths
        for path in &self.watched_paths {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| LoopError::WatcherFailed {
                    message: e.to_string(),
                })?;
        }

        // Keep watcher alive
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
```

### 3. Prompt Cache (src/prompt/cache.rs)

```rust
//! Prompt caching.

use super::types::Prompt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for prompt cache.
#[derive(Debug, Clone)]
pub struct PromptCacheConfig {
    /// Maximum entries in cache.
    pub max_entries: usize,
    /// Enable automatic invalidation.
    pub auto_invalidate: bool,
}

impl Default for PromptCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 50,
            auto_invalidate: true,
        }
    }
}

/// Caches loaded prompts.
pub struct PromptCache {
    config: PromptCacheConfig,
    cache: RwLock<HashMap<PathBuf, CacheEntry>>,
}

/// A cached prompt entry.
#[derive(Debug, Clone)]
struct CacheEntry {
    prompt: Arc<Prompt>,
    content_hash: String,
    cached_at: chrono::DateTime<chrono::Utc>,
}

impl PromptCache {
    /// Create a new cache.
    pub fn new(config: PromptCacheConfig) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get a cached prompt if valid.
    pub async fn get(&self, path: &PathBuf, current_hash: &str) -> Option<Arc<Prompt>> {
        let cache = self.cache.read().await;

        cache.get(path).and_then(|entry| {
            if self.config.auto_invalidate && entry.content_hash != current_hash {
                None
            } else {
                Some(entry.prompt.clone())
            }
        })
    }

    /// Store a prompt in cache.
    pub async fn put(&self, prompt: Prompt) {
        let mut cache = self.cache.write().await;

        // Evict if at capacity
        if cache.len() >= self.config.max_entries {
            // Remove oldest entry
            if let Some(oldest) = cache
                .iter()
                .min_by_key(|(_, e)| e.cached_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest);
            }
        }

        let path = prompt.source_path.clone();
        let hash = prompt.content_hash.clone();

        cache.insert(
            path,
            CacheEntry {
                prompt: Arc::new(prompt),
                content_hash: hash,
                cached_at: chrono::Utc::now(),
            },
        );
    }

    /// Invalidate a specific path.
    pub async fn invalidate(&self, path: &PathBuf) {
        let mut cache = self.cache.write().await;
        cache.remove(path);
    }

    /// Clear the entire cache.
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            entries: cache.len(),
            max_entries: self.config.max_entries,
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub max_entries: usize,
}
```

### 4. Module Root (src/prompt/mod.rs)

```rust
//! Prompt loading and management.

pub mod cache;
pub mod loader;
pub mod types;

pub use cache::{CacheStats, PromptCache, PromptCacheConfig};
pub use loader::{PromptChange, PromptChangeType, PromptLoader, PromptLoaderConfig, PromptWatcher};
pub use types::{IncludedContent, IterationSettings, Prompt, PromptMetadata, PromptValidationError};
```

---

## Testing Requirements

1. Load simple prompt without frontmatter
2. Parse YAML frontmatter correctly
3. Resolve single-level includes
4. Handle nested includes with depth limit
5. Cache invalidates on content change
6. File watcher detects modifications
7. Invalid frontmatter produces helpful error
8. Missing include file produces helpful error

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Depends on: [029-file-system-utilities.md](../phase-01-common/029-file-system-utilities.md)
- Next: [099-prompt-templates.md](099-prompt-templates.md)
- Related: [097-loop-iteration.md](097-loop-iteration.md)
