# Spec 581: NAPI-RS Forge Bindings

**Priority:** P0  
**Status:** planned  
**Estimated Effort:** 4 hours  
**Target Files:**
- `crates/tachikoma-napi/Cargo.toml` (new crate)
- `crates/tachikoma-napi/src/lib.rs` (new)
- `crates/tachikoma-napi/src/forge.rs` (new)
- `electron/main/native.ts` (update to use real bindings)
- `Cargo.toml` (add workspace member)

---

## Overview

Create NAPI-RS bindings to expose `tachikoma-forge` functionality to Electron's main process. This replaces the placeholder implementations in `native.ts` with real Rust calls.

---

## Acceptance Criteria

- [ ] Create new crate `crates/tachikoma-napi` with napi-rs dependencies
- [ ] Add crate to workspace in root `Cargo.toml`
- [ ] Export `create_forge_session` function via NAPI
- [ ] Export `start_deliberation` function that returns a stream handle
- [ ] Export `stop_deliberation` function
- [ ] Export `get_session` and `list_sessions` functions
- [ ] Build produces `.node` file in `electron/` directory
- [ ] Update `electron/main/native.ts` to import and use the `.node` module
- [ ] Verify `npm run build` in electron directory succeeds

---

## Implementation

```toml
# crates/tachikoma-napi/Cargo.toml
[package]
name = "tachikoma-napi"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "2", features = ["async", "napi4"] }
napi-derive = "2"
tachikoma-forge = { path = "../tachikoma-forge" }
tokio = { version = "1", features = ["rt-multi-thread"] }
serde_json = "1"

[build-dependencies]
napi-build = "2"
```

```rust
// crates/tachikoma-napi/src/lib.rs
#![deny(clippy::all)]

use napi_derive::napi;

mod forge;
pub use forge::*;
```

```rust
// crates/tachikoma-napi/src/forge.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tachikoma_forge::{ForgeSession, ForgeSessionBuilder};

#[napi(object)]
pub struct JsForgeSession {
  pub id: String,
  pub name: String,
  pub goal: String,
  pub phase: String,
}

#[napi]
pub fn create_forge_session(name: String, goal: String) -> Result<JsForgeSession> {
  let session = ForgeSessionBuilder::new(name.clone(), goal.clone())
    .build()
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  
  Ok(JsForgeSession {
    id: session.id.to_string(),
    name: session.name,
    goal: session.goal,
    phase: "configuring".to_string(),
  })
}
```
