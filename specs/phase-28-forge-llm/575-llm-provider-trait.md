# Spec 575: LLM Provider Trait

**Priority:** P0  
**Status:** planned  
**Estimated Effort:** 2 hours  
**Target Files:**
- `crates/tachikoma-forge/src/llm/mod.rs` (new)
- `crates/tachikoma-forge/src/llm/provider.rs` (new)
- `crates/tachikoma-forge/src/lib.rs` (update exports)
- `crates/tachikoma-forge/Cargo.toml` (add dependencies)

---

## Overview

Create the foundational LLM provider trait and types that all LLM implementations will use. This establishes the contract for streaming LLM responses.

---

## Acceptance Criteria

- [x] Add dependencies to `crates/tachikoma-forge/Cargo.toml`: `async-trait`, `futures`, `async-stream`, `reqwest` with `stream` feature, `tokio-stream`
- [x] Create `crates/tachikoma-forge/src/llm/mod.rs` that exports the module
- [x] Create `LlmProvider` trait with `complete()` and `complete_stream()` methods
- [x] Define `LlmRequest` struct with: model, messages, temperature, max_tokens, system_prompt
- [x] Define `LlmMessage` struct with: role (User/Assistant/System), content
- [x] Define `LlmResponse` with: content, role, finish_reason, usage (tokens)
- [x] Define `LlmStreamChunk` for streaming: delta (String), is_complete (bool), finish_reason
- [x] Define `LlmError` enum for: MissingApiKey, NetworkError, ParseError, RateLimited
- [x] Export `llm` module from `crates/tachikoma-forge/src/lib.rs`
- [x] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/llm/mod.rs
mod provider;
pub use provider::*;

// crates/tachikoma-forge/src/llm/provider.rs
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub finish_reason: Option<String>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct LlmStreamChunk {
    pub delta: String,
    pub is_complete: bool,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Missing API key: {0}")]
    MissingApiKey(&'static str),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Rate limited")]
    RateLimited,
}

pub type LlmStream = Pin<Box<dyn Stream<Item = Result<LlmStreamChunk, LlmError>> + Send>>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError>;
}
```

---

## Cargo.toml additions

```toml
[dependencies]
async-trait = "0.1"
async-stream = "0.3"
futures = "0.3"
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio-stream = "0.1"
thiserror = "1.0"
```
