# 136d - Forge Participant Types

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 136d
**Status:** Planned
**Dependencies:** 136c-forge-round-types
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define participant (model) types and model response types for Forge sessions.

---

## Acceptance Criteria

- [ ] `Participant` struct with model info
- [ ] `ModelProvider` enum
- [ ] `ParticipantRole` enum with role-specific prompts
- [ ] `ModelResponse` struct
- [ ] Helper constructors for common models

---

## Implementation Details

### 1. Participant Types (src/participant.rs)

```rust
//! Participant (model) types.

use serde::{Deserialize, Serialize};

/// A participant in a Forge session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Participant {
    pub model_id: String,
    pub display_name: String,
    pub provider: ModelProvider,
    pub role: ParticipantRole,
}

impl Participant {
    pub fn new(
        model_id: impl Into<String>,
        display_name: impl Into<String>,
        provider: ModelProvider,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            display_name: display_name.into(),
            provider,
            role: ParticipantRole::Generalist,
        }
    }

    pub fn with_role(mut self, role: ParticipantRole) -> Self {
        self.role = role;
        self
    }

    pub fn claude_opus() -> Self {
        Self::new("claude-3-opus-20240229", "Claude Opus", ModelProvider::Anthropic)
    }

    pub fn claude_sonnet() -> Self {
        Self::new("claude-3-5-sonnet-20241022", "Claude Sonnet", ModelProvider::Anthropic)
    }

    pub fn gpt4() -> Self {
        Self::new("gpt-4-turbo", "GPT-4 Turbo", ModelProvider::OpenAI)
    }

    pub fn gemini_pro() -> Self {
        Self::new("gemini-pro", "Gemini Pro", ModelProvider::Google)
    }
}

/// Model provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    Anthropic,
    OpenAI,
    Google,
    Local,
    Custom,
}

/// Role in the Forge session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Generalist,
    Drafter,
    Critic,
    Synthesizer,
    DomainExpert,
    CodeReviewer,
    DevilsAdvocate,
}

impl ParticipantRole {
    pub fn system_prompt_modifier(&self) -> &'static str {
        match self {
            Self::Generalist => "",
            Self::Drafter => "You are the primary drafter. Focus on creating comprehensive initial content.",
            Self::Critic => "You are a critic. Be thorough in identifying weaknesses and suggesting improvements.",
            Self::Synthesizer => "You are the synthesizer. Merge different perspectives into a coherent whole.",
            Self::DomainExpert => "You are a domain expert. Focus on technical accuracy and best practices.",
            Self::CodeReviewer => "You are a code reviewer. Focus on code quality, security, and maintainability.",
            Self::DevilsAdvocate => "You are the devil's advocate. Challenge assumptions and find edge cases.",
        }
    }
}
```

### 2. Response Types (src/response.rs)

```rust
//! Model response types.

use serde::{Deserialize, Serialize};
use tachikoma_common_core::Timestamp;

use crate::{Participant, TokenCount};

/// A response from a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub participant: Participant,
    pub content: String,
    pub tokens: TokenCount,
    pub duration_ms: u64,
    pub timestamp: Timestamp,
    pub stop_reason: StopReason,
}

impl ModelResponse {
    pub fn new(participant: Participant, content: String) -> Self {
        Self {
            participant,
            content,
            tokens: TokenCount::default(),
            duration_ms: 0,
            timestamp: Timestamp::now(),
            stop_reason: StopReason::EndTurn,
        }
    }

    pub fn with_tokens(mut self, input: u64, output: u64) -> Self {
        self.tokens = TokenCount { input, output };
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Why the model stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    Error,
}
```

---

## Testing Requirements

1. Participant creation with various providers
2. Role-specific prompts are available
3. Response types serialize correctly

---

## Related Specs

- Depends on: [136c-forge-round-types.md](136c-forge-round-types.md)
- Next: [137-forge-config.md](137-forge-config.md)
