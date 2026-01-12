# 136a - Forge Types Crate Setup

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 136a
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Create the `tachikoma-forge-types` crate structure with Cargo.toml and library root for the Forge multi-model brainstorming system.

---

## Acceptance Criteria

- [ ] `tachikoma-forge-types` crate created
- [ ] Cargo.toml with dependencies
- [ ] Library root with module declarations

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-forge-types/Cargo.toml)

```toml
[package]
name = "tachikoma-forge-types"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core types for Tachikoma Spec Forge"

[dependencies]
tachikoma-common-core.workspace = true
serde = { workspace = true, features = ["derive"] }
thiserror.workspace = true
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
serde_json.workspace = true
```

### 2. Library Root (src/lib.rs)

```rust
//! Tachikoma Forge types.
//!
//! Core types for the Spec Forge multi-model brainstorming system.

#![warn(missing_docs)]

pub mod participant;
pub mod response;
pub mod round;
pub mod session;

pub use participant::*;
pub use response::*;
pub use round::*;
pub use session::*;
```

---

## Testing Requirements

1. Crate compiles successfully
2. All modules are accessible

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [136b-forge-session-types.md](136b-forge-session-types.md)
