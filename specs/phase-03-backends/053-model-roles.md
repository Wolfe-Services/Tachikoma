# 053 - Model Roles (Brain/ThinkTank Abstraction)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 053
**Status:** Planned
**Dependencies:** 051-backend-trait, 052-backend-config
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define the Brain and ThinkTank abstractions that assign semantic roles to LLM backends. The "Brain" is the primary reasoning model (Claude Opus/Sonnet), while "ThinkTank" members are specialized models for critique, synthesis, and alternative perspectives.

---

## Acceptance Criteria

- [x] `ModelRole` enum (Brain, Critic, Synthesizer, etc.)
- [x] `Brain` struct wrapping primary backend
- [x] `ThinkTank` managing multiple model instances
- [x] Role-based model selection
- [x] Conversation routing based on role
- [x] Model capability matching for roles

---

## Implementation Details

### 1. Role Types (src/roles/mod.rs)

```rust
//! Model role definitions for Tachikoma's multi-model architecture.

use serde::{Deserialize, Serialize};

/// Semantic role a model plays in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    /// Primary reasoning model - handles main task execution.
    Brain,
    /// Critical analysis - identifies flaws and improvements.
    Critic,
    /// Synthesis - combines multiple perspectives.
    Synthesizer,
    /// Alternative viewpoint - devil's advocate.
    Contrarian,
    /// Specialist in a specific domain.
    DomainExpert,
    /// Quick, low-latency responses.
    FastResponder,
    /// Code-specialized model.
    Coder,
    /// Orchestrator for multi-agent coordination.
    Orchestrator,
}

impl ModelRole {
    /// Get recommended capabilities for this role.
    pub fn recommended_capabilities(&self) -> RoleCapabilities {
        match self {
            Self::Brain => RoleCapabilities {
                min_context_tokens: 100_000,
                requires_tool_calling: true,
                requires_streaming: true,
                prefers_high_reasoning: true,
                prefers_low_latency: false,
            },
            Self::Critic => RoleCapabilities {
                min_context_tokens: 32_000,
                requires_tool_calling: false,
                requires_streaming: false,
                prefers_high_reasoning: true,
                prefers_low_latency: false,
            },
            Self::Synthesizer => RoleCapabilities {
                min_context_tokens: 64_000,
                requires_tool_calling: false,
                requires_streaming: true,
                prefers_high_reasoning: true,
                prefers_low_latency: false,
            },
            Self::Contrarian => RoleCapabilities {
                min_context_tokens: 16_000,
                requires_tool_calling: false,
                requires_streaming: false,
                prefers_high_reasoning: false,
                prefers_low_latency: true,
            },
            Self::DomainExpert => RoleCapabilities {
                min_context_tokens: 32_000,
                requires_tool_calling: true,
                requires_streaming: false,
                prefers_high_reasoning: true,
                prefers_low_latency: false,
            },
            Self::FastResponder => RoleCapabilities {
                min_context_tokens: 8_000,
                requires_tool_calling: false,
                requires_streaming: true,
                prefers_high_reasoning: false,
                prefers_low_latency: true,
            },
            Self::Coder => RoleCapabilities {
                min_context_tokens: 64_000,
                requires_tool_calling: true,
                requires_streaming: true,
                prefers_high_reasoning: true,
                prefers_low_latency: false,
            },
            Self::Orchestrator => RoleCapabilities {
                min_context_tokens: 32_000,
                requires_tool_calling: true,
                requires_streaming: false,
                prefers_high_reasoning: true,
                prefers_low_latency: false,
            },
        }
    }

    /// Get the default system prompt modifier for this role.
    pub fn system_prompt_modifier(&self) -> &'static str {
        match self {
            Self::Brain => "",
            Self::Critic => "Your role is to critically analyze proposals and identify potential issues, edge cases, and improvements. Be thorough but constructive.",
            Self::Synthesizer => "Your role is to synthesize multiple perspectives into a coherent whole, resolving conflicts and finding common ground.",
            Self::Contrarian => "Your role is to play devil's advocate, challenging assumptions and exploring alternative approaches.",
            Self::DomainExpert => "You are a domain expert. Focus on technical accuracy and best practices in your area of expertise.",
            Self::FastResponder => "Provide concise, direct responses. Prioritize speed over exhaustive analysis.",
            Self::Coder => "You are a coding specialist. Focus on writing clean, efficient, well-documented code.",
            Self::Orchestrator => "You coordinate multiple agents. Focus on task decomposition and result aggregation.",
        }
    }
}

/// Capability requirements for a role.
#[derive(Debug, Clone, Copy)]
pub struct RoleCapabilities {
    /// Minimum context window size.
    pub min_context_tokens: u32,
    /// Whether tool calling is required.
    pub requires_tool_calling: bool,
    /// Whether streaming is required.
    pub requires_streaming: bool,
    /// Whether high reasoning ability is preferred.
    pub prefers_high_reasoning: bool,
    /// Whether low latency is preferred.
    pub prefers_low_latency: bool,
}
```

### 2. Brain Abstraction (src/roles/brain.rs)

```rust
//! Brain - the primary reasoning model.

use crate::{
    Backend, BackendError, CompletionRequest, CompletionResponse, CompletionStream,
};
use super::{ModelRole, RoleCapabilities};
use std::sync::Arc;

/// The Brain is the primary reasoning model for task execution.
///
/// It handles the main conversation flow, tool calling, and decision making.
/// Typically backed by Claude Opus or Sonnet.
#[derive(Debug)]
pub struct Brain {
    /// Underlying backend.
    backend: Arc<dyn Backend>,
    /// System prompt for the brain.
    system_prompt: String,
    /// Current context usage.
    context_tokens: u32,
    /// Maximum context before redline.
    max_context: u32,
    /// Redline threshold (percentage).
    redline_threshold: f32,
}

impl Brain {
    /// Create a new Brain with the given backend.
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        let capabilities = backend.capabilities();
        Self {
            backend,
            system_prompt: String::new(),
            context_tokens: 0,
            max_context: capabilities.max_context_tokens,
            redline_threshold: 0.85,
        }
    }

    /// Set the system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Set the redline threshold (0.0 - 1.0).
    pub fn with_redline_threshold(mut self, threshold: f32) -> Self {
        self.redline_threshold = threshold.clamp(0.5, 0.95);
        self
    }

    /// Get the role.
    pub fn role(&self) -> ModelRole {
        ModelRole::Brain
    }

    /// Get role capabilities.
    pub fn capabilities(&self) -> RoleCapabilities {
        self.role().recommended_capabilities()
    }

    /// Check if context is approaching redline.
    pub fn is_approaching_redline(&self) -> bool {
        let threshold = (self.max_context as f32 * self.redline_threshold) as u32;
        self.context_tokens >= threshold
    }

    /// Get current context usage percentage.
    pub fn context_usage(&self) -> f32 {
        self.context_tokens as f32 / self.max_context as f32
    }

    /// Get remaining context tokens.
    pub fn remaining_context(&self) -> u32 {
        self.max_context.saturating_sub(self.context_tokens)
    }

    /// Update context token count.
    pub fn update_context(&mut self, tokens: u32) {
        self.context_tokens = tokens;
    }

    /// Reset context (after conversation reset).
    pub fn reset_context(&mut self) {
        self.context_tokens = 0;
    }

    /// Complete a request.
    pub async fn complete(&mut self, mut request: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        // Prepend system prompt if set and not already present
        if !self.system_prompt.is_empty() {
            let has_system = request.messages.first()
                .map(|m| m.role == crate::Role::System)
                .unwrap_or(false);

            if !has_system {
                request.messages.insert(0, crate::Message::system(&self.system_prompt));
            }
        }

        let response = self.backend.complete(request).await?;
        self.context_tokens = response.usage.total_tokens;
        Ok(response)
    }

    /// Complete with streaming.
    pub async fn complete_stream(&mut self, mut request: CompletionRequest) -> Result<CompletionStream, BackendError> {
        if !self.system_prompt.is_empty() {
            let has_system = request.messages.first()
                .map(|m| m.role == crate::Role::System)
                .unwrap_or(false);

            if !has_system {
                request.messages.insert(0, crate::Message::system(&self.system_prompt));
            }
        }

        self.backend.complete_stream(request).await
    }

    /// Get the underlying backend.
    pub fn backend(&self) -> &Arc<dyn Backend> {
        &self.backend
    }
}
```

### 3. ThinkTank (src/roles/think_tank.rs)

```rust
//! ThinkTank - multi-model collaboration system.

use crate::{
    Backend, BackendError, CompletionRequest, CompletionResponse, Message, Role,
};
use super::ModelRole;
use std::collections::HashMap;
use std::sync::Arc;

/// A member of the ThinkTank.
#[derive(Debug)]
pub struct TankMember {
    /// Member's role.
    pub role: ModelRole,
    /// Member's backend.
    pub backend: Arc<dyn Backend>,
    /// Role-specific system prompt.
    pub system_prompt: String,
    /// Whether this member is currently active.
    pub active: bool,
}

impl TankMember {
    /// Create a new tank member.
    pub fn new(role: ModelRole, backend: Arc<dyn Backend>) -> Self {
        let system_prompt = role.system_prompt_modifier().to_string();
        Self {
            role,
            backend,
            system_prompt,
            active: true,
        }
    }

    /// Set a custom system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Complete a request with this member's role context.
    pub async fn complete(&self, mut request: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        if !self.system_prompt.is_empty() {
            request.messages.insert(0, Message::system(&self.system_prompt));
        }
        self.backend.complete(request).await
    }
}

/// The ThinkTank manages multiple models for collaborative reasoning.
///
/// It orchestrates critique, synthesis, and alternative perspectives
/// from multiple LLM backends.
#[derive(Debug, Default)]
pub struct ThinkTank {
    /// Members by role.
    members: HashMap<ModelRole, TankMember>,
}

impl ThinkTank {
    /// Create a new empty ThinkTank.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a member to the tank.
    pub fn add_member(&mut self, member: TankMember) {
        self.members.insert(member.role, member);
    }

    /// Remove a member from the tank.
    pub fn remove_member(&mut self, role: ModelRole) -> Option<TankMember> {
        self.members.remove(&role)
    }

    /// Get a member by role.
    pub fn get_member(&self, role: ModelRole) -> Option<&TankMember> {
        self.members.get(&role)
    }

    /// Get all active members.
    pub fn active_members(&self) -> impl Iterator<Item = &TankMember> {
        self.members.values().filter(|m| m.active)
    }

    /// Check if a role is available.
    pub fn has_role(&self, role: ModelRole) -> bool {
        self.members.get(&role).map(|m| m.active).unwrap_or(false)
    }

    /// Request critique from the Critic role.
    pub async fn critique(&self, content: &str) -> Result<Option<String>, BackendError> {
        let Some(critic) = self.members.get(&ModelRole::Critic) else {
            return Ok(None);
        };

        if !critic.active {
            return Ok(None);
        }

        let request = CompletionRequest::new(vec![
            Message::user(format!("Please critique the following:\n\n{}", content)),
        ]);

        let response = critic.complete(request).await?;
        Ok(response.content)
    }

    /// Request synthesis of multiple perspectives.
    pub async fn synthesize(&self, perspectives: &[String]) -> Result<Option<String>, BackendError> {
        let Some(synthesizer) = self.members.get(&ModelRole::Synthesizer) else {
            return Ok(None);
        };

        if !synthesizer.active {
            return Ok(None);
        }

        let formatted = perspectives
            .iter()
            .enumerate()
            .map(|(i, p)| format!("Perspective {}:\n{}", i + 1, p))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let request = CompletionRequest::new(vec![
            Message::user(format!(
                "Please synthesize these perspectives into a coherent response:\n\n{}",
                formatted
            )),
        ]);

        let response = synthesizer.complete(request).await?;
        Ok(response.content)
    }

    /// Request alternative viewpoint from Contrarian.
    pub async fn counter(&self, proposal: &str) -> Result<Option<String>, BackendError> {
        let Some(contrarian) = self.members.get(&ModelRole::Contrarian) else {
            return Ok(None);
        };

        if !contrarian.active {
            return Ok(None);
        }

        let request = CompletionRequest::new(vec![
            Message::user(format!(
                "Please provide an alternative perspective or challenge this proposal:\n\n{}",
                proposal
            )),
        ]);

        let response = contrarian.complete(request).await?;
        Ok(response.content)
    }

    /// Run a full Forge session: collect perspectives, critique, and synthesize.
    pub async fn forge(
        &self,
        topic: &str,
        initial_response: &str,
    ) -> Result<ForgeResult, BackendError> {
        let mut perspectives = vec![initial_response.to_string()];

        // Collect critique
        if let Some(critique) = self.critique(initial_response).await? {
            perspectives.push(critique);
        }

        // Collect counter perspective
        if let Some(counter) = self.counter(initial_response).await? {
            perspectives.push(counter);
        }

        // Synthesize all perspectives
        let synthesis = self.synthesize(&perspectives).await?;

        Ok(ForgeResult {
            topic: topic.to_string(),
            perspectives,
            synthesis,
        })
    }
}

/// Result of a Forge session.
#[derive(Debug, Clone)]
pub struct ForgeResult {
    /// Original topic.
    pub topic: String,
    /// Collected perspectives.
    pub perspectives: Vec<String>,
    /// Final synthesis (if synthesizer available).
    pub synthesis: Option<String>,
}

impl ForgeResult {
    /// Get the final response (synthesis if available, else first perspective).
    pub fn final_response(&self) -> &str {
        self.synthesis
            .as_deref()
            .or(self.perspectives.first().map(|s| s.as_str()))
            .unwrap_or("")
    }
}
```

### 4. Model Selection (src/roles/selection.rs)

```rust
//! Model selection based on role requirements.

use crate::{Backend, BackendCapabilities, BackendInfo};
use super::{ModelRole, RoleCapabilities};
use std::sync::Arc;

/// Score a backend for a given role.
pub fn score_backend_for_role(
    backend: &dyn Backend,
    role: ModelRole,
) -> u32 {
    let caps = backend.capabilities();
    let reqs = role.recommended_capabilities();
    let mut score = 0u32;

    // Context window check
    if caps.max_context_tokens >= reqs.min_context_tokens {
        score += 100;
    } else {
        // Partial credit for close matches
        let ratio = caps.max_context_tokens as f32 / reqs.min_context_tokens as f32;
        score += (ratio * 50.0) as u32;
    }

    // Tool calling
    if reqs.requires_tool_calling {
        if caps.tool_calling {
            score += 50;
        } else {
            score = score.saturating_sub(100); // Penalty for missing required feature
        }
    }

    // Streaming
    if reqs.requires_streaming {
        if caps.streaming {
            score += 25;
        } else {
            score = score.saturating_sub(50);
        }
    }

    // High reasoning preference (larger models score higher)
    if reqs.prefers_high_reasoning {
        score += (caps.max_context_tokens / 10_000).min(50);
    }

    // Low latency preference (smaller models score higher)
    if reqs.prefers_low_latency {
        score += (100_000u32.saturating_sub(caps.max_context_tokens) / 2_000).min(50);
    }

    score
}

/// Select the best backend for a role from available options.
pub fn select_backend_for_role(
    backends: &[Arc<dyn Backend>],
    role: ModelRole,
) -> Option<Arc<dyn Backend>> {
    backends
        .iter()
        .map(|b| (b, score_backend_for_role(b.as_ref(), role)))
        .max_by_key(|(_, score)| *score)
        .filter(|(_, score)| *score >= 50) // Minimum viable score
        .map(|(b, _)| Arc::clone(b))
}

/// Check if a backend meets minimum requirements for a role.
pub fn backend_meets_requirements(
    backend: &dyn Backend,
    role: ModelRole,
) -> bool {
    let caps = backend.capabilities();
    let reqs = role.recommended_capabilities();

    // Check hard requirements
    if reqs.requires_tool_calling && !caps.tool_calling {
        return false;
    }

    if reqs.requires_streaming && !caps.streaming {
        return false;
    }

    // Flexible on context size (allow 50% of requirement)
    if caps.max_context_tokens < reqs.min_context_tokens / 2 {
        return false;
    }

    true
}
```

### 5. Module Exports (src/roles/mod.rs - continued)

```rust
// Add to the existing mod.rs

mod brain;
mod think_tank;
mod selection;

pub use brain::Brain;
pub use think_tank::{ForgeResult, TankMember, ThinkTank};
pub use selection::{backend_meets_requirements, score_backend_for_role, select_backend_for_role};
```

---

## Testing Requirements

1. Brain correctly tracks context usage
2. ThinkTank critique/synthesis/counter methods work
3. Role capability requirements are sensible
4. Backend scoring produces correct rankings
5. System prompts are injected correctly

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Depends on: [052-backend-config.md](052-backend-config.md)
- Next: [054-tool-definitions.md](054-tool-definitions.md)
- Used by: Mission Control, Forge sessions
