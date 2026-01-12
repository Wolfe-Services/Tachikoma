# 039 - Bash Error Handling

**Phase:** 2 - Five Primitives
**Spec ID:** 039
**Status:** Planned
**Dependencies:** 036-bash-exec-core
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive error handling for bash commands including exit code interpretation, signal handling, and helpful error messages.

---

## Acceptance Criteria

- [ ] Exit code to error mapping
- [ ] Signal name interpretation (SIGTERM, SIGKILL, etc.)
- [ ] Common error pattern detection
- [ ] Helpful error messages with suggestions
- [ ] Error categorization for retry logic
- [ ] Serializable error responses

---

## Implementation Details

### 1. Bash Errors Module (src/bash/error.rs)

```rust
//! Error types for bash command execution.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Errors from bash command execution.
#[derive(Debug, Error)]
pub enum BashError {
    /// Command not found.
    #[error("command not found: {command}")]
    CommandNotFound { command: String },

    /// Permission denied.
    #[error("permission denied: {message}")]
    PermissionDenied { message: String },

    /// Command failed with exit code.
    #[error("command failed with exit code {exit_code}: {message}")]
    ExitCode {
        exit_code: i32,
        message: String,
        stderr: String,
    },

    /// Command terminated by signal.
    #[error("command killed by signal: {signal_name} ({signal_num})")]
    Signal {
        signal_num: i32,
        signal_name: String,
    },

    /// Command timed out.
    #[error("command timed out after {duration:?}")]
    Timeout {
        duration: Duration,
        partial_stdout: String,
        partial_stderr: String,
    },

    /// Working directory invalid.
    #[error("invalid working directory: {path}")]
    InvalidWorkingDir { path: String },

    /// Command blocked by security policy.
    #[error("command blocked: {reason}")]
    Blocked { reason: String },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Process spawn failed.
    #[error("failed to spawn process: {message}")]
    SpawnFailed { message: String },
}

impl BashError {
    /// Get the exit code if available.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::ExitCode { exit_code, .. } => Some(*exit_code),
            Self::Signal { signal_num, .. } => Some(128 + signal_num),
            _ => None,
        }
    }

    /// Get a recovery suggestion.
    pub fn recovery_suggestion(&self) -> String {
        match self {
            Self::CommandNotFound { command } => {
                format!(
                    "Check if '{}' is installed and in PATH. Try 'which {}' or 'command -v {}'.",
                    command, command, command
                )
            }
            Self::PermissionDenied { .. } => {
                "Check file permissions or try with appropriate privileges.".to_string()
            }
            Self::ExitCode { exit_code, .. } => {
                match exit_code {
                    1 => "General error. Check stderr for details.".to_string(),
                    2 => "Misuse of shell command or invalid arguments.".to_string(),
                    126 => "Command found but not executable. Check permissions.".to_string(),
                    127 => "Command not found. Check spelling and PATH.".to_string(),
                    128 => "Invalid exit argument.".to_string(),
                    _ if *exit_code > 128 => {
                        format!("Process killed by signal {}.", exit_code - 128)
                    }
                    _ => "Check stderr output for error details.".to_string(),
                }
            }
            Self::Signal { signal_name, .. } => {
                match signal_name.as_str() {
                    "SIGKILL" => "Process was forcefully killed. May need more resources.".to_string(),
                    "SIGTERM" => "Process was terminated. May have been cancelled.".to_string(),
                    "SIGSEGV" => "Segmentation fault. Command has a bug.".to_string(),
                    "SIGABRT" => "Process aborted. Check for assertion failures.".to_string(),
                    _ => format!("Process received {} signal.", signal_name),
                }
            }
            Self::Timeout { duration, .. } => {
                format!(
                    "Increase timeout (was {:?}) or check if command is hanging.",
                    duration
                )
            }
            Self::InvalidWorkingDir { path } => {
                format!("Verify '{}' exists and is a directory.", path)
            }
            Self::Blocked { .. } => {
                "This command is blocked for security reasons.".to_string()
            }
            Self::Io(_) => "Check system resources and permissions.".to_string(),
            Self::SpawnFailed { .. } => "Check if bash is available at /bin/bash.".to_string(),
        }
    }

    /// Get error code for categorization.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::CommandNotFound { .. } => "BASH_CMD_NOT_FOUND",
            Self::PermissionDenied { .. } => "BASH_PERMISSION_DENIED",
            Self::ExitCode { .. } => "BASH_EXIT_CODE",
            Self::Signal { .. } => "BASH_SIGNAL",
            Self::Timeout { .. } => "BASH_TIMEOUT",
            Self::InvalidWorkingDir { .. } => "BASH_INVALID_DIR",
            Self::Blocked { .. } => "BASH_BLOCKED",
            Self::Io(_) => "BASH_IO_ERROR",
            Self::SpawnFailed { .. } => "BASH_SPAWN_FAILED",
        }
    }

    /// Check if error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } => true,
            Self::Io(_) => true,
            Self::SpawnFailed { .. } => true,
            _ => false,
        }
    }
}

/// Convert exit code to signal name.
pub fn signal_name(code: i32) -> Option<&'static str> {
    // Standard signal numbers (may vary by OS)
    match code {
        1 => Some("SIGHUP"),
        2 => Some("SIGINT"),
        3 => Some("SIGQUIT"),
        6 => Some("SIGABRT"),
        9 => Some("SIGKILL"),
        11 => Some("SIGSEGV"),
        13 => Some("SIGPIPE"),
        14 => Some("SIGALRM"),
        15 => Some("SIGTERM"),
        _ => None,
    }
}

/// Analyze stderr for common error patterns.
pub fn analyze_stderr(stderr: &str) -> Option<BashError> {
    let stderr_lower = stderr.to_lowercase();

    // Command not found patterns
    if stderr_lower.contains("command not found")
        || stderr_lower.contains("not found")
        || stderr_lower.contains("no such file or directory")
    {
        // Try to extract command name
        if let Some(cmd) = extract_command_from_error(stderr) {
            return Some(BashError::CommandNotFound { command: cmd });
        }
    }

    // Permission denied
    if stderr_lower.contains("permission denied") {
        return Some(BashError::PermissionDenied {
            message: stderr.lines().next().unwrap_or("").to_string(),
        });
    }

    None
}

/// Try to extract command name from error message.
fn extract_command_from_error(stderr: &str) -> Option<String> {
    // Pattern: "bash: command: not found"
    if let Some(line) = stderr.lines().next() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 {
            let cmd = parts[1].trim();
            if !cmd.is_empty() {
                return Some(cmd.to_string());
            }
        }
    }
    None
}

/// Create appropriate error from exit code and stderr.
pub fn error_from_exit(exit_code: i32, stderr: &str) -> BashError {
    // Check for signal (exit codes > 128)
    if exit_code > 128 {
        let sig_num = exit_code - 128;
        return BashError::Signal {
            signal_num: sig_num,
            signal_name: signal_name(sig_num).unwrap_or("UNKNOWN").to_string(),
        };
    }

    // Try to analyze stderr
    if let Some(err) = analyze_stderr(stderr) {
        return err;
    }

    // Generic exit code error
    BashError::ExitCode {
        exit_code,
        message: format!("Command exited with code {}", exit_code),
        stderr: stderr.to_string(),
    }
}

/// Serializable error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashErrorResponse {
    pub code: String,
    pub message: String,
    pub exit_code: Option<i32>,
    pub stderr: Option<String>,
    pub suggestion: String,
    pub retryable: bool,
}

impl From<&BashError> for BashErrorResponse {
    fn from(err: &BashError) -> Self {
        let stderr = match err {
            BashError::ExitCode { stderr, .. } => Some(stderr.clone()),
            BashError::Timeout { partial_stderr, .. } => Some(partial_stderr.clone()),
            _ => None,
        };

        Self {
            code: err.error_code().to_string(),
            message: err.to_string(),
            exit_code: err.exit_code(),
            stderr,
            suggestion: err.recovery_suggestion(),
            retryable: err.is_retryable(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_name() {
        assert_eq!(signal_name(9), Some("SIGKILL"));
        assert_eq!(signal_name(15), Some("SIGTERM"));
        assert_eq!(signal_name(99), None);
    }

    #[test]
    fn test_analyze_stderr_command_not_found() {
        let stderr = "bash: foo: command not found";
        let err = analyze_stderr(stderr).unwrap();
        assert!(matches!(err, BashError::CommandNotFound { command } if command == "foo"));
    }

    #[test]
    fn test_analyze_stderr_permission() {
        let stderr = "bash: /etc/passwd: Permission denied";
        let err = analyze_stderr(stderr).unwrap();
        assert!(matches!(err, BashError::PermissionDenied { .. }));
    }

    #[test]
    fn test_error_from_exit_signal() {
        let err = error_from_exit(137, ""); // 128 + 9 = SIGKILL
        assert!(matches!(
            err,
            BashError::Signal { signal_num: 9, signal_name } if signal_name == "SIGKILL"
        ));
    }

    #[test]
    fn test_recovery_suggestions() {
        let err = BashError::CommandNotFound {
            command: "foo".to_string(),
        };
        let suggestion = err.recovery_suggestion();
        assert!(suggestion.contains("which"));
    }
}
```

---

## Testing Requirements

1. Exit codes map to correct errors
2. Signal numbers convert to names correctly
3. Stderr analysis detects common patterns
4. Error suggestions are helpful
5. Error responses serialize correctly
6. Retryable errors are identified correctly

---

## Related Specs

- Depends on: [036-bash-exec-core.md](036-bash-exec-core.md)
- Next: [040-edit-file-core.md](040-edit-file-core.md)
- Related: [037-bash-timeout.md](037-bash-timeout.md)
