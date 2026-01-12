//! Error types for bash command execution.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Errors from bash command execution.
#[derive(Debug, Error)]
pub enum BashError {
    /// Command not found.
    #[error("command not found: {command}")]
    CommandNotFound { 
        /// The command that was not found.
        command: String 
    },

    /// Permission denied.
    #[error("permission denied: {message}")]
    PermissionDenied { 
        /// Permission error message.
        message: String 
    },

    /// Command failed with exit code.
    #[error("command failed with exit code {exit_code}: {message}")]
    ExitCode {
        /// The exit code returned by the command.
        exit_code: i32,
        /// Description of the error.
        message: String,
        /// Standard error output.
        stderr: String,
    },

    /// Command terminated by signal.
    #[error("command killed by signal: {signal_name} ({signal_num})")]
    Signal {
        /// Signal number.
        signal_num: i32,
        /// Signal name.
        signal_name: String,
    },

    /// Command timed out.
    #[error("command timed out after {duration:?}")]
    Timeout {
        /// Duration after which the command timed out.
        duration: Duration,
        /// Partial stdout captured before timeout.
        partial_stdout: String,
        /// Partial stderr captured before timeout.
        partial_stderr: String,
    },

    /// Working directory invalid.
    #[error("invalid working directory: {path}")]
    InvalidWorkingDir { 
        /// Invalid directory path.
        path: String 
    },

    /// Command blocked by security policy.
    #[error("command blocked: {reason}")]
    Blocked { 
        /// Reason for blocking the command.
        reason: String 
    },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Process spawn failed.
    #[error("failed to spawn process: {message}")]
    SpawnFailed { 
        /// Error message describing the spawn failure.
        message: String 
    },
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
    /// Error code for categorization.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Exit code if available.
    pub exit_code: Option<i32>,
    /// Standard error output if available.
    pub stderr: Option<String>,
    /// Suggested recovery action.
    pub suggestion: String,
    /// Whether the error is retryable.
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

    #[test]
    fn test_exit_code_extraction() {
        let err = BashError::ExitCode {
            exit_code: 42,
            message: "test".to_string(),
            stderr: "".to_string(),
        };
        assert_eq!(err.exit_code(), Some(42));

        let err = BashError::Signal {
            signal_num: 9,
            signal_name: "SIGKILL".to_string(),
        };
        assert_eq!(err.exit_code(), Some(137)); // 128 + 9
    }

    #[test]
    fn test_retryable_errors() {
        let timeout_err = BashError::Timeout {
            duration: Duration::from_secs(30),
            partial_stdout: "".to_string(),
            partial_stderr: "".to_string(),
        };
        assert!(timeout_err.is_retryable());

        let cmd_not_found = BashError::CommandNotFound {
            command: "foo".to_string(),
        };
        assert!(!cmd_not_found.is_retryable());
    }

    #[test]
    fn test_error_response_serialization() {
        let err = BashError::CommandNotFound {
            command: "foo".to_string(),
        };
        let response = BashErrorResponse::from(&err);
        
        assert_eq!(response.code, "BASH_CMD_NOT_FOUND");
        assert!(response.message.contains("foo"));
        assert!(response.suggestion.contains("which"));
        assert!(!response.retryable);
    }

    #[test]
    fn test_error_codes() {
        let err = BashError::CommandNotFound {
            command: "foo".to_string(),
        };
        assert_eq!(err.error_code(), "BASH_CMD_NOT_FOUND");

        let err = BashError::Signal {
            signal_num: 9,
            signal_name: "SIGKILL".to_string(),
        };
        assert_eq!(err.error_code(), "BASH_SIGNAL");
    }

    #[test]
    fn test_extract_command_from_error() {
        assert_eq!(
            extract_command_from_error("bash: foo: command not found"),
            Some("foo".to_string())
        );
        
        assert_eq!(
            extract_command_from_error("bash: bar: No such file or directory"),
            Some("bar".to_string())
        );
        
        assert_eq!(extract_command_from_error("invalid format"), None);
    }

    #[test]
    fn test_common_exit_code_suggestions() {
        let err = BashError::ExitCode {
            exit_code: 127,
            message: "test".to_string(),
            stderr: "".to_string(),
        };
        let suggestion = err.recovery_suggestion();
        assert!(suggestion.contains("Command not found"));
        assert!(suggestion.contains("PATH"));

        let err = BashError::ExitCode {
            exit_code: 126,
            message: "test".to_string(),
            stderr: "".to_string(),
        };
        let suggestion = err.recovery_suggestion();
        assert!(suggestion.contains("not executable"));
        assert!(suggestion.contains("permissions"));
    }

    #[test]
    fn test_comprehensive_error_handling() {
        // Test 1: Exit code to error mapping
        let exit_code_err = error_from_exit(42, "");
        assert!(matches!(exit_code_err, BashError::ExitCode { exit_code: 42, .. }));
        assert_eq!(exit_code_err.exit_code(), Some(42));

        // Test 2: Signal name interpretation 
        let signal_err = error_from_exit(137, ""); // 128 + 9 = SIGKILL
        assert!(matches!(signal_err, BashError::Signal { signal_num: 9, ref signal_name } if signal_name == "SIGKILL"));
        assert_eq!(signal_err.exit_code(), Some(137));

        // Test 3: Common error pattern detection
        let cmd_not_found = analyze_stderr("bash: nonexistentcmd: command not found").unwrap();
        assert!(matches!(cmd_not_found, BashError::CommandNotFound { ref command } if command == "nonexistentcmd"));

        let perm_denied = analyze_stderr("bash: /etc/shadow: Permission denied").unwrap();
        assert!(matches!(perm_denied, BashError::PermissionDenied { .. }));

        // Test 4: Helpful error messages with suggestions
        let cmd_err = BashError::CommandNotFound { command: "foo".to_string() };
        let suggestion = cmd_err.recovery_suggestion();
        assert!(suggestion.contains("which") && suggestion.contains("command -v"));

        let timeout_err = BashError::Timeout {
            duration: Duration::from_secs(30),
            partial_stdout: "".to_string(),
            partial_stderr: "".to_string(),
        };
        let timeout_suggestion = timeout_err.recovery_suggestion();
        assert!(timeout_suggestion.contains("Increase timeout") && timeout_suggestion.contains("30s"));

        // Test 5: Error categorization for retry logic
        assert!(timeout_err.is_retryable());
        assert!(!cmd_err.is_retryable());
        let io_err = BashError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(io_err.is_retryable());

        // Test 6: Serializable error responses  
        let response = BashErrorResponse::from(&cmd_err);
        assert_eq!(response.code, "BASH_CMD_NOT_FOUND");
        assert!(response.message.contains("foo"));
        assert!(response.suggestion.contains("which"));
        assert!(!response.retryable);
        assert_eq!(response.exit_code, None);

        let exit_response = BashErrorResponse::from(&exit_code_err);
        assert_eq!(exit_response.code, "BASH_EXIT_CODE");
        assert_eq!(exit_response.exit_code, Some(42));
        assert!(!exit_response.retryable);
    }
}

/// Demonstrate comprehensive bash error analysis capabilities.
///
/// This function shows how the new error handling works with various scenarios.
#[cfg(test)]
pub fn demo_error_analysis() -> Vec<BashErrorResponse> {
    let scenarios = vec![
        // Exit code scenarios
        error_from_exit(42, "Some generic error message"),
        error_from_exit(127, "bash: unknowncmd: command not found"),
        error_from_exit(126, "/usr/bin/protected: Permission denied"),
        error_from_exit(137, "Process terminated"),

        // Common error patterns
        BashError::CommandNotFound {
            command: "missingcmd".to_string(),
        },
        BashError::PermissionDenied {
            message: "Access denied to /root/secret".to_string(),
        },
        BashError::Timeout {
            duration: Duration::from_secs(300),
            partial_stdout: "Started processing...".to_string(),
            partial_stderr: "Warning: process taking longer than expected".to_string(),
        },
        BashError::Signal {
            signal_num: 9,
            signal_name: "SIGKILL".to_string(),
        },
    ];

    scenarios.iter().map(BashErrorResponse::from).collect()
}

#[cfg(test)]
mod demo_tests {
    use super::*;

    #[test]
    fn test_error_analysis_demo() {
        let error_responses = demo_error_analysis();
        
        // We should have all different error types represented
        assert_eq!(error_responses.len(), 8);
        
        // Check that we have the expected error codes
        let error_codes: Vec<&str> = error_responses.iter().map(|r| r.code.as_str()).collect();
        assert!(error_codes.contains(&"BASH_EXIT_CODE"));
        assert!(error_codes.contains(&"BASH_CMD_NOT_FOUND"));
        assert!(error_codes.contains(&"BASH_PERMISSION_DENIED"));
        assert!(error_codes.contains(&"BASH_TIMEOUT"));
        assert!(error_codes.contains(&"BASH_SIGNAL"));

        // Check that retryable errors are correctly identified
        let retryable_count = error_responses.iter().filter(|r| r.retryable).count();
        assert!(retryable_count > 0);

        // Check that all responses have suggestions
        assert!(error_responses.iter().all(|r| !r.suggestion.is_empty()));

        // Print error analysis results for manual verification
        for (i, response) in error_responses.iter().enumerate() {
            println!("Error {}: {} - {}", i + 1, response.code, response.message);
            println!("  Suggestion: {}", response.suggestion);
            println!("  Retryable: {}", response.retryable);
            if let Some(exit_code) = response.exit_code {
                println!("  Exit code: {}", exit_code);
            }
            println!();
        }
    }
}