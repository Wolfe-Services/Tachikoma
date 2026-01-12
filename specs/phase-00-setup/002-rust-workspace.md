# 002 - Rust Workspace Setup

**Phase:** 0 - Setup
**Spec ID:** 002
**Status:** Planned
**Dependencies:** 001-project-structure
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Configure the Rust workspace with Cargo.toml, shared dependencies, and initial crate structure for modular development.

---

## Acceptance Criteria

- [x] Root `Cargo.toml` defines workspace
- [x] Workspace members pattern configured
- [x] Shared dependencies in `[workspace.dependencies]`
- [x] Common Rust version and edition set
- [x] Profile configurations for dev/release
- [x] Clippy and rustfmt configurations

---

## Implementation Details

### 1. Root Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/*",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Tachikoma Team"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/tachikoma"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging/Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# HTTP
reqwest = { version = "0.11", features = ["json", "stream"] }

# CLI
clap = { version = "4.4", features = ["derive", "env"] }

# Testing
proptest = "1.4"
insta = "1.34"

# NAPI for Electron binding
napi = "2.14"
napi-derive = "2.14"

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = "thin"
strip = true

[profile.release-debug]
inherits = "release"
debug = true
strip = false
```

### 2. rustfmt.toml

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
newline_style = "Unix"
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### 3. clippy.toml

```toml
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 8
type-complexity-threshold = 300
```

### 4. .cargo/config.toml

```toml
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[alias]
t = "test"
c = "check"
b = "build"
r = "run"
```

### 5. Create Initial Placeholder Crate

Create `crates/tachikoma-common-core/Cargo.toml`:

```toml
[package]
name = "tachikoma-common-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true

[dev-dependencies]
proptest.workspace = true
```

Create `crates/tachikoma-common-core/src/lib.rs`:

```rust
//! Tachikoma common core types and utilities.

pub mod error;
pub mod types;

pub use error::Error;
pub use types::*;
```

---

## Testing Requirements

1. `cargo check` succeeds
2. `cargo clippy` passes with no warnings
3. `cargo fmt --check` passes
4. Workspace structure validated

---

## Commands to Verify

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

---

## Related Specs

- Depends on: [001-project-structure.md](001-project-structure.md)
- Next: [003-electron-shell.md](003-electron-shell.md)
