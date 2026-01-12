# 031d - Primitives Error Types

**Phase:** 2 - Five Primitives
**Spec ID:** 031d
**Status:** Planned
**Dependencies:** 031c-primitives-results
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define the error types for primitive operations with clear error messages and proper error chaining.

---

## Acceptance Criteria

- [ ] `PrimitiveError` enum with all error variants
- [ ] `PrimitiveResult` type alias
- [ ] Clear error messages for each variant

---

## Implementation Details

### 1. Error Types (src/error.rs)

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

1. Error messages are clear and helpful
2. IO errors convert properly
3. Error variants cover all failure cases

---

## Related Specs

- Depends on: [031c-primitives-results.md](031c-primitives-results.md)
- Next: [032-read-file-impl.md](032-read-file-impl.md)
