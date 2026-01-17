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

    pub fn patch(path: &str) -> Self {
        Self {
            method: "PATCH".into(),
            path_pattern: Some(path.into()),
            headers: Vec::new(),
            query_params: Vec::new(),
            body_matcher: None,
        }
    }

    pub fn head(path: &str) -> Self {
        Self {
            method: "HEAD".into(),
            path_pattern: Some(path.into()),
            headers: Vec::new(),
            query_params: Vec::new(),
            body_matcher: None,
        }
    }

    pub fn options(path: &str) -> Self {
        Self {
            method: "OPTIONS".into(),
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

/// GraphQL request matching support
pub mod graphql {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct GraphQLRequest {
        pub query: String,
        pub variables: Option<serde_json::Value>,
        pub operation_name: Option<String>,
    }

    impl MockBuilder {
        /// Match GraphQL requests by operation name
        pub fn graphql_operation(mut self, operation_name: &str) -> Self {
            // Create a matcher that checks for GraphQL operation name
            // This is a simplified approach - in practice you'd want more sophisticated matching
            let graphql_body = serde_json::json!({
                "operationName": operation_name
            });
            self.body_matcher = Some(graphql_body);
            self
        }

        /// Match GraphQL requests by query pattern
        pub fn graphql_query(mut self, query_pattern: &str) -> Self {
            let graphql_body = serde_json::json!({
                "query": query_pattern
            });
            self.body_matcher = Some(graphql_body);
            self
        }
    }

    /// Helper to create GraphQL response
    pub fn graphql_response<T: Serialize>(data: &T) -> ResponseTemplate {
        ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "data": data
            }))
    }

    /// Helper to create GraphQL error response
    pub fn graphql_error(message: &str) -> ResponseTemplate {
        ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "errors": [{
                    "message": message
                }]
            }))
    }
}