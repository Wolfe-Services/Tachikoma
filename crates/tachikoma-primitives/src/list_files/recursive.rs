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
    /// Progress callback frequency (call every N entries).
    pub progress_callback_frequency: Option<usize>,
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
            progress_callback_frequency: Some(1000), // Every 1000 entries by default
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

    /// Set progress callback frequency.
    pub fn progress_frequency(mut self, frequency: usize) -> Self {
        self.progress_callback_frequency = Some(frequency);
        self
    }

    /// Disable progress callbacks.
    pub fn no_progress_callbacks(mut self) -> Self {
        self.progress_callback_frequency = None;
        self
    }
}

/// List files recursively with optional progress callback.
#[instrument(skip(ctx), fields(path = %path, op_id = %ctx.operation_id))]
pub async fn list_files_recursive(
    ctx: &PrimitiveContext,
    path: &str,
    options: RecursiveOptions,
) -> PrimitiveResult<ListFilesResult> {
    list_files_recursive_with_callback(ctx, path, options, None::<fn(usize, &Path)>).await
}

/// List files recursively with optional progress callback.
#[instrument(skip(ctx, progress_callback), fields(path = %path, op_id = %ctx.operation_id))]
pub async fn list_files_recursive_with_callback<F>(
    ctx: &PrimitiveContext,
    path: &str,
    options: RecursiveOptions,
    mut progress_callback: Option<F>,
) -> PrimitiveResult<ListFilesResult>
where
    F: FnMut(usize, &Path) + Send,
{
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
    let walker = WalkDir::new(&resolved_path)
        .max_depth(options.max_depth)
        .follow_links(options.follow_symlinks);

    // Collect entries
    let mut entries = Vec::new();
    let mut truncated = false;
    let max_results = options.max_results.unwrap_or(usize::MAX);

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

        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());

        entries.push(FileEntry {
            path: entry_path,
            is_dir,
            size,
            extension,
            modified,
        });

        // Call progress callback if configured
        if let Some(freq) = options.progress_callback_frequency {
            if entries.len() % freq == 0 {
                if let Some(ref mut callback) = progress_callback {
                    callback(entries.len(), entry.path());
                }
            }
        }

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
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let inner = &pattern[1..pattern.len() - 1];
        name.contains(inner)
    } else if pattern.starts_with('*') {
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
    walker: walkdir::IntoIter,
    options: RecursiveOptions,
    ignore_patterns: HashSet<String>,
    count: usize,
    ctx: PrimitiveContext,
    base_path: PathBuf,
}

impl RecursiveIterator {
    /// Create a new recursive iterator.
    pub fn new(
        ctx: PrimitiveContext, 
        path: &Path, 
        options: RecursiveOptions
    ) -> PrimitiveResult<Self> {
        let gitignore_patterns = if options.use_gitignore {
            load_gitignore_patterns(path)
        } else {
            HashSet::new()
        };

        let all_ignore: HashSet<_> = options
            .ignore_patterns
            .iter()
            .cloned()
            .chain(gitignore_patterns)
            .collect();

        let walker = WalkDir::new(path)
            .max_depth(options.max_depth)
            .follow_links(options.follow_symlinks)
            .into_iter();

        Ok(Self {
            walker,
            options,
            ignore_patterns: all_ignore,
            count: 0,
            ctx,
            base_path: path.to_path_buf(),
        })
    }

    fn process_entry(&self, entry: DirEntry) -> Option<FileEntry> {
        // Skip root
        if entry.path() == self.base_path {
            return None;
        }

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

        // Extension filter
        if let Some(ref ext) = self.options.extension {
            if !is_dir {
                let file_ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase());
                if file_ext.as_deref() != Some(ext.to_lowercase().as_str()) {
                    return None;
                }
            }
        }

        // Check path allowed
        let entry_path = entry.path().to_path_buf();
        if !self.ctx.is_path_allowed(&entry_path) {
            return None;
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

        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());

        Some(FileEntry {
            path: entry_path,
            is_dir,
            size,
            extension,
            modified,
        })
    }
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

    #[tokio::test]
    async fn test_recursive_symlinks() {
        let dir = tempdir().unwrap();
        write(dir.path().join("real.txt"), "content").unwrap();
        
        // Create a symlink (skip on Windows if it fails)
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if symlink(dir.path().join("real.txt"), dir.path().join("link.txt")).is_ok() {
                let ctx = PrimitiveContext::new(dir.path().to_path_buf());
                
                // Don't follow symlinks
                let opts = RecursiveOptions::new();
                let result = list_files_recursive(&ctx, ".", opts).await.unwrap();
                // Should include both real.txt and link.txt (symlink itself, not target)
                assert_eq!(result.entries.len(), 2); 
                
                // Follow symlinks - same result because we see both the link and the original
                let opts = RecursiveOptions::new().follow_symlinks();
                let result = list_files_recursive(&ctx, ".", opts).await.unwrap();
                assert_eq!(result.entries.len(), 2);
            }
        }
        
        #[cfg(not(unix))]
        {
            // On non-Unix systems, just test basic functionality
            let ctx = PrimitiveContext::new(dir.path().to_path_buf());
            let opts = RecursiveOptions::new();
            let result = list_files_recursive(&ctx, ".", opts).await.unwrap();
            assert_eq!(result.entries.len(), 1); // Only real.txt
        }
    }

    #[tokio::test]
    async fn test_recursive_max_results() {
        let dir = tempdir().unwrap();
        for i in 0..100 {
            write(dir.path().join(format!("file{:03}.txt", i)), "content").unwrap();
        }

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = RecursiveOptions::new().max_results(50);
        let result = list_files_recursive(&ctx, ".", opts).await.unwrap();

        assert_eq!(result.entries.len(), 50);
        assert!(result.truncated);
    }

    #[tokio::test]
    async fn test_recursive_iterator() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.txt"), "a").unwrap();
        create_dir_all(dir.path().join("sub")).unwrap();
        write(dir.path().join("sub/b.txt"), "b").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = RecursiveOptions::new().depth(10);
        let mut iter = RecursiveIterator::new(ctx, dir.path(), opts).unwrap();

        let mut count = 0;
        while let Some(entry_result) = iter.next() {
            assert!(entry_result.is_ok());
            count += 1;
        }
        
        assert_eq!(count, 2); // a.txt and sub/b.txt
    }

    #[tokio::test]
    async fn test_recursive_progress_callback() {
        let dir = tempdir().unwrap();
        // Create multiple files to trigger progress callbacks
        for i in 0..15 {
            write(dir.path().join(format!("file{:02}.txt", i)), "content").unwrap();
        }

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = RecursiveOptions::new().progress_frequency(5); // Every 5 files
        
        let mut progress_calls = Vec::new();
        let callback = |count: usize, path: &Path| {
            progress_calls.push((count, path.to_path_buf()));
        };
        
        let result = list_files_recursive_with_callback(&ctx, ".", opts, Some(callback)).await.unwrap();
        
        assert_eq!(result.entries.len(), 15);
        // Should have been called at counts 5, 10, 15
        assert_eq!(progress_calls.len(), 3);
        assert_eq!(progress_calls[0].0, 5);
        assert_eq!(progress_calls[1].0, 10);
        assert_eq!(progress_calls[2].0, 15);
    }
}