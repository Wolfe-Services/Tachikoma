# 333 - Server Startup

**Phase:** 15 - Server
**Spec ID:** 333
**Status:** Planned
**Dependencies:** 332-server-config, 317-axum-router
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement server startup sequence with dependency initialization, health checks, and graceful service registration.

---

## Acceptance Criteria

- [ ] Configuration loading
- [ ] Database connection initialization
- [ ] Middleware stack setup
- [ ] Route registration
- [ ] Service dependency injection
- [ ] Startup health verification
- [ ] Startup logging

---

## Implementation Details

### 1. Server Builder (crates/tachikoma-server/src/startup/builder.rs)

```rust
//! Server builder for configurable startup.

use crate::{
    config::types::ServerConfig,
    health::handlers::HealthState,
    metrics::types::AppMetrics,
    websocket::handler::WsState,
};
use anyhow::{Context, Result};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Server builder for constructing the application.
pub struct ServerBuilder {
    config: ServerConfig,
    db_pool: Option<PgPool>,
    metrics: Option<Arc<AppMetrics>>,
    health_state: Option<Arc<HealthState>>,
    ws_state: Option<Arc<WsState>>,
    additional_routes: Vec<Router>,
}

impl ServerBuilder {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            db_pool: None,
            metrics: None,
            health_state: None,
            ws_state: None,
            additional_routes: Vec::new(),
        }
    }

    /// Set database pool.
    pub fn with_db_pool(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    /// Set metrics collector.
    pub fn with_metrics(mut self, metrics: Arc<AppMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Set health state.
    pub fn with_health(mut self, health: Arc<HealthState>) -> Self {
        self.health_state = Some(health);
        self
    }

    /// Set WebSocket state.
    pub fn with_websocket(mut self, ws: Arc<WsState>) -> Self {
        self.ws_state = Some(ws);
        self
    }

    /// Add additional routes.
    pub fn with_routes(mut self, routes: Router) -> Self {
        self.additional_routes.push(routes);
        self
    }

    /// Build the application.
    pub async fn build(self) -> Result<Application> {
        info!("Building application...");

        // Initialize database if not provided
        let db_pool = match self.db_pool {
            Some(pool) => pool,
            None => {
                info!("Initializing database connection...");
                create_db_pool(&self.config.database).await?
            }
        };

        // Initialize metrics
        let metrics = self.metrics.unwrap_or_else(|| Arc::new(AppMetrics::new()));

        // Initialize health state
        let health_state = self.health_state.unwrap_or_else(|| {
            Arc::new(HealthState::new(env!("CARGO_PKG_VERSION").to_string()))
        });

        // Initialize WebSocket state
        let ws_state = self.ws_state.unwrap_or_else(|| {
            Arc::new(WsState::new(
                crate::websocket::config::WebSocketConfig::default(),
            ))
        });

        // Build application state
        let app_state = AppState {
            config: self.config.clone(),
            db_pool,
            metrics,
            health_state,
            ws_state,
        };

        // Build router
        let router = build_router(app_state.clone(), self.additional_routes);

        Ok(Application {
            config: self.config,
            router,
            state: app_state,
        })
    }
}

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: ServerConfig,
    pub db_pool: PgPool,
    pub metrics: Arc<AppMetrics>,
    pub health_state: Arc<HealthState>,
    pub ws_state: Arc<WsState>,
}

/// Built application ready to run.
pub struct Application {
    pub config: ServerConfig,
    pub router: Router,
    pub state: AppState,
}

impl Application {
    /// Get the socket address.
    pub fn addr(&self) -> std::net::SocketAddr {
        self.config.server.socket_addr()
    }

    /// Get the router.
    pub fn router(&self) -> Router {
        self.router.clone()
    }
}

async fn create_db_pool(config: &crate::config::types::DatabaseConfig) -> Result<PgPool> {
    use std::time::Duration;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .connect(&config.url)
        .await
        .context("Failed to create database pool")?;

    // Verify connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .context("Failed to verify database connection")?;

    info!("Database connection established");
    Ok(pool)
}

fn build_router(state: AppState, additional: Vec<Router>) -> Router {
    use crate::{
        health::router::health_routes,
        metrics::router::metrics_routes,
        websocket::router::ws_routes,
    };

    let mut router = Router::new();

    // Add health routes
    router = router.merge(health_routes(state.health_state.clone()));

    // Add metrics routes
    router = router.merge(metrics_routes(state.metrics.clone()));

    // Add WebSocket routes
    router = router.merge(ws_routes(state.ws_state.clone()));

    // Add additional routes
    for routes in additional {
        router = router.merge(routes);
    }

    router
}
```

### 2. Startup Sequence (crates/tachikoma-server/src/startup/sequence.rs)

```rust
//! Server startup sequence.

use super::builder::{Application, ServerBuilder, AppState};
use crate::{
    config::{loader::load_config, validation::validate_config},
    health::{checks::DatabaseCheck, handlers::HealthState},
    metrics::types::AppMetrics,
};
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Run the full startup sequence.
pub async fn startup() -> Result<Application> {
    info!("Starting Tachikoma server...");

    // Step 1: Load configuration
    info!("Loading configuration...");
    let config = load_config().context("Failed to load configuration")?;

    // Step 2: Validate configuration
    info!("Validating configuration...");
    if let Err(errors) = validate_config(&config) {
        for error in &errors {
            error!("Configuration error: {}", error);
        }
        anyhow::bail!("Configuration validation failed with {} errors", errors.len());
    }

    // Step 3: Initialize tracing
    info!("Initializing tracing...");
    init_tracing(&config.logging)?;

    // Step 4: Initialize metrics
    info!("Initializing metrics...");
    let metrics = Arc::new(AppMetrics::new());

    // Step 5: Initialize health state
    info!("Initializing health checks...");
    let health_state = Arc::new(HealthState::new(
        env!("CARGO_PKG_VERSION").to_string(),
    ));

    // Step 6: Build application
    let app = ServerBuilder::new(config)
        .with_metrics(metrics)
        .with_health(health_state.clone())
        .build()
        .await?;

    // Step 7: Register health checks
    register_health_checks(&app.state).await;

    // Step 8: Run startup checks
    info!("Running startup health checks...");
    let health = app.state.health_state.check_health().await;
    if health.status == crate::health::types::HealthStatus::Unhealthy {
        warn!("Server starting in unhealthy state");
    }

    // Step 9: Mark as started
    app.state.health_state.mark_started();

    info!("Startup complete!");
    Ok(app)
}

async fn register_health_checks(state: &AppState) {
    // Database health check
    let db_check = Box::new(DatabaseCheck::new(state.db_pool.clone()));
    state.health_state.add_check(db_check).await;

    // Memory health check
    let memory_check = Box::new(crate::health::checks::MemoryCheck::new(1024)); // 1GB threshold
    state.health_state.add_check(memory_check).await;
}

fn init_tracing(config: &crate::config::types::LoggingConfig) -> Result<()> {
    use tracing_subscriber::{
        fmt,
        layer::SubscriberExt,
        util::SubscriberInitExt,
        EnvFilter,
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let subscriber = tracing_subscriber::registry().with(filter);

    match config.format.as_str() {
        "json" => {
            subscriber
                .with(fmt::layer().json())
                .try_init()
                .ok();
        }
        _ => {
            subscriber
                .with(fmt::layer().pretty())
                .try_init()
                .ok();
        }
    }

    Ok(())
}

/// Quick startup for testing (skips some checks).
pub async fn startup_test() -> Result<Application> {
    let config = load_config()?;

    ServerBuilder::new(config)
        .build()
        .await
}
```

### 3. Server Runner (crates/tachikoma-server/src/startup/runner.rs)

```rust
//! Server runner with signal handling.

use super::builder::Application;
use anyhow::Result;
use axum::serve::Serve;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};

/// Run the server.
pub async fn run(app: Application) -> Result<()> {
    let addr = app.addr();

    info!("Binding to {}", addr);
    let listener = TcpListener::bind(addr).await?;

    info!("Server listening on {}", addr);

    // Create server
    let server = axum::serve(listener, app.router());

    // Run with graceful shutdown
    server
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server stopped");
    Ok(())
}

/// Run server with TLS.
#[cfg(feature = "tls")]
pub async fn run_tls(app: Application) -> Result<()> {
    use axum_server::tls_rustls::RustlsConfig;

    let config = &app.config.server;
    let addr = app.addr();

    let cert_path = config.tls_cert_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("TLS cert path not configured"))?;
    let key_path = config.tls_key_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("TLS key path not configured"))?;

    let tls_config = RustlsConfig::from_pem_file(cert_path, key_path).await?;

    info!("Server listening on {} (TLS)", addr);

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.router().into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Wait for shutdown signal.
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
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}

/// Get the actual bound address (useful when using port 0).
pub async fn get_bound_addr(listener: &TcpListener) -> SocketAddr {
    listener.local_addr().expect("Failed to get local address")
}
```

### 4. Main Entry Point (crates/tachikoma-server/src/bin/server.rs)

```rust
//! Server binary entry point.

use anyhow::Result;
use tachikoma_server::startup::{runner::run, sequence::startup};
use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
    // Run startup sequence
    let app = match startup().await {
        Ok(app) => app,
        Err(e) => {
            error!("Startup failed: {}", e);
            std::process::exit(1);
        }
    };

    // Run server
    if let Err(e) = run(app).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
```

---

## Testing Requirements

1. Configuration loads correctly
2. Database initializes properly
3. Health checks registered
4. Middleware stack correct
5. Routes accessible
6. Startup logging complete
7. Signal handling works

---

## Related Specs

- Depends on: [332-server-config.md](332-server-config.md), [317-axum-router.md](317-axum-router.md)
- Next: [334-graceful-shutdown.md](334-graceful-shutdown.md)
- Used by: Server binary
