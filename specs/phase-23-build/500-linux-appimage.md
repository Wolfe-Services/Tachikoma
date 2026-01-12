# 500 - Linux AppImage

**Phase:** 23 - Build & Distribution
**Spec ID:** 500
**Status:** Planned
**Dependencies:** 494-electron-packaging
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure Linux AppImage packaging for portable, distribution-agnostic deployment that runs on any Linux distribution without installation.

---

## Acceptance Criteria

- [ ] AppImage created with all dependencies bundled
- [ ] Desktop integration (icons, file associations)
- [ ] FUSE-less operation supported
- [ ] Auto-update integration
- [ ] AppImageHub metadata included
- [ ] Sandbox compatibility

---

## Implementation Details

### 1. AppImage Configuration

Update `electron/electron-builder.config.js`:

```javascript
// Linux configuration
linux: {
  icon: 'build/icons',
  category: 'Development',
  executableName: 'tachikoma',
  desktop: {
    Name: 'Tachikoma',
    GenericName: 'AI Development Environment',
    Comment: 'Autonomous AI-powered development tool',
    Type: 'Application',
    Categories: 'Development;IDE;',
    Keywords: 'ai;development;coding;llm;',
    StartupNotify: true,
    StartupWMClass: 'tachikoma',
    MimeType: 'application/x-tachikoma-spec;x-scheme-handler/tachikoma;',
  },
  target: [
    {
      target: 'AppImage',
      arch: ['x64', 'arm64'],
    },
  ],
  fileAssociations: [
    {
      ext: 'tspec',
      name: 'Tachikoma Spec',
      mimeType: 'application/x-tachikoma-spec',
    },
  ],
},

// AppImage specific
appImage: {
  artifactName: '${productName}-${version}-${arch}.${ext}',

  // Include desktop integration
  desktop: {
    entry: {
      Name: 'Tachikoma',
      Exec: 'tachikoma %U',
      Icon: 'tachikoma',
      Type: 'Application',
      Categories: 'Development;',
      MimeType: 'x-scheme-handler/tachikoma;application/x-tachikoma-spec;',
    },
  },

  // License file
  license: '../LICENSE',

  // Synopsis for AppImageHub
  synopsis: 'AI-powered autonomous development environment',

  // Category for AppImageHub
  category: 'Development',
},
```

### 2. AppStream Metadata

Create `electron/build/tachikoma.appdata.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.tachikoma.app</id>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <name>Tachikoma</name>
  <summary>AI-powered autonomous development environment</summary>
  <description>
    <p>
      Tachikoma is an AI-powered development environment that autonomously
      executes development tasks using state-of-the-art language models.
    </p>
    <p>Features include:</p>
    <ul>
      <li>Multi-model support (Claude, GPT, Gemini, Ollama)</li>
      <li>Spec-driven development workflow</li>
      <li>Autonomous loop runner for unattended operation</li>
      <li>Spec Forge for multi-model brainstorming</li>
    </ul>
  </description>
  <launchable type="desktop-id">tachikoma.desktop</launchable>
  <url type="homepage">https://github.com/tachikoma/tachikoma</url>
  <url type="bugtracker">https://github.com/tachikoma/tachikoma/issues</url>
  <screenshots>
    <screenshot type="default">
      <caption>Mission Panel</caption>
      <image>https://tachikoma.dev/screenshots/mission-panel.png</image>
    </screenshot>
    <screenshot>
      <caption>Spec Browser</caption>
      <image>https://tachikoma.dev/screenshots/spec-browser.png</image>
    </screenshot>
  </screenshots>
  <content_rating type="oars-1.1" />
  <releases>
    <release version="0.1.0" date="2024-01-15">
      <description>
        <p>Initial release of Tachikoma.</p>
      </description>
    </release>
  </releases>
  <developer_name>Tachikoma Team</developer_name>
  <keywords>
    <keyword>ai</keyword>
    <keyword>development</keyword>
    <keyword>llm</keyword>
    <keyword>coding</keyword>
    <keyword>automation</keyword>
  </keywords>
</component>
```

### 3. Build Script

Create `electron/scripts/build-appimage.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ARCH="${1:-x64}"
VERSION=$(node -p "require('./package.json').version")

echo "Building AppImage for Linux ($ARCH)..."

# Install dependencies if needed
if ! command -v appimagetool &> /dev/null; then
    echo "Installing appimagetool..."
    wget -q https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O /tmp/appimagetool
    chmod +x /tmp/appimagetool
    sudo mv /tmp/appimagetool /usr/local/bin/appimagetool
fi

# Build using electron-builder
npx electron-builder --linux AppImage --arch "$ARCH"

# Verify AppImage
APPIMAGE_FILE="out/Tachikoma-${VERSION}-${ARCH}.AppImage"

if [ -f "$APPIMAGE_FILE" ]; then
    echo "AppImage created: $APPIMAGE_FILE"
    ls -lh "$APPIMAGE_FILE"

    # Make executable
    chmod +x "$APPIMAGE_FILE"

    # Verify it runs (basic check)
    echo "Verifying AppImage..."
    "$APPIMAGE_FILE" --version || true

    echo "AppImage build complete!"
else
    echo "Error: AppImage not found at $APPIMAGE_FILE"
    exit 1
fi
```

### 4. Desktop Entry Integration

Create `electron/build/tachikoma.desktop`:

```ini
[Desktop Entry]
Name=Tachikoma
GenericName=AI Development Environment
Comment=AI-powered autonomous development tool
Exec=tachikoma %U
Icon=tachikoma
Type=Application
Categories=Development;IDE;
Keywords=ai;development;coding;llm;automation;
StartupNotify=true
StartupWMClass=tachikoma
MimeType=application/x-tachikoma-spec;x-scheme-handler/tachikoma;
Actions=new-window;

[Desktop Action new-window]
Name=New Window
Exec=tachikoma --new-window
```

### 5. MIME Type Registration

Create `electron/build/tachikoma-mime.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-tachikoma-spec">
    <comment>Tachikoma Specification</comment>
    <glob pattern="*.tspec"/>
    <icon name="application-x-tachikoma-spec"/>
  </mime-type>
</mime-info>
```

### 6. AppImage Update Information

For delta updates, add update information:

```javascript
// In electron-builder.config.js
appImage: {
  // ... other config

  // Update info for AppImageUpdate
  // Format: zsync|<URL>
  // updateInfo: 'zsync|https://releases.tachikoma.dev/Tachikoma-latest-x86_64.AppImage.zsync',
},

// Or use GitHub releases
publish: {
  provider: 'github',
  owner: 'tachikoma',
  repo: 'tachikoma',
},
```

### 7. Extract and Run Script

For systems without FUSE, provide extraction script:

Create `electron/scripts/run-extracted.sh`:

```bash
#!/usr/bin/env bash
# Run Tachikoma without FUSE (extract and execute)

APPIMAGE="${1:-Tachikoma.AppImage}"
EXTRACT_DIR="${HOME}/.local/share/tachikoma-extracted"

if [ ! -f "$APPIMAGE" ]; then
    echo "AppImage not found: $APPIMAGE"
    exit 1
fi

# Extract if not already extracted or if AppImage is newer
if [ ! -d "$EXTRACT_DIR" ] || [ "$APPIMAGE" -nt "$EXTRACT_DIR" ]; then
    echo "Extracting AppImage..."
    rm -rf "$EXTRACT_DIR"
    chmod +x "$APPIMAGE"
    "$APPIMAGE" --appimage-extract
    mv squashfs-root "$EXTRACT_DIR"
fi

# Run the extracted app
exec "$EXTRACT_DIR/AppRun" "$@"
```

---

## Testing Requirements

1. AppImage runs on Ubuntu, Fedora, Arch
2. Desktop integration works correctly
3. File associations open .tspec files
4. Protocol handler works
5. Update mechanism functions

---

## Related Specs

- Depends on: [494-electron-packaging.md](494-electron-packaging.md)
- Next: [501-linux-deb.md](501-linux-deb.md)
- Related: [179-linux-build.md](../phase-08-electron/179-linux-build.md)
