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