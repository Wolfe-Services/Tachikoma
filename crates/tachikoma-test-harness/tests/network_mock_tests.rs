use tachikoma_test_harness::mocks::network::*;
use std::time::Duration;

#[tokio::test]
async fn test_mock_get_json() {
    let server = TestHttpServer::start().await;

    server.get_json("/api/users/1", &serde_json::json!({
        "id": 1,
        "name": "Test User"
    })).await;

    let client = reqwest::Client::new();
    let response = client
        .get(server.url_for("/api/users/1"))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["name"], "Test User");
}

#[tokio::test]
async fn test_mock_builder_pattern() {
    let server = TestHttpServer::start().await;

    MockBuilder::post("/api/login")
        .with_header("Content-Type", "application/json")
        .respond_json(200, &serde_json::json!({
            "token": "abc123"
        }))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(server.url_for("/api/login"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "username": "test" }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_mock_error_response() {
    let server = TestHttpServer::start().await;

    server.error("/api/protected", 401, "Unauthorized").await;

    let client = reqwest::Client::new();
    let response = client
        .get(server.url_for("/api/protected"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_mock_sequence() {
    let server = TestHttpServer::start().await;

    server.sequence("/api/data", vec![
        responses::rate_limited(1),
        responses::ok().set_body_json(serde_json::json!({ "data": "success" })),
    ]).await;

    let client = reqwest::Client::new();

    // First request gets rate limited
    let r1 = client.get(server.url_for("/api/data")).send().await.unwrap();
    assert_eq!(r1.status(), 429);

    // Second request succeeds
    let r2 = client.get(server.url_for("/api/data")).send().await.unwrap();
    assert_eq!(r2.status(), 200);
}

#[tokio::test]
async fn test_verify_requests() {
    let server = TestHttpServer::start().await;

    server.get_json("/api/health", &serde_json::json!({ "status": "ok" })).await;

    let client = reqwest::Client::new();
    client.get(server.url_for("/api/health")).send().await.unwrap();
    client.get(server.url_for("/api/health")).send().await.unwrap();

    server.verify_received("/api/health", 2).await;
}

#[tokio::test]
async fn test_all_http_methods() {
    let server = TestHttpServer::start().await;

    // Test GET
    MockBuilder::get("/api/get")
        .respond_json(200, &serde_json::json!({"method": "GET"}))
        .mount(&server)
        .await;

    // Test POST
    MockBuilder::post("/api/post")
        .respond_json(201, &serde_json::json!({"method": "POST"}))
        .mount(&server)
        .await;

    // Test PUT
    MockBuilder::put("/api/put")
        .respond_json(200, &serde_json::json!({"method": "PUT"}))
        .mount(&server)
        .await;

    // Test DELETE
    MockBuilder::delete("/api/delete")
        .respond_json(204, &serde_json::json!({}))
        .mount(&server)
        .await;

    // Test PATCH
    MockBuilder::patch("/api/patch")
        .respond_json(200, &serde_json::json!({"method": "PATCH"}))
        .mount(&server)
        .await;

    // Test HEAD
    MockBuilder::head("/api/head")
        .respond_with(responses::ok())
        .mount(&server)
        .await;

    // Test OPTIONS
    MockBuilder::options("/api/options")
        .respond_json(200, &serde_json::json!({"method": "OPTIONS"}))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();

    // Verify all methods work
    let get_resp = client.get(server.url_for("/api/get")).send().await.unwrap();
    assert_eq!(get_resp.status(), 200);

    let post_resp = client.post(server.url_for("/api/post")).send().await.unwrap();
    assert_eq!(post_resp.status(), 201);

    let put_resp = client.put(server.url_for("/api/put")).send().await.unwrap();
    assert_eq!(put_resp.status(), 200);

    let delete_resp = client.delete(server.url_for("/api/delete")).send().await.unwrap();
    assert_eq!(delete_resp.status(), 204);

    let patch_resp = client.patch(server.url_for("/api/patch")).send().await.unwrap();
    assert_eq!(patch_resp.status(), 200);

    let head_resp = client.head(server.url_for("/api/head")).send().await.unwrap();
    assert_eq!(head_resp.status(), 200);
}

#[tokio::test]
async fn test_latency_simulation() {
    let server = TestHttpServer::start().await;

    server.with_latency("/api/slow", &serde_json::json!({"data": "delayed"}), Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let response = client
        .get(server.url_for("/api/slow"))
        .send()
        .await
        .unwrap();
    let elapsed = start.elapsed();

    assert!(response.status().is_success());
    assert!(elapsed >= Duration::from_millis(100));
}

#[tokio::test]
async fn test_custom_headers() {
    let server = TestHttpServer::start().await;

    MockBuilder::get("/api/headers")
        .with_header("Authorization", "Bearer token123")
        .with_header("Custom-Header", "custom-value")
        .respond_json(200, &serde_json::json!({"authenticated": true}))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(server.url_for("/api/headers"))
        .header("Authorization", "Bearer token123")
        .header("Custom-Header", "custom-value")
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_query_parameters() {
    let server = TestHttpServer::start().await;

    MockBuilder::get("/api/search")
        .with_query("q", "rust")
        .with_query("limit", "10")
        .respond_json(200, &serde_json::json!({"results": ["result1", "result2"]}))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}?q=rust&limit=10", server.url_for("/api/search")))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_graphql_operations() {
    let server = TestHttpServer::start().await;

    // Test GraphQL operation matching
    MockBuilder::post("/graphql")
        .graphql_operation("GetUser")
        .respond_json(200, &graphql::graphql_response(&serde_json::json!({
            "user": {
                "id": "1",
                "name": "Test User"
            }
        })))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(server.url_for("/graphql"))
        .json(&serde_json::json!({
            "query": "query GetUser { user { id name } }",
            "operationName": "GetUser"
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.get("data").is_some());
}

#[tokio::test]
async fn test_json_body_matching() {
    let server = TestHttpServer::start().await;

    MockBuilder::post("/api/users")
        .with_json_body(&serde_json::json!({
            "name": "John Doe",
            "email": "john@example.com"
        }))
        .respond_json(201, &serde_json::json!({
            "id": "123",
            "name": "John Doe",
            "email": "john@example.com"
        }))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(server.url_for("/api/users"))
        .json(&serde_json::json!({
            "name": "John Doe",
            "email": "john@example.com"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
}