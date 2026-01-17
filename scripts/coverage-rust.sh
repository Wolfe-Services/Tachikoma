#!/usr/bin/env bash
set -euo pipefail

# Colors
GREEN='\033[0;32m'
NC='\033[0m'
log() { echo -e "${GREEN}[COVERAGE]${NC} $1"; }

# Ensure cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    log "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

OUTPUT_DIR="${1:-coverage/rust}"
mkdir -p "$OUTPUT_DIR"

log "Running Rust tests with coverage..."

# Generate LCOV report
cargo llvm-cov --workspace \
    --lcov \
    --output-path "$OUTPUT_DIR/lcov.info" \
    --ignore-filename-regex '(tests/|test_|_test\.rs)'

# Generate HTML report
cargo llvm-cov --workspace \
    --html \
    --output-dir "$OUTPUT_DIR/html" \
    --ignore-filename-regex '(tests/|test_|_test\.rs)'

# Generate JSON for processing
cargo llvm-cov --workspace \
    --json \
    --output-path "$OUTPUT_DIR/coverage.json" \
    --ignore-filename-regex '(tests/|test_|_test\.rs)'

log "Coverage reports generated in $OUTPUT_DIR"

# Print summary
cargo llvm-cov --workspace report --summary-only