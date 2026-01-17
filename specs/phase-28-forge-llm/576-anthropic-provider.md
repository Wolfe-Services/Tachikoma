# Spec 576: Anthropic Provider

**Priority:** P0  
**Status:** planned  
**Depends on:** 575  
**Estimated Effort:** 3 hours  
**Target Files:**
- `crates/tachikoma-forge/src/llm/anthropic.rs` (new)
- `crates/tachikoma-forge/src/llm/mod.rs` (update)

---

## Overview

Implement the Anthropic (Claude) LLM provider with streaming support. This is the primary provider for Forge deliberations.

---

## Acceptance Criteria

- [x] Create `crates/tachikoma-forge/src/llm/anthropic.rs`
- [x] Implement `AnthropicProvider` struct with `client: reqwest::Client`, `api_key: String`, `model: String`
- [x] Add `AnthropicProvider::new(model: &str)` that reads `ANTHROPIC_API_KEY` env var
- [x] Add convenience constructors: `claude_sonnet_4()`, `claude_3_5_sonnet()`
- [x] Implement `LlmProvider::complete_stream()` with SSE parsing
- [x] Handle Anthropic event types: `content_block_delta`, `message_delta`, `message_stop`
- [x] Implement `LlmProvider::complete()` for non-streaming
- [x] Export `AnthropicProvider` from `llm/mod.rs`
- [x] Write a test that mocks the API response (don't call real API in tests)
- [x] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/llm/anthropic.rs

use super::provider::*;
use async_stream::stream;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(model: &str) -> Result<Self, LlmError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| LlmError::MissingApiKey("ANTHROPIC_API_KEY"))?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            model: model.to_string(),
        })
    }
    
    pub fn claude_sonnet_4() -> Result<Self, LlmError> {
        Self::new("claude-sonnet-4-20250514")
    }
    
    pub fn claude_3_5_sonnet() -> Result<Self, LlmError> {
        Self::new("claude-3-5-sonnet-20241022")
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    text: Option<String>,
    stop_reason: Option<String>,
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }
    fn model(&self) -> &str { &self.model }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        // Build messages, extracting system prompt
        let mut system_prompt = None;
        let messages: Vec<_> = request.messages
            .into_iter()
            .filter_map(|m| {
                match m.role {
                    MessageRole::System => {
                        system_prompt = Some(m.content);
                        None
                    }
                    MessageRole::User => Some(AnthropicMessage {
                        role: "user".to_string(),
                        content: m.content,
                    }),
                    MessageRole::Assistant => Some(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: m.content,
                    }),
                }
            })
            .collect();
        
        let api_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(4096),
            messages,
            system: system_prompt,
            stream: true,
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&api_request)
            .send()
            .await?;
        
        let stream = stream! {
            let mut reader = response.bytes_stream();
            let mut buffer = String::new();
            
            while let Some(chunk) = reader.next().await {
                let chunk = chunk.map_err(LlmError::Network)?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                
                while let Some(idx) = buffer.find("\n\n") {
                    let event_data = buffer[..idx].to_string();
                    buffer = buffer[idx + 2..].to_string();
                    
                    for line in event_data.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(json_str) {
                                match event.event_type.as_str() {
                                    "content_block_delta" => {
                                        if let Some(delta) = event.delta {
                                            if let Some(text) = delta.text {
                                                yield Ok(LlmStreamChunk {
                                                    delta: text,
                                                    is_complete: false,
                                                    finish_reason: None,
                                                });
                                            }
                                        }
                                    }
                                    "message_stop" => {
                                        yield Ok(LlmStreamChunk {
                                            delta: String::new(),
                                            is_complete: true,
                                            finish_reason: Some("end_turn".to_string()),
                                        });
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        };
        
        Ok(Box::pin(stream))
    }
    
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        use futures::TryStreamExt;
        
        let stream = self.complete_stream(request).await?;
        let chunks: Vec<_> = stream.try_collect().await?;
        
        let content = chunks.iter().map(|c| c.delta.as_str()).collect();
        let finish_reason = chunks.last().and_then(|c| c.finish_reason.clone());
        
        Ok(LlmResponse {
            content,
            finish_reason,
            usage: TokenUsage::default(), // TODO: parse from message_delta
        })
    }
}
```

---

## Update mod.rs

```rust
// crates/tachikoma-forge/src/llm/mod.rs
mod provider;
mod anthropic;

pub use provider::*;
pub use anthropic::AnthropicProvider;
```
