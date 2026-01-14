//! Rate limit storage backends.

use super::types::{RateLimitConfig, RateLimitState};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// Trait for rate limit storage.
#[async_trait]
pub trait RateLimitStore: Send + Sync {
    /// Check if request is allowed and consume a token.
    async fn check_and_consume(&self, key: &str, config: &RateLimitConfig) -> RateLimitResult;

    /// Get current state for a key.
    async fn get_state(&self, key: &str) -> Option<RateLimitState>;
}

/// Result of rate limit check.
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: std::time::Instant,
    pub retry_after: Option<std::time::Duration>,
}

/// In-memory rate limit store.
pub struct InMemoryStore {
    states: DashMap<String, RateLimitState>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimitStore for InMemoryStore {
    async fn check_and_consume(&self, key: &str, config: &RateLimitConfig) -> RateLimitResult {
        let mut entry = self.states
            .entry(key.to_string())
            .or_insert_with(|| RateLimitState::new(config));

        let state = entry.value_mut();
        let allowed = state.try_consume();

        RateLimitResult {
            allowed,
            limit: state.max_tokens,
            remaining: state.tokens,
            reset_at: state.last_refill + config.window,
            retry_after: if allowed { None } else { Some(state.retry_after()) },
        }
    }

    async fn get_state(&self, key: &str) -> Option<RateLimitState> {
        self.states.get(key).map(|entry| entry.value().clone())
    }
}

/// Redis-backed rate limit store (for distributed systems).
#[cfg(feature = "redis")]
pub struct RedisStore {
    client: redis::Client,
    prefix: String,
}

#[cfg(feature = "redis")]
impl RedisStore {
    pub fn new(redis_url: &str, prefix: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
            prefix: prefix.to_string(),
        })
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl RateLimitStore for RedisStore {
    async fn check_and_consume(&self, key: &str, config: &RateLimitConfig) -> RateLimitResult {
        // Implement using Redis MULTI/EXEC or Lua script for atomicity
        todo!("Implement Redis rate limiting")
    }

    async fn get_state(&self, key: &str) -> Option<RateLimitState> {
        todo!("Implement Redis state retrieval")
    }
}