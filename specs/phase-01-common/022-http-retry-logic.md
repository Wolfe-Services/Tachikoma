# 022 - HTTP Retry Logic

**Phase:** 1 - Core Common Crates
**Spec ID:** 022
**Status:** Planned
**Dependencies:** 020-http-client-foundation
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement retry logic with exponential backoff for transient HTTP errors, rate limiting, and configurable retry policies.

---

## Acceptance Criteria

- [ ] Exponential backoff implementation
- [ ] Jitter to prevent thundering herd
- [ ] Configurable max retries
- [ ] Rate limit header parsing
- [ ] Retry-able error detection

---

## Implementation Details

### 1. Retry Module (crates/tachikoma-common-http/src/retry.rs)

```rust
//! HTTP retry logic with exponential backoff.

use std::time::Duration;
use tokio::time::sleep;

/// Retry policy configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub multiplier: f64,
    /// Add random jitter to delay.
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a policy that doesn't retry.
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            ..Default::default()
        }
    }

    /// Create an aggressive retry policy for critical operations.
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Calculate delay for a given attempt (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let base_delay = self.initial_delay.as_millis() as f64
            * self.multiplier.powi((attempt - 1) as i32);

        let delay_ms = base_delay.min(self.max_delay.as_millis() as f64);

        let delay_ms = if self.jitter {
            // Add 0-25% jitter
            let jitter = rand::random::<f64>() * 0.25;
            delay_ms * (1.0 + jitter)
        } else {
            delay_ms
        };

        Duration::from_millis(delay_ms as u64)
    }
}

/// Execute an async operation with retry.
pub async fn with_retry<F, Fut, T, E>(
    policy: &RetryPolicy,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: RetryableError,
{
    let mut last_error = None;

    for attempt in 0..policy.max_attempts {
        // Wait before retry (no wait on first attempt)
        let delay = policy.delay_for_attempt(attempt);
        if !delay.is_zero() {
            sleep(delay).await;
        }

        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if !e.is_retryable() || attempt + 1 >= policy.max_attempts {
                    return Err(e);
                }

                // Check for rate limit with specific delay
                if let Some(retry_after) = e.retry_after() {
                    sleep(retry_after).await;
                }

                last_error = Some(e);
            }
        }
    }

    Err(last_error.expect("at least one attempt should have been made"))
}

/// Trait for errors that may be retried.
pub trait RetryableError {
    /// Should this error be retried?
    fn is_retryable(&self) -> bool;

    /// Explicit retry delay from rate limiting.
    fn retry_after(&self) -> Option<Duration> {
        None
    }
}

impl RetryableError for super::HttpError {
    fn is_retryable(&self) -> bool {
        matches!(
            self,
            super::HttpError::Timeout
                | super::HttpError::RateLimited { .. }
                | super::HttpError::ServerError { status, .. } if *status >= 500
        )
    }

    fn retry_after(&self) -> Option<Duration> {
        match self {
            super::HttpError::RateLimited { retry_after } => *retry_after,
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_calculation() {
        let policy = RetryPolicy {
            initial_delay: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };

        assert_eq!(policy.delay_for_attempt(0), Duration::ZERO);
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_cap() {
        let policy = RetryPolicy {
            initial_delay: Duration::from_secs(10),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };

        // 10 * 2^5 = 320 seconds, should be capped at 30
        assert!(policy.delay_for_attempt(6) <= Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_retry_success_on_third_attempt() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            jitter: false,
            ..Default::default()
        };

        let mut attempts = 0;
        let result = with_retry(&policy, || {
            attempts += 1;
            async move {
                if attempts < 3 {
                    Err(TestError { retryable: true })
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);
    }

    #[derive(Debug)]
    struct TestError {
        retryable: bool,
    }

    impl RetryableError for TestError {
        fn is_retryable(&self) -> bool {
            self.retryable
        }
    }
}
```

### 2. Add Dependencies

```toml
[dependencies]
rand = "0.8"
```

---

## Testing Requirements

1. Backoff delays increase exponentially
2. Jitter adds randomness to delays
3. Max delay is respected
4. Non-retryable errors fail immediately

---

## Related Specs

- Depends on: [020-http-client-foundation.md](020-http-client-foundation.md)
- Next: [023-i18n-core-setup.md](023-i18n-core-setup.md)
