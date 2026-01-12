//! Bash command execution primitive.

mod cancel;
mod error;
mod options;
mod output;
mod sanitize;
mod timeout;

pub use cancel::*;
pub use error::*;
pub use options::*;
pub use output::*;
pub use sanitize::*;
pub use timeout::*;

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{BashResult, ExecutionMetadata},
};
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use tracing::{debug, instrument};

/// Execute a bash command.
///
/// # Arguments
///
/// * `ctx` - Execution context
/// * `command` - Command to execute
/// * `options` - Optional configuration
///
/// # Returns
///
/// Result containing command output and exit code.
///
/// # Example
///
/// ```no_run
/// use tachikoma_primitives::{PrimitiveContext, bash, BashOptions};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ctx = PrimitiveContext::new(PathBuf::from("."));
///
/// // Simple command
/// let result = bash(&ctx, "ls -la", None).await?;
/// println!("Output: {}", result.stdout);
///
/// // With options
/// let opts = BashOptions::new()
///     .working_dir("/tmp")
///     .env("MY_VAR", "value");
/// let result = bash(&ctx, "echo $MY_VAR", Some(opts)).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(ctx, options), fields(command = %command, op_id = %ctx.operation_id))]
pub async fn bash(
    ctx: &PrimitiveContext,
    command: &str,
    options: Option<BashOptions>,
) -> PrimitiveResult<BashResult> {
    let start = Instant::now();
    let options = options.unwrap_or_default();

    // Use timeout if specified
    if let Some(timeout_duration) = options.timeout {
        return bash_with_timeout_and_config(
            ctx, 
            command, 
            timeout_duration, 
            options.working_dir.as_deref(),
            &options.output_config
        ).await;
    }

    // Validate command
    let validator = CommandValidator::new(&options.blocked_commands);
    validator.validate(command)?;

    debug!("Executing command: {}", command);

    // Determine working directory
    let working_dir = options
        .working_dir
        .as_ref()
        .map(|p| ctx.resolve_path(p))
        .unwrap_or_else(|| ctx.working_dir.clone());

    // Check working directory is allowed
    if !ctx.is_path_allowed(&working_dir) {
        return Err(PrimitiveError::PathNotAllowed { path: working_dir });
    }

    // Build command
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(command)
        .current_dir(&working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    // Set environment variables
    if options.clear_env {
        cmd.env_clear();
    }

    for (key, value) in &options.env_vars {
        cmd.env(key, value);
    }

    // Spawn process
    let mut child = cmd.spawn().map_err(PrimitiveError::Io)?;

    // Capture output using the advanced output module
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let captured = capture_output(stdout, stderr, &options.output_config).await;

    // Wait for completion
    let status = child.wait().await.map_err(PrimitiveError::Io)?;

    let exit_code = status.code().unwrap_or(-1);
    let duration = start.elapsed();

    debug!(
        "Command completed with exit code {} in {:?}",
        exit_code, duration
    );

    Ok(BashResult {
        exit_code,
        stdout: captured.stdout,
        stderr: captured.stderr,
        timed_out: false,
        stdout_truncated: captured.stdout_truncated,
        stderr_truncated: captured.stderr_truncated,
        stdout_total_bytes: captured.stdout_total_bytes,
        stderr_total_bytes: captured.stderr_total_bytes,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "bash".to_string(),
        },
    })
}

/// Execute a command and check for success.
pub async fn bash_success(
    ctx: &PrimitiveContext,
    command: &str,
    options: Option<BashOptions>,
) -> PrimitiveResult<BashResult> {
    let result = bash(ctx, command, options).await?;

    if result.exit_code != 0 {
        return Err(PrimitiveError::CommandFailed {
            exit_code: result.exit_code,
            message: result.stderr.clone(),
        });
    }

    Ok(result)
}

/// Execute multiple commands in sequence.
pub async fn bash_sequence(
    ctx: &PrimitiveContext,
    commands: &[&str],
    options: Option<BashOptions>,
) -> PrimitiveResult<Vec<BashResult>> {
    let mut results = Vec::new();

    for command in commands {
        let result = bash(ctx, command, options.clone()).await?;
        let failed = result.exit_code != 0;
        results.push(result);

        if failed {
            break;
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_bash_echo() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "echo 'hello world'", None).await.unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "hello world");
        assert!(result.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_bash_exit_code() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "exit 42", None).await.unwrap();

        assert_eq!(result.exit_code, 42);
    }

    #[tokio::test]
    async fn test_bash_stderr() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "echo error >&2", None).await.unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr.trim(), "error");
    }

    #[tokio::test]
    async fn test_bash_env() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().env("TEST_VAR", "test_value");
        let result = bash(&ctx, "echo $TEST_VAR", Some(opts)).await.unwrap();

        assert_eq!(result.stdout.trim(), "test_value");
    }

    #[tokio::test]
    async fn test_bash_working_dir() {
        let ctx = PrimitiveContext::new(PathBuf::from("/"));
        let opts = BashOptions::new().working_dir("/tmp");
        let result = bash(&ctx, "pwd", Some(opts)).await.unwrap();

        assert!(result.stdout.contains("/tmp") || result.stdout.contains("/private/tmp"));
    }

    #[tokio::test]
    async fn test_bash_output_truncation() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new()
            .max_stdout(10) // Very small limit
            .max_stderr(10)
            .trim_output(false); // Don't trim to test exact byte counts
        
        // Command that outputs more than 10 bytes
        let result = bash(&ctx, "echo 'This is a very long output line that should be truncated'", Some(opts)).await.unwrap();
        
        assert!(result.is_output_truncated());
        assert!(result.stdout_total_bytes > 10);
        assert_eq!(result.stdout.len(), 10);
    }

    #[tokio::test]
    async fn test_bash_separate_output_capture() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "echo 'stdout message' && echo 'stderr message' >&2", None).await.unwrap();
        
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("stdout message"));
        assert!(result.stderr.contains("stderr message"));
        assert!(!result.is_output_truncated());
    }

    #[tokio::test]
    async fn test_bash_combined_output() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "echo 'stdout' && echo 'stderr' >&2", None).await.unwrap();
        
        let combined = result.combined_output();
        assert!(combined.contains("stdout"));
        assert!(combined.contains("stderr"));
    }

    #[tokio::test]
    async fn test_bash_ansi_stripping() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().strip_ansi(true);
        let result = bash(&ctx, "echo -e '\\x1b[31mRed\\x1b[0m Text'", Some(opts)).await.unwrap();
        
        // ANSI codes should be stripped
        assert!(result.stdout.contains("Red Text"));
        assert!(!result.stdout.contains("\x1b"));
    }

    #[tokio::test]
    async fn test_bash_no_ansi_stripping() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().strip_ansi(false);
        let result = bash(&ctx, "echo -e '\\x1b[31mRed\\x1b[0m Text'", Some(opts)).await.unwrap();
        
        // ANSI codes should remain
        assert!(result.stdout.contains("\x1b"));
    }

    #[tokio::test] 
    async fn test_bash_output_trimming() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().trim_output(true);
        let result = bash(&ctx, "echo '  padded output  '", Some(opts)).await.unwrap();
        
        // Output should be trimmed
        assert_eq!(result.stdout, "padded output");
    }

    #[tokio::test]
    async fn test_bash_no_output_trimming() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().trim_output(false);
        let result = bash(&ctx, "echo '  padded output  '", Some(opts)).await.unwrap();
        
        // Output should include whitespace
        assert!(result.stdout.starts_with("  "));
        assert!(result.stdout.ends_with("  \n"));
    }

    #[tokio::test]
    async fn test_bash_binary_output_handling() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        // Create some binary data (should not panic)
        let result = bash(&ctx, "printf '\\x00\\x01\\x02\\xFF'", None).await.unwrap();
        
        assert_eq!(result.exit_code, 0);
        // Binary data should be handled without panic using lossy UTF-8 conversion
        assert!(result.stdout.len() > 0);
    }

    #[tokio::test]
    async fn test_bash_with_streaming() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().stream_output();
        let result = bash(&ctx, "echo 'line1' && echo 'line2'", Some(opts)).await.unwrap();
        
        assert_eq!(result.exit_code, 0);
        // Output should contain both lines
        assert!(result.stdout.contains("line1"));
        assert!(result.stdout.contains("line2"));
    }

    #[tokio::test]
    async fn test_output_callbacks() {
        use super::output::{CollectingCallback, OutputCallback};
        
        let callback = CollectingCallback::new();
        
        // Test callback functionality
        callback.on_stdout("test stdout line");
        callback.on_stderr("test stderr line");
        
        let stdout_lines = callback.stdout();
        let stderr_lines = callback.stderr();
        
        assert_eq!(stdout_lines.len(), 1);
        assert_eq!(stdout_lines[0], "test stdout line");
        assert_eq!(stderr_lines.len(), 1);
        assert_eq!(stderr_lines[0], "test stderr line");
    }
}