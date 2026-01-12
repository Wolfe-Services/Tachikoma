# 051d - Backend Trait Definition

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 051d
**Status:** Planned
**Dependencies:** 051c-backend-completion-types
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define the core `Backend` trait that all LLM providers implement, including capability querying and health checks.

---

## Acceptance Criteria

- [ ] `Backend` trait with async completion methods
- [ ] `BackendCapabilities` struct
- [ ] `BackendInfo` struct
- [ ] `BackendExt` extension trait with retry logic
- [ ] Send + Sync bounds for concurrent use

---

## Implementation Details

### 1. Backend Trait (src/backend.rs)

```rust
//! Core backend trait definition.

use crate::completion::{CompletionRequest, CompletionResponse};
use crate::error::BackendError;
use crate::stream::CompletionStream;
use async_trait::async_trait;
use std::fmt::Debug;

/// Capabilities supported by a backend.
#[derive(Debug, Clone, Default)]
pub struct BackendCapabilities {
    /// Supports streaming responses.
    pub streaming: bool,
    /// Supports tool/function calling.
    pub tool_calling: bool,
    /// Supports vision/images.
    pub vision: bool,
    /// Supports JSON mode.
    pub json_mode: bool,
    /// Maximum context window size.
    pub max_context_tokens: u32,
    /// Maximum output tokens.
    pub max_output_tokens: u32,
}

/// Information about a backend.
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Backend name (e.g., "claude", "codex").
    pub name: String,
    /// Backend version.
    pub version: String,
    /// Default model.
    pub default_model: String,
    /// Available models.
    pub available_models: Vec<String>,
    /// Backend capabilities.
    pub capabilities: BackendCapabilities,
}

/// Core trait for LLM backends.
///
/// This trait must be implemented by all LLM providers to enable
/// unified access to different AI models.
#[async_trait]
pub trait Backend: Send + Sync + Debug {
    /// Get backend information.
    fn info(&self) -> &BackendInfo;

    /// Get the backend name.
    fn name(&self) -> &str {
        &self.info().name
    }

    /// Get backend capabilities.
    fn capabilities(&self) -> &BackendCapabilities {
        &self.info().capabilities
    }

    /// Check if the backend supports a specific model.
    fn supports_model(&self, model: &str) -> bool {
        self.info().available_models.iter().any(|m| m == model)
    }

    /// Create a completion (non-streaming).
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, BackendError>;

    /// Create a streaming completion.
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError>;

    /// Check if the backend is healthy and accessible.
    async fn health_check(&self) -> Result<bool, BackendError>;

    /// Count tokens in text (approximate if not supported natively).
    fn count_tokens(&self, text: &str) -> u32 {
        (text.len() / 4) as u32
    }
}

/// Extension trait for backends with additional features.
#[async_trait]
pub trait BackendExt: Backend {
    /// Complete with automatic retries on transient errors.
    async fn complete_with_retry(
        &self,
        request: CompletionRequest,
        max_retries: u32,
    ) -> Result<CompletionResponse, BackendError> {
        let mut last_error = None;
        for attempt in 0..=max_retries {
            match self.complete(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if e.is_retryable() && attempt < max_retries => {
                    let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_error.unwrap())
    }
}

// Blanket implementation
impl<T: Backend> BackendExt for T {}
```

---

## Testing Requirements

1. Backend trait is object-safe
2. BackendExt retry logic works correctly
3. Capability checking works

---

## Related Specs

- Depends on: [051c-backend-completion-types.md](051c-backend-completion-types.md)
- Next: [051e-backend-stream-types.md](051e-backend-stream-types.md)
