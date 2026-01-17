#!/usr/bin/env bash
set -euo pipefail

echo "Building universal macOS binary..."

# Build for both architectures
echo "Building x64..."
npx electron-builder --mac --x64 --dir

echo "Building arm64..."
npx electron-builder --mac --arm64 --dir

# Create universal binary using lipo
echo "Creating universal binary..."

APP_NAME="Tachikoma"
X64_APP="release/mac-x64/${APP_NAME}.app"
ARM64_APP="release/mac-arm64/${APP_NAME}.app"
UNIVERSAL_APP="release/mac-universal/${APP_NAME}.app"

# Create output directory
mkdir -p "release/mac-universal"

# Copy base app structure from arm64
cp -R "$ARM64_APP" "$UNIVERSAL_APP"

# Find all Mach-O binaries and merge them
find "$UNIVERSAL_APP" -type f -exec file {} \; | grep "Mach-O" | cut -d: -f1 | while read -r binary; do
    relative_path="${binary#$UNIVERSAL_APP/}"
    x64_binary="$X64_APP/$relative_path"
    arm64_binary="$ARM64_APP/$relative_path"

    if [ -f "$x64_binary" ] && [ -f "$arm64_binary" ]; then
        echo "Merging: $relative_path"
        lipo -create "$x64_binary" "$arm64_binary" -output "$binary"
    fi
done

# Build universal DMG
echo "Building universal DMG..."
npx electron-builder --mac --universal --prepackaged "$UNIVERSAL_APP"

echo "Universal binary build complete!"