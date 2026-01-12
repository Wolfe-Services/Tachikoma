//! Token bucket rate limiting algorithm.

use std::time::{Duration, Instant};

/// Token bucket for rate limiting.
pub struct TokenBucket {
    /// Maximum tokens (burst size).
    capacity: u64,
    /// Current token count.
    tokens: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Last refill time.
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket.
    pub fn new(tokens_per_second: u64, burst_size: u64) -> Self {
        Self {
            capacity: burst_size,
            tokens: burst_size as f64,
            refill_rate: tokens_per_second as f64,
            last_refill: Instant::now(),
        }
    }

    /// Try to acquire a token.
    pub fn try_acquire(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Try to acquire multiple tokens.
    pub fn try_acquire_n(&mut self, n: u64) -> bool {
        self.refill();

        let n = n as f64;
        if self.tokens >= n {
            self.tokens -= n;
            true
        } else {
            false
        }
    }

    /// Get available tokens.
    pub fn available_tokens(&self) -> u64 {
        self.tokens as u64
    }

    /// Time until next token available.
    pub fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            let seconds = needed / self.refill_rate;
            Duration::from_secs_f64(seconds)
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = elapsed.as_secs_f64() * self.refill_rate;

        self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
        self.last_refill = now;
    }

    /// Reset the bucket to full.
    pub fn reset(&mut self) {
        self.tokens = self.capacity as f64;
        self.last_refill = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10, 5);

        // Start with burst capacity
        assert_eq!(bucket.available_tokens(), 5);

        // Use tokens
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert_eq!(bucket.available_tokens(), 3);
    }

    #[test]
    fn test_token_bucket_exhaustion() {
        let mut bucket = TokenBucket::new(1, 2);

        // Use all tokens
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10, 5);

        // Use all tokens
        for _ in 0..5 {
            assert!(bucket.try_acquire());
        }
        assert!(!bucket.try_acquire());

        // Wait for refill (100ms = 1 token at 10/s)
        sleep(Duration::from_millis(120));

        // Should have new token
        assert!(bucket.try_acquire());
    }

    #[test]
    fn test_acquire_multiple() {
        let mut bucket = TokenBucket::new(10, 5);

        assert!(bucket.try_acquire_n(3));
        assert_eq!(bucket.available_tokens(), 2);
        assert!(!bucket.try_acquire_n(3));
    }
}