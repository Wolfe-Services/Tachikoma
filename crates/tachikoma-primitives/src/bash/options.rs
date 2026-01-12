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