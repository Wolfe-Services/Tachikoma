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
    ci)
        BUILD_ARGS="$BUILD_ARGS --profile ci"
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