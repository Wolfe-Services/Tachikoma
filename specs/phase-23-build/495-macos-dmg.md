# 495 - macOS DMG Creation

**Phase:** 23 - Build & Distribution
**Spec ID:** 495
**Status:** Planned
**Dependencies:** 494-electron-packaging
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure macOS DMG disk image creation with custom background, icon positioning, and branding for professional distribution.

---

## Acceptance Criteria

- [x] DMG created with custom background image
- [x] App icon and Applications folder properly positioned
- [x] Both x64 and arm64 architectures supported
- [x] Universal binary option available
- [x] DMG is code signed
- [x] Volume icon customized

---

## Implementation Details

### 1. DMG Configuration

Update `electron/electron-builder.config.js` DMG section:

```javascript
// DMG configuration
dmg: {
  artifactName: '${productName}-${version}-${arch}.${ext}',

  // Window configuration
  window: {
    width: 660,
    height: 400,
  },

  // Background
  background: 'build/dmg-background.png',
  backgroundColor: '#1a1a2e',

  // Icon configuration
  icon: 'build/icon.icns',
  iconSize: 128,
  iconTextSize: 14,

  // Contents positioning
  contents: [
    {
      x: 180,
      y: 170,
      type: 'file',
    },
    {
      x: 480,
      y: 170,
      type: 'link',
      path: '/Applications',
    },
  ],

  // Code signing (DMG itself)
  sign: true,

  // Volume title
  title: '${productName} ${version}',

  // Format
  format: 'ULFO', // ULFO = LZFSE compression (fast, good ratio)

  // Internet-enable (allows Safari to auto-open)
  internetEnabled: true,

  // Write update info for Sparkle
  writeUpdateInfo: false,
},
```

### 2. DMG Background Design

Create `electron/build/dmg-background.png` (660x400 pixels):

```
Design specifications:
- Dimensions: 660x400 pixels
- Background color: #1a1a2e (dark theme)
- Left side: App icon drop zone indicator
- Right side: Applications folder indicator
- Arrow or visual guide between them
- Tachikoma branding/logo at top
- "Drag to install" instruction text
```

### 3. DMG Build Script

Create `electron/scripts/build-dmg.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ARCH="${1:-$(uname -m)}"
VERSION=$(node -p "require('./package.json').version")

echo "Building DMG for macOS ($ARCH)..."

# Map architecture names
case "$ARCH" in
    x86_64|x64)
        ELECTRON_ARCH="x64"
        ;;
    arm64|aarch64)
        ELECTRON_ARCH="arm64"
        ;;
    universal)
        ELECTRON_ARCH="universal"
        ;;
    *)
        echo "Unknown architecture: $ARCH"
        exit 1
        ;;
esac

# Build DMG
if [ "$ELECTRON_ARCH" = "universal" ]; then
    # Build universal binary
    npx electron-builder --mac --universal
else
    npx electron-builder --mac dmg --arch "$ELECTRON_ARCH"
fi

# Verify output
DMG_FILE="out/Tachikoma-${VERSION}-${ELECTRON_ARCH}.dmg"
if [ -f "$DMG_FILE" ]; then
    echo "DMG created: $DMG_FILE"
    ls -lh "$DMG_FILE"

    # Verify DMG
    echo "Verifying DMG..."
    hdiutil verify "$DMG_FILE"

    echo "DMG contents:"
    hdiutil attach "$DMG_FILE" -mountpoint /tmp/dmg-verify
    ls -la /tmp/dmg-verify/
    hdiutil detach /tmp/dmg-verify
else
    echo "Error: DMG not found at $DMG_FILE"
    exit 1
fi

echo "DMG build complete!"
```

### 4. Universal Binary Support

Create `electron/scripts/build-universal.sh`:

```bash
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
X64_APP="out/mac-x64/${APP_NAME}.app"
ARM64_APP="out/mac-arm64/${APP_NAME}.app"
UNIVERSAL_APP="out/mac-universal/${APP_NAME}.app"

# Create output directory
mkdir -p "out/mac-universal"

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
```

### 5. DMG Verification Script

Create `electron/scripts/verify-dmg.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

DMG_FILE="${1:?DMG file path required}"

echo "Verifying DMG: $DMG_FILE"

# Check file exists
if [ ! -f "$DMG_FILE" ]; then
    echo "Error: DMG file not found"
    exit 1
fi

# Verify DMG integrity
echo "Checking DMG integrity..."
hdiutil verify "$DMG_FILE"

# Mount and inspect
MOUNT_POINT=$(mktemp -d)
echo "Mounting DMG to $MOUNT_POINT..."
hdiutil attach "$DMG_FILE" -mountpoint "$MOUNT_POINT" -nobrowse

# Check app exists
APP_PATH="$MOUNT_POINT/Tachikoma.app"
if [ ! -d "$APP_PATH" ]; then
    echo "Error: App not found in DMG"
    hdiutil detach "$MOUNT_POINT"
    exit 1
fi

# Check code signature
echo "Checking code signature..."
codesign --verify --deep --strict "$APP_PATH"
if [ $? -eq 0 ]; then
    echo "Code signature: Valid"
else
    echo "Warning: Code signature invalid or missing"
fi

# Check architecture
echo "Checking architecture..."
MAIN_BINARY="$APP_PATH/Contents/MacOS/Tachikoma"
file "$MAIN_BINARY"
lipo -info "$MAIN_BINARY"

# Check entitlements
echo "Checking entitlements..."
codesign -d --entitlements - "$APP_PATH" 2>/dev/null || echo "No entitlements found"

# Check notarization
echo "Checking notarization..."
spctl --assess --type execute "$APP_PATH" 2>&1 || echo "Not notarized or Gatekeeper disabled"

# Cleanup
hdiutil detach "$MOUNT_POINT"
rmdir "$MOUNT_POINT"

echo "DMG verification complete!"
```

---

## Testing Requirements

1. DMG opens with correct window size and layout
2. App icon and Applications folder are properly positioned
3. Drag-to-install works correctly
4. Both x64 and arm64 DMGs are valid
5. Universal binary runs on both architectures

---

## Related Specs

- Depends on: [494-electron-packaging.md](494-electron-packaging.md)
- Next: [496-macos-signing.md](496-macos-signing.md)
- Related: [497-macos-notarize.md](497-macos-notarize.md)
