# 413 - Event Capture API

## Overview

HTTP API endpoints for capturing analytics events from web, mobile, and server-side clients.

## Rust Implementation

```rust
// crates/analytics/src/capture.rs

use crate::event_types::{AnalyticsEvent, EventCategory, EventId, EventSource, Platform};
use crate::schema::{EventSanitizer, SchemaValidator};
use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Capture API state
pub struct CaptureState {
    /// Event processing channel
    pub event_tx: mpsc::Sender<AnalyticsEvent>,
    /// Schema validator
    pub validator: SchemaValidator,
    /// Event sanitizer
    pub sanitizer: EventSanitizer,
    /// API key validator
    pub api_key_validator: Arc<dyn ApiKeyValidator>,
}

/// API key validation trait
#[async_trait::async_trait]
pub trait ApiKeyValidator: Send + Sync {
    async fn validate(&self, api_key: &str) -> Result<ProjectInfo, CaptureError>;
}

/// Project information from API key
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub project_id: String,
    pub environment: String,
    pub rate_limit: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    #[error("Batch too large: {0} events (max {1})")]
    BatchTooLarge(usize, usize),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for CaptureError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            CaptureError::InvalidApiKey => (StatusCode::UNAUTHORIZED, self.to_string()),
            CaptureError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            CaptureError::InvalidEvent(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            CaptureError::BatchTooLarge(_, _) => (StatusCode::BAD_REQUEST, self.to_string()),
            CaptureError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };

        (status, Json(CaptureErrorResponse { error: message })).into_response()
    }
}

#[derive(Serialize)]
struct CaptureErrorResponse {
    error: String,
}

/// Single event capture request
#[derive(Debug, Deserialize)]
pub struct CaptureRequest {
    /// API key (can also be in header)
    pub api_key: Option<String>,
    /// Event name
    pub event: String,
    /// Distinct user ID
    pub distinct_id: String,
    /// Event properties
    #[serde(default)]
    pub properties: serde_json::Map<String, serde_json::Value>,
    /// User properties to set
    #[serde(rename = "$set")]
    pub user_set: Option<serde_json::Map<String, serde_json::Value>>,
    /// User properties to set once
    #[serde(rename = "$set_once")]
    pub user_set_once: Option<serde_json::Map<String, serde_json::Value>>,
    /// Timestamp (optional, defaults to now)
    pub timestamp: Option<String>,
}

/// Batch capture request
#[derive(Debug, Deserialize)]
pub struct BatchCaptureRequest {
    /// API key
    pub api_key: Option<String>,
    /// Batch of events
    pub batch: Vec<CaptureRequest>,
}

/// Capture response
#[derive(Debug, Serialize)]
pub struct CaptureResponse {
    pub status: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_remaining: Option<u32>,
}

/// Create capture router
pub fn capture_router(state: Arc<CaptureState>) -> Router {
    Router::new()
        .route("/capture", post(capture_single))
        .route("/capture/", post(capture_single))
        .route("/batch", post(capture_batch))
        .route("/batch/", post(capture_batch))
        .route("/e", post(capture_single))  // PostHog compatibility
        .route("/track", post(capture_single))  // Segment compatibility
        .with_state(state)
}

/// Handle single event capture
async fn capture_single(
    State(state): State<Arc<CaptureState>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CaptureRequest>,
) -> Result<Json<CaptureResponse>, CaptureError> {
    // Extract and validate API key
    let api_key = extract_api_key(&headers, req.api_key.as_deref())?;
    let project = state.api_key_validator.validate(&api_key).await?;

    // Build event
    let mut event = build_event(req, &project, &headers, addr)?;

    // Sanitize
    state.sanitizer.sanitize(&mut event);

    // Validate
    if let Err(errors) = state.validator.validate(&event) {
        return Err(CaptureError::InvalidEvent(
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
        ));
    }

    // Send to processing pipeline
    state.event_tx.send(event).await
        .map_err(|e| CaptureError::Internal(e.to_string()))?;

    Ok(Json(CaptureResponse {
        status: 1,
        quota_remaining: None,
    }))
}

/// Handle batch event capture
async fn capture_batch(
    State(state): State<Arc<CaptureState>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<BatchCaptureRequest>,
) -> Result<Json<CaptureResponse>, CaptureError> {
    const MAX_BATCH_SIZE: usize = 1000;

    if req.batch.len() > MAX_BATCH_SIZE {
        return Err(CaptureError::BatchTooLarge(req.batch.len(), MAX_BATCH_SIZE));
    }

    // Extract and validate API key
    let api_key = extract_api_key(&headers, req.api_key.as_deref())?;
    let project = state.api_key_validator.validate(&api_key).await?;

    // Process each event
    for event_req in req.batch {
        let mut event = build_event(event_req, &project, &headers, addr)?;
        state.sanitizer.sanitize(&mut event);

        // Skip invalid events in batch (log but don't fail)
        if state.validator.validate(&event).is_ok() {
            let _ = state.event_tx.send(event).await;
        }
    }

    Ok(Json(CaptureResponse {
        status: 1,
        quota_remaining: None,
    }))
}

/// Extract API key from headers or body
fn extract_api_key(headers: &HeaderMap, body_key: Option<&str>) -> Result<String, CaptureError> {
    // Try Authorization header
    if let Some(auth) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                return Ok(key.to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(key) = headers.get("X-API-Key") {
        if let Ok(key_str) = key.to_str() {
            return Ok(key_str.to_string());
        }
    }

    // Try body
    if let Some(key) = body_key {
        return Ok(key.to_string());
    }

    Err(CaptureError::InvalidApiKey)
}

/// Build analytics event from request
fn build_event(
    req: CaptureRequest,
    project: &ProjectInfo,
    headers: &HeaderMap,
    addr: SocketAddr,
) -> Result<AnalyticsEvent, CaptureError> {
    let timestamp = if let Some(ts) = req.timestamp {
        chrono::DateTime::parse_from_rfc3339(&ts)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    let category = categorize_event(&req.event);

    let mut properties: std::collections::HashMap<String, serde_json::Value> =
        req.properties.into_iter().collect();

    // Add IP address
    properties.insert("$ip".to_string(), serde_json::json!(addr.ip().to_string()));

    // Add user agent
    if let Some(ua) = headers.get(header::USER_AGENT) {
        if let Ok(ua_str) = ua.to_str() {
            properties.insert("$user_agent".to_string(), serde_json::json!(ua_str));
        }
    }

    // Detect platform from user agent
    let platform = detect_platform(headers);

    let mut user_properties = None;
    if let Some(set) = req.user_set {
        user_properties = Some(set.into_iter().collect());
    }

    let sdk_info = extract_sdk_info(headers);

    Ok(AnalyticsEvent {
        id: EventId::new(),
        event: req.event,
        category,
        distinct_id: req.distinct_id,
        timestamp,
        properties,
        user_properties,
        session_id: None,
        environment: project.environment.clone(),
        source: EventSource {
            sdk: sdk_info.0,
            sdk_version: sdk_info.1,
            platform,
            library: None,
        },
        received_at: Utc::now(),
    })
}

fn categorize_event(event_name: &str) -> EventCategory {
    match event_name {
        "$pageview" | "$screen" => EventCategory::Pageview,
        "$identify" => EventCategory::Identify,
        "$group" => EventCategory::Group,
        "$feature_flag" => EventCategory::FeatureFlag,
        "$revenue" | "$purchase" => EventCategory::Revenue,
        "$session_start" | "$session_end" => EventCategory::Session,
        name if name.starts_with("$") => EventCategory::System,
        _ => EventCategory::Custom,
    }
}

fn detect_platform(headers: &HeaderMap) -> Platform {
    if let Some(ua) = headers.get(header::USER_AGENT) {
        if let Ok(ua_str) = ua.to_str() {
            let ua_lower = ua_str.to_lowercase();

            if ua_lower.contains("iphone") || ua_lower.contains("ipad") {
                return Platform::Ios;
            }
            if ua_lower.contains("android") {
                return Platform::Android;
            }
            if ua_lower.contains("mozilla") || ua_lower.contains("chrome") || ua_lower.contains("safari") {
                return Platform::Web;
            }
        }
    }

    // Check for SDK header
    if let Some(sdk) = headers.get("X-SDK-Name") {
        if let Ok(sdk_str) = sdk.to_str() {
            match sdk_str.to_lowercase().as_str() {
                "posthog-ios" | "tachikoma-ios" => return Platform::Ios,
                "posthog-android" | "tachikoma-android" => return Platform::Android,
                "posthog-js" | "tachikoma-js" => return Platform::Web,
                "posthog-node" | "tachikoma-rust" => return Platform::Server,
                _ => {}
            }
        }
    }

    Platform::Unknown
}

fn extract_sdk_info(headers: &HeaderMap) -> (String, String) {
    let sdk_name = headers.get("X-SDK-Name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let sdk_version = headers.get("X-SDK-Version")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0.0.0")
        .to_string();

    (sdk_name, sdk_version)
}

/// Pixel tracking endpoint (for email opens, etc.)
pub async fn pixel_handler(
    State(state): State<Arc<CaptureState>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Extract event data from query params
    if let (Some(api_key), Some(event), Some(distinct_id)) = (
        params.get("k"),
        params.get("e"),
        params.get("d"),
    ) {
        if let Ok(project) = state.api_key_validator.validate(api_key).await {
            let req = CaptureRequest {
                api_key: None,
                event: event.clone(),
                distinct_id: distinct_id.clone(),
                properties: params.iter()
                    .filter(|(k, _)| !["k", "e", "d"].contains(&k.as_str()))
                    .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                    .collect(),
                user_set: None,
                user_set_once: None,
                timestamp: None,
            };

            if let Ok(event) = build_event(req, &project, &headers, addr) {
                let _ = state.event_tx.send(event).await;
            }
        }
    }

    // Return 1x1 transparent GIF
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/gif")],
        vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00,
             0x80, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x2c,
             0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02,
             0x02, 0x44, 0x01, 0x00, 0x3b],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockApiKeyValidator;

    #[async_trait::async_trait]
    impl ApiKeyValidator for MockApiKeyValidator {
        async fn validate(&self, _api_key: &str) -> Result<ProjectInfo, CaptureError> {
            Ok(ProjectInfo {
                project_id: "test-project".to_string(),
                environment: "test".to_string(),
                rate_limit: None,
            })
        }
    }

    #[test]
    fn test_categorize_event() {
        assert_eq!(categorize_event("$pageview"), EventCategory::Pageview);
        assert_eq!(categorize_event("$identify"), EventCategory::Identify);
        assert_eq!(categorize_event("button_click"), EventCategory::Custom);
    }
}
```

## API Documentation

### POST /capture

Capture a single event.

```json
{
  "api_key": "phc_xxx",
  "event": "button_clicked",
  "distinct_id": "user-123",
  "properties": {
    "button_id": "signup",
    "$current_url": "https://example.com"
  },
  "$set": {
    "email": "user@example.com"
  }
}
```

### POST /batch

Capture multiple events.

```json
{
  "api_key": "phc_xxx",
  "batch": [
    {
      "event": "$pageview",
      "distinct_id": "user-123",
      "properties": { "$current_url": "https://example.com" }
    },
    {
      "event": "button_clicked",
      "distinct_id": "user-123",
      "properties": { "button_id": "signup" }
    }
  ]
}
```

## Related Specs

- 411-event-types.md - Event types
- 414-event-batching.md - Batching logic
- 415-event-persistence.md - Storage
