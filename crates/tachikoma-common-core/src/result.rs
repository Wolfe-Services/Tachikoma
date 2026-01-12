//! Result and Option extensions.

use crate::error::{Error, Result};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;
    use std::io;

    #[test]
    fn test_result_context() {
        let io_result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let with_context = io_result.context("Failed to read config file");

        assert!(with_context.is_err());
        let err = with_context.unwrap_err();
        match err {
            Error::Internal { message, source } => {
                assert_eq!(message, "Failed to read config file");
                assert!(source.is_some());
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_result_with_context() {
        let io_result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"));
        let with_context = io_result.with_context(|| format!("Failed to write to file at {}", "/tmp/test.txt"));

        assert!(with_context.is_err());
        let err = with_context.unwrap_err();
        match err {
            Error::Internal { message, source } => {
                assert_eq!(message, "Failed to write to file at /tmp/test.txt");
                assert!(source.is_some());
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_result_map_err_to() {
        let io_result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::InvalidData, "corrupted"));
        let mapped = io_result.map_err_to(|| Error::validation("Invalid file format"));

        assert!(mapped.is_err());
        let err = mapped.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert_eq!(message, "Invalid file format");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_option_ok_or_err() {
        let none_option: Option<String> = None;
        let result = none_option.ok_or_err("Value was None");

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::Internal { message, source } => {
                assert_eq!(message, "Value was None");
                assert!(source.is_none());
            }
            _ => panic!("Expected Internal error"),
        }

        // Test Some case
        let some_option = Some("value".to_string());
        let result = some_option.ok_or_err("Should not happen");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "value");
    }

    #[test]
    fn test_option_ok_or_not_found() {
        let none_option: Option<String> = None;
        let result = none_option.ok_or_not_found("/path/to/file.txt");

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::FileSystem { code, message, path, .. } => {
                assert_eq!(code, ErrorCode::FILE_NOT_FOUND);
                assert!(message.contains("file not found"));
                assert_eq!(path, Some("/path/to/file.txt".to_string()));
            }
            _ => panic!("Expected FileSystem error"),
        }
    }

    #[test]
    fn test_option_ok_or_invalid() {
        let none_option: Option<i32> = None;
        let result = none_option.ok_or_invalid("Number is required");

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::Validation { code, message, .. } => {
                assert_eq!(code, ErrorCode::VALIDATION_FAILED);
                assert_eq!(message, "Number is required");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        use std::cell::Cell;
        use std::rc::Rc;
        
        let call_count = Rc::new(Cell::new(0));
        let operation = || {
            let count = call_count.clone();
            count.set(count.get() + 1);
            async move { Ok::<_, &'static str>("success") }
        };

        let result = retry(3, 10, operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.get(), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        use std::cell::Cell;
        use std::rc::Rc;
        
        let call_count = Rc::new(Cell::new(0));
        let operation = || {
            let count = call_count.clone();
            count.set(count.get() + 1);
            let current_count = count.get();
            async move {
                if current_count < 3 {
                    Err("temporary failure")
                } else {
                    Ok("success")
                }
            }
        };

        let result = retry(5, 1, operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.get(), 3);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_reached() {
        use std::cell::Cell;
        use std::rc::Rc;
        
        let call_count = Rc::new(Cell::new(0));
        let operation = || {
            let count = call_count.clone();
            count.set(count.get() + 1);
            async move { Err::<&str, _>("persistent failure") }
        };

        let result = retry(3, 1, operation).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "persistent failure");
        assert_eq!(call_count.get(), 3);
    }

    #[tokio::test]
    async fn test_retry_exponential_backoff() {
        use tokio::time::{Duration, Instant};

        let operation = || async { Err::<(), _>("always fails") };
        
        let start = Instant::now();
        let _ = retry(3, 10, operation).await;
        let elapsed = start.elapsed();

        // Should wait at least 10ms + 20ms = 30ms for 3 attempts
        // (first attempt immediate, then 10ms wait, then 20ms wait)
        assert!(elapsed >= Duration::from_millis(25)); // Allow some margin for timing
    }

    #[test]
    fn test_ensure_macro() {
        fn test_function(value: i32) -> Result<()> {
            ensure!(value > 0, "Value must be positive");
            Ok(())
        }

        // Test success case
        assert!(test_function(5).is_ok());

        // Test failure case
        let result = test_function(-1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert_eq!(message, "Value must be positive");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_ensure_macro_with_code() {
        fn test_function(name: &str) -> Result<()> {
            ensure!(!name.is_empty(), ErrorCode::VALIDATION_FAILED, "Name cannot be empty");
            Ok(())
        }

        // Test success case
        assert!(test_function("valid").is_ok());

        // Test failure case
        let result = test_function("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::Validation { code, message, .. } => {
                assert_eq!(code, ErrorCode::VALIDATION_FAILED);
                assert_eq!(message, "Name cannot be empty");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_bail_macro() {
        fn test_function(should_fail: bool) -> Result<String> {
            if should_fail {
                bail!("Something went wrong");
            }
            Ok("success".to_string())
        }

        // Test success case
        let result = test_function(false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Test failure case
        let result = test_function(true);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::Internal { message, source } => {
                assert_eq!(message, "Something went wrong");
                assert!(source.is_none());
            }
            _ => panic!("Expected Internal error"),
        }
    }
}