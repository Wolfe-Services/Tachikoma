# 476 - Mock Backends

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 476
**Status:** Planned
**Dependencies:** 471-test-harness, 051-backend-trait
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create comprehensive mock implementations of LLM backends (Claude, Codex, Gemini, Ollama) for testing, enabling deterministic test execution without real API calls or costs.

---

## Acceptance Criteria

- [x] Mock backend implements full Backend trait
- [x] Configurable responses for different scenarios
- [x] Streaming response simulation supported
- [x] Tool call mocking with configurable behavior
- [x] Error simulation (rate limits, timeouts, auth failures)
- [x] Request recording for verification

---

## Implementation Details

### 1. Mock Backend Core

Create `crates/tachikoma-test-harness/src/mocks/backend.rs`:

```rust
//! Mock backend implementations for testing.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

/// Recorded request for verification
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timestamp: std::time::Instant,
}

/// Configurable mock response
#[derive(Debug, Clone)]
pub enum MockResponse {
    /// Return a text response
    Text(String),
    /// Return a tool call
    ToolCall {
        name: String,
        arguments: serde_json::Value,
    },
    /// Simulate an error
    Error(MockError),
    /// Stream chunks with delays
    Streaming {
        chunks: Vec<String>,
        delay_ms: u64,
    },
    /// Multiple responses in sequence (for tool use)
    Sequence(Vec<MockResponse>),
}

/// Mock error types
#[derive(Debug, Clone)]
pub enum MockError {
    RateLimit { retry_after_secs: u32 },
    AuthenticationFailed,
    Timeout,
    ServerError(String),
    InvalidRequest(String),
    ContextTooLong { max: usize, actual: usize },
}

/// Mock backend configuration
#[derive(Debug, Clone)]
pub struct MockBackendConfig {
    /// Default response when no specific response is configured
    pub default_response: MockResponse,
    /// Simulated latency in milliseconds
    pub latency_ms: u64,
    /// Whether to record requests
    pub record_requests: bool,
    /// Maximum context tokens
    pub max_context_tokens: usize,
}

impl Default for MockBackendConfig {
    fn default() -> Self {
        Self {
            default_response: MockResponse::Text("Mock response".into()),
            latency_ms: 0,
            record_requests: true,
            max_context_tokens: 100_000,
        }
    }
}

/// Mock backend for testing
pub struct MockBackend {
    config: MockBackendConfig,
    responses: Arc<Mutex<VecDeque<MockResponse>>>,
    recorded_requests: Arc<Mutex<Vec<RecordedRequest>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockBackend {
    /// Create a new mock backend
    pub fn new(config: MockBackendConfig) -> Self {
        Self {
            config,
            responses: Arc::new(Mutex::new(VecDeque::new())),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create with default configuration
    pub fn default_mock() -> Self {
        Self::new(MockBackendConfig::default())
    }

    /// Queue a response to be returned
    pub fn queue_response(&self, response: MockResponse) {
        self.responses.lock().unwrap().push_back(response);
    }

    /// Queue multiple responses
    pub fn queue_responses(&self, responses: impl IntoIterator<Item = MockResponse>) {
        let mut queue = self.responses.lock().unwrap();
        for response in responses {
            queue.push_back(response);
        }
    }

    /// Get recorded requests
    pub fn recorded_requests(&self) -> Vec<RecordedRequest> {
        self.recorded_requests.lock().unwrap().clone()
    }

    /// Get the number of times the backend was called
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Clear recorded requests
    pub fn clear_recorded(&self) {
        self.recorded_requests.lock().unwrap().clear();
        *self.call_count.lock().unwrap() = 0;
    }

    /// Get the next response
    fn next_response(&self) -> MockResponse {
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| self.config.default_response.clone())
    }

    /// Record a request
    fn record(&self, request: RecordedRequest) {
        if self.config.record_requests {
            self.recorded_requests.lock().unwrap().push(request);
        }
        *self.call_count.lock().unwrap() += 1;
    }
}

// Placeholder types - would be imported from backend crate
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCallResponse>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone)]
pub struct ToolCallResponse {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// Backend trait (simplified for mock implementation)
#[async_trait]
pub trait Backend: Send + Sync {
    async fn complete(&self, messages: &[Message], tools: &[ToolDefinition])
        -> Result<CompletionResponse, BackendError>;

    fn name(&self) -> &str;
    fn max_context(&self) -> usize;
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Rate limited, retry after {retry_after} seconds")]
    RateLimit { retry_after: u32 },
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Request timed out")]
    Timeout,
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Context too long: {actual} tokens exceeds maximum {max}")]
    ContextTooLong { max: usize, actual: usize },
}

#[async_trait]
impl Backend for MockBackend {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<CompletionResponse, BackendError> {
        // Record the request
        self.record(RecordedRequest {
            messages: messages.to_vec(),
            tools: tools.to_vec(),
            temperature: None,
            max_tokens: None,
            timestamp: std::time::Instant::now(),
        });

        // Simulate latency
        if self.config.latency_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.config.latency_ms)).await;
        }

        // Get response
        let response = self.next_response();

        match response {
            MockResponse::Text(text) => Ok(CompletionResponse {
                content: text,
                tool_calls: vec![],
                usage: TokenUsage::default(),
            }),
            MockResponse::ToolCall { name, arguments } => Ok(CompletionResponse {
                content: String::new(),
                tool_calls: vec![ToolCallResponse { name, arguments }],
                usage: TokenUsage::default(),
            }),
            MockResponse::Error(err) => Err(err.into()),
            MockResponse::Streaming { chunks, delay_ms } => {
                // For non-streaming API, concatenate all chunks
                let mut content = String::new();
                for chunk in chunks {
                    if delay_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                    content.push_str(&chunk);
                }
                Ok(CompletionResponse {
                    content,
                    tool_calls: vec![],
                    usage: TokenUsage::default(),
                })
            }
            MockResponse::Sequence(responses) => {
                // Return first response, queue the rest
                let mut iter = responses.into_iter();
                if let Some(first) = iter.next() {
                    for remaining in iter {
                        self.queue_response(remaining);
                    }
                    // Recursively handle the first response
                    self.responses.lock().unwrap().push_front(first);
                    Box::pin(self.complete(messages, tools)).await
                } else {
                    Ok(CompletionResponse {
                        content: String::new(),
                        tool_calls: vec![],
                        usage: TokenUsage::default(),
                    })
                }
            }
        }
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn max_context(&self) -> usize {
        self.config.max_context_tokens
    }
}

impl From<MockError> for BackendError {
    fn from(err: MockError) -> Self {
        match err {
            MockError::RateLimit { retry_after_secs } => {
                BackendError::RateLimit { retry_after: retry_after_secs }
            }
            MockError::AuthenticationFailed => BackendError::AuthenticationFailed,
            MockError::Timeout => BackendError::Timeout,
            MockError::ServerError(msg) => BackendError::ServerError(msg),
            MockError::InvalidRequest(msg) => BackendError::InvalidRequest(msg),
            MockError::ContextTooLong { max, actual } => {
                BackendError::ContextTooLong { max, actual }
            }
        }
    }
}
```

### 2. Mock Backend Builder

Create `crates/tachikoma-test-harness/src/mocks/backend_builder.rs`:

```rust
//! Builder pattern for configuring mock backends.

use super::backend::*;

/// Builder for MockBackend
pub struct MockBackendBuilder {
    config: MockBackendConfig,
    initial_responses: Vec<MockResponse>,
}

impl MockBackendBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: MockBackendConfig::default(),
            initial_responses: Vec::new(),
        }
    }

    /// Set default response for unconfigured calls
    pub fn default_response(mut self, response: MockResponse) -> Self {
        self.config.default_response = response;
        self
    }

    /// Set default text response
    pub fn default_text(self, text: impl Into<String>) -> Self {
        self.default_response(MockResponse::Text(text.into()))
    }

    /// Add simulated latency
    pub fn with_latency_ms(mut self, latency_ms: u64) -> Self {
        self.config.latency_ms = latency_ms;
        self
    }

    /// Set maximum context tokens
    pub fn max_context_tokens(mut self, max: usize) -> Self {
        self.config.max_context_tokens = max;
        self
    }

    /// Disable request recording
    pub fn no_recording(mut self) -> Self {
        self.config.record_requests = false;
        self
    }

    /// Queue a text response
    pub fn with_text_response(mut self, text: impl Into<String>) -> Self {
        self.initial_responses.push(MockResponse::Text(text.into()));
        self
    }

    /// Queue a tool call response
    pub fn with_tool_call(mut self, name: impl Into<String>, args: serde_json::Value) -> Self {
        self.initial_responses.push(MockResponse::ToolCall {
            name: name.into(),
            arguments: args,
        });
        self
    }

    /// Queue an error response
    pub fn with_error(mut self, error: MockError) -> Self {
        self.initial_responses.push(MockResponse::Error(error));
        self
    }

    /// Queue a rate limit error
    pub fn with_rate_limit(self, retry_after: u32) -> Self {
        self.with_error(MockError::RateLimit { retry_after_secs: retry_after })
    }

    /// Queue a streaming response
    pub fn with_streaming(mut self, chunks: Vec<String>, delay_ms: u64) -> Self {
        self.initial_responses.push(MockResponse::Streaming { chunks, delay_ms });
        self
    }

    /// Build the mock backend
    pub fn build(self) -> MockBackend {
        let backend = MockBackend::new(self.config);
        backend.queue_responses(self.initial_responses);
        backend
    }
}

impl Default for MockBackendBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience functions
impl MockBackend {
    /// Create a mock that always returns the given text
    pub fn always_returns(text: impl Into<String>) -> Self {
        MockBackendBuilder::new()
            .default_text(text)
            .build()
    }

    /// Create a mock that simulates rate limiting
    pub fn rate_limited(retry_after: u32) -> Self {
        MockBackendBuilder::new()
            .with_rate_limit(retry_after)
            .default_text("Rate limit cleared")
            .build()
    }

    /// Create a mock for testing tool calls
    pub fn with_tool_responses(tool_calls: Vec<(String, serde_json::Value)>) -> Self {
        let mut builder = MockBackendBuilder::new();
        for (name, args) in tool_calls {
            builder = builder.with_tool_call(name, args);
        }
        builder.build()
    }
}
```

### 3. Example Tests Using Mock Backends

Create `crates/tachikoma-test-harness/tests/mock_backend_tests.rs`:

```rust
use tachikoma_test_harness::mocks::backend::*;
use tachikoma_test_harness::mocks::backend_builder::*;

#[tokio::test]
async fn test_mock_backend_returns_configured_response() {
    let backend = MockBackendBuilder::new()
        .with_text_response("Hello, world!")
        .build();

    let response = backend
        .complete(&[], &[])
        .await
        .expect("Should succeed");

    assert_eq!(response.content, "Hello, world!");
}

#[tokio::test]
async fn test_mock_backend_records_requests() {
    let backend = MockBackendBuilder::new()
        .default_text("response")
        .build();

    let messages = vec![
        Message { role: "user".into(), content: "Hello".into() },
    ];

    backend.complete(&messages, &[]).await.unwrap();

    let recorded = backend.recorded_requests();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].messages[0].content, "Hello");
}

#[tokio::test]
async fn test_mock_backend_simulates_rate_limit() {
    let backend = MockBackendBuilder::new()
        .with_rate_limit(60)
        .default_text("Success after retry")
        .build();

    // First call should fail with rate limit
    let result = backend.complete(&[], &[]).await;
    assert!(matches!(result, Err(BackendError::RateLimit { .. })));

    // Second call should succeed
    let result = backend.complete(&[], &[]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_backend_tool_calls() {
    let backend = MockBackendBuilder::new()
        .with_tool_call("read_file", serde_json::json!({"path": "/test.txt"}))
        .build();

    let response = backend.complete(&[], &[]).await.unwrap();

    assert_eq!(response.tool_calls.len(), 1);
    assert_eq!(response.tool_calls[0].name, "read_file");
}

#[tokio::test]
async fn test_mock_backend_counts_calls() {
    let backend = MockBackend::always_returns("test");

    backend.complete(&[], &[]).await.unwrap();
    backend.complete(&[], &[]).await.unwrap();
    backend.complete(&[], &[]).await.unwrap();

    assert_eq!(backend.call_count(), 3);
}
```

---

## Testing Requirements

1. Mock backend implements full Backend trait
2. All response types work correctly
3. Request recording captures all details
4. Error simulation triggers correctly
5. Builder pattern creates properly configured mocks

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [051-backend-trait.md](../phase-03-backends/051-backend-trait.md)
- Next: [477-mock-filesystem.md](477-mock-filesystem.md)
- Related: [478-mock-network.md](478-mock-network.md)
