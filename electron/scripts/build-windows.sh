#!/usr/bin/env bash
set -euo pipefail

echo "Building Windows installer..."

# Ensure we're in the electron directory
cd "$(dirname "$0")/.."

# Build TypeScript
echo "Compiling TypeScript..."
npm run build

# Create NSIS installer
echo "Creating NSIS installer..."
npx electron-builder --win nsis

# List output
echo "Build complete! Artifacts:"
ls -la release/*/

echo "Done!"