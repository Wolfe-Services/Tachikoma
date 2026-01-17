# Spec 585: Multi-Provider Orchestrator

**Priority:** P0  
**Status:** planned  
**Depends on:** 576, 582, 584  
**Estimated Effort:** 4 hours  
**Target Files:**
- `crates/tachikoma-forge/src/orchestrator.rs` (update)
- `crates/tachikoma-forge/src/llm/openai.rs` (new)
- `crates/tachikoma-forge/src/llm/ollama.rs` (new)
- `crates/tachikoma-forge/src/llm/mod.rs` (update)

---

## Overview

The orchestrator routes each participant's requests to their configured LLM provider. This enables true multi-model deliberations where Claude, GPT-4, and Ollama models can debate each other.

---

## Acceptance Criteria

- [ ] Create `OpenAiProvider` implementing `LlmProvider` trait
- [ ] Create `OllamaProvider` implementing `LlmProvider` trait  
- [ ] Add `ProviderFactory::create(config: &ModelConfig) -> Box<dyn LlmProvider>`
- [ ] Update `ForgeOrchestrator` to use `ProviderFactory` per participant
- [ ] Each participant calls their own configured model
- [ ] Stream responses from all providers uniformly
- [ ] Handle provider-specific errors gracefully
- [ ] Add fallback logic: if provider fails, try next available
- [ ] Export all providers from llm/mod.rs
- [ ] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/llm/openai.rs

use super::provider::*;
use async_stream::stream;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

impl OpenAiProvider {
    pub fn new(model: &str, temperature: f32, max_tokens: u32) -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| LlmError::MissingApiKey("OPENAI_API_KEY"))?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            model: model.to_string(),
            temperature,
            max_tokens,
        })
    }
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str { "openai" }
    fn model(&self) -> &str { &self.model }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        let messages: Vec<_> = request.messages
            .into_iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                }.to_string(),
                content: m.content,
            })
            .collect();
        
        let api_request = OpenAiRequest {
            model: self.model.clone(),
            messages,
            temperature: request.temperature.unwrap_or(self.temperature),
            max_tokens: request.max_tokens.unwrap_or(self.max_tokens),
            stream: true,
        };
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&api_request)
            .send()
            .await
            .map_err(LlmError::NetworkError)?;
        
        let stream = stream! {
            let mut reader = response.bytes_stream();
            
            while let Some(chunk) = reader.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(LlmError::NetworkError(e));
                        return;
                    }
                };
                
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            yield Ok(LlmStreamChunk {
                                delta: String::new(),
                                is_complete: true,
                                finish_reason: Some("stop".to_string()),
                            });
                            return;
                        }
                        
                        // Parse OpenAI SSE format
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                                yield Ok(LlmStreamChunk {
                                    delta: content.to_string(),
                                    is_complete: false,
                                    finish_reason: None,
                                });
                            }
                        }
                    }
                }
            }
        };
        
        Ok(Box::pin(stream))
    }
    
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        // Similar to stream but with stream: false
        todo!("Implement non-streaming for OpenAI")
    }
}
```

```rust
// crates/tachikoma-forge/src/llm/ollama.rs

use super::provider::*;
use async_stream::stream;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(model: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: model.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: Option<String>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }
    fn model(&self) -> &str { &self.model }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        // Combine messages into prompt
        let system = request.messages.iter()
            .find(|m| matches!(m.role, MessageRole::System))
            .map(|m| m.content.clone());
        
        let prompt = request.messages.iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let api_request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            system,
            stream: true,
        };
        
        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&api_request)
            .send()
            .await
            .map_err(LlmError::NetworkError)?;
        
        let stream = stream! {
            let mut reader = response.bytes_stream();
            
            while let Some(chunk) = reader.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(LlmError::NetworkError(e));
                        return;
                    }
                };
                
                // Ollama uses newline-delimited JSON
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    if let Ok(parsed) = serde_json::from_str::<OllamaResponse>(line) {
                        yield Ok(LlmStreamChunk {
                            delta: parsed.response,
                            is_complete: parsed.done,
                            finish_reason: if parsed.done { Some("stop".to_string()) } else { None },
                        });
                        
                        if parsed.done {
                            return;
                        }
                    }
                }
            }
        };
        
        Ok(Box::pin(stream))
    }
    
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        todo!("Implement non-streaming for Ollama")
    }
}
```

```rust
// Add to crates/tachikoma-forge/src/llm/mod.rs

mod provider;
mod anthropic;
mod openai;
mod ollama;

pub use provider::*;
pub use anthropic::AnthropicProvider;
pub use openai::OpenAiProvider;
pub use ollama::OllamaProvider;

use crate::participant::{ModelConfig, LlmProvider as ProviderType};

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: &ModelConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
        match config.provider {
            ProviderType::Anthropic => {
                let provider = AnthropicProvider::new(&config.model_name)?;
                Ok(Box::new(provider))
            }
            ProviderType::OpenAi => {
                let provider = OpenAiProvider::new(
                    &config.model_name,
                    config.temperature,
                    config.max_tokens,
                )?;
                Ok(Box::new(provider))
            }
            ProviderType::Ollama => {
                let provider = OllamaProvider::new(&config.model_name);
                Ok(Box::new(provider))
            }
        }
    }
}
```
