# Spec 311: Axum Server Setup

## Phase
15 - Server/API Layer

## Spec ID
311

## Status
Planned

## Dependencies
- Spec 101: Core Types
- Spec 201: Storage Layer

## Estimated Context
~10%

---

## Objective

Set up the foundational Axum web server infrastructure for Tachikoma's HTTP API. This includes the server initialization, TCP listener binding, and basic application structure that all other server components will build upon.

---

## Acceptance Criteria

- [ ] Axum server initializes and binds to configurable host/port
- [ ] Server supports graceful startup with proper error handling
- [ ] Application state is properly shared across handlers
- [ ] Server can be started in both blocking and async modes
- [ ] Structured logging is integrated from startup
- [ ] Server version and build info are exposed
- [ ] Tower service layers are properly configured

---

## Implementation Details

### Dependencies (Cargo.toml)

```toml
[dependencies]
axum = { version = "0.7", features = ["macros", "ws"] }
tokio = { version = "1", features = ["full"] }
tower = { version = "0.4", features = ["util", "timeout", "limit"] }
tower-http = { version = "0.5", features = ["trace", "cors", "compression-gzip"] }
hyper = { version = "1.0", features = ["server", "http1", "http2"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Server Module Structure

```rust
// src/server/mod.rs
pub mod app;
pub mod config;
pub mod error;
pub mod routes;
pub mod state;

pub use app::TachikomaServer;
pub use config::ServerConfig;
pub use state::AppState;
```

### Application State

```rust
// src/server/state.rs
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::storage::Storage;
use crate::forge::ForgeRegistry;
use crate::backend::BackendManager;

/// Shared application state accessible by all handlers
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    /// Storage layer for persistence
    storage: Arc<dyn Storage>,

    /// Forge registry for managing forge connections
    forge_registry: Arc<RwLock<ForgeRegistry>>,

    /// Backend manager for LLM providers
    backend_manager: Arc<BackendManager>,

    /// Server configuration
    config: ServerConfig,

    /// Server start time for uptime tracking
    start_time: std::time::Instant,

    /// Build information
    build_info: BuildInfo,
}

#[derive(Clone, Debug)]
pub struct BuildInfo {
    pub version: &'static str,
    pub git_hash: Option<&'static str>,
    pub build_time: &'static str,
    pub rust_version: &'static str,
}

impl BuildInfo {
    pub fn current() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_hash: option_env!("GIT_HASH"),
            build_time: env!("BUILD_TIME"),
            rust_version: env!("RUST_VERSION"),
        }
    }
}

impl AppState {
    pub fn new(
        storage: Arc<dyn Storage>,
        forge_registry: ForgeRegistry,
        backend_manager: BackendManager,
        config: ServerConfig,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                storage,
                forge_registry: Arc::new(RwLock::new(forge_registry)),
                backend_manager: Arc::new(backend_manager),
                config,
                start_time: std::time::Instant::now(),
                build_info: BuildInfo::current(),
            }),
        }
    }

    pub fn storage(&self) -> &Arc<dyn Storage> {
        &self.inner.storage
    }

    pub fn forge_registry(&self) -> &Arc<RwLock<ForgeRegistry>> {
        &self.inner.forge_registry
    }

    pub fn backend_manager(&self) -> &Arc<BackendManager> {
        &self.inner.backend_manager
    }

    pub fn config(&self) -> &ServerConfig {
        &self.inner.config
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.inner.start_time.elapsed()
    }

    pub fn build_info(&self) -> &BuildInfo {
        &self.inner.build_info
    }
}
```

### Server Implementation

```rust
// src/server/app.rs
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware,
    response::IntoResponse,
};
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::TraceLayer,
};
use tracing::{info, error, Level};

use crate::server::{
    config::ServerConfig,
    error::ServerError,
    routes,
    state::AppState,
};

/// Main Tachikoma server
pub struct TachikomaServer {
    config: ServerConfig,
    state: AppState,
}

impl TachikomaServer {
    /// Create a new server instance
    pub fn new(config: ServerConfig, state: AppState) -> Self {
        Self { config, state }
    }

    /// Build the application router with all middleware and routes
    pub fn router(&self) -> Router {
        let middleware_stack = ServiceBuilder::new()
            // High-level logging of requests and responses
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<_>| {
                        tracing::info_span!(
                            "http_request",
                            method = %request.method(),
                            uri = %request.uri(),
                            version = ?request.version(),
                        )
                    })
                    .on_response(
                        |response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                            tracing::info!(
                                status = %response.status(),
                                latency = ?latency,
                                "response"
                            );
                        }
                    )
            )
            // Response compression
            .layer(CompressionLayer::new());

        Router::new()
            .merge(routes::api_routes())
            .merge(routes::websocket_routes())
            .layer(middleware_stack)
            .with_state(self.state.clone())
            .fallback(fallback_handler)
    }

    /// Run the server
    pub async fn run(self) -> Result<(), ServerError> {
        let addr = SocketAddr::new(
            self.config.host.parse()?,
            self.config.port,
        );

        info!(
            version = %self.state.build_info().version,
            address = %addr,
            "Starting Tachikoma server"
        );

        let listener = TcpListener::bind(addr).await?;
        let app = self.router();

        info!("Server listening on http://{}", addr);

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        info!("Server shutdown complete");
        Ok(())
    }

    /// Run the server with a custom shutdown signal
    pub async fn run_with_shutdown<F>(self, shutdown: F) -> Result<(), ServerError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let addr = SocketAddr::new(
            self.config.host.parse()?,
            self.config.port,
        );

        let listener = TcpListener::bind(addr).await?;
        let app = self.router();

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await?;

        Ok(())
    }

    /// Get the server's socket address (useful for tests)
    pub async fn bind(&self) -> Result<(TcpListener, SocketAddr), ServerError> {
        let addr = SocketAddr::new(
            self.config.host.parse()?,
            self.config.port,
        );
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        Ok((listener, local_addr))
    }
}

/// Fallback handler for unmatched routes
async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// Signal handler for graceful shutdown
async fn shutdown_signal() {
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
            info!("Received Ctrl+C, initiating graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}
```

### Server Builder Pattern

```rust
// src/server/builder.rs
use crate::server::{
    app::TachikomaServer,
    config::ServerConfig,
    state::AppState,
};
use crate::storage::Storage;
use crate::forge::ForgeRegistry;
use crate::backend::BackendManager;

use std::sync::Arc;

/// Builder for constructing a TachikomaServer
pub struct ServerBuilder {
    config: Option<ServerConfig>,
    storage: Option<Arc<dyn Storage>>,
    forge_registry: Option<ForgeRegistry>,
    backend_manager: Option<BackendManager>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            storage: None,
            forge_registry: None,
            backend_manager: None,
        }
    }

    pub fn config(mut self, config: ServerConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn storage(mut self, storage: Arc<dyn Storage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn forge_registry(mut self, registry: ForgeRegistry) -> Self {
        self.forge_registry = Some(registry);
        self
    }

    pub fn backend_manager(mut self, manager: BackendManager) -> Self {
        self.backend_manager = Some(manager);
        self
    }

    pub fn build(self) -> Result<TachikomaServer, &'static str> {
        let config = self.config.ok_or("ServerConfig is required")?;
        let storage = self.storage.ok_or("Storage is required")?;
        let forge_registry = self.forge_registry.ok_or("ForgeRegistry is required")?;
        let backend_manager = self.backend_manager.ok_or("BackendManager is required")?;

        let state = AppState::new(storage, forge_registry, backend_manager, config.clone());

        Ok(TachikomaServer::new(config, state))
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

### Main Entry Point

```rust
// src/main.rs
use tachikoma::server::{ServerBuilder, ServerConfig};
use tachikoma::storage::SqliteStorage;
use tachikoma::forge::ForgeRegistry;
use tachikoma::backend::BackendManager;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tachikoma=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = ServerConfig::from_env()?;

    // Initialize storage
    let storage = Arc::new(SqliteStorage::new(&config.database_url).await?);

    // Initialize forge registry
    let forge_registry = ForgeRegistry::new();

    // Initialize backend manager
    let backend_manager = BackendManager::new();

    // Build and run server
    let server = ServerBuilder::new()
        .config(config)
        .storage(storage)
        .forge_registry(forge_registry)
        .backend_manager(backend_manager)
        .build()?;

    server.run().await?;

    Ok(())
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_server_builds_successfully() {
        let config = ServerConfig::default();
        let storage = Arc::new(MockStorage::new());
        let forge_registry = ForgeRegistry::new();
        let backend_manager = BackendManager::new();

        let server = ServerBuilder::new()
            .config(config)
            .storage(storage)
            .forge_registry(forge_registry)
            .backend_manager(backend_manager)
            .build();

        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_fallback_returns_404() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_app_state_uptime() {
        let state = create_test_state();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        assert!(state.uptime().as_millis() >= 100);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use reqwest::Client;
    use std::time::Duration;

    #[tokio::test]
    async fn test_server_starts_and_accepts_connections() {
        let server = create_test_server();
        let (listener, addr) = server.bind().await.unwrap();

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, server.router()).await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = Client::new();
        let response = client
            .get(format!("http://{}/api/health", addr))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        assert!(response.is_ok());

        server_handle.abort();
    }
}
```

---

## Related Specs

- **Spec 312**: Server Configuration
- **Spec 313**: Route Definitions
- **Spec 314**: Middleware Stack
- **Spec 331**: Shared State Management
- **Spec 332**: Graceful Shutdown
