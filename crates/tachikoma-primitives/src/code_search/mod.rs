//! Code search primitive using ripgrep.

mod options;
mod parser;

pub use options::CodeSearchOptions;
pub use parser::{RipgrepMatch, RipgrepOutput};

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{CodeSearchResult, ExecutionMetadata, SearchMatch},
};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{debug, instrument, warn};

/// Maximum matches to return by default.
const DEFAULT_MAX_MATCHES: usize = 100;

/// Execute a code search using ripgrep.
///
/// # Arguments
///
/// * `ctx` - Execution context
/// * `pattern` - Regex pattern to search for
/// * `path` - Directory or file to search
/// * `options` - Optional search configuration
///
/// # Returns
///
/// Result containing search matches.
///
/// # Example
///
/// ```no_run
/// use tachikoma_primitives::{PrimitiveContext, code_search, CodeSearchOptions};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ctx = PrimitiveContext::new(PathBuf::from("."));
///
/// // Simple search
/// let result = code_search(&ctx, "fn main", "src", None).await?;
///
/// // With options
/// let opts = CodeSearchOptions::new()
///     .file_type("rust")
///     .context(2);
/// let result = code_search(&ctx, "TODO", ".", Some(opts)).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(ctx, options), fields(pattern = %pattern, path = %path, op_id = %ctx.operation_id))]
pub async fn code_search(
    ctx: &PrimitiveContext,
    pattern: &str,
    path: &str,
    options: Option<CodeSearchOptions>,
) -> PrimitiveResult<CodeSearchResult> {
    let start = Instant::now();
    let options = options.unwrap_or_default();

    // Validate pattern
    if pattern.is_empty() {
        return Err(PrimitiveError::Validation {
            message: "Search pattern cannot be empty".to_string(),
        });
    }

    // Resolve path
    let resolved_path = ctx.resolve_path(path);
    debug!("Searching in: {:?} for pattern: {}", resolved_path, pattern);

    if !ctx.is_path_allowed(&resolved_path) {
        return Err(PrimitiveError::PathNotAllowed {
            path: resolved_path,
        });
    }

    // Build ripgrep command
    let mut cmd = Command::new("rg");

    // Base options
    cmd.arg("--json") // JSON output for parsing
        .arg("--line-number")
        .arg("--column");

    // Context lines
    if options.context_before > 0 {
        cmd.arg("-B").arg(options.context_before.to_string());
    }
    if options.context_after > 0 {
        cmd.arg("-A").arg(options.context_after.to_string());
    }

    // File type filter
    if let Some(ref file_type) = options.file_type {
        cmd.arg("--type").arg(file_type);
    }

    // Glob patterns
    for glob in &options.globs {
        cmd.arg("--glob").arg(glob);
    }

    // Case sensitivity
    if options.case_insensitive {
        cmd.arg("--ignore-case");
    } else if options.smart_case {
        cmd.arg("--smart-case");
    }

    // Respect gitignore
    if !options.no_ignore {
        cmd.arg("--ignore");
    } else {
        cmd.arg("--no-ignore");
    }

    // Hidden files
    if options.include_hidden {
        cmd.arg("--hidden");
    }

    // Max count
    let max_matches = options.max_matches.unwrap_or(DEFAULT_MAX_MATCHES);
    cmd.arg("--max-count").arg(max_matches.to_string());

    // Pattern and path
    cmd.arg("--").arg(pattern).arg(&resolved_path);

    // Execute
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PrimitiveError::Validation {
                message: "ripgrep (rg) not found. Please install ripgrep.".to_string(),
            }
        } else {
            PrimitiveError::Io(e)
        }
    })?;

    // Read output
    let mut stdout = String::new();
    let mut stderr = String::new();

    if let Some(ref mut out) = child.stdout {
        out.read_to_string(&mut stdout).await?;
    }
    if let Some(ref mut err) = child.stderr {
        err.read_to_string(&mut stderr).await?;
    }

    let status = child.wait().await?;

    // Parse results
    let (matches, total_count, truncated) = parser::parse_ripgrep_output(&stdout, max_matches)?;

    // Check for errors
    if !status.success() && status.code() != Some(1) {
        // Exit code 1 means no matches, which is fine
        if !stderr.is_empty() {
            warn!("ripgrep stderr: {}", stderr);
        }
        if let Some(code) = status.code() {
            if code != 1 {
                return Err(PrimitiveError::Validation {
                    message: format!("ripgrep failed: {}", stderr),
                });
            }
        }
    }

    let duration = start.elapsed();
    debug!("Found {} matches in {:?}", matches.len(), duration);

    Ok(CodeSearchResult {
        matches,
        pattern: pattern.to_string(),
        total_count,
        truncated,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "code_search".to_string(),
        },
    })
}

/// Search for a literal string (not regex).
pub async fn search_literal(
    ctx: &PrimitiveContext,
    text: &str,
    path: &str,
    options: Option<CodeSearchOptions>,
) -> PrimitiveResult<CodeSearchResult> {
    let escaped = regex::escape(text);
    code_search(ctx, &escaped, path, options).await
}

/// Find files matching a pattern.
pub async fn find_files(
    ctx: &PrimitiveContext,
    pattern: &str,
    path: &str,
) -> PrimitiveResult<Vec<std::path::PathBuf>> {
    let resolved_path = ctx.resolve_path(path);

    let mut cmd = Command::new("rg");
    cmd.arg("--files")
        .arg("--glob")
        .arg(pattern)
        .arg(&resolved_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<_> = stdout
        .lines()
        .map(|line| std::path::PathBuf::from(line.trim()))
        .collect();

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs::write;

    #[tokio::test]
    async fn test_code_search_basic() {
        let dir = tempdir().unwrap();
        write(dir.path().join("test.rs"), "fn main() {\n    println!(\"hello\");\n}").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = code_search(&ctx, "main", ".", None).await.unwrap();

        assert_eq!(result.total_count, 1);
        assert!(!result.matches.is_empty());
    }

    #[tokio::test]
    async fn test_code_search_with_type() {
        let dir = tempdir().unwrap();
        write(dir.path().join("test.rs"), "fn test() {}").unwrap();
        write(dir.path().join("test.txt"), "fn test() {}").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = CodeSearchOptions::new().file_type("rust");
        let result = code_search(&ctx, "fn test", ".", Some(opts)).await.unwrap();

        assert_eq!(result.matches.len(), 1);
        assert!(result.matches[0].path.to_string_lossy().contains(".rs"));
    }

    #[tokio::test]
    async fn test_code_search_context() {
        let dir = tempdir().unwrap();
        write(
            dir.path().join("test.rs"),
            "line1\nline2\ntarget\nline4\nline5",
        ).unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let opts = CodeSearchOptions::new().context(1);
        let result = code_search(&ctx, "target", ".", Some(opts)).await.unwrap();

        assert!(!result.matches.is_empty());
        assert!(!result.matches[0].context_before.is_empty());
        assert!(!result.matches[0].context_after.is_empty());
    }

    #[tokio::test]
    async fn test_code_search_no_matches() {
        let dir = tempdir().unwrap();
        write(dir.path().join("test.txt"), "hello world").unwrap();

        let ctx = PrimitiveContext::new(dir.path().to_path_buf());
        let result = code_search(&ctx, "notfound", ".", None).await.unwrap();

        assert_eq!(result.total_count, 0);
        assert!(result.matches.is_empty());
    }
}