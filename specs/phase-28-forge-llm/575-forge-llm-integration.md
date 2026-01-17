# Spec 575: Forge LLM Integration

**Priority:** P0  
**Status:** planned  
**Estimated Effort:** 2-3 days  
**Target Files:**
- `crates/tachikoma-forge/src/llm/mod.rs` (new)
- `crates/tachikoma-forge/src/llm/provider.rs` (new)
- `crates/tachikoma-forge/src/llm/openai.rs` (new)
- `crates/tachikoma-forge/src/llm/anthropic.rs` (new)
- `crates/tachikoma-forge/src/llm/ollama.rs` (new)
- `crates/tachikoma-forge/src/orchestrator.rs` (new)
- `crates/tachikoma-forge/src/lib.rs` (update)
- `crates/tachikoma-server/src/routes/v1.rs` (update handlers)

---

## Overview

Implement real LLM integration for the Forge Think Tank deliberation system. Multiple AI models (Claude, GPT-4, Ollama) participate in structured brainstorming rounds, streaming their responses in real-time.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        ForgeOrchestrator                         │
│  - Manages deliberation rounds                                   │
│  - Coordinates multiple LLM participants                         │
│  - Tracks convergence                                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      LlmProvider Trait                           │
│  async fn complete(&self, request) -> Stream<Response>           │
│  async fn complete_json(&self, request) -> StructuredResponse    │
└─────────────────────────────────────────────────────────────────┘
        │                     │                     │
        ▼                     ▼                     ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   OpenAI    │      │  Anthropic  │      │   Ollama    │
│  Provider   │      │   Provider  │      │  Provider   │
└─────────────┘      └─────────────┘      └─────────────┘
```

---

## Acceptance Criteria

### 1. LLM Provider Trait
- [ ] Create `LlmProvider` trait with async streaming support
- [ ] Define `LlmRequest` struct with: model, messages, temperature, max_tokens, system_prompt
- [ ] Define `LlmResponse` with: content, role, finish_reason, usage (tokens)
- [ ] Define `LlmStreamChunk` for streaming: delta_content, is_complete

### 2. Anthropic Provider (Primary)
- [ ] Implement `AnthropicProvider` for Claude models
- [ ] Support models: claude-sonnet-4-20250514, claude-3-5-sonnet-20241022, claude-3-opus
- [ ] Read API key from `ANTHROPIC_API_KEY` env var
- [ ] Handle Anthropic's SSE format (content_block_delta events)
- [ ] Stream responses via `async_stream` or `tokio::sync::mpsc`
- [ ] Track input/output tokens from usage response

### 3. OpenAI Provider (Optional)
- [ ] Implement `OpenAiProvider` using `reqwest` + SSE parsing
- [ ] Support models: gpt-4-turbo, gpt-4o, gpt-3.5-turbo
- [ ] Read API key from `OPENAI_API_KEY` env var
- [ ] Parse SSE `data: [DONE]` termination
- [ ] Track token usage from response headers/body

### 4. Ollama Provider
- [ ] Implement `OllamaProvider` for local models
- [ ] Connect to `http://localhost:11434/api/generate`
- [ ] Support streaming via Ollama's newline-delimited JSON
- [ ] No API key required (local only)

### 5. Forge Orchestrator
- [ ] Create `ForgeOrchestrator` struct that owns:
  - `session: ForgeSession`
  - `participants: Vec<(Participant, Box<dyn LlmProvider>)>`
  - `event_tx: broadcast::Sender<ForgeEvent>`
- [ ] Implement `run_round(&mut self, round_type: RoundType)` method
- [ ] Round types: Draft, Critique, Synthesis, Convergence
- [ ] For each participant, call LLM with appropriate prompt
- [ ] Stream responses through event channel
- [ ] Collect all responses, update session state

### 6. Prompt Templates
- [ ] Create prompts module with templates for each round type
- [ ] **Draft prompt**: "Given the goal: {goal}, propose a solution..."
- [ ] **Critique prompt**: "Review the following proposals: {drafts}. Identify strengths, weaknesses, gaps..."
- [ ] **Synthesis prompt**: "Given these proposals and critiques: {context}. Create a unified solution..."
- [ ] **Convergence prompt**: "Does this solution adequately address the goal? Vote: agree/disagree with reasoning"

### 7. Server Handler Integration
- [ ] Update `forge_create_session` to initialize orchestrator
- [ ] Update `forge_add_draft` to trigger draft round
- [ ] Update `forge_synthesize` to run synthesis
- [ ] Update `forge_converge` to run convergence check
- [ ] Stream events via WebSocket to connected clients

### 8. Configuration
- [ ] Add `ForgeConfig` to server config:
  ```toml
  [forge]
  default_model = "claude-sonnet-4-20250514"
  max_rounds = 10
  convergence_threshold = 0.8
  round_timeout_seconds = 300
  
  [forge.providers.anthropic]
  enabled = true
  
  [forge.providers.openai]
  enabled = false  # Optional, enable if API key available
  
  [forge.providers.ollama]
  enabled = true
  base_url = "http://localhost:11434"
  ```

---

## Implementation Details

### LlmProvider Trait

```rust
// crates/tachikoma-forge/src/llm/provider.rs

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

#[derive(Debug, Clone)]
pub struct LlmStreamChunk {
    pub delta: String,
    pub is_complete: bool,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub finish_reason: String,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

pub type LlmStream = Pin<Box<dyn Stream<Item = Result<LlmStreamChunk, LlmError>> + Send>>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError>;
}
```

### Anthropic Implementation (Primary)

```rust
// crates/tachikoma-forge/src/llm/anthropic.rs

use super::provider::*;
use async_stream::stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use futures::StreamExt;

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

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }
    fn model(&self) -> &str { &self.model }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        let messages: Vec<_> = request.messages
            .into_iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|m| AnthropicMessage {
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => unreachable!(),
                },
                content: m.content,
            })
            .collect();
        
        let system = request.messages
            .iter()
            .find(|m| matches!(m.role, MessageRole::System))
            .map(|m| m.content.clone());
        
        let api_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(4096),
            messages,
            system,
            stream: true,
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&api_request)
            .send()
            .await?;
        
        let stream = stream! {
            let mut reader = response.bytes_stream();
            let mut buffer = String::new();
            
            while let Some(chunk) = reader.next().await {
                let chunk = chunk?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                
                // Process complete SSE events
                while let Some(idx) = buffer.find("\n\n") {
                    let event_data = buffer[..idx].to_string();
                    buffer = buffer[idx + 2..].to_string();
                    
                    for line in event_data.lines() {
                        if line.starts_with("data: ") {
                            let json_str = &line[6..];
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
                                            yield Ok(LlmStreamChunk {
                                                delta: String::new(),
                                                is_complete: delta.stop_reason.is_some(),
                                                finish_reason: delta.stop_reason,
                                            });
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
        // Non-streaming implementation
        // ... similar to above but without stream: true
        todo!("implement non-streaming complete")
    }
}
```

### OpenAI Implementation (Optional)

```rust
// crates/tachikoma-forge/src/llm/openai.rs

use super::provider::*;
use async_stream::stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(model: &str) -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| LlmError::MissingApiKey("OPENAI_API_KEY"))?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            model: model.to_string(),
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str { "openai" }
    fn model(&self) -> &str { &self.model }
    
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmStream, LlmError> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&OpenAiRequest::from(request))
            .send()
            .await?;
        
        let stream = stream! {
            let mut reader = response.bytes_stream();
            while let Some(chunk) = reader.next().await {
                let chunk = chunk?;
                let text = String::from_utf8_lossy(&chunk);
                
                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" {
                            yield Ok(LlmStreamChunk {
                                delta: String::new(),
                                is_complete: true,
                                finish_reason: Some("stop".into()),
                            });
                            return;
                        }
                        
                        if let Ok(parsed) = serde_json::from_str::<OpenAiStreamResponse>(data) {
                            if let Some(choice) = parsed.choices.first() {
                                yield Ok(LlmStreamChunk {
                                    delta: choice.delta.content.clone().unwrap_or_default(),
                                    is_complete: false,
                                    finish_reason: choice.finish_reason.clone(),
                                });
                            }
                        }
                    }
                }
            }
        };
        
        Ok(Box::pin(stream))
    }
    
    // ... complete() implementation
}
```

### ForgeOrchestrator

```rust
// crates/tachikoma-forge/src/orchestrator.rs

use crate::{ForgeSession, ForgeRound, Participant, TokenUsage};
use crate::llm::{LlmProvider, LlmRequest, LlmMessage, MessageRole};
use tokio::sync::broadcast;
use uuid::Uuid;

pub struct ForgeOrchestrator {
    session: ForgeSession,
    participants: Vec<ParticipantWithProvider>,
    event_tx: broadcast::Sender<ForgeEvent>,
}

struct ParticipantWithProvider {
    participant: Participant,
    provider: Box<dyn LlmProvider>,
}

#[derive(Debug, Clone)]
pub enum ForgeEvent {
    RoundStarted { round: u32, round_type: RoundType },
    ParticipantThinking { participant_id: String },
    ContentDelta { participant_id: String, delta: String },
    ParticipantComplete { participant_id: String, content: String, tokens: TokenUsage },
    RoundComplete { round: u32, summary: String },
    SessionConverged { score: f64 },
    Error { message: String },
}

#[derive(Debug, Clone, Copy)]
pub enum RoundType {
    Draft,
    Critique,
    Synthesis,
    Convergence,
}

impl ForgeOrchestrator {
    pub fn new(
        session: ForgeSession,
        event_tx: broadcast::Sender<ForgeEvent>,
    ) -> Self {
        Self {
            session,
            participants: vec![],
            event_tx,
        }
    }
    
    pub fn add_participant(&mut self, participant: Participant, provider: Box<dyn LlmProvider>) {
        self.participants.push(ParticipantWithProvider { participant, provider });
    }
    
    pub async fn run_round(&mut self, round_type: RoundType) -> Result<ForgeRound, ForgeError> {
        let round_num = self.session.rounds.len() + 1;
        
        // Notify round started
        let _ = self.event_tx.send(ForgeEvent::RoundStarted {
            round: round_num as u32,
            round_type,
        });
        
        let prompt = self.build_prompt(round_type);
        let mut contributions = vec![];
        
        // Run each participant
        for pwp in &self.participants {
            // Skip human participants in AI rounds
            if matches!(pwp.participant.type_, ParticipantType::Human) {
                continue;
            }
            
            let _ = self.event_tx.send(ForgeEvent::ParticipantThinking {
                participant_id: pwp.participant.id.clone(),
            });
            
            let request = LlmRequest {
                model: pwp.provider.model().to_string(),
                messages: vec![
                    LlmMessage {
                        role: MessageRole::System,
                        content: self.system_prompt(round_type),
                    },
                    LlmMessage {
                        role: MessageRole::User,
                        content: prompt.clone(),
                    },
                ],
                temperature: 0.7,
                max_tokens: 2000,
                stream: true,
            };
            
            // Stream response
            let mut stream = pwp.provider.complete_stream(request).await?;
            let mut full_content = String::new();
            
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                full_content.push_str(&chunk.delta);
                
                let _ = self.event_tx.send(ForgeEvent::ContentDelta {
                    participant_id: pwp.participant.id.clone(),
                    delta: chunk.delta,
                });
                
                if chunk.is_complete {
                    break;
                }
            }
            
            contributions.push(Contribution {
                participant: pwp.participant.clone(),
                content: full_content.clone(),
                tokens: TokenUsage::default(), // Updated from stream
            });
            
            let _ = self.event_tx.send(ForgeEvent::ParticipantComplete {
                participant_id: pwp.participant.id.clone(),
                content: full_content,
                tokens: TokenUsage::default(),
            });
        }
        
        // Build round result based on type
        let round = match round_type {
            RoundType::Draft => ForgeRound::Draft(DraftRound {
                drafter: contributions[0].participant.clone(),
                content: contributions[0].content.clone(),
                tokens: contributions[0].tokens.clone(),
                duration_ms: 0, // Track timing
            }),
            RoundType::Critique => ForgeRound::Critique(CritiqueRound {
                critiques: contributions.into_iter().map(|c| {
                    Critique {
                        critic: c.participant,
                        score: 75, // Parse from response
                        strengths: vec![],
                        weaknesses: vec![],
                        suggestions: vec![],
                        raw_content: c.content,
                        tokens: c.tokens,
                        duration_ms: 0,
                    }
                }).collect(),
            }),
            // ... other round types
        };
        
        self.session.add_round(round.clone());
        
        let _ = self.event_tx.send(ForgeEvent::RoundComplete {
            round: round_num as u32,
            summary: format!("{} contributions received", contributions.len()),
        });
        
        Ok(round)
    }
    
    fn system_prompt(&self, round_type: RoundType) -> String {
        match round_type {
            RoundType::Draft => include_str!("prompts/draft_system.txt").to_string(),
            RoundType::Critique => include_str!("prompts/critique_system.txt").to_string(),
            RoundType::Synthesis => include_str!("prompts/synthesis_system.txt").to_string(),
            RoundType::Convergence => include_str!("prompts/convergence_system.txt").to_string(),
        }
    }
    
    fn build_prompt(&self, round_type: RoundType) -> String {
        let goal = &self.session.topic.description;
        
        match round_type {
            RoundType::Draft => {
                format!(
                    "# Goal\n{}\n\n# Task\nPropose a solution or approach to achieve this goal. \
                    Be specific and actionable. Include:\n\
                    1. Key components or steps\n\
                    2. Implementation considerations\n\
                    3. Potential challenges and mitigations",
                    goal
                )
            }
            RoundType::Critique => {
                let drafts = self.collect_drafts();
                format!(
                    "# Goal\n{}\n\n# Proposals to Review\n{}\n\n# Task\n\
                    Critically evaluate these proposals. For each:\n\
                    1. Identify strengths\n\
                    2. Identify weaknesses or gaps\n\
                    3. Suggest improvements\n\
                    4. Rate overall quality (1-100)",
                    goal, drafts
                )
            }
            RoundType::Synthesis => {
                let context = self.collect_all_contributions();
                format!(
                    "# Goal\n{}\n\n# Discussion So Far\n{}\n\n# Task\n\
                    Synthesize the best ideas into a unified solution. \
                    Resolve any conflicts and create a coherent proposal.",
                    goal, context
                )
            }
            RoundType::Convergence => {
                let synthesis = self.get_latest_synthesis();
                format!(
                    "# Goal\n{}\n\n# Proposed Solution\n{}\n\n# Task\n\
                    Evaluate whether this solution adequately addresses the goal.\n\
                    Respond with:\n\
                    - AGREE or DISAGREE\n\
                    - Brief reasoning\n\
                    - Any remaining concerns",
                    goal, synthesis
                )
            }
        }
    }
}
```

### Server Integration

```rust
// Update crates/tachikoma-server/src/routes/v1.rs

use tachikoma_forge::{ForgeOrchestrator, ForgeSession, LlmProviderRegistry};
use tokio::sync::broadcast;

async fn forge_create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<ForgeSession>, ApiError> {
    // Create session
    let session = ForgeSession::new(
        request.config,
        ForgeTopic {
            title: request.name,
            description: request.goal,
            constraints: vec![],
        },
    );
    
    // Store session
    state.forge_sessions.insert(session.id.clone(), session.clone());
    
    Ok(Json(session))
}

async fn forge_start_deliberation(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_deliberation(socket, state, session_id))
}

async fn handle_deliberation(
    mut socket: WebSocket,
    state: AppState,
    session_id: Uuid,
) {
    let session = state.forge_sessions.get(&session_id).unwrap().clone();
    let (event_tx, mut event_rx) = broadcast::channel(100);
    
    let mut orchestrator = ForgeOrchestrator::new(session, event_tx);
    
    // Add AI participants based on session config
    for participant in &session.participants {
        if participant.type_ == ParticipantType::Ai {
            let provider = state.llm_registry.get_provider(&participant.model_name)?;
            orchestrator.add_participant(participant.clone(), provider);
        }
    }
    
    // Spawn event forwarder
    let socket_tx = socket.clone();
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            if socket_tx.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    // Run deliberation phases
    orchestrator.run_round(RoundType::Draft).await?;
    orchestrator.run_round(RoundType::Critique).await?;
    orchestrator.run_round(RoundType::Synthesis).await?;
    
    // Check convergence
    loop {
        let round = orchestrator.run_round(RoundType::Convergence).await?;
        if let ForgeRound::Convergence(c) = round {
            if c.converged || orchestrator.session.rounds.len() >= 10 {
                break;
            }
        }
        // Refine and try again
        orchestrator.run_round(RoundType::Synthesis).await?;
    }
    
    socket.close().await.ok();
}
```

---

## Prompt Templates

Create these files:

### `crates/tachikoma-forge/src/prompts/draft_system.txt`
```
You are a thoughtful participant in a collaborative brainstorming session. 
Your role is to propose creative, practical solutions.

Guidelines:
- Be specific and actionable
- Consider implementation details
- Acknowledge trade-offs
- Build on the goal's constraints
- Use markdown formatting
```

### `crates/tachikoma-forge/src/prompts/critique_system.txt`
```
You are a critical reviewer in a collaborative brainstorming session.
Your role is to identify strengths, weaknesses, and areas for improvement.

Guidelines:
- Be constructive, not dismissive
- Cite specific parts of proposals
- Suggest concrete improvements
- Consider edge cases and risks
- Rate proposals objectively (1-100)
```

### `crates/tachikoma-forge/src/prompts/synthesis_system.txt`
```
You are a synthesizer in a collaborative brainstorming session.
Your role is to merge the best ideas into a coherent solution.

Guidelines:
- Incorporate feedback from critiques
- Resolve conflicting approaches
- Create a unified, implementable plan
- Be concise but complete
- Structure with clear sections
```

### `crates/tachikoma-forge/src/prompts/convergence_system.txt`
```
You are an evaluator in a collaborative brainstorming session.
Your role is to determine if the proposed solution meets the goal.

Guidelines:
- Compare solution against original goal
- Identify any gaps or concerns
- Vote AGREE or DISAGREE
- Provide brief reasoning
- Be objective and fair
```

---

## Dependencies to Add

```toml
# Cargo.toml for tachikoma-forge
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
async-trait = "0.1"
async-stream = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
futures = "0.3"
tracing = "0.1"
thiserror = "1"
```

---

## Testing

### Unit Tests
- [ ] Test each provider with mocked HTTP responses
- [ ] Test prompt generation for all round types
- [ ] Test token counting/aggregation

### Integration Tests  
- [ ] Test full deliberation flow with live API (use cheap model)
- [ ] Test WebSocket streaming to client
- [ ] Test error handling (rate limits, API errors)

### Manual Testing
```bash
# Start server
OPENAI_API_KEY=sk-... cargo run --bin tachikoma-server

# Create session via API
curl -X POST http://localhost:3000/api/v1/forge/sessions \
  -H "Content-Type: application/json" \
  -d '{"name": "Test", "goal": "Design a TODO app API"}'

# Connect WebSocket and watch deliberation
websocat ws://localhost:3000/api/v1/forge/sessions/{id}/deliberate
```

---

## Success Metrics

1. ✅ Create session with multiple AI participants
2. ✅ Watch real-time streaming of AI responses  
3. ✅ See critique round with scores
4. ✅ Get synthesized final output
5. ✅ Export as markdown/JSON/YAML

---

## Notes

- **Start with Anthropic (Claude)** - uses `ANTHROPIC_API_KEY` env var
- Use `claude-sonnet-4-20250514` for best quality/speed balance
- Use `claude-3-5-sonnet-20241022` as fallback
- OpenAI is optional - add later if multi-vendor support needed
- Ollama is great for local dev without API costs
