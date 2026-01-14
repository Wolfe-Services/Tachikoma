//! Database configuration.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// Database connection URL.
    pub url: String,
    /// Maximum connections in pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Minimum connections in pool.
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// Connection acquire timeout.
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout_secs: u64,
    /// Connection idle timeout.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    /// Maximum connection lifetime.
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,
    /// Statement cache size.
    #[serde(default = "default_statement_cache")]
    pub statement_cache_size: usize,
    /// Enable query logging.
    #[serde(default)]
    pub log_queries: bool,
    /// Slow query threshold (ms).
    #[serde(default = "default_slow_query")]
    pub slow_query_threshold_ms: u64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_acquire_timeout() -> u64 {
    10
}

fn default_idle_timeout() -> u64 {
    600
}

fn default_max_lifetime() -> u64 {
    3600
}

fn default_statement_cache() -> usize {
    100
}

fn default_slow_query() -> u64 {
    1000
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            acquire_timeout_secs: default_acquire_timeout(),
            idle_timeout_secs: default_idle_timeout(),
            max_lifetime_secs: default_max_lifetime(),
            statement_cache_size: default_statement_cache(),
            log_queries: false,
            slow_query_threshold_ms: default_slow_query(),
        }
    }
}

impl DbConfig {
    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_secs(self.acquire_timeout_secs)
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }

    pub fn slow_query_threshold(&self) -> Duration {
        Duration::from_millis(self.slow_query_threshold_ms)
    }
}