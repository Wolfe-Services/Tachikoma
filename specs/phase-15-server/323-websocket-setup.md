# Spec 323: WebSocket Setup

## Phase
15 - Server/API Layer

## Spec ID
323

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 312: Server Configuration

## Estimated Context
~10%

---

## Objective

Implement WebSocket support for the Tachikoma server, enabling real-time bidirectional communication for streaming LLM responses, live updates, and collaborative features.

---

## Acceptance Criteria

- [ ] WebSocket endpoint accepts connections
- [ ] Connection authentication and validation
- [ ] Heartbeat/ping-pong for connection health
- [ ] Graceful connection handling and cleanup
- [ ] Message framing and protocol definition
- [ ] Connection state management
- [ ] Reconnection support with session resumption

---

## Implementation Details

### WebSocket Configuration

```rust
// src/server/websocket/config.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WebSocketConfig {
    /// Enable WebSocket support
    pub enabled: bool,

    /// Maximum message size in bytes
    pub max_message_size: usize,

    /// Maximum frame size in bytes
    pub max_frame_size: usize,

    /// Ping interval in seconds
    pub ping_interval_secs: u64,

    /// Pong timeout in seconds
    pub pong_timeout_secs: u64,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Maximum connections per client
    pub max_connections_per_client: usize,

    /// Enable compression
    pub compression: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_message_size: 16 * 1024 * 1024, // 16MB
            max_frame_size: 64 * 1024,           // 64KB
            ping_interval_secs: 30,
            pong_timeout_secs: 10,
            connection_timeout_secs: 60,
            max_connections_per_client: 5,
            compression: true,
        }
    }
}

impl WebSocketConfig {
    pub fn ping_interval(&self) -> Duration {
        Duration::from_secs(self.ping_interval_secs)
    }

    pub fn pong_timeout(&self) -> Duration {
        Duration::from_secs(self.pong_timeout_secs)
    }

    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }
}
```

### Connection Manager

```rust
// src/server/websocket/manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use super::connection::WebSocketConnection;

/// Manages all active WebSocket connections
pub struct ConnectionManager {
    connections: RwLock<HashMap<Uuid, Arc<WebSocketConnection>>>,
    client_connections: RwLock<HashMap<String, Vec<Uuid>>>,
    broadcast_tx: mpsc::Sender<BroadcastMessage>,
}

pub struct BroadcastMessage {
    pub channel: Option<String>,
    pub message: String,
    pub exclude: Option<Uuid>,
}

impl ConnectionManager {
    pub fn new() -> (Self, mpsc::Receiver<BroadcastMessage>) {
        let (broadcast_tx, broadcast_rx) = mpsc::channel(1000);

        let manager = Self {
            connections: RwLock::new(HashMap::new()),
            client_connections: RwLock::new(HashMap::new()),
            broadcast_tx,
        };

        (manager, broadcast_rx)
    }

    /// Register a new connection
    pub async fn register(&self, connection: Arc<WebSocketConnection>) {
        let id = connection.id();
        let client_id = connection.client_id().map(|s| s.to_string());

        // Add to connections map
        self.connections.write().await.insert(id, connection.clone());

        // Track client connections
        if let Some(ref client) = client_id {
            self.client_connections
                .write()
                .await
                .entry(client.clone())
                .or_default()
                .push(id);
        }

        tracing::info!(
            connection_id = %id,
            client_id = ?client_id,
            total_connections = %self.connections.read().await.len(),
            "WebSocket connection registered"
        );
    }

    /// Unregister a connection
    pub async fn unregister(&self, id: Uuid) {
        if let Some(connection) = self.connections.write().await.remove(&id) {
            // Remove from client tracking
            if let Some(client_id) = connection.client_id() {
                let mut client_conns = self.client_connections.write().await;
                if let Some(conns) = client_conns.get_mut(client_id) {
                    conns.retain(|&conn_id| conn_id != id);
                    if conns.is_empty() {
                        client_conns.remove(client_id);
                    }
                }
            }

            tracing::info!(
                connection_id = %id,
                total_connections = %self.connections.read().await.len(),
                "WebSocket connection unregistered"
            );
        }
    }

    /// Get a connection by ID
    pub async fn get(&self, id: Uuid) -> Option<Arc<WebSocketConnection>> {
        self.connections.read().await.get(&id).cloned()
    }

    /// Get all connections for a client
    pub async fn get_client_connections(&self, client_id: &str) -> Vec<Arc<WebSocketConnection>> {
        let client_conns = self.client_connections.read().await;
        let connections = self.connections.read().await;

        client_conns
            .get(client_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| connections.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Count connections for a client
    pub async fn client_connection_count(&self, client_id: &str) -> usize {
        self.client_connections
            .read()
            .await
            .get(client_id)
            .map(|c| c.len())
            .unwrap_or(0)
    }

    /// Send message to a specific connection
    pub async fn send_to(&self, id: Uuid, message: String) -> Result<(), SendError> {
        if let Some(conn) = self.get(id).await {
            conn.send(message).await
        } else {
            Err(SendError::ConnectionNotFound)
        }
    }

    /// Send message to all connections of a client
    pub async fn send_to_client(&self, client_id: &str, message: String) -> Result<(), SendError> {
        let connections = self.get_client_connections(client_id).await;

        for conn in connections {
            if let Err(e) = conn.send(message.clone()).await {
                tracing::warn!(
                    connection_id = %conn.id(),
                    error = %e,
                    "Failed to send to client connection"
                );
            }
        }

        Ok(())
    }

    /// Broadcast to all connections
    pub async fn broadcast(&self, message: String, exclude: Option<Uuid>) {
        let _ = self.broadcast_tx.send(BroadcastMessage {
            channel: None,
            message,
            exclude,
        }).await;
    }

    /// Broadcast to a specific channel
    pub async fn broadcast_to_channel(&self, channel: &str, message: String) {
        let _ = self.broadcast_tx.send(BroadcastMessage {
            channel: Some(channel.to_string()),
            message,
            exclude: None,
        }).await;
    }

    /// Get total connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get all connection IDs
    pub async fn connection_ids(&self) -> Vec<Uuid> {
        self.connections.read().await.keys().cloned().collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Connection not found")]
    ConnectionNotFound,
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Connection closed")]
    ConnectionClosed,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new().0
    }
}
```

### WebSocket Connection

```rust
// src/server/websocket/connection.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;
use futures::{SinkExt, StreamExt};
use axum::extract::ws::{Message, WebSocket};

use super::config::WebSocketConfig;
use super::manager::SendError;
use super::protocol::{ClientMessage, ServerMessage};

/// Represents a single WebSocket connection
pub struct WebSocketConnection {
    id: Uuid,
    client_id: Option<String>,
    session_id: Option<String>,
    sender: mpsc::Sender<String>,
    subscriptions: Mutex<Vec<String>>,
    last_activity: Mutex<Instant>,
    created_at: chrono::DateTime<chrono::Utc>,
    metadata: Mutex<ConnectionMetadata>,
}

#[derive(Debug, Default)]
pub struct ConnectionMetadata {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl WebSocketConnection {
    /// Create a new connection handler
    pub fn new(
        socket: WebSocket,
        config: &WebSocketConfig,
        client_id: Option<String>,
    ) -> (Arc<Self>, impl std::future::Future<Output = ()>) {
        let id = Uuid::new_v4();
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<String>(100);

        let connection = Arc::new(Self {
            id,
            client_id,
            session_id: Some(Uuid::new_v4().to_string()),
            sender: outgoing_tx,
            subscriptions: Mutex::new(Vec::new()),
            last_activity: Mutex::new(Instant::now()),
            created_at: chrono::Utc::now(),
            metadata: Mutex::new(ConnectionMetadata::default()),
        });

        let handler = Self::run(connection.clone(), socket, outgoing_rx, config.clone());

        (connection, handler)
    }

    /// Main connection handler loop
    async fn run(
        connection: Arc<Self>,
        socket: WebSocket,
        mut outgoing_rx: mpsc::Receiver<String>,
        config: WebSocketConfig,
    ) {
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Ping task
        let ping_connection = connection.clone();
        let ping_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.ping_interval());
            loop {
                interval.tick().await;

                let last = *ping_connection.last_activity.lock().await;
                if last.elapsed() > config.pong_timeout() * 2 {
                    tracing::warn!(
                        connection_id = %ping_connection.id,
                        "Connection timed out"
                    );
                    break;
                }
            }
        });

        // Outgoing message task
        let outgoing_task = tokio::spawn(async move {
            while let Some(message) = outgoing_rx.recv().await {
                if ws_sender.send(Message::Text(message)).await.is_err() {
                    break;
                }
            }
        });

        // Incoming message loop
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(message) => {
                    *connection.last_activity.lock().await = Instant::now();

                    match message {
                        Message::Text(text) => {
                            connection.metadata.lock().await.messages_received += 1;
                            connection.handle_message(&text).await;
                        }
                        Message::Binary(data) => {
                            // Handle binary messages if needed
                            tracing::debug!(
                                connection_id = %connection.id,
                                size = data.len(),
                                "Received binary message"
                            );
                        }
                        Message::Ping(data) => {
                            if let Err(e) = connection.sender.send(
                                serde_json::to_string(&ServerMessage::Pong).unwrap()
                            ).await {
                                tracing::warn!(error = %e, "Failed to send pong");
                            }
                        }
                        Message::Pong(_) => {
                            // Pong received, connection is alive
                        }
                        Message::Close(_) => {
                            tracing::info!(
                                connection_id = %connection.id,
                                "Client initiated close"
                            );
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        connection_id = %connection.id,
                        error = %e,
                        "WebSocket error"
                    );
                    break;
                }
            }
        }

        // Cleanup
        ping_task.abort();
        outgoing_task.abort();

        tracing::info!(
            connection_id = %connection.id,
            duration_secs = %(chrono::Utc::now() - connection.created_at).num_seconds(),
            "Connection closed"
        );
    }

    /// Handle an incoming message
    async fn handle_message(&self, text: &str) {
        match serde_json::from_str::<ClientMessage>(text) {
            Ok(message) => {
                tracing::debug!(
                    connection_id = %self.id,
                    message_type = ?message.message_type(),
                    "Received message"
                );

                // Message handling is delegated to the event processor
                // This will be handled by the WebSocket event system (Spec 324)
            }
            Err(e) => {
                tracing::warn!(
                    connection_id = %self.id,
                    error = %e,
                    "Failed to parse message"
                );

                let error = ServerMessage::Error {
                    code: "INVALID_MESSAGE".to_string(),
                    message: "Failed to parse message".to_string(),
                };

                let _ = self.send(serde_json::to_string(&error).unwrap()).await;
            }
        }
    }

    /// Send a message to this connection
    pub async fn send(&self, message: String) -> Result<(), SendError> {
        self.metadata.lock().await.messages_sent += 1;

        self.sender
            .send(message)
            .await
            .map_err(|_| SendError::ConnectionClosed)
    }

    /// Subscribe to a channel
    pub async fn subscribe(&self, channel: String) {
        let mut subs = self.subscriptions.lock().await;
        if !subs.contains(&channel) {
            subs.push(channel);
        }
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: &str) {
        self.subscriptions.lock().await.retain(|c| c != channel);
    }

    /// Check if subscribed to a channel
    pub async fn is_subscribed(&self, channel: &str) -> bool {
        self.subscriptions.lock().await.contains(&channel.to_string())
    }

    /// Get connection ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get client ID
    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get subscriptions
    pub async fn subscriptions(&self) -> Vec<String> {
        self.subscriptions.lock().await.clone()
    }

    /// Get connection age
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.created_at
    }
}
```

### WebSocket Handler

```rust
// src/server/handlers/websocket.rs
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::Deserialize;

use crate::server::state::AppState;
use crate::server::websocket::{WebSocketConnection, ConnectionManager};

#[derive(Debug, Deserialize)]
pub struct WebSocketParams {
    /// Client identifier for reconnection
    pub client_id: Option<String>,
    /// Session ID for resuming
    pub session_id: Option<String>,
    /// Authentication token
    pub token: Option<String>,
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WebSocketParams>,
) -> Response {
    // Validate token if authentication is required
    if state.config().security.api_key.is_some() {
        if let Some(token) = &params.token {
            if !validate_token(token, &state).await {
                return Response::builder()
                    .status(401)
                    .body("Unauthorized".into())
                    .unwrap();
            }
        } else {
            return Response::builder()
                .status(401)
                .body("Token required".into())
                .unwrap();
        }
    }

    // Check connection limits
    if let Some(ref client_id) = params.client_id {
        let count = state.ws_manager().client_connection_count(client_id).await;
        let max = state.config().websocket.max_connections_per_client;

        if count >= max {
            return Response::builder()
                .status(429)
                .body(format!("Maximum {} connections per client", max).into())
                .unwrap();
        }
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, params))
}

async fn handle_socket(socket: WebSocket, state: AppState, params: WebSocketParams) {
    let config = state.config().websocket.clone();
    let manager = state.ws_manager();

    // Create connection
    let (connection, handler) = WebSocketConnection::new(
        socket,
        &config,
        params.client_id,
    );

    // Register with manager
    manager.register(connection.clone()).await;

    // Send welcome message
    let welcome = ServerMessage::Welcome {
        connection_id: connection.id().to_string(),
        session_id: connection.session_id().map(|s| s.to_string()),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let _ = connection.send(serde_json::to_string(&welcome).unwrap()).await;

    // Run the connection handler
    handler.await;

    // Unregister when done
    manager.unregister(connection.id()).await;
}

async fn validate_token(token: &str, state: &AppState) -> bool {
    if let Some(ref api_key) = state.config().security.api_key {
        token == api_key.expose()
    } else {
        true
    }
}
```

### Routes

```rust
// src/server/routes/websocket.rs
use axum::{
    Router,
    routing::get,
};

use crate::server::state::AppState;
use crate::server::handlers::websocket as handlers;

pub fn ws_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::websocket_handler))
        .route("/stream", get(handlers::stream_handler))
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager_register_unregister() {
        let (manager, _rx) = ConnectionManager::new();

        // Create mock connection
        let id = Uuid::new_v4();
        // ... setup mock connection

        assert_eq!(manager.connection_count().await, 0);

        // Register
        // manager.register(connection).await;
        // assert_eq!(manager.connection_count().await, 1);

        // Unregister
        manager.unregister(id).await;
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_client_connection_limit() {
        let (manager, _rx) = ConnectionManager::new();
        let client_id = "test-client";

        // Add multiple connections
        for _ in 0..5 {
            // ... create and register connections
        }

        assert_eq!(manager.client_connection_count(client_id).await, 5);
    }

    #[tokio::test]
    async fn test_subscription_management() {
        // Create connection and test subscriptions
        // ... test implementation
    }
}
```

---

## Related Specs

- **Spec 324**: WebSocket Events
- **Spec 325**: WebSocket Streaming
- **Spec 326**: SSE Streaming (alternative)
