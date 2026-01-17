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
    
    pub fn llama3_8b() -> Self {
        Self::new("llama3:8b")
    }
    
    pub fn mistral() -> Self {
        Self::new("mistral")
    }
    
    pub fn codellama() -> Self {
        Self::new("codellama")
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str { 
        "ollama" 
    }
    
    fn model(&self) -> &str { 
        &self.model 
    }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        // Combine messages into prompt format for Ollama
        let system = request.system_prompt.or_else(|| {
            request.messages.iter()
                .find(|m| matches!(m.role, MessageRole::System))
                .map(|m| m.content.clone())
        });
        
        let prompt = request.messages.iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|m| match m.role {
                MessageRole::User => format!("User: {}", m.content),
                MessageRole::Assistant => format!("Assistant: {}", m.content),
                MessageRole::System => m.content.clone(),
            })
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
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ParseError(format!("Ollama API error {}: {}", status, text)));
        }
        
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
                    if line.trim().is_empty() {
                        continue;
                    }
                    
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
        // Combine messages into prompt format for Ollama
        let system = request.system_prompt.or_else(|| {
            request.messages.iter()
                .find(|m| matches!(m.role, MessageRole::System))
                .map(|m| m.content.clone())
        });
        
        let prompt = request.messages.iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|m| match m.role {
                MessageRole::User => format!("User: {}", m.content),
                MessageRole::Assistant => format!("Assistant: {}", m.content),
                MessageRole::System => m.content.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let api_request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            system,
            stream: false,
        };
        
        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&api_request)
            .send()
            .await
            .map_err(LlmError::NetworkError)?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ParseError(format!("Ollama API error {}: {}", status, text)));
        }
        
        let ollama_response: OllamaResponse = response.json().await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        let usage = TokenUsage {
            input_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
            output_tokens: ollama_response.eval_count.unwrap_or(0),
        };
        
        Ok(LlmResponse {
            content: ollama_response.response,
            role: MessageRole::Assistant,
            finish_reason: Some("stop".to_string()),
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new("llama3:8b");
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.model(), "llama3:8b");
        assert_eq!(provider.base_url, "http://localhost:11434");
    }
    
    #[test]
    fn test_custom_base_url() {
        std::env::set_var("OLLAMA_BASE_URL", "http://custom:11434");
        let provider = OllamaProvider::new("mistral");
        assert_eq!(provider.base_url, "http://custom:11434");
        std::env::remove_var("OLLAMA_BASE_URL");
    }
    
    #[test]
    fn test_convenience_constructors() {
        let provider1 = OllamaProvider::llama3_8b();
        assert_eq!(provider1.model(), "llama3:8b");
        
        let provider2 = OllamaProvider::mistral();
        assert_eq!(provider2.model(), "mistral");
        
        let provider3 = OllamaProvider::codellama();
        assert_eq!(provider3.model(), "codellama");
    }
    
    #[test]
    fn test_request_serialization() {
        let request = OllamaRequest {
            model: "llama3:8b".to_string(),
            prompt: "Hello world".to_string(),
            system: Some("You are helpful".to_string()),
            stream: false,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("llama3:8b"));
        assert!(json.contains("Hello world"));
        assert!(json.contains("helpful"));
    }
}