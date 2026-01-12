# 329 - WebSocket Setup

**Phase:** 15 - Server
**Spec ID:** 329
**Status:** Planned
**Dependencies:** 317-axum-router
**Estimated Context:** ~7% of Sonnet window

---

## Objective

Set up WebSocket infrastructure for real-time communication including connection handling, upgrade negotiation, and session management.

---

## Acceptance Criteria

- [ ] WebSocket upgrade handling
- [ ] Connection lifecycle management
- [ ] Ping/pong heartbeat
- [ ] Connection authentication
- [ ] Session state management
- [ ] Graceful disconnection
- [ ] Reconnection support

---

## Implementation Details

### 1. WebSocket Config (crates/tachikoma-server/src/websocket/config.rs)

```rust
//! WebSocket configuration.

use std::time::Duration;

/// WebSocket configuration.
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Maximum message size in bytes.
    pub max_message_size: usize,
    /// Ping interval for keepalive.
    pub ping_interval: Duration,
    /// Pong timeout (disconnect if no pong received).
    pub pong_timeout: Duration,
    /// Maximum pending messages in send buffer.
    pub max_pending_messages: usize,
    /// Whether to require authentication.
    pub require_auth: bool,
    /// Authentication timeout.
    pub auth_timeout: Duration,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_message_size: 64 * 1024, // 64KB
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
            max_pending_messages: 100,
            require_auth: true,
            auth_timeout: Duration::from_secs(5),
        }
    }
}

impl WebSocketConfig {
    /// Create config for development (more permissive).
    pub fn development() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            ping_interval: Duration::from_secs(60),
            pong_timeout: Duration::from_secs(30),
            max_pending_messages: 1000,
            require_auth: false,
            auth_timeout: Duration::from_secs(30),
        }
    }

    /// Create config for production.
    pub fn production() -> Self {
        Self::default()
    }
}
```

### 2. WebSocket Session (crates/tachikoma-server/src/websocket/session.rs)

```rust
//! WebSocket session management.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// WebSocket session state.
#[derive(Debug, Clone)]
pub struct WsSession {
    /// Unique session ID.
    pub id: Uuid,
    /// User ID if authenticated.
    pub user_id: Option<Uuid>,
    /// Session subscriptions.
    pub subscriptions: Vec<String>,
    /// Connected at timestamp.
    pub connected_at: DateTime<Utc>,
    /// Last activity timestamp.
    pub last_activity: DateTime<Utc>,
    /// Client metadata.
    pub metadata: HashMap<String, String>,
}

impl WsSession {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            subscriptions: Vec::new(),
            connected_at: now,
            last_activity: now,
            metadata: HashMap::new(),
        }
    }

    /// Set authenticated user.
    pub fn authenticate(&mut self, user_id: Uuid) {
        self.user_id = Some(user_id);
    }

    /// Check if session is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Add subscription.
    pub fn subscribe(&mut self, topic: impl Into<String>) {
        let topic = topic.into();
        if !self.subscriptions.contains(&topic) {
            self.subscriptions.push(topic);
        }
    }

    /// Remove subscription.
    pub fn unsubscribe(&mut self, topic: &str) {
        self.subscriptions.retain(|t| t != topic);
    }

    /// Check if subscribed to topic.
    pub fn is_subscribed(&self, topic: &str) -> bool {
        self.subscriptions.iter().any(|t| t == topic || topic.starts_with(t))
    }

    /// Update last activity.
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }
}

impl Default for WsSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager for tracking all WebSocket connections.
pub struct SessionManager {
    sessions: RwLock<HashMap<Uuid, WsSession>>,
    /// Sender channels for each session.
    senders: RwLock<HashMap<Uuid, mpsc::Sender<WsOutgoingMessage>>>,
}

/// Outgoing message to send to a WebSocket.
#[derive(Debug, Clone)]
pub enum WsOutgoingMessage {
    Text(String),
    Binary(Vec<u8>),
    Close,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            senders: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new session.
    pub async fn register(
        &self,
        session: WsSession,
        sender: mpsc::Sender<WsOutgoingMessage>,
    ) {
        let id = session.id;
        self.sessions.write().await.insert(id, session);
        self.senders.write().await.insert(id, sender);
    }

    /// Unregister a session.
    pub async fn unregister(&self, session_id: Uuid) {
        self.sessions.write().await.remove(&session_id);
        self.senders.write().await.remove(&session_id);
    }

    /// Get session by ID.
    pub async fn get_session(&self, session_id: Uuid) -> Option<WsSession> {
        self.sessions.read().await.get(&session_id).cloned()
    }

    /// Update a session.
    pub async fn update_session<F>(&self, session_id: Uuid, f: F)
    where
        F: FnOnce(&mut WsSession),
    {
        if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
            f(session);
        }
    }

    /// Send message to a specific session.
    pub async fn send_to(&self, session_id: Uuid, message: WsOutgoingMessage) -> bool {
        if let Some(sender) = self.senders.read().await.get(&session_id) {
            sender.send(message).await.is_ok()
        } else {
            false
        }
    }

    /// Send message to all sessions subscribed to a topic.
    pub async fn broadcast_to_topic(&self, topic: &str, message: WsOutgoingMessage) {
        let sessions = self.sessions.read().await;
        let senders = self.senders.read().await;

        for (id, session) in sessions.iter() {
            if session.is_subscribed(topic) {
                if let Some(sender) = senders.get(id) {
                    let _ = sender.send(message.clone()).await;
                }
            }
        }
    }

    /// Send message to all authenticated sessions for a user.
    pub async fn send_to_user(&self, user_id: Uuid, message: WsOutgoingMessage) {
        let sessions = self.sessions.read().await;
        let senders = self.senders.read().await;

        for (id, session) in sessions.iter() {
            if session.user_id == Some(user_id) {
                if let Some(sender) = senders.get(id) {
                    let _ = sender.send(message.clone()).await;
                }
            }
        }
    }

    /// Broadcast to all connected sessions.
    pub async fn broadcast(&self, message: WsOutgoingMessage) {
        let senders = self.senders.read().await;
        for sender in senders.values() {
            let _ = sender.send(message.clone()).await;
        }
    }

    /// Get count of active sessions.
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get all sessions for a user.
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Vec<WsSession> {
        self.sessions
            .read()
            .await
            .values()
            .filter(|s| s.user_id == Some(user_id))
            .cloned()
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
```

### 3. WebSocket Handler (crates/tachikoma-server/src/websocket/handler.rs)

```rust
//! WebSocket connection handler.

use super::{
    config::WebSocketConfig,
    session::{SessionManager, WsOutgoingMessage, WsSession},
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// WebSocket handler state.
pub struct WsState {
    pub config: WebSocketConfig,
    pub session_manager: Arc<SessionManager>,
}

impl WsState {
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            session_manager: Arc::new(SessionManager::new()),
        }
    }
}

/// WebSocket upgrade handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection.
async fn handle_socket(socket: WebSocket, state: Arc<WsState>) {
    let session = WsSession::new();
    let session_id = session.id;

    info!(session_id = %session_id, "WebSocket connected");

    // Create channel for sending messages
    let (tx, mut rx) = mpsc::channel::<WsOutgoingMessage>(state.config.max_pending_messages);

    // Register session
    state.session_manager.register(session, tx).await;

    // Split socket
    let (mut sender, mut receiver) = socket.split();

    // Spawn sender task
    let sender_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let msg = match message {
                WsOutgoingMessage::Text(text) => Message::Text(text),
                WsOutgoingMessage::Binary(data) => Message::Binary(data),
                WsOutgoingMessage::Close => {
                    let _ = sender.close().await;
                    break;
                }
            };

            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Spawn ping task
    let ping_interval = state.config.ping_interval;
    let session_manager_ping = state.session_manager.clone();
    let ping_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(ping_interval);
        loop {
            interval.tick().await;
            if !session_manager_ping.send_to(session_id, WsOutgoingMessage::Text(
                r#"{"type":"ping"}"#.to_string()
            )).await {
                break;
            }
        }
    });

    // Handle incoming messages
    let session_manager = state.session_manager.clone();
    while let Some(result) = receiver.next().await {
        match result {
            Ok(message) => {
                // Update last activity
                session_manager.update_session(session_id, |s| s.touch()).await;

                match message {
                    Message::Text(text) => {
                        debug!(session_id = %session_id, "Received text message");
                        handle_text_message(&session_manager, session_id, &text).await;
                    }
                    Message::Binary(data) => {
                        debug!(session_id = %session_id, "Received binary message");
                        handle_binary_message(&session_manager, session_id, &data).await;
                    }
                    Message::Ping(_) => {
                        // Axum handles pong automatically
                    }
                    Message::Pong(_) => {
                        debug!(session_id = %session_id, "Received pong");
                    }
                    Message::Close(_) => {
                        info!(session_id = %session_id, "Client requested close");
                        break;
                    }
                }
            }
            Err(e) => {
                error!(session_id = %session_id, error = %e, "WebSocket error");
                break;
            }
        }
    }

    // Cleanup
    ping_task.abort();
    sender_task.abort();
    state.session_manager.unregister(session_id).await;

    info!(session_id = %session_id, "WebSocket disconnected");
}

async fn handle_text_message(
    session_manager: &SessionManager,
    session_id: uuid::Uuid,
    text: &str,
) {
    // Parse message and handle based on type
    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
            match msg_type {
                "pong" => {
                    // Client responded to ping
                }
                "subscribe" => {
                    if let Some(topic) = msg.get("topic").and_then(|v| v.as_str()) {
                        session_manager.update_session(session_id, |s| {
                            s.subscribe(topic);
                        }).await;
                    }
                }
                "unsubscribe" => {
                    if let Some(topic) = msg.get("topic").and_then(|v| v.as_str()) {
                        session_manager.update_session(session_id, |s| {
                            s.unsubscribe(topic);
                        }).await;
                    }
                }
                _ => {
                    warn!(session_id = %session_id, msg_type = msg_type, "Unknown message type");
                }
            }
        }
    }
}

async fn handle_binary_message(
    _session_manager: &SessionManager,
    session_id: uuid::Uuid,
    _data: &[u8],
) {
    warn!(session_id = %session_id, "Binary messages not supported");
}
```

### 4. WebSocket Router (crates/tachikoma-server/src/websocket/router.rs)

```rust
//! WebSocket routes.

use super::handler::{ws_handler, WsState};
use axum::{routing::get, Router};
use std::sync::Arc;

/// Create WebSocket routes.
pub fn ws_routes(state: Arc<WsState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
}
```

---

## Testing Requirements

1. Connection upgrade works
2. Ping/pong keepalive works
3. Session tracking accurate
4. Subscriptions work correctly
5. Broadcast reaches all subscribers
6. Clean disconnection
7. Session cleanup on disconnect

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [330-ws-message-types.md](330-ws-message-types.md)
- Used by: Real-time features
