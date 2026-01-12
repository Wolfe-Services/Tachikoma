//! Edit file primitive implementation.

mod options;
mod diff;
mod unique;

pub use options::EditFileOptions;
pub use diff::Diff;
pub use unique::{
    UniquenessResult, MatchLocation, MatchSelection, EditValidationError,
    check_uniqueness, format_matches, select_match, validate_edit_target
};

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{EditFileResult, ExecutionMetadata},
};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, instrument};

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

    // Read file with encoding preservation
    let original_bytes = fs::read(&resolved_path).map_err(|e| {
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

    // Detect line endings
    let line_ending = detect_line_endings(&original_bytes);

    // Convert to string - use lossy conversion to handle various encodings
    let content = String::from_utf8_lossy(&original_bytes);

    // Count occurrences and handle uniqueness
    let count = content.matches(old_string).count();

    if count == 0 {
        return Err(PrimitiveError::TargetNotFound);
    }

    // Handle non-unique matches
    if !options.replace_all && count > 1 {
        if let Some(selection) = options.force_selection {
            // Force mode: validate the selection exists
            let uniqueness = unique::check_uniqueness(&content, old_string, 3);
            let selected_match = unique::select_match(&uniqueness, selection);
            
            if selected_match.is_none() {
                return Err(PrimitiveError::Validation {
                    message: format!(
                        "Invalid force selection {:?}: {} matches available",
                        selection, count
                    ),
                });
            }
        } else {
            // Normal mode: return error with detailed match information
            let uniqueness = unique::check_uniqueness(&content, old_string, 3);
            return Err(PrimitiveError::NotUnique {
                count,
                details: unique::format_matches(&uniqueness),
            });
        }
    }

    // Perform replacement
    let new_content = if options.replace_all {
        content.replace(old_string, new_string)
    } else if let Some(selection) = options.force_selection {
        // Force mode with specific match selection
        let uniqueness = unique::check_uniqueness(&content, old_string, 3);
        if let Some(selected_match) = unique::select_match(&uniqueness, selection) {
            // We need to find and replace the exact match at the specified location
            // Convert content to bytes for accurate offset handling
            let content_bytes = content.as_bytes();
            let old_string_bytes = old_string.as_bytes();
            let new_string_bytes = new_string.as_bytes();
            
            let start = selected_match.offset;
            let end = start + old_string_bytes.len();
            
            if end <= content_bytes.len() && &content_bytes[start..end] == old_string_bytes {
                let mut new_bytes = Vec::new();
                new_bytes.extend_from_slice(&content_bytes[..start]);
                new_bytes.extend_from_slice(new_string_bytes);
                new_bytes.extend_from_slice(&content_bytes[end..]);
                
                String::from_utf8_lossy(&new_bytes).into_owned()
            } else {
                return Err(PrimitiveError::Validation {
                    message: "Match location is invalid".to_string(),
                });
            }
        } else {
            return Err(PrimitiveError::Validation {
                message: "Selected match not found".to_string(),
            });
        }
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
        let backup_path = resolved_path.with_extension(
            format!("{}.bak", resolved_path.extension().and_then(|s| s.to_str()).unwrap_or(""))
        );
        fs::copy(&resolved_path, &backup_path)?;
        debug!("Created backup at {:?}", backup_path);
    }

    // Convert back to bytes with original line endings preserved
    let new_bytes = preserve_line_endings(&new_content, line_ending);

    // Preserve original file metadata
    let original_metadata = fs::metadata(&resolved_path)?;

    // Write new content
    fs::write(&resolved_path, &new_bytes)?;

    // Preserve permissions if requested
    if options.preserve_permissions {
        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            let permissions = Permissions::from_mode(original_metadata.permissions().mode());
            fs::set_permissions(&resolved_path, permissions)?;
        }
        
        #[cfg(not(unix))]
        {
            // On non-Unix systems, just set the readonly flag if it was set
            let permissions = original_metadata.permissions();
            fs::set_permissions(&resolved_path, permissions)?;
        }
    }

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

    let original_bytes = fs::read(&resolved_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PrimitiveError::FileNotFound {
                path: resolved_path.clone(),
            }
        } else {
            PrimitiveError::Io(e)
        }
    })?;

    let content = String::from_utf8_lossy(&original_bytes);
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

/// Detect line endings in file content.
#[derive(Debug, Clone, Copy)]
enum LineEnding {
    Unix,    // LF (\n)
    Windows, // CRLF (\r\n)
    Classic, // CR (\r)
    Mixed,   // Multiple types found
}

/// Detect the primary line ending style in the file.
fn detect_line_endings(bytes: &[u8]) -> LineEnding {
    let mut crlf_count = 0;
    let mut lf_count = 0;
    let mut cr_count = 0;

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                crlf_count += 1;
                i += 2; // Skip both \r and \n
            } else {
                cr_count += 1;
                i += 1;
            }
        } else if bytes[i] == b'\n' {
            lf_count += 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    // Determine the predominant line ending
    if crlf_count > lf_count && crlf_count > cr_count {
        LineEnding::Windows
    } else if lf_count > crlf_count && lf_count > cr_count {
        LineEnding::Unix
    } else if cr_count > crlf_count && cr_count > lf_count {
        LineEnding::Classic
    } else if crlf_count > 0 || lf_count > 0 || cr_count > 0 {
        LineEnding::Mixed
    } else {
        // Default to Unix if no line endings found
        LineEnding::Unix
    }
}

/// Convert string content to bytes while preserving the original line ending style.
fn preserve_line_endings(content: &str, line_ending: LineEnding) -> Vec<u8> {
    match line_ending {
        LineEnding::Windows => content.replace('\n', "\r\n").into_bytes(),
        LineEnding::Classic => content.replace('\n', "\r").into_bytes(),
        LineEnding::Unix | LineEnding::Mixed => content.as_bytes().to_vec(),
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

        assert!(matches!(result, Err(PrimitiveError::NotUnique { count: 3, .. })));
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

    #[tokio::test]
    async fn test_edit_file_backup() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = EditFileOptions::new().with_backup();
        let result = edit_file(&ctx, "test.txt", "World", "Rust", Some(opts)).await.unwrap();

        assert!(result.success);

        // Check original file was changed
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, Rust!");

        // Check backup was created
        let backup_path = dir.path().join("test.txt.bak");
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_edit_file_windows_line_endings() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        // Write file with Windows line endings
        fs::write(&file_path, b"line1\r\nline2\r\nline3\r\n").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = edit_file(&ctx, "test.txt", "line2", "modified", None).await.unwrap();

        assert!(result.success);

        // Check that Windows line endings are preserved
        let bytes = fs::read(&file_path).unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("\r\n"));
        assert!(content.contains("modified"));
    }

    #[tokio::test]
    async fn test_edit_preview() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!\nGoodbye, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let preview = edit_file_preview(&ctx, "test.txt", "World", "Rust").await.unwrap();

        assert_eq!(preview.occurrences, 2);
        assert!(preview.has_changes());
        assert!(!preview.is_unique());
        assert_eq!(preview.old_string, "World");
        assert_eq!(preview.new_string, "Rust");

        let diff_str = preview.format_diff();
        assert!(diff_str.contains("-Hello, World!"));
        assert!(diff_str.contains("+Hello, Rust!"));
    }

    #[tokio::test]
    async fn test_empty_old_string_validation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = edit_file(&ctx, "test.txt", "", "replacement", None).await;

        assert!(matches!(result, Err(PrimitiveError::Validation { .. })));
    }

    #[tokio::test]
    async fn test_identical_strings_validation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = edit_file(&ctx, "test.txt", "same", "same", None).await;

        assert!(matches!(result, Err(PrimitiveError::Validation { .. })));
    }

    #[tokio::test]
    async fn test_edit_file_comprehensive() {
        // Test comprehensive functionality with all features
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("comprehensive.txt");
        let content = "function old_name() {\n    console.log('old');\n    return old_value;\n}";
        write(&file_path, content).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // First, preview the changes
        let preview = edit_file_preview(&ctx, "comprehensive.txt", "old", "new").await.unwrap();
        assert_eq!(preview.occurrences, 3);
        assert!(preview.has_changes());
        assert!(!preview.is_unique());
        
        let diff = preview.format_diff();
        assert!(diff.contains("-function old_name()"));
        assert!(diff.contains("+function new_name()"));
        
        // Now perform the actual edit with backup
        let opts = EditFileOptions::new()
            .replace_all()
            .with_backup()
            .preserve_permissions();
            
        let result = edit_file(&ctx, "comprehensive.txt", "old", "new", Some(opts)).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.replacements, 3);
        
        // Check the file was changed
        let new_content = fs::read_to_string(&file_path).unwrap();
        assert!(new_content.contains("function new_name()"));
        assert!(new_content.contains("console.log('new');"));
        assert!(new_content.contains("return new_value;"));
        assert!(!new_content.contains("old"));
        
        // Check backup was created
        let backup_path = dir.path().join("comprehensive.txt.bak");
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, content);
    }

    #[tokio::test]
    async fn test_edit_file_force_selection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "foo bar\nbaz foo\nfoo qux").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Force edit first match
        let opts = EditFileOptions::new().force_first();
        let result = edit_file(&ctx, "test.txt", "foo", "replaced", Some(opts)).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.replacements, 1);
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "replaced bar\nbaz foo\nfoo qux");
    }

    #[tokio::test]
    async fn test_edit_file_force_selection_by_line() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "foo bar\nbaz foo\nfoo qux").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Force edit match on line 3
        let opts = EditFileOptions::new().force_line(3);
        let result = edit_file(&ctx, "test.txt", "foo", "replaced", Some(opts)).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.replacements, 1);
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "foo bar\nbaz foo\nreplaced qux");
    }

    #[tokio::test]
    async fn test_edit_file_force_invalid_selection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "foo bar\nbaz foo\nfoo qux").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Try to force edit a non-existent line
        let opts = EditFileOptions::new().force_line(10);
        let result = edit_file(&ctx, "test.txt", "foo", "replaced", Some(opts)).await;
        
        assert!(matches!(result, Err(PrimitiveError::Validation { .. })));
    }
}