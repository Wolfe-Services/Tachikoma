# 007 - Build System Configuration

**Phase:** 0 - Setup
**Spec ID:** 007
**Status:** Planned
**Dependencies:** 002-rust-workspace, 003-electron-shell, 004-svelte-integration
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Configure the complete build pipeline for development and production builds across Rust, Electron, and Svelte.

---

## Acceptance Criteria

- [x] Development builds fast and incremental
- [x] Production builds optimized
- [x] Cross-platform build scripts
- [x] Rust native modules built correctly
- [x] Asset bundling configured
- [x] Environment variable handling

---

## Implementation Details

### 1. Build Script (scripts/build.sh)

```bash
#!/usr/bin/env bash
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[BUILD]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Determine platform
case "$(uname -s)" in
    Darwin*) PLATFORM="darwin" ;;
    Linux*)  PLATFORM="linux" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="win32" ;;
    *) error "Unsupported platform" ;;
esac

# Build mode
MODE="${1:-release}"
log "Building Tachikoma for $PLATFORM ($MODE)"

# Step 1: Build Rust workspace
log "Building Rust workspace..."
if [ "$MODE" = "release" ]; then
    cargo build --release --workspace
else
    cargo build --workspace
fi

# Step 2: Build native module for Electron
log "Building native module..."
cd crates/tachikoma-native
npm run build
cd ../..

# Step 3: Build web frontend
log "Building web frontend..."
cd web
npm run build
cd ..

# Step 4: Build Electron app
log "Building Electron app..."
cd electron
npm run build
cd ..

log "Build complete!"
```

### 2. Makefile (Alternative)

```makefile
.PHONY: all dev build clean test lint

# Default target
all: build

# Development
dev:
	npm run dev

# Build all
build: build-rust build-web build-electron

build-rust:
	cargo build --release --workspace

build-web:
	cd web && npm run build

build-electron:
	cd electron && npm run build

# Clean all
clean:
	cargo clean
	rm -rf web/dist
	rm -rf electron/dist
	rm -rf electron/out

# Test all
test: test-rust test-web

test-rust:
	cargo test --workspace

test-web:
	cd web && npm test

# Lint all
lint: lint-rust lint-web

lint-rust:
	cargo clippy --workspace -- -D warnings
	cargo fmt --all -- --check

lint-web:
	cd web && npm run lint
	cd web && npm run check

# Package for distribution
package:
	cd electron && npm run package

# Install dependencies
install:
	npm install
	cd web && npm install
	cd electron && npm install
```

### 3. Environment Configuration

Create `.env.example`:

```bash
# Tachikoma Environment Configuration

# API Keys (required for backends)
ANTHROPIC_API_KEY=
OPENAI_API_KEY=
GOOGLE_API_KEY=

# Development
NODE_ENV=development
VITE_DEV_SERVER_URL=http://localhost:5173

# Build
RUST_LOG=info
RUST_BACKTRACE=1

# Electron
ELECTRON_ENABLE_LOGGING=1
```

### 4. Build Constants (web/src/lib/constants.ts)

```typescript
export const BUILD_INFO = {
  version: __APP_VERSION__,
  commit: __GIT_COMMIT__,
  buildTime: __BUILD_TIME__,
  platform: __PLATFORM__,
  isDev: import.meta.env.DEV
} as const;

// Declare build-time constants
declare const __APP_VERSION__: string;
declare const __GIT_COMMIT__: string;
declare const __BUILD_TIME__: string;
declare const __PLATFORM__: string;
```

### 5. Vite Build Plugin (web/vite.config.ts addition)

```typescript
import { execSync } from 'child_process';

function getBuildInfo() {
  const version = process.env.npm_package_version ?? '0.0.0';
  let commit = 'unknown';
  try {
    commit = execSync('git rev-parse --short HEAD').toString().trim();
  } catch {}

  return {
    __APP_VERSION__: JSON.stringify(version),
    __GIT_COMMIT__: JSON.stringify(commit),
    __BUILD_TIME__: JSON.stringify(new Date().toISOString()),
    __PLATFORM__: JSON.stringify(process.platform)
  };
}

export default defineConfig({
  // ... existing config
  define: getBuildInfo()
});
```

### 6. Cargo Build Script (build.rs template)

For crates that need build-time generation:

```rust
// crates/tachikoma-common-core/build.rs
fn main() {
    // Re-run if build script changes
    println!("cargo:rerun-if-changed=build.rs");

    // Embed git info
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());
        }
    }

    // Build timestamp
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    println!("cargo:rustc-env=BUILD_TIME={}", now);
}
```

---

## Testing Requirements

1. `make build` completes without errors
2. `make test` runs all test suites
3. `make lint` passes all checks
4. Production build size is reasonable

---

## Related Specs

- Depends on: [002-rust-workspace.md](002-rust-workspace.md), [003-electron-shell.md](003-electron-shell.md), [004-svelte-integration.md](004-svelte-integration.md)
- Next: [008-test-infrastructure.md](008-test-infrastructure.md)
