# 074 - Backend Context Tracking

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 074
**Status:** Planned
**Dependencies:** 051-backend-trait, 073-backend-tokens
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement context window tracking for backends to monitor usage, detect approaching limits, and trigger warnings before hitting the "redline" threshold that requires conversation reset.

---

## Acceptance Criteria

- [ ] Context usage tracking per conversation
- [ ] Redline threshold detection
- [ ] Context compression suggestions
- [ ] Conversation reset handling
- [ ] Context visualization support

---

## Implementation Details

### 1. Context Types (src/context/types.rs)

```rust
//! Context tracking types.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Context window state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextState {
    /// Plenty of room.
    Nominal,
    /// Getting full (>50%).
    Elevated,
    /// Approaching limit (>75%).
    Warning,
    /// Near capacity (>85%).
    Critical,
    /// At or over limit.
    Redlined,
}

impl ContextState {
    /// Get state from usage percentage.
    pub fn from_percentage(pct: f32) -> Self {
        match pct {
            p if p >= 100.0 => Self::Redlined,
            p if p >= 85.0 => Self::Critical,
            p if p >= 75.0 => Self::Warning,
            p if p >= 50.0 => Self::Elevated,
            _ => Self::Nominal,
        }
    }

    /// Check if conversation should be reset.
    pub fn needs_reset(&self) -> bool {
        matches!(self, Self::Redlined)
    }

    /// Check if approaching limits.
    pub fn is_concerning(&self) -> bool {
        matches!(self, Self::Warning | Self::Critical | Self::Redlined)
    }
}

/// Context usage snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsage {
    /// Tokens currently in context.
    pub tokens_used: u32,
    /// Maximum context window size.
    pub max_tokens: u32,
    /// Current state.
    pub state: ContextState,
    /// Percentage used.
    pub usage_percent: f32,
    /// Tokens remaining.
    pub tokens_remaining: u32,
    /// Timestamp of measurement.
    #[serde(skip)]
    pub timestamp: Option<Instant>,
}

impl ContextUsage {
    /// Create new usage snapshot.
    pub fn new(tokens_used: u32, max_tokens: u32) -> Self {
        let usage_percent = (tokens_used as f32 / max_tokens as f32) * 100.0;
        Self {
            tokens_used,
            max_tokens,
            state: ContextState::from_percentage(usage_percent),
            usage_percent,
            tokens_remaining: max_tokens.saturating_sub(tokens_used),
            timestamp: Some(Instant::now()),
        }
    }

    /// Check if there's room for more tokens.
    pub fn has_room_for(&self, tokens: u32) -> bool {
        self.tokens_remaining >= tokens
    }

    /// Estimate tokens that can be added before warning.
    pub fn tokens_until_warning(&self) -> u32 {
        let warning_threshold = (self.max_tokens as f32 * 0.75) as u32;
        warning_threshold.saturating_sub(self.tokens_used)
    }

    /// Estimate tokens that can be added before redline.
    pub fn tokens_until_redline(&self) -> u32 {
        let redline_threshold = (self.max_tokens as f32 * 0.90) as u32;
        redline_threshold.saturating_sub(self.tokens_used)
    }
}

/// Configuration for context tracking.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Warning threshold (percentage).
    pub warning_threshold: f32,
    /// Critical threshold (percentage).
    pub critical_threshold: f32,
    /// Redline threshold (percentage).
    pub redline_threshold: f32,
    /// Auto-compress at this threshold.
    pub auto_compress_threshold: f32,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            warning_threshold: 75.0,
            critical_threshold: 85.0,
            redline_threshold: 90.0,
            auto_compress_threshold: 80.0,
        }
    }
}
```

### 2. Context Tracker (src/context/tracker.rs)

```rust
//! Context usage tracker.

use super::types::{ContextConfig, ContextState, ContextUsage};
use crate::tokens::TokenCounter;
use std::sync::Arc;
use tachikoma_backends_core::Message;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Tracks context window usage for a conversation.
pub struct ContextTracker {
    /// Maximum context size.
    max_tokens: u32,
    /// Current token count.
    current_tokens: u32,
    /// Messages in context.
    messages: Vec<MessageEntry>,
    /// Token counter.
    counter: Arc<dyn TokenCounter>,
    /// Configuration.
    config: ContextConfig,
    /// History of usage snapshots.
    history: Vec<ContextUsage>,
}

/// Entry for a message in context.
#[derive(Debug, Clone)]
struct MessageEntry {
    message: Message,
    tokens: u32,
    timestamp: std::time::Instant,
}

impl ContextTracker {
    /// Create a new context tracker.
    pub fn new(
        max_tokens: u32,
        counter: Arc<dyn TokenCounter>,
        config: ContextConfig,
    ) -> Self {
        Self {
            max_tokens,
            current_tokens: 0,
            messages: Vec::new(),
            counter,
            config,
            history: Vec::new(),
        }
    }

    /// Add a message to the context.
    pub fn add_message(&mut self, message: Message) {
        let tokens = self.counter.count_message(&message);
        self.current_tokens += tokens;

        self.messages.push(MessageEntry {
            message,
            tokens,
            timestamp: std::time::Instant::now(),
        });

        let usage = self.current_usage();
        self.history.push(usage.clone());

        debug!(
            tokens_added = tokens,
            total = self.current_tokens,
            state = ?usage.state,
            "Added message to context"
        );

        // Check thresholds
        if usage.state.is_concerning() {
            warn!(
                usage_percent = format!("{:.1}%", usage.usage_percent),
                state = ?usage.state,
                "Context usage is concerning"
            );
        }
    }

    /// Get current usage.
    pub fn current_usage(&self) -> ContextUsage {
        ContextUsage::new(self.current_tokens, self.max_tokens)
    }

    /// Get current state.
    pub fn state(&self) -> ContextState {
        self.current_usage().state
    }

    /// Check if at or over redline.
    pub fn is_redlined(&self) -> bool {
        self.current_usage().state.needs_reset()
    }

    /// Get all messages.
    pub fn messages(&self) -> Vec<&Message> {
        self.messages.iter().map(|e| &e.message).collect()
    }

    /// Get message count.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Clear context (reset).
    pub fn clear(&mut self) {
        info!(
            tokens_cleared = self.current_tokens,
            messages = self.messages.len(),
            "Clearing context"
        );
        self.messages.clear();
        self.current_tokens = 0;
    }

    /// Remove oldest messages until under threshold.
    pub fn compress_to_threshold(&mut self, target_percent: f32) {
        let target_tokens = (self.max_tokens as f32 * target_percent / 100.0) as u32;

        let mut removed_count = 0;
        while self.current_tokens > target_tokens && self.messages.len() > 1 {
            if let Some(entry) = self.messages.first() {
                // Don't remove system messages
                if entry.message.role != tachikoma_backends_core::Role::System {
                    let entry = self.messages.remove(0);
                    self.current_tokens -= entry.tokens;
                    removed_count += 1;
                } else if self.messages.len() > 1 {
                    // Skip system, remove next
                    let entry = self.messages.remove(1);
                    self.current_tokens -= entry.tokens;
                    removed_count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if removed_count > 0 {
            info!(
                messages_removed = removed_count,
                new_tokens = self.current_tokens,
                target = target_tokens,
                "Compressed context"
            );
        }
    }

    /// Get compression suggestions.
    pub fn compression_suggestions(&self) -> Vec<CompressionSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest removing old user messages
        let user_messages: Vec<_> = self.messages.iter()
            .enumerate()
            .filter(|(_, e)| e.message.role == tachikoma_backends_core::Role::User)
            .collect();

        if user_messages.len() > 10 {
            let old_count = user_messages.len() - 5;
            let old_tokens: u32 = user_messages[..old_count].iter().map(|(_, e)| e.tokens).sum();
            suggestions.push(CompressionSuggestion {
                description: format!("Remove {} oldest user messages", old_count),
                tokens_saved: old_tokens,
                impact: CompressionImpact::Low,
            });
        }

        // Suggest summarizing if many assistant messages
        let assistant_tokens: u32 = self.messages.iter()
            .filter(|e| e.message.role == tachikoma_backends_core::Role::Assistant)
            .map(|e| e.tokens)
            .sum();

        if assistant_tokens > self.max_tokens / 3 {
            suggestions.push(CompressionSuggestion {
                description: "Summarize assistant responses".to_string(),
                tokens_saved: assistant_tokens / 2,
                impact: CompressionImpact::Medium,
            });
        }

        suggestions
    }

    /// Get usage history.
    pub fn history(&self) -> &[ContextUsage] {
        &self.history
    }
}

/// Suggestion for context compression.
#[derive(Debug, Clone)]
pub struct CompressionSuggestion {
    pub description: String,
    pub tokens_saved: u32,
    pub impact: CompressionImpact,
}

/// Impact level of compression.
#[derive(Debug, Clone, Copy)]
pub enum CompressionImpact {
    /// Minimal impact on conversation quality.
    Low,
    /// Some context may be lost.
    Medium,
    /// Significant context loss.
    High,
}

/// Thread-safe context tracker.
pub struct SharedContextTracker {
    inner: Arc<RwLock<ContextTracker>>,
}

impl SharedContextTracker {
    /// Create a new shared tracker.
    pub fn new(
        max_tokens: u32,
        counter: Arc<dyn TokenCounter>,
        config: ContextConfig,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ContextTracker::new(max_tokens, counter, config))),
        }
    }

    /// Add a message.
    pub async fn add_message(&self, message: Message) {
        self.inner.write().await.add_message(message);
    }

    /// Get current usage.
    pub async fn current_usage(&self) -> ContextUsage {
        self.inner.read().await.current_usage()
    }

    /// Check if redlined.
    pub async fn is_redlined(&self) -> bool {
        self.inner.read().await.is_redlined()
    }

    /// Clear context.
    pub async fn clear(&self) {
        self.inner.write().await.clear();
    }

    /// Compress to threshold.
    pub async fn compress_to_threshold(&self, target_percent: f32) {
        self.inner.write().await.compress_to_threshold(target_percent);
    }
}

impl Clone for SharedContextTracker {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
```

### 3. Context Events (src/context/events.rs)

```rust
//! Context-related events.

use super::types::ContextState;
use tokio::sync::broadcast;

/// Event types for context changes.
#[derive(Debug, Clone)]
pub enum ContextEvent {
    /// State changed.
    StateChanged {
        old: ContextState,
        new: ContextState,
        usage_percent: f32,
    },
    /// Approaching redline.
    ApproachingRedline { usage_percent: f32 },
    /// Hit redline.
    Redlined { tokens_used: u32, max_tokens: u32 },
    /// Context cleared.
    Cleared { tokens_cleared: u32 },
    /// Context compressed.
    Compressed {
        tokens_before: u32,
        tokens_after: u32,
    },
}

/// Event emitter for context changes.
pub struct ContextEventEmitter {
    sender: broadcast::Sender<ContextEvent>,
}

impl ContextEventEmitter {
    /// Create a new emitter.
    pub fn new() -> (Self, broadcast::Receiver<ContextEvent>) {
        let (sender, receiver) = broadcast::channel(100);
        (Self { sender }, receiver)
    }

    /// Emit an event.
    pub fn emit(&self, event: ContextEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> broadcast::Receiver<ContextEvent> {
        self.sender.subscribe()
    }
}

impl Default for ContextEventEmitter {
    fn default() -> Self {
        Self::new().0
    }
}
```

### 4. Module Exports (src/context/mod.rs)

```rust
//! Context tracking for backends.

mod events;
mod tracker;
mod types;

pub use events::{ContextEvent, ContextEventEmitter};
pub use tracker::{
    CompressionImpact, CompressionSuggestion, ContextTracker, SharedContextTracker,
};
pub use types::{ContextConfig, ContextState, ContextUsage};
```

---

## Testing Requirements

1. Context tracking counts correctly
2. State transitions at thresholds
3. Compression removes old messages
4. Redline detection works
5. Events emit on state changes

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Depends on: [073-backend-tokens.md](073-backend-tokens.md)
- Next: [075-backend-tests.md](075-backend-tests.md)
