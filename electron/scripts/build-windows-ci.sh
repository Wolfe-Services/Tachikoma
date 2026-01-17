#!/usr/bin/env bash
set -euo pipefail

echo "Building Windows installer (cross-compile)..."

# Install Wine for cross-compilation (if on Linux)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Installing Wine..."
    sudo dpkg --add-architecture i386
    sudo apt-get update
    sudo apt-get install -y wine64 wine32
fi

# Ensure we're in the electron directory
cd "$(dirname "$0")/.."

# Build TypeScript
echo "Compiling TypeScript..."
npm run build

# Build Windows targets
echo "Creating Windows packages..."
npx electron-builder --win

echo "Windows build complete!"