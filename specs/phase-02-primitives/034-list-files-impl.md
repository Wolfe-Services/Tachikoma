# 034 - List Files Implementation

**Phase:** 2 - Five Primitives
**Spec ID:** 034
**Status:** Planned
**Dependencies:** 031-primitives-crate
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement the `list_files` primitive that lists directory contents with filtering, sorting, and metadata options.

---

## Acceptance Criteria

- [ ] List files in a directory
- [ ] Filter by extension/pattern
- [ ] Include/exclude directories
- [ ] Return file metadata (size, modified time)
- [ ] Support pagination for large directories
- [ ] Handle permission errors gracefully

---

## Implementation Details

### 1. List Files Module (src/list_files/mod.rs)

```rust
//! List files primitive implementation.

mod options;

pub use options::ListFilesOptions;

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{ExecutionMetadata, FileEntry, ListFilesResult},
};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, instrument};

/// List files in a directory.
///
/// # Arguments
///
/// * `ctx` - Execution context
/// * `path` - Directory path (relative or absolute)
/// * `options` - Optional configuration for listing
///
/// # Returns
///
/// Result containing list of file entries and metadata.
///
/// # Example
///
/// ```no_run
/// use tachikoma_primitives::{PrimitiveContext, list_files, ListFilesOptions};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ctx = PrimitiveContext::new(PathBuf::from("."));
///
/// // List all files
/// let result = list_files(&ctx, "src", None).await?;
///
/// // List only Rust files
/// let opts = ListFilesOptions::new().extension("rs");
/// let result = list_files(&ctx, "src", Some(opts)).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(ctx), fields(path = %path, op_id = %ctx.operation_id))]
pub async fn list_files(
    ctx: &PrimitiveContext,
    path: &str,
    options: Option<ListFilesOptions>,
) -> PrimitiveResult<ListFilesResult> {
    let start = Instant::now();
    let options = options.unwrap_or_default();

    // Resolve and validate path
    let resolved_path = ctx.resolve_path(path);
    debug!("Listing directory: {:?}", resolved_path);

    if !ctx.is_path_allowed(&resolved_path) {
        return Err(PrimitiveError::PathNotAllowed {
            path: resolved_path,
        });
    }

    // Check directory exists
    if !resolved_path.exists() {
        return Err(PrimitiveError::FileNotFound {
            path: resolved_path,
        });
    }

    if !resolved_path.is_dir() {
        return Err(PrimitiveError::Validation {
            message: format!("Path is not a directory: {:?}", resolved_path),
        });
    }

    // Read directory entries
    let entries = read_directory(&resolved_path, &options, ctx)?;

    // Apply sorting
    let mut entries = entries;
    sort_entries(&mut entries, &options);

    // Apply pagination
    let total_count = entries.len();
    let truncated = if let Some(limit) = options.limit {
        let offset = options.offset.unwrap_or(0);
        if offset < entries.len() {
            entries = entries.into_iter().skip(offset).take(limit).collect();
            offset + limit < total_count
        } else {
            entries.clear();
            false
        }
    } else {
        false
    };

    let duration = start.elapsed();
    debug!("Listed {} entries in {:?}", entries.len(), duration);

    Ok(ListFilesResult {
        entries,
        base_path: resolved_path,
        total_count,
        truncated,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "list_files".to_string(),
        },
    })
}

/// Read directory entries with filtering.
fn read_directory(
    path: &PathBuf,
    options: &ListFilesOptions,
    ctx: &PrimitiveContext,
) -> PrimitiveResult<Vec<FileEntry>> {
    let read_dir = fs::read_dir(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            PrimitiveError::PermissionDenied { path: path.clone() }
        } else {
            PrimitiveError::Io(e)
        }
    })?;

    let mut entries = Vec::new();

    for entry_result in read_dir {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                debug!("Error reading entry: {}", e);
                continue;
            }
        };

        let entry_path = entry.path();

        // Check path allowed
        if !ctx.is_path_allowed(&entry_path) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();

        // Apply filters
        if !options.include_dirs && is_dir {
            continue;
        }

        if options.dirs_only && !is_dir {
            continue;
        }

        if !options.include_hidden {
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
        }

        // Extension filter
        if let Some(ref ext) = options.extension {
            if !is_dir {
                let file_ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase());
                if file_ext.as_deref() != Some(ext.to_lowercase().as_str()) {
                    continue;
                }
            }
        }

        // Pattern filter
        if let Some(ref pattern) = options.pattern {
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if !matches_glob(name, pattern) {
                    continue;
                }
            }
        }

        let size = if is_dir { None } else { Some(metadata.len()) };

        let extension = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        entries.push(FileEntry {
            path: entry_path,
            is_dir,
            size,
            extension,
        });
    }

    Ok(entries)
}

/// Sort entries based on options.
fn sort_entries(entries: &mut Vec<FileEntry>, options: &ListFilesOptions) {
    match options.sort_by {
        SortBy::Name => {
            entries.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
        }
        SortBy::Size => {
            entries.sort_by(|a, b| a.size.cmp(&b.size));
        }
        SortBy::Extension => {
            entries.sort_by(|a, b| a.extension.cmp(&b.extension));
        }
        SortBy::Type => {
            // Directories first, then files
            entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir));
        }
    }

    if options.reverse {
        entries.reverse();
    }
}

/// Simple glob pattern matching.
fn matches_glob(name: &str, pattern: &str) -> bool {
    // Simple implementation supporting * wildcard
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with('*') && pattern.ends_with('*') {
        let inner = &pattern[1..pattern.len() - 1];
        return name.contains(inner);
    }

    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        return name.ends_with(suffix);
    }

    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return name.starts_with(prefix);
    }

    name == pattern
}

/// Sorting options.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortBy {
    #[default]
    Name,
    Size,
    Extension,
    Type,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::{create_dir, write};

    #[tokio::test]
    async fn test_list_files_basic() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.txt"), "a").unwrap();
        write(dir.path().join("b.rs"), "b").unwrap();
        create_dir(dir.path().join("subdir")).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = list_files(&ctx, ".", None).await.unwrap();

        assert_eq!(result.total_count, 2); // exclude dirs by default
    }

    #[tokio::test]
    async fn test_list_files_with_extension() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.txt"), "a").unwrap();
        write(dir.path().join("b.rs"), "b").unwrap();
        write(dir.path().join("c.rs"), "c").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = ListFilesOptions::new().extension("rs");
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();

        assert_eq!(result.entries.len(), 2);
        assert!(result.entries.iter().all(|e| e.extension.as_deref() == Some("rs")));
    }

    #[tokio::test]
    async fn test_list_files_include_dirs() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.txt"), "a").unwrap();
        create_dir(dir.path().join("subdir")).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = ListFilesOptions::new().include_directories();
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();

        assert_eq!(result.total_count, 2);
        assert!(result.entries.iter().any(|e| e.is_dir));
    }
}
```

### 2. List Files Options (src/list_files/options.rs)

```rust
//! Options for list_files primitive.

use super::SortBy;

/// Options for listing files.
#[derive(Debug, Clone, Default)]
pub struct ListFilesOptions {
    /// Filter by file extension.
    pub extension: Option<String>,
    /// Filter by glob pattern.
    pub pattern: Option<String>,
    /// Include directories in output.
    pub include_dirs: bool,
    /// Only list directories.
    pub dirs_only: bool,
    /// Include hidden files (starting with .).
    pub include_hidden: bool,
    /// Maximum number of results.
    pub limit: Option<usize>,
    /// Offset for pagination.
    pub offset: Option<usize>,
    /// Sort order.
    pub sort_by: SortBy,
    /// Reverse sort order.
    pub reverse: bool,
}

impl ListFilesOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by extension.
    pub fn extension(mut self, ext: &str) -> Self {
        self.extension = Some(ext.trim_start_matches('.').to_string());
        self
    }

    /// Filter by glob pattern.
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }

    /// Include directories in results.
    pub fn include_directories(mut self) -> Self {
        self.include_dirs = true;
        self
    }

    /// Only list directories.
    pub fn directories_only(mut self) -> Self {
        self.dirs_only = true;
        self.include_dirs = true;
        self
    }

    /// Include hidden files.
    pub fn include_hidden(mut self) -> Self {
        self.include_hidden = true;
        self
    }

    /// Limit number of results.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set pagination offset.
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set sort order.
    pub fn sort(mut self, sort_by: SortBy) -> Self {
        self.sort_by = sort_by;
        self
    }

    /// Reverse sort order.
    pub fn reversed(mut self) -> Self {
        self.reverse = true;
        self
    }
}
```

---

## Testing Requirements

1. Basic directory listing works
2. Extension filtering is case-insensitive
3. Pattern matching supports wildcards
4. Hidden files are excluded by default
5. Pagination works correctly
6. Sorting by different fields works
7. Empty directories return empty list
8. Permission errors are handled gracefully

---

## Related Specs

- Depends on: [031-primitives-crate.md](031-primitives-crate.md)
- Next: [035-list-files-recursive.md](035-list-files-recursive.md)
- Used by: Agent loop for directory exploration
