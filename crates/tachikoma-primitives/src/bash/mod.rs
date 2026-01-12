//! Bash command execution primitive.

mod options;
mod sanitize;

pub use options::BashOptions;
pub use sanitize::CommandValidator;

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{BashResult, ExecutionMetadata},
};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{debug, instrument};

/// Maximum output size (10 MB).
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024;

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
    let child = cmd.spawn().map_err(|e| PrimitiveError::Io(e))?;

    // Execute with optional timeout
    let result = if let Some(timeout) = options.timeout {
        execute_with_timeout(child, timeout).await
    } else {
        execute_without_timeout(child).await
    };

    let (exit_code, stdout_content, stderr_content, timed_out) = result?;
    let duration = start.elapsed();

    debug!(
        "Command completed with exit code {} in {:?}{}",
        exit_code, duration, if timed_out { " (timed out)" } else { "" }
    );

    Ok(BashResult {
        exit_code,
        stdout: stdout_content,
        stderr: stderr_content,
        timed_out,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "bash".to_string(),
        },
    })
}

/// Execute command without timeout.
async fn execute_without_timeout(
    mut child: tokio::process::Child,
) -> PrimitiveResult<(i32, String, String, bool)> {
    // Read output
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (stdout_content, stderr_content) = read_output(stdout, stderr).await?;

    // Wait for completion
    let status = child.wait().await.map_err(|e| PrimitiveError::Io(e))?;
    let exit_code = status.code().unwrap_or(-1);

    Ok((exit_code, stdout_content, stderr_content, false))
}

/// Execute command with timeout.
async fn execute_with_timeout(
    mut child: tokio::process::Child,
    timeout: std::time::Duration,
) -> PrimitiveResult<(i32, String, String, bool)> {
    // Extract streams before creating futures
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let output_future = read_output(stdout, stderr);

    // Race the timeout against the command completion
    let result = tokio::select! {
        output_result = output_future => {
            match output_result {
                Ok((stdout_content, stderr_content)) => {
                    // Output read successfully, wait for process
                    match child.wait().await {
                        Ok(status) => Ok((status.code().unwrap_or(-1), stdout_content, stderr_content, false)),
                        Err(e) => Err(PrimitiveError::Io(e)),
                    }
                }
                Err(e) => Err(e),
            }
        }
        _ = tokio::time::sleep(timeout) => {
            // Kill the process
            let _ = child.kill().await;
            
            // Return timeout result
            Ok((-1, String::new(), format!("Command timed out after {:?}", timeout), true))
        }
    };

    result
}

/// Read stdout and stderr concurrently.
async fn read_output(
    stdout: Option<tokio::process::ChildStdout>,
    stderr: Option<tokio::process::ChildStderr>,
) -> PrimitiveResult<(String, String)> {
    let stdout_future = async {
        let mut content = Vec::new();
        if let Some(stdout) = stdout {
            stdout.take(MAX_OUTPUT_SIZE as u64).read_to_end(&mut content).await?;
        }
        Ok::<_, std::io::Error>(String::from_utf8_lossy(&content).into_owned())
    };

    let stderr_future = async {
        let mut content = Vec::new();
        if let Some(stderr) = stderr {
            stderr.take(MAX_OUTPUT_SIZE as u64).read_to_end(&mut content).await?;
        }
        Ok::<_, std::io::Error>(String::from_utf8_lossy(&content).into_owned())
    };

    let (stdout_result, stderr_result) = tokio::join!(stdout_future, stderr_future);

    Ok((
        stdout_result.map_err(PrimitiveError::Io)?,
        stderr_result.map_err(PrimitiveError::Io)?,
    ))
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
    async fn test_bash_success() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash_success(&ctx, "echo success", None).await.unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_bash_success_fails() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash_success(&ctx, "exit 1", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_sequence() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let commands = ["echo first", "echo second"];
        let results = bash_sequence(&ctx, &commands, None).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].stdout.trim(), "first");
        assert_eq!(results[1].stdout.trim(), "second");
    }

    #[tokio::test]
    async fn test_bash_sequence_early_exit() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let commands = ["echo first", "exit 1", "echo should not run"];
        let results = bash_sequence(&ctx, &commands, None).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].exit_code, 0);
        assert_eq!(results[1].exit_code, 1);
    }

    #[tokio::test]
    async fn test_bash_blocked_command() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "rm -rf /", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_timeout() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().timeout(std::time::Duration::from_millis(100));
        let result = bash(&ctx, "sleep 1", Some(opts)).await.unwrap();
        
        assert_eq!(result.exit_code, -1);
        assert!(result.timed_out);
    }

    #[tokio::test]
    async fn test_bash_no_timeout() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().no_timeout();
        let result = bash(&ctx, "echo no timeout", Some(opts)).await.unwrap();
        
        assert_eq!(result.exit_code, 0);
        assert!(!result.timed_out);
        assert_eq!(result.stdout.trim(), "no timeout");
    }

    #[tokio::test]
    async fn test_bash_clear_env() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new()
            .clear_env()
            .env("NEW_VAR", "new_value");
        
        // This command should fail if environment is cleared and PATH is not available
        // But our NEW_VAR should still be available
        let result = bash(&ctx, "echo $NEW_VAR", Some(opts)).await.unwrap();
        assert_eq!(result.stdout.trim(), "new_value");
    }
}