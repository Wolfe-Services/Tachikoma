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