# 404 - Feature Flag Synchronization

## Overview

Real-time synchronization of feature flag changes across distributed systems using Server-Sent Events (SSE) and WebSocket connections.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/sync.rs

use crate::definition::FlagDefinition;
use crate::storage::FlagStorage;
use crate::types::FlagId;
use axum::{
    extract::{Query, State},
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// Event types for flag synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FlagEvent {
    /// Flag was created
    FlagCreated(FlagDefinition),
    /// Flag was updated
    FlagUpdated(FlagDefinition),
    /// Flag was deleted
    FlagDeleted { flag_id: String },
    /// Flag status changed
    FlagStatusChanged {
        flag_id: String,
        old_status: String,
        new_status: String,
    },
    /// Initial sync complete
    SyncComplete { flag_count: usize },
    /// Heartbeat to keep connection alive
    Heartbeat { timestamp: i64 },
}

/// Flag synchronization hub
pub struct FlagSyncHub {
    /// Broadcast channel for flag events
    sender: broadcast::Sender<FlagEvent>,
    /// Storage backend
    storage: Arc<dyn FlagStorage>,
    /// Connected clients count
    client_count: Arc<std::sync::atomic::AtomicU64>,
}

impl FlagSyncHub {
    pub fn new(storage: Arc<dyn FlagStorage>) -> Self {
        let (sender, _) = broadcast::channel(1000);

        Self {
            sender,
            storage,
            client_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Broadcast a flag event to all connected clients
    pub fn broadcast(&self, event: FlagEvent) {
        // Ignore errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to flag events
    pub fn subscribe(&self) -> broadcast::Receiver<FlagEvent> {
        self.client_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.sender.subscribe()
    }

    /// Unsubscribe (decrement counter)
    pub fn unsubscribe(&self) {
        self.client_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get current client count
    pub fn client_count(&self) -> u64 {
        self.client_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Start heartbeat task
    pub fn start_heartbeat(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                self.broadcast(FlagEvent::Heartbeat {
                    timestamp: chrono::Utc::now().timestamp(),
                });
            }
        });
    }

    /// Notify flag created
    pub fn notify_created(&self, flag: FlagDefinition) {
        self.broadcast(FlagEvent::FlagCreated(flag));
    }

    /// Notify flag updated
    pub fn notify_updated(&self, flag: FlagDefinition) {
        self.broadcast(FlagEvent::FlagUpdated(flag));
    }

    /// Notify flag deleted
    pub fn notify_deleted(&self, flag_id: &FlagId) {
        self.broadcast(FlagEvent::FlagDeleted {
            flag_id: flag_id.as_str().to_string(),
        });
    }

    /// Notify flag status changed
    pub fn notify_status_changed(&self, flag_id: &FlagId, old_status: &str, new_status: &str) {
        self.broadcast(FlagEvent::FlagStatusChanged {
            flag_id: flag_id.as_str().to_string(),
            old_status: old_status.to_string(),
            new_status: new_status.to_string(),
        });
    }
}

/// SSE stream parameters
#[derive(Debug, Deserialize)]
pub struct StreamParams {
    /// SDK key for authentication
    key: String,
    /// Environment filter
    env: Option<String>,
    /// Last event ID for resumption
    last_event_id: Option<String>,
}

/// Create SSE endpoint handler
pub async fn sse_handler(
    State(hub): State<Arc<FlagSyncHub>>,
    Query(params): Query<StreamParams>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    // Validate SDK key (simplified)
    // In production, validate against database

    let receiver = hub.subscribe();
    let hub_clone = hub.clone();

    let stream = BroadcastStream::new(receiver)
        .filter_map(move |result| {
            match result {
                Ok(event) => {
                    let json = serde_json::to_string(&event).ok()?;
                    Some(Ok(Event::default().data(json)))
                }
                Err(_) => None,
            }
        });

    // Send initial sync
    let storage = hub.storage.clone();
    tokio::spawn(async move {
        if let Ok(flags) = storage.list(Default::default()).await {
            for stored in &flags {
                hub_clone.broadcast(FlagEvent::FlagUpdated(stored.definition.clone()));
            }
            hub_clone.broadcast(FlagEvent::SyncComplete {
                flag_count: flags.len(),
            });
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}

/// WebSocket-based synchronization
pub mod websocket {
    use super::*;
    use axum::{
        extract::ws::{Message, WebSocket, WebSocketUpgrade},
        response::IntoResponse,
    };
    use futures::{SinkExt, StreamExt};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum WsMessage {
        /// Subscribe to flag updates
        Subscribe { flags: Option<Vec<String>> },
        /// Unsubscribe from updates
        Unsubscribe,
        /// Request specific flag
        GetFlag { flag_id: String },
        /// Request all flags
        GetAllFlags,
        /// Ping for keepalive
        Ping,
        /// Pong response
        Pong,
    }

    pub async fn ws_handler(
        ws: WebSocketUpgrade,
        State(hub): State<Arc<FlagSyncHub>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| handle_socket(socket, hub))
    }

    async fn handle_socket(socket: WebSocket, hub: Arc<FlagSyncHub>) {
        let (mut sender, mut receiver) = socket.split();
        let mut event_receiver = hub.subscribe();

        // Spawn task to forward flag events to WebSocket
        let send_task = tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                if let Ok(json) = serde_json::to_string(&event) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Handle incoming messages
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => {
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                        match msg {
                            WsMessage::Ping => {
                                // Pong is handled by the framework
                            }
                            WsMessage::GetAllFlags => {
                                if let Ok(flags) = hub.storage.list(Default::default()).await {
                                    for stored in flags {
                                        hub.broadcast(FlagEvent::FlagUpdated(stored.definition));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        send_task.abort();
        hub.unsubscribe();
    }
}

/// Polling-based sync for environments without SSE/WebSocket
pub mod polling {
    use super::*;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Serialize)]
    pub struct PollResponse {
        pub flags: Vec<FlagDefinition>,
        pub last_modified: DateTime<Utc>,
        pub etag: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct PollParams {
        /// Last known etag
        pub if_none_match: Option<String>,
        /// Last modified timestamp
        pub if_modified_since: Option<DateTime<Utc>>,
    }

    pub async fn poll_handler(
        State(storage): State<Arc<dyn FlagStorage>>,
        Query(params): Query<PollParams>,
    ) -> Result<axum::Json<PollResponse>, axum::http::StatusCode> {
        let flags = storage.list(Default::default()).await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

        // Check if modified since
        if let Some(since) = params.if_modified_since {
            let any_modified = flags.iter()
                .any(|f| f.definition.metadata.updated_at > since);

            if !any_modified {
                return Err(axum::http::StatusCode::NOT_MODIFIED);
            }
        }

        let last_modified = flags.iter()
            .map(|f| f.definition.metadata.updated_at)
            .max()
            .unwrap_or_else(Utc::now);

        let etag = format!("\"{}\"", last_modified.timestamp());

        // Check etag
        if params.if_none_match.as_ref() == Some(&etag) {
            return Err(axum::http::StatusCode::NOT_MODIFIED);
        }

        Ok(axum::Json(PollResponse {
            flags: flags.into_iter().map(|f| f.definition).collect(),
            last_modified,
            etag,
        }))
    }
}

/// Create sync router
pub fn sync_router(hub: Arc<FlagSyncHub>) -> Router {
    Router::new()
        .route("/stream", get(sse_handler))
        .route("/ws", get(websocket::ws_handler))
        .route("/poll", get(polling::poll_handler))
        .with_state(hub)
}

/// Client-side sync manager
pub struct SyncClient {
    api_url: String,
    sdk_key: String,
    on_update: Box<dyn Fn(FlagEvent) + Send + Sync>,
}

impl SyncClient {
    pub fn new(
        api_url: &str,
        sdk_key: &str,
        on_update: impl Fn(FlagEvent) + Send + Sync + 'static,
    ) -> Self {
        Self {
            api_url: api_url.to_string(),
            sdk_key: sdk_key.to_string(),
            on_update: Box::new(on_update),
        }
    }

    /// Start SSE connection
    pub async fn start_sse(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/sdk/stream?key={}", self.api_url, self.sdk_key);

        let client = reqwest::Client::new();
        let mut response = client
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await?;

        while let Some(chunk) = response.chunk().await? {
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(event) = serde_json::from_str::<FlagEvent>(data) {
                        (self.on_update)(event);
                    }
                }
            }
        }

        Ok(())
    }

    /// Fallback to polling
    pub async fn poll(&self) -> Result<Vec<FlagDefinition>, Box<dyn std::error::Error>> {
        let url = format!("{}/sdk/poll?key={}", self.api_url, self.sdk_key);

        let response: polling::PollResponse = reqwest::get(&url).await?.json().await?;

        Ok(response.flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;

    #[tokio::test]
    async fn test_sync_hub_broadcast() {
        let storage = Arc::new(InMemoryStorage::new());
        let hub = Arc::new(FlagSyncHub::new(storage));

        let mut receiver = hub.subscribe();

        hub.notify_deleted(&FlagId::new("test-flag"));

        let event = receiver.recv().await.unwrap();
        assert!(matches!(event, FlagEvent::FlagDeleted { .. }));
    }

    #[tokio::test]
    async fn test_client_count() {
        let storage = Arc::new(InMemoryStorage::new());
        let hub = Arc::new(FlagSyncHub::new(storage));

        assert_eq!(hub.client_count(), 0);

        let _receiver1 = hub.subscribe();
        assert_eq!(hub.client_count(), 1);

        let _receiver2 = hub.subscribe();
        assert_eq!(hub.client_count(), 2);

        hub.unsubscribe();
        assert_eq!(hub.client_count(), 1);
    }
}
```

## Client Configuration

```typescript
// TypeScript client reconnection logic
class FlagSyncClient {
  private eventSource: EventSource | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 1000;

  connect(url: string): void {
    this.eventSource = new EventSource(url);

    this.eventSource.onopen = () => {
      this.reconnectAttempts = 0;
    };

    this.eventSource.onerror = () => {
      this.reconnect(url);
    };

    this.eventSource.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.handleEvent(data);
    };
  }

  private reconnect(url: string): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      this.fallbackToPolling();
      return;
    }

    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts);
    this.reconnectAttempts++;

    setTimeout(() => this.connect(url), delay);
  }

  private fallbackToPolling(): void {
    // Start polling every 30 seconds
    setInterval(() => this.poll(), 30000);
  }
}
```

## Related Specs

- 402-flag-sdk-rust.md - Rust SDK
- 403-flag-sdk-ts.md - TypeScript SDK
- 405-flag-caching.md - Caching layer
