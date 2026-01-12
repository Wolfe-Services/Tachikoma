# 051a - Backend Core Crate Setup

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 051a
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Create the `tachikoma-backends-core` crate with Cargo.toml and library root. This provides the foundation for the backend abstraction layer.

---

## Acceptance Criteria

- [ ] `tachikoma-backends-core` crate created
- [ ] Cargo.toml with all dependencies
- [ ] Library root with module declarations

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-backends-core/Cargo.toml)

```toml
[package]
name = "tachikoma-backends-core"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core backend traits and types for Tachikoma LLM integration"

[dependencies]
tachikoma-common-core.workspace = true
async-trait = "0.1"
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
tokio = { workspace = true, features = ["sync"] }
futures = "0.3"
pin-project-lite = "0.2"

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
```

### 2. Library Root (src/lib.rs)

```rust
//! Tachikoma Backend Core
//!
//! This crate provides the core traits and types for LLM backend integration.
//! All backend implementations (Claude, Codex, Gemini, Ollama) implement the
//! `Backend` trait defined here.

#![warn(missing_docs)]

pub mod backend;
pub mod completion;
pub mod error;
pub mod message;
pub mod stream;
pub mod tool;

pub use backend::{Backend, BackendCapabilities, BackendExt, BackendInfo};
pub use completion::{CompletionRequest, CompletionResponse, FinishReason, ToolChoice, Usage};
pub use error::BackendError;
pub use message::{ContentPart, ImageSource, Message, MessageContent, Role};
pub use stream::{CollectingStream, CompletionChunk, CompletionStream, ToolCallDelta};
pub use tool::{ToolCall, ToolDefinition, ToolParameter, ToolResult};
```

---

## Testing Requirements

1. Crate compiles successfully
2. All modules are accessible

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [051b-backend-message-types.md](051b-backend-message-types.md)
