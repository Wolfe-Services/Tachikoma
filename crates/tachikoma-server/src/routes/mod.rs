//! Route configuration for the Tachikoma API server.

mod internal;
mod v1;

use crate::{
    state::AppState,
};
use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};
use std::time::Duration;

/// Create the main application router.
pub fn create_router(state: AppState) -> Router {
    // Common middleware stack applied to all routes
    let common_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
        ))
        .layer(CatchPanicLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
        ;

    Router::new()
        // API routes
        .nest("/api/v1", v1::router(state.clone()))
        // Internal routes (health, metrics, etc.)
        .nest("/internal", internal::router(state.clone()))
        // WebSocket endpoint
        .route("/ws", get(ws_handler))
        // Root redirect to API docs
        .route("/", get(root_handler))
        // Fallback for unmatched routes
        .fallback(fallback_handler.into_service())
        // Apply common middleware
        .layer(common_middleware)
        // Attach state
        .with_state(state)
}

async fn root_handler() -> impl IntoResponse {
    axum::response::Redirect::to("/api/v1/docs")
}

async fn fallback_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({
            "error": "not_found",
            "message": "The requested resource was not found"
        })),
    )
}

// Placeholder WebSocket handler
async fn ws_handler() -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        axum::Json(serde_json::json!({
            "error": "not_implemented",
            "message": "WebSocket endpoint not yet implemented"
        })),
    )
}