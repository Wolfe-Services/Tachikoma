//! Tachikoma Test Harness
//!
//! Provides unified testing utilities, fixtures, and conventions for all
//! Tachikoma crates. This harness ensures consistent testing patterns and
//! shared infrastructure across the workspace.

pub mod assertions;
pub mod context;
pub mod fixtures;
pub mod flaky;
pub mod generators;
pub mod isolation;
pub mod macros;
pub mod mocks;
pub mod patterns;
pub mod reporters;
pub mod setup;
pub mod snapshot;

// Re-export snapshot macros
pub use snapshot::*;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing_subscriber::EnvFilter;

/// Global test counter for unique resource naming
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Set of active test contexts for cleanup tracking
static ACTIVE_CONTEXTS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Initialize the test harness with tracing support
pub fn init() {
    static INIT: Lazy<()> = Lazy::new(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("warn,tachikoma=debug"));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_test_writer()
            .with_file(true)
            .with_line_number(true)
            .try_init()
            .ok();
    });

    Lazy::force(&INIT);
}

/// Generate a unique test ID
pub fn unique_test_id() -> String {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test_{}_{}", timestamp, count)
}

/// Register an active test context
pub fn register_context(id: &str) {
    ACTIVE_CONTEXTS.lock().insert(id.to_string());
}

/// Deregister a test context (for cleanup)
pub fn deregister_context(id: &str) {
    ACTIVE_CONTEXTS.lock().remove(id);
}

/// Get count of active test contexts
pub fn active_context_count() -> usize {
    ACTIVE_CONTEXTS.lock().len()
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Whether to enable verbose output
    pub verbose: bool,
    /// Test timeout in seconds
    pub timeout_secs: u64,
    /// Whether to capture stdout/stderr
    pub capture_output: bool,
    /// Number of retries for flaky tests
    pub retries: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            verbose: std::env::var("TEST_VERBOSE").is_ok(),
            timeout_secs: 30,
            capture_output: true,
            retries: 0,
        }
    }
}