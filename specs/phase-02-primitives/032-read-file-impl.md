# 032 - Read File Implementation

**Phase:** 2 - Five Primitives
**Spec ID:** 032
**Status:** Planned
**Dependencies:** 031-primitives-crate
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement the `read_file` primitive that reads file contents with support for line ranges, size limits, and encoding detection.

---

## Acceptance Criteria

- [ ] Read entire file contents
- [ ] Support line range selection (start_line, end_line)
- [ ] Enforce maximum file size limit
- [ ] Handle binary file detection
- [ ] Return file metadata (size, truncated status)
- [ ] Proper UTF-8 handling with lossy fallback

---

## Implementation Details

### 1. Read File Module (src/read_file/mod.rs)

```rust
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

    // Check size limit
    if file_size > max_size && options.start_line.is_none() {
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
    let mut reader = BufReader::new(file);
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
}
```

### 2. Read File Options (src/read_file/options.rs)

```rust
//! Options for read_file primitive.

/// Options for reading a file.
#[derive(Debug, Clone, Default)]
pub struct ReadFileOptions {
    /// Starting line number (1-indexed).
    pub start_line: Option<usize>,
    /// Ending line number (1-indexed, inclusive).
    pub end_line: Option<usize>,
    /// Maximum size to read in bytes.
    pub max_size: Option<usize>,
    /// Include line numbers in output.
    pub show_line_numbers: bool,
}

impl ReadFileOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set line range to read.
    pub fn lines(mut self, start: usize, end: usize) -> Self {
        self.start_line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Set start line only.
    pub fn from_line(mut self, start: usize) -> Self {
        self.start_line = Some(start);
        self
    }

    /// Set maximum size.
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }

    /// Include line numbers in output.
    pub fn with_line_numbers(mut self) -> Self {
        self.show_line_numbers = true;
        self
    }
}
```

---

## Testing Requirements

1. Read entire file returns correct content
2. Line range selection works correctly
3. Large files are properly truncated
4. Binary files are detected and handled
5. File not found returns appropriate error
6. Permission denied returns appropriate error
7. UTF-8 lossy conversion handles invalid sequences

---

## Related Specs

- Depends on: [031-primitives-crate.md](031-primitives-crate.md)
- Next: [033-read-file-errors.md](033-read-file-errors.md)
- Used by: Agent loop for file reading operations
