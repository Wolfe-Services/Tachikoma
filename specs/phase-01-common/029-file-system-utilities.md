# 029 - File System Utilities

**Phase:** 1 - Core Common Crates
**Spec ID:** 029
**Status:** Planned
**Dependencies:** 012-error-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Provide file system utilities for safe file operations including atomic writes, directory creation, and file watching.

---

## Acceptance Criteria

- [x] Atomic file writes
- [x] Safe directory creation
- [x] File read with encoding detection
- [x] Temporary file utilities
- [x] File permissions handling

---

## Implementation Details

### 1. File System Module (crates/tachikoma-common-fs/src/lib.rs)

```rust
//! File system utilities for Tachikoma.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// File system errors.
#[derive(Debug, Error)]
pub enum FsError {
    #[error("file not found: {path}")]
    NotFound { path: PathBuf },

    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("path is not valid UTF-8: {path:?}")]
    InvalidPath { path: PathBuf },
}

/// Read a file to string with size limit.
pub fn read_to_string(path: impl AsRef<Path>, max_size: usize) -> Result<String, FsError> {
    let path = path.as_ref();

    let metadata = fs::metadata(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => FsError::NotFound {
            path: path.to_path_buf(),
        },
        io::ErrorKind::PermissionDenied => FsError::PermissionDenied {
            path: path.to_path_buf(),
        },
        _ => FsError::Io(e),
    })?;

    if metadata.len() as usize > max_size {
        return Err(FsError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file too large: {} bytes", metadata.len()),
        )));
    }

    fs::read_to_string(path).map_err(FsError::Io)
}

/// Read a file to bytes.
pub fn read_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, FsError> {
    let path = path.as_ref();
    fs::read(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => FsError::NotFound {
            path: path.to_path_buf(),
        },
        _ => FsError::Io(e),
    })
}

/// Write to a file atomically (write to temp, then rename).
pub fn write_atomic(path: impl AsRef<Path>, contents: &[u8]) -> Result<(), FsError> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or(Path::new("."));

    // Create parent directory if needed
    fs::create_dir_all(parent)?;

    // Write to temporary file
    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path)?;
    file.write_all(contents)?;
    file.sync_all()?;

    // Atomic rename
    fs::rename(&temp_path, path)?;

    Ok(())
}

/// Write string to file atomically.
pub fn write_string_atomic(path: impl AsRef<Path>, contents: &str) -> Result<(), FsError> {
    write_atomic(path, contents.as_bytes())
}

/// Ensure a directory exists.
pub fn ensure_dir(path: impl AsRef<Path>) -> Result<(), FsError> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Check if a path exists and is a file.
pub fn is_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

/// Check if a path exists and is a directory.
pub fn is_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

/// Get the file extension as a string.
pub fn extension(path: impl AsRef<Path>) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

/// Copy a file with optional overwrite.
pub fn copy_file(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    overwrite: bool,
) -> Result<u64, FsError> {
    let dst = dst.as_ref();
    if !overwrite && dst.exists() {
        return Err(FsError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "destination already exists",
        )));
    }

    if let Some(parent) = dst.parent() {
        ensure_dir(parent)?;
    }

    fs::copy(src, dst).map_err(FsError::Io)
}

/// Delete a file if it exists.
pub fn remove_file_if_exists(path: impl AsRef<Path>) -> Result<bool, FsError> {
    let path = path.as_ref();
    if path.exists() {
        fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// List files in a directory.
pub fn list_files(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, FsError> {
    let dir = dir.as_ref();
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

/// List directories in a directory.
pub fn list_dirs(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, FsError> {
    let dir = dir.as_ref();
    let mut dirs = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }

    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        write_string_atomic(&path, "hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");

        write_string_atomic(&path, "world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "world");
    }

    #[test]
    fn test_ensure_dir() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a/b/c");

        ensure_dir(&path).unwrap();
        assert!(path.is_dir());
    }

    #[test]
    fn test_file_not_found() {
        let result = read_to_string("/nonexistent/path", 1024);
        assert!(matches!(result, Err(FsError::NotFound { .. })));
    }
}
```

### 2. Crate Setup

```toml
[package]
name = "tachikoma-common-fs"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror.workspace = true

[dev-dependencies]
tempfile = "3.9"
```

---

## Testing Requirements

1. Atomic writes don't corrupt files on failure
2. Directory creation is recursive
3. File not found errors are clear
4. Copy respects overwrite flag

---

## Related Specs

- Depends on: [012-error-types.md](012-error-types.md)
- Next: [030-path-handling.md](030-path-handling.md)
