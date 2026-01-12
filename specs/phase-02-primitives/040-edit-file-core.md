# 040 - Edit File Core Implementation

**Phase:** 2 - Five Primitives
**Spec ID:** 040
**Status:** Planned
**Dependencies:** 031-primitives-crate
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement the `edit_file` primitive that performs search-and-replace operations in files with exact string matching.

---

## Acceptance Criteria

- [ ] Search and replace with exact string matching
- [ ] Support for multi-line old_string/new_string
- [ ] Preserve file encoding and line endings
- [ ] Backup original file option
- [ ] Dry-run mode for preview
- [ ] Return diff of changes

---

## Implementation Details

### 1. Edit File Module (src/edit_file/mod.rs)

```rust
//! Edit file primitive implementation.

mod options;
mod diff;

pub use options::EditFileOptions;
pub use diff::Diff;

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{EditFileResult, ExecutionMetadata},
};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, instrument, warn};

/// Edit a file using search and replace.
///
/// # Arguments
///
/// * `ctx` - Execution context
/// * `path` - Path to the file
/// * `old_string` - String to search for
/// * `new_string` - String to replace with
/// * `options` - Optional configuration
///
/// # Returns
///
/// Result indicating success and number of replacements.
///
/// # Example
///
/// ```no_run
/// use tachikoma_primitives::{PrimitiveContext, edit_file, EditFileOptions};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ctx = PrimitiveContext::new(PathBuf::from("."));
///
/// // Simple replacement
/// let result = edit_file(
///     &ctx,
///     "src/main.rs",
///     "old_function",
///     "new_function",
///     None,
/// ).await?;
///
/// println!("Made {} replacements", result.replacements);
/// # Ok(())
/// # }
/// ```
#[instrument(skip(ctx, old_string, new_string, options), fields(path = %path, op_id = %ctx.operation_id))]
pub async fn edit_file(
    ctx: &PrimitiveContext,
    path: &str,
    old_string: &str,
    new_string: &str,
    options: Option<EditFileOptions>,
) -> PrimitiveResult<EditFileResult> {
    let start = Instant::now();
    let options = options.unwrap_or_default();

    // Validate inputs
    if old_string.is_empty() {
        return Err(PrimitiveError::Validation {
            message: "old_string cannot be empty".to_string(),
        });
    }

    if old_string == new_string {
        return Err(PrimitiveError::Validation {
            message: "old_string and new_string are identical".to_string(),
        });
    }

    // Resolve and validate path
    let resolved_path = ctx.resolve_path(path);
    debug!("Editing file: {:?}", resolved_path);

    if !ctx.is_path_allowed(&resolved_path) {
        return Err(PrimitiveError::PathNotAllowed {
            path: resolved_path,
        });
    }

    // Read file
    let content = fs::read_to_string(&resolved_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PrimitiveError::FileNotFound {
                path: resolved_path.clone(),
            }
        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
            PrimitiveError::PermissionDenied {
                path: resolved_path.clone(),
            }
        } else {
            PrimitiveError::Io(e)
        }
    })?;

    // Count occurrences
    let count = content.matches(old_string).count();

    if count == 0 {
        return Err(PrimitiveError::TargetNotFound);
    }

    // Check uniqueness if required
    if !options.replace_all && count > 1 {
        return Err(PrimitiveError::NotUnique { count });
    }

    // Perform replacement
    let new_content = if options.replace_all {
        content.replace(old_string, new_string)
    } else {
        content.replacen(old_string, new_string, 1)
    };

    let replacements = if options.replace_all { count } else { 1 };

    // Dry run - don't write
    if options.dry_run {
        let duration = start.elapsed();
        return Ok(EditFileResult {
            success: true,
            replacements,
            path: resolved_path,
            metadata: ExecutionMetadata {
                duration,
                operation_id: ctx.operation_id.clone(),
                primitive: "edit_file".to_string(),
            },
        });
    }

    // Create backup if requested
    if options.backup {
        let backup_path = resolved_path.with_extension("bak");
        fs::copy(&resolved_path, &backup_path)?;
        debug!("Created backup at {:?}", backup_path);
    }

    // Write new content
    fs::write(&resolved_path, &new_content)?;

    let duration = start.elapsed();
    debug!(
        "Made {} replacement(s) in {:?}",
        replacements, duration
    );

    Ok(EditFileResult {
        success: true,
        replacements,
        path: resolved_path,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "edit_file".to_string(),
        },
    })
}

/// Preview the changes without applying them.
pub async fn edit_file_preview(
    ctx: &PrimitiveContext,
    path: &str,
    old_string: &str,
    new_string: &str,
) -> PrimitiveResult<EditPreview> {
    let resolved_path = ctx.resolve_path(path);

    if !ctx.is_path_allowed(&resolved_path) {
        return Err(PrimitiveError::PathNotAllowed {
            path: resolved_path,
        });
    }

    let content = fs::read_to_string(&resolved_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PrimitiveError::FileNotFound {
                path: resolved_path.clone(),
            }
        } else {
            PrimitiveError::Io(e)
        }
    })?;

    let count = content.matches(old_string).count();
    let new_content = content.replace(old_string, new_string);
    let diff = diff::create_diff(&content, &new_content);

    Ok(EditPreview {
        path: resolved_path,
        occurrences: count,
        diff,
        old_string: old_string.to_string(),
        new_string: new_string.to_string(),
    })
}

/// Preview of edit changes.
#[derive(Debug, Clone)]
pub struct EditPreview {
    /// File path.
    pub path: PathBuf,
    /// Number of occurrences found.
    pub occurrences: usize,
    /// Diff of changes.
    pub diff: Diff,
    /// Original string.
    pub old_string: String,
    /// Replacement string.
    pub new_string: String,
}

impl EditPreview {
    /// Check if changes would be made.
    pub fn has_changes(&self) -> bool {
        self.occurrences > 0
    }

    /// Check if target is unique.
    pub fn is_unique(&self) -> bool {
        self.occurrences == 1
    }

    /// Get formatted diff for display.
    pub fn format_diff(&self) -> String {
        self.diff.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::write;

    #[tokio::test]
    async fn test_edit_file_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = edit_file(&ctx, "test.txt", "World", "Rust", None).await.unwrap();

        assert!(result.success);
        assert_eq!(result.replacements, 1);

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, Rust!");
    }

    #[tokio::test]
    async fn test_edit_file_not_found() {
        let dir = tempdir().unwrap();
        let ctx = PrimitiveContext::new(dir.path().to_path_buf());

        let result = edit_file(&ctx, "nonexistent.txt", "a", "b", None).await;
        assert!(matches!(result, Err(PrimitiveError::FileNotFound { .. })));
    }

    #[tokio::test]
    async fn test_edit_file_target_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = edit_file(&ctx, "test.txt", "NotHere", "Something", None).await;

        assert!(matches!(result, Err(PrimitiveError::TargetNotFound)));
    }

    #[tokio::test]
    async fn test_edit_file_not_unique() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "foo bar foo baz foo").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = edit_file(&ctx, "test.txt", "foo", "qux", None).await;

        assert!(matches!(result, Err(PrimitiveError::NotUnique { count: 3 })));
    }

    #[tokio::test]
    async fn test_edit_file_replace_all() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "foo bar foo baz foo").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = EditFileOptions::new().replace_all();
        let result = edit_file(&ctx, "test.txt", "foo", "qux", Some(opts)).await.unwrap();

        assert!(result.success);
        assert_eq!(result.replacements, 3);

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "qux bar qux baz qux");
    }

    #[tokio::test]
    async fn test_edit_file_multiline() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "fn old() {\n    println!(\"old\");\n}").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = edit_file(
            &ctx,
            "test.txt",
            "fn old() {\n    println!(\"old\");\n}",
            "fn new() {\n    println!(\"new\");\n}",
            None,
        ).await.unwrap();

        assert!(result.success);
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("fn new()"));
        assert!(content.contains("println!(\"new\")"));
    }

    #[tokio::test]
    async fn test_edit_file_dry_run() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = EditFileOptions::new().dry_run();
        let result = edit_file(&ctx, "test.txt", "World", "Rust", Some(opts)).await.unwrap();

        assert!(result.success);
        assert_eq!(result.replacements, 1);

        // File should be unchanged
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }
}
```

### 2. Edit File Options (src/edit_file/options.rs)

```rust
//! Options for edit_file primitive.

/// Options for editing a file.
#[derive(Debug, Clone, Default)]
pub struct EditFileOptions {
    /// Replace all occurrences (not just first unique match).
    pub replace_all: bool,
    /// Create a backup of the original file.
    pub backup: bool,
    /// Don't actually write changes (preview mode).
    pub dry_run: bool,
    /// Preserve original file permissions.
    pub preserve_permissions: bool,
}

impl EditFileOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace all occurrences.
    pub fn replace_all(mut self) -> Self {
        self.replace_all = true;
        self
    }

    /// Create backup before editing.
    pub fn with_backup(mut self) -> Self {
        self.backup = true;
        self
    }

    /// Preview changes without writing.
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Preserve file permissions.
    pub fn preserve_permissions(mut self) -> Self {
        self.preserve_permissions = true;
        self
    }
}
```

### 3. Diff Generation (src/edit_file/diff.rs)

```rust
//! Diff generation for edit preview.

use std::fmt;

/// A unified diff representation.
#[derive(Debug, Clone)]
pub struct Diff {
    /// Diff hunks.
    pub hunks: Vec<DiffHunk>,
}

/// A single diff hunk.
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Starting line in old file.
    pub old_start: usize,
    /// Number of lines in old file.
    pub old_count: usize,
    /// Starting line in new file.
    pub new_start: usize,
    /// Number of lines in new file.
    pub new_count: usize,
    /// Lines in the hunk.
    pub lines: Vec<DiffLine>,
}

/// A single diff line.
#[derive(Debug, Clone)]
pub enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
}

impl fmt::Display for DiffLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffLine::Context(s) => write!(f, " {}", s),
            DiffLine::Added(s) => write!(f, "+{}", s),
            DiffLine::Removed(s) => write!(f, "-{}", s),
        }
    }
}

impl fmt::Display for DiffHunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_count, self.new_start, self.new_count
        )?;
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for hunk in &self.hunks {
            write!(f, "{}", hunk)?;
        }
        Ok(())
    }
}

/// Create a diff between two strings.
pub fn create_diff(old: &str, new: &str) -> Diff {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut hunks = Vec::new();
    let mut old_idx = 0;
    let mut new_idx = 0;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        // Find next difference
        while old_idx < old_lines.len()
            && new_idx < new_lines.len()
            && old_lines[old_idx] == new_lines[new_idx]
        {
            old_idx += 1;
            new_idx += 1;
        }

        if old_idx >= old_lines.len() && new_idx >= new_lines.len() {
            break;
        }

        // Create hunk
        let hunk_old_start = old_idx.saturating_sub(2);
        let hunk_new_start = new_idx.saturating_sub(2);

        let mut lines = Vec::new();

        // Context before
        for i in hunk_old_start..old_idx {
            if i < old_lines.len() {
                lines.push(DiffLine::Context(old_lines[i].to_string()));
            }
        }

        // Find extent of changes
        let mut old_end = old_idx;
        let mut new_end = new_idx;

        while old_end < old_lines.len() || new_end < new_lines.len() {
            if old_end < old_lines.len()
                && new_end < new_lines.len()
                && old_lines[old_end] == new_lines[new_end]
            {
                // Check if we have enough context to end hunk
                let mut context_count = 0;
                let mut check_old = old_end;
                let mut check_new = new_end;
                while check_old < old_lines.len()
                    && check_new < new_lines.len()
                    && old_lines[check_old] == new_lines[check_new]
                {
                    context_count += 1;
                    check_old += 1;
                    check_new += 1;
                    if context_count >= 4 {
                        break;
                    }
                }
                if context_count >= 4 || (check_old >= old_lines.len() && check_new >= new_lines.len()) {
                    break;
                }
            }
            old_end += 1;
            new_end += 1;
        }

        // Add removed lines
        for i in old_idx..old_end.min(old_lines.len()) {
            lines.push(DiffLine::Removed(old_lines[i].to_string()));
        }

        // Add added lines
        for i in new_idx..new_end.min(new_lines.len()) {
            lines.push(DiffLine::Added(new_lines[i].to_string()));
        }

        // Context after
        let context_end = old_end + 2;
        for i in old_end..context_end.min(old_lines.len()) {
            lines.push(DiffLine::Context(old_lines[i].to_string()));
        }

        if !lines.is_empty() {
            hunks.push(DiffHunk {
                old_start: hunk_old_start + 1,
                old_count: old_end - hunk_old_start,
                new_start: hunk_new_start + 1,
                new_count: new_end - hunk_new_start,
                lines,
            });
        }

        old_idx = old_end + 2;
        new_idx = new_end + 2;
    }

    Diff { hunks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_diff() {
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3";

        let diff = create_diff(old, new);
        let formatted = diff.to_string();

        assert!(formatted.contains("-line2"));
        assert!(formatted.contains("+modified"));
    }

    #[test]
    fn test_no_changes() {
        let content = "line1\nline2\nline3";
        let diff = create_diff(content, content);

        assert!(diff.hunks.is_empty());
    }
}
```

---

## Testing Requirements

1. Basic replacement works correctly
2. Multi-line strings are handled
3. Target not found returns appropriate error
4. Non-unique matches fail without replace_all
5. replace_all replaces all occurrences
6. Dry run doesn't modify file
7. Backup creates .bak file
8. File permissions are preserved

---

## Related Specs

- Depends on: [031-primitives-crate.md](031-primitives-crate.md)
- Next: [041-edit-file-unique.md](041-edit-file-unique.md)
- Related: [042-edit-file-atomic.md](042-edit-file-atomic.md)
