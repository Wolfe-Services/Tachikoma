//! Scaling configuration.

use serde::{Deserialize, Serialize};

/// Server scaling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Number of worker threads.
    #[serde(default = "default_workers")]
    pub workers: usize,
    /// Maximum concurrent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    /// Maximum pending connections (backlog).
    #[serde(default = "default_backlog")]
    pub backlog: u32,
    /// Request queue size.
    #[serde(default = "default_queue_size")]
    pub queue_size: usize,
    /// Enable connection keep-alive.
    #[serde(default = "default_true")]
    pub keep_alive: bool,
    /// Keep-alive timeout (seconds).
    #[serde(default = "default_keepalive_timeout")]
    pub keepalive_timeout_secs: u64,
    /// Request timeout (seconds).
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    /// Enable HTTP/2.
    #[serde(default = "default_true")]
    pub http2: bool,
}

fn default_workers() -> usize {
    num_cpus::get()
}

fn default_max_connections() -> usize {
    10000
}

fn default_backlog() -> u32 {
    1024
}

fn default_queue_size() -> usize {
    1000
}

fn default_true() -> bool {
    true
}

fn default_keepalive_timeout() -> u64 {
    75
}

fn default_request_timeout() -> u64 {
    30
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            workers: default_workers(),
            max_connections: default_max_connections(),
            backlog: default_backlog(),
            queue_size: default_queue_size(),
            keep_alive: true,
            keepalive_timeout_secs: default_keepalive_timeout(),
            request_timeout_secs: default_request_timeout(),
            http2: true,
        }
    }
}

impl ScalingConfig {
    /// Configuration for development.
    pub fn development() -> Self {
        Self {
            workers: 2,
            max_connections: 1000,
            backlog: 128,
            queue_size: 100,
            ..Default::default()
        }
    }

    /// Configuration for production.
    pub fn production() -> Self {
        Self {
            workers: num_cpus::get() * 2,
            max_connections: 50000,
            backlog: 2048,
            queue_size: 10000,
            ..Default::default()
        }
    }
}