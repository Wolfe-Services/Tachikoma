# 031b - Primitives Context Types

**Phase:** 2 - Five Primitives
**Spec ID:** 031b
**Status:** Planned
**Dependencies:** 031a-primitives-crate-setup
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define the execution context and configuration types for primitive operations, including path validation, timeouts, and security boundaries.

---

## Acceptance Criteria

- [ ] `PrimitiveConfig` struct with sensible defaults
- [ ] `PrimitiveContext` struct for execution context
- [ ] Path allowlist/denylist validation
- [ ] Working directory resolution

---

## Implementation Details

### 1. Context Module (src/context.rs)

```rust
//! Execution context for primitives.

use std::path::PathBuf;
use std::time::Duration;

/// Configuration for primitive execution.
#[derive(Debug, Clone)]
pub struct PrimitiveConfig {
    /// Maximum file size to read (in bytes).
    pub max_file_size: usize,
    /// Maximum directory depth for recursive operations.
    pub max_depth: usize,
    /// Default timeout for operations.
    pub default_timeout: Duration,
    /// Whether to follow symlinks.
    pub follow_symlinks: bool,
    /// Allowed paths (if empty, all paths allowed).
    pub allowed_paths: Vec<PathBuf>,
    /// Denied paths.
    pub denied_paths: Vec<PathBuf>,
}

impl Default for PrimitiveConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_depth: 20,
            default_timeout: Duration::from_secs(30),
            follow_symlinks: false,
            allowed_paths: Vec::new(),
            denied_paths: vec![
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/etc/shadow"),
            ],
        }
    }
}

/// Execution context passed to primitives.
#[derive(Debug, Clone)]
pub struct PrimitiveContext {
    /// Working directory for relative paths.
    pub working_dir: PathBuf,
    /// Configuration.
    pub config: PrimitiveConfig,
    /// Unique operation ID for logging.
    pub operation_id: String,
}

impl PrimitiveContext {
    /// Create a new context with defaults.
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            config: PrimitiveConfig::default(),
            operation_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create with custom config.
    pub fn with_config(working_dir: PathBuf, config: PrimitiveConfig) -> Self {
        Self {
            working_dir,
            config,
            operation_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Resolve a path relative to working directory.
    pub fn resolve_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else {
            self.working_dir.join(path)
        }
    }

    /// Check if a path is allowed.
    pub fn is_path_allowed(&self, path: &PathBuf) -> bool {
        // Check denied paths first
        for denied in &self.config.denied_paths {
            if path.starts_with(denied) {
                return false;
            }
        }

        // If allowed_paths is empty, all non-denied paths are allowed
        if self.config.allowed_paths.is_empty() {
            return true;
        }

        // Check if path is under an allowed path
        for allowed in &self.config.allowed_paths {
            if path.starts_with(allowed) {
                return true;
            }
        }

        false
    }
}
```

---

## Testing Requirements

1. Context resolves relative paths correctly
2. Path allowlist/denylist works correctly
3. Default configuration is sensible
4. Absolute paths are preserved

---

## Related Specs

- Depends on: [031a-primitives-crate-setup.md](031a-primitives-crate-setup.md)
- Next: [031c-primitives-results.md](031c-primitives-results.md)
