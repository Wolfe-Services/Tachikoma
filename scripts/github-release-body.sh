#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required}"
REPO_URL="https://github.com/tachikoma/tachikoma"

# Extract highlights from release notes if they exist
HIGHLIGHTS=""
RELEASE_NOTES_FILE="docs/release-notes/v${VERSION}.md"

if [[ -f "$RELEASE_NOTES_FILE" ]]; then
  # Extract the highlights section
  HIGHLIGHTS=$(sed -n '/## Highlights/,/## What/p' "$RELEASE_NOTES_FILE" | grep -v "^##" | sed '/^$/d' | head -3 || true)
fi

# Generate release body for GitHub
cat << EOF
## Tachikoma v${VERSION}

${HIGHLIGHTS:-"New release of Tachikoma with improvements and bug fixes."}

See the [full release notes](${REPO_URL}/blob/main/docs/release-notes/v${VERSION}.md) for complete details.

### Quick Links
- 📖 [Documentation](https://docs.tachikoma.dev)
- 📋 [Changelog](${REPO_URL}/blob/main/CHANGELOG.md)
- 🐛 [Report Issues](${REPO_URL}/issues)
- 💬 [Discussions](${REPO_URL}/discussions)

### Installation

Download the appropriate package for your platform below, or use one of these methods:

#### Package Managers
\`\`\`bash
# macOS (Homebrew) - Coming soon
# brew install --cask tachikoma

# Linux (Snap) - Coming soon  
# snap install tachikoma

# Windows (Chocolatey) - Coming soon
# choco install tachikoma
\`\`\`

#### Manual Installation
1. Download the appropriate package for your platform below
2. Run the installer
3. Launch Tachikoma from your applications folder or start menu

### What's New
EOF

# Add brief feature list if release notes exist
if [[ -f "$RELEASE_NOTES_FILE" ]]; then
  echo ""
  echo "Key highlights in this release:"
  echo ""
  # Extract feature titles from release notes
  sed -n '/## What'\''s New/,/## Improvements/p' "$RELEASE_NOTES_FILE" | grep "^###" | sed 's/### /- /' || true
fi

echo ""
echo "---"
echo ""
echo "*For detailed information about changes, breaking changes, migration guides, and known issues, please see the [full release notes](${REPO_URL}/blob/main/docs/release-notes/v${VERSION}.md).*"