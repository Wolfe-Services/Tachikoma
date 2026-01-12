# Spec 335: Integration Tests

## Phase
15 - Server/API Layer

## Spec ID
335

## Status
Planned

## Dependencies
- All Phase 15 Specs (311-334)

## Estimated Context
~12%

---

## Objective

Implement comprehensive integration tests for the Tachikoma server, ensuring all API endpoints, WebSocket functionality, and system behaviors work correctly together in realistic scenarios.

---

## Acceptance Criteria

- [ ] Full API endpoint coverage
- [ ] WebSocket connection and messaging tests
- [ ] Authentication and authorization tests
- [ ] Rate limiting behavior tests
- [ ] Error handling verification
- [ ] Database integration tests
- [ ] End-to-end workflow tests
- [ ] Performance benchmarks

---

## Implementation Details

### Test Infrastructure

```rust
// tests/common/mod.rs
use std::sync::Arc;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use tower::ServiceExt;
use uuid::Uuid;

use tachikoma::server::{AppState, AppStateBuilder, ServerConfig};
use tachikoma::storage::SqliteStorage;

/// Test application wrapper
pub struct TestApp {
    pub app: Router,
    pub state: AppState,
    pub db_path: String,
}

impl TestApp {
    /// Create a new test application
    pub async fn new() -> Self {
        let db_path = format!("/tmp/tachikoma_test_{}.db", Uuid::new_v4());

        let storage = Arc::new(
            SqliteStorage::new(&format!("sqlite://{}", db_path))
                .await
                .unwrap()
        );

        // Run migrations
        storage.migrate().await.unwrap();

        let config = ServerConfig {
            server: ServerBindConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Random port
                ..Default::default()
            },
            ..Default::default()
        };

        let state = AppStateBuilder::new(config.clone(), storage)
            .build();

        let server = TachikomaServer::new(config, state.clone());
        let app = server.router();

        Self {
            app,
            state,
            db_path,
        }
    }

    /// Make a request to the test app
    pub async fn request(&self, request: Request<Body>) -> Response<Body> {
        self.app
            .clone()
            .oneshot(request)
            .await
            .unwrap()
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Response<Body> {
        self.request(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .unwrap()
        ).await
    }

    /// Make a POST request with JSON body
    pub async fn post_json(&self, path: &str, body: impl serde::Serialize) -> Response<Body> {
        self.request(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        ).await
    }

    /// Make a PUT request with JSON body
    pub async fn put_json(&self, path: &str, body: impl serde::Serialize) -> Response<Body> {
        self.request(
            Request::builder()
                .method("PUT")
                .uri(path)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        ).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> Response<Body> {
        self.request(
            Request::builder()
                .method("DELETE")
                .uri(path)
                .body(Body::empty())
                .unwrap()
        ).await
    }

    /// Extract JSON from response
    pub async fn json<T: serde::de::DeserializeOwned>(response: Response<Body>) -> T {
        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// Create a test mission
    pub async fn create_mission(&self, name: &str) -> serde_json::Value {
        let response = self.post_json("/api/v1/missions", serde_json::json!({
            "name": name
        })).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        Self::json(response).await
    }

    /// Create a test phase
    pub async fn create_phase(&self, mission_id: &str, name: &str) -> serde_json::Value {
        let response = self.post_json(
            &format!("/api/v1/missions/{}/phases", mission_id),
            serde_json::json!({ "name": name })
        ).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        Self::json(response).await
    }

    /// Create a test spec
    pub async fn create_spec(&self, phase_id: &str, spec_id: &str, title: &str) -> serde_json::Value {
        let response = self.post_json("/api/v1/specs", serde_json::json!({
            "phase_id": phase_id,
            "spec_id": spec_id,
            "title": title
        })).await;

        assert_eq!(response.status(), StatusCode::CREATED);
        Self::json(response).await
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Clean up test database
        let _ = std::fs::remove_file(&self.db_path);
    }
}
```

### API Integration Tests

```rust
// tests/api/missions_test.rs
use crate::common::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn test_create_mission() {
    let app = TestApp::new().await;

    let response = app.post_json("/api/v1/missions", serde_json::json!({
        "name": "Test Mission",
        "description": "A test mission"
    })).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: serde_json::Value = TestApp::json(response).await;
    assert_eq!(body["data"]["name"], "Test Mission");
    assert!(body["data"]["id"].is_string());
}

#[tokio::test]
async fn test_list_missions() {
    let app = TestApp::new().await;

    // Create some missions
    app.create_mission("Mission 1").await;
    app.create_mission("Mission 2").await;
    app.create_mission("Mission 3").await;

    let response = app.get("/api/v1/missions").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = TestApp::json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
    assert_eq!(body["pagination"]["total"], 3);
}

#[tokio::test]
async fn test_get_mission() {
    let app = TestApp::new().await;

    let created = app.create_mission("Test Mission").await;
    let id = created["data"]["id"].as_str().unwrap();

    let response = app.get(&format!("/api/v1/missions/{}", id)).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = TestApp::json(response).await;
    assert_eq!(body["data"]["name"], "Test Mission");
}

#[tokio::test]
async fn test_update_mission() {
    let app = TestApp::new().await;

    let created = app.create_mission("Original Name").await;
    let id = created["data"]["id"].as_str().unwrap();
    let version = created["data"]["version"].as_i64().unwrap();

    let response = app.put_json(
        &format!("/api/v1/missions/{}", id),
        serde_json::json!({
            "name": "Updated Name",
            "version": version
        })
    ).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = TestApp::json(response).await;
    assert_eq!(body["data"]["name"], "Updated Name");
    assert_eq!(body["data"]["version"], version + 1);
}

#[tokio::test]
async fn test_delete_mission() {
    let app = TestApp::new().await;

    let created = app.create_mission("To Delete").await;
    let id = created["data"]["id"].as_str().unwrap();

    let response = app.delete(&format!("/api/v1/missions/{}", id)).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let response = app.get(&format!("/api/v1/missions/{}", id)).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_mission_not_found() {
    let app = TestApp::new().await;

    let response = app.get("/api/v1/missions/00000000-0000-0000-0000-000000000000").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = TestApp::json(response).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_mission_validation() {
    let app = TestApp::new().await;

    // Empty name
    let response = app.post_json("/api/v1/missions", serde_json::json!({
        "name": ""
    })).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = TestApp::json(response).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(body["error"]["details"]["fields"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_optimistic_locking() {
    let app = TestApp::new().await;

    let created = app.create_mission("Test").await;
    let id = created["data"]["id"].as_str().unwrap();

    // First update succeeds
    let response = app.put_json(
        &format!("/api/v1/missions/{}", id),
        serde_json::json!({ "name": "Update 1", "version": 1 })
    ).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Second update with old version fails
    let response = app.put_json(
        &format!("/api/v1/missions/{}", id),
        serde_json::json!({ "name": "Update 2", "version": 1 })
    ).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
```

### Specs API Tests

```rust
// tests/api/specs_test.rs
use crate::common::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn test_spec_crud_flow() {
    let app = TestApp::new().await;

    // Setup
    let mission = app.create_mission("Test Mission").await;
    let mission_id = mission["data"]["id"].as_str().unwrap();

    let phase = app.create_phase(mission_id, "Phase 1").await;
    let phase_id = phase["data"]["id"].as_str().unwrap();

    // Create spec
    let response = app.post_json("/api/v1/specs", serde_json::json!({
        "phase_id": phase_id,
        "spec_id": "311",
        "title": "Server Setup",
        "description": "Set up the Axum server"
    })).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let spec: serde_json::Value = TestApp::json(response).await;
    let spec_id = spec["data"]["id"].as_str().unwrap();

    // Read spec
    let response = app.get(&format!("/api/v1/specs/{}", spec_id)).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Update status
    let response = app.request(
        Request::builder()
            .method("PATCH")
            .uri(&format!("/api/v1/specs/{}/status", spec_id))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "in_progress"}"#))
            .unwrap()
    ).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Delete spec
    let response = app.delete(&format!("/api/v1/specs/{}", spec_id)).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_spec_dependencies() {
    let app = TestApp::new().await;

    // Setup
    let mission = app.create_mission("Test").await;
    let mission_id = mission["data"]["id"].as_str().unwrap();
    let phase = app.create_phase(mission_id, "Phase 1").await;
    let phase_id = phase["data"]["id"].as_str().unwrap();

    // Create specs
    let spec1 = app.create_spec(phase_id, "101", "Base Spec").await;
    let spec1_id = spec1["data"]["id"].as_str().unwrap();

    let response = app.post_json("/api/v1/specs", serde_json::json!({
        "phase_id": phase_id,
        "spec_id": "102",
        "title": "Dependent Spec",
        "dependencies": [spec1_id]
    })).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let spec2: serde_json::Value = TestApp::json(response).await;

    // Check dependencies
    let response = app.get(&format!("/api/v1/specs/{}/dependencies", spec2["data"]["id"].as_str().unwrap())).await;
    assert_eq!(response.status(), StatusCode::OK);

    let deps: serde_json::Value = TestApp::json(response).await;
    assert!(!deps["nodes"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_spec_status_transitions() {
    let app = TestApp::new().await;

    let mission = app.create_mission("Test").await;
    let phase = app.create_phase(mission["data"]["id"].as_str().unwrap(), "Phase 1").await;
    let spec = app.create_spec(phase["data"]["id"].as_str().unwrap(), "101", "Test").await;
    let spec_id = spec["data"]["id"].as_str().unwrap();

    // Valid transition: planned -> in_progress
    let response = app.request(
        Request::builder()
            .method("PATCH")
            .uri(&format!("/api/v1/specs/{}/status", spec_id))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "in_progress"}"#))
            .unwrap()
    ).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Invalid transition: in_progress -> completed (should go through review/testing)
    let response = app.request(
        Request::builder()
            .method("PATCH")
            .uri(&format!("/api/v1/specs/{}/status", spec_id))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "completed"}"#))
            .unwrap()
    ).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
```

### WebSocket Tests

```rust
// tests/websocket/connection_test.rs
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn test_websocket_connection() {
    let app = TestApp::new().await;
    let addr = app.start_server().await;

    let (mut ws_stream, _) = connect_async(format!("ws://{}/ws", addr))
        .await
        .expect("Failed to connect");

    // Should receive welcome message
    let msg = ws_stream.next().await.unwrap().unwrap();
    let text = msg.to_text().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();

    assert_eq!(parsed["type"], "welcome");
    assert!(parsed["connection_id"].is_string());

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn test_websocket_ping_pong() {
    let app = TestApp::new().await;
    let addr = app.start_server().await;

    let (mut ws_stream, _) = connect_async(format!("ws://{}/ws", addr))
        .await
        .unwrap();

    // Skip welcome message
    ws_stream.next().await;

    // Send ping
    ws_stream.send(Message::Text(r#"{"type": "ping"}"#.to_string())).await.unwrap();

    // Should receive pong
    let msg = ws_stream.next().await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();

    assert_eq!(parsed["type"], "pong");
}

#[tokio::test]
async fn test_websocket_subscription() {
    let app = TestApp::new().await;
    let addr = app.start_server().await;

    // Create a spec to subscribe to
    let mission = app.create_mission("Test").await;
    let phase = app.create_phase(mission["data"]["id"].as_str().unwrap(), "Phase").await;
    let spec = app.create_spec(phase["data"]["id"].as_str().unwrap(), "101", "Test").await;
    let spec_id = spec["data"]["id"].as_str().unwrap();

    let (mut ws_stream, _) = connect_async(format!("ws://{}/ws", addr))
        .await
        .unwrap();

    // Skip welcome
    ws_stream.next().await;

    // Subscribe to spec channel
    ws_stream.send(Message::Text(format!(
        r#"{{"type": "subscribe", "channel": "spec:{}"}}"#,
        spec_id
    ))).await.unwrap();

    // Should receive subscription confirmation
    let msg = ws_stream.next().await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();

    assert_eq!(parsed["type"], "subscribed");
    assert!(parsed["channel"].as_str().unwrap().contains(spec_id));
}
```

### Health Check Tests

```rust
// tests/health/health_test.rs
use crate::common::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn test_liveness_probe() {
    let app = TestApp::new().await;

    let response = app.get("/health/live").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = TestApp::json(response).await;
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_readiness_probe() {
    let app = TestApp::new().await;

    let response = app.get("/health/ready").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = TestApp::json(response).await;
    assert!(body["ready"].as_bool().unwrap());
}

#[tokio::test]
async fn test_detailed_health() {
    let app = TestApp::new().await;

    let response = app.get("/health").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = TestApp::json(response).await;
    assert!(body["version"]["version"].is_string());
    assert!(body["uptime"]["uptime_seconds"].is_number());
    assert!(body["components"].is_array());
}
```

### Rate Limiting Tests

```rust
// tests/ratelimit/ratelimit_test.rs
use crate::common::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn test_rate_limit_headers() {
    let app = TestApp::new().await;

    let response = app.get("/api/v1/missions").await;

    assert!(response.headers().contains_key("x-ratelimit-limit"));
    assert!(response.headers().contains_key("x-ratelimit-remaining"));
    assert!(response.headers().contains_key("x-ratelimit-reset"));
}

#[tokio::test]
async fn test_rate_limit_exceeded() {
    let app = TestApp::new().await;

    // Make requests until rate limited
    for _ in 0..150 {
        let response = app.get("/api/v1/missions").await;
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            // Verify retry-after header
            assert!(response.headers().contains_key("retry-after"));
            return;
        }
    }

    panic!("Rate limit was not triggered");
}
```

### End-to-End Workflow Tests

```rust
// tests/e2e/workflow_test.rs
use crate::common::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn test_complete_workflow() {
    let app = TestApp::new().await;

    // 1. Create a mission
    let mission = app.create_mission("E2E Test Mission").await;
    let mission_id = mission["data"]["id"].as_str().unwrap();
    assert_eq!(mission["data"]["status"], "draft");

    // 2. Create phases
    let phase1 = app.create_phase(mission_id, "Phase 1: Foundation").await;
    let phase1_id = phase1["data"]["id"].as_str().unwrap();

    let phase2 = app.create_phase(mission_id, "Phase 2: Features").await;
    let phase2_id = phase2["data"]["id"].as_str().unwrap();

    // 3. Create specs with dependencies
    let spec1 = app.create_spec(phase1_id, "101", "Core Types").await;
    let spec1_id = spec1["data"]["id"].as_str().unwrap();

    let response = app.post_json("/api/v1/specs", serde_json::json!({
        "phase_id": phase1_id,
        "spec_id": "102",
        "title": "Storage Layer",
        "dependencies": [spec1_id]
    })).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let spec2: serde_json::Value = TestApp::json(response).await;
    let spec2_id = spec2["data"]["id"].as_str().unwrap();

    // 4. Activate mission
    let response = app.post_json(&format!("/api/v1/missions/{}/activate", mission_id), serde_json::json!({})).await;
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Progress spec1 to completion
    app.update_spec_status(spec1_id, "in_progress").await;
    app.update_spec_status(spec1_id, "in_review").await;
    app.update_spec_status(spec1_id, "testing").await;
    app.update_spec_status(spec1_id, "completed").await;

    // 6. Now spec2 can be started (dependency met)
    let response = app.request(
        Request::builder()
            .method("PATCH")
            .uri(&format!("/api/v1/specs/{}/status", spec2_id))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "in_progress"}"#))
            .unwrap()
    ).await;
    assert_eq!(response.status(), StatusCode::OK);

    // 7. Check mission stats
    let response = app.get(&format!("/api/v1/missions/{}", mission_id)).await;
    let mission: serde_json::Value = TestApp::json(response).await;

    assert!(mission["data"]["stats"]["completed_specs"].as_i64().unwrap() >= 1);
}
```

### Performance Benchmarks

```rust
// benches/api_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_list_missions(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let app = rt.block_on(async { TestApp::new().await });

    // Create test data
    rt.block_on(async {
        for i in 0..100 {
            app.create_mission(&format!("Mission {}", i)).await;
        }
    });

    c.bench_function("list_missions", |b| {
        b.iter(|| {
            rt.block_on(async {
                app.get("/api/v1/missions").await
            })
        })
    });
}

fn benchmark_create_spec(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let app = rt.block_on(async { TestApp::new().await });

    let (mission_id, phase_id) = rt.block_on(async {
        let mission = app.create_mission("Benchmark Mission").await;
        let phase = app.create_phase(
            mission["data"]["id"].as_str().unwrap(),
            "Phase 1"
        ).await;
        (
            mission["data"]["id"].as_str().unwrap().to_string(),
            phase["data"]["id"].as_str().unwrap().to_string(),
        )
    });

    let mut counter = 0;

    c.bench_function("create_spec", |b| {
        b.iter(|| {
            counter += 1;
            rt.block_on(async {
                app.create_spec(&phase_id, &format!("{:03}", counter), "Benchmark Spec").await
            })
        })
    });
}

criterion_group!(benches, benchmark_list_missions, benchmark_create_spec);
criterion_main!(benches);
```

---

## Testing Requirements

### Running Tests

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test '*'

# Run specific test file
cargo test --test api_missions_test

# Run with logging
RUST_LOG=debug cargo test

# Run benchmarks
cargo bench
```

### CI Configuration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run tests
        run: cargo test --all-features

      - name: Run benchmarks
        run: cargo bench --no-run
```

---

## Related Specs

- All Phase 15 Specs (311-334)
