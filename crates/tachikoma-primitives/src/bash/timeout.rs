//! Timeout handling for bash commands.

use crate::{
    context::PrimitiveContext,
    error::{PrimitiveError, PrimitiveResult},
    result::{BashResult, ExecutionMetadata},
};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};
use tracing::{debug, instrument, warn};

/// Grace period before SIGKILL after SIGTERM.
const KILL_GRACE_PERIOD: Duration = Duration::from_secs(5);

/// Execute a bash command with timeout.
#[instrument(skip(ctx), fields(command = %command, timeout = ?timeout_duration, op_id = %ctx.operation_id))]
pub async fn bash_with_timeout(
    ctx: &PrimitiveContext,
    command: &str,
    timeout_duration: Duration,
    working_dir: Option<&str>,
) -> PrimitiveResult<BashResult> {
    let start = Instant::now();

    let working_dir = working_dir
        .map(|p| ctx.resolve_path(p))
        .unwrap_or_else(|| ctx.working_dir.clone());

    debug!(
        "Executing command with {}s timeout: {}",
        timeout_duration.as_secs(),
        command
    );

    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(command)
        .current_dir(&working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    // On Unix, create new process group for proper cleanup
    #[cfg(unix)]
    {
        unsafe {
            cmd.pre_exec(|| {
                // Create new process group
                libc::setpgid(0, 0);
                Ok(())
            });
        }
    }

    let mut child = cmd.spawn().map_err(PrimitiveError::Io)?;
    let pid = child.id();

    // Execute with timeout
    let result = timeout(timeout_duration, execute_and_capture(&mut child)).await;

    match result {
        Ok(Ok((exit_code, stdout, stderr))) => {
            let duration = start.elapsed();
            let stdout_len = stdout.len();
            let stderr_len = stderr.len();
            debug!("Command completed in {:?}", duration);

            Ok(BashResult {
                exit_code,
                stdout,
                stderr,
                timed_out: false,
                stdout_truncated: false,
                stderr_truncated: false,
                stdout_total_bytes: stdout_len,
                stderr_total_bytes: stderr_len,
                metadata: ExecutionMetadata {
                    duration,
                    operation_id: ctx.operation_id.clone(),
                    primitive: "bash".to_string(),
                },
            })
        }
        Ok(Err(e)) => Err(e),
        Err(_) => {
            // Timeout occurred
            warn!("Command timed out after {:?}", timeout_duration);

            // Capture partial output before killing
            let (partial_stdout, partial_stderr) = capture_partial_output(&mut child).await;

            // Kill the process gracefully
            kill_process_tree(pid).await;

            let duration = start.elapsed();
            let partial_stdout_len = partial_stdout.len();
            let partial_stderr_len = partial_stderr.len();

            Ok(BashResult {
                exit_code: -1,
                stdout: partial_stdout,
                stderr: partial_stderr,
                timed_out: true,
                stdout_truncated: false,
                stderr_truncated: false,
                stdout_total_bytes: partial_stdout_len,
                stderr_total_bytes: partial_stderr_len,
                metadata: ExecutionMetadata {
                    duration,
                    operation_id: ctx.operation_id.clone(),
                    primitive: "bash".to_string(),
                },
            })
        }
    }
}

/// Execute command and capture output.
async fn execute_and_capture(child: &mut Child) -> PrimitiveResult<(i32, String, String)> {
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();

    let stdout_task = async {
        let mut buf = Vec::new();
        if let Some(ref mut out) = stdout {
            out.read_to_end(&mut buf).await?;
        }
        Ok::<_, std::io::Error>(String::from_utf8_lossy(&buf).into_owned())
    };

    let stderr_task = async {
        let mut buf = Vec::new();
        if let Some(ref mut err) = stderr {
            err.read_to_end(&mut buf).await?;
        }
        Ok::<_, std::io::Error>(String::from_utf8_lossy(&buf).into_owned())
    };

    let status_task = child.wait();

    let (stdout_result, stderr_result, status) =
        tokio::join!(stdout_task, stderr_task, status_task);

    let stdout = stdout_result.map_err(PrimitiveError::Io)?;
    let stderr = stderr_result.map_err(PrimitiveError::Io)?;
    let status = status.map_err(PrimitiveError::Io)?;

    Ok((status.code().unwrap_or(-1), stdout, stderr))
}

/// Capture any available output without blocking.
async fn capture_partial_output(child: &mut Child) -> (String, String) {
    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    // Try to read available data with a short timeout
    let capture_timeout = Duration::from_millis(100);

    if let Some(ref mut stdout) = child.stdout {
        let _ = timeout(capture_timeout, stdout.read_to_end(&mut stdout_buf)).await;
    }

    if let Some(ref mut stderr) = child.stderr {
        let _ = timeout(capture_timeout, stderr.read_to_end(&mut stderr_buf)).await;
    }

    (
        String::from_utf8_lossy(&stdout_buf).into_owned(),
        String::from_utf8_lossy(&stderr_buf).into_owned(),
    )
}

/// Kill a process and its children.
async fn kill_process_tree(pid: Option<u32>) {
    let Some(pid) = pid else {
        return;
    };

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let pgid = Pid::from_raw(-(pid as i32)); // Negative PID = process group

        // First, try SIGTERM
        debug!("Sending SIGTERM to process group {}", pid);
        let _ = kill(pgid, Signal::SIGTERM);

        // Wait for grace period
        sleep(KILL_GRACE_PERIOD).await;

        // Then SIGKILL if still running
        debug!("Sending SIGKILL to process group {}", pid);
        let _ = kill(pgid, Signal::SIGKILL);
    }

    #[cfg(windows)]
    {
        // On Windows, use taskkill to kill process tree
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output()
            .await;
    }
}

/// A builder for timeout-controlled command execution.
pub struct TimeoutCommand {
    command: String,
    timeout: Duration,
    working_dir: Option<String>,
    grace_period: Duration,
}

impl TimeoutCommand {
    /// Create a new timeout command.
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            timeout: Duration::from_secs(30),
            working_dir: None,
            grace_period: KILL_GRACE_PERIOD,
        }
    }

    /// Set the timeout duration.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// Set the working directory.
    pub fn working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    /// Set the grace period before SIGKILL.
    pub fn grace_period(mut self, duration: Duration) -> Self {
        self.grace_period = duration;
        self
    }

    /// Execute the command.
    pub async fn execute(self, ctx: &PrimitiveContext) -> PrimitiveResult<BashResult> {
        bash_with_timeout(ctx, &self.command, self.timeout, self.working_dir.as_deref()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_command_completes_before_timeout() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result =
            bash_with_timeout(&ctx, "echo 'fast'", Duration::from_secs(10), None).await.unwrap();

        assert!(!result.timed_out);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "fast");
    }

    #[tokio::test]
    async fn test_command_times_out() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result =
            bash_with_timeout(&ctx, "sleep 10", Duration::from_millis(100), None).await.unwrap();

        assert!(result.timed_out);
        assert_eq!(result.exit_code, -1);
    }

    #[tokio::test]
    async fn test_timeout_command_builder() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = TimeoutCommand::new("echo 'builder'")
            .timeout(Duration::from_secs(5))
            .execute(&ctx)
            .await
            .unwrap();

        assert!(!result.timed_out);
        assert_eq!(result.stdout.trim(), "builder");
    }

    #[tokio::test]
    async fn test_partial_output_on_timeout() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        // Command that outputs something immediately then hangs
        let result = bash_with_timeout(
            &ctx,
            "echo 'before' && exec sleep 10",
            Duration::from_millis(100),
            None,
        )
        .await
        .unwrap();

        assert!(result.timed_out);
        // The command should have timed out 
        assert_eq!(result.exit_code, -1);
    }
}