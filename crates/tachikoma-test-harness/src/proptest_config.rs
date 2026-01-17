//! Property testing configuration and utilities using proptest.
//!
//! Property-based testing generates many random test cases to verify
//! that invariants hold. When a test fails, proptest automatically
//! shrinks the input to find the minimal failing case.

use proptest::prelude::*;
use proptest::test_runner::Config;

/// Standard proptest configuration for Tachikoma
pub fn standard_config() -> Config {
    Config {
        // Number of successful tests before passing
        cases: 256,
        // Maximum shrink iterations
        max_shrink_iters: 10_000,
        // Save failing cases for regression testing
        failure_persistence: Some(Box::new(
            proptest::test_runner::FileFailurePersistence::WithSource("proptest-regressions"),
        )),
        // Timeout per test case
        timeout: 1_000,
        ..Config::default()
    }
}

/// Quick configuration for development (fewer cases)
pub fn quick_config() -> Config {
    Config {
        cases: 32,
        max_shrink_iters: 1_000,
        ..standard_config()
    }
}

/// Thorough configuration for CI (more cases)
pub fn ci_config() -> Config {
    Config {
        cases: 1_024,
        max_shrink_iters: 50_000,
        ..standard_config()
    }
}

/// Get configuration based on environment
pub fn env_config() -> Config {
    match std::env::var("PROPTEST_CASES").ok().as_deref() {
        Some("quick") => quick_config(),
        Some("ci") => ci_config(),
        _ => standard_config(),
    }
}