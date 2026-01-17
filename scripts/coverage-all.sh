#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
NC='\033[0m'
log() { echo -e "${GREEN}[COVERAGE]${NC} $1"; }

OUTPUT_DIR="coverage"
mkdir -p "$OUTPUT_DIR"

# Run Rust coverage
log "Collecting Rust coverage..."
./scripts/coverage-rust.sh "$OUTPUT_DIR/rust"

# Run TypeScript coverage
log "Collecting TypeScript coverage..."
cd web && npm run test -- --coverage --run
cd ..
mv web/coverage "$OUTPUT_DIR/typescript"

# Merge LCOV reports
log "Merging coverage reports..."
if command -v lcov &> /dev/null; then
    lcov \
        -a "$OUTPUT_DIR/rust/lcov.info" \
        -a "$OUTPUT_DIR/typescript/lcov.info" \
        -o "$OUTPUT_DIR/merged.info"
else
    log "lcov not installed, skipping merge"
fi

# Generate summary
log "Coverage Summary"
echo "================"
echo ""
echo "Rust Coverage:"
cat "$OUTPUT_DIR/rust/coverage.json" | jq '.data[0].totals'
echo ""
echo "TypeScript Coverage:"
cat "$OUTPUT_DIR/typescript/coverage-summary.json" | jq '.total'

log "Reports available at:"
echo "  - Rust HTML: $OUTPUT_DIR/rust/html/index.html"
echo "  - TypeScript HTML: $OUTPUT_DIR/typescript/index.html"