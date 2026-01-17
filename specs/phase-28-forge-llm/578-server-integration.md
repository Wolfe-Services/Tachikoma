# Spec 578: Server Integration

**Priority:** P0  
**Status:** planned  
**Depends on:** 577  
**Estimated Effort:** 3 hours  
**Target Files:**
- `crates/tachikoma-server/src/routes/v1.rs` (update)
- `crates/tachikoma-server/src/state.rs` (update if exists, else create)
- `crates/tachikoma-server/Cargo.toml` (add tachikoma-forge dependency)

---

## Overview

Wire the ForgeOrchestrator to the HTTP/WebSocket server so the frontend can start deliberations and receive streaming events.

---

## Acceptance Criteria

- [x] Add `tachikoma-forge` as a dependency in `crates/tachikoma-server/Cargo.toml`
- [x] Update `forge_create_session` handler to create real ForgeSession
- [x] Update `forge_start_round` handler to instantiate orchestrator and run a round
- [x] Add WebSocket endpoint for streaming ForgeEvents to clients
- [x] Store active orchestrators in shared state (Arc<RwLock<HashMap<SessionId, ForgeOrchestrator>>>)
- [x] Handle errors gracefully and return proper HTTP status codes
- [x] Verify `cargo check -p tachikoma-server` passes

---

## Implementation

```rust
// In crates/tachikoma-server/src/routes/v1.rs

use tachikoma_forge::{ForgeSession, ForgeOrchestrator, ForgeEvent};
use tachikoma_forge::llm::AnthropicProvider;
use axum::{
    extract::{Path, State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    Json,
};
use tokio::sync::broadcast;

// Add to AppState
pub struct AppState {
    pub orchestrators: Arc<RwLock<HashMap<String, broadcast::Sender<ForgeEvent>>>>,
}

pub async fn forge_create_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    let session = ForgeSession::new(request.name, request.goal);
    let session_id = session.id.clone();
    
    let (tx, _) = broadcast::channel(100);
    state.orchestrators.write().await.insert(session_id.clone(), tx);
    
    Json(session)
}

pub async fn forge_start_round(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<StartRoundRequest>,
) -> impl IntoResponse {
    // Get or create orchestrator
    let tx = match state.orchestrators.read().await.get(&session_id) {
        Some(tx) => tx.clone(),
        None => return (StatusCode::NOT_FOUND, "Session not found").into_response(),
    };
    
    // Create provider for each participant
    let provider = match AnthropicProvider::claude_sonnet_4() {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    
    // Run in background task
    tokio::spawn(async move {
        let mut orchestrator = ForgeOrchestrator::new(session, tx);
        // Add participants...
        if let Err(e) = orchestrator.run_round(request.round_type.into()).await {
            eprintln!("Round failed: {}", e);
        }
    });
    
    StatusCode::ACCEPTED.into_response()
}

pub async fn forge_events_ws(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_forge_ws(socket, state, session_id))
}

async fn handle_forge_ws(mut socket: WebSocket, state: Arc<AppState>, session_id: String) {
    let rx = match state.orchestrators.read().await.get(&session_id) {
        Some(tx) => tx.subscribe(),
        None => return,
    };
    
    let mut rx = rx;
    while let Ok(event) = rx.recv().await {
        let json = serde_json::to_string(&event).unwrap();
        if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
            break;
        }
    }
}
```

---

## Routes to add

```rust
pub fn forge_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sessions", post(forge_create_session))
        .route("/sessions/:id", get(forge_get_session))
        .route("/sessions/:id/round", post(forge_start_round))
        .route("/sessions/:id/events", get(forge_events_ws))
}
```
