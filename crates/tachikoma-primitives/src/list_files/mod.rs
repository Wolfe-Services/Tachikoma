//! List files primitive implementation.

mod options;
mod recursive;

pub use options::{ListFilesOptions, SortBy};
pub use recursive::{list_files_recursive, list_files_recursive_with_callback, RecursiveIterator, RecursiveOptions};

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

    // Check if recursive mode is enabled
    if options.recursive {
        // Convert to recursive options
        let mut recursive_opts = recursive::RecursiveOptions::new()
            .include_hidden()  // Respect include_hidden setting
            .include_dirs();   // Respect include_dirs setting
            
        if !options.include_hidden {
            recursive_opts.include_hidden = false;
        }
        
        if !options.include_dirs {
            recursive_opts.include_dirs = false;
        }
        
        if let Some(ext) = &options.extension {
            recursive_opts = recursive_opts.extension(ext);
        }
        
        if let Some(limit) = options.limit {
            recursive_opts = recursive_opts.max_results(limit);
        }
        
        return recursive::list_files_recursive(ctx, path, recursive_opts).await;
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
            let remaining_after_offset = entries.len() - offset;
            entries = entries.into_iter().skip(offset).take(limit).collect();
            limit < remaining_after_offset
        } else {
            entries.clear();
            false
        }
    } else if let Some(offset) = options.offset {
        if offset < entries.len() {
            entries = entries.into_iter().skip(offset).collect();
        } else {
            entries.clear();
        }
        false
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

        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());

        entries.push(FileEntry {
            path: entry_path,
            is_dir,
            size,
            extension,
            modified,
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
        assert_eq!(result.entries.len(), 2);
        assert!(result.entries.iter().all(|e| !e.is_dir));
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
        assert!(result.entries.iter().any(|e| !e.is_dir));
    }

    #[tokio::test]
    async fn test_list_files_dirs_only() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.txt"), "a").unwrap();
        create_dir(dir.path().join("subdir1")).unwrap();
        create_dir(dir.path().join("subdir2")).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = ListFilesOptions::new().directories_only();
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();

        assert_eq!(result.entries.len(), 2);
        assert!(result.entries.iter().all(|e| e.is_dir));
    }

    #[tokio::test]
    async fn test_list_files_hidden() {
        let dir = tempdir().unwrap();
        write(dir.path().join("visible.txt"), "a").unwrap();
        write(dir.path().join(".hidden.txt"), "b").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Default - exclude hidden
        let result = list_files(&ctx, ".", None).await.unwrap();
        assert_eq!(result.entries.len(), 1);
        assert!(!result.entries[0].path.file_name().unwrap().to_str().unwrap().starts_with('.'));

        // Include hidden
        let opts = ListFilesOptions::new().include_hidden();
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        assert_eq!(result.entries.len(), 2);
    }

    #[tokio::test]
    async fn test_list_files_pattern() {
        let dir = tempdir().unwrap();
        write(dir.path().join("test_file.txt"), "a").unwrap();
        write(dir.path().join("other_file.txt"), "b").unwrap();
        write(dir.path().join("test_data.rs"), "c").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = ListFilesOptions::new().pattern("test*");
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();

        assert_eq!(result.entries.len(), 2);
        assert!(result.entries.iter().all(|e| 
            e.path.file_name().unwrap().to_str().unwrap().starts_with("test")
        ));
    }

    #[tokio::test]
    async fn test_list_files_pagination() {
        let dir = tempdir().unwrap();
        for i in 0..10 {
            write(dir.path().join(format!("file{:02}.txt", i)), "content").unwrap();
        }

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Limit without offset
        let opts = ListFilesOptions::new().limit(5);
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        assert_eq!(result.entries.len(), 5);
        assert_eq!(result.total_count, 10);
        assert!(result.truncated);

        // With offset
        let opts = ListFilesOptions::new().limit(3).offset(2);
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        assert_eq!(result.entries.len(), 3);
        assert_eq!(result.total_count, 10);
        assert!(result.truncated);

        // Beyond available entries
        let opts = ListFilesOptions::new().offset(15);
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        assert_eq!(result.entries.len(), 0);
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn test_list_files_sorting() {
        let dir = tempdir().unwrap();
        write(dir.path().join("c.txt"), "large").unwrap();
        write(dir.path().join("a.txt"), "small").unwrap();
        write(dir.path().join("b.txt"), "medium").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Sort by name
        let opts = ListFilesOptions::new().sort(SortBy::Name);
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        let names: Vec<_> = result.entries.iter()
            .map(|e| e.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);

        // Sort by size
        let opts = ListFilesOptions::new().sort(SortBy::Size);
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        let sizes: Vec<_> = result.entries.iter().map(|e| e.size.unwrap()).collect();
        assert!(sizes.is_sorted());

        // Reverse sort
        let opts = ListFilesOptions::new().sort(SortBy::Name).reversed();
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        let names: Vec<_> = result.entries.iter()
            .map(|e| e.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["c.txt", "b.txt", "a.txt"]);
    }

    #[tokio::test]
    async fn test_list_files_not_found() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = list_files(&ctx, "nonexistent", None).await;

        assert!(matches!(result, Err(PrimitiveError::FileNotFound { .. })));
    }

    #[tokio::test]
    async fn test_list_files_not_directory() {
        let dir = tempdir().unwrap();
        write(dir.path().join("file.txt"), "content").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = list_files(&ctx, "file.txt", None).await;

        assert!(matches!(result, Err(PrimitiveError::Validation { .. })));
    }

    #[tokio::test]
    async fn test_glob_patterns() {
        assert!(matches_glob("test.txt", "*"));
        assert!(matches_glob("test.txt", "test*"));
        assert!(matches_glob("test.txt", "*.txt"));
        assert!(matches_glob("test.txt", "*test*"));
        assert!(!matches_glob("test.txt", "*.rs"));
        assert!(!matches_glob("test.txt", "other*"));
    }

    #[tokio::test]
    async fn test_list_files_metadata() {
        let dir = tempdir().unwrap();
        write(dir.path().join("test.txt"), "content").unwrap();
        create_dir(dir.path().join("subdir")).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = ListFilesOptions::new().include_directories();
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();

        assert_eq!(result.entries.len(), 2);
        
        for entry in result.entries {
            // All entries should have modified time
            assert!(entry.modified.is_some());
            
            if entry.is_dir {
                // Directory should not have size
                assert!(entry.size.is_none());
            } else {
                // File should have size
                assert!(entry.size.is_some());
                assert_eq!(entry.size.unwrap(), 7); // "content" = 7 bytes
            }
        }
    }

    #[tokio::test]
    async fn test_list_files_permission_error() {
        // This test may not work on all systems, so we'll skip it if it fails
        let ctx = PrimitiveContext::new(PathBuf::from("/"));
        let result = list_files(&ctx, "/root", None).await;

        // Should either get permission denied or file not found (if /root doesn't exist)
        // or path not allowed (if security prevents access)
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::PermissionDenied { .. }) => {
                // Expected case
            }
            Err(PrimitiveError::FileNotFound { .. }) => {
                // Also acceptable if /root doesn't exist
            }
            Err(PrimitiveError::PathNotAllowed { .. }) => {
                // Also acceptable if security prevents access
            }
            _ => panic!("Expected permission, not found, or path not allowed error"),
        }
    }

    #[tokio::test]
    async fn test_list_files_recursive_option() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.txt"), "a").unwrap();
        create_dir(dir.path().join("subdir")).unwrap();
        write(dir.path().join("subdir/b.txt"), "b").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Non-recursive - should only see a.txt
        let opts = ListFilesOptions::new();
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        assert_eq!(result.entries.len(), 1);
        assert!(result.entries[0].path.file_name().unwrap().to_str().unwrap() == "a.txt");
        
        // Recursive - should see both a.txt and subdir/b.txt
        let opts = ListFilesOptions::new().recursive();
        let result = list_files(&ctx, ".", Some(opts)).await.unwrap();
        assert_eq!(result.entries.len(), 2);
        
        let names: Vec<_> = result.entries.iter()
            .filter_map(|e| e.path.file_name())
            .filter_map(|n| n.to_str())
            .collect();
        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"b.txt"));
    }
}