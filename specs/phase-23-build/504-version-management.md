# 504 - Version Management

**Phase:** 23 - Build & Distribution
**Spec ID:** 504
**Status:** Planned
**Dependencies:** 491-build-overview
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement consistent version management across all components (Rust crates, npm packages, Electron app) following semantic versioning principles.

---

## Acceptance Criteria

- [ ] Single source of truth for version
- [ ] Semantic versioning enforced
- [ ] Version bump scripts automated
- [ ] Pre-release versions supported
- [ ] Build metadata included
- [ ] All components stay in sync

---

## Implementation Details

### 1. Version Configuration

Create `version.json` (single source of truth):

```json
{
  "version": "0.1.0",
  "channel": "stable",
  "prerelease": null,
  "build": null
}
```

### 2. Version Sync Script

Create `scripts/version.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Version management script
 * Keeps all version numbers in sync across the project
 */

import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

const ROOT_DIR = path.join(__dirname, '..');
const VERSION_FILE = path.join(ROOT_DIR, 'version.json');

interface VersionConfig {
  version: string;
  channel: 'stable' | 'beta' | 'alpha';
  prerelease: string | null;
  build: string | null;
}

function loadVersion(): VersionConfig {
  return JSON.parse(fs.readFileSync(VERSION_FILE, 'utf-8'));
}

function saveVersion(config: VersionConfig): void {
  fs.writeFileSync(VERSION_FILE, JSON.stringify(config, null, 2) + '\n');
}

function getFullVersion(config: VersionConfig): string {
  let version = config.version;
  if (config.prerelease) {
    version += `-${config.prerelease}`;
  }
  if (config.build) {
    version += `+${config.build}`;
  }
  return version;
}

function updatePackageJson(filePath: string, version: string): void {
  const pkg = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
  pkg.version = version;
  fs.writeFileSync(filePath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`Updated ${filePath}`);
}

function updateCargoToml(filePath: string, version: string): void {
  let content = fs.readFileSync(filePath, 'utf-8');

  // Update workspace version
  content = content.replace(
    /^version = ".*"/m,
    `version = "${version}"`
  );

  // Update workspace.package version
  content = content.replace(
    /(\[workspace\.package\][\s\S]*?)version = ".*"/m,
    `$1version = "${version}"`
  );

  fs.writeFileSync(filePath, content);
  console.log(`Updated ${filePath}`);
}

function syncVersions(): void {
  const config = loadVersion();
  const version = config.version; // Use base version for files

  console.log(`Syncing version: ${getFullVersion(config)}`);

  // Update root package.json
  updatePackageJson(path.join(ROOT_DIR, 'package.json'), version);

  // Update web package.json
  updatePackageJson(path.join(ROOT_DIR, 'web/package.json'), version);

  // Update electron package.json
  updatePackageJson(path.join(ROOT_DIR, 'electron/package.json'), version);

  // Update root Cargo.toml
  updateCargoToml(path.join(ROOT_DIR, 'Cargo.toml'), version);

  console.log('Version sync complete!');
}

function bumpVersion(type: 'major' | 'minor' | 'patch' | 'prerelease'): void {
  const config = loadVersion();
  const [major, minor, patch] = config.version.split('.').map(Number);

  switch (type) {
    case 'major':
      config.version = `${major + 1}.0.0`;
      config.prerelease = null;
      break;
    case 'minor':
      config.version = `${major}.${minor + 1}.0`;
      config.prerelease = null;
      break;
    case 'patch':
      config.version = `${major}.${minor}.${patch + 1}`;
      config.prerelease = null;
      break;
    case 'prerelease':
      if (config.prerelease) {
        const match = config.prerelease.match(/^(\w+)\.(\d+)$/);
        if (match) {
          config.prerelease = `${match[1]}.${Number(match[2]) + 1}`;
        }
      } else {
        config.prerelease = 'beta.1';
      }
      break;
  }

  saveVersion(config);
  syncVersions();

  console.log(`Version bumped to ${getFullVersion(config)}`);
}

function setBuildMetadata(build: string | null): void {
  const config = loadVersion();
  config.build = build;
  saveVersion(config);
  console.log(`Build metadata set to: ${build || '(none)'}`);
}

// CLI
const command = process.argv[2];
const arg = process.argv[3];

switch (command) {
  case 'sync':
    syncVersions();
    break;
  case 'bump':
    if (!['major', 'minor', 'patch', 'prerelease'].includes(arg)) {
      console.error('Usage: version.ts bump <major|minor|patch|prerelease>');
      process.exit(1);
    }
    bumpVersion(arg as any);
    break;
  case 'set':
    if (!arg) {
      console.error('Usage: version.ts set <version>');
      process.exit(1);
    }
    const config = loadVersion();
    config.version = arg;
    saveVersion(config);
    syncVersions();
    break;
  case 'build':
    setBuildMetadata(arg || null);
    break;
  case 'get':
    console.log(getFullVersion(loadVersion()));
    break;
  default:
    console.log('Usage: version.ts <sync|bump|set|build|get>');
    process.exit(1);
}
```

### 3. Package.json Scripts

Update root `package.json`:

```json
{
  "scripts": {
    "version:sync": "ts-node scripts/version.ts sync",
    "version:bump:major": "ts-node scripts/version.ts bump major",
    "version:bump:minor": "ts-node scripts/version.ts bump minor",
    "version:bump:patch": "ts-node scripts/version.ts bump patch",
    "version:bump:prerelease": "ts-node scripts/version.ts bump prerelease",
    "version:set": "ts-node scripts/version.ts set",
    "version:get": "ts-node scripts/version.ts get"
  }
}
```

### 4. CI Version Validation

Create `.github/workflows/version-check.yml`:

```yaml
name: Version Check

on:
  pull_request:
    branches: [main]

jobs:
  check-version:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Check version sync
        run: |
          npm run version:sync
          if [ -n "$(git status --porcelain)" ]; then
            echo "Version files are out of sync!"
            git diff
            exit 1
          fi

      - name: Validate semver
        run: |
          VERSION=$(npm run version:get --silent)
          if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?(\+.+)?$ ]]; then
            echo "Invalid version format: $VERSION"
            exit 1
          fi
```

---

## Testing Requirements

1. Version sync updates all files correctly
2. Version bump follows semver rules
3. Pre-release versions are handled correctly
4. All components report same version
5. CI validates version consistency

---

## Related Specs

- Depends on: [491-build-overview.md](491-build-overview.md)
- Next: [505-release-tagging.md](505-release-tagging.md)
- Related: [506-changelog.md](506-changelog.md)
