//! Custom assertions and assertion helpers for testing.

pub use assert_matches::assert_matches;
pub use pretty_assertions::{assert_eq, assert_ne};

/// Assert that a result is an error with a specific message
#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $expected:expr) => {
        match $result {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => {
                let error_msg = format!("{}", e);
                assert!(
                    error_msg.contains($expected),
                    "Expected error message to contain '{}', but got: '{}'",
                    $expected,
                    error_msg
                );
            }
        }
    };
}

/// Assert that two values are approximately equal (for floating point comparisons)
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $tolerance:expr) => {
        let diff = ($left - $right).abs();
        assert!(
            diff <= $tolerance,
            "Values are not approximately equal: {} and {} (diff: {}, tolerance: {})",
            $left,
            $right,
            diff,
            $tolerance
        );
    };
}

/// Assert that an async operation completes within a timeout
#[macro_export]
macro_rules! assert_timeout {
    ($duration:expr, $future:expr) => {
        tokio::time::timeout($duration, $future)
            .await
            .expect("Operation timed out")
    };
}

/// Assert that a future is pending (doesn't complete immediately)
#[macro_export]
macro_rules! assert_pending {
    ($future:expr) => {
        let mut future = std::pin::Pin::new(&mut $future);
        let waker = futures_util::task::noop_waker();
        let mut context = std::task::Context::from_waker(&waker);
        
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Pending => {},
            std::task::Poll::Ready(_) => panic!("Expected future to be pending, but it completed"),
        }
    };
}

/// Assert that a value matches a predicate
pub fn assert_that<T>(value: T, predicate: impl Fn(&T) -> bool, message: &str) {
    assert!(predicate(&value), "{}", message);
}

/// Assert that all values in a collection satisfy a predicate
pub fn assert_all<T>(
    values: impl IntoIterator<Item = T>,
    predicate: impl Fn(&T) -> bool,
    message: &str,
) {
    for value in values {
        assert!(predicate(&value), "{}", message);
    }
}

/// Assert that any value in a collection satisfies a predicate
pub fn assert_any<T>(
    values: impl IntoIterator<Item = T>,
    predicate: impl Fn(&T) -> bool,
    message: &str,
) {
    let mut found = false;
    for value in values {
        if predicate(&value) {
            found = true;
            break;
        }
    }
    assert!(found, "{}", message);
}