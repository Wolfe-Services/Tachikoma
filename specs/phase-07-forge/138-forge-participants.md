# 138 - Forge Participant Management

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 138
**Status:** Planned
**Dependencies:** 136-forge-session-types, 137-forge-config
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement participant management for Forge sessions, including model selection, role assignment, load balancing, and API client abstraction for multiple LLM providers.

---

## Acceptance Criteria

- [ ] `ParticipantManager` for model coordination
- [ ] Provider-agnostic API client trait
- [ ] Anthropic, OpenAI, and Google client implementations
- [ ] Role-based model selection
- [ ] Rate limiting and quota management
- [ ] Health checking for providers
- [ ] Graceful fallback on provider failure

---

## Implementation Details

### 1. Provider Trait (src/provider.rs)

```rust
//! LLM provider abstraction.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    ForgeError, ForgeResult, ModelResponse, Participant, TokenCount,
};

/// Request to send to a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    /// System prompt.
    pub system: String,
    /// User messages.
    pub messages: Vec<Message>,
    /// Maximum tokens to generate.
    pub max_tokens: usize,
    /// Temperature.
    pub temperature: f32,
    /// Stop sequences.
    pub stop_sequences: Vec<String>,
}

impl ModelRequest {
    /// Create a new request.
    pub fn new(system: impl Into<String>) -> Self {
        Self {
            system: system.into(),
            messages: Vec::new(),
            max_tokens: 4096,
            temperature: 0.7,
            stop_sequences: Vec::new(),
        }
    }

    /// Add a user message.
    pub fn with_user_message(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message {
            role: MessageRole::User,
            content: content.into(),
        });
        self
    }

    /// Add an assistant message.
    pub fn with_assistant_message(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: content.into(),
        });
        self
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    /// Set temperature.
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }
}

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message author.
    pub role: MessageRole,
    /// Message content.
    pub content: String,
}

/// Message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Provider trait for LLM APIs.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &str;

    /// Check if the provider is healthy.
    async fn health_check(&self) -> ForgeResult<bool>;

    /// Send a request and get a response.
    async fn complete(&self, participant: &Participant, request: ModelRequest) -> ForgeResult<ModelResponse>;

    /// Count tokens for a request (estimate).
    fn count_tokens(&self, request: &ModelRequest) -> ForgeResult<u64>;

    /// Get rate limit status.
    fn rate_limit_status(&self) -> RateLimitStatus;
}

/// Rate limit status.
#[derive(Debug, Clone, Default)]
pub struct RateLimitStatus {
    /// Requests remaining.
    pub requests_remaining: Option<u64>,
    /// Tokens remaining.
    pub tokens_remaining: Option<u64>,
    /// Reset time.
    pub reset_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Currently rate limited.
    pub is_limited: bool,
}
```

### 2. Anthropic Provider (src/providers/anthropic.rs)

```rust
//! Anthropic (Claude) provider implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::{
    ForgeError, ForgeResult, Message, MessageRole, ModelRequest, ModelResponse,
    Participant, Provider, RateLimitStatus, StopReason, TokenCount,
};

/// Anthropic API provider.
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    rate_limit: Arc<RwLock<RateLimitStatus>>,
    request_count: AtomicU64,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
            rate_limit: Arc::new(RwLock::new(RateLimitStatus::default())),
            request_count: AtomicU64::new(0),
        }
    }

    /// Create with custom base URL (for testing).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    system: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop_sequences: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    content: Vec<ContentBlock>,
    stop_reason: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "Anthropic"
    }

    async fn health_check(&self) -> ForgeResult<bool> {
        // Simple health check - try to reach the API
        let response = self.client
            .get(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await;

        // Even a 401 means the API is reachable
        Ok(response.is_ok())
    }

    async fn complete(&self, participant: &Participant, request: ModelRequest) -> ForgeResult<ModelResponse> {
        let start = Instant::now();
        self.request_count.fetch_add(1, Ordering::SeqCst);

        let messages: Vec<AnthropicMessage> = request.messages.iter().map(|m| {
            AnthropicMessage {
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            }
        }).collect();

        let api_request = AnthropicRequest {
            model: participant.model_id.clone(),
            max_tokens: request.max_tokens,
            system: request.system,
            messages,
            temperature: Some(request.temperature),
            stop_sequences: request.stop_sequences,
        };

        let response = self.client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&api_request)
            .send()
            .await
            .map_err(|e| ForgeError::Provider(format!("Anthropic request failed: {}", e)))?;

        // Update rate limit status from headers
        if let Some(remaining) = response.headers().get("x-ratelimit-remaining-requests") {
            if let Ok(val) = remaining.to_str().unwrap_or("0").parse() {
                let mut status = self.rate_limit.write().await;
                status.requests_remaining = Some(val);
            }
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ForgeError::Provider(
                format!("Anthropic API error {}: {}", status, body)
            ));
        }

        let api_response: AnthropicResponse = response.json().await
            .map_err(|e| ForgeError::Provider(format!("Failed to parse Anthropic response: {}", e)))?;

        let content = api_response.content
            .iter()
            .filter_map(|b| b.text.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("");

        let stop_reason = match api_response.stop_reason.as_str() {
            "end_turn" => StopReason::EndTurn,
            "max_tokens" => StopReason::MaxTokens,
            "stop_sequence" => StopReason::StopSequence,
            _ => StopReason::EndTurn,
        };

        Ok(ModelResponse {
            participant: participant.clone(),
            content,
            tokens: TokenCount {
                input: api_response.usage.input_tokens,
                output: api_response.usage.output_tokens,
            },
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: tachikoma_common_core::Timestamp::now(),
            stop_reason,
            raw_response: None,
        })
    }

    fn count_tokens(&self, request: &ModelRequest) -> ForgeResult<u64> {
        // Rough estimation: ~4 chars per token
        let total_chars: usize = request.system.len()
            + request.messages.iter().map(|m| m.content.len()).sum::<usize>();
        Ok((total_chars / 4) as u64)
    }

    fn rate_limit_status(&self) -> RateLimitStatus {
        // Return cached status
        RateLimitStatus::default()
    }
}
```

### 3. Participant Manager (src/participant_manager.rs)

```rust
//! Participant management for Forge sessions.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    ForgeConfig, ForgeError, ForgeResult, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, Participant, ParticipantRole, Provider,
};

/// Manages participants and their provider connections.
pub struct ParticipantManager {
    /// Configuration.
    config: ForgeConfig,
    /// Provider instances.
    providers: HashMap<ModelProvider, Arc<dyn Provider>>,
    /// Active participants.
    participants: RwLock<Vec<Participant>>,
    /// Provider health status.
    health_status: RwLock<HashMap<ModelProvider, bool>>,
}

impl ParticipantManager {
    /// Create a new participant manager.
    pub fn new(config: ForgeConfig) -> Self {
        Self {
            config,
            providers: HashMap::new(),
            participants: RwLock::new(Vec::new()),
            health_status: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider.
    pub fn register_provider(&mut self, provider_type: ModelProvider, provider: Arc<dyn Provider>) {
        self.providers.insert(provider_type, provider);
    }

    /// Initialize participants for a session.
    pub async fn initialize_participants(&self, roles: &[ParticipantRole]) -> ForgeResult<Vec<Participant>> {
        let mut participants = Vec::new();

        for role in roles {
            let participant = self.select_participant_for_role(*role).await?;
            participants.push(participant);
        }

        *self.participants.write().await = participants.clone();
        Ok(participants)
    }

    /// Select the best participant for a given role.
    async fn select_participant_for_role(&self, role: ParticipantRole) -> ForgeResult<Participant> {
        // Find models that prefer this role
        let mut candidates: Vec<(&String, &ModelConfig)> = self.config.models.available
            .iter()
            .filter(|(_, m)| m.enabled && m.preferred_roles.contains(&role))
            .collect();

        // If no preferred models, use any enabled model
        if candidates.is_empty() {
            candidates = self.config.models.available
                .iter()
                .filter(|(_, m)| m.enabled)
                .collect();
        }

        if candidates.is_empty() {
            return Err(ForgeError::Config("No enabled models available".to_string()));
        }

        // Check health and select first healthy candidate
        for (name, model_config) in &candidates {
            let provider_healthy = self.check_provider_health(model_config.provider).await;
            if provider_healthy {
                return Ok(Participant {
                    model_id: model_config.model_id.clone(),
                    display_name: model_config.display_name.clone(),
                    provider: model_config.provider,
                    role,
                });
            }
        }

        // Fallback to first candidate even if health unknown
        let (_, model_config) = candidates[0];
        Ok(Participant {
            model_id: model_config.model_id.clone(),
            display_name: model_config.display_name.clone(),
            provider: model_config.provider,
            role,
        })
    }

    /// Check if a provider is healthy.
    async fn check_provider_health(&self, provider_type: ModelProvider) -> bool {
        // Check cache first
        if let Some(&status) = self.health_status.read().await.get(&provider_type) {
            return status;
        }

        // Perform health check
        if let Some(provider) = self.providers.get(&provider_type) {
            let healthy = provider.health_check().await.unwrap_or(false);
            self.health_status.write().await.insert(provider_type, healthy);
            healthy
        } else {
            false
        }
    }

    /// Send a request to a participant.
    pub async fn send_request(
        &self,
        participant: &Participant,
        request: ModelRequest,
    ) -> ForgeResult<ModelResponse> {
        let provider = self.providers.get(&participant.provider)
            .ok_or_else(|| ForgeError::Provider(
                format!("No provider registered for {:?}", participant.provider)
            ))?;

        // Check rate limits
        let rate_status = provider.rate_limit_status();
        if rate_status.is_limited {
            return Err(ForgeError::RateLimit(format!(
                "{} is rate limited",
                provider.name()
            )));
        }

        provider.complete(participant, request).await
    }

    /// Send requests to multiple participants in parallel.
    pub async fn send_parallel_requests(
        &self,
        requests: Vec<(Participant, ModelRequest)>,
    ) -> Vec<ForgeResult<ModelResponse>> {
        use futures::future::join_all;

        let futures = requests.into_iter().map(|(participant, request)| {
            let manager = self;
            async move {
                manager.send_request(&participant, request).await
            }
        });

        join_all(futures).await
    }

    /// Get the default drafter.
    pub async fn get_drafter(&self) -> ForgeResult<Participant> {
        self.select_participant_for_role(ParticipantRole::Drafter).await
    }

    /// Get critics.
    pub async fn get_critics(&self, count: usize) -> ForgeResult<Vec<Participant>> {
        let mut critics = Vec::new();
        let mut used_models = std::collections::HashSet::new();

        for _ in 0..count {
            let mut critic = self.select_participant_for_role(ParticipantRole::Critic).await?;

            // Try to get diverse critics
            let mut attempts = 0;
            while used_models.contains(&critic.model_id) && attempts < 5 {
                // Try different roles that can critique
                let alt_roles = [
                    ParticipantRole::CodeReviewer,
                    ParticipantRole::DevilsAdvocate,
                    ParticipantRole::Generalist,
                ];
                for role in alt_roles {
                    if let Ok(alt) = self.select_participant_for_role(role).await {
                        if !used_models.contains(&alt.model_id) {
                            critic = alt;
                            break;
                        }
                    }
                }
                attempts += 1;
            }

            used_models.insert(critic.model_id.clone());
            critics.push(critic);
        }

        Ok(critics)
    }

    /// Get the synthesizer.
    pub async fn get_synthesizer(&self) -> ForgeResult<Participant> {
        self.select_participant_for_role(ParticipantRole::Synthesizer).await
    }

    /// Get all active participants.
    pub async fn active_participants(&self) -> Vec<Participant> {
        self.participants.read().await.clone()
    }

    /// Calculate cost for a model.
    pub fn calculate_cost(&self, participant: &Participant, tokens: &crate::TokenCount) -> f64 {
        // Find model config
        for (_, config) in &self.config.models.available {
            if config.model_id == participant.model_id {
                return config.calculate_cost(tokens.input, tokens.output);
            }
        }
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_select_participant() {
        let config = ForgeConfig::default();
        let manager = ParticipantManager::new(config);

        let participant = manager.select_participant_for_role(ParticipantRole::Drafter).await;
        assert!(participant.is_ok());
    }
}
```

### 4. Provider Factory (src/providers/mod.rs)

```rust
//! Provider implementations.

mod anthropic;
mod openai;
mod google;

pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;
pub use google::GoogleProvider;

use std::sync::Arc;
use crate::{ForgeConfig, ForgeResult, ModelProvider, ParticipantManager};

/// Create a participant manager with all providers.
pub fn create_participant_manager(config: ForgeConfig) -> ForgeResult<ParticipantManager> {
    let mut manager = ParticipantManager::new(config);

    // Register Anthropic provider
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        let provider = AnthropicProvider::new(api_key);
        manager.register_provider(ModelProvider::Anthropic, Arc::new(provider));
    }

    // Register OpenAI provider
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        let provider = OpenAIProvider::new(api_key);
        manager.register_provider(ModelProvider::OpenAI, Arc::new(provider));
    }

    // Register Google provider
    if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
        let provider = GoogleProvider::new(api_key);
        manager.register_provider(ModelProvider::Google, Arc::new(provider));
    }

    Ok(manager)
}
```

---

## Testing Requirements

1. Provider trait implementation for all supported providers
2. Participant selection prioritizes preferred roles
3. Health checking caches results appropriately
4. Parallel request handling works correctly
5. Rate limiting prevents excessive requests
6. Fallback works when primary provider fails

---

## Related Specs

- Depends on: [136-forge-session-types.md](136-forge-session-types.md)
- Depends on: [137-forge-config.md](137-forge-config.md)
- Next: [139-forge-rounds.md](139-forge-rounds.md)
- Used by: Round orchestration (139-145)
