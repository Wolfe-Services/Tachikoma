#!/usr/bin/env bash
set -euo pipefail

# Source Rust environment if available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

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

# Step 2: Build web frontend
log "Building web frontend..."
cd web
npm run build
cd ..

# Step 3: Build Electron app
log "Building Electron app..."
cd electron
npm run build
cd ..

log "Build complete!"