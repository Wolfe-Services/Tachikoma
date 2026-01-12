//! Rate limit configuration.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Global tokens per second.
    pub global_tokens_per_second: u64,
    /// Global burst size.
    pub global_burst_size: u64,
    /// Default per-primitive tokens per second.
    pub default_tokens_per_second: u64,
    /// Default per-primitive burst size.
    pub default_burst_size: u64,
    /// Per-primitive rate limits (tokens/second).
    pub primitive_limits: HashMap<String, u64>,
    /// Per-primitive burst sizes.
    pub primitive_burst: HashMap<String, u64>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut primitive_limits = HashMap::new();
        primitive_limits.insert("read_file".to_string(), 100);
        primitive_limits.insert("list_files".to_string(), 50);
        primitive_limits.insert("bash".to_string(), 10);
        primitive_limits.insert("edit_file".to_string(), 20);
        primitive_limits.insert("code_search".to_string(), 30);

        let mut primitive_burst = HashMap::new();
        primitive_burst.insert("read_file".to_string(), 200);
        primitive_burst.insert("list_files".to_string(), 100);
        primitive_burst.insert("bash".to_string(), 20);
        primitive_burst.insert("edit_file".to_string(), 40);
        primitive_burst.insert("code_search".to_string(), 60);

        Self {
            global_tokens_per_second: 200,
            global_burst_size: 500,
            default_tokens_per_second: 50,
            default_burst_size: 100,
            primitive_limits,
            primitive_burst,
        }
    }
}

impl RateLimitConfig {
    /// Create a new config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable rate limiting (for testing).
    pub fn disabled() -> Self {
        Self {
            global_tokens_per_second: u64::MAX,
            global_burst_size: u64::MAX,
            default_tokens_per_second: u64::MAX,
            default_burst_size: u64::MAX,
            primitive_limits: HashMap::new(),
            primitive_burst: HashMap::new(),
        }
    }

    /// Set global limit.
    pub fn global_limit(mut self, tokens_per_second: u64, burst: u64) -> Self {
        self.global_tokens_per_second = tokens_per_second;
        self.global_burst_size = burst;
        self
    }

    /// Set limit for a primitive.
    pub fn primitive_limit(
        mut self,
        primitive: &str,
        tokens_per_second: u64,
        burst: u64,
    ) -> Self {
        self.primitive_limits.insert(primitive.to_string(), tokens_per_second);
        self.primitive_burst.insert(primitive.to_string(), burst);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert!(config.primitive_limits.contains_key("bash"));
        assert!(config.global_tokens_per_second > 0);
    }

    #[test]
    fn test_disabled_config() {
        let config = RateLimitConfig::disabled();
        assert_eq!(config.global_tokens_per_second, u64::MAX);
    }

    #[test]
    fn test_builder() {
        let config = RateLimitConfig::new()
            .global_limit(100, 200)
            .primitive_limit("bash", 5, 10);

        assert_eq!(config.global_tokens_per_second, 100);
        assert_eq!(config.primitive_limits.get("bash"), Some(&5));
    }
}