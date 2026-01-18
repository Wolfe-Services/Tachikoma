use napi::bindgen_prelude::*;
use napi_derive::napi;
use tachikoma_forge::{
    ForgeSession, ForgeSessionConfig, ForgeTopic, ForgeSessionStatus,
    ForgeEvent, TokenUsage as ForgeTokenUsage, ForgeOrchestrator, RoundType,
    Participant, ParticipantRole,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

// Global session storage
type SessionStorage = Arc<Mutex<HashMap<String, ForgeSession>>>;
static mut SESSIONS: Option<SessionStorage> = None;

type ParticipantStorage = Arc<Mutex<HashMap<String, Vec<Participant>>>>;
static mut SESSION_PARTICIPANTS: Option<ParticipantStorage> = None;

type ActiveDeliberationStorage = Arc<Mutex<HashMap<String, JoinHandle<()>>>>;
static mut ACTIVE_DELIBERATIONS: Option<ActiveDeliberationStorage> = None;

fn get_sessions() -> &'static SessionStorage {
    unsafe {
        SESSIONS.get_or_insert_with(|| Arc::new(Mutex::new(HashMap::new())))
    }
}

fn get_participants() -> &'static ParticipantStorage {
    unsafe {
        SESSION_PARTICIPANTS.get_or_insert_with(|| Arc::new(Mutex::new(HashMap::new())))
    }
}

fn get_active_deliberations() -> &'static ActiveDeliberationStorage {
    unsafe {
        ACTIVE_DELIBERATIONS.get_or_insert_with(|| Arc::new(Mutex::new(HashMap::new())))
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
#[derive(Clone)]
pub struct JsForgeParticipant {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub role: String,
    pub model_id: Option<String>,
    pub status: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct JsForgeOracle {
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub config: String,  // JSON string instead of Value
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
        // Initial integration runs a Draft round first; treat "active" as "drafting" for UI.
        ForgeSessionStatus::Active => "drafting".to_string(),
        ForgeSessionStatus::Converged => "completed".to_string(),
        ForgeSessionStatus::Stopped => "paused".to_string(),
        ForgeSessionStatus::Failed(_) => "error".to_string(),
    }
}

fn parse_role(role: &str) -> ParticipantRole {
    match role.to_ascii_lowercase().as_str() {
        "architect" => ParticipantRole::Architect,
        "critic" => ParticipantRole::Critic,
        "advocate" => ParticipantRole::Advocate,
        "synthesizer" => ParticipantRole::Synthesizer,
        "specialist" => ParticipantRole::Specialist,
        other => ParticipantRole::Custom(other.to_string()),
    }
}

fn to_participant(p: &JsForgeParticipant) -> Participant {
    let role = parse_role(&p.role);
    if p.r#type.to_ascii_lowercase() == "human" {
        Participant::human(p.name.clone(), role)
    } else {
        let model_id = p
            .model_id
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        let mut builder = Participant::builder(p.name.clone()).role(role);
        let model_id_lc = model_id.to_ascii_lowercase();

        if model_id_lc.starts_with("ollama/") || model_id_lc.contains("ollama") {
            builder = builder.ollama(&model_id);
        } else if model_id_lc.contains("gpt") || model_id_lc.contains("openai") {
            builder = builder.openai(&model_id);
        } else {
            builder = builder.anthropic(&model_id);
        }

        builder.build()
    }
}

impl From<ForgeSession> for JsForgeSessionResponse {
    fn from(session: ForgeSession) -> Self {
        Self {
            id: session.id.to_string(),
            name: session.topic.title.clone(),
            goal: session.topic.description.clone(),
            phase: forge_session_status_to_phase(&session.status),
            participants: vec![], // Filled during create_forge_session for now
            oracle: None, // Filled during create_forge_session for now
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
    let mut response = JsForgeSessionResponse::from(session.clone());
    response.participants = request.participants.clone();
    response.oracle = request.oracle.clone();
    
    // Store session
    let sessions = get_sessions();
    sessions.lock().unwrap().insert(session_id, session);

    // Store participants for deliberation
    let participants = response.participants.iter().map(to_participant).collect::<Vec<_>>();
    let participant_store = get_participants();
    participant_store
        .lock()
        .unwrap()
        .insert(response.id.clone(), participants);
    
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
    pub async unsafe fn next(&mut self) -> Result<Option<String>> {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.recv().await {
                Ok(event) => {
                    let event_json = serde_json::to_string(&serde_json::json!({
                        "type": match &event {
                            ForgeEvent::RoundStarted { .. } => "round_started",
                            ForgeEvent::ParticipantThinking { .. } => "participant_thinking",
                            ForgeEvent::ContentDelta { .. } => "content_delta",
                            ForgeEvent::ParticipantComplete { .. } => "participant_complete",
                            ForgeEvent::ParticipantError { .. } => "participant_error",
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
    // Stop any prior deliberation for this session (best-effort)
    {
        let active = get_active_deliberations();
        if let Some(handle) = active.lock().unwrap().remove(&session_id) {
            handle.abort();
        }
    }

    let (event_tx, rx) = broadcast::channel(256);

    // Fetch session + participants snapshot
    let session_opt = {
        let sessions = get_sessions();
        sessions.lock().unwrap().get(&session_id).cloned()
    };
    let Some(session) = session_opt else {
        return Ok(DeliberationStream {
            session_id,
            receiver: Some(rx),
        });
    };

    let participants = {
        let store = get_participants();
        store
            .lock()
            .unwrap()
            .get(&session_id)
            .cloned()
            .unwrap_or_default()
    };

    // Mark session active
    {
        let sessions = get_sessions();
        if let Some(s) = sessions.lock().unwrap().get_mut(&session_id) {
            s.set_status(ForgeSessionStatus::Active);
        }
    }

    // Spawn orchestrator: run a simple multi-round pipeline so models can respond to each other.
    let sessions_for_task: SessionStorage = get_sessions().clone();
    let session_id_for_task = session_id.clone();
    let handle = tokio::spawn(async move {
        let mut orchestrator = ForgeOrchestrator::new(session, event_tx.clone());

        for p in participants {
            if let Err(e) = orchestrator.add_participant(p) {
                let _ = event_tx.send(ForgeEvent::Error {
                    message: format!("Failed to add participant: {}", e),
                });
            }
        }

        let result = async {
            orchestrator.run_round(RoundType::Draft).await?;
            orchestrator.run_round(RoundType::Critique).await?;
            orchestrator.run_round(RoundType::Synthesis).await?;
            Ok::<(), tachikoma_forge::llm::LlmError>(())
        }
        .await;

        match result {
            Ok(_) => {
                if let Some(s) = sessions_for_task.lock().unwrap().get_mut(&session_id_for_task) {
                    *s = orchestrator.session.clone();
                    s.set_status(ForgeSessionStatus::Converged);
                }
            }
            Err(e) => {
                let _ = event_tx.send(ForgeEvent::Error {
                    message: format!("Deliberation failed: {}", e),
                });
                if let Some(s) = sessions_for_task.lock().unwrap().get_mut(&session_id_for_task) {
                    s.set_status(ForgeSessionStatus::Failed(e.to_string()));
                }
            }
        };
    });

    {
        let active = get_active_deliberations();
        active.lock().unwrap().insert(session_id.clone(), handle);
    }
    
    Ok(DeliberationStream {
        session_id,
        receiver: Some(rx),
    })
}

#[napi]
pub fn stop_deliberation(session_id: String) -> Result<bool> {
    // Abort running task (if any)
    {
        let active = get_active_deliberations();
        if let Some(handle) = active.lock().unwrap().remove(&session_id) {
            handle.abort();
        }
    }

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