# 035 - List Files Recursive Walking

**Phase:** 2 - Five Primitives
**Spec ID:** 035
**Status:** Planned
**Dependencies:** 034-list-files-impl
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Extend `list_files` with recursive directory walking using the `walkdir` crate, with depth limits and gitignore support.

---

## Acceptance Criteria

- [ ] Recursive directory traversal
- [ ] Maximum depth limit enforcement
- [ ] Gitignore pattern support
- [ ] Symlink handling (follow or skip)
- [ ] Progress callbacks for large directories
- [ ] Memory-efficient streaming iteration

---

## Implementation Details

### 1. Recursive Walker (src/list_files/recursive.rs)

```rust
//! Recursive directory walking implementation.

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{ExecutionMetadata, FileEntry, ListFilesResult},
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, instrument, warn};
use walkdir::{DirEntry, WalkDir};

/// Options for recursive listing.
#[derive(Debug, Clone)]
pub struct RecursiveOptions {
    /// Maximum depth (1 = immediate children only).
    pub max_depth: usize,
    /// Follow symbolic links.
    pub follow_symlinks: bool,
    /// Respect gitignore files.
    pub use_gitignore: bool,
    /// Additional ignore patterns.
    pub ignore_patterns: Vec<String>,
    /// File extension filter.
    pub extension: Option<String>,
    /// Maximum number of results.
    pub max_results: Option<usize>,
    /// Include directories in results.
    pub include_dirs: bool,
    /// Include hidden files.
    pub include_hidden: bool,
}

impl Default for RecursiveOptions {
    fn default() -> Self {
        Self {
            max_depth: 10,
            follow_symlinks: false,
            use_gitignore: true,
            ignore_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "__pycache__".to_string(),
                "*.pyc".to_string(),
            ],
            extension: None,
            max_results: Some(10000),
            include_dirs: false,
            include_hidden: false,
        }
    }
}

impl RecursiveOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum depth.
    pub fn depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Follow symlinks.
    pub fn follow_symlinks(mut self) -> Self {
        self.follow_symlinks = true;
        self
    }

    /// Disable gitignore.
    pub fn no_gitignore(mut self) -> Self {
        self.use_gitignore = false;
        self
    }

    /// Add ignore pattern.
    pub fn ignore(mut self, pattern: &str) -> Self {
        self.ignore_patterns.push(pattern.to_string());
        self
    }

    /// Filter by extension.
    pub fn extension(mut self, ext: &str) -> Self {
        self.extension = Some(ext.trim_start_matches('.').to_string());
        self
    }

    /// Set maximum results.
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// Include directories.
    pub fn include_dirs(mut self) -> Self {
        self.include_dirs = true;
        self
    }

    /// Include hidden files.
    pub fn include_hidden(mut self) -> Self {
        self.include_hidden = true;
        self
    }
}

/// List files recursively.
#[instrument(skip(ctx), fields(path = %path, op_id = %ctx.operation_id))]
pub async fn list_files_recursive(
    ctx: &PrimitiveContext,
    path: &str,
    options: RecursiveOptions,
) -> PrimitiveResult<ListFilesResult> {
    let start = Instant::now();

    let resolved_path = ctx.resolve_path(path);
    debug!("Recursively listing: {:?}", resolved_path);

    if !ctx.is_path_allowed(&resolved_path) {
        return Err(PrimitiveError::PathNotAllowed {
            path: resolved_path,
        });
    }

    if !resolved_path.exists() {
        return Err(PrimitiveError::FileNotFound {
            path: resolved_path,
        });
    }

    // Load gitignore patterns if enabled
    let gitignore_patterns = if options.use_gitignore {
        load_gitignore_patterns(&resolved_path)
    } else {
        HashSet::new()
    };

    // Combine all ignore patterns
    let all_ignore: HashSet<_> = options
        .ignore_patterns
        .iter()
        .cloned()
        .chain(gitignore_patterns)
        .collect();

    // Configure walker
    let mut walker = WalkDir::new(&resolved_path)
        .max_depth(options.max_depth)
        .follow_links(options.follow_symlinks);

    if !options.include_hidden {
        walker = walker.into_iter()
            .filter_entry(|e| !is_hidden(e))
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
    }

    // Collect entries
    let mut entries = Vec::new();
    let mut truncated = false;
    let max_results = options.max_results.unwrap_or(usize::MAX);

    let walker = WalkDir::new(&resolved_path)
        .max_depth(options.max_depth)
        .follow_links(options.follow_symlinks);

    for entry_result in walker {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                warn!("Error walking directory: {}", e);
                continue;
            }
        };

        // Skip root
        if entry.path() == resolved_path {
            continue;
        }

        // Apply filters
        if !options.include_hidden && is_hidden(&entry) {
            continue;
        }

        if should_ignore(entry.path(), &all_ignore) {
            continue;
        }

        let is_dir = entry.file_type().is_dir();

        if !options.include_dirs && is_dir {
            continue;
        }

        // Extension filter
        if let Some(ref ext) = options.extension {
            if !is_dir {
                let file_ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase());
                if file_ext.as_deref() != Some(ext.to_lowercase().as_str()) {
                    continue;
                }
            }
        }

        // Check path allowed
        let entry_path = entry.path().to_path_buf();
        if !ctx.is_path_allowed(&entry_path) {
            continue;
        }

        let size = if is_dir {
            None
        } else {
            entry.metadata().ok().map(|m| m.len())
        };

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

        // Check limit
        if entries.len() >= max_results {
            truncated = true;
            break;
        }
    }

    let duration = start.elapsed();
    debug!("Found {} entries in {:?}", entries.len(), duration);

    Ok(ListFilesResult {
        entries: entries.clone(),
        base_path: resolved_path,
        total_count: entries.len(),
        truncated,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "list_files_recursive".to_string(),
        },
    })
}

/// Check if entry is hidden.
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// Check if path should be ignored.
fn should_ignore(path: &Path, patterns: &HashSet<String>) -> bool {
    for component in path.components() {
        if let Some(name) = component.as_os_str().to_str() {
            for pattern in patterns {
                if matches_pattern(name, pattern) {
                    return true;
                }
            }
        }
    }
    false
}

/// Simple pattern matching.
fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        name.ends_with(suffix)
    } else if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        name.starts_with(prefix)
    } else {
        name == pattern
    }
}

/// Load patterns from .gitignore files.
fn load_gitignore_patterns(root: &Path) -> HashSet<String> {
    let mut patterns = HashSet::new();
    let gitignore_path = root.join(".gitignore");

    if gitignore_path.exists() {
        if let Ok(content) = fs::read_to_string(&gitignore_path) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    patterns.insert(line.to_string());
                }
            }
        }
    }

    patterns
}

/// Iterator for streaming results.
pub struct RecursiveIterator {
    walker: Box<dyn Iterator<Item = walkdir::Result<DirEntry>> + Send>,
    options: RecursiveOptions,
    ignore_patterns: HashSet<String>,
    count: usize,
}

impl Iterator for RecursiveIterator {
    type Item = PrimitiveResult<FileEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let max = self.options.max_results.unwrap_or(usize::MAX);

        while self.count < max {
            match self.walker.next()? {
                Ok(entry) => {
                    if let Some(file_entry) = self.process_entry(entry) {
                        self.count += 1;
                        return Some(Ok(file_entry));
                    }
                }
                Err(e) => {
                    warn!("Walk error: {}", e);
                    continue;
                }
            }
        }
        None
    }
}

impl RecursiveIterator {
    fn process_entry(&self, entry: DirEntry) -> Option<FileEntry> {
        if !self.options.include_hidden && is_hidden(&entry) {
            return None;
        }

        if should_ignore(entry.path(), &self.ignore_patterns) {
            return None;
        }

        let is_dir = entry.file_type().is_dir();
        if !self.options.include_dirs && is_dir {
            return None;
        }

        Some(FileEntry {
            path: entry.path().to_path_buf(),
            is_dir,
            size: if is_dir { None } else { entry.metadata().ok().map(|m| m.len()) },
            extension: entry.path().extension().and_then(|e| e.to_str()).map(|s| s.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::{create_dir_all, write};

    #[tokio::test]
    async fn test_recursive_basic() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.txt"), "a").unwrap();
        create_dir_all(dir.path().join("sub/deep")).unwrap();
        write(dir.path().join("sub/b.txt"), "b").unwrap();
        write(dir.path().join("sub/deep/c.txt"), "c").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = RecursiveOptions::new().depth(10);
        let result = list_files_recursive(&ctx, ".", opts).await.unwrap();

        assert_eq!(result.entries.len(), 3);
    }

    #[tokio::test]
    async fn test_recursive_depth_limit() {
        let dir = tempdir().unwrap();
        create_dir_all(dir.path().join("a/b/c/d")).unwrap();
        write(dir.path().join("a/1.txt"), "1").unwrap();
        write(dir.path().join("a/b/2.txt"), "2").unwrap();
        write(dir.path().join("a/b/c/3.txt"), "3").unwrap();
        write(dir.path().join("a/b/c/d/4.txt"), "4").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = RecursiveOptions::new().depth(2);
        let result = list_files_recursive(&ctx, ".", opts).await.unwrap();

        // Should only get files up to depth 2
        assert!(result.entries.len() <= 2);
    }

    #[tokio::test]
    async fn test_recursive_gitignore() {
        let dir = tempdir().unwrap();
        write(dir.path().join(".gitignore"), "ignored.txt\n*.log").unwrap();
        write(dir.path().join("keep.txt"), "keep").unwrap();
        write(dir.path().join("ignored.txt"), "ignored").unwrap();
        write(dir.path().join("test.log"), "log").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = RecursiveOptions::new();
        let result = list_files_recursive(&ctx, ".", opts).await.unwrap();

        let names: Vec<_> = result.entries.iter()
            .filter_map(|e| e.path.file_name())
            .filter_map(|n| n.to_str())
            .collect();

        assert!(names.contains(&"keep.txt"));
        assert!(!names.contains(&"ignored.txt"));
        assert!(!names.contains(&"test.log"));
    }
}
```

---

## Testing Requirements

1. Recursive traversal finds nested files
2. Depth limit is enforced
3. Gitignore patterns are respected
4. Hidden files are excluded by default
5. Symlinks are handled according to options
6. Large directories don't cause memory issues
7. Maximum results limit works

---

## Related Specs

- Depends on: [034-list-files-impl.md](034-list-files-impl.md)
- Next: [036-bash-exec-core.md](036-bash-exec-core.md)
- Related: [043-code-search-core.md](043-code-search-core.md)
