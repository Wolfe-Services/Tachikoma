use napi::bindgen_prelude::*;
use napi_derive::napi;
use tachikoma_forge::{
    ForgeSession, ForgeSessionConfig, ForgeTopic, ForgeSessionStatus,
    ForgeEvent, TokenUsage as ForgeTokenUsage,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

// Global session storage
type SessionStorage = Arc<Mutex<HashMap<String, ForgeSession>>>;
static mut SESSIONS: Option<SessionStorage> = None;

fn get_sessions() -> &'static SessionStorage {
    unsafe {
        SESSIONS.get_or_insert_with(|| Arc::new(Mutex::new(HashMap::new())))
    }
}

// JavaScript-compatible types
#[napi(object)]
pub struct JsTokenUsage {
    pub input: u32,
    pub output: u32,
}

impl From<ForgeTokenUsage> for JsTokenUsage {
    fn from(usage: ForgeTokenUsage) -> Self {
        Self {
            input: usage.input,
            output: usage.output,
        }
    }
}

#[napi(object)]
pub struct JsForgeParticipant {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub role: String,
    pub model_id: Option<String>,
    pub status: String,
}

#[napi(object)]
pub struct JsForgeOracle {
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub config: serde_json::Value,
}

#[napi(object)]
pub struct JsForgeSessionConfig {
    pub max_rounds: u32,
    pub convergence_threshold: f64,
    pub round_timeout_ms: u32,
    pub allow_human_intervention: bool,
}

impl From<JsForgeSessionConfig> for ForgeSessionConfig {
    fn from(js_config: JsForgeSessionConfig) -> Self {
        Self {
            max_rounds: js_config.max_rounds as usize,
            convergence_threshold: js_config.convergence_threshold,
            round_timeout_ms: js_config.round_timeout_ms as u64,
        }
    }
}

#[napi(object)]
pub struct JsForgeSessionRequest {
    pub name: String,
    pub goal: String,
    pub participants: Vec<JsForgeParticipant>,
    pub oracle: Option<JsForgeOracle>,
    pub config: JsForgeSessionConfig,
}

#[napi(object)]
pub struct JsForgeSessionResponse {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub phase: String,
    pub participants: Vec<JsForgeParticipant>,
    pub oracle: Option<JsForgeOracle>,
    pub config: JsForgeSessionConfig,
    pub round_count: u32,
    pub total_cost_usd: f64,
    pub total_tokens: JsTokenUsage,
    pub created_at: String,
    pub updated_at: String,
}

fn forge_session_status_to_phase(status: &ForgeSessionStatus) -> String {
    match status {
        ForgeSessionStatus::Creating => "configuring".to_string(),
        ForgeSessionStatus::Active => "deliberating".to_string(),
        ForgeSessionStatus::Converged => "completed".to_string(),
        ForgeSessionStatus::Stopped => "paused".to_string(),
        ForgeSessionStatus::Failed(_) => "error".to_string(),
    }
}

impl From<ForgeSession> for JsForgeSessionResponse {
    fn from(session: ForgeSession) -> Self {
        Self {
            id: session.id.to_string(),
            name: session.topic.title.clone(),
            goal: session.topic.description.clone(),
            phase: forge_session_status_to_phase(&session.status),
            participants: vec![], // TODO: Implement participant conversion
            oracle: None, // TODO: Implement oracle conversion
            config: JsForgeSessionConfig {
                max_rounds: session.config.max_rounds as u32,
                convergence_threshold: session.config.convergence_threshold,
                round_timeout_ms: session.config.round_timeout_ms as u32,
                allow_human_intervention: true, // Default value for now
            },
            round_count: session.rounds.len() as u32,
            total_cost_usd: session.total_cost_usd,
            total_tokens: session.total_tokens.into(),
            created_at: session.created_at.to_rfc3339(),
            updated_at: session.updated_at.to_rfc3339(),
        }
    }
}

#[napi]
pub fn create_forge_session(request: JsForgeSessionRequest) -> Result<JsForgeSessionResponse> {
    let topic = ForgeTopic {
        title: request.name.clone(),
        description: request.goal.clone(),
        constraints: vec![], // TODO: Add constraints support
    };
    
    let config = request.config.into();
    let session = ForgeSession::new(config, topic);
    let session_id = session.id.to_string();
    let response = JsForgeSessionResponse::from(session.clone());
    
    // Store session
    let sessions = get_sessions();
    sessions.lock().unwrap().insert(session_id, session);
    
    Ok(response)
}

#[napi]
pub fn get_session(session_id: String) -> Result<Option<JsForgeSessionResponse>> {
    let sessions = get_sessions();
    let session_map = sessions.lock().unwrap();
    
    if let Some(session) = session_map.get(&session_id) {
        Ok(Some(JsForgeSessionResponse::from(session.clone())))
    } else {
        Ok(None)
    }
}

#[napi]
pub fn list_sessions() -> Result<Vec<JsForgeSessionResponse>> {
    let sessions = get_sessions();
    let session_map = sessions.lock().unwrap();
    
    let responses: Vec<JsForgeSessionResponse> = session_map
        .values()
        .map(|session| JsForgeSessionResponse::from(session.clone()))
        .collect();
    
    Ok(responses)
}

// Stream handle for deliberation events
#[napi]
pub struct DeliberationStream {
    session_id: String,
    receiver: Option<broadcast::Receiver<ForgeEvent>>,
}

#[napi]
impl DeliberationStream {
    #[napi]
    pub async fn next(&mut self) -> Result<Option<String>> {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.recv().await {
                Ok(event) => {
                    let event_json = serde_json::to_string(&serde_json::json!({
                        "type": match event {
                            ForgeEvent::RoundStarted { .. } => "round_started",
                            ForgeEvent::ParticipantThinking { .. } => "participant_thinking",
                            ForgeEvent::ContentDelta { .. } => "content_delta",
                            ForgeEvent::ParticipantComplete { .. } => "participant_complete",
                            ForgeEvent::RoundComplete { .. } => "round_complete",
                            ForgeEvent::Error { .. } => "error",
                        },
                        "data": event
                    }))
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                    Ok(Some(event_json))
                }
                Err(_) => Ok(None), // Channel closed
            }
        } else {
            Ok(None)
        }
    }
    
    #[napi]
    pub fn close(&mut self) {
        self.receiver = None;
    }
}

#[napi]
pub fn start_deliberation(session_id: String) -> Result<DeliberationStream> {
    // For now, return a mock stream
    // TODO: Implement actual orchestrator integration
    let (tx, rx) = broadcast::channel(100);
    
    Ok(DeliberationStream {
        session_id,
        receiver: Some(rx),
    })
}

#[napi]
pub fn stop_deliberation(session_id: String) -> Result<bool> {
    // Update session status to stopped
    let sessions = get_sessions();
    let mut session_map = sessions.lock().unwrap();
    
    if let Some(session) = session_map.get_mut(&session_id) {
        session.set_status(ForgeSessionStatus::Stopped);
        Ok(true)
    } else {
        Ok(false)
    }
}