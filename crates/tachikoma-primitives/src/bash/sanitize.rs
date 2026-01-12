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
    fn test_validate_empty() {
        let validator = CommandValidator::new(&[]);
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err());
    }

    #[test]
    fn test_escape_arg() {
        assert_eq!(CommandValidator::escape_arg("hello"), "'hello'");
        assert_eq!(CommandValidator::escape_arg("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_build_safe_command() {
        let result = CommandValidator::build_safe_command("echo", &["hello", "world"]);
        assert_eq!(result, "echo 'hello' 'world'");
    }
}