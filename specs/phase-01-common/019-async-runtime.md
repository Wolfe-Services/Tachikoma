# 019 - Async Runtime Setup

**Phase:** 1 - Core Common Crates
**Spec ID:** 019
**Status:** Planned
**Dependencies:** 002-rust-workspace
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure the Tokio async runtime with proper settings for the Tachikoma application, including runtime builders, task spawning utilities, and graceful shutdown.

---

## Acceptance Criteria

- [x] Tokio runtime configuration
- [x] Runtime builder helpers
- [x] Task spawning utilities
- [x] Graceful shutdown support
- [x] Runtime metrics

---

## Implementation Details

### 1. Runtime Module (crates/tachikoma-common-async/src/lib.rs)

```rust
//! Async runtime utilities.

use std::future::Future;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::broadcast;

/// Configuration for the Tachikoma runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Number of worker threads (0 = num_cpus).
    pub worker_threads: usize,
    /// Thread name prefix.
    pub thread_name: String,
    /// Enable I/O driver.
    pub enable_io: bool,
    /// Enable time driver.
    pub enable_time: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: 0, // Use num_cpus
            thread_name: "tachikoma".to_string(),
            enable_io: true,
            enable_time: true,
        }
    }
}

/// Build a configured Tokio runtime.
pub fn build_runtime(config: RuntimeConfig) -> std::io::Result<Runtime> {
    let mut builder = Builder::new_multi_thread();

    if config.worker_threads > 0 {
        builder.worker_threads(config.worker_threads);
    }

    builder.thread_name(&config.thread_name);

    if config.enable_io {
        builder.enable_io();
    }

    if config.enable_time {
        builder.enable_time();
    }

    builder.build()
}

/// A handle for coordinating graceful shutdown.
#[derive(Clone)]
pub struct ShutdownHandle {
    sender: broadcast::Sender<()>,
}

impl ShutdownHandle {
    /// Create a new shutdown handle.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self { sender }
    }

    /// Get a receiver for shutdown signals.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    /// Signal shutdown to all receivers.
    pub fn shutdown(&self) {
        let _ = self.sender.send(());
    }
}

impl Default for ShutdownHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a future with a timeout.
pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| TimeoutError)
}

/// Timeout error.
#[derive(Debug, Clone, thiserror::Error)]
#[error("operation timed out")]
pub struct TimeoutError;

/// Spawn a task with a name for debugging.
pub fn spawn_named<F>(name: &'static str, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::Builder::new()
        .name(name)
        .spawn(future)
        .expect("failed to spawn task")
}

/// Run multiple futures concurrently, returning when all complete.
pub async fn join_all<I, F, T>(futures: I) -> Vec<T>
where
    I: IntoIterator<Item = F>,
    F: Future<Output = T>,
{
    futures::future::join_all(futures).await
}

/// Run multiple futures concurrently, returning when the first completes.
pub async fn select_first<F1, F2, T>(f1: F1, f2: F2) -> T
where
    F1: Future<Output = T>,
    F2: Future<Output = T>,
{
    tokio::select! {
        v = f1 => v,
        v = f2 => v,
    }
}

/// Sleep for a duration.
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await
}

/// Yield to the runtime.
pub async fn yield_now() {
    tokio::task::yield_now().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_success() {
        let result = with_timeout(Duration::from_secs(1), async { 42 }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_failure() {
        let result = with_timeout(Duration::from_millis(10), async {
            sleep(Duration::from_secs(1)).await;
            42
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shutdown_handle() {
        let handle = ShutdownHandle::new();
        let mut rx = handle.subscribe();

        tokio::spawn({
            let handle = handle.clone();
            async move {
                sleep(Duration::from_millis(10)).await;
                handle.shutdown();
            }
        });

        let _ = rx.recv().await;
    }
}
```

### 2. Crate Setup

```toml
[package]
name = "tachikoma-common-async"
version.workspace = true
edition.workspace = true

[dependencies]
tokio = { workspace = true, features = ["full", "tracing"] }
futures = "0.3"
thiserror.workspace = true
```

---

## Testing Requirements

1. Runtime builds with default config
2. Timeout properly cancels slow futures
3. Shutdown signal propagates to all subscribers
4. Named tasks appear in debugger

---

## Related Specs

- Depends on: [002-rust-workspace.md](../phase-00-setup/002-rust-workspace.md)
- Next: [020-http-client-foundation.md](020-http-client-foundation.md)
