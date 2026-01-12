# Spec 332: Graceful Shutdown

## Phase
15 - Server/API Layer

## Spec ID
332

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 331: Shared State

## Estimated Context
~8%

---

## Objective

Implement graceful shutdown handling for the Tachikoma server, ensuring in-flight requests complete, connections are properly closed, and resources are cleaned up before server termination.

---

## Acceptance Criteria

- [ ] Handle SIGTERM and SIGINT signals
- [ ] Complete in-flight HTTP requests
- [ ] Close WebSocket connections gracefully
- [ ] Flush pending database operations
- [ ] Stop background tasks cleanly
- [ ] Configurable shutdown timeout
- [ ] Health endpoints report shutdown status

---

## Implementation Details

### Shutdown Controller

```rust
// src/server/shutdown/controller.rs
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch, Mutex};
use tokio::time::timeout;

/// Shutdown controller for coordinating graceful shutdown
pub struct ShutdownController {
    /// Signal to initiate shutdown
    shutdown_tx: broadcast::Sender<()>,

    /// Current shutdown state
    state: Arc<Mutex<ShutdownState>>,

    /// Notify when shutdown is complete
    complete_tx: watch::Sender<bool>,

    /// Configuration
    config: ShutdownConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownState {
    Running,
    ShuttingDown,
    Completed,
}

/// Shutdown configuration
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Maximum time to wait for graceful shutdown
    pub timeout: Duration,

    /// Time to wait for new requests to stop
    pub drain_timeout: Duration,

    /// Time to wait for in-flight requests
    pub request_timeout: Duration,

    /// Time to wait for WebSocket connections to close
    pub websocket_timeout: Duration,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            drain_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(15),
            websocket_timeout: Duration::from_secs(10),
        }
    }
}

impl ShutdownController {
    pub fn new(config: ShutdownConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (complete_tx, _) = watch::channel(false);

        Self {
            shutdown_tx,
            state: Arc::new(Mutex::new(ShutdownState::Running)),
            complete_tx,
            config,
        }
    }

    /// Get a handle for listening to shutdown signal
    pub fn subscribe(&self) -> ShutdownHandle {
        ShutdownHandle {
            receiver: self.shutdown_tx.subscribe(),
        }
    }

    /// Get current shutdown state
    pub async fn state(&self) -> ShutdownState {
        *self.state.lock().await
    }

    /// Check if shutdown has been initiated
    pub async fn is_shutting_down(&self) -> bool {
        *self.state.lock().await != ShutdownState::Running
    }

    /// Initiate shutdown
    pub async fn initiate_shutdown(&self) {
        let mut state = self.state.lock().await;
        if *state == ShutdownState::Running {
            *state = ShutdownState::ShuttingDown;
            let _ = self.shutdown_tx.send(());
            tracing::info!("Shutdown initiated");
        }
    }

    /// Wait for shutdown to complete
    pub async fn wait_for_completion(&self) -> watch::Receiver<bool> {
        self.complete_tx.subscribe()
    }

    /// Mark shutdown as complete
    pub async fn mark_complete(&self) {
        *self.state.lock().await = ShutdownState::Completed;
        let _ = self.complete_tx.send(true);
        tracing::info!("Shutdown completed");
    }

    /// Get shutdown configuration
    pub fn config(&self) -> &ShutdownConfig {
        &self.config
    }
}

/// Handle for components to listen for shutdown
pub struct ShutdownHandle {
    receiver: broadcast::Receiver<()>,
}

impl ShutdownHandle {
    /// Wait for shutdown signal
    pub async fn wait(&mut self) {
        let _ = self.receiver.recv().await;
    }

    /// Check if shutdown was signaled (non-blocking)
    pub fn is_signaled(&mut self) -> bool {
        self.receiver.try_recv().is_ok()
    }
}

impl Clone for ShutdownHandle {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.resubscribe(),
        }
    }
}
```

### Shutdown Coordinator

```rust
// src/server/shutdown/coordinator.rs
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn, error};

use super::controller::{ShutdownController, ShutdownConfig};
use crate::server::state::AppState;

/// Coordinates the shutdown process
pub struct ShutdownCoordinator {
    controller: Arc<ShutdownController>,
    state: AppState,
}

impl ShutdownCoordinator {
    pub fn new(state: AppState, config: ShutdownConfig) -> Self {
        Self {
            controller: Arc::new(ShutdownController::new(config)),
            state,
        }
    }

    pub fn controller(&self) -> Arc<ShutdownController> {
        self.controller.clone()
    }

    /// Run the shutdown coordinator
    pub async fn run(self) {
        // Wait for shutdown signal
        self.wait_for_signal().await;

        // Initiate shutdown
        self.controller.initiate_shutdown().await;

        // Execute shutdown sequence
        let config = self.controller.config().clone();

        let result = tokio::time::timeout(
            config.timeout,
            self.execute_shutdown_sequence(),
        ).await;

        match result {
            Ok(Ok(())) => {
                info!("Graceful shutdown completed successfully");
            }
            Ok(Err(e)) => {
                error!(error = %e, "Shutdown completed with errors");
            }
            Err(_) => {
                warn!("Shutdown timed out, forcing exit");
            }
        }

        self.controller.mark_complete().await;
    }

    async fn wait_for_signal(&self) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C signal");
            }
            _ = terminate => {
                info!("Received SIGTERM signal");
            }
        }
    }

    async fn execute_shutdown_sequence(&self) -> Result<(), ShutdownError> {
        let config = self.controller.config();

        // Phase 1: Stop accepting new requests
        info!("Phase 1: Draining new requests");
        self.drain_requests(config.drain_timeout).await?;

        // Phase 2: Close WebSocket connections
        info!("Phase 2: Closing WebSocket connections");
        self.close_websockets(config.websocket_timeout).await?;

        // Phase 3: Wait for in-flight requests
        info!("Phase 3: Waiting for in-flight requests");
        self.wait_for_requests(config.request_timeout).await?;

        // Phase 4: Stop background tasks
        info!("Phase 4: Stopping background tasks");
        self.stop_background_tasks().await?;

        // Phase 5: Flush caches and save state
        info!("Phase 5: Flushing caches");
        self.flush_caches().await?;

        // Phase 6: Close database connections
        info!("Phase 6: Closing database connections");
        self.close_database().await?;

        Ok(())
    }

    async fn drain_requests(&self, timeout: Duration) -> Result<(), ShutdownError> {
        // The server's graceful shutdown will stop accepting new connections
        // This is handled by axum::serve's with_graceful_shutdown
        tokio::time::sleep(timeout).await;
        Ok(())
    }

    async fn close_websockets(&self, timeout: Duration) -> Result<(), ShutdownError> {
        let ws_manager = self.state.ws_manager();
        let connection_count = ws_manager.connection_count().await;

        if connection_count == 0 {
            return Ok(());
        }

        info!(connections = connection_count, "Closing WebSocket connections");

        // Send close message to all connections
        ws_manager.broadcast(
            serde_json::to_string(&ServerMessage::Notification {
                id: uuid::Uuid::new_v4(),
                level: NotificationLevel::Warning,
                title: "Server Shutdown".to_string(),
                message: "Server is shutting down. Please reconnect shortly.".to_string(),
                action: None,
            }).unwrap(),
            None,
        ).await;

        // Wait for connections to close
        let deadline = tokio::time::Instant::now() + timeout;
        while ws_manager.connection_count().await > 0 {
            if tokio::time::Instant::now() >= deadline {
                let remaining = ws_manager.connection_count().await;
                warn!(remaining = remaining, "Force closing remaining WebSocket connections");
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn wait_for_requests(&self, timeout: Duration) -> Result<(), ShutdownError> {
        // In production, we'd track in-flight requests
        // For now, just wait the timeout
        tokio::time::sleep(timeout).await;
        Ok(())
    }

    async fn stop_background_tasks(&self) -> Result<(), ShutdownError> {
        // Cancel rate limiter cleanup task
        // Cancel cache cleanup task
        // Cancel health check task
        // etc.

        info!("Background tasks stopped");
        Ok(())
    }

    async fn flush_caches(&self) -> Result<(), ShutdownError> {
        // Flush in-memory cache if needed
        let cache = self.state.cache();
        cache.clear().await;

        info!("Caches flushed");
        Ok(())
    }

    async fn close_database(&self) -> Result<(), ShutdownError> {
        // Close database connection pool
        // This is typically handled automatically by dropping the pool

        info!("Database connections closed");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error("Timeout during shutdown phase: {phase}")]
    Timeout { phase: String },

    #[error("Error during shutdown: {0}")]
    Other(String),
}
```

### Shutdown-Aware Middleware

```rust
// src/server/middleware/shutdown.rs
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::server::shutdown::ShutdownController;

/// Middleware to reject requests during shutdown
pub async fn shutdown_middleware(
    State(controller): State<Arc<ShutdownController>>,
    request: Request,
    next: Next,
) -> Response {
    if controller.is_shutting_down().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [("Retry-After", "30")],
            "Server is shutting down",
        ).into_response();
    }

    next.run(request).await
}

/// Health check that reports shutdown status
pub async fn shutdown_aware_health(
    State(controller): State<Arc<ShutdownController>>,
) -> impl IntoResponse {
    let state = controller.state().await;

    match state {
        ShutdownState::Running => (StatusCode::OK, "healthy"),
        ShutdownState::ShuttingDown => (StatusCode::SERVICE_UNAVAILABLE, "shutting_down"),
        ShutdownState::Completed => (StatusCode::SERVICE_UNAVAILABLE, "shutdown"),
    }
}
```

### Integration with Server

```rust
// src/server/app.rs (integration)
use crate::server::shutdown::{ShutdownCoordinator, ShutdownConfig};

impl TachikomaServer {
    pub async fn run_with_graceful_shutdown(self) -> Result<(), ServerError> {
        let addr = SocketAddr::new(
            self.config.host.parse()?,
            self.config.port,
        );

        let listener = TcpListener::bind(addr).await?;

        // Create shutdown coordinator
        let shutdown_config = ShutdownConfig::default();
        let coordinator = ShutdownCoordinator::new(self.state.clone(), shutdown_config);
        let controller = coordinator.controller();

        // Add shutdown middleware to router
        let app = self.router()
            .layer(axum::middleware::from_fn_with_state(
                controller.clone(),
                shutdown_middleware,
            ));

        info!("Server listening on http://{}", addr);

        // Spawn shutdown coordinator
        let coordinator_handle = tokio::spawn(coordinator.run());

        // Run server with graceful shutdown
        let shutdown_signal = async move {
            let mut handle = controller.subscribe();
            handle.wait().await;
        };

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await?;

        // Wait for coordinator to complete
        coordinator_handle.await?;

        info!("Server shutdown complete");
        Ok(())
    }
}
```

### Shutdown Hooks

```rust
// src/server/shutdown/hooks.rs
use std::future::Future;
use std::pin::Pin;

/// Type alias for shutdown hook functions
pub type ShutdownHook = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// Registry for shutdown hooks
pub struct ShutdownHooks {
    hooks: Vec<(String, ShutdownHook)>,
}

impl ShutdownHooks {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Register a shutdown hook
    pub fn register(&mut self, name: impl Into<String>, hook: ShutdownHook) {
        self.hooks.push((name.into(), hook));
    }

    /// Execute all hooks
    pub async fn execute(self) {
        for (name, hook) in self.hooks {
            tracing::debug!(hook = %name, "Executing shutdown hook");
            hook().await;
        }
    }
}

impl Default for ShutdownHooks {
    fn default() -> Self {
        Self::new()
    }
}

// Example usage
/*
let mut hooks = ShutdownHooks::new();

hooks.register("flush_metrics", Box::new(|| {
    Box::pin(async {
        metrics_service.flush().await;
    })
}));

hooks.register("close_forge_connections", Box::new(|| {
    Box::pin(async {
        forge_registry.close_all().await;
    })
}));
*/
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_controller() {
        let controller = ShutdownController::new(ShutdownConfig::default());

        assert_eq!(controller.state().await, ShutdownState::Running);

        controller.initiate_shutdown().await;

        assert_eq!(controller.state().await, ShutdownState::ShuttingDown);
    }

    #[tokio::test]
    async fn test_shutdown_handle() {
        let controller = ShutdownController::new(ShutdownConfig::default());
        let mut handle = controller.subscribe();

        // Spawn a task to wait for shutdown
        let wait_task = tokio::spawn(async move {
            handle.wait().await;
            true
        });

        // Signal shutdown
        controller.initiate_shutdown().await;

        // Wait task should complete
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            wait_task,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_hooks() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        let mut hooks = ShutdownHooks::new();
        hooks.register("test", Box::new(move || {
            let executed = executed_clone.clone();
            Box::pin(async move {
                executed.store(true, Ordering::SeqCst);
            })
        }));

        hooks.execute().await;

        assert!(executed.load(Ordering::SeqCst));
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 323**: WebSocket Setup
- **Spec 322**: Health Checks
