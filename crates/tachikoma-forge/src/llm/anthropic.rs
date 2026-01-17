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

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[serde(rename = "type")]
    response_type: String,
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
    #[serde(default)]
    message: Option<AnthropicMessage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    text: Option<String>,
    stop_reason: Option<String>,
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str { 
        "anthropic" 
    }
    
    fn model(&self) -> &str { 
        &self.model 
    }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        // Build messages, extracting system prompt
        let mut system_prompt = request.system_prompt;
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
            .await
            .map_err(LlmError::NetworkError)?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ParseError(format!("API error {}: {}", status, text)));
        }
        
        let stream = stream! {
            let mut reader = response.bytes_stream();
            let mut buffer = String::new();
            
            while let Some(chunk) = reader.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(LlmError::NetworkError(e));
                        return;
                    }
                };
                
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                
                while let Some(idx) = buffer.find("\n\n") {
                    let event_data = buffer[..idx].to_string();
                    buffer = buffer[idx + 2..].to_string();
                    
                    for line in event_data.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if json_str == "[DONE]" {
                                yield Ok(LlmStreamChunk {
                                    delta: String::new(),
                                    is_complete: true,
                                    finish_reason: Some("end_turn".to_string()),
                                });
                                return;
                            }
                            
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
                                    "message_delta" => {
                                        if let Some(delta) = event.delta {
                                            if let Some(stop_reason) = delta.stop_reason {
                                                yield Ok(LlmStreamChunk {
                                                    delta: String::new(),
                                                    is_complete: true,
                                                    finish_reason: Some(stop_reason),
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
        // Build messages, extracting system prompt
        let mut system_prompt = request.system_prompt;
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
            stream: false,
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&api_request)
            .send()
            .await
            .map_err(LlmError::NetworkError)?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ParseError(format!("API error {}: {}", status, text)));
        }
        
        let anthropic_response: AnthropicResponse = response.json().await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        let content = anthropic_response.content
            .into_iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");
        
        let usage = anthropic_response.usage
            .map(|u| TokenUsage {
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
            })
            .unwrap_or_default();
        
        Ok(LlmResponse {
            content,
            role: MessageRole::Assistant,
            finish_reason: anthropic_response.stop_reason,
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::TryStreamExt;
    
    #[tokio::test]
    async fn test_anthropic_provider_creation() {
        // Test with missing API key
        std::env::remove_var("ANTHROPIC_API_KEY");
        assert!(AnthropicProvider::new("claude-3-5-sonnet-20241022").is_err());
        
        // Test with API key
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        let provider = AnthropicProvider::new("claude-3-5-sonnet-20241022").unwrap();
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.model(), "claude-3-5-sonnet-20241022");
    }
    
    #[tokio::test]
    async fn test_convenience_constructors() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        
        let provider1 = AnthropicProvider::claude_3_5_sonnet().unwrap();
        assert_eq!(provider1.model(), "claude-3-5-sonnet-20241022");
        
        let provider2 = AnthropicProvider::claude_sonnet_4().unwrap();
        assert_eq!(provider2.model(), "claude-sonnet-4-20250514");
    }
    
    #[test]
    fn test_request_serialization() {
        let request = AnthropicRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            messages: vec![
                AnthropicMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                }
            ],
            system: Some("You are a helpful assistant".to_string()),
            stream: false,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-3-5-sonnet-20241022"));
        assert!(json.contains("Hello"));
        assert!(json.contains("helpful assistant"));
    }
    
    #[test]
    fn test_stream_event_deserialization() {
        let event_json = r#"{"type": "content_block_delta", "delta": {"text": "Hello"}}"#;
        let event: AnthropicStreamEvent = serde_json::from_str(event_json).unwrap();
        assert_eq!(event.event_type, "content_block_delta");
        assert_eq!(event.delta.unwrap().text.unwrap(), "Hello");
        
        let stop_json = r#"{"type": "message_stop"}"#;
        let stop_event: AnthropicStreamEvent = serde_json::from_str(stop_json).unwrap();
        assert_eq!(stop_event.event_type, "message_stop");
    }
}