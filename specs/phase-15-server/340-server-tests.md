# 340 - Server Tests

**Phase:** 15 - Server
**Spec ID:** 340
**Status:** Planned
**Dependencies:** All Phase 15 specs
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive test suites for the server including unit tests, integration tests, and end-to-end API tests.

---

## Acceptance Criteria

- [x] Unit tests for all modules
- [x] Integration tests with test database
- [x] API endpoint tests
- [x] WebSocket tests
- [x] Authentication tests
- [x] Rate limiting tests
- [x] Test utilities and helpers

---

## Implementation Details

### 1. Test Utilities (crates/tachikoma-server/src/testing/utils.rs)

```rust
//! Test utilities and helpers.

use crate::{
    config::types::ServerConfig,
    startup::builder::{AppState, ServerBuilder},
};
use axum::{body::Body, http::Request, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

/// Test application builder.
pub struct TestApp {
    pub router: Router,
    pub state: AppState,
    pub pool: PgPool,
}

impl TestApp {
    /// Create a new test application.
    pub async fn new() -> Self {
        let config = test_config();
        let pool = create_test_pool().await;

        let app = ServerBuilder::new(config)
            .with_db_pool(pool.clone())
            .build()
            .await
            .expect("Failed to build test app");

        Self {
            router: app.router,
            state: app.state,
            pool,
        }
    }

    /// Make a request to the app.
    pub async fn request(&self, req: Request<Body>) -> axum::response::Response {
        self.router
            .clone()
            .oneshot(req)
            .await
            .expect("Request failed")
    }

    /// Make a GET request.
    pub async fn get(&self, uri: &str) -> axum::response::Response {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        self.request(req).await
    }

    /// Make a POST request with JSON body.
    pub async fn post_json<T: serde::Serialize>(
        &self,
        uri: &str,
        body: &T,
    ) -> axum::response::Response {
        let body = serde_json::to_string(body).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();
        self.request(req).await
    }

    /// Make an authenticated request.
    pub async fn authenticated_request(
        &self,
        req: Request<Body>,
        token: &str,
    ) -> axum::response::Response {
        let (mut parts, body) = req.into_parts();
        parts.headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
        let req = Request::from_parts(parts, body);
        self.request(req).await
    }
}

/// Create test configuration.
pub fn test_config() -> ServerConfig {
    ServerConfig {
        server: crate::config::types::ServerBindConfig {
            host: "127.0.0.1".to_string(),
            port: 0, // Random port
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            request_timeout_secs: 30,
            keepalive_secs: 75,
        },
        database: crate::config::types::DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/tachikoma_test".to_string()),
            max_connections: 5,
            min_connections: 1,
            connect_timeout_secs: 5,
            idle_timeout_secs: 60,
            log_queries: true,
        },
        auth: crate::config::types::AuthConfig {
            jwt_secret: "test_secret_key_for_testing_only_32chars".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 86400,
            enable_api_keys: true,
        },
        rate_limit: crate::config::types::RateLimitConfig {
            enabled: false, // Disable for tests
            requests_per_window: 1000,
            window_secs: 60,
            burst: 100,
        },
        logging: crate::config::types::LoggingConfig {
            level: "debug".to_string(),
            format: "pretty".to_string(),
            log_requests: true,
            exclude_paths: vec![],
        },
        cors: crate::config::types::CorsConfig {
            allowed_origins: vec![],
            allow_any_origin: true,
            allow_credentials: true,
            max_age_secs: 3600,
        },
        websocket: crate::config::types::WebSocketConfig {
            enabled: true,
            ping_interval_secs: 30,
            max_message_size: 65536,
            require_auth: false,
        },
        cache: Default::default(),
        features: Default::default(),
    }
}

/// Create test database pool.
pub async fn create_test_pool() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/tachikoma_test".to_string());

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to create test pool")
}

/// Generate a test JWT token.
pub fn generate_test_token(user_id: uuid::Uuid, roles: Vec<String>) -> String {
    use crate::middleware::auth::{jwt::encode_token, types::Claims};

    let claims = Claims::new_access(user_id, "test@example.com", roles, 3600);
    encode_token(&claims, "test_secret_key_for_testing_only_32chars")
        .expect("Failed to generate test token")
}
```

### 2. Integration Tests (crates/tachikoma-server/tests/integration/mod.rs)

```rust
//! Integration tests.

mod api;
mod auth;
mod health;
mod websocket;

use tachikoma_server::testing::utils::TestApp;

/// Setup function for integration tests.
pub async fn setup() -> TestApp {
    TestApp::new().await
}
```

### 3. Health Endpoint Tests (crates/tachikoma-server/tests/integration/health.rs)

```rust
//! Health endpoint tests.

use super::setup;
use axum::http::StatusCode;

#[tokio::test]
async fn test_liveness_probe() {
    let app = setup().await;

    let response = app.get("/health/live").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap(),
    )
    .unwrap();

    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_readiness_probe() {
    let app = setup().await;

    let response = app.get("/health/ready").await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_detailed_health() {
    let app = setup().await;

    let response = app.get("/health").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.get("version").is_some());
    assert!(body.get("uptime_seconds").is_some());
}
```

### 4. Authentication Tests (crates/tachikoma-server/tests/integration/auth.rs)

```rust
//! Authentication tests.

use super::setup;
use axum::http::StatusCode;
use tachikoma_server::testing::utils::generate_test_token;
use uuid::Uuid;

#[tokio::test]
async fn test_unauthenticated_request_rejected() {
    let app = setup().await;

    let response = app.get("/api/v1/protected").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_token_rejected() {
    let app = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/protected")
        .header("Authorization", "Bearer invalid_token")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.request(req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_valid_token_accepted() {
    let app = setup().await;

    let user_id = Uuid::new_v4();
    let token = generate_test_token(user_id, vec!["user".to_string()]);

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/user/me")
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.request(req).await;

    // Should not be unauthorized (might be 404 if endpoint doesn't exist)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_expired_token_rejected() {
    use tachikoma_server::middleware::auth::{jwt::encode_token, types::Claims};

    let app = setup().await;

    // Create token that expired 1 hour ago
    let claims = Claims::new_access(
        Uuid::new_v4(),
        "test@example.com",
        vec!["user".to_string()],
        -3600, // Negative expiry = already expired
    );
    let token = encode_token(&claims, "test_secret_key_for_testing_only_32chars").unwrap();

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/protected")
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.request(req).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### 5. Rate Limiting Tests (crates/tachikoma-server/tests/integration/rate_limit.rs)

```rust
//! Rate limiting tests.

use tachikoma_server::middleware::rate_limit::{
    store::InMemoryStore,
    types::{RateLimitConfig, RateLimitState},
};
use std::time::Duration;

#[tokio::test]
async fn test_token_bucket_refill() {
    let config = RateLimitConfig::new(10, 60); // 10 requests per minute
    let mut state = RateLimitState::new(&config);

    // Consume all tokens
    for _ in 0..10 {
        assert!(state.try_consume());
    }

    // Should be rate limited
    assert!(!state.try_consume());
}

#[tokio::test]
async fn test_burst_allowance() {
    let config = RateLimitConfig::new(10, 60).with_burst(5);
    let mut state = RateLimitState::new(&config);

    // Should allow 15 requests (10 + 5 burst)
    for _ in 0..15 {
        assert!(state.try_consume());
    }

    // Should be rate limited
    assert!(!state.try_consume());
}

#[tokio::test]
async fn test_in_memory_store() {
    let store = InMemoryStore::new();
    let config = RateLimitConfig::new(2, 60);

    // First two requests allowed
    let result1 = store.check_and_consume("test_key", &config).await;
    assert!(result1.allowed);

    let result2 = store.check_and_consume("test_key", &config).await;
    assert!(result2.allowed);

    // Third request should be rate limited
    let result3 = store.check_and_consume("test_key", &config).await;
    assert!(!result3.allowed);
    assert!(result3.retry_after.is_some());
}

#[tokio::test]
async fn test_different_keys_independent() {
    let store = InMemoryStore::new();
    let config = RateLimitConfig::new(1, 60);

    // First key
    let result1 = store.check_and_consume("key1", &config).await;
    assert!(result1.allowed);

    let result2 = store.check_and_consume("key1", &config).await;
    assert!(!result2.allowed);

    // Second key should still be allowed
    let result3 = store.check_and_consume("key2", &config).await;
    assert!(result3.allowed);
}
```

### 6. WebSocket Tests (crates/tachikoma-server/tests/integration/websocket.rs)

```rust
//! WebSocket tests.

use tachikoma_server::websocket::{
    messages::{IncomingMessage, OutgoingMessage},
    session::{SessionManager, WsOutgoingMessage, WsSession},
};
use tokio::sync::mpsc;
use uuid::Uuid;

#[tokio::test]
async fn test_session_creation() {
    let session = WsSession::new();

    assert!(!session.is_authenticated());
    assert!(session.subscriptions.is_empty());
}

#[tokio::test]
async fn test_session_authentication() {
    let mut session = WsSession::new();
    let user_id = Uuid::new_v4();

    session.authenticate(user_id);

    assert!(session.is_authenticated());
    assert_eq!(session.user_id, Some(user_id));
}

#[tokio::test]
async fn test_session_subscriptions() {
    let mut session = WsSession::new();

    session.subscribe("missions");
    session.subscribe("missions/123");

    assert!(session.is_subscribed("missions"));
    assert!(session.is_subscribed("missions/123"));
    assert!(!session.is_subscribed("other"));

    session.unsubscribe("missions");
    assert!(!session.subscriptions.contains(&"missions".to_string()));
}

#[tokio::test]
async fn test_session_manager() {
    let manager = SessionManager::new();
    let session = WsSession::new();
    let session_id = session.id;

    let (tx, _rx) = mpsc::channel(10);
    manager.register(session, tx).await;

    assert_eq!(manager.session_count().await, 1);

    manager.unregister(session_id).await;
    assert_eq!(manager.session_count().await, 0);
}

#[tokio::test]
async fn test_broadcast_to_topic() {
    let manager = SessionManager::new();

    // Create sessions with different subscriptions
    let mut session1 = WsSession::new();
    session1.subscribe("missions");
    let id1 = session1.id;

    let mut session2 = WsSession::new();
    session2.subscribe("other");
    let id2 = session2.id;

    let (tx1, mut rx1) = mpsc::channel(10);
    let (tx2, mut rx2) = mpsc::channel(10);

    manager.register(session1, tx1).await;
    manager.register(session2, tx2).await;

    // Broadcast to "missions" topic
    manager
        .broadcast_to_topic(
            "missions",
            WsOutgoingMessage::Text("test message".to_string()),
        )
        .await;

    // Only session1 should receive the message
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    assert!(rx1.try_recv().is_ok());
    assert!(rx2.try_recv().is_err());
}
```

### 7. Test Database Setup (crates/tachikoma-server/tests/fixtures/setup.sql)

```sql
-- Test database setup

-- Clean up existing test data
TRUNCATE TABLE missions CASCADE;
TRUNCATE TABLE specs CASCADE;
TRUNCATE TABLE users CASCADE;

-- Insert test users
INSERT INTO users (id, email, password_hash, roles, created_at)
VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin@test.com', '$argon2id$v=19$m=65536,t=3,p=4$test', ARRAY['admin', 'user'], NOW()),
    ('00000000-0000-0000-0000-000000000002', 'user@test.com', '$argon2id$v=19$m=65536,t=3,p=4$test', ARRAY['user'], NOW());

-- Insert test missions
INSERT INTO missions (id, name, description, status, created_at, user_id)
VALUES
    ('00000000-0000-0000-0000-000000000001', 'Test Mission 1', 'A test mission', 'pending', NOW(), '00000000-0000-0000-0000-000000000001'),
    ('00000000-0000-0000-0000-000000000002', 'Test Mission 2', 'Another test mission', 'running', NOW(), '00000000-0000-0000-0000-000000000001');
```

---

## Testing Requirements

1. All unit tests pass
2. Integration tests with database work
3. API tests cover all endpoints
4. WebSocket tests verify real-time functionality
5. Auth tests verify security
6. Rate limit tests verify limits
7. Test coverage meets targets

---

## Related Specs

- Depends on: All Phase 15 specs
- Used by: CI/CD pipeline

---

## Test Commands

```bash
# Run all tests
cargo test -p tachikoma-server

# Run with coverage
cargo llvm-cov --package tachikoma-server

# Run integration tests only
cargo test -p tachikoma-server --test '*'

# Run with test database
TEST_DATABASE_URL=postgres://localhost/tachikoma_test cargo test

# Run specific test
cargo test -p tachikoma-server test_liveness_probe
```
