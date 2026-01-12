# 334 - Graceful Shutdown

**Phase:** 15 - Server
**Spec ID:** 334
**Status:** Planned
**Dependencies:** 333-server-startup
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Implement graceful shutdown handling that properly closes connections, drains requests, and cleans up resources.

---

## Acceptance Criteria

- [ ] Signal handling (SIGTERM, SIGINT)
- [ ] Request draining
- [ ] WebSocket connection closure
- [ ] Database connection cleanup
- [ ] Background task cancellation
- [ ] Shutdown timeout
- [ ] Shutdown hooks

---

## Implementation Details

### 1. Shutdown Coordinator (crates/tachikoma-server/src/shutdown/coordinator.rs)

```rust
//! Shutdown coordination.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch};
use tracing::{info, warn};

/// Shutdown coordinator for graceful shutdown.
#[derive(Clone)]
pub struct ShutdownCoordinator {
    /// Shutdown signal sender.
    sender: broadcast::Sender<()>,
    /// Shutdown initiated flag.
    initiated: Arc<AtomicBool>,
    /// Shutdown complete notifier.
    complete_tx: Arc<watch::Sender<bool>>,
    /// Shutdown complete receiver.
    complete_rx: watch::Receiver<bool>,
    /// Shutdown timeout.
    timeout: Duration,
}

impl ShutdownCoordinator {
    pub fn new(timeout: Duration) -> Self {
        let (sender, _) = broadcast::channel(1);
        let (complete_tx, complete_rx) = watch::channel(false);

        Self {
            sender,
            initiated: Arc::new(AtomicBool::new(false)),
            complete_tx: Arc::new(complete_tx),
            complete_rx,
            timeout,
        }
    }

    /// Subscribe to shutdown signal.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    /// Check if shutdown has been initiated.
    pub fn is_shutting_down(&self) -> bool {
        self.initiated.load(Ordering::SeqCst)
    }

    /// Initiate shutdown.
    pub fn initiate(&self) {
        if self.initiated.swap(true, Ordering::SeqCst) {
            // Already initiated
            return;
        }

        info!("Initiating graceful shutdown...");
        let _ = self.sender.send(());
    }

    /// Wait for shutdown completion.
    pub async fn wait_for_completion(&self) {
        let mut rx = self.complete_rx.clone();
        let _ = rx.wait_for(|&complete| complete).await;
    }

    /// Mark shutdown as complete.
    pub fn complete(&self) {
        info!("Shutdown complete");
        let _ = self.complete_tx.send(true);
    }

    /// Get shutdown timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}
```

### 2. Shutdown Handler (crates/tachikoma-server/src/shutdown/handler.rs)

```rust
//! Shutdown handling logic.

use super::coordinator::ShutdownCoordinator;
use super::hooks::ShutdownHook;
use crate::startup::builder::AppState;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Shutdown handler managing the shutdown sequence.
pub struct ShutdownHandler {
    coordinator: ShutdownCoordinator,
    hooks: RwLock<Vec<Box<dyn ShutdownHook + Send + Sync>>>,
    state: Option<AppState>,
}

impl ShutdownHandler {
    pub fn new(coordinator: ShutdownCoordinator) -> Self {
        Self {
            coordinator,
            hooks: RwLock::new(Vec::new()),
            state: None,
        }
    }

    /// Set application state.
    pub fn with_state(mut self, state: AppState) -> Self {
        self.state = Some(state);
        self
    }

    /// Register a shutdown hook.
    pub async fn register_hook(&self, hook: Box<dyn ShutdownHook + Send + Sync>) {
        self.hooks.write().await.push(hook);
    }

    /// Execute shutdown sequence.
    pub async fn execute(&self) {
        let timeout = self.coordinator.timeout();

        // Run shutdown with timeout
        let result = tokio::time::timeout(timeout, self.shutdown_sequence()).await;

        match result {
            Ok(()) => {
                info!("Graceful shutdown completed successfully");
            }
            Err(_) => {
                warn!("Shutdown timed out after {:?}", timeout);
            }
        }

        self.coordinator.complete();
    }

    async fn shutdown_sequence(&self) {
        info!("Running shutdown sequence...");

        // Step 1: Stop accepting new requests
        info!("Stopping new request acceptance...");

        // Step 2: Close WebSocket connections
        if let Some(state) = &self.state {
            info!("Closing WebSocket connections...");
            self.close_websockets(state).await;
        }

        // Step 3: Drain existing requests
        info!("Waiting for in-flight requests to complete...");
        self.drain_requests().await;

        // Step 4: Run custom shutdown hooks
        info!("Running shutdown hooks...");
        self.run_hooks().await;

        // Step 5: Close database connections
        if let Some(state) = &self.state {
            info!("Closing database connections...");
            self.close_database(state).await;
        }

        info!("Shutdown sequence complete");
    }

    async fn close_websockets(&self, state: &AppState) {
        use crate::websocket::session::WsOutgoingMessage;

        // Broadcast close message to all WebSocket connections
        state.ws_state.session_manager
            .broadcast(WsOutgoingMessage::Close)
            .await;

        // Give connections time to close gracefully
        tokio::time::sleep(Duration::from_secs(1)).await;

        let remaining = state.ws_state.session_manager.session_count().await;
        if remaining > 0 {
            warn!("{} WebSocket connections still open", remaining);
        }
    }

    async fn drain_requests(&self) {
        // Wait for in-flight metrics to reach zero
        if let Some(state) = &self.state {
            let mut attempts = 0;
            while attempts < 30 {
                let in_flight = state.metrics.http_requests_in_flight.get();
                if in_flight == 0 {
                    info!("All in-flight requests completed");
                    return;
                }

                info!("{} requests still in flight", in_flight);
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }

            warn!("Request draining timed out");
        }
    }

    async fn run_hooks(&self) {
        let hooks = self.hooks.read().await;

        for hook in hooks.iter() {
            match hook.on_shutdown().await {
                Ok(()) => {
                    info!("Shutdown hook '{}' completed", hook.name());
                }
                Err(e) => {
                    error!("Shutdown hook '{}' failed: {}", hook.name(), e);
                }
            }
        }
    }

    async fn close_database(&self, state: &AppState) {
        // Close the database pool
        state.db_pool.close().await;
        info!("Database connections closed");
    }
}
```

### 3. Shutdown Hooks (crates/tachikoma-server/src/shutdown/hooks.rs)

```rust
//! Shutdown hook trait and implementations.

use async_trait::async_trait;
use anyhow::Result;

/// Trait for shutdown hooks.
#[async_trait]
pub trait ShutdownHook: Send + Sync {
    /// Hook name for logging.
    fn name(&self) -> &str;

    /// Called during shutdown.
    async fn on_shutdown(&self) -> Result<()>;

    /// Priority (higher runs first).
    fn priority(&self) -> i32 {
        0
    }
}

/// Shutdown hook for flushing metrics.
pub struct FlushMetricsHook;

#[async_trait]
impl ShutdownHook for FlushMetricsHook {
    fn name(&self) -> &str {
        "flush_metrics"
    }

    async fn on_shutdown(&self) -> Result<()> {
        // Flush any buffered metrics
        Ok(())
    }

    fn priority(&self) -> i32 {
        100 // Run early
    }
}

/// Shutdown hook for clearing caches.
pub struct ClearCacheHook;

#[async_trait]
impl ShutdownHook for ClearCacheHook {
    fn name(&self) -> &str {
        "clear_cache"
    }

    async fn on_shutdown(&self) -> Result<()> {
        // Clear any in-memory caches
        Ok(())
    }
}

/// Shutdown hook for notifying external services.
pub struct NotifyServicesHook {
    services: Vec<String>,
}

impl NotifyServicesHook {
    pub fn new(services: Vec<String>) -> Self {
        Self { services }
    }
}

#[async_trait]
impl ShutdownHook for NotifyServicesHook {
    fn name(&self) -> &str {
        "notify_services"
    }

    async fn on_shutdown(&self) -> Result<()> {
        for service in &self.services {
            tracing::info!("Notifying service of shutdown: {}", service);
            // Send deregistration request
        }
        Ok(())
    }

    fn priority(&self) -> i32 {
        -100 // Run late
    }
}

/// Shutdown hook for saving state.
pub struct SaveStateHook<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync,
{
    name: String,
    save_fn: F,
}

impl<F> SaveStateHook<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync,
{
    pub fn new(name: impl Into<String>, save_fn: F) -> Self {
        Self {
            name: name.into(),
            save_fn,
        }
    }
}

#[async_trait]
impl<F> ShutdownHook for SaveStateHook<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn on_shutdown(&self) -> Result<()> {
        (self.save_fn)().await
    }

    fn priority(&self) -> i32 {
        50 // Run fairly early
    }
}
```

### 4. Signal Handler (crates/tachikoma-server/src/shutdown/signal.rs)

```rust
//! OS signal handling.

use super::coordinator::ShutdownCoordinator;
use std::sync::Arc;
use tracing::info;

/// Set up signal handlers for graceful shutdown.
pub fn setup_signal_handlers(coordinator: Arc<ShutdownCoordinator>) {
    // Ctrl+C handler
    let ctrl_c_coordinator = coordinator.clone();
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for Ctrl+C: {}", e);
            return;
        }

        info!("Received Ctrl+C signal");
        ctrl_c_coordinator.initiate();
    });

    // SIGTERM handler (Unix only)
    #[cfg(unix)]
    {
        let term_coordinator = coordinator.clone();
        tokio::spawn(async move {
            let mut stream = match tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate(),
            ) {
                Ok(stream) => stream,
                Err(e) => {
                    tracing::error!("Failed to listen for SIGTERM: {}", e);
                    return;
                }
            };

            stream.recv().await;
            info!("Received SIGTERM signal");
            term_coordinator.initiate();
        });
    }

    // SIGHUP handler (Unix only) - for config reload
    #[cfg(unix)]
    {
        let hup_coordinator = coordinator;
        tokio::spawn(async move {
            let mut stream = match tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::hangup(),
            ) {
                Ok(stream) => stream,
                Err(e) => {
                    tracing::error!("Failed to listen for SIGHUP: {}", e);
                    return;
                }
            };

            loop {
                stream.recv().await;
                info!("Received SIGHUP signal - config reload not yet implemented");
                // TODO: Implement config reload
            }
        });
    }
}

/// Create a shutdown signal future.
pub async fn shutdown_signal(coordinator: &ShutdownCoordinator) {
    let mut rx = coordinator.subscribe();
    let _ = rx.recv().await;
}
```

---

## Testing Requirements

1. Ctrl+C triggers shutdown
2. SIGTERM triggers shutdown
3. In-flight requests complete
4. WebSockets close gracefully
5. Database closes properly
6. Hooks execute in order
7. Timeout enforced

---

## Related Specs

- Depends on: [333-server-startup.md](333-server-startup.md)
- Next: [335-tls-config.md](335-tls-config.md)
- Used by: Server shutdown
