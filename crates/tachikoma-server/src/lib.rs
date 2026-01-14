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
pub mod cache;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod request;
pub mod response;
pub mod routes;
pub mod services;
pub mod shutdown;
pub mod state;
pub mod tls;
pub mod versioning;
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