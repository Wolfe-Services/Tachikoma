# 075 - Backend Integration Tests

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 075
**Status:** Planned
**Dependencies:** All previous Phase 3 specs
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define comprehensive integration tests for the backend abstraction layer, including mock backends for testing, provider-specific tests, and end-to-end testing scenarios.

---

## Acceptance Criteria

- [x] Mock backend for testing
- [x] Unit tests for all components
- [x] Integration tests for each provider
- [x] End-to-end streaming tests
- [x] Tool calling tests
- [x] Error handling tests

---

## Implementation Details

### 1. Mock Backend (tests/common/mock_backend.rs)

```rust
//! Mock backend for testing.

use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tachikoma_backends_core::{
    Backend, BackendCapabilities, BackendError, BackendInfo,
    CompletionChunk, CompletionRequest, CompletionResponse, CompletionStream,
    FinishReason, Message, ToolCall, Usage,
};

/// Mock backend for testing.
#[derive(Debug)]
pub struct MockBackend {
    info: BackendInfo,
    responses: Arc<Mutex<VecDeque<MockResponse>>>,
    requests: Arc<Mutex<Vec<CompletionRequest>>>,
    should_fail: Arc<Mutex<Option<BackendError>>>,
}

/// A mock response.
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
}

impl MockResponse {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            tool_calls: vec![],
            usage: Usage::new(100, 50),
        }
    }

    pub fn with_tool_call(mut self, call: ToolCall) -> Self {
        self.tool_calls.push(call);
        self
    }
}

impl MockBackend {
    /// Create a new mock backend.
    pub fn new() -> Self {
        Self {
            info: BackendInfo {
                name: "mock".to_string(),
                version: "test".to_string(),
                default_model: "mock-model".to_string(),
                available_models: vec!["mock-model".to_string()],
                capabilities: BackendCapabilities {
                    streaming: true,
                    tool_calling: true,
                    vision: false,
                    json_mode: true,
                    max_context_tokens: 100_000,
                    max_output_tokens: 4096,
                },
            },
            responses: Arc::new(Mutex::new(VecDeque::new())),
            requests: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(None)),
        }
    }

    /// Queue a response.
    pub fn queue_response(&self, response: MockResponse) {
        self.responses.lock().unwrap().push_back(response);
    }

    /// Set to always fail with error.
    pub fn set_failure(&self, error: BackendError) {
        *self.should_fail.lock().unwrap() = Some(error);
    }

    /// Clear failure.
    pub fn clear_failure(&self) {
        *self.should_fail.lock().unwrap() = None;
    }

    /// Get recorded requests.
    pub fn get_requests(&self) -> Vec<CompletionRequest> {
        self.requests.lock().unwrap().clone()
    }

    /// Get last request.
    pub fn last_request(&self) -> Option<CompletionRequest> {
        self.requests.lock().unwrap().last().cloned()
    }

    /// Clear recorded requests.
    pub fn clear_requests(&self) {
        self.requests.lock().unwrap().clear();
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Backend for MockBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        // Check for forced failure
        if let Some(error) = self.should_fail.lock().unwrap().clone() {
            return Err(error);
        }

        // Record request
        self.requests.lock().unwrap().push(request.clone());

        // Get queued response or default
        let response = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| MockResponse::text("Mock response"));

        Ok(CompletionResponse {
            content: response.content,
            tool_calls: response.tool_calls,
            finish_reason: FinishReason::Stop,
            usage: response.usage,
            model: "mock-model".to_string(),
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError> {
        // For simplicity, use non-streaming internally
        let response = self.complete(request).await?;

        // Convert to stream
        let chunks = vec![
            Ok(CompletionChunk::text(response.content.unwrap_or_default())),
            Ok(CompletionChunk::final_chunk(response.usage, response.finish_reason)),
        ];

        Ok(Box::pin(futures::stream::iter(chunks)))
    }

    async fn health_check(&self) -> Result<bool, BackendError> {
        if self.should_fail.lock().unwrap().is_some() {
            return Ok(false);
        }
        Ok(true)
    }
}
```

### 2. Backend Trait Tests (tests/backend_trait_tests.rs)

```rust
//! Tests for the Backend trait.

use tachikoma_backends_core::*;
use common::mock_backend::{MockBackend, MockResponse};

mod common;

#[tokio::test]
async fn test_basic_completion() {
    let backend = MockBackend::new();
    backend.queue_response(MockResponse::text("Hello, world!"));

    let request = CompletionRequest::new(vec![Message::user("Hi")]);
    let response = backend.complete(request).await.unwrap();

    assert_eq!(response.content, Some("Hello, world!".to_string()));
    assert_eq!(response.finish_reason, FinishReason::Stop);
}

#[tokio::test]
async fn test_tool_calling() {
    let backend = MockBackend::new();
    backend.queue_response(
        MockResponse::text("")
            .with_tool_call(ToolCall {
                id: "call_1".to_string(),
                name: "read_file".to_string(),
                arguments: r#"{"path": "/tmp/test"}"#.to_string(),
            }),
    );

    let request = CompletionRequest::new(vec![Message::user("Read /tmp/test")]);
    let response = backend.complete(request).await.unwrap();

    assert_eq!(response.tool_calls.len(), 1);
    assert_eq!(response.tool_calls[0].name, "read_file");
}

#[tokio::test]
async fn test_streaming() {
    use futures::StreamExt;

    let backend = MockBackend::new();
    backend.queue_response(MockResponse::text("Streaming response"));

    let request = CompletionRequest::new(vec![Message::user("Stream test")]);
    let mut stream = backend.complete_stream(request).await.unwrap();

    let mut content = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        content.push_str(&chunk.delta);
    }

    assert!(content.contains("Streaming"));
}

#[tokio::test]
async fn test_error_handling() {
    let backend = MockBackend::new();
    backend.set_failure(BackendError::RateLimit {
        retry_after: Some(std::time::Duration::from_secs(60)),
        message: "Rate limited".to_string(),
    });

    let request = CompletionRequest::new(vec![Message::user("Test")]);
    let result = backend.complete(request).await;

    assert!(matches!(result, Err(BackendError::RateLimit { .. })));
}

#[tokio::test]
async fn test_health_check() {
    let backend = MockBackend::new();

    assert!(backend.health_check().await.unwrap());

    backend.set_failure(BackendError::Network("Connection failed".to_string()));
    assert!(!backend.health_check().await.unwrap());
}

#[tokio::test]
async fn test_request_recording() {
    let backend = MockBackend::new();
    backend.queue_response(MockResponse::text("Response"));

    let request = CompletionRequest::new(vec![
        Message::system("You are helpful"),
        Message::user("Hello"),
    ]);
    backend.complete(request).await.unwrap();

    let recorded = backend.last_request().unwrap();
    assert_eq!(recorded.messages.len(), 2);
}
```

### 3. Tool Definition Tests (tests/tool_tests.rs)

```rust
//! Tests for tool definitions and calling.

use tachikoma_backends_core::*;

#[test]
fn test_tool_definition_builder() {
    let tool = ToolDefinition::builder("test_tool")
        .description("A test tool")
        .required_string("input", "The input string")
        .optional_int("count", "Number of items")
        .build()
        .unwrap();

    assert_eq!(tool.name, "test_tool");
    assert!(tool.parameters.required.contains(&"input".to_string()));
    assert!(!tool.parameters.required.contains(&"count".to_string()));
}

#[test]
fn test_tool_json_schema() {
    let tool = ToolDefinition::builder("read_file")
        .description("Read a file")
        .required_string("path", "File path")
        .build()
        .unwrap();

    let schema = tool.parameters.to_json_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["path"].is_object());
}

#[test]
fn test_tool_call_parsing() {
    let call = ToolCall {
        id: "call_1".to_string(),
        name: "test".to_string(),
        arguments: r#"{"name": "test", "count": 5}"#.to_string(),
    };

    assert_eq!(call.get_string("name").unwrap(), "test");
    assert_eq!(call.get_int("count").unwrap(), 5);
}

#[test]
fn test_tool_result_formatting() {
    let result = ToolResult::success("call_1", "test", "Success!");

    assert!(result.success);
    assert_eq!(result.content.as_text(), "Success!");
}

#[test]
fn test_predefined_tools() {
    let tools = all_predefined_tools();
    assert!(!tools.is_empty());

    // Check file_read tool
    let read_tool = tools.iter().find(|t| t.name == "read_file").unwrap();
    assert!(read_tool.parameters.required.contains(&"path".to_string()));
}
```

### 4. Provider Integration Tests (tests/provider_tests.rs)

```rust
//! Integration tests for specific providers.
//! These tests require environment variables to be set.

use tachikoma_backends_core::*;

/// Skip test if env var not set.
macro_rules! require_env {
    ($var:expr) => {
        if std::env::var($var).is_err() {
            eprintln!("Skipping test: {} not set", $var);
            return;
        }
    };
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_claude_integration() {
    require_env!("ANTHROPIC_API_KEY");

    use tachikoma_backend_claude::{ClaudeBackend, ClaudeBackendConfig, ClaudeModel};
    use tachikoma_common_config::Secret;

    let config = ClaudeBackendConfig {
        api_key: Secret::new(std::env::var("ANTHROPIC_API_KEY").unwrap()),
        base_url: "https://api.anthropic.com".to_string(),
        model: ClaudeModel::Haiku35, // Use cheapest model
        api_version: "2023-06-01".to_string(),
        max_tokens: 100,
    };

    let backend = ClaudeBackend::new(config).unwrap();

    // Test health check
    assert!(backend.health_check().await.unwrap());

    // Test completion
    let request = CompletionRequest::new(vec![Message::user("Say hi")])
        .with_max_tokens(10);
    let response = backend.complete(request).await.unwrap();

    assert!(response.content.is_some());
}

#[tokio::test]
#[ignore]
async fn test_openai_integration() {
    require_env!("OPENAI_API_KEY");

    use tachikoma_backend_codex::{CodexBackend, CodexBackendConfig, OpenAIModel};
    use tachikoma_common_config::Secret;

    let config = CodexBackendConfig {
        api_key: Secret::new(std::env::var("OPENAI_API_KEY").unwrap()),
        organization: None,
        base_url: "https://api.openai.com".to_string(),
        model: OpenAIModel::Gpt4oMini, // Use cheapest model
        max_tokens: 100,
    };

    let backend = CodexBackend::new(config).unwrap();
    assert!(backend.health_check().await.unwrap());
}

#[tokio::test]
#[ignore]
async fn test_ollama_integration() {
    use tachikoma_backend_ollama::{OllamaBackend, OllamaBackendConfig, OllamaServerConfig};

    let server_config = OllamaServerConfig::default();
    let backend_config = OllamaBackendConfig {
        model: "llama3.1:8b".to_string(),
        ..Default::default()
    };

    let backend = match OllamaBackend::new(server_config, backend_config).await {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Skipping test: Ollama not running");
            return;
        }
    };

    assert!(backend.health_check().await.unwrap());
}
```

### 5. Rate Limiting Tests (tests/rate_limit_tests.rs)

```rust
//! Tests for rate limiting.

use tachikoma_backends_core::rate_limit::*;
use std::time::Duration;

#[tokio::test]
async fn test_rate_limiter_allows_requests() {
    let config = RateLimitConfig {
        requests_per_minute: 10,
        tokens_per_minute: 10000,
        max_concurrent: 5,
        burst_allowance: 2,
    };

    let limiter = RateLimiter::new(config);

    // Should allow first request
    let decision = limiter.check(100).await;
    assert!(matches!(decision, RateLimitDecision::Allow));
}

#[tokio::test]
async fn test_rate_limiter_blocks_on_limit() {
    let config = RateLimitConfig {
        requests_per_minute: 2,
        tokens_per_minute: 10000,
        max_concurrent: 5,
        burst_allowance: 0,
    };

    let limiter = RateLimiter::new(config);

    // Use up requests
    let _ = limiter.acquire(100).await.unwrap();
    let _ = limiter.acquire(100).await.unwrap();

    // Third should wait
    let decision = limiter.check(100).await;
    assert!(matches!(decision, RateLimitDecision::Wait(_)));
}

#[tokio::test]
async fn test_rate_limiter_token_limit() {
    let config = RateLimitConfig {
        requests_per_minute: 100,
        tokens_per_minute: 500,
        max_concurrent: 5,
        burst_allowance: 0,
    };

    let limiter = RateLimiter::new(config);

    // Request that would exceed token limit
    let decision = limiter.check(600).await;
    assert!(matches!(decision, RateLimitDecision::Wait(_)));
}

#[tokio::test]
async fn test_adaptive_rate_limiter() {
    let config = RateLimitConfig::claude_default();
    let limiter = AdaptiveRateLimiter::new(config);

    // Record some successes
    for _ in 0..10 {
        limiter.record_success();
    }

    // Record rate limit
    limiter.record_rate_limit(Some(Duration::from_secs(60))).await;

    let stats = limiter.stats();
    assert!(stats.multiplier < 1.0);
}
```

### 6. Context Tracking Tests (tests/context_tests.rs)

```rust
//! Tests for context tracking.

use tachikoma_backends_core::context::*;
use tachikoma_backends_core::tokens::*;
use tachikoma_backends_core::Message;
use std::sync::Arc;

#[test]
fn test_context_usage_calculation() {
    let usage = ContextUsage::new(50000, 100000);

    assert_eq!(usage.usage_percent, 50.0);
    assert_eq!(usage.tokens_remaining, 50000);
    assert_eq!(usage.state, ContextState::Elevated);
}

#[test]
fn test_context_state_thresholds() {
    assert_eq!(ContextState::from_percentage(40.0), ContextState::Nominal);
    assert_eq!(ContextState::from_percentage(60.0), ContextState::Elevated);
    assert_eq!(ContextState::from_percentage(80.0), ContextState::Warning);
    assert_eq!(ContextState::from_percentage(90.0), ContextState::Critical);
    assert_eq!(ContextState::from_percentage(100.0), ContextState::Redlined);
}

#[test]
fn test_context_tracker() {
    let counter: Arc<dyn TokenCounter> = Arc::new(SimpleTokenCounter::new());
    let config = ContextConfig::default();
    let mut tracker = ContextTracker::new(1000, counter, config);

    // Add some messages
    tracker.add_message(Message::user("Hello"));
    tracker.add_message(Message::assistant("Hi there!"));

    let usage = tracker.current_usage();
    assert!(usage.tokens_used > 0);
    assert_eq!(tracker.message_count(), 2);
}

#[test]
fn test_context_compression() {
    let counter: Arc<dyn TokenCounter> = Arc::new(SimpleTokenCounter::new());
    let config = ContextConfig::default();
    let mut tracker = ContextTracker::new(100, counter, config);

    // Add many messages to exceed threshold
    for i in 0..20 {
        tracker.add_message(Message::user(format!("Message {}", i)));
    }

    let before = tracker.current_usage().tokens_used;
    tracker.compress_to_threshold(50.0);
    let after = tracker.current_usage().tokens_used;

    assert!(after < before);
}
```

### 7. Module Exports (tests/lib.rs)

```rust
//! Integration tests for tachikoma-backends.

mod common;
mod backend_trait_tests;
mod tool_tests;
mod rate_limit_tests;
mod context_tests;

#[cfg(feature = "integration")]
mod provider_tests;
```

---

## Testing Requirements

1. All unit tests pass
2. Mock backend provides predictable behavior
3. Integration tests work with real APIs (when available)
4. Error conditions are properly tested
5. Concurrent usage is tested

---

## Related Specs

- Depends on: All Phase 3 specs
- Used by: Phase 4 (Orchestration layer)
- Next Phase: [Phase 4 - Orchestration](../phase-04-orchestration/)
