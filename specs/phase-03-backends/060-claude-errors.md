# 060 - Claude Error Handling

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 060
**Status:** Planned
**Dependencies:** 056-claude-api-client
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive error handling for the Claude backend, including API error parsing, retry strategies, rate limit handling, and error recovery mechanisms.

---

## Acceptance Criteria

- [ ] Parse all Claude API error types
- [ ] Map to `BackendError` variants
- [ ] Implement retry logic with backoff
- [ ] Handle rate limits with Retry-After
- [ ] Graceful degradation on overload
- [ ] Error context preservation

---

## Implementation Details

### 1. Error Types (src/error/types.rs)

```rust
//! Claude-specific error types.

use serde::Deserialize;
use std::time::Duration;
use tachikoma_backends_core::BackendError;

/// Claude API error response.
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeApiError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: ClaudeErrorDetail,
}

/// Detail of a Claude API error.
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Claude-specific error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeErrorType {
    /// Invalid API key or authentication failure.
    Authentication,
    /// Invalid request parameters.
    InvalidRequest,
    /// Rate limit exceeded.
    RateLimit,
    /// API overloaded.
    Overloaded,
    /// Permission denied.
    Permission,
    /// Resource not found.
    NotFound,
    /// Internal server error.
    Server,
    /// Unknown error type.
    Unknown,
}

impl ClaudeErrorType {
    /// Parse from API error type string.
    pub fn from_api_type(s: &str) -> Self {
        match s {
            "authentication_error" => Self::Authentication,
            "invalid_request_error" => Self::InvalidRequest,
            "rate_limit_error" => Self::RateLimit,
            "overloaded_error" => Self::Overloaded,
            "permission_error" => Self::Permission,
            "not_found_error" => Self::NotFound,
            "api_error" | "server_error" => Self::Server,
            _ => Self::Unknown,
        }
    }

    /// Check if this error type is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimit | Self::Overloaded | Self::Server)
    }

    /// Get recommended retry delay for this error type.
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            Self::RateLimit => Some(Duration::from_secs(60)),
            Self::Overloaded => Some(Duration::from_secs(30)),
            Self::Server => Some(Duration::from_secs(5)),
            _ => None,
        }
    }
}

/// Convert Claude API error to BackendError.
pub fn to_backend_error(
    status: u16,
    api_error: Option<ClaudeApiError>,
    body: &str,
) -> BackendError {
    let (error_type, message) = if let Some(err) = api_error {
        (
            ClaudeErrorType::from_api_type(&err.error.error_type),
            err.error.message,
        )
    } else {
        (error_type_from_status(status), body.to_string())
    };

    match error_type {
        ClaudeErrorType::Authentication => BackendError::Authentication(message),
        ClaudeErrorType::InvalidRequest => BackendError::InvalidRequest(message),
        ClaudeErrorType::RateLimit => BackendError::RateLimit {
            retry_after: error_type.retry_delay(),
            message,
        },
        ClaudeErrorType::Overloaded => BackendError::ServiceUnavailable {
            message,
            retry_after: error_type.retry_delay(),
        },
        ClaudeErrorType::Permission => BackendError::Authentication(message),
        ClaudeErrorType::NotFound => BackendError::InvalidRequest(message),
        ClaudeErrorType::Server => BackendError::Api { status, message },
        ClaudeErrorType::Unknown => BackendError::Api { status, message },
    }
}

/// Infer error type from HTTP status code.
fn error_type_from_status(status: u16) -> ClaudeErrorType {
    match status {
        401 => ClaudeErrorType::Authentication,
        403 => ClaudeErrorType::Permission,
        404 => ClaudeErrorType::NotFound,
        400 | 422 => ClaudeErrorType::InvalidRequest,
        429 => ClaudeErrorType::RateLimit,
        529 => ClaudeErrorType::Overloaded,
        500..=599 => ClaudeErrorType::Server,
        _ => ClaudeErrorType::Unknown,
    }
}
```

### 2. Retry Strategy (src/error/retry.rs)

```rust
//! Retry strategies for Claude API.

use std::time::Duration;
use tachikoma_backends_core::BackendError;
use tracing::{debug, info, warn};

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier.
    pub backoff_factor: f64,
    /// Add jitter to delays.
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a config for aggressive retries.
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(30),
            backoff_factor: 1.5,
            jitter: true,
        }
    }

    /// Create a config for patient retries.
    pub fn patient() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(120),
            backoff_factor: 2.5,
            jitter: true,
        }
    }

    /// Calculate delay for a given attempt.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_secs_f64()
            * self.backoff_factor.powi(attempt as i32);
        let capped_delay = base_delay.min(self.max_delay.as_secs_f64());

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter = rand::random::<f64>() * 0.25 * capped_delay;
            capped_delay + jitter
        } else {
            capped_delay
        };

        Duration::from_secs_f64(final_delay)
    }
}

/// Retry state tracker.
#[derive(Debug)]
pub struct RetryState {
    config: RetryConfig,
    attempt: u32,
    last_error: Option<BackendError>,
}

impl RetryState {
    /// Create a new retry state.
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            attempt: 0,
            last_error: None,
        }
    }

    /// Check if we should retry.
    pub fn should_retry(&self, error: &BackendError) -> bool {
        if self.attempt >= self.config.max_retries {
            debug!(attempts = self.attempt, "Max retries reached");
            return false;
        }

        error.is_retryable()
    }

    /// Get the delay before next retry.
    pub fn next_delay(&self, error: &BackendError) -> Duration {
        // Use Retry-After header if available
        if let BackendError::RateLimit { retry_after: Some(delay), .. } = error {
            return *delay;
        }
        if let BackendError::ServiceUnavailable { retry_after: Some(delay), .. } = error {
            return *delay;
        }

        self.config.delay_for_attempt(self.attempt)
    }

    /// Record a retry attempt.
    pub fn record_attempt(&mut self, error: BackendError) {
        self.attempt += 1;
        self.last_error = Some(error);
        info!(attempt = self.attempt, "Retry attempt recorded");
    }

    /// Get the number of attempts made.
    pub fn attempts(&self) -> u32 {
        self.attempt
    }

    /// Get the last error.
    pub fn last_error(&self) -> Option<&BackendError> {
        self.last_error.as_ref()
    }
}

/// Execute with retry.
pub async fn with_retry<F, Fut, T>(
    config: RetryConfig,
    operation: F,
) -> Result<T, BackendError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, BackendError>>,
{
    let mut state = RetryState::new(config);

    loop {
        match operation().await {
            Ok(result) => {
                if state.attempts() > 0 {
                    info!(attempts = state.attempts(), "Operation succeeded after retries");
                }
                return Ok(result);
            }
            Err(error) => {
                if !state.should_retry(&error) {
                    return Err(error);
                }

                let delay = state.next_delay(&error);
                warn!(
                    error = %error,
                    attempt = state.attempts() + 1,
                    delay_ms = delay.as_millis(),
                    "Retrying after error"
                );

                state.record_attempt(error);
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

### 3. Rate Limit Handler (src/error/rate_limit.rs)

```rust
//! Rate limit handling for Claude API.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Rate limiter for Claude API requests.
#[derive(Debug)]
pub struct RateLimiter {
    /// Maximum requests per minute.
    requests_per_minute: u32,
    /// Maximum tokens per minute.
    tokens_per_minute: u32,
    /// Request semaphore.
    request_semaphore: Arc<Semaphore>,
    /// Current request count.
    request_count: AtomicU32,
    /// Current token count.
    token_count: AtomicU64,
    /// Window start time.
    window_start: std::sync::Mutex<Instant>,
    /// Backoff until this time (if rate limited).
    backoff_until: std::sync::Mutex<Option<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(requests_per_minute: u32, tokens_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            tokens_per_minute,
            request_semaphore: Arc::new(Semaphore::new(requests_per_minute as usize)),
            request_count: AtomicU32::new(0),
            token_count: AtomicU64::new(0),
            window_start: std::sync::Mutex::new(Instant::now()),
            backoff_until: std::sync::Mutex::new(None),
        }
    }

    /// Create with default Claude limits.
    pub fn default_claude() -> Self {
        // Conservative defaults - actual limits vary by tier
        Self::new(50, 40_000)
    }

    /// Check if we should wait before making a request.
    pub fn should_wait(&self) -> Option<Duration> {
        // Check if in backoff period
        if let Some(until) = *self.backoff_until.lock().unwrap() {
            if Instant::now() < until {
                return Some(until - Instant::now());
            }
        }

        // Check request count
        let count = self.request_count.load(Ordering::Relaxed);
        if count >= self.requests_per_minute {
            let window_start = *self.window_start.lock().unwrap();
            let elapsed = window_start.elapsed();
            if elapsed < Duration::from_secs(60) {
                return Some(Duration::from_secs(60) - elapsed);
            }
        }

        None
    }

    /// Wait for rate limit if necessary.
    pub async fn wait_if_needed(&self) {
        if let Some(delay) = self.should_wait() {
            info!(delay_ms = delay.as_millis(), "Waiting for rate limit");
            tokio::time::sleep(delay).await;
            self.reset_window_if_needed();
        }
    }

    /// Acquire a permit for a request.
    pub async fn acquire(&self) -> RateLimitPermit {
        self.wait_if_needed().await;

        let _permit = self.request_semaphore.clone().acquire_owned().await.unwrap();
        self.request_count.fetch_add(1, Ordering::Relaxed);

        RateLimitPermit {
            limiter: self,
            tokens_used: 0,
        }
    }

    /// Record token usage.
    pub fn record_tokens(&self, tokens: u64) {
        self.token_count.fetch_add(tokens, Ordering::Relaxed);
    }

    /// Set backoff after rate limit error.
    pub fn set_backoff(&self, duration: Duration) {
        *self.backoff_until.lock().unwrap() = Some(Instant::now() + duration);
        warn!(duration_secs = duration.as_secs(), "Rate limit backoff set");
    }

    /// Reset the window if a minute has passed.
    fn reset_window_if_needed(&self) {
        let mut window_start = self.window_start.lock().unwrap();
        if window_start.elapsed() >= Duration::from_secs(60) {
            *window_start = Instant::now();
            self.request_count.store(0, Ordering::Relaxed);
            self.token_count.store(0, Ordering::Relaxed);
            debug!("Rate limit window reset");
        }
    }

    /// Get current usage stats.
    pub fn current_usage(&self) -> RateLimitUsage {
        RateLimitUsage {
            requests: self.request_count.load(Ordering::Relaxed),
            requests_limit: self.requests_per_minute,
            tokens: self.token_count.load(Ordering::Relaxed),
            tokens_limit: self.tokens_per_minute as u64,
        }
    }
}

/// A permit for a rate-limited request.
pub struct RateLimitPermit<'a> {
    limiter: &'a RateLimiter,
    tokens_used: u64,
}

impl<'a> RateLimitPermit<'a> {
    /// Record tokens used by this request.
    pub fn record_tokens(&mut self, tokens: u64) {
        self.tokens_used = tokens;
        self.limiter.record_tokens(tokens);
    }
}

/// Current rate limit usage.
#[derive(Debug, Clone)]
pub struct RateLimitUsage {
    pub requests: u32,
    pub requests_limit: u32,
    pub tokens: u64,
    pub tokens_limit: u64,
}

impl RateLimitUsage {
    /// Get request usage as percentage.
    pub fn request_usage_percent(&self) -> f32 {
        self.requests as f32 / self.requests_limit as f32 * 100.0
    }

    /// Get token usage as percentage.
    pub fn token_usage_percent(&self) -> f32 {
        self.tokens as f32 / self.tokens_limit as f32 * 100.0
    }
}
```

### 4. Error Recovery (src/error/recovery.rs)

```rust
//! Error recovery strategies.

use tachikoma_backends_core::BackendError;
use tracing::{debug, info, warn};

/// Recovery action to take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Retry the request.
    Retry,
    /// Switch to a fallback model.
    Fallback,
    /// Reduce request size (e.g., truncate context).
    ReduceSize,
    /// Refresh authentication.
    RefreshAuth,
    /// Wait and retry.
    WaitAndRetry(std::time::Duration),
    /// Abort the operation.
    Abort,
}

/// Determine recovery action for an error.
pub fn determine_recovery(error: &BackendError, context: &RecoveryContext) -> RecoveryAction {
    match error {
        BackendError::RateLimit { retry_after, .. } => {
            if let Some(delay) = retry_after {
                RecoveryAction::WaitAndRetry(*delay)
            } else {
                RecoveryAction::WaitAndRetry(std::time::Duration::from_secs(60))
            }
        }

        BackendError::ServiceUnavailable { retry_after, .. } => {
            if context.fallback_available {
                RecoveryAction::Fallback
            } else if let Some(delay) = retry_after {
                RecoveryAction::WaitAndRetry(*delay)
            } else {
                RecoveryAction::Retry
            }
        }

        BackendError::Authentication(_) => {
            if context.can_refresh_auth {
                RecoveryAction::RefreshAuth
            } else {
                RecoveryAction::Abort
            }
        }

        BackendError::InvalidRequest(msg) => {
            if msg.contains("context") || msg.contains("tokens") {
                if context.can_reduce_size {
                    RecoveryAction::ReduceSize
                } else {
                    RecoveryAction::Abort
                }
            } else {
                RecoveryAction::Abort
            }
        }

        BackendError::Network(_) => RecoveryAction::Retry,

        BackendError::Api { status, .. } if *status >= 500 => {
            if context.fallback_available {
                RecoveryAction::Fallback
            } else {
                RecoveryAction::Retry
            }
        }

        _ => RecoveryAction::Abort,
    }
}

/// Context for recovery decisions.
#[derive(Debug, Default)]
pub struct RecoveryContext {
    /// Whether a fallback model is available.
    pub fallback_available: bool,
    /// Whether we can reduce request size.
    pub can_reduce_size: bool,
    /// Whether we can refresh authentication.
    pub can_refresh_auth: bool,
    /// Number of retries already attempted.
    pub retry_count: u32,
    /// Maximum retries allowed.
    pub max_retries: u32,
}

impl RecoveryContext {
    /// Create with defaults.
    pub fn new() -> Self {
        Self {
            fallback_available: false,
            can_reduce_size: true,
            can_refresh_auth: false,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Check if more retries are allowed.
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count.
    pub fn record_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Execute with automatic recovery.
pub async fn with_recovery<F, Fut, T>(
    mut context: RecoveryContext,
    operation: F,
) -> Result<T, BackendError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, BackendError>>,
{
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                let action = determine_recovery(&error, &context);
                debug!(error = %error, action = ?action, "Determined recovery action");

                match action {
                    RecoveryAction::Retry if context.can_retry() => {
                        context.record_retry();
                        continue;
                    }
                    RecoveryAction::WaitAndRetry(delay) if context.can_retry() => {
                        context.record_retry();
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    RecoveryAction::Abort | _ => {
                        return Err(error);
                    }
                }
            }
        }
    }
}
```

### 5. Module Exports (src/error/mod.rs)

```rust
//! Error handling for Claude backend.

mod rate_limit;
mod recovery;
mod retry;
mod types;

pub use rate_limit::{RateLimiter, RateLimitPermit, RateLimitUsage};
pub use recovery::{determine_recovery, with_recovery, RecoveryAction, RecoveryContext};
pub use retry::{with_retry, RetryConfig, RetryState};
pub use types::{to_backend_error, ClaudeApiError, ClaudeErrorDetail, ClaudeErrorType};
```

---

## Testing Requirements

1. API error parsing handles all error types
2. Retry logic respects max retries
3. Rate limiter tracks usage correctly
4. Recovery actions are appropriate
5. Backoff delays are calculated correctly

---

## Related Specs

- Depends on: [056-claude-api-client.md](056-claude-api-client.md)
- Next: [061-codex-api-client.md](061-codex-api-client.md)
- Used by: All Claude API interactions
