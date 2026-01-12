# 031 - Primitives Crate Structure

**Phase:** 2 - Five Primitives
**Spec ID:** 031
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types, 029-file-system-utilities
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create the `tachikoma-primitives` crate that provides the five core primitives: `read_file`, `list_files`, `bash`, `edit_file`, and `code_search`. This crate defines the common structure, traits, and re-exports for all primitives.

---

## Acceptance Criteria

- [ ] `tachikoma-primitives` crate created with proper structure
- [ ] Common primitive trait defined
- [ ] All five primitives re-exported from single entry point
- [ ] Primitive result types defined
- [ ] Context and configuration types established
- [ ] Feature flags for optional primitives

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-primitives/Cargo.toml)

```toml
[package]
name = "tachikoma-primitives"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core primitives for Tachikoma agent operations"

[features]
default = ["all"]
all = ["read-file", "list-files", "bash", "edit-file", "code-search"]
read-file = []
list-files = ["dep:walkdir"]
bash = ["dep:tokio"]
edit-file = []
code-search = ["dep:serde_json"]

[dependencies]
tachikoma-common-core.workspace = true
tachikoma-common-error.workspace = true

async-trait = "0.1"
serde = { workspace = true, features = ["derive"] }
thiserror.workspace = true
tracing.workspace = true

# Optional dependencies
walkdir = { version = "2.4", optional = true }
tokio = { workspace = true, features = ["process", "time"], optional = true }
serde_json = { workspace = true, optional = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
tempfile = "3.9"
```

### 2. Primitive Context (src/context.rs)

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

### 3. Primitive Result Types (src/result.rs)

```rust
//! Result types for primitive operations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Metadata about primitive execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Time taken to execute.
    pub duration: Duration,
    /// Operation ID.
    pub operation_id: String,
    /// Primitive name.
    pub primitive: String,
}

/// Result of a read_file operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileResult {
    /// File content.
    pub content: String,
    /// File path.
    pub path: PathBuf,
    /// File size in bytes.
    pub size: usize,
    /// Whether content was truncated.
    pub truncated: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// Result of a list_files operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesResult {
    /// List of file entries.
    pub entries: Vec<FileEntry>,
    /// Base directory.
    pub base_path: PathBuf,
    /// Total files found.
    pub total_count: usize,
    /// Whether results were truncated.
    pub truncated: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// A file entry in list results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// File path.
    pub path: PathBuf,
    /// Whether it's a directory.
    pub is_dir: bool,
    /// File size (None for directories).
    pub size: Option<u64>,
    /// File extension.
    pub extension: Option<String>,
}

/// Result of a bash operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashResult {
    /// Exit code.
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Whether the command timed out.
    pub timed_out: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// Result of an edit_file operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFileResult {
    /// Whether the edit was successful.
    pub success: bool,
    /// Number of replacements made.
    pub replacements: usize,
    /// File path.
    pub path: PathBuf,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// Result of a code_search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchResult {
    /// Search matches.
    pub matches: Vec<SearchMatch>,
    /// Pattern used.
    pub pattern: String,
    /// Total matches found.
    pub total_count: usize,
    /// Whether results were truncated.
    pub truncated: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// A single search match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    /// File path.
    pub path: PathBuf,
    /// Line number (1-indexed).
    pub line_number: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Matched line content.
    pub line_content: String,
    /// Context lines before.
    pub context_before: Vec<String>,
    /// Context lines after.
    pub context_after: Vec<String>,
}
```

### 4. Library Root (src/lib.rs)

```rust
//! Tachikoma primitives - core agent operations.
//!
//! This crate provides the five primitives that form the foundation
//! of Tachikoma's agent capabilities:
//!
//! - `read_file` - Read file contents
//! - `list_files` - List directory contents
//! - `bash` - Execute shell commands
//! - `edit_file` - Search and replace in files
//! - `code_search` - Search code with ripgrep
//!
//! # Example
//!
//! ```no_run
//! use tachikoma_primitives::{PrimitiveContext, read_file};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ctx = PrimitiveContext::new(PathBuf::from("."));
//! let result = read_file(&ctx, "src/main.rs", None).await?;
//! println!("Content: {}", result.content);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

pub mod context;
pub mod error;
pub mod result;
pub mod traits;

#[cfg(feature = "read-file")]
pub mod read_file;

#[cfg(feature = "list-files")]
pub mod list_files;

#[cfg(feature = "bash")]
pub mod bash;

#[cfg(feature = "edit-file")]
pub mod edit_file;

#[cfg(feature = "code-search")]
pub mod code_search;

// Re-exports
pub use context::{PrimitiveConfig, PrimitiveContext};
pub use error::{PrimitiveError, PrimitiveResult};
pub use result::*;
pub use traits::Primitive;

#[cfg(feature = "read-file")]
pub use read_file::read_file;

#[cfg(feature = "list-files")]
pub use list_files::list_files;

#[cfg(feature = "bash")]
pub use bash::bash;

#[cfg(feature = "edit-file")]
pub use edit_file::edit_file;

#[cfg(feature = "code-search")]
pub use code_search::code_search;
```

### 5. Error Types (src/error.rs)

```rust
//! Error types for primitives.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during primitive execution.
#[derive(Debug, Error)]
pub enum PrimitiveError {
    /// File not found.
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Permission denied.
    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    /// Path not allowed by configuration.
    #[error("path not allowed: {path}")]
    PathNotAllowed { path: PathBuf },

    /// File too large.
    #[error("file too large: {size} bytes (max: {max})")]
    FileTooLarge { size: usize, max: usize },

    /// Operation timed out.
    #[error("operation timed out after {duration:?}")]
    Timeout { duration: std::time::Duration },

    /// Command execution failed.
    #[error("command failed with exit code {exit_code}: {message}")]
    CommandFailed { exit_code: i32, message: String },

    /// Search pattern invalid.
    #[error("invalid search pattern: {pattern}")]
    InvalidPattern { pattern: String },

    /// Edit target not unique.
    #[error("edit target not unique: found {count} matches")]
    NotUnique { count: usize },

    /// Edit target not found.
    #[error("edit target not found in file")]
    TargetNotFound,

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Validation error.
    #[error("validation error: {message}")]
    Validation { message: String },
}

/// Result type alias for primitives.
pub type PrimitiveResult<T> = std::result::Result<T, PrimitiveError>;
```

---

## Testing Requirements

1. Context resolves relative paths correctly
2. Path allowlist/denylist works correctly
3. Default configuration is sensible
4. All primitives can be imported and used
5. Feature flags enable/disable correct modules

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Depends on: [012-error-types.md](../phase-01-common/012-error-types.md)
- Next: [032-read-file-impl.md](032-read-file-impl.md)
- Used by: [046-primitives-trait.md](046-primitives-trait.md)
