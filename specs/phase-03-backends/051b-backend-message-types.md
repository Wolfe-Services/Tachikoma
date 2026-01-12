# 051b - Backend Message Types

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 051b
**Status:** Planned
**Dependencies:** 051a-backend-crate-setup
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define message types for LLM communication including roles, content types, and helper methods.

---

## Acceptance Criteria

- [ ] `Role` enum (System, User, Assistant, Tool)
- [ ] `Message` struct with content and metadata
- [ ] `MessageContent` for text and multi-part content
- [ ] Helper methods for message creation

---

## Implementation Details

### 1. Message Types (src/message.rs)

```rust
//! Message types for LLM communication.

use serde::{Deserialize, Serialize};

/// Role of a message participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt/instructions.
    System,
    /// User input.
    User,
    /// Assistant response.
    Assistant,
    /// Tool/function result.
    Tool,
}

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the sender.
    pub role: Role,
    /// Message content.
    pub content: MessageContent,
    /// Optional name (for tool results).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tool call ID (for tool results).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: None,
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: None,
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: None,
        }
    }

    /// Create a tool result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content.
    Text(String),
    /// Multi-part content (text + images).
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    /// Get text content if available.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Parts(parts) => parts.iter().find_map(|p| {
                if let ContentPart::Text { text } = p {
                    Some(text.as_str())
                } else {
                    None
                }
            }),
        }
    }

    /// Convert to string, concatenating all text parts.
    pub fn to_text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Parts(parts) => parts
                .iter()
                .filter_map(|p| {
                    if let ContentPart::Text { text } = p {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

/// A part of multi-part content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content.
    #[serde(rename = "text")]
    Text { text: String },
    /// Image content.
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

/// Source of an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    /// Base64-encoded image.
    #[serde(rename = "base64")]
    Base64 { media_type: String, data: String },
    /// URL to image.
    #[serde(rename = "url")]
    Url { url: String },
}
```

---

## Testing Requirements

1. Message creation helpers work correctly
2. Content serialization/deserialization works
3. Text extraction from multi-part content works

---

## Related Specs

- Depends on: [051a-backend-crate-setup.md](051a-backend-crate-setup.md)
- Next: [051c-backend-completion-types.md](051c-backend-completion-types.md)
