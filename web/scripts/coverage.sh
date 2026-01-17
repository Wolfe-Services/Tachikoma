#!/usr/bin/env bash
set -euo pipefail

OUTPUT_DIR="${1:-coverage}"
mkdir -p "$OUTPUT_DIR"

echo "Running TypeScript tests with coverage..."

npm run test -- --coverage --run

echo "Coverage reports generated in $OUTPUT_DIR"