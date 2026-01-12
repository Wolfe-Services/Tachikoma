# 331 - WebSocket Connection Management

**Phase:** 15 - Server
**Spec ID:** 331
**Status:** Planned
**Dependencies:** 329-websocket-setup, 330-ws-message-types
**Estimated Context:** ~7% of Sonnet window

---

## Objective

Implement WebSocket connection management with authentication integration, message routing, and connection pools.

---

## Acceptance Criteria

- [ ] Connection authentication flow
- [ ] Message routing to handlers
- [ ] Connection pool management
- [ ] Backpressure handling
- [ ] Connection metrics
- [ ] Graceful connection closure
- [ ] Connection rate limiting

---

## Implementation Details

### 1. Connection Manager (crates/tachikoma-server/src/websocket/connection/manager.rs)

```rust
//! WebSocket connection management.

use super::pool::ConnectionPool;
use crate::websocket::{
    config::WebSocketConfig,
    messages::{Command, OutgoingMessage, IncomingMessage, ErrorResponse},
    session::{SessionManager, WsOutgoingMessage, WsSession},
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Connection manager for WebSocket connections.
pub struct ConnectionManager {
    config: WebSocketConfig,
    session_manager: Arc<SessionManager>,
    pool: Arc<ConnectionPool>,
    message_handler: Arc<dyn MessageHandler + Send + Sync>,
}

/// Trait for handling WebSocket messages.
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(
        &self,
        session_id: Uuid,
        message: IncomingMessage,
    ) -> Option<OutgoingMessage>;
}

impl ConnectionManager {
    pub fn new(
        config: WebSocketConfig,
        session_manager: Arc<SessionManager>,
        message_handler: Arc<dyn MessageHandler + Send + Sync>,
    ) -> Self {
        Self {
            config,
            session_manager,
            pool: Arc::new(ConnectionPool::new(1000)), // Max 1000 connections
            message_handler,
        }
    }

    /// Handle new WebSocket connection.
    pub async fn handle_connection(
        &self,
        session_id: Uuid,
        mut receiver: mpsc::Receiver<String>,
        sender: mpsc::Sender<WsOutgoingMessage>,
    ) {
        // Create session
        let session = WsSession::new();
        let session_id = session.id;

        // Register with session manager
        self.session_manager.register(session, sender.clone()).await;

        // Add to connection pool
        self.pool.add(session_id).await;

        info!(session_id = %session_id, "New WebSocket connection");

        // Handle authentication if required
        if self.config.require_auth {
            match self.handle_authentication(session_id, &mut receiver, &sender).await {
                Ok(true) => {
                    debug!(session_id = %session_id, "Authentication successful");
                }
                Ok(false) => {
                    warn!(session_id = %session_id, "Authentication failed");
                    self.close_connection(session_id, "Authentication failed").await;
                    return;
                }
                Err(e) => {
                    error!(session_id = %session_id, error = %e, "Authentication error");
                    self.close_connection(session_id, &e).await;
                    return;
                }
            }
        }

        // Process messages
        while let Some(text) = receiver.recv().await {
            match self.process_message(session_id, &text).await {
                Ok(Some(response)) => {
                    let _ = sender.send(WsOutgoingMessage::Text(response.to_json())).await;
                }
                Ok(None) => {}
                Err(e) => {
                    let error = OutgoingMessage::new("error", ErrorResponse::new("error", e));
                    let _ = sender.send(WsOutgoingMessage::Text(error.to_json())).await;
                }
            }
        }

        // Cleanup
        self.cleanup_connection(session_id).await;
    }

    /// Handle authentication flow.
    async fn handle_authentication(
        &self,
        session_id: Uuid,
        receiver: &mut mpsc::Receiver<String>,
        sender: &mpsc::Sender<WsOutgoingMessage>,
    ) -> Result<bool, String> {
        use tokio::time::{timeout, Duration};

        let auth_timeout = self.config.auth_timeout;

        // Wait for auth message
        let result = timeout(auth_timeout, receiver.recv()).await;

        match result {
            Ok(Some(text)) => {
                if let Ok(msg) = serde_json::from_str::<IncomingMessage>(&text) {
                    if msg.msg_type == "authenticate" {
                        if let Ok(token) = msg.payload.get("token")
                            .and_then(|v| v.as_str())
                            .ok_or("Missing token")
                        {
                            // Validate token and authenticate session
                            match self.validate_token(token).await {
                                Ok(user_id) => {
                                    self.session_manager
                                        .update_session(session_id, |s| s.authenticate(user_id))
                                        .await;

                                    let response = OutgoingMessage::new("auth_result", serde_json::json!({
                                        "authenticated": true,
                                        "user_id": user_id.to_string(),
                                    }));
                                    let _ = sender.send(WsOutgoingMessage::Text(response.to_json())).await;

                                    return Ok(true);
                                }
                                Err(e) => {
                                    let response = OutgoingMessage::new("auth_result", serde_json::json!({
                                        "authenticated": false,
                                        "error": e,
                                    }));
                                    let _ = sender.send(WsOutgoingMessage::Text(response.to_json())).await;

                                    return Ok(false);
                                }
                            }
                        }
                    }
                }
                Err("Invalid authentication message".to_string())
            }
            Ok(None) => Err("Connection closed during authentication".to_string()),
            Err(_) => Err("Authentication timeout".to_string()),
        }
    }

    /// Validate authentication token.
    async fn validate_token(&self, token: &str) -> Result<Uuid, String> {
        // JWT validation would go here
        // For now, just parse as UUID for testing
        Uuid::parse_str(token).map_err(|_| "Invalid token".to_string())
    }

    /// Process incoming message.
    async fn process_message(
        &self,
        session_id: Uuid,
        text: &str,
    ) -> Result<Option<OutgoingMessage>, String> {
        let message: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| format!("Invalid message format: {}", e))?;

        debug!(
            session_id = %session_id,
            msg_type = %message.msg_type,
            "Processing WebSocket message"
        );

        // Update session activity
        self.session_manager.update_session(session_id, |s| s.touch()).await;

        // Route to message handler
        Ok(self.message_handler.handle_message(session_id, message).await)
    }

    /// Close a connection.
    async fn close_connection(&self, session_id: Uuid, reason: &str) {
        info!(session_id = %session_id, reason = reason, "Closing WebSocket connection");

        self.session_manager
            .send_to(session_id, WsOutgoingMessage::Close)
            .await;

        self.cleanup_connection(session_id).await;
    }

    /// Cleanup after connection closes.
    async fn cleanup_connection(&self, session_id: Uuid) {
        self.session_manager.unregister(session_id).await;
        self.pool.remove(session_id).await;

        info!(session_id = %session_id, "WebSocket connection cleaned up");
    }

    /// Get connection count.
    pub async fn connection_count(&self) -> usize {
        self.pool.size().await
    }

    /// Broadcast message to all connections.
    pub async fn broadcast(&self, message: OutgoingMessage) {
        self.session_manager
            .broadcast(WsOutgoingMessage::Text(message.to_json()))
            .await;
    }
}
```

### 2. Connection Pool (crates/tachikoma-server/src/websocket/connection/pool.rs)

```rust
//! WebSocket connection pool.

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Connection pool for tracking active WebSocket connections.
pub struct ConnectionPool {
    /// Maximum connections allowed.
    max_connections: usize,
    /// Active connection IDs.
    connections: RwLock<HashSet<Uuid>>,
    /// Connection counter.
    counter: AtomicUsize,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            max_connections,
            connections: RwLock::new(HashSet::new()),
            counter: AtomicUsize::new(0),
        }
    }

    /// Add a connection to the pool.
    pub async fn add(&self, id: Uuid) -> bool {
        let current = self.counter.load(Ordering::SeqCst);
        if current >= self.max_connections {
            return false;
        }

        let mut connections = self.connections.write().await;
        if connections.insert(id) {
            self.counter.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    /// Remove a connection from the pool.
    pub async fn remove(&self, id: Uuid) -> bool {
        let mut connections = self.connections.write().await;
        if connections.remove(&id) {
            self.counter.fetch_sub(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    /// Check if connection exists.
    pub async fn contains(&self, id: Uuid) -> bool {
        self.connections.read().await.contains(&id)
    }

    /// Get current pool size.
    pub async fn size(&self) -> usize {
        self.counter.load(Ordering::SeqCst)
    }

    /// Check if pool is full.
    pub fn is_full(&self) -> bool {
        self.counter.load(Ordering::SeqCst) >= self.max_connections
    }

    /// Get all connection IDs.
    pub async fn all(&self) -> Vec<Uuid> {
        self.connections.read().await.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new(2);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        assert!(pool.add(id1).await);
        assert!(pool.add(id2).await);
        assert!(!pool.add(id3).await); // Pool full

        assert_eq!(pool.size().await, 2);

        pool.remove(id1).await;
        assert!(pool.add(id3).await); // Now has room
    }
}
```

### 3. Message Router (crates/tachikoma-server/src/websocket/connection/router.rs)

```rust
//! WebSocket message routing.

use crate::websocket::messages::{
    Command, IncomingMessage, OutgoingMessage, AckResponse, ErrorResponse,
    PongResponse, SubscriptionListResponse,
};
use crate::websocket::session::SessionManager;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

/// Default message handler implementation.
pub struct DefaultMessageHandler {
    session_manager: Arc<SessionManager>,
}

impl DefaultMessageHandler {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    async fn handle_subscribe(&self, session_id: Uuid, topic: &str) -> OutgoingMessage {
        self.session_manager
            .update_session(session_id, |s| s.subscribe(topic))
            .await;

        OutgoingMessage::new("ack", AckResponse::success_with_message(
            format!("Subscribed to {}", topic)
        ))
    }

    async fn handle_unsubscribe(&self, session_id: Uuid, topic: &str) -> OutgoingMessage {
        self.session_manager
            .update_session(session_id, |s| s.unsubscribe(topic))
            .await;

        OutgoingMessage::new("ack", AckResponse::success_with_message(
            format!("Unsubscribed from {}", topic)
        ))
    }

    async fn handle_list_subscriptions(&self, session_id: Uuid) -> OutgoingMessage {
        if let Some(session) = self.session_manager.get_session(session_id).await {
            OutgoingMessage::new("subscriptions", SubscriptionListResponse {
                subscriptions: session.subscriptions,
            })
        } else {
            OutgoingMessage::new("error", ErrorResponse::not_found("Session"))
        }
    }

    async fn handle_ping(&self) -> OutgoingMessage {
        OutgoingMessage::new("pong", PongResponse::now())
    }
}

#[async_trait::async_trait]
impl super::manager::MessageHandler for DefaultMessageHandler {
    async fn handle_message(
        &self,
        session_id: Uuid,
        message: IncomingMessage,
    ) -> Option<OutgoingMessage> {
        let msg_type = message.msg_type.as_str();
        let payload = message.payload;

        debug!(session_id = %session_id, msg_type = msg_type, "Routing message");

        let response = match msg_type {
            "ping" => self.handle_ping().await,

            "subscribe" => {
                if let Some(topic) = payload.get("topic").and_then(|v| v.as_str()) {
                    self.handle_subscribe(session_id, topic).await
                } else {
                    OutgoingMessage::new("error", ErrorResponse::invalid_message("Missing topic"))
                }
            }

            "unsubscribe" => {
                if let Some(topic) = payload.get("topic").and_then(|v| v.as_str()) {
                    self.handle_unsubscribe(session_id, topic).await
                } else {
                    OutgoingMessage::new("error", ErrorResponse::invalid_message("Missing topic"))
                }
            }

            "list_subscriptions" => {
                self.handle_list_subscriptions(session_id).await
            }

            _ => {
                warn!(session_id = %session_id, msg_type = msg_type, "Unknown message type");
                OutgoingMessage::new("error", ErrorResponse::new(
                    "unknown_message",
                    format!("Unknown message type: {}", msg_type),
                ))
            }
        };

        // Set reply_to if original message had an ID
        let response = if let Some(msg_id) = message.id {
            OutgoingMessage {
                reply_to: Some(msg_id),
                ..response
            }
        } else {
            response
        };

        Some(response)
    }
}
```

### 4. Rate Limiter (crates/tachikoma-server/src/websocket/connection/rate_limit.rs)

```rust
//! WebSocket connection rate limiting.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Connection rate limiter.
pub struct ConnectionRateLimiter {
    /// Maximum connections per IP per window.
    max_connections: u32,
    /// Time window.
    window: Duration,
    /// Connection attempts by IP.
    attempts: RwLock<HashMap<IpAddr, Vec<Instant>>>,
}

impl ConnectionRateLimiter {
    pub fn new(max_connections: u32, window_secs: u64) -> Self {
        Self {
            max_connections,
            window: Duration::from_secs(window_secs),
            attempts: RwLock::new(HashMap::new()),
        }
    }

    /// Check if connection is allowed.
    pub async fn allow(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut attempts = self.attempts.write().await;

        let entry = attempts.entry(ip).or_insert_with(Vec::new);

        // Remove old attempts
        entry.retain(|t| now.duration_since(*t) < self.window);

        // Check limit
        if entry.len() >= self.max_connections as usize {
            false
        } else {
            entry.push(now);
            true
        }
    }

    /// Clean up old entries.
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let mut attempts = self.attempts.write().await;

        attempts.retain(|_, v| {
            v.retain(|t| now.duration_since(*t) < self.window);
            !v.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = ConnectionRateLimiter::new(2, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert!(limiter.allow(ip).await);
        assert!(limiter.allow(ip).await);
        assert!(!limiter.allow(ip).await); // Exceeded limit
    }
}
```

---

## Testing Requirements

1. Authentication flow works
2. Message routing correct
3. Connection pool limits enforced
4. Rate limiting works
5. Cleanup on disconnect
6. Broadcast reaches all clients
7. Backpressure handled

---

## Related Specs

- Depends on: [329-websocket-setup.md](329-websocket-setup.md), [330-ws-message-types.md](330-ws-message-types.md)
- Next: [332-server-config.md](332-server-config.md)
- Used by: Real-time features
