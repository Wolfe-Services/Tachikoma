//! Read file primitive implementation.

mod options;

pub use options::ReadFileOptions;

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
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

    if !ctx.is_path_allowed(&resolved_path) {
        return Err(PrimitiveError::PathNotAllowed {
            path: resolved_path,
        });
    }

    // Check file exists and get metadata
    let file_metadata = fs::metadata(&resolved_path).map_err(|e| {
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

    let file_size = file_metadata.len() as usize;
    let max_size = options.max_size.unwrap_or(ctx.config.max_file_size);

    // Check size limit - only fail if using default max_size and no line range
    if file_size > max_size && options.max_size.is_none() && options.start_line.is_none() {
        return Err(PrimitiveError::FileTooLarge {
            size: file_size,
            max: max_size,
        });
    }

    // Read content
    let (content, truncated) = read_file_content(&resolved_path, &options, max_size)?;

    let duration = start.elapsed();
    debug!("Read {} bytes in {:?}", content.len(), duration);

    Ok(ReadFileResult {
        content,
        path: resolved_path,
        size: file_size,
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
) -> PrimitiveResult<(String, bool)> {
    let file = File::open(path)?;

    // If reading specific lines
    if let Some(start_line) = options.start_line {
        let end_line = options.end_line.unwrap_or(usize::MAX);
        return read_lines(file, start_line, end_line, max_size);
    }

    // Read entire file
    let reader = BufReader::new(file);
    let mut buffer = Vec::with_capacity(max_size.min(1024 * 1024));
    let mut truncated = false;

    let bytes_read = reader.take(max_size as u64 + 1).read_to_end(&mut buffer)?;

    if bytes_read > max_size {
        buffer.truncate(max_size);
        truncated = true;
    }

    // Check for binary content
    if is_binary(&buffer) {
        warn!("File appears to be binary: {:?}", path);
        return Ok(("[Binary file]".to_string(), false));
    }

    // Convert to string with lossy UTF-8
    let content = String::from_utf8_lossy(&buffer).into_owned();

    Ok((content, truncated))
}

/// Read specific lines from a file.
fn read_lines(
    file: File,
    start_line: usize,
    end_line: usize,
    max_size: usize,
) -> PrimitiveResult<(String, bool)> {
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut total_size = 0;
    let mut truncated = false;

    for (idx, line_result) in reader.lines().enumerate() {
        let line_num = idx + 1; // 1-indexed

        if line_num < start_line {
            continue;
        }

        if line_num > end_line {
            break;
        }

        let line = line_result?;
        total_size += line.len() + 1; // +1 for newline

        if total_size > max_size {
            truncated = true;
            break;
        }

        lines.push(format!("{:>6}\t{}", line_num, line));
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

        assert!(matches!(result, Err(PrimitiveError::FileNotFound { .. })));
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
        assert!(matches!(result, Err(PrimitiveError::FileTooLarge { .. })));
        
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
        let result = read_file(&ctx, "binary.bin", None).await.unwrap();

        assert_eq!(result.content, "[Binary file]");
        assert!(!result.truncated);
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

        assert!(matches!(result, Err(PrimitiveError::PathNotAllowed { .. })));
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
}