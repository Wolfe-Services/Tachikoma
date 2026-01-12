# 316 - Server Crate

**Phase:** 15 - Server
**Spec ID:** 316
**Status:** Planned
**Dependencies:** 002-rust-workspace, 011-common-core-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create the main server crate structure for the Tachikoma API server using Axum, establishing the foundational module organization, dependencies, and project layout.

---

## Acceptance Criteria

- [ ] `tachikoma-server` crate created
- [ ] Cargo.toml with all dependencies
- [ ] Module structure defined
- [ ] Binary and library separation
- [ ] Feature flags configured
- [ ] Dev dependencies setup
- [ ] Documentation structure

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-server/Cargo.toml)

```toml
[package]
name = "tachikoma-server"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Tachikoma API server"

[[bin]]
name = "tachikoma-server"
path = "src/bin/server.rs"

[lib]
name = "tachikoma_server"
path = "src/lib.rs"

[features]
default = ["full"]
full = ["websocket", "metrics", "tracing"]
websocket = ["tokio-tungstenite"]
metrics = ["dep:prometheus"]
tracing = ["dep:tracing-subscriber"]

[dependencies]
# Workspace dependencies
tachikoma-common-core.workspace = true
tachikoma-common-config.workspace = true
tachikoma-common-error.workspace = true

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Web framework
axum = { version = "0.7", features = ["ws", "multipart", "macros"] }
axum-extra = { version = "0.9", features = ["typed-header", "cookie"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = [
    "cors",
    "compression-gzip",
    "trace",
    "timeout",
    "limit",
    "request-id",
    "catch-panic"
] }
hyper = { version = "1.1", features = ["full"] }

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }

# Authentication
jsonwebtoken = "9.2"
argon2 = "0.5"

# WebSocket
tokio-tungstenite = { version = "0.21", optional = true }

# Metrics
prometheus = { version = "0.13", optional = true }

# Logging/Tracing
tracing.workspace = true
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"], optional = true }

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror.workspace = true
anyhow.workspace = true
once_cell = "1.19"
parking_lot = "0.12"
dashmap = "5.5"
bytes = "1.5"
futures = "0.3"

# Configuration
config = "0.14"
dotenvy = "0.15"

# TLS
rustls = "0.22"
rustls-pemfile = "2.0"
tokio-rustls = "0.25"

[dev-dependencies]
tokio-test = "0.4"
reqwest = { version = "0.11", features = ["json"] }
wiremock = "0.5"
testcontainers = "0.15"
proptest.workspace = true
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "api_benchmarks"
harness = false
```

### 2. Library Root (crates/tachikoma-server/src/lib.rs)

```rust
//! Tachikoma API Server
//!
//! This crate provides the HTTP/WebSocket server for the Tachikoma platform.
//!
//! # Architecture
//!
//! The server is built on Axum and follows a layered architecture:
//!
//! - **Routes**: HTTP endpoint definitions
//! - **Handlers**: Request processing logic
//! - **Services**: Business logic layer
//! - **Repositories**: Data access layer
//! - **Middleware**: Cross-cutting concerns
//!
//! # Features
//!
//! - `websocket` - WebSocket support for real-time updates
//! - `metrics` - Prometheus metrics endpoint
//! - `tracing` - Structured logging with tracing

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod state;
pub mod ws;

pub use config::ServerConfig;
pub use error::{ApiError, ApiResult};
pub use state::AppState;

use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Server builder for constructing and running the API server.
pub struct Server {
    config: ServerConfig,
    state: AppState,
}

impl Server {
    /// Create a new server with the given configuration.
    pub async fn new(config: ServerConfig) -> Result<Self, anyhow::Error> {
        let state = AppState::new(&config).await?;
        Ok(Self { config, state })
    }

    /// Build the router with all routes and middleware.
    pub fn router(&self) -> Router {
        routes::create_router(self.state.clone())
            .layer(TraceLayer::new_for_http())
    }

    /// Run the server, binding to the configured address.
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(addr).await?;

        info!("Server listening on {}", addr);

        axum::serve(listener, self.router())
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }

    /// Get the server's socket address.
    pub fn addr(&self) -> SocketAddr {
        self.config.socket_addr()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}
```

### 3. Binary Entry Point (crates/tachikoma-server/src/bin/server.rs)

```rust
//! Tachikoma Server Binary

use anyhow::Result;
use tachikoma_server::{Server, ServerConfig};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = ServerConfig::from_env()?;

    info!(
        "Starting Tachikoma Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create and run server
    let server = Server::new(config).await?;
    server.run().await?;

    info!("Server shutdown complete");
    Ok(())
}
```

### 4. Module Structure

```
crates/tachikoma-server/src/
├── lib.rs              # Library root
├── bin/
│   └── server.rs       # Binary entry point
├── config/
│   ├── mod.rs          # Configuration module
│   └── server.rs       # Server-specific config
├── error/
│   ├── mod.rs          # Error handling
│   └── api_error.rs    # API error types
├── handlers/
│   ├── mod.rs          # Handler module
│   ├── auth.rs         # Authentication handlers
│   ├── missions.rs     # Mission handlers
│   ├── specs.rs        # Spec handlers
│   ├── metrics.rs      # Metrics handlers
│   └── health.rs       # Health check handlers
├── middleware/
│   ├── mod.rs          # Middleware module
│   ├── auth.rs         # Authentication middleware
│   ├── authz.rs        # Authorization middleware
│   ├── rate_limit.rs   # Rate limiting
│   ├── logging.rs      # Request logging
│   └── cors.rs         # CORS configuration
├── routes/
│   ├── mod.rs          # Router construction
│   ├── v1.rs           # API v1 routes
│   └── internal.rs     # Internal routes
├── services/
│   ├── mod.rs          # Service layer
│   ├── auth.rs         # Auth service
│   ├── mission.rs      # Mission service
│   └── spec.rs         # Spec service
├── state/
│   ├── mod.rs          # Application state
│   └── app_state.rs    # AppState definition
└── ws/
    ├── mod.rs          # WebSocket module
    ├── handler.rs      # WS connection handler
    └── messages.rs     # WS message types
```

### 5. Application State (crates/tachikoma-server/src/state/app_state.rs)

```rust
//! Application state shared across handlers.

use crate::config::ServerConfig;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool.
    pub db: PgPool,
    /// Server configuration.
    pub config: Arc<ServerConfig>,
    /// Active WebSocket connections.
    pub ws_connections: Arc<DashMap<String, WsConnection>>,
    /// Broadcast channel for real-time events.
    pub event_tx: broadcast::Sender<ServerEvent>,
}

/// WebSocket connection metadata.
pub struct WsConnection {
    pub user_id: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// Server-wide events for broadcasting.
#[derive(Clone, Debug)]
pub enum ServerEvent {
    MissionUpdate { mission_id: String },
    MetricsUpdate,
    Notification { user_id: String, message: String },
}

impl AppState {
    /// Create new application state.
    pub async fn new(config: &ServerConfig) -> Result<Self, anyhow::Error> {
        let db = PgPool::connect(&config.database_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&db).await?;

        let (event_tx, _) = broadcast::channel(1000);

        Ok(Self {
            db,
            config: Arc::new(config.clone()),
            ws_connections: Arc::new(DashMap::new()),
            event_tx,
        })
    }
}
```

---

## Testing Requirements

1. Crate compiles without errors
2. All features compile independently
3. Server starts and binds to port
4. Graceful shutdown works
5. Database connection established
6. Module structure is correct

---

## Related Specs

- Depends on: [002-rust-workspace.md](../phase-00-setup/002-rust-workspace.md)
- Next: [317-axum-router.md](317-axum-router.md)
- Used by: All server specs
