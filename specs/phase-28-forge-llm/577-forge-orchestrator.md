# Spec 577: Forge Orchestrator

**Priority:** P0  
**Status:** planned  
**Depends on:** 575, 576  
**Estimated Effort:** 4 hours  
**Target Files:**
- `crates/tachikoma-forge/src/orchestrator.rs` (new)
- `crates/tachikoma-forge/src/prompts.rs` (new)
- `crates/tachikoma-forge/src/lib.rs` (update)

---

## Overview

Create the ForgeOrchestrator that coordinates LLM participants through deliberation rounds (Draft, Critique, Synthesis, Convergence).

---

## Acceptance Criteria

- [x] Create `crates/tachikoma-forge/src/orchestrator.rs`
- [x] Define `ForgeOrchestrator` struct with session, participants list, event sender
- [x] Define `ForgeEvent` enum: RoundStarted, ParticipantThinking, ContentDelta, ParticipantComplete, RoundComplete, Error
- [x] Define `RoundType` enum: Draft, Critique, Synthesis, Convergence
- [x] Implement `ForgeOrchestrator::new()` that takes session and participant configs
- [x] Implement `run_round(&mut self, round_type: RoundType)` that calls each participant
- [x] Stream content deltas through the event channel as they arrive
- [x] Create `crates/tachikoma-forge/src/prompts.rs` with round prompt templates
- [x] Export orchestrator and prompts from lib.rs
- [x] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/orchestrator.rs

use crate::{ForgeSession, Participant, TokenUsage};
use crate::llm::{LlmProvider, LlmRequest, LlmMessage, MessageRole, LlmError};
use tokio::sync::broadcast;
use futures::StreamExt;

pub struct ForgeOrchestrator {
    pub session: ForgeSession,
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
    ParticipantThinking { participant_id: String, participant_name: String },
    ContentDelta { participant_id: String, delta: String },
    ParticipantComplete { participant_id: String, content: String, tokens: TokenUsage },
    RoundComplete { round: u32 },
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
            participants: Vec::new(),
            event_tx,
        }
    }
    
    pub fn add_participant(&mut self, participant: Participant, provider: Box<dyn LlmProvider>) {
        self.participants.push(ParticipantWithProvider { participant, provider });
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ForgeEvent> {
        self.event_tx.subscribe()
    }
    
    pub async fn run_round(&mut self, round_type: RoundType) -> Result<(), LlmError> {
        let round = self.session.rounds.len() as u32 + 1;
        
        let _ = self.event_tx.send(ForgeEvent::RoundStarted { round, round_type });
        
        for pwp in &self.participants {
            let participant_id = pwp.participant.id.clone();
            let participant_name = pwp.participant.name.clone();
            
            let _ = self.event_tx.send(ForgeEvent::ParticipantThinking {
                participant_id: participant_id.clone(),
                participant_name: participant_name.clone(),
            });
            
            let prompt = crate::prompts::build_prompt(
                round_type,
                &self.session.goal,
                &pwp.participant,
                &self.session,
            );
            
            let request = LlmRequest {
                model: pwp.provider.model().to_string(),
                messages: vec![
                    LlmMessage {
                        role: MessageRole::System,
                        content: pwp.participant.system_prompt.clone().unwrap_or_default(),
                    },
                    LlmMessage {
                        role: MessageRole::User,
                        content: prompt,
                    },
                ],
                temperature: Some(0.7),
                max_tokens: Some(2048),
            };
            
            let mut stream = pwp.provider.complete_stream(request).await?;
            let mut full_content = String::new();
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        full_content.push_str(&chunk.delta);
                        let _ = self.event_tx.send(ForgeEvent::ContentDelta {
                            participant_id: participant_id.clone(),
                            delta: chunk.delta,
                        });
                    }
                    Err(e) => {
                        let _ = self.event_tx.send(ForgeEvent::Error {
                            message: e.to_string(),
                        });
                        return Err(e);
                    }
                }
            }
            
            let _ = self.event_tx.send(ForgeEvent::ParticipantComplete {
                participant_id: participant_id.clone(),
                content: full_content,
                tokens: TokenUsage::default(),
            });
        }
        
        let _ = self.event_tx.send(ForgeEvent::RoundComplete { round });
        
        Ok(())
    }
}
```

```rust
// crates/tachikoma-forge/src/prompts.rs

use crate::{ForgeSession, Participant};
use super::orchestrator::RoundType;

pub fn build_prompt(
    round_type: RoundType,
    goal: &str,
    participant: &Participant,
    session: &ForgeSession,
) -> String {
    match round_type {
        RoundType::Draft => format!(
            "You are {}.\n\n\
            Goal: {}\n\n\
            Propose a solution or approach to this goal. Be specific and actionable.\n\n\
            Your expertise: {}",
            participant.name,
            goal,
            participant.expertise.as_deref().unwrap_or("general problem solving")
        ),
        
        RoundType::Critique => {
            let drafts = session.rounds
                .iter()
                .filter(|r| r.round_type == "draft")
                .flat_map(|r| &r.contributions)
                .map(|c| format!("**{}**: {}", c.participant_name, c.content))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            
            format!(
                "You are {}.\n\n\
                Review these proposals and provide constructive critique:\n\n{}\n\n\
                Identify strengths, weaknesses, gaps, and potential improvements.",
                participant.name,
                drafts
            )
        }
        
        RoundType::Synthesis => {
            format!(
                "You are {}.\n\n\
                Based on all proposals and critiques so far, synthesize the best elements \
                into a unified solution.\n\n\
                Goal: {}\n\n\
                Create a cohesive approach that addresses the critiques.",
                participant.name,
                goal
            )
        }
        
        RoundType::Convergence => {
            format!(
                "You are {}.\n\n\
                Review the synthesized solution.\n\n\
                Vote: Do you AGREE or DISAGREE that this adequately addresses the goal?\n\n\
                Provide your reasoning. If you disagree, specify what's missing.",
                participant.name
            )
        }
    }
}
```
