# 478 - Mock Network

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 478
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create HTTP mocking infrastructure using wiremock for Rust and MSW for TypeScript, enabling tests to intercept and stub network requests without making real HTTP calls.

---

## Acceptance Criteria

- [x] HTTP request interception for all methods (GET, POST, PUT, DELETE, etc.)
- [x] Response stubbing with custom status codes, headers, and bodies
- [x] Request verification (assertions on received requests)
- [x] Latency simulation for timeout testing
- [x] Sequence matching for multi-request scenarios
- [x] GraphQL request matching support

---

## Implementation Details

### 1. Wiremock Utilities for Rust

Create `crates/tachikoma-test-harness/src/mocks/network.rs`:

```rust
//! HTTP mocking utilities using wiremock.

use wiremock::{Mock, MockServer, ResponseTemplate, Request};
use wiremock::matchers::{method, path, header, body_json, query_param};
use serde::Serialize;
use std::time::Duration;

/// HTTP mock server wrapper with convenience methods
pub struct TestHttpServer {
    server: MockServer,
}

impl TestHttpServer {
    /// Start a new mock server
    pub async fn start() -> Self {
        Self {
            server: MockServer::start().await,
        }
    }

    /// Get the server URL
    pub fn url(&self) -> String {
        self.server.uri()
    }

    /// Get URL for a specific path
    pub fn url_for(&self, path: &str) -> String {
        format!("{}{}", self.server.uri(), path)
    }

    /// Access the underlying MockServer
    pub fn inner(&self) -> &MockServer {
        &self.server
    }

    /// Register a GET endpoint that returns JSON
    pub async fn get_json<T: Serialize>(&self, endpoint: &str, response: &T) {
        Mock::given(method("GET"))
            .and(path(endpoint))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Register a POST endpoint that returns JSON
    pub async fn post_json<T: Serialize>(&self, endpoint: &str, response: &T) {
        Mock::given(method("POST"))
            .and(path(endpoint))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Register an endpoint that returns an error
    pub async fn error(&self, endpoint: &str, status: u16, message: &str) {
        Mock::given(path(endpoint))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(serde_json::json!({ "error": message })),
            )
            .mount(&self.server)
            .await;
    }

    /// Register an endpoint with simulated latency
    pub async fn with_latency<T: Serialize>(
        &self,
        endpoint: &str,
        response: &T,
        latency: Duration,
    ) {
        Mock::given(path(endpoint))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .set_delay(latency),
            )
            .mount(&self.server)
            .await;
    }

    /// Register a sequence of responses
    pub async fn sequence(&self, endpoint: &str, responses: Vec<ResponseTemplate>) {
        for (i, response) in responses.into_iter().enumerate() {
            Mock::given(path(endpoint))
                .respond_with(response)
                .up_to_n_times(1)
                .with_priority(100 - i as u8)
                .mount(&self.server)
                .await;
        }
    }

    /// Verify that a request was received
    pub async fn verify_received(&self, endpoint: &str, times: u64) {
        // Wiremock tracks requests automatically
        let received = self.server.received_requests().await.unwrap_or_default();
        let count = received
            .iter()
            .filter(|r| r.url.path() == endpoint)
            .count() as u64;
        assert_eq!(
            count, times,
            "Expected {} requests to {}, got {}",
            times, endpoint, count
        );
    }

    /// Get all received requests
    pub async fn received_requests(&self) -> Vec<Request> {
        self.server.received_requests().await.unwrap_or_default()
    }

    /// Clear all recorded requests
    pub async fn reset(&self) {
        self.server.reset().await;
    }
}

/// Builder for complex mock setups
pub struct MockBuilder {
    method: String,
    path_pattern: Option<String>,
    headers: Vec<(String, String)>,
    query_params: Vec<(String, String)>,
    body_matcher: Option<serde_json::Value>,
}

impl MockBuilder {
    pub fn get(path: &str) -> Self {
        Self {
            method: "GET".into(),
            path_pattern: Some(path.into()),
            headers: Vec::new(),
            query_params: Vec::new(),
            body_matcher: None,
        }
    }

    pub fn post(path: &str) -> Self {
        Self {
            method: "POST".into(),
            path_pattern: Some(path.into()),
            headers: Vec::new(),
            query_params: Vec::new(),
            body_matcher: None,
        }
    }

    pub fn put(path: &str) -> Self {
        Self {
            method: "PUT".into(),
            path_pattern: Some(path.into()),
            headers: Vec::new(),
            query_params: Vec::new(),
            body_matcher: None,
        }
    }

    pub fn delete(path: &str) -> Self {
        Self {
            method: "DELETE".into(),
            path_pattern: Some(path.into()),
            headers: Vec::new(),
            query_params: Vec::new(),
            body_matcher: None,
        }
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn with_query(mut self, name: &str, value: &str) -> Self {
        self.query_params.push((name.into(), value.into()));
        self
    }

    pub fn with_json_body<T: Serialize>(mut self, body: &T) -> Self {
        self.body_matcher = Some(serde_json::to_value(body).unwrap());
        self
    }

    pub fn respond_with(self, response: ResponseTemplate) -> MockSetup {
        MockSetup {
            builder: self,
            response,
        }
    }

    pub fn respond_json<T: Serialize>(self, status: u16, body: &T) -> MockSetup {
        self.respond_with(
            ResponseTemplate::new(status).set_body_json(body),
        )
    }

    pub fn respond_error(self, status: u16, message: &str) -> MockSetup {
        self.respond_with(
            ResponseTemplate::new(status)
                .set_body_json(serde_json::json!({ "error": message })),
        )
    }
}

pub struct MockSetup {
    builder: MockBuilder,
    response: ResponseTemplate,
}

impl MockSetup {
    pub async fn mount(self, server: &TestHttpServer) {
        let mut mock = Mock::given(method(&self.builder.method));

        if let Some(path_pattern) = &self.builder.path_pattern {
            mock = mock.and(path(path_pattern));
        }

        for (name, value) in &self.builder.headers {
            mock = mock.and(header(name.as_str(), value.as_str()));
        }

        for (name, value) in &self.builder.query_params {
            mock = mock.and(query_param(name.as_str(), value.as_str()));
        }

        if let Some(body) = &self.builder.body_matcher {
            mock = mock.and(body_json(body));
        }

        mock.respond_with(self.response)
            .mount(server.inner())
            .await;
    }
}

/// Common response templates
pub mod responses {
    use super::*;

    pub fn ok() -> ResponseTemplate {
        ResponseTemplate::new(200)
    }

    pub fn created() -> ResponseTemplate {
        ResponseTemplate::new(201)
    }

    pub fn no_content() -> ResponseTemplate {
        ResponseTemplate::new(204)
    }

    pub fn bad_request(message: &str) -> ResponseTemplate {
        ResponseTemplate::new(400)
            .set_body_json(serde_json::json!({ "error": message }))
    }

    pub fn unauthorized() -> ResponseTemplate {
        ResponseTemplate::new(401)
            .set_body_json(serde_json::json!({ "error": "Unauthorized" }))
    }

    pub fn forbidden() -> ResponseTemplate {
        ResponseTemplate::new(403)
            .set_body_json(serde_json::json!({ "error": "Forbidden" }))
    }

    pub fn not_found() -> ResponseTemplate {
        ResponseTemplate::new(404)
            .set_body_json(serde_json::json!({ "error": "Not found" }))
    }

    pub fn rate_limited(retry_after: u32) -> ResponseTemplate {
        ResponseTemplate::new(429)
            .insert_header("Retry-After", retry_after.to_string())
            .set_body_json(serde_json::json!({ "error": "Rate limited" }))
    }

    pub fn server_error() -> ResponseTemplate {
        ResponseTemplate::new(500)
            .set_body_json(serde_json::json!({ "error": "Internal server error" }))
    }

    pub fn timeout(delay: Duration) -> ResponseTemplate {
        ResponseTemplate::new(200).set_delay(delay)
    }
}
```

### 2. Example Network Mock Tests

Create `crates/tachikoma-test-harness/tests/network_mock_tests.rs`:

```rust
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
```

### 3. TypeScript Network Mocking (MSW)

Create `web/src/test/mocks/network.ts`:

```typescript
/**
 * HTTP mocking utilities using MSW (Mock Service Worker).
 */

import { setupServer } from 'msw/node';
import { http, HttpResponse, delay } from 'msw';

export type { HttpHandler } from 'msw';

/**
 * Create a test server with MSW
 */
export function createTestServer() {
  const server = setupServer();

  return {
    server,

    /** Start listening for requests */
    listen: () => server.listen({ onUnhandledRequest: 'error' }),

    /** Stop the server */
    close: () => server.close(),

    /** Reset handlers between tests */
    reset: () => server.resetHandlers(),

    /** Add GET handler */
    get: <T>(path: string, response: T, options?: { status?: number; delay?: number }) => {
      server.use(
        http.get(path, async () => {
          if (options?.delay) {
            await delay(options.delay);
          }
          return HttpResponse.json(response, { status: options?.status ?? 200 });
        })
      );
    },

    /** Add POST handler */
    post: <T>(path: string, response: T, options?: { status?: number }) => {
      server.use(
        http.post(path, () => {
          return HttpResponse.json(response, { status: options?.status ?? 200 });
        })
      );
    },

    /** Add error handler */
    error: (path: string, status: number, message: string) => {
      server.use(
        http.all(path, () => {
          return HttpResponse.json({ error: message }, { status });
        })
      );
    },

    /** Add network error */
    networkError: (path: string) => {
      server.use(
        http.all(path, () => {
          return HttpResponse.error();
        })
      );
    },
  };
}

/**
 * Common response factories
 */
export const mockResponses = {
  ok: <T>(data: T) => HttpResponse.json(data),

  created: <T>(data: T) => HttpResponse.json(data, { status: 201 }),

  noContent: () => new HttpResponse(null, { status: 204 }),

  badRequest: (message: string) => HttpResponse.json({ error: message }, { status: 400 }),

  unauthorized: () => HttpResponse.json({ error: 'Unauthorized' }, { status: 401 }),

  forbidden: () => HttpResponse.json({ error: 'Forbidden' }, { status: 403 }),

  notFound: () => HttpResponse.json({ error: 'Not found' }, { status: 404 }),

  rateLimited: (retryAfter = 60) =>
    HttpResponse.json(
      { error: 'Rate limited' },
      {
        status: 429,
        headers: { 'Retry-After': String(retryAfter) },
      }
    ),

  serverError: () => HttpResponse.json({ error: 'Internal server error' }, { status: 500 }),
};

/**
 * Request recorder for verification
 */
export class RequestRecorder {
  private requests: Request[] = [];

  record(request: Request) {
    this.requests.push(request.clone());
  }

  getRequests(): Request[] {
    return this.requests;
  }

  getRequestsTo(path: string): Request[] {
    return this.requests.filter(r => new URL(r.url).pathname === path);
  }

  getRequestCount(path?: string): number {
    if (path) {
      return this.getRequestsTo(path).length;
    }
    return this.requests.length;
  }

  clear() {
    this.requests = [];
  }
}
```

Create `web/src/test/mocks/network.test.ts`:

```typescript
import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { createTestServer, mockResponses } from './network';

describe('Network Mocking', () => {
  const testServer = createTestServer();

  beforeAll(() => testServer.listen());
  afterAll(() => testServer.close());
  beforeEach(() => testServer.reset());

  it('should mock GET requests', async () => {
    testServer.get('http://api.example.com/users/1', {
      id: '1',
      name: 'Test User',
    });

    const response = await fetch('http://api.example.com/users/1');
    const data = await response.json();

    expect(response.status).toBe(200);
    expect(data.name).toBe('Test User');
  });

  it('should mock POST requests', async () => {
    testServer.post('http://api.example.com/users', {
      id: '2',
      name: 'New User',
    }, { status: 201 });

    const response = await fetch('http://api.example.com/users', {
      method: 'POST',
      body: JSON.stringify({ name: 'New User' }),
    });

    expect(response.status).toBe(201);
  });

  it('should mock error responses', async () => {
    testServer.error('http://api.example.com/protected', 401, 'Unauthorized');

    const response = await fetch('http://api.example.com/protected');

    expect(response.status).toBe(401);
  });
});
```

---

## Testing Requirements

1. HTTP mocking intercepts all configured requests
2. Responses can be customized with status, headers, and body
3. Request verification works correctly
4. Latency simulation enables timeout testing
5. Both Rust and TypeScript implementations follow same patterns

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Next: [479-test-fixtures.md](479-test-fixtures.md)
- Related: [476-mock-backends.md](476-mock-backends.md)
