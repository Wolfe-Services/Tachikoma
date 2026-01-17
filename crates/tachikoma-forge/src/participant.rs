//! Participant configuration for multi-model brainstorming sessions.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A participant in a forge session with model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub display_name: String,
    pub role: ParticipantRole,
    pub model_config: ModelConfig,
    pub system_prompt: String,
    pub is_human: bool,
}

/// Roles that participants can take in forge sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Architect,      // Designs overall structure
    Critic,         // Finds flaws and risks  
    Advocate,       // Champions the solution
    Synthesizer,    // Combines perspectives
    Specialist,     // Domain expert
    Custom(String), // User-defined role
}

/// Configuration for LLM model settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: LlmProvider,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

/// Supported LLM providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    Anthropic,
    OpenAi,
    Ollama,
}

impl Participant {
    /// Create a new participant builder.
    pub fn builder(name: impl Into<String>) -> ParticipantBuilder {
        ParticipantBuilder::new(name)
    }
    
    /// Claude Sonnet analyst - fast, balanced reasoning
    pub fn claude_analyst(name: impl Into<String>) -> Self {
        Self::builder(name)
            .role(ParticipantRole::Architect)
            .anthropic("claude-sonnet-4-20250514")
            .system_prompt("You are a systems architect. Design elegant, maintainable solutions.")
            .build()
    }
    
    /// Claude Opus deep thinker - thorough analysis
    pub fn claude_critic(name: impl Into<String>) -> Self {
        Self::builder(name)
            .role(ParticipantRole::Critic)
            .anthropic("claude-3-opus-20240229")
            .temperature(0.3)
            .system_prompt("You are a critical reviewer. Find flaws, edge cases, and risks.")
            .build()
    }
    
    /// GPT-4 for diverse perspective
    pub fn gpt_advocate(name: impl Into<String>) -> Self {
        Self::builder(name)
            .role(ParticipantRole::Advocate)
            .openai("gpt-4-turbo")
            .system_prompt("You champion practical solutions. Focus on what works.")
            .build()
    }
    
    /// Human participant (no LLM calls)
    pub fn human(name: impl Into<String>, role: ParticipantRole) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            display_name: name.into(),
            role,
            model_config: ModelConfig::none(),
            system_prompt: String::new(),
            is_human: true,
        }
    }
}

/// Builder for creating participants with fluent interface.
pub struct ParticipantBuilder {
    display_name: String,
    role: ParticipantRole,
    model_config: ModelConfig,
    system_prompt: String,
}

impl ParticipantBuilder {
    /// Create a new participant builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            display_name: name.into(),
            role: ParticipantRole::Specialist,
            model_config: ModelConfig::default_claude(),
            system_prompt: String::new(),
        }
    }
    
    /// Set the participant role.
    pub fn role(mut self, role: ParticipantRole) -> Self {
        self.role = role;
        self
    }
    
    /// Configure for Anthropic model.
    pub fn anthropic(mut self, model: &str) -> Self {
        self.model_config = ModelConfig {
            provider: LlmProvider::Anthropic,
            model_name: model.to_string(),
            ..self.model_config
        };
        self
    }
    
    /// Configure for OpenAI model.
    pub fn openai(mut self, model: &str) -> Self {
        self.model_config = ModelConfig {
            provider: LlmProvider::OpenAi,
            model_name: model.to_string(),
            ..self.model_config
        };
        self
    }
    
    /// Configure for Ollama model.
    pub fn ollama(mut self, model: &str) -> Self {
        self.model_config = ModelConfig {
            provider: LlmProvider::Ollama,
            model_name: model.to_string(),
            ..self.model_config
        };
        self
    }
    
    /// Set temperature for model generation.
    pub fn temperature(mut self, temp: f32) -> Self {
        self.model_config.temperature = temp;
        self
    }
    
    /// Set maximum tokens for model generation.
    pub fn max_tokens(mut self, tokens: u32) -> Self {
        self.model_config.max_tokens = tokens;
        self
    }
    
    /// Set system prompt for the participant.
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }
    
    /// Build the participant.
    pub fn build(self) -> Participant {
        Participant {
            id: Uuid::new_v4().to_string(),
            display_name: self.display_name,
            role: self.role,
            model_config: self.model_config,
            system_prompt: self.system_prompt,
            is_human: false,
        }
    }
}

impl ModelConfig {
    /// Default Claude Sonnet configuration.
    pub fn default_claude() -> Self {
        Self {
            provider: LlmProvider::Anthropic,
            model_name: "claude-sonnet-4-20250514".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
    
    /// Empty configuration for human participants.
    pub fn none() -> Self {
        Self {
            provider: LlmProvider::Anthropic,
            model_name: String::new(),
            temperature: 0.0,
            max_tokens: 0,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::default_claude()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_participant_builder() {
        let participant = Participant::builder("Alice")
            .role(ParticipantRole::Critic)
            .anthropic("claude-3-opus-20240229")
            .temperature(0.3)
            .max_tokens(4096)
            .system_prompt("You are a thorough critic.")
            .build();

        assert_eq!(participant.name, "Alice");
        assert!(matches!(participant.role, ParticipantRole::Critic));
        assert!(matches!(participant.model_config.provider, LlmProvider::Anthropic));
        assert_eq!(participant.model_config.model_name, "claude-3-opus-20240229");
        assert_eq!(participant.model_config.temperature, 0.3);
        assert_eq!(participant.model_config.max_tokens, 4096);
        assert_eq!(participant.system_prompt, "You are a thorough critic.");
        assert!(!participant.is_human);
    }

    #[test]
    fn test_convenience_methods() {
        let analyst = Participant::claude_analyst("Alice");
        assert_eq!(analyst.name, "Alice");
        assert!(matches!(analyst.role, ParticipantRole::Architect));
        assert!(matches!(analyst.model_config.provider, LlmProvider::Anthropic));

        let critic = Participant::claude_critic("Bob");
        assert_eq!(critic.name, "Bob");
        assert!(matches!(critic.role, ParticipantRole::Critic));
        assert_eq!(critic.model_config.temperature, 0.3);

        let advocate = Participant::gpt_advocate("Charlie");
        assert_eq!(advocate.name, "Charlie");
        assert!(matches!(advocate.role, ParticipantRole::Advocate));
        assert!(matches!(advocate.model_config.provider, LlmProvider::OpenAi));
    }

    #[test]
    fn test_human_participant() {
        let human = Participant::human("David", ParticipantRole::Synthesizer);
        assert_eq!(human.name, "David");
        assert!(matches!(human.role, ParticipantRole::Synthesizer));
        assert!(human.is_human);
        assert_eq!(human.model_config.max_tokens, 0);
        assert!(human.system_prompt.is_empty());
    }

    #[test]
    fn test_multiple_providers() {
        let anthropic = Participant::builder("A").anthropic("claude-3-opus").build();
        assert!(matches!(anthropic.model_config.provider, LlmProvider::Anthropic));

        let openai = Participant::builder("B").openai("gpt-4").build();
        assert!(matches!(openai.model_config.provider, LlmProvider::OpenAi));

        let ollama = Participant::builder("C").ollama("llama2").build();
        assert!(matches!(ollama.model_config.provider, LlmProvider::Ollama));
    }

    #[test]
    fn test_custom_role() {
        let custom = Participant::builder("Expert")
            .role(ParticipantRole::Custom("Database Expert".to_string()))
            .build();
        
        assert!(matches!(custom.role, ParticipantRole::Custom(_)));
        if let ParticipantRole::Custom(ref role_name) = custom.role {
            assert_eq!(role_name, "Database Expert");
        }
    }

    #[test]
    fn test_independent_settings() {
        let low_temp = Participant::builder("Conservative")
            .temperature(0.1)
            .max_tokens(1000)
            .build();

        let high_temp = Participant::builder("Creative")
            .temperature(0.9)
            .max_tokens(3000)
            .build();

        assert_eq!(low_temp.model_config.temperature, 0.1);
        assert_eq!(low_temp.model_config.max_tokens, 1000);
        assert_eq!(high_temp.model_config.temperature, 0.9);
        assert_eq!(high_temp.model_config.max_tokens, 3000);
    }

    #[test]
    fn test_serialization() {
        let participant = Participant::claude_analyst("Test");
        
        // Test JSON serialization
        let json = serde_json::to_string(&participant).unwrap();
        let deserialized: Participant = serde_json::from_str(&json).unwrap();
        
        assert_eq!(participant.name, deserialized.name);
        assert_eq!(participant.model_config.model_name, deserialized.model_config.model_name);
    }
}