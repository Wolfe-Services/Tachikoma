# Spec 582: Participant Model Configuration

**Priority:** P0  
**Status:** planned  
**Depends on:** 576  
**Estimated Effort:** 3 hours  
**Target Files:**
- `crates/tachikoma-forge/src/participant.rs` (new)
- `crates/tachikoma-forge/src/lib.rs` (update)

---

## Overview

Each Think Tank participant can use a different LLM model. This allows diverse perspectives:
- Claude Sonnet 4 for fast iteration
- Claude Opus for deep reasoning  
- GPT-4 for different training biases
- Ollama for local/free testing

---

## Acceptance Criteria

- [ ] Create `Participant` struct with: id, name, role, model_config, system_prompt
- [ ] Create `ModelConfig` struct with: provider (anthropic/openai/ollama), model_name, temperature, max_tokens
- [ ] Add `ParticipantBuilder` for fluent construction
- [ ] Support at least 3 providers: Anthropic, OpenAI, Ollama
- [ ] Each participant can have independent temperature/token settings
- [ ] Add convenience methods: `Participant::claude_analyst()`, `Participant::gpt_critic()`, etc.
- [ ] Export from lib.rs
- [ ] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/participant.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub name: String,
    pub role: ParticipantRole,
    pub model_config: ModelConfig,
    pub system_prompt: String,
    pub is_human: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Architect,      // Designs overall structure
    Critic,         // Finds flaws and risks  
    Advocate,       // Champions the solution
    Synthesizer,    // Combines perspectives
    Specialist,     // Domain expert
    Custom(String), // User-defined role
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: LlmProvider,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    Anthropic,
    OpenAi,
    Ollama,
}

impl Participant {
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
            id: Uuid::new_v4(),
            name: name.into(),
            role,
            model_config: ModelConfig::none(),
            system_prompt: String::new(),
            is_human: true,
        }
    }
}

pub struct ParticipantBuilder {
    name: String,
    role: ParticipantRole,
    model_config: ModelConfig,
    system_prompt: String,
}

impl ParticipantBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            role: ParticipantRole::Specialist,
            model_config: ModelConfig::default_claude(),
            system_prompt: String::new(),
        }
    }
    
    pub fn role(mut self, role: ParticipantRole) -> Self {
        self.role = role;
        self
    }
    
    pub fn anthropic(mut self, model: &str) -> Self {
        self.model_config = ModelConfig {
            provider: LlmProvider::Anthropic,
            model_name: model.to_string(),
            ..self.model_config
        };
        self
    }
    
    pub fn openai(mut self, model: &str) -> Self {
        self.model_config = ModelConfig {
            provider: LlmProvider::OpenAi,
            model_name: model.to_string(),
            ..self.model_config
        };
        self
    }
    
    pub fn ollama(mut self, model: &str) -> Self {
        self.model_config = ModelConfig {
            provider: LlmProvider::Ollama,
            model_name: model.to_string(),
            ..self.model_config
        };
        self
    }
    
    pub fn temperature(mut self, temp: f32) -> Self {
        self.model_config.temperature = temp;
        self
    }
    
    pub fn max_tokens(mut self, tokens: u32) -> Self {
        self.model_config.max_tokens = tokens;
        self
    }
    
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }
    
    pub fn build(self) -> Participant {
        Participant {
            id: Uuid::new_v4(),
            name: self.name,
            role: self.role,
            model_config: self.model_config,
            system_prompt: self.system_prompt,
            is_human: false,
        }
    }
}

impl ModelConfig {
    pub fn default_claude() -> Self {
        Self {
            provider: LlmProvider::Anthropic,
            model_name: "claude-sonnet-4-20250514".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
    
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
```
