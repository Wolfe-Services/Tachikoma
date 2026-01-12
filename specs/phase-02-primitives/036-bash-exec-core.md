# 036 - Bash Execution Core

**Phase:** 2 - Five Primitives
**Spec ID:** 036
**Status:** Planned
**Dependencies:** 031-primitives-crate
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement the `bash` primitive for executing shell commands with proper process management, environment handling, and security controls.

---

## Acceptance Criteria

- [ ] Execute bash commands asynchronously
- [ ] Capture stdout and stderr separately
- [ ] Support working directory specification
- [ ] Environment variable injection
- [ ] Command sanitization and validation
- [ ] Process cleanup on cancellation

---

## Implementation Details

### 1. Bash Module (src/bash/mod.rs)

```rust
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
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{debug, instrument, warn};

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
    let mut child = cmd.spawn().map_err(|e| PrimitiveError::Io(e))?;

    // Read output
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (stdout_content, stderr_content) = read_output(stdout, stderr).await?;

    // Wait for completion
    let status = child.wait().await.map_err(|e| PrimitiveError::Io(e))?;

    let exit_code = status.code().unwrap_or(-1);
    let duration = start.elapsed();

    debug!(
        "Command completed with exit code {} in {:?}",
        exit_code, duration
    );

    Ok(BashResult {
        exit_code,
        stdout: stdout_content,
        stderr: stderr_content,
        timed_out: false,
        metadata: ExecutionMetadata {
            duration,
            operation_id: ctx.operation_id.clone(),
            primitive: "bash".to_string(),
        },
    })
}

/// Read stdout and stderr concurrently.
async fn read_output(
    stdout: Option<tokio::process::ChildStdout>,
    stderr: Option<tokio::process::ChildStderr>,
) -> PrimitiveResult<(String, String)> {
    let stdout_future = async {
        let mut content = Vec::new();
        if let Some(mut stdout) = stdout {
            stdout.take(MAX_OUTPUT_SIZE as u64).read_to_end(&mut content).await?;
        }
        Ok::<_, std::io::Error>(String::from_utf8_lossy(&content).into_owned())
    };

    let stderr_future = async {
        let mut content = Vec::new();
        if let Some(mut stderr) = stderr {
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
}
```

### 2. Bash Options (src/bash/options.rs)

```rust
//! Options for bash command execution.

use std::collections::HashMap;
use std::time::Duration;

/// Options for bash command execution.
#[derive(Debug, Clone)]
pub struct BashOptions {
    /// Working directory for the command.
    pub working_dir: Option<String>,
    /// Environment variables to set.
    pub env_vars: HashMap<String, String>,
    /// Clear environment before setting vars.
    pub clear_env: bool,
    /// Command timeout.
    pub timeout: Option<Duration>,
    /// Blocked command patterns.
    pub blocked_commands: Vec<String>,
    /// Maximum output size in bytes.
    pub max_output_size: usize,
}

impl Default for BashOptions {
    fn default() -> Self {
        Self {
            working_dir: None,
            env_vars: HashMap::new(),
            clear_env: false,
            timeout: Some(Duration::from_secs(120)),
            blocked_commands: vec![
                "rm -rf /".to_string(),
                ":(){ :|:& };:".to_string(), // Fork bomb
                "mkfs".to_string(),
                "dd if=/dev/".to_string(),
                "> /dev/sd".to_string(),
            ],
            max_output_size: 10 * 1024 * 1024,
        }
    }
}

impl BashOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set working directory.
    pub fn working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    /// Add environment variable.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Add multiple environment variables.
    pub fn envs(mut self, vars: HashMap<String, String>) -> Self {
        self.env_vars.extend(vars);
        self
    }

    /// Clear environment before command.
    pub fn clear_env(mut self) -> Self {
        self.clear_env = true;
        self
    }

    /// Set timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// No timeout.
    pub fn no_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Add blocked command pattern.
    pub fn block_command(mut self, pattern: &str) -> Self {
        self.blocked_commands.push(pattern.to_string());
        self
    }

    /// Set max output size.
    pub fn max_output(mut self, size: usize) -> Self {
        self.max_output_size = size;
        self
    }
}
```

### 3. Command Sanitization (src/bash/sanitize.rs)

```rust
//! Command validation and sanitization.

use crate::error::{PrimitiveError, PrimitiveResult};
use tracing::warn;

/// Validates commands before execution.
pub struct CommandValidator {
    blocked_patterns: Vec<String>,
}

impl CommandValidator {
    /// Create a new validator with blocked patterns.
    pub fn new(blocked: &[String]) -> Self {
        Self {
            blocked_patterns: blocked.to_vec(),
        }
    }

    /// Validate a command.
    pub fn validate(&self, command: &str) -> PrimitiveResult<()> {
        // Check empty command
        if command.trim().is_empty() {
            return Err(PrimitiveError::Validation {
                message: "Empty command".to_string(),
            });
        }

        // Check blocked patterns
        for pattern in &self.blocked_patterns {
            if command.contains(pattern) {
                warn!("Blocked command pattern detected: {}", pattern);
                return Err(PrimitiveError::Validation {
                    message: format!("Command contains blocked pattern: {}", pattern),
                });
            }
        }

        // Check for obviously dangerous patterns
        if self.is_dangerous(command) {
            return Err(PrimitiveError::Validation {
                message: "Command appears to be dangerous".to_string(),
            });
        }

        Ok(())
    }

    /// Check for dangerous command patterns.
    fn is_dangerous(&self, command: &str) -> bool {
        let dangerous_patterns = [
            // System destruction
            ("rm", "-rf", "/"),
            // Disk operations
            ("dd", "if=", "/dev/"),
            // Network backdoors
            ("nc", "-e", "/bin/"),
            ("ncat", "-e", "/bin/"),
        ];

        let lower = command.to_lowercase();

        for (cmd, arg1, arg2) in dangerous_patterns {
            if lower.contains(cmd) && lower.contains(arg1) && lower.contains(arg2) {
                return true;
            }
        }

        // Check for encoded commands
        if lower.contains("base64") && (lower.contains("|bash") || lower.contains("| bash")) {
            return true;
        }

        false
    }

    /// Escape a string for safe shell use.
    pub fn escape_arg(arg: &str) -> String {
        // Use single quotes and escape any single quotes in the string
        format!("'{}'", arg.replace('\'', "'\\''"))
    }

    /// Build a safe command with arguments.
    pub fn build_safe_command(cmd: &str, args: &[&str]) -> String {
        let escaped_args: Vec<String> = args.iter().map(|a| Self::escape_arg(a)).collect();
        format!("{} {}", cmd, escaped_args.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_normal_command() {
        let validator = CommandValidator::new(&[]);
        assert!(validator.validate("ls -la").is_ok());
        assert!(validator.validate("git status").is_ok());
        assert!(validator.validate("echo 'hello'").is_ok());
    }

    #[test]
    fn test_validate_blocked_pattern() {
        let validator = CommandValidator::new(&["rm -rf /".to_string()]);
        assert!(validator.validate("rm -rf /").is_err());
    }

    #[test]
    fn test_validate_dangerous() {
        let validator = CommandValidator::new(&[]);
        assert!(validator.validate("rm -rf /").is_err());
        assert!(validator.validate("dd if=/dev/zero of=/dev/sda").is_err());
    }

    #[test]
    fn test_escape_arg() {
        assert_eq!(CommandValidator::escape_arg("hello"), "'hello'");
        assert_eq!(CommandValidator::escape_arg("it's"), "'it'\\''s'");
    }
}
```

---

## Testing Requirements

1. Basic commands execute correctly
2. Exit codes are captured accurately
3. Stdout and stderr are separated
4. Environment variables are set correctly
5. Working directory is respected
6. Blocked commands are rejected
7. Process cleanup works on cancellation

---

## Related Specs

- Depends on: [031-primitives-crate.md](031-primitives-crate.md)
- Next: [037-bash-timeout.md](037-bash-timeout.md)
- Related: [039-bash-errors.md](039-bash-errors.md)
