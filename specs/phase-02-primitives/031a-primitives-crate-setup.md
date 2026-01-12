# 031a - Primitives Crate Setup

**Phase:** 2 - Five Primitives
**Spec ID:** 031a
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Create the `tachikoma-primitives` crate structure with Cargo.toml, feature flags, and library root. This is the foundation for the five core primitives.

---

## Acceptance Criteria

- [ ] `tachikoma-primitives` crate created with Cargo.toml
- [ ] Feature flags for optional primitives configured
- [ ] Library root (lib.rs) with module declarations

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-primitives/Cargo.toml)

```toml
[package]
name = "tachikoma-primitives"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core primitives for Tachikoma agent operations"

[features]
default = ["all"]
all = ["read-file", "list-files", "bash", "edit-file", "code-search"]
read-file = []
list-files = ["dep:walkdir"]
bash = ["dep:tokio"]
edit-file = []
code-search = ["dep:serde_json"]

[dependencies]
tachikoma-common-core.workspace = true

async-trait = "0.1"
serde = { workspace = true, features = ["derive"] }
thiserror.workspace = true
tracing.workspace = true
uuid = { version = "1.6", features = ["v4"] }

# Optional dependencies
walkdir = { version = "2.4", optional = true }
tokio = { workspace = true, features = ["process", "time"], optional = true }
serde_json = { workspace = true, optional = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
tempfile = "3.9"
```

### 2. Library Root (src/lib.rs)

```rust
//! Tachikoma primitives - core agent operations.
//!
//! This crate provides the five primitives that form the foundation
//! of Tachikoma's agent capabilities:
//!
//! - `read_file` - Read file contents
//! - `list_files` - List directory contents
//! - `bash` - Execute shell commands
//! - `edit_file` - Search and replace in files
//! - `code_search` - Search code with ripgrep

#![warn(missing_docs)]

pub mod context;
pub mod error;
pub mod result;

// Re-exports
pub use context::{PrimitiveConfig, PrimitiveContext};
pub use error::{PrimitiveError, PrimitiveResult};
```

---

## Testing Requirements

1. Crate compiles with default features
2. Crate compiles with individual features enabled
3. Feature flags correctly gate dependencies

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [031b-primitives-context.md](031b-primitives-context.md)
