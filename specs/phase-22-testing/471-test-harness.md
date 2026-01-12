# 471 - Test Harness Setup

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 471
**Status:** Planned
**Dependencies:** 008-test-infrastructure, 002-rust-workspace
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Establish a comprehensive test harness that unifies Rust, TypeScript, and end-to-end testing into a cohesive framework with shared utilities, consistent conventions, and centralized configuration.

---

## Acceptance Criteria

- [ ] Unified test configuration across all crates and packages
- [ ] Shared test utilities accessible from all test modules
- [ ] Consistent test naming and organization conventions
- [ ] Test isolation mechanisms prevent cross-test contamination
- [ ] Parallel test execution with proper resource management
- [ ] Test categorization (unit, integration, e2e) with selective execution

---

## Implementation Details

### 1. Test Harness Core Configuration

Create `crates/tachikoma-test-harness/Cargo.toml`:

```toml
[package]
name = "tachikoma-test-harness"
version.workspace = true
edition.workspace = true
description = "Unified test harness for Tachikoma"

[dependencies]
# Test utilities
tempfile = "3.9"
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "sync", "time"] }
once_cell = "1.19"
parking_lot = "0.12"

# Test frameworks
proptest = "1.4"
insta = { version = "1.34", features = ["yaml", "json"] }
test-case = "3.3"
mockall = "0.12"
tokio-test = "0.4"
wiremock = "0.6"
fake = { version = "2.9", features = ["derive"] }
rstest = "0.18"

# Assertions
pretty_assertions = "1.4"
assert_matches = "1.5"

# Tracing for tests
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
```

### 2. Test Harness Core Module

Create `crates/tachikoma-test-harness/src/lib.rs`:

```rust
//! Tachikoma Test Harness
//!
//! Provides unified testing utilities, fixtures, and conventions for all
//! Tachikoma crates. This harness ensures consistent testing patterns and
//! shared infrastructure across the workspace.

pub mod assertions;
pub mod context;
pub mod fixtures;
pub mod generators;
pub mod isolation;
pub mod macros;
pub mod mocks;
pub mod reporters;
pub mod setup;

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
```

### 3. Test Context and Isolation

Create `crates/tachikoma-test-harness/src/context.rs`:

```rust
//! Test context management for isolated test environments.

use std::path::PathBuf;
use tempfile::TempDir;

/// An isolated test context with its own temporary directory and resources.
pub struct TestContext {
    id: String,
    temp_dir: TempDir,
    cleanup_handlers: Vec<Box<dyn FnOnce() + Send>>,
}

impl TestContext {
    /// Create a new test context
    pub fn new() -> Self {
        let id = crate::unique_test_id();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        crate::register_context(&id);

        Self {
            id,
            temp_dir,
            cleanup_handlers: Vec::new(),
        }
    }

    /// Get the test context ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a file in the temp directory
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        std::fs::write(&path, content).expect("Failed to write file");
        path
    }

    /// Create a directory in the temp directory
    pub fn create_dir(&self, name: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&path).expect("Failed to create directory");
        path
    }

    /// Register a cleanup handler
    pub fn on_cleanup<F: FnOnce() + Send + 'static>(&mut self, handler: F) {
        self.cleanup_handlers.push(Box::new(handler));
    }

    /// Get a unique path within the temp directory
    pub fn unique_path(&self, prefix: &str) -> PathBuf {
        self.temp_dir.path().join(format!("{}_{}", prefix, crate::unique_test_id()))
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Run cleanup handlers
        for handler in self.cleanup_handlers.drain(..) {
            handler();
        }
        crate::deregister_context(&self.id);
    }
}

/// Async test context with tokio runtime support
pub struct AsyncTestContext {
    inner: TestContext,
    runtime: Option<tokio::runtime::Runtime>,
}

impl AsyncTestContext {
    /// Create a new async test context
    pub fn new() -> Self {
        Self {
            inner: TestContext::new(),
            runtime: None,
        }
    }

    /// Get or create the tokio runtime
    pub fn runtime(&mut self) -> &tokio::runtime::Runtime {
        self.runtime.get_or_insert_with(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime")
        })
    }

    /// Access the inner test context
    pub fn inner(&self) -> &TestContext {
        &self.inner
    }

    /// Access the inner test context mutably
    pub fn inner_mut(&mut self) -> &mut TestContext {
        &mut self.inner
    }
}

impl Default for AsyncTestContext {
    fn default() -> Self {
        Self::new()
    }
}
```

### 4. Test Runner Script

Create `scripts/test-harness.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${GREEN}[TEST]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Test categories
CATEGORY="${1:-all}"

case "$CATEGORY" in
    unit)
        log "Running unit tests..."
        cargo test --workspace --lib
        cd web && npm run test:unit
        ;;
    integration)
        log "Running integration tests..."
        cargo test --workspace --test '*'
        cd web && npm run test:integration
        ;;
    e2e)
        log "Running end-to-end tests..."
        cd web && npm run test:e2e
        ;;
    rust)
        log "Running all Rust tests..."
        cargo test --workspace
        ;;
    ts)
        log "Running all TypeScript tests..."
        cd web && npm test
        ;;
    all)
        log "Running all tests..."
        cargo test --workspace
        cd web && npm test
        ;;
    *)
        error "Unknown category: $CATEGORY"
        echo "Usage: $0 [unit|integration|e2e|rust|ts|all]"
        exit 1
        ;;
esac

log "Tests completed!"
```

---

## Testing Requirements

1. `cargo test -p tachikoma-test-harness` passes
2. Test context provides proper isolation
3. Cleanup handlers execute correctly
4. Parallel tests do not interfere with each other
5. Test IDs are unique across all test runs

---

## Related Specs

- Depends on: [008-test-infrastructure.md](../phase-00-setup/008-test-infrastructure.md), [002-rust-workspace.md](../phase-00-setup/002-rust-workspace.md)
- Next: [472-unit-patterns.md](472-unit-patterns.md)
- Related: [481-test-coverage.md](481-test-coverage.md)
