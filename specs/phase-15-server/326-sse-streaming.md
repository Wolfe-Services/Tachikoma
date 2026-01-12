# Spec 326: SSE Streaming Alternative

## Phase
15 - Server/API Layer

## Spec ID
326

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 313: Route Definitions

## Estimated Context
~9%

---

## Objective

Implement Server-Sent Events (SSE) as an alternative streaming mechanism to WebSocket, providing simpler one-way streaming for LLM responses that works better through proxies and firewalls.

---

## Acceptance Criteria

- [ ] SSE endpoint streams LLM responses
- [ ] Proper event formatting with event types
- [ ] Reconnection support with Last-Event-ID
- [ ] Heartbeat events for connection health
- [ ] Multiple concurrent streams supported
- [ ] Graceful connection cleanup
- [ ] Compatible with EventSource API

---

## Implementation Details

### SSE Types

```rust
// src/server/sse/types.rs
use serde::Serialize;
use uuid::Uuid;

/// SSE event types
#[derive(Debug, Clone, Copy)]
pub enum SseEventType {
    Token,
    Delta,
    Progress,
    FileChange,
    Complete,
    Error,
    Heartbeat,
}

impl SseEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SseEventType::Token => "token",
            SseEventType::Delta => "delta",
            SseEventType::Progress => "progress",
            SseEventType::FileChange => "file_change",
            SseEventType::Complete => "complete",
            SseEventType::Error => "error",
            SseEventType::Heartbeat => "heartbeat",
        }
    }
}

/// SSE event wrapper
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub id: Option<String>,
    pub event_type: SseEventType,
    pub data: String,
    pub retry: Option<u32>,
}

impl SseEvent {
    pub fn new(event_type: SseEventType, data: impl Serialize) -> Self {
        Self {
            id: Some(Uuid::new_v4().to_string()),
            event_type,
            data: serde_json::to_string(&data).unwrap_or_default(),
            retry: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_retry(mut self, retry_ms: u32) -> Self {
        self.retry = Some(retry_ms);
        self
    }

    /// Format as SSE message
    pub fn to_sse_string(&self) -> String {
        let mut result = String::new();

        if let Some(ref id) = self.id {
            result.push_str(&format!("id: {}\n", id));
        }

        result.push_str(&format!("event: {}\n", self.event_type.as_str()));

        // Handle multi-line data
        for line in self.data.lines() {
            result.push_str(&format!("data: {}\n", line));
        }

        if let Some(retry) = self.retry {
            result.push_str(&format!("retry: {}\n", retry));
        }

        result.push('\n'); // End of event
        result
    }
}

/// Token event data
#[derive(Debug, Clone, Serialize)]
pub struct TokenEventData {
    pub execution_id: Uuid,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Delta event data
#[derive(Debug, Clone, Serialize)]
pub struct DeltaEventData {
    pub execution_id: Uuid,
    pub delta_type: String,
    pub content: serde_json::Value,
}

/// Progress event data
#[derive(Debug, Clone, Serialize)]
pub struct ProgressEventData {
    pub execution_id: Uuid,
    pub progress: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Complete event data
#[derive(Debug, Clone, Serialize)]
pub struct CompleteEventData {
    pub execution_id: Uuid,
    pub spec_id: Uuid,
    pub message_id: Uuid,
    pub tokens_used: TokenUsageData,
    pub file_changes: Vec<FileChangeSummaryData>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsageData {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileChangeSummaryData {
    pub change_id: Uuid,
    pub file_path: String,
    pub change_type: String,
}

/// Error event data
#[derive(Debug, Clone, Serialize)]
pub struct ErrorEventData {
    pub execution_id: Option<Uuid>,
    pub code: String,
    pub message: String,
}

/// Heartbeat event data
#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatEventData {
    pub timestamp: i64,
}
```

### SSE Stream Handler

```rust
// src/server/sse/handler.rs
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{sse::{Event, KeepAlive, Sse}, IntoResponse},
};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use uuid::Uuid;

use super::types::*;
use crate::server::state::AppState;
use crate::server::error::ApiError;
use crate::backend::StreamEvent;

#[derive(Debug, Deserialize)]
pub struct StreamParams {
    /// Backend ID to use
    pub backend_id: Option<Uuid>,
    /// Model to use
    pub model: Option<String>,
    /// Temperature
    pub temperature: Option<f32>,
    /// Max tokens
    pub max_tokens: Option<u32>,
    /// Resume from event ID
    pub last_event_id: Option<String>,
}

/// SSE stream handler for spec execution
pub async fn stream_execution(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Query(params): Query<StreamParams>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate spec exists
    let storage = state.storage();
    let spec = storage.specs().get(spec_id).await?;

    // Create execution
    let execution_id = Uuid::new_v4();

    // Get backend
    let backend = if let Some(backend_id) = params.backend_id {
        state.backend_manager().get(backend_id)
    } else {
        state.backend_manager().get_default()
    }.ok_or_else(|| ApiError::bad_request("No backend available"))?;

    // Build prompt
    let messages = storage.messages().list_for_spec(spec_id).await?;
    let prompt_messages = build_prompt_messages(&spec, &messages);

    // Create stream options
    let options = StreamOptions {
        backend_id: params.backend_id,
        model: params.model,
        temperature: params.temperature,
        max_tokens: params.max_tokens,
        ..Default::default()
    };

    // Create the event stream
    let stream = create_execution_stream(
        state.clone(),
        execution_id,
        spec_id,
        backend,
        prompt_messages,
        options,
    );

    // Return SSE response
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("heartbeat")
    ))
}

fn create_execution_stream(
    state: AppState,
    execution_id: Uuid,
    spec_id: Uuid,
    backend: Arc<dyn Backend>,
    messages: Vec<PromptMessage>,
    options: StreamOptions,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let (tx, rx) = mpsc::channel::<SseEvent>(100);

    // Spawn execution task
    tokio::spawn(async move {
        if let Err(e) = run_sse_execution(
            state,
            execution_id,
            spec_id,
            backend,
            messages,
            options,
            tx.clone(),
        ).await {
            let error_event = SseEvent::new(
                SseEventType::Error,
                ErrorEventData {
                    execution_id: Some(execution_id),
                    code: "EXECUTION_FAILED".to_string(),
                    message: e.to_string(),
                },
            );
            let _ = tx.send(error_event).await;
        }
    });

    // Convert to SSE events
    tokio_stream::wrappers::ReceiverStream::new(rx).map(|sse_event| {
        let event = Event::default()
            .event(sse_event.event_type.as_str())
            .data(sse_event.data);

        let event = if let Some(id) = sse_event.id {
            event.id(id)
        } else {
            event
        };

        let event = if let Some(retry) = sse_event.retry {
            event.retry(Duration::from_millis(retry as u64))
        } else {
            event
        };

        Ok(event)
    })
}

async fn run_sse_execution(
    state: AppState,
    execution_id: Uuid,
    spec_id: Uuid,
    backend: Arc<dyn Backend>,
    messages: Vec<PromptMessage>,
    options: StreamOptions,
    tx: mpsc::Sender<SseEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let storage = state.storage();

    let mut accumulated_content = String::new();
    let mut prompt_tokens = 0u32;
    let mut completion_tokens = 0u32;
    let started_at = std::time::Instant::now();

    // Create the stream
    let mut stream = backend.stream(&messages, &options).await?;

    // Process events
    while let Some(event) = stream.next().await {
        match event {
            Ok(StreamEvent::Token(token)) => {
                completion_tokens += 1;
                accumulated_content.push_str(&token);

                let sse_event = SseEvent::new(
                    SseEventType::Token,
                    TokenEventData {
                        execution_id,
                        token,
                        finish_reason: None,
                    },
                );

                if tx.send(sse_event).await.is_err() {
                    // Client disconnected
                    break;
                }
            }
            Ok(StreamEvent::ContentDelta(delta)) => {
                let delta_data = match delta {
                    ContentDelta::Text { content } => {
                        accumulated_content.push_str(&content);
                        DeltaEventData {
                            execution_id,
                            delta_type: "text".to_string(),
                            content: serde_json::json!({ "text": content }),
                        }
                    }
                    ContentDelta::ToolCall { id, name, arguments } => {
                        DeltaEventData {
                            execution_id,
                            delta_type: "tool_call".to_string(),
                            content: serde_json::json!({
                                "id": id,
                                "name": name,
                                "arguments": arguments,
                            }),
                        }
                    }
                    _ => continue,
                };

                let sse_event = SseEvent::new(SseEventType::Delta, delta_data);
                if tx.send(sse_event).await.is_err() {
                    break;
                }
            }
            Ok(StreamEvent::Usage { prompt, completion }) => {
                prompt_tokens = prompt;
                completion_tokens = completion;
            }
            Ok(StreamEvent::Done { finish_reason }) => {
                // Final token event
                let sse_event = SseEvent::new(
                    SseEventType::Token,
                    TokenEventData {
                        execution_id,
                        token: String::new(),
                        finish_reason: Some(finish_reason),
                    },
                );
                let _ = tx.send(sse_event).await;
                break;
            }
            Err(e) => {
                let sse_event = SseEvent::new(
                    SseEventType::Error,
                    ErrorEventData {
                        execution_id: Some(execution_id),
                        code: "STREAM_ERROR".to_string(),
                        message: e.to_string(),
                    },
                );
                let _ = tx.send(sse_event).await;
                return Err(e.into());
            }
        }
    }

    // Save assistant message
    let assistant_message = Message {
        id: Uuid::new_v4(),
        spec_id,
        role: MessageRole::Assistant,
        content: accumulated_content.clone(),
        tokens: Some(completion_tokens as i32),
        model: Some(options.model.unwrap_or_else(|| backend.default_model().to_string())),
        created_at: Utc::now(),
    };
    let saved_message = storage.messages().create(assistant_message).await?;

    // Process file changes (simplified - full implementation in streaming service)
    let file_changes = Vec::new();

    // Send completion event
    let duration = started_at.elapsed();
    let complete_event = SseEvent::new(
        SseEventType::Complete,
        CompleteEventData {
            execution_id,
            spec_id,
            message_id: saved_message.id,
            tokens_used: TokenUsageData {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            file_changes,
            duration_ms: duration.as_millis() as u64,
        },
    );
    let _ = tx.send(complete_event).await;

    Ok(())
}

/// SSE stream for spec updates (subscribing to changes)
pub async fn stream_spec_updates(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate spec exists
    let storage = state.storage();
    storage.specs().get(spec_id).await?;

    // Create update stream
    let stream = create_spec_update_stream(state, spec_id);

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat")
    ))
}

fn create_spec_update_stream(
    state: AppState,
    spec_id: Uuid,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let (tx, rx) = mpsc::channel::<SseEvent>(50);

    // Spawn update listener
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));

        loop {
            interval.tick().await;

            // Send heartbeat
            let heartbeat = SseEvent::new(
                SseEventType::Heartbeat,
                HeartbeatEventData {
                    timestamp: Utc::now().timestamp(),
                },
            );

            if tx.send(heartbeat).await.is_err() {
                // Client disconnected
                break;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx).map(|sse_event| {
        Ok(Event::default()
            .event(sse_event.event_type.as_str())
            .data(sse_event.data))
    })
}
```

### Routes

```rust
// src/server/routes/sse.rs
use axum::{
    Router,
    routing::get,
};

use crate::server::state::AppState;
use crate::server::sse::handler;

pub fn sse_routes() -> Router<AppState> {
    Router::new()
        // Stream execution for a spec
        .route("/specs/:spec_id/stream", get(handler::stream_execution))
        // Subscribe to spec updates
        .route("/specs/:spec_id/updates", get(handler::stream_spec_updates))
        // Stream mission updates
        .route("/missions/:mission_id/updates", get(handler::stream_mission_updates))
}
```

### Client Usage Example

```javascript
// JavaScript EventSource client example
const eventSource = new EventSource('/api/v1/sse/specs/{specId}/stream?backend_id=xxx');

eventSource.addEventListener('token', (event) => {
    const data = JSON.parse(event.data);
    console.log('Token:', data.token);
    // Append token to output
});

eventSource.addEventListener('delta', (event) => {
    const data = JSON.parse(event.data);
    console.log('Delta:', data.delta_type, data.content);
});

eventSource.addEventListener('file_change', (event) => {
    const data = JSON.parse(event.data);
    console.log('File change:', data.file_path, data.change_type);
});

eventSource.addEventListener('complete', (event) => {
    const data = JSON.parse(event.data);
    console.log('Complete:', data);
    eventSource.close();
});

eventSource.addEventListener('error', (event) => {
    const data = JSON.parse(event.data);
    console.error('Error:', data.message);
    eventSource.close();
});

// Handle connection errors
eventSource.onerror = (error) => {
    console.error('Connection error:', error);
    // EventSource will automatically attempt to reconnect
};
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_formatting() {
        let event = SseEvent::new(
            SseEventType::Token,
            TokenEventData {
                execution_id: Uuid::nil(),
                token: "Hello".to_string(),
                finish_reason: None,
            },
        ).with_id("event-1");

        let formatted = event.to_sse_string();

        assert!(formatted.contains("id: event-1"));
        assert!(formatted.contains("event: token"));
        assert!(formatted.contains("data: "));
        assert!(formatted.ends_with("\n\n"));
    }

    #[test]
    fn test_multiline_data() {
        let event = SseEvent {
            id: None,
            event_type: SseEventType::Delta,
            data: "line1\nline2\nline3".to_string(),
            retry: None,
        };

        let formatted = event.to_sse_string();

        assert!(formatted.contains("data: line1"));
        assert!(formatted.contains("data: line2"));
        assert!(formatted.contains("data: line3"));
    }

    #[tokio::test]
    async fn test_stream_creation() {
        let state = create_test_state().await;
        let spec_id = create_test_spec(&state).await;

        // Test that stream can be created
        let stream = create_spec_update_stream(state, spec_id);

        // Collect first event (should be heartbeat)
        let first = stream.take(1).collect::<Vec<_>>().await;
        assert!(!first.is_empty());
    }
}
```

---

## Related Specs

- **Spec 323**: WebSocket Setup
- **Spec 325**: WebSocket Streaming
- **Spec 318**: Specs API
