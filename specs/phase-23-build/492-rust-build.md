# 492 - Rust Build Configuration

**Phase:** 23 - Build & Distribution
**Spec ID:** 492
**Status:** Planned
**Dependencies:** 491-build-overview, 002-rust-workspace
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Configure optimized Rust build settings for development, release, and distribution builds, including profile configuration, feature flags, and cross-compilation support.

---

## Acceptance Criteria

- [ ] Development builds compile quickly with debug info
- [ ] Release builds are fully optimized
- [ ] LTO and codegen optimizations configured
- [ ] Cross-compilation targets supported
- [ ] Build caching maximized
- [ ] Binary size optimized for distribution

---

## Implementation Details

### 1. Workspace Cargo Configuration

Update `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/tachikoma-common-core",
    "crates/tachikoma-primitives",
    "crates/tachikoma-backends",
    "crates/tachikoma-loop",
    "crates/tachikoma-cli",
    "crates/tachikoma-server",
    "crates/tachikoma-native",
    "crates/tachikoma-test-harness",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"
repository = "https://github.com/tachikoma/tachikoma"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# HTTP
reqwest = { version = "0.11", features = ["json", "stream"] }

# Testing
proptest = "1.4"
insta = { version = "1.34", features = ["yaml", "json"] }

# Build profiles
[profile.dev]
opt-level = 0
debug = true
debug-assertions = true
overflow-checks = true
lto = false
panic = "unwind"
incremental = true
codegen-units = 256

[profile.dev.package."*"]
opt-level = 2  # Optimize dependencies even in dev

[profile.release]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
lto = "thin"
panic = "abort"
incremental = false
codegen-units = 16
strip = true

# Distribution profile (maximum optimization)
[profile.dist]
inherits = "release"
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"

# Profile for benchmarks
[profile.bench]
inherits = "release"
debug = true  # For profiling
lto = "thin"

# Profile for CI tests (balance speed and optimization)
[profile.ci]
inherits = "dev"
opt-level = 1
debug = 1
```

### 2. Build Script Configuration

Create `crates/tachikoma-common-core/build.rs`:

```rust
//! Build script for embedding version and build information.

use std::process::Command;

fn main() {
    // Re-run if build script or Cargo.toml changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Git branch
    let git_branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);

    // Build timestamp
    let build_time = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    // Target triple
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);

    // Profile
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

    // Detect CI environment
    let is_ci = std::env::var("CI").is_ok();
    println!("cargo:rustc-env=BUILD_CI={}", is_ci);
}
```

### 3. Feature Flags Configuration

Update `crates/tachikoma-common-core/Cargo.toml`:

```toml
[package]
name = "tachikoma-common-core"
version.workspace = true
edition.workspace = true

[features]
default = []

# Development features
dev = ["debug-logging", "hot-reload"]
debug-logging = []
hot-reload = []

# Testing features
test-utils = ["proptest", "insta"]
proptest = ["dep:proptest"]
mocks = []

# Backend features
claude = []
openai = []
gemini = []
ollama = []
all-backends = ["claude", "openai", "gemini", "ollama"]

# Platform features
server = ["axum", "tower"]
native = ["napi", "napi-derive"]

# Optimization features
simd = []
mimalloc = ["dep:mimalloc"]

[dependencies]
# Core dependencies
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true

# Optional dependencies
proptest = { workspace = true, optional = true }
insta = { workspace = true, optional = true }
axum = { version = "0.7", optional = true }
tower = { version = "0.4", optional = true }
napi = { version = "2", optional = true }
napi-derive = { version = "2", optional = true }
mimalloc = { version = "0.1", optional = true }

[build-dependencies]
chrono = "0.4"
```

### 4. Cross-Compilation Configuration

Create `.cargo/config.toml`:

```toml
[build]
# Use sccache if available
rustc-wrapper = "sccache"

# Default target for development
# target = "x86_64-unknown-linux-gnu"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
# Use system linker

[target.aarch64-apple-darwin]
# Use system linker

[target.x86_64-pc-windows-msvc]
# Use MSVC linker

# Cross-compilation targets
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[alias]
# Custom commands
b = "build"
br = "build --release"
bd = "build --profile dist"
t = "test"
tr = "test --release"
c = "clippy"
```

### 5. Rust Build Script

Create `scripts/build-rust.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[RUST]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

# Configuration
PROFILE="${1:-release}"
TARGET="${2:-}"

log "Building Rust workspace"
log "  Profile: $PROFILE"
log "  Target: ${TARGET:-native}"

# Build arguments
BUILD_ARGS="--workspace"

case "$PROFILE" in
    dev|debug)
        BUILD_ARGS="$BUILD_ARGS"
        ;;
    release)
        BUILD_ARGS="$BUILD_ARGS --release"
        ;;
    dist)
        BUILD_ARGS="$BUILD_ARGS --profile dist"
        ;;
    *)
        warn "Unknown profile: $PROFILE, using release"
        BUILD_ARGS="$BUILD_ARGS --release"
        ;;
esac

if [ -n "$TARGET" ]; then
    BUILD_ARGS="$BUILD_ARGS --target $TARGET"
fi

# Check for sccache
if command -v sccache &> /dev/null; then
    export RUSTC_WRAPPER=sccache
    log "Using sccache for compilation"
fi

# Build
log "Running: cargo build $BUILD_ARGS"
cargo build $BUILD_ARGS

# Report binary sizes for release builds
if [ "$PROFILE" = "release" ] || [ "$PROFILE" = "dist" ]; then
    log "Binary sizes:"
    if [ -n "$TARGET" ]; then
        BINARY_DIR="target/$TARGET/release"
    else
        BINARY_DIR="target/release"
    fi

    for binary in tachikoma tachikoma-server; do
        if [ -f "$BINARY_DIR/$binary" ]; then
            SIZE=$(du -h "$BINARY_DIR/$binary" | cut -f1)
            log "  $binary: $SIZE"
        fi
    done
fi

log "Rust build complete!"
```

### 6. Binary Size Optimization

Create `scripts/optimize-binary.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

BINARY="${1:?Binary path required}"

echo "Optimizing binary: $BINARY"

# Strip debug symbols (if not already stripped)
if command -v strip &> /dev/null; then
    strip "$BINARY" 2>/dev/null || true
fi

# UPX compression (optional, can affect startup time)
if command -v upx &> /dev/null && [ "${USE_UPX:-false}" = "true" ]; then
    upx --best --lzma "$BINARY"
fi

# Report final size
SIZE=$(du -h "$BINARY" | cut -f1)
echo "Final size: $SIZE"
```

---

## Testing Requirements

1. Development builds compile in under 30 seconds (incremental)
2. Release builds produce optimized binaries
3. Cross-compilation targets build successfully
4. Binary sizes are reasonable for distribution
5. Build info is correctly embedded

---

## Related Specs

- Depends on: [491-build-overview.md](491-build-overview.md), [002-rust-workspace.md](../phase-00-setup/002-rust-workspace.md)
- Next: [493-ts-build.md](493-ts-build.md)
- Related: [174-napi-rs-setup.md](../phase-08-electron/174-napi-rs-setup.md)
