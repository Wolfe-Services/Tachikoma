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
    
    pub fn gpt_4_turbo() -> Result<Self, LlmError> {
        Self::new("gpt-4-turbo", 0.7, 4096)
    }
    
    pub fn gpt_4o() -> Result<Self, LlmError> {
        Self::new("gpt-4o", 0.7, 4096)
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

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
    delta: Option<OpenAiDelta>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiChoice>,
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str { 
        "openai" 
    }
    
    fn model(&self) -> &str { 
        &self.model 
    }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        // Handle system prompt by converting to system message
        let mut messages: Vec<_> = Vec::new();
        
        if let Some(system_prompt) = request.system_prompt {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: system_prompt,
            });
        }
        
        // Add regular messages
        for message in request.messages {
            messages.push(OpenAiMessage {
                role: match message.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                }.to_string(),
                content: message.content,
            });
        }
        
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
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ParseError(format!("OpenAI API error {}: {}", status, text)));
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
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                yield Ok(LlmStreamChunk {
                                    delta: String::new(),
                                    is_complete: true,
                                    finish_reason: Some("stop".to_string()),
                                });
                                return;
                            }
                            
                            if let Ok(chunk) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(delta) = &choice.delta {
                                        if let Some(content) = &delta.content {
                                            yield Ok(LlmStreamChunk {
                                                delta: content.clone(),
                                                is_complete: false,
                                                finish_reason: None,
                                            });
                                        }
                                    }
                                    
                                    if let Some(finish_reason) = &choice.finish_reason {
                                        yield Ok(LlmStreamChunk {
                                            delta: String::new(),
                                            is_complete: true,
                                            finish_reason: Some(finish_reason.clone()),
                                        });
                                        return;
                                    }
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
        // Handle system prompt by converting to system message
        let mut messages: Vec<_> = Vec::new();
        
        if let Some(system_prompt) = request.system_prompt {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: system_prompt,
            });
        }
        
        // Add regular messages
        for message in request.messages {
            messages.push(OpenAiMessage {
                role: match message.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                }.to_string(),
                content: message.content,
            });
        }
        
        let api_request = OpenAiRequest {
            model: self.model.clone(),
            messages,
            temperature: request.temperature.unwrap_or(self.temperature),
            max_tokens: request.max_tokens.unwrap_or(self.max_tokens),
            stream: false,
        };
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&api_request)
            .send()
            .await
            .map_err(LlmError::NetworkError)?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ParseError(format!("OpenAI API error {}: {}", status, text)));
        }
        
        let openai_response: OpenAiResponse = response.json().await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        let choice = openai_response.choices.into_iter().next()
            .ok_or_else(|| LlmError::ParseError("No choices in OpenAI response".to_string()))?;
        
        let message = choice.message
            .ok_or_else(|| LlmError::ParseError("No message in OpenAI choice".to_string()))?;
        
        let usage = openai_response.usage
            .map(|u| TokenUsage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
            })
            .unwrap_or_default();
        
        Ok(LlmResponse {
            content: message.content,
            role: MessageRole::Assistant,
            finish_reason: choice.finish_reason,
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_openai_provider_creation() {
        // Test with missing API key
        std::env::remove_var("OPENAI_API_KEY");
        assert!(OpenAiProvider::new("gpt-4-turbo", 0.7, 4096).is_err());
        
        // Test with API key
        std::env::set_var("OPENAI_API_KEY", "test-key");
        let provider = OpenAiProvider::new("gpt-4-turbo", 0.7, 4096).unwrap();
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4-turbo");
        assert_eq!(provider.temperature, 0.7);
        assert_eq!(provider.max_tokens, 4096);
    }
    
    #[tokio::test]
    async fn test_convenience_constructors() {
        std::env::set_var("OPENAI_API_KEY", "test-key");
        
        let provider1 = OpenAiProvider::gpt_4_turbo().unwrap();
        assert_eq!(provider1.model(), "gpt-4-turbo");
        
        let provider2 = OpenAiProvider::gpt_4o().unwrap();
        assert_eq!(provider2.model(), "gpt-4o");
    }
    
    #[test]
    fn test_request_serialization() {
        let request = OpenAiRequest {
            model: "gpt-4-turbo".to_string(),
            messages: vec![
                OpenAiMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                }
            ],
            temperature: 0.7,
            max_tokens: 4096,
            stream: false,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4-turbo"));
        assert!(json.contains("Hello"));
        assert!(json.contains("0.7"));
    }
}