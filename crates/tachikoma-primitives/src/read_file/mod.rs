//! Read file primitive implementation.

mod error;
mod logging;
mod options;
mod suggest;

pub use error::{ReadFileError, ReadFileErrorResponse};
pub use logging::{log_read_error, log_read_success};
pub use options::ReadFileOptions;
pub use suggest::find_similar_files;

use crate::{
    context::PrimitiveContext,
    error::PrimitiveResult,
    result::{ExecutionMetadata, ReadFileResult},
};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::time::Instant;
use tracing::{debug, instrument, warn};

/// Read a file's contents.
///
/// # Arguments
///
/// * `ctx` - Execution context
/// * `path` - Path to the file (relative or absolute)
/// * `options` - Optional configuration for the read operation
///
/// # Returns
///
/// Result containing file contents and metadata.
///
/// # Example
///
/// ```no_run
/// use tachikoma_primitives::{PrimitiveContext, read_file, ReadFileOptions};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ctx = PrimitiveContext::new(PathBuf::from("."));
///
/// // Read entire file
/// let result = read_file(&ctx, "README.md", None).await?;
///
/// // Read specific lines
/// let opts = ReadFileOptions::new().lines(10, 20);
/// let result = read_file(&ctx, "src/main.rs", Some(opts)).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(ctx), fields(path = %path, op_id = %ctx.operation_id))]
pub async fn read_file(
    ctx: &PrimitiveContext,
    path: &str,
    options: Option<ReadFileOptions>,
) -> PrimitiveResult<ReadFileResult> {
    let start = Instant::now();
    let options = options.unwrap_or_default();

    // Resolve and validate path
    let resolved_path = ctx.resolve_path(path);
    debug!("Reading file: {:?}", resolved_path);

    // Check path permission first
    if !ctx.is_path_allowed(&resolved_path) {
        let err = ReadFileError::PathNotAllowed {
            path: resolved_path,
            reason: "Path is outside allowed directories".to_string(),
        };
        log_read_error(&err, &ctx.operation_id);
        return Err(err.into());
    }

    // Check file exists and get metadata
    let file_metadata = fs::metadata(&resolved_path).map_err(|e| {
        let read_err = error::io_error_with_path(e, resolved_path.clone());
        
        // Add file suggestion for not found errors
        if let ReadFileError::NotFound { path, .. } = &read_err {
            let suggestions = suggest::find_similar_files(path, 1);
            let suggestion = suggestions.into_iter().next();
            let enhanced_err = ReadFileError::NotFound {
                path: path.clone(),
                suggestion,
            };
            log_read_error(&enhanced_err, &ctx.operation_id);
            return enhanced_err;
        }
        
        log_read_error(&read_err, &ctx.operation_id);
        read_err
    })?;

    // Check if it's actually a file
    if !file_metadata.is_file() {
        let actual_type = if file_metadata.is_dir() {
            "directory"
        } else if file_metadata.file_type().is_symlink() {
            "symbolic link"
        } else {
            "special file"
        };
        
        let err = ReadFileError::NotAFile {
            path: resolved_path,
            actual_type: actual_type.to_string(),
        };
        log_read_error(&err, &ctx.operation_id);
        return Err(err.into());
    }

    let file_size = file_metadata.len();
    let max_size = options.max_size.unwrap_or(ctx.config.max_file_size);

    // Check size limit - only fail if using default max_size and no line range
    if file_size as usize > max_size && options.max_size.is_none() && options.start_line.is_none() {
        let err = ReadFileError::TooLarge {
            path: resolved_path,
            actual_size: file_size,
            max_size,
        };
        log_read_error(&err, &ctx.operation_id);
        return Err(err.into());
    }

    // Validate line range if specified
    if let Some(start_line) = options.start_line {
        let end_line = options.end_line.unwrap_or(usize::MAX);
        if start_line == 0 || (end_line != usize::MAX && start_line > end_line) {
            let err = ReadFileError::InvalidLineRange {
                start: start_line,
                end: end_line,
                total_lines: 0, // We don't know yet, but range is invalid anyway
            };
            log_read_error(&err, &ctx.operation_id);
            return Err(err.into());
        }
    }

    // Read content
    let (content, truncated) = read_file_content(&resolved_path, &options, max_size, &ctx.operation_id)?;

    let duration = start.elapsed();
    debug!("Read {} bytes in {:?}", content.len(), duration);
    
    // Log successful read
    log_read_success(&resolved_path, content.len(), &ctx.operation_id);

    Ok(ReadFileResult {
        content,
        path: resolved_path,
        size: file_size as usize,
        truncated,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "read_file".to_string(),
        },
    })
}

/// Internal function to read file content.
fn read_file_content(
    path: &Path,
    options: &ReadFileOptions,
    max_size: usize,
    _operation_id: &str,
) -> Result<(String, bool), ReadFileError> {
    let file = File::open(path).map_err(|e| error::io_error_with_path(e, path.to_path_buf()))?;

    // If reading specific lines
    if let Some(start_line) = options.start_line {
        let end_line = options.end_line.unwrap_or(usize::MAX);
        return read_lines(file, start_line, end_line, max_size, path, _operation_id);
    }

    // Read entire file
    let reader = BufReader::new(file);
    let mut buffer = Vec::with_capacity(max_size.min(1024 * 1024));
    let mut truncated = false;

    let bytes_read = reader.take(max_size as u64 + 1).read_to_end(&mut buffer)
        .map_err(|e| error::io_error_with_path(e, path.to_path_buf()))?;

    if bytes_read > max_size {
        buffer.truncate(max_size);
        truncated = true;
    }

    // Check for binary content
    if is_binary(&buffer) {
        warn!("File appears to be binary: {:?}", path);
        return Err(ReadFileError::BinaryFile {
            path: path.to_path_buf(),
            mime_type: None, // Could add mime detection here
        });
    }

    // Convert to string with lossy UTF-8
    let content = String::from_utf8_lossy(&buffer);
    
    // Check for encoding errors by comparing lengths
    if content.len() != buffer.len() && !content.contains('\u{FFFD}') {
        // Length difference without replacement characters suggests encoding issues
        return Err(ReadFileError::EncodingError {
            path: path.to_path_buf(),
            position: None,
        });
    }

    Ok((content.into_owned(), truncated))
}

/// Read specific lines from a file.
fn read_lines(
    file: File,
    start_line: usize,
    end_line: usize,
    max_size: usize,
    path: &Path,
    _operation_id: &str,
) -> Result<(String, bool), ReadFileError> {
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut total_size = 0;
    let mut truncated = false;
    let mut total_lines = 0;

    for (idx, line_result) in reader.lines().enumerate() {
        total_lines = idx + 1; // Track total lines seen
        let line_num = idx + 1; // 1-indexed

        if line_num < start_line {
            continue;
        }

        if line_num > end_line {
            break;
        }

        let line = line_result.map_err(|e| error::io_error_with_path(e, path.to_path_buf()))?;
        total_size += line.len() + 1; // +1 for newline

        if total_size > max_size {
            truncated = true;
            break;
        }

        lines.push(format!("{:>6}\t{}", line_num, line));
    }

    // Check if the line range was invalid after reading
    if lines.is_empty() && start_line > total_lines {
        return Err(ReadFileError::InvalidLineRange {
            start: start_line,
            end: end_line.min(total_lines), // Cap end at actual line count
            total_lines,
        });
    }

    Ok((lines.join("\n"), truncated))
}

/// Check if content appears to be binary.
fn is_binary(content: &[u8]) -> bool {
    // Check first 8KB for null bytes (common binary indicator)
    let check_len = content.len().min(8192);
    content[..check_len].contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs::write;

    #[tokio::test]
    async fn test_read_file_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello, World!").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = read_file(&ctx, "test.txt", None).await.unwrap();

        assert_eq!(result.content, "Hello, World!");
        assert_eq!(result.size, 13);
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn test_read_file_lines() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "line1\nline2\nline3\nline4\nline5").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = ReadFileOptions::new().lines(2, 4);
        let result = read_file(&ctx, "test.txt", Some(opts)).await.unwrap();

        assert!(result.content.contains("line2"));
        assert!(result.content.contains("line3"));
        assert!(result.content.contains("line4"));
        assert!(!result.content.contains("line1"));
        assert!(!result.content.contains("line5"));
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = read_file(&ctx, "nonexistent.txt", None).await;

        assert!(matches!(result, Err(crate::error::PrimitiveError::ReadFile(ReadFileError::NotFound { .. }))));
    }

    #[tokio::test]
    async fn test_read_file_truncation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.txt");
        let large_content = "x".repeat(200);
        write(&file_path, &large_content).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = ReadFileOptions::new().max_size(100);
        let result = read_file(&ctx, "large.txt", Some(opts)).await.unwrap();

        assert!(result.truncated);
        assert_eq!(result.content.len(), 100);
        assert_eq!(result.size, 200); // Original file size
    }

    #[tokio::test]
    async fn test_read_file_max_size() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.txt");
        let large_content = "x".repeat(1000);
        write(&file_path, &large_content).unwrap();

        let mut ctx = PrimitiveContext::new(dir.path().to_path_buf());
        ctx.config.max_file_size = 100;
        
        // Should fail with FileTooLarge when reading entire file
        let result = read_file(&ctx, "large.txt", None).await;
        assert!(matches!(result, Err(crate::error::PrimitiveError::ReadFile(ReadFileError::TooLarge { .. }))));
        
        // But should work with custom max_size option that is larger
        let opts = ReadFileOptions::new().max_size(1500);
        let result = read_file(&ctx, "large.txt", Some(opts)).await.unwrap();
        assert!(!result.truncated);
        assert_eq!(result.content.len(), 1000);
    }

    #[tokio::test]
    async fn test_read_file_binary_detection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("binary.bin");
        // Create a binary file with null bytes
        write(&file_path, &[0u8, 1u8, 2u8, 0u8, 255u8]).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = read_file(&ctx, "binary.bin", None).await;

        // Should now error with BinaryFile error instead of returning "[Binary file]"
        assert!(matches!(result, Err(crate::error::PrimitiveError::ReadFile(ReadFileError::BinaryFile { .. }))));
    }

    #[tokio::test]
    async fn test_read_file_utf8_lossy() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid_utf8.txt");
        // Create file with invalid UTF-8
        let invalid_utf8 = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xFF, 0x21]; // "Hello" + invalid byte + "!"
        std::fs::write(&file_path, invalid_utf8).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = read_file(&ctx, "invalid_utf8.txt", None).await.unwrap();

        assert!(result.content.starts_with("Hello"));
        assert!(result.content.contains("�")); // Replacement character for invalid UTF-8
    }

    #[tokio::test]
    async fn test_read_file_path_not_allowed() {
        let ctx = PrimitiveContext::new(PathBuf::from("/"));
        let result = read_file(&ctx, "/etc/passwd", None).await;

        assert!(matches!(result, Err(crate::error::PrimitiveError::ReadFile(ReadFileError::PathNotAllowed { .. }))));
    }

    #[tokio::test]
    async fn test_read_file_metadata() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "Hello").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = read_file(&ctx, "test.txt", None).await.unwrap();

        assert_eq!(result.metadata.primitive, "read_file");
        assert!(!result.metadata.operation_id.is_empty());
        assert!(result.metadata.duration.as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_read_file_error_suggestions() {
        let dir = tempdir().unwrap();
        
        // Create similar files
        write(dir.path().join("config.yaml"), "test").unwrap();
        write(dir.path().join("config.yml"), "test").unwrap();
        
        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = read_file(&ctx, "config.yaml", None).await; // Typo: missing 'l'
        
        // Should succeed since file exists
        assert!(result.is_ok());
        
        // Try with actual typo
        let result = read_file(&ctx, "confg.yaml", None).await;
        assert!(result.is_err());
        
        // Extract error and check suggestion
        if let Err(crate::error::PrimitiveError::ReadFile(ReadFileError::NotFound { suggestion, .. })) = result {
            assert!(suggestion.is_some());
        } else {
            panic!("Expected NotFound error with suggestion");
        }
    }

    #[tokio::test]
    async fn test_read_file_invalid_line_range() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        write(&file_path, "line1\nline2\nline3").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        
        // Test invalid range - start line too high
        let opts = ReadFileOptions::new().lines(10, 15);
        let result = read_file(&ctx, "test.txt", Some(opts)).await;
        
        assert!(matches!(
            result, 
            Err(crate::error::PrimitiveError::ReadFile(ReadFileError::InvalidLineRange { .. }))
        ));
        
        // Test invalid range - start > end
        let opts = ReadFileOptions::new().lines(5, 3);
        let result = read_file(&ctx, "test.txt", Some(opts)).await;
        
        assert!(matches!(
            result, 
            Err(crate::error::PrimitiveError::ReadFile(ReadFileError::InvalidLineRange { .. }))
        ));
    }

    #[tokio::test]
    async fn test_read_file_not_a_file() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = read_file(&ctx, "subdir", None).await;

        assert!(matches!(
            result, 
            Err(crate::error::PrimitiveError::ReadFile(ReadFileError::NotAFile { .. }))
        ));
    }

    #[tokio::test]
    async fn test_read_file_error_serialization() {
        let error = ReadFileError::NotFound {
            path: PathBuf::from("test.txt"),
            suggestion: Some("test.md".to_string()),
        };
        
        let response: ReadFileErrorResponse = (&error).into();
        
        assert_eq!(response.code, "READ_FILE_NOT_FOUND");
        assert!(response.message.contains("test.txt"));
        assert!(response.suggestion.contains("test.md"));
        assert!(!response.retryable);
        
        // Test serialization
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: ReadFileErrorResponse = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(response.code, deserialized.code);
        assert_eq!(response.message, deserialized.message);
        assert_eq!(response.suggestion, deserialized.suggestion);
        assert_eq!(response.retryable, deserialized.retryable);
    }

    #[tokio::test] 
    async fn test_read_file_recovery_suggestions() {
        // Test all error types have meaningful recovery suggestions
        let errors = vec![
            ReadFileError::NotFound {
                path: PathBuf::from("test.txt"),
                suggestion: Some("test.md".to_string()),
            },
            ReadFileError::PermissionDenied {
                path: PathBuf::from("test.txt"),
                required: "read".to_string(),
            },
            ReadFileError::TooLarge {
                path: PathBuf::from("test.txt"),
                actual_size: 1000,
                max_size: 100,
            },
            ReadFileError::BinaryFile {
                path: PathBuf::from("test.bin"),
                mime_type: Some("application/octet-stream".to_string()),
            },
        ];
        
        for error in errors {
            let suggestion = error.recovery_suggestion();
            assert!(!suggestion.is_empty());
            assert!(suggestion.len() > 10); // Should be meaningful
        }
    }
}