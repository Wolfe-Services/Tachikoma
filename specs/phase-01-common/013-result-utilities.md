# 013 - Result Utilities

**Phase:** 1 - Core Common Crates
**Spec ID:** 013
**Status:** Planned
**Dependencies:** 012-error-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Provide utility traits and extension methods for Result and Option types to simplify error handling patterns.

---

## Acceptance Criteria

- [ ] ResultExt trait for context addition
- [ ] OptionExt trait for conversion to Result
- [ ] Retry utilities for fallible operations
- [ ] Helper macros for common patterns

---

## Implementation Details

### 1. Result Extensions (src/result.rs)

```rust
//! Result and Option extensions.

use crate::error::{Error, ErrorCode, Result};

/// Extension trait for Result types.
pub trait ResultExt<T> {
    /// Add context to an error.
    fn context(self, message: impl Into<String>) -> Result<T>;

    /// Add context with a closure (lazy evaluation).
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Convert to a different error type.
    fn map_err_to<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> Error;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T>
    for std::result::Result<T, E>
{
    fn context(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|e| Error::Internal {
            message: message.into(),
            source: Some(Box::new(e)),
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| Error::Internal {
            message: f(),
            source: Some(Box::new(e)),
        })
    }

    fn map_err_to<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> Error,
    {
        self.map_err(|_| f())
    }
}

/// Extension trait for Option types.
pub trait OptionExt<T> {
    /// Convert None to an error.
    fn ok_or_err(self, message: impl Into<String>) -> Result<T>;

    /// Convert None to a file not found error.
    fn ok_or_not_found(self, path: impl Into<String>) -> Result<T>;

    /// Convert None to a validation error.
    fn ok_or_invalid(self, message: impl Into<String>) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_err(self, message: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| Error::Internal {
            message: message.into(),
            source: None,
        })
    }

    fn ok_or_not_found(self, path: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| Error::file_not_found(path))
    }

    fn ok_or_invalid(self, message: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| Error::validation(message))
    }
}

/// Retry a fallible operation with exponential backoff.
pub async fn retry<T, E, F, Fut>(
    max_attempts: u32,
    initial_delay_ms: u64,
    operation: F,
) -> std::result::Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
{
    let mut delay = initial_delay_ms;

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) if attempt == max_attempts => return Err(e),
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}

/// Ensure macro - return early if condition is false.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($crate::error::Error::validation($msg));
        }
    };
    ($cond:expr, $code:expr, $msg:expr) => {
        if !$cond {
            return Err($crate::error::Error::Validation {
                code: $code,
                message: $msg.into(),
                field: None,
            });
        }
    };
}

/// Bail macro - return error immediately.
#[macro_export]
macro_rules! bail {
    ($msg:expr) => {
        return Err($crate::error::Error::Internal {
            message: $msg.into(),
            source: None,
        })
    };
}
```

---

## Testing Requirements

1. `context()` preserves original error as source
2. `ok_or_err()` converts None to correct error type
3. Retry respects max attempts
4. Retry implements exponential backoff

---

## Related Specs

- Depends on: [012-error-types.md](012-error-types.md)
- Next: [014-config-core-types.md](014-config-core-types.md)
