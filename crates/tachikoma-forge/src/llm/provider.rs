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
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub role: MessageRole,
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
    NetworkError(#[from] reqwest::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
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