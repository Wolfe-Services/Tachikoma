# 507 - Release Notes

**Phase:** 23 - Build & Distribution
**Spec ID:** 507
**Status:** Planned
**Dependencies:** 506-changelog
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Create user-friendly release notes for each version that highlight key features, breaking changes, and upgrade instructions in an accessible format.

---

## Acceptance Criteria

- [ ] Release notes template defined
- [ ] Key features highlighted prominently
- [ ] Breaking changes have migration guides
- [ ] Known issues documented
- [ ] Download links included
- [ ] Notes published with each release

---

## Implementation Details

### 1. Release Notes Template

Create `docs/release-notes/template.md`:

```markdown
# Tachikoma v{VERSION} Release Notes

**Release Date:** {DATE}
**Type:** {Major|Minor|Patch} Release

## Highlights

Brief summary of the most exciting features in this release (2-3 sentences).

## What's New

### Feature Name
Description of the feature and why it matters to users.

![Screenshot if applicable](./images/feature.png)

### Another Feature
...

## Improvements

- Improvement 1 with brief description
- Improvement 2 with brief description
- Performance improvement details

## Bug Fixes

- Fixed issue where [description] (#123)
- Fixed crash when [description] (#124)

## Breaking Changes

### Change Title
**What changed:** Description of the change.

**Why:** Reason for the change.

**Migration:** Steps to migrate from the previous version.

```diff
- Old code or configuration
+ New code or configuration
```

## Known Issues

- Issue description and workaround if available
- Link to tracking issue if applicable

## Upgrade Instructions

### From v{PREV_VERSION}

1. Step 1
2. Step 2
3. Step 3

### Configuration Changes

Any configuration file changes needed.

## Downloads

- [macOS (Intel)](https://github.com/tachikoma/tachikoma/releases/download/v{VERSION}/Tachikoma-{VERSION}-x64.dmg)
- [macOS (Apple Silicon)](https://github.com/tachikoma/tachikoma/releases/download/v{VERSION}/Tachikoma-{VERSION}-arm64.dmg)
- [Windows](https://github.com/tachikoma/tachikoma/releases/download/v{VERSION}/Tachikoma-Setup-{VERSION}.exe)
- [Linux (AppImage)](https://github.com/tachikoma/tachikoma/releases/download/v{VERSION}/Tachikoma-{VERSION}.AppImage)
- [Linux (deb)](https://github.com/tachikoma/tachikoma/releases/download/v{VERSION}/tachikoma_{VERSION}_amd64.deb)

## Checksums

```
SHA256 checksums:
abc123... Tachikoma-{VERSION}-x64.dmg
def456... Tachikoma-{VERSION}-arm64.dmg
...
```

## Thank You

Thanks to all contributors who made this release possible!

@contributor1, @contributor2, ...

---

[Full Changelog](https://github.com/tachikoma/tachikoma/blob/main/CHANGELOG.md)
[Documentation](https://docs.tachikoma.dev)
[Report Issues](https://github.com/tachikoma/tachikoma/issues)
```

### 2. Release Notes Generator

Create `scripts/generate-release-notes.ts`:

```typescript
#!/usr/bin/env ts-node
import * as fs from 'fs';
import { execSync } from 'child_process';

interface ReleaseConfig {
  version: string;
  date: string;
  type: 'major' | 'minor' | 'patch';
  highlights: string;
  features: Array<{ title: string; description: string }>;
  improvements: string[];
  bugfixes: Array<{ description: string; issue?: string }>;
  breakingChanges: Array<{ title: string; what: string; why: string; migration: string }>;
  knownIssues: string[];
  upgradeSteps: string[];
}

function loadConfig(path: string): ReleaseConfig {
  return JSON.parse(fs.readFileSync(path, 'utf-8'));
}

function generateReleaseNotes(config: ReleaseConfig): string {
  const template = fs.readFileSync('docs/release-notes/template.md', 'utf-8');

  let notes = template
    .replace(/{VERSION}/g, config.version)
    .replace(/{DATE}/g, config.date)
    .replace(/{Major\|Minor\|Patch}/g, capitalize(config.type));

  // Generate features section
  const featuresSection = config.features
    .map(f => `### ${f.title}\n${f.description}`)
    .join('\n\n');

  // Generate improvements
  const improvementsSection = config.improvements
    .map(i => `- ${i}`)
    .join('\n');

  // Generate bug fixes
  const bugfixesSection = config.bugfixes
    .map(b => `- ${b.description}${b.issue ? ` (#${b.issue})` : ''}`)
    .join('\n');

  // Generate breaking changes
  const breakingSection = config.breakingChanges
    .map(bc => `### ${bc.title}\n**What changed:** ${bc.what}\n\n**Why:** ${bc.why}\n\n**Migration:**\n${bc.migration}`)
    .join('\n\n');

  // Replace sections
  notes = notes
    .replace(/## What's New[\s\S]*?(?=## Improvements)/, `## What's New\n\n${featuresSection}\n\n`)
    .replace(/## Improvements[\s\S]*?(?=## Bug Fixes)/, `## Improvements\n\n${improvementsSection}\n\n`)
    .replace(/## Bug Fixes[\s\S]*?(?=## Breaking Changes)/, `## Bug Fixes\n\n${bugfixesSection}\n\n`)
    .replace(/## Breaking Changes[\s\S]*?(?=## Known Issues)/, `## Breaking Changes\n\n${breakingSection}\n\n`);

  return notes;
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

// Generate checksums
function generateChecksums(dir: string): string {
  const files = fs.readdirSync(dir).filter(f => !f.endsWith('.yml') && !f.endsWith('.blockmap'));
  const checksums = files.map(f => {
    const hash = execSync(`shasum -a 256 "${dir}/${f}"`, { encoding: 'utf-8' }).split(' ')[0];
    return `${hash}  ${f}`;
  });
  return checksums.join('\n');
}

// CLI
const configPath = process.argv[2] || 'release-config.json';
const config = loadConfig(configPath);
const notes = generateReleaseNotes(config);

const outputPath = `docs/release-notes/v${config.version}.md`;
fs.writeFileSync(outputPath, notes);
console.log(`Release notes written to ${outputPath}`);
```

### 3. Release Config Example

Create `release-config.example.json`:

```json
{
  "version": "1.0.0",
  "date": "2024-01-15",
  "type": "major",
  "highlights": "Tachikoma 1.0 brings autonomous AI-powered development to your desktop with multi-model support and spec-driven workflows.",
  "features": [
    {
      "title": "Multi-Model Backend Support",
      "description": "Choose between Claude, GPT-4, Gemini, or local Ollama models for your development tasks."
    },
    {
      "title": "Spec Forge",
      "description": "Collaborative specification creation using multiple AI models to brainstorm and refine your project specs."
    }
  ],
  "improvements": [
    "50% faster startup time",
    "Reduced memory usage during long sessions"
  ],
  "bugfixes": [
    { "description": "Fixed crash when opening large files", "issue": "123" }
  ],
  "breakingChanges": [],
  "knownIssues": [],
  "upgradeSteps": [
    "Download the new version",
    "Your settings will be automatically migrated"
  ]
}
```

### 4. GitHub Release Body Generator

Create `scripts/github-release-body.sh`:

```bash
#!/usr/bin/env bash
VERSION="${1:?Version required}"

# Generate release body for GitHub
cat << EOF
## Tachikoma v${VERSION}

See the [full release notes](https://github.com/tachikoma/tachikoma/blob/main/docs/release-notes/v${VERSION}.md) for details.

### Quick Links
- [Documentation](https://docs.tachikoma.dev)
- [Changelog](https://github.com/tachikoma/tachikoma/blob/main/CHANGELOG.md)
- [Report Issues](https://github.com/tachikoma/tachikoma/issues)

### Installation
Download the appropriate package for your platform below, or use:

\`\`\`bash
# macOS (Homebrew)
brew install --cask tachikoma

# Linux (Snap)
snap install tachikoma
\`\`\`
EOF
```

---

## Testing Requirements

1. Release notes generate from config
2. Template renders correctly
3. Download links are valid
4. Checksums match actual files
5. Notes publish with GitHub release

---

## Related Specs

- Depends on: [506-changelog.md](506-changelog.md)
- Next: [508-distribution-cdn.md](508-distribution-cdn.md)
- Related: [502-auto-update-server.md](502-auto-update-server.md)
