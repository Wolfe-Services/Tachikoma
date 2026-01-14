# Spec 386: Authentication Rate Limiting

## Overview
Implement rate limiting for authentication endpoints to prevent brute force attacks and abuse.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Rate Limiter
```rust
// src/auth/rate_limit.rs

use chrono::{DateTime, Duration, Utc};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn, instrument};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Login attempts per window
    pub login_attempts: u32,
    /// Login window duration
    pub login_window: Duration,
    /// Registration attempts per window
    pub registration_attempts: u32,
    /// Registration window duration
    pub registration_window: Duration,
    /// Password reset attempts per window
    pub password_reset_attempts: u32,
    /// Password reset window duration
    pub password_reset_window: Duration,
    /// Magic link attempts per window
    pub magic_link_attempts: u32,
    /// Magic link window duration
    pub magic_link_window: Duration,
    /// Global API requests per window
    pub api_requests: u32,
    /// API request window duration
    pub api_window: Duration,
    /// Enable progressive delays after failures
    pub progressive_delay: bool,
    /// Base delay multiplier (seconds)
    pub delay_multiplier: u32,
    /// Maximum delay (seconds)
    pub max_delay: u32,
    /// Lockout threshold (failures before lockout)
    pub lockout_threshold: u32,
    /// Lockout duration
    pub lockout_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            login_attempts: 5,
            login_window: Duration::minutes(15),
            registration_attempts: 3,
            registration_window: Duration::hours(1),
            password_reset_attempts: 3,
            password_reset_window: Duration::hours(1),
            magic_link_attempts: 5,
            magic_link_window: Duration::hours(1),
            api_requests: 100,
            api_window: Duration::minutes(1),
            progressive_delay: true,
            delay_multiplier: 2,
            max_delay: 30,
            lockout_threshold: 10,
            lockout_duration: Duration::hours(1),
        }
    }
}

/// Rate limit action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitAction {
    Login,
    Registration,
    PasswordReset,
    MagicLink,
    ApiRequest,
    TokenRefresh,
    DeviceCode,
}

impl RateLimitAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Registration => "registration",
            Self::PasswordReset => "password_reset",
            Self::MagicLink => "magic_link",
            Self::ApiRequest => "api_request",
            Self::TokenRefresh => "token_refresh",
            Self::DeviceCode => "device_code",
        }
    }
}

/// Rate limit key (identifier for rate limiting)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RateLimitKey {
    pub action: RateLimitAction,
    pub identifier: String,  // email, IP, or user_id
    pub identifier_type: IdentifierType,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum IdentifierType {
    Email,
    IpAddress,
    UserId,
}

impl RateLimitKey {
    pub fn email(action: RateLimitAction, email: &str) -> Self {
        Self {
            action,
            identifier: email.to_lowercase(),
            identifier_type: IdentifierType::Email,
        }
    }

    pub fn ip(action: RateLimitAction, ip: &str) -> Self {
        Self {
            action,
            identifier: ip.to_string(),
            identifier_type: IdentifierType::IpAddress,
        }
    }

    pub fn user(action: RateLimitAction, user_id: &str) -> Self {
        Self {
            action,
            identifier: user_id.to_string(),
            identifier_type: IdentifierType::UserId,
        }
    }

    fn cache_key(&self) -> String {
        format!("{}:{}:{}", self.action.as_str(), self.identifier_type_str(), self.identifier)
    }

    fn identifier_type_str(&self) -> &'static str {
        match self.identifier_type {
            IdentifierType::Email => "email",
            IdentifierType::IpAddress => "ip",
            IdentifierType::UserId => "user",
        }
    }
}

/// Rate limit entry
#[derive(Debug, Clone)]
pub struct RateLimitEntry {
    pub count: u32,
    pub first_attempt: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
    pub locked_until: Option<DateTime<Utc>>,
}

impl RateLimitEntry {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            count: 1,
            first_attempt: now,
            last_attempt: now,
            locked_until: None,
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
        self.last_attempt = Utc::now();
    }

    pub fn is_locked(&self) -> bool {
        self.locked_until
            .map(|until| Utc::now() < until)
            .unwrap_or(false)
    }

    pub fn lock(&mut self, duration: Duration) {
        self.locked_until = Some(Utc::now() + duration);
    }

    pub fn is_window_expired(&self, window: Duration) -> bool {
        Utc::now() > self.first_attempt + window
    }

    pub fn reset(&mut self) {
        let now = Utc::now();
        self.count = 1;
        self.first_attempt = now;
        self.last_attempt = now;
        self.locked_until = None;
    }
}

impl Default for RateLimitEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request allowed
    Allowed {
        remaining: u32,
        reset_at: DateTime<Utc>,
    },
    /// Request rate limited
    Limited {
        retry_after: Duration,
        reset_at: DateTime<Utc>,
    },
    /// Account locked out
    Locked {
        until: DateTime<Utc>,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }

    pub fn retry_after_seconds(&self) -> Option<u64> {
        match self {
            Self::Limited { retry_after, .. } => Some(retry_after.num_seconds() as u64),
            Self::Locked { until } => {
                let duration = *until - Utc::now();
                Some(duration.num_seconds().max(0) as u64)
            }
            Self::Allowed { .. } => None,
        }
    }
}

/// In-memory rate limiter with optional database persistence
pub struct RateLimiter {
    config: RateLimitConfig,
    cache: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    pool: Option<SqlitePool>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            pool: None,
        }
    }

    pub fn with_persistence(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Check if request is allowed
    #[instrument(skip(self))]
    pub async fn check(&self, key: &RateLimitKey) -> RateLimitResult {
        let (limit, window) = self.get_limits(key.action);
        let cache_key = key.cache_key();

        let mut cache = self.cache.write().await;

        let entry = cache.entry(cache_key.clone()).or_insert_with(RateLimitEntry::new);

        // Check if locked
        if entry.is_locked() {
            return RateLimitResult::Locked {
                until: entry.locked_until.unwrap(),
            };
        }

        // Check if window expired - reset if so
        if entry.is_window_expired(window) {
            entry.reset();
        }

        // Check if over limit
        if entry.count >= limit {
            // Check if should lock
            if entry.count >= self.config.lockout_threshold {
                entry.lock(self.config.lockout_duration);
                warn!("Locking {} due to too many attempts", cache_key);
                return RateLimitResult::Locked {
                    until: entry.locked_until.unwrap(),
                };
            }

            let retry_after = self.calculate_delay(entry.count);
            return RateLimitResult::Limited {
                retry_after,
                reset_at: entry.first_attempt + window,
            };
        }

        RateLimitResult::Allowed {
            remaining: limit - entry.count,
            reset_at: entry.first_attempt + window,
        }
    }

    /// Record an attempt (call after check if proceeding)
    pub async fn record(&self, key: &RateLimitKey) {
        let cache_key = key.cache_key();
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(&cache_key) {
            entry.increment();
        }

        // Persist to database if configured
        if let Some(pool) = &self.pool {
            let _ = self.persist_entry(pool, key, cache.get(&cache_key)).await;
        }
    }

    /// Record a failed attempt (increments failure counter)
    pub async fn record_failure(&self, key: &RateLimitKey) {
        self.record(key).await;
    }

    /// Record a successful attempt (may reset counter)
    pub async fn record_success(&self, key: &RateLimitKey) {
        let cache_key = key.cache_key();
        let mut cache = self.cache.write().await;

        // Reset on success for login attempts
        if key.action == RateLimitAction::Login {
            cache.remove(&cache_key);
        }

        // Clean up database entry
        if let Some(pool) = &self.pool {
            let _ = self.clear_entry(pool, key).await;
        }
    }

    /// Clear rate limit for a key
    pub async fn clear(&self, key: &RateLimitKey) {
        let cache_key = key.cache_key();
        let mut cache = self.cache.write().await;
        cache.remove(&cache_key);

        if let Some(pool) = &self.pool {
            let _ = self.clear_entry(pool, key).await;
        }
    }

    /// Get limits for an action
    fn get_limits(&self, action: RateLimitAction) -> (u32, Duration) {
        match action {
            RateLimitAction::Login => (self.config.login_attempts, self.config.login_window),
            RateLimitAction::Registration => (self.config.registration_attempts, self.config.registration_window),
            RateLimitAction::PasswordReset => (self.config.password_reset_attempts, self.config.password_reset_window),
            RateLimitAction::MagicLink => (self.config.magic_link_attempts, self.config.magic_link_window),
            RateLimitAction::ApiRequest => (self.config.api_requests, self.config.api_window),
            RateLimitAction::TokenRefresh => (self.config.api_requests, self.config.api_window),
            RateLimitAction::DeviceCode => (self.config.login_attempts, self.config.login_window),
        }
    }

    /// Calculate delay for progressive rate limiting
    fn calculate_delay(&self, attempt_count: u32) -> Duration {
        if !self.config.progressive_delay {
            return Duration::seconds(5);
        }

        let delay_seconds = (self.config.delay_multiplier as i64)
            .pow(attempt_count.saturating_sub(1).min(10))
            .min(self.config.max_delay as i64);

        Duration::seconds(delay_seconds)
    }

    /// Persist entry to database
    async fn persist_entry(
        &self,
        pool: &SqlitePool,
        key: &RateLimitKey,
        entry: Option<&RateLimitEntry>,
    ) -> Result<(), sqlx::Error> {
        if let Some(entry) = entry {
            sqlx::query(r#"
                INSERT INTO rate_limits (key, action, identifier_type, count, first_attempt, last_attempt, locked_until)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(key) DO UPDATE SET
                    count = excluded.count,
                    last_attempt = excluded.last_attempt,
                    locked_until = excluded.locked_until
            "#)
            .bind(&key.cache_key())
            .bind(key.action.as_str())
            .bind(key.identifier_type_str())
            .bind(entry.count as i32)
            .bind(entry.first_attempt)
            .bind(entry.last_attempt)
            .bind(entry.locked_until)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Clear entry from database
    async fn clear_entry(&self, pool: &SqlitePool, key: &RateLimitKey) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM rate_limits WHERE key = ?")
            .bind(&key.cache_key())
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Cleanup old entries
    pub async fn cleanup(&self) -> Result<usize, sqlx::Error> {
        // Clean in-memory cache
        let mut cache = self.cache.write().await;
        let before = cache.len();

        cache.retain(|_, entry| {
            !entry.is_window_expired(Duration::hours(24)) || entry.is_locked()
        });

        let removed = before - cache.len();

        // Clean database
        if let Some(pool) = &self.pool {
            sqlx::query(
                "DELETE FROM rate_limits WHERE last_attempt < datetime('now', '-1 day') AND locked_until IS NULL"
            )
            .execute(pool)
            .await?;
        }

        debug!("Cleaned up {} rate limit entries", removed);
        Ok(removed)
    }
}

/// Rate limit middleware state
pub struct RateLimitState {
    pub limiter: Arc<RateLimiter>,
}

/// Rate limit database schema
pub fn rate_limit_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS rate_limits (
    key TEXT PRIMARY KEY NOT NULL,
    action TEXT NOT NULL,
    identifier_type TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    first_attempt TEXT NOT NULL DEFAULT (datetime('now')),
    last_attempt TEXT NOT NULL DEFAULT (datetime('now')),
    locked_until TEXT
);

CREATE INDEX IF NOT EXISTS idx_rate_limits_action ON rate_limits(action);
CREATE INDEX IF NOT EXISTS idx_rate_limits_last ON rate_limits(last_attempt);
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_initial_requests() {
        let limiter = RateLimiter::new(RateLimitConfig::default());
        let key = RateLimitKey::email(RateLimitAction::Login, "test@example.com");

        let result = limiter.check(&key).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_after_limit() {
        let config = RateLimitConfig {
            login_attempts: 2,
            login_window: Duration::minutes(15),
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        let key = RateLimitKey::email(RateLimitAction::Login, "test@example.com");

        // First two should be allowed
        assert!(limiter.check(&key).await.is_allowed());
        limiter.record(&key).await;

        assert!(limiter.check(&key).await.is_allowed());
        limiter.record(&key).await;

        // Third should be blocked
        let result = limiter.check(&key).await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_success_resets_counter() {
        let config = RateLimitConfig {
            login_attempts: 2,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        let key = RateLimitKey::email(RateLimitAction::Login, "test@example.com");

        limiter.record(&key).await;
        limiter.record_success(&key).await;

        // Should be allowed again
        let result = limiter.check(&key).await;
        assert!(result.is_allowed());
    }
}
```

## Files to Create
- `src/auth/rate_limit.rs` - Rate limiting implementation
