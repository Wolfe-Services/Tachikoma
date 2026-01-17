//! Test setup and initialization utilities.

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the test environment with common setup
pub fn init_test_environment() {
    INIT.call_once(|| {
        crate::init();
        
        // Set common environment variables for tests
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("RUST_BACKTRACE", "1");
        
        // Disable colored output in tests for consistency
        std::env::set_var("NO_COLOR", "1");
        
        // Set test-specific configuration
        std::env::set_var("TACHIKOMA_TEST_MODE", "true");
    });
}

/// Macro to automatically initialize test environment
#[macro_export]
macro_rules! test_init {
    () => {
        $crate::setup::init_test_environment();
    };
}

/// Test case wrapper that handles common setup
pub fn test_case<F, R>(name: &str, test_fn: F) -> R
where
    F: FnOnce() -> R,
{
    init_test_environment();
    tracing::info!("Starting test: {}", name);
    let start = std::time::Instant::now();
    
    let result = test_fn();
    
    let duration = start.elapsed();
    tracing::info!("Test {} completed in {:?}", name, duration);
    
    result
}

/// Async test case wrapper
pub async fn async_test_case<F, Fut, R>(name: &str, test_fn: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    init_test_environment();
    tracing::info!("Starting async test: {}", name);
    let start = std::time::Instant::now();
    
    let result = test_fn().await;
    
    let duration = start.elapsed();
    tracing::info!("Async test {} completed in {:?}", name, duration);
    
    result
}