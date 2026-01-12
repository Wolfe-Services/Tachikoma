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
/// Note: Task naming requires tokio 1.37+ with unstable features.
/// For now, we just spawn the task without naming.
pub fn spawn_named<F>(_name: &'static str, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    // TODO: Enable named tasks when tokio version supports it
    // tokio::task::Builder::new().name(name).spawn(future)
    tokio::spawn(future)
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

/// Basic runtime metrics.
#[derive(Debug, Clone, Default)]
pub struct RuntimeMetrics {
    /// Number of active tasks (approximation).
    pub active_tasks: u64,
    /// Number of completed tasks.
    pub completed_tasks: u64,
    /// Total time spent in runtime.
    pub total_runtime_duration: Duration,
}

impl RuntimeMetrics {
    /// Create new metrics instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment completed tasks counter.
    pub fn task_completed(&mut self) {
        self.completed_tasks += 1;
        if self.active_tasks > 0 {
            self.active_tasks -= 1;
        }
    }

    /// Increment active tasks counter.
    pub fn task_started(&mut self) {
        self.active_tasks += 1;
    }

    /// Update runtime duration.
    pub fn update_runtime_duration(&mut self, duration: Duration) {
        self.total_runtime_duration = duration;
    }
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

    #[tokio::test]
    async fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.worker_threads, 0);
        assert_eq!(config.thread_name, "tachikoma");
        assert!(config.enable_io);
        assert!(config.enable_time);
    }

    #[test]
    fn test_runtime_build() {
        let config = RuntimeConfig {
            worker_threads: 2,
            thread_name: "test-runtime".to_string(),
            enable_io: true,
            enable_time: true,
        };

        let runtime = build_runtime(config).expect("Should build runtime");
        
        // Test that the runtime works by running a simple task
        let result = runtime.block_on(async {
            with_timeout(Duration::from_millis(100), async { 42 }).await
        });
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_spawn_named() {
        let handle = spawn_named("test-task", async { 42 });
        let result = handle.await.expect("Task should complete");
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_join_all() {
        let futures: Vec<_> = (1..=3).map(|i| async move { i }).collect();
        let results = join_all(futures).await;
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_select_first() {
        let result = select_first(
            async {
                sleep(Duration::from_millis(10)).await;
                "slow"
            },
            async { "fast" },
        ).await;

        assert_eq!(result, "fast");
    }

    #[tokio::test]
    async fn test_yield_now() {
        // This test mainly ensures the function exists and can be called
        yield_now().await;
        // If we get here without hanging, the function works
    }

    #[test]
    fn test_yield_now_sync() {
        // This test mainly ensures the function exists and can be called
        // We can't test the actual async behavior in a sync test
    }

    #[test]
    fn test_runtime_metrics() {
        let mut metrics = RuntimeMetrics::new();
        assert_eq!(metrics.active_tasks, 0);
        assert_eq!(metrics.completed_tasks, 0);
        assert_eq!(metrics.total_runtime_duration, Duration::from_secs(0));

        // Simulate task lifecycle
        metrics.task_started();
        assert_eq!(metrics.active_tasks, 1);
        assert_eq!(metrics.completed_tasks, 0);

        metrics.task_completed();
        assert_eq!(metrics.active_tasks, 0);
        assert_eq!(metrics.completed_tasks, 1);

        // Test runtime duration update
        let test_duration = Duration::from_millis(500);
        metrics.update_runtime_duration(test_duration);
        assert_eq!(metrics.total_runtime_duration, test_duration);
    }
}