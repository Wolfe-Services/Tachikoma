use crate::{ForgeSession, Participant};
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
    ParticipantComplete { participant_id: String, content: String, tokens: crate::session::TokenUsage },
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
                participant_id: participant_id.to_string(),
                participant_name: participant_name.clone(),
            });
            
            let prompt = crate::prompts::build_prompt(
                round_type,
                &self.session.topic.description,
                &pwp.participant,
                &self.session,
            );
            
            let request = LlmRequest {
                model: pwp.provider.model().to_string(),
                messages: vec![
                    LlmMessage {
                        role: MessageRole::User,
                        content: prompt,
                    },
                ],
                temperature: Some(0.7),
                max_tokens: Some(2048),
                system_prompt: None,
            };
            
            let mut stream = pwp.provider.complete_stream(request).await?;
            let mut full_content = String::new();
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        full_content.push_str(&chunk.delta);
                        let _ = self.event_tx.send(ForgeEvent::ContentDelta {
                            participant_id: participant_id.to_string(),
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
                participant_id: participant_id.to_string(),
                content: full_content,
                tokens: crate::session::TokenUsage::default(),
            });
        }
        
        let _ = self.event_tx.send(ForgeEvent::RoundComplete { round });
        
        Ok(())
    }
}