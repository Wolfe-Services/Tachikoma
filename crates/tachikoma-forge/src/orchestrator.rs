use crate::{ForgeSession, Participant};
use crate::llm::{LlmProvider, LlmRequest, LlmMessage, MessageRole, LlmError, ProviderFactory};
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
    fallback_providers: Vec<Box<dyn LlmProvider>>,
}

#[derive(Debug, Clone)]
pub enum ForgeEvent {
    RoundStarted { round: u32, round_type: RoundType },
    ParticipantThinking { participant_id: String, participant_name: String },
    ContentDelta { participant_id: String, delta: String },
    ParticipantComplete { participant_id: String, content: String, tokens: crate::session::TokenUsage },
    ParticipantError { participant_id: String, error: String, retrying_with: Option<String> },
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
    
    pub fn add_participant(&mut self, participant: Participant) -> Result<(), LlmError> {
        let primary_provider = ProviderFactory::create(&participant.model_config)?;
        
        // Create fallback providers - try other providers if the primary fails
        let mut fallback_providers = Vec::new();
        
        // Add Claude as fallback for non-Anthropic providers
        if !matches!(participant.model_config.provider, crate::participant::LlmProvider::Anthropic) {
            if let Ok(fallback) = crate::llm::AnthropicProvider::claude_3_5_sonnet() {
                fallback_providers.push(Box::new(fallback) as Box<dyn LlmProvider>);
            }
        }
        
        // Add GPT-4 as fallback for non-OpenAI providers
        if !matches!(participant.model_config.provider, crate::participant::LlmProvider::OpenAi) {
            if let Ok(fallback) = crate::llm::OpenAiProvider::gpt_4_turbo() {
                fallback_providers.push(Box::new(fallback) as Box<dyn LlmProvider>);
            }
        }
        
        self.participants.push(ParticipantWithProvider { 
            participant, 
            provider: primary_provider,
            fallback_providers,
        });
        
        Ok(())
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ForgeEvent> {
        self.event_tx.subscribe()
    }
    
    pub async fn run_round(&mut self, round_type: RoundType) -> Result<(), LlmError> {
        let round = self.session.rounds.len() as u32 + 1;
        
        let _ = self.event_tx.send(ForgeEvent::RoundStarted { round, round_type });
        
        for pwp in &self.participants {
            if pwp.participant.is_human {
                // Skip human participants for now
                continue;
            }
            
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
            
            // Try primary provider first, then fallbacks
            let mut providers = vec![&*pwp.provider];
            for fallback in &pwp.fallback_providers {
                providers.push(&**fallback);
            }
            
            let mut last_error = None;
            let mut successful = false;
            
            for (i, provider) in providers.iter().enumerate() {
                let is_fallback = i > 0;
                
                if is_fallback {
                    let _ = self.event_tx.send(ForgeEvent::ParticipantError {
                        participant_id: participant_id.to_string(),
                        error: last_error.clone().unwrap_or_default(),
                        retrying_with: Some(format!("{} ({})", provider.name(), provider.model())),
                    });
                }
                
                let request = LlmRequest {
                    model: provider.model().to_string(),
                    messages: vec![
                        LlmMessage {
                            role: MessageRole::User,
                            content: prompt.clone(),
                        },
                    ],
                    temperature: Some(pwp.participant.model_config.temperature),
                    max_tokens: Some(pwp.participant.model_config.max_tokens),
                    system_prompt: if !pwp.participant.system_prompt.is_empty() {
                        Some(pwp.participant.system_prompt.clone())
                    } else {
                        None
                    },
                };
                
                match provider.complete_stream(request).await {
                    Ok(mut stream) => {
                        let mut full_content = String::new();
                        let mut stream_error = false;
                        
                        while let Some(chunk_result) = stream.next().await {
                            match chunk_result {
                                Ok(chunk) => {
                                    full_content.push_str(&chunk.delta);
                                    let _ = self.event_tx.send(ForgeEvent::ContentDelta {
                                        participant_id: participant_id.to_string(),
                                        delta: chunk.delta,
                                    });
                                    
                                    if chunk.is_complete {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    stream_error = true;
                                    last_error = Some(e.to_string());
                                    break;
                                }
                            }
                        }
                        
                        if !stream_error {
                            let _ = self.event_tx.send(ForgeEvent::ParticipantComplete {
                                participant_id: participant_id.to_string(),
                                content: full_content,
                                tokens: crate::session::TokenUsage::default(),
                            });
                            successful = true;
                            break;
                        }
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                    }
                }
            }
            
            if !successful {
                let error_msg = last_error.unwrap_or_else(|| "All providers failed".to_string());
                let _ = self.event_tx.send(ForgeEvent::Error {
                    message: format!("Participant {} failed: {}", participant_name, error_msg),
                });
                return Err(LlmError::ParseError(error_msg));
            }
        }
        
        let _ = self.event_tx.send(ForgeEvent::RoundComplete { round });
        
        Ok(())
    }
}