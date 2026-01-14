//! Rate limiting types.

use std::time::Duration;

/// Rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests allowed.
    pub max_requests: u32,
    /// Time window for the limit.
    pub window: Duration,
    /// Burst allowance (extra requests allowed in short bursts).
    pub burst: u32,
    /// Key extraction strategy.
    pub key_strategy: KeyStrategy,
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(window_secs),
            burst: 0,
            key_strategy: KeyStrategy::Ip,
        }
    }

    pub fn with_burst(mut self, burst: u32) -> Self {
        self.burst = burst;
        self
    }

    pub fn by_user(mut self) -> Self {
        self.key_strategy = KeyStrategy::User;
        self
    }

    pub fn by_ip(mut self) -> Self {
        self.key_strategy = KeyStrategy::Ip;
        self
    }

    pub fn by_api_key(mut self) -> Self {
        self.key_strategy = KeyStrategy::ApiKey;
        self
    }

    pub fn composite(mut self) -> Self {
        self.key_strategy = KeyStrategy::Composite;
        self
    }
}

/// Strategy for extracting rate limit key.
#[derive(Debug, Clone, Copy)]
pub enum KeyStrategy {
    /// Rate limit by IP address.
    Ip,
    /// Rate limit by authenticated user.
    User,
    /// Rate limit by API key.
    ApiKey,
    /// Composite key (IP + User if authenticated).
    Composite,
}

/// Rate limit state for a key.
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// Current token count.
    pub tokens: u32,
    /// Last refill timestamp.
    pub last_refill: std::time::Instant,
    /// Maximum tokens (max_requests + burst).
    pub max_tokens: u32,
    /// Refill rate (tokens per second).
    pub refill_rate: f64,
}

impl RateLimitState {
    pub fn new(config: &RateLimitConfig) -> Self {
        let max_tokens = config.max_requests + config.burst;
        let refill_rate = config.max_requests as f64 / config.window.as_secs_f64();

        Self {
            tokens: max_tokens,
            last_refill: std::time::Instant::now(),
            max_tokens,
            refill_rate,
        }
    }

    /// Refill tokens based on elapsed time.
    pub fn refill(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = (elapsed.as_secs_f64() * self.refill_rate) as u32;

        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
            self.last_refill = now;
        }
    }

    /// Try to consume a token. Returns true if successful.
    pub fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Get time until next token is available.
    pub fn retry_after(&self) -> Duration {
        if self.tokens > 0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(1.0 / self.refill_rate)
        }
    }
}

/// Rate limit info for response headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset: u64,
    pub retry_after: Option<u64>,
}