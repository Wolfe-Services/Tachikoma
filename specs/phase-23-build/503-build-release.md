# Spec 503: Release Workflow

## Phase
23 - Build/Package System

## Spec ID
503

## Status
Planned

## Dependencies
- Spec 502 (CI Integration)
- Spec 498 (Code Signing)
- Spec 504 (Version Management)

## Estimated Context
~10%

---

## Objective

Implement the complete release workflow for publishing Tachikoma releases. This includes version bumping, changelog generation, release notes, artifact publishing, and distribution to various package repositories.

---

## Acceptance Criteria

- [ ] Automated version bumping (semver)
- [ ] Changelog generation from commits
- [ ] Release notes generation
- [ ] GitHub Release creation
- [ ] Asset upload to GitHub Releases
- [ ] Package repository publishing
- [ ] Auto-update feed generation
- [ ] Release announcement automation
- [ ] Rollback capability
- [ ] Release verification checks

---

## Implementation Details

### Release Workflow (.github/workflows/release.yml)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., 1.2.3)'
        required: true
      prerelease:
        description: 'Is this a prerelease?'
        type: boolean
        default: false

jobs:
  # Validate release
  validate:
    name: Validate Release
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      prerelease: ${{ steps.version.outputs.prerelease }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Determine version
        id: version
        run: |
          if [[ "${{ github.event_name }}" == "workflow_dispatch" ]]; then
            VERSION="${{ github.event.inputs.version }}"
            PRERELEASE="${{ github.event.inputs.prerelease }}"
          else
            VERSION="${GITHUB_REF#refs/tags/v}"
            if [[ "$VERSION" == *"-"* ]]; then
              PRERELEASE="true"
            else
              PRERELEASE="false"
            fi
          fi

          echo "version=$VERSION" >> $GITHUB_OUTPUT
          echo "prerelease=$PRERELEASE" >> $GITHUB_OUTPUT
          echo "Releasing version: $VERSION (prerelease: $PRERELEASE)"

      - name: Validate version format
        run: |
          VERSION="${{ steps.version.outputs.version }}"
          if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
            echo "Invalid version format: $VERSION"
            exit 1
          fi

      - name: Check version not already released
        run: |
          VERSION="${{ steps.version.outputs.version }}"
          if gh release view "v$VERSION" &>/dev/null; then
            echo "Version v$VERSION already exists!"
            exit 1
          fi
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  # Build all platforms
  build:
    name: Build
    needs: validate
    uses: ./.github/workflows/build.yml
    with:
      platforms: linux,darwin,win32
    secrets: inherit

  # Sign all artifacts
  sign:
    name: Sign
    needs: build
    uses: ./.github/workflows/sign.yml
    secrets: inherit

  # Generate changelog
  changelog:
    name: Generate Changelog
    needs: validate
    runs-on: ubuntu-latest
    outputs:
      changelog: ${{ steps.changelog.outputs.changelog }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Generate changelog
        id: changelog
        run: |
          npx conventional-changelog-cli -p angular -r 1 > CHANGELOG_RELEASE.md

          # Read and escape for output
          CHANGELOG=$(cat CHANGELOG_RELEASE.md)
          echo "changelog<<EOF" >> $GITHUB_OUTPUT
          echo "$CHANGELOG" >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT

      - name: Upload changelog
        uses: actions/upload-artifact@v4
        with:
          name: changelog
          path: CHANGELOG_RELEASE.md

  # Create GitHub release
  release:
    name: Create Release
    needs: [validate, sign, changelog]
    runs-on: ubuntu-latest
    outputs:
      release_url: ${{ steps.release.outputs.url }}
    steps:
      - uses: actions/checkout@v4

      - name: Download signed artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Prepare release assets
        run: |
          mkdir -p release-assets
          find artifacts -type f \( \
            -name "*.dmg" -o \
            -name "*.zip" -o \
            -name "*.exe" -o \
            -name "*.msi" -o \
            -name "*.AppImage" -o \
            -name "*.deb" -o \
            -name "*.rpm" -o \
            -name "*.snap" \
          \) -exec cp {} release-assets/ \;

          # Generate checksums
          cd release-assets
          sha256sum * > SHA256SUMS.txt
          cd ..

      - name: Create release notes
        run: |
          VERSION="${{ needs.validate.outputs.version }}"
          cat > RELEASE_NOTES.md << EOF
          # Tachikoma v$VERSION

          ${{ needs.changelog.outputs.changelog }}

          ## Installation

          ### macOS
          - **Apple Silicon**: Download \`Tachikoma-$VERSION-arm64.dmg\`
          - **Intel**: Download \`Tachikoma-$VERSION-x64.dmg\`
          - **Universal**: Download \`Tachikoma-$VERSION-universal.dmg\`

          ### Windows
          - **Installer**: Download \`Tachikoma-Setup-$VERSION.exe\`
          - **Portable**: Download \`Tachikoma-$VERSION-portable.exe\`
          - **MSI**: Download \`Tachikoma-$VERSION.msi\`

          ### Linux
          - **AppImage**: Download \`Tachikoma-$VERSION.AppImage\`
          - **Debian/Ubuntu**: Download \`tachikoma_$VERSION_amd64.deb\`
          - **Fedora/RHEL**: Download \`tachikoma-$VERSION.x86_64.rpm\`
          - **Snap**: \`sudo snap install tachikoma\`

          ## Verification

          All release assets are signed. Verify checksums with:
          \`\`\`bash
          sha256sum -c SHA256SUMS.txt
          \`\`\`

          ## Changelog

          See [CHANGELOG.md](https://github.com/tachikoma/tachikoma/blob/main/CHANGELOG.md) for full history.
          EOF

      - name: Create GitHub Release
        id: release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: v${{ needs.validate.outputs.version }}
          name: Tachikoma v${{ needs.validate.outputs.version }}
          body_path: RELEASE_NOTES.md
          prerelease: ${{ needs.validate.outputs.prerelease == 'true' }}
          files: release-assets/*
          fail_on_unmatched_files: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  # Publish to package repositories
  publish:
    name: Publish
    needs: [validate, release]
    runs-on: ubuntu-latest
    if: needs.validate.outputs.prerelease == 'false'
    steps:
      - uses: actions/checkout@v4

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Publish to Snapcraft
        run: |
          echo "${{ secrets.SNAPCRAFT_TOKEN }}" | snapcraft login --with -
          snapcraft upload artifacts/signed-linux/*.snap --release=stable
        continue-on-error: true

      - name: Update Homebrew tap
        run: |
          # Trigger Homebrew tap update
          gh workflow run update-formula.yml \
            --repo tachikoma/homebrew-tap \
            -f version=${{ needs.validate.outputs.version }}
        env:
          GH_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
        continue-on-error: true

      - name: Update AUR package
        run: |
          # Trigger AUR package update
          gh workflow run update-aur.yml \
            -f version=${{ needs.validate.outputs.version }}
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        continue-on-error: true

  # Update auto-update feed
  update-feed:
    name: Update Feed
    needs: [validate, release]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: tachikoma/releases
          token: ${{ secrets.RELEASES_REPO_TOKEN }}

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          name: changelog
          path: .

      - name: Generate update feeds
        run: |
          VERSION="${{ needs.validate.outputs.version }}"

          # Generate Sparkle appcast (macOS)
          npx ts-node scripts/generate-appcast.ts \
            --version "$VERSION" \
            --changelog CHANGELOG_RELEASE.md \
            --output appcast.xml

          # Generate NSIS update info (Windows)
          npx ts-node scripts/generate-nsis-update.ts \
            --version "$VERSION" \
            --output latest.yml

          # Generate AppImage update info
          npx ts-node scripts/generate-appimage-update.ts \
            --version "$VERSION" \
            --output latest-linux.yml

      - name: Commit and push
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add .
          git commit -m "Update feeds for v${{ needs.validate.outputs.version }}"
          git push

  # Notify
  notify:
    name: Notify
    needs: [validate, release, publish]
    runs-on: ubuntu-latest
    if: always()
    steps:
      - name: Send Discord notification
        if: needs.release.result == 'success'
        run: |
          curl -H "Content-Type: application/json" \
            -d "{\"content\": \"Tachikoma v${{ needs.validate.outputs.version }} has been released! ${{ needs.release.outputs.release_url }}\"}" \
            "${{ secrets.DISCORD_WEBHOOK_URL }}"
        continue-on-error: true

      - name: Send Slack notification
        if: needs.release.result == 'success'
        run: |
          curl -X POST \
            -H "Content-Type: application/json" \
            -d "{\"text\": \"Tachikoma v${{ needs.validate.outputs.version }} has been released!\"}" \
            "${{ secrets.SLACK_WEBHOOK_URL }}"
        continue-on-error: true
```

### Release Script (scripts/release.ts)

```typescript
// scripts/release.ts
import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as readline from 'readline';

interface ReleaseConfig {
  version: string;
  prerelease: boolean;
  dryRun: boolean;
  skipTests: boolean;
  skipBuild: boolean;
}

async function prompt(question: string): Promise<string> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      rl.close();
      resolve(answer);
    });
  });
}

async function runCommand(
  command: string,
  args: string[],
  options: { dryRun?: boolean; cwd?: string } = {}
): Promise<void> {
  if (options.dryRun) {
    console.log(`[DRY RUN] ${command} ${args.join(' ')}`);
    return;
  }

  return new Promise((resolve, reject) => {
    const proc = spawn(command, args, {
      cwd: options.cwd,
      stdio: 'inherit',
      shell: true,
    });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${command} failed with code ${code}`));
      }
    });
  });
}

async function getCurrentVersion(): Promise<string> {
  const packageJson = JSON.parse(
    fs.readFileSync('package.json', 'utf-8')
  );
  return packageJson.version;
}

async function updateVersion(version: string, dryRun: boolean): Promise<void> {
  console.log(`Updating version to ${version}...`);

  // Update root package.json
  const packageJsonPath = 'package.json';
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
  packageJson.version = version;

  if (!dryRun) {
    fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
  }

  // Update Cargo.toml
  const cargoTomlPath = 'Cargo.toml';
  let cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
  cargoToml = cargoToml.replace(
    /^version = "[^"]+"/m,
    `version = "${version}"`
  );

  if (!dryRun) {
    fs.writeFileSync(cargoTomlPath, cargoToml);
  }

  // Update web/package.json
  const webPackageJsonPath = 'web/package.json';
  const webPackageJson = JSON.parse(fs.readFileSync(webPackageJsonPath, 'utf-8'));
  webPackageJson.version = version;

  if (!dryRun) {
    fs.writeFileSync(webPackageJsonPath, JSON.stringify(webPackageJson, null, 2) + '\n');
  }

  // Update electron/package.json
  const electronPackageJsonPath = 'electron/package.json';
  const electronPackageJson = JSON.parse(fs.readFileSync(electronPackageJsonPath, 'utf-8'));
  electronPackageJson.version = version;

  if (!dryRun) {
    fs.writeFileSync(electronPackageJsonPath, JSON.stringify(electronPackageJson, null, 2) + '\n');
  }
}

async function generateChangelog(dryRun: boolean): Promise<void> {
  console.log('Generating changelog...');
  await runCommand('npx', ['conventional-changelog', '-p', 'angular', '-i', 'CHANGELOG.md', '-s'], { dryRun });
}

async function createGitTag(version: string, dryRun: boolean): Promise<void> {
  console.log(`Creating git tag v${version}...`);

  await runCommand('git', ['add', '-A'], { dryRun });
  await runCommand('git', ['commit', '-m', `chore(release): v${version}`], { dryRun });
  await runCommand('git', ['tag', '-a', `v${version}`, '-m', `Release v${version}`], { dryRun });
}

async function pushRelease(version: string, dryRun: boolean): Promise<void> {
  console.log('Pushing to remote...');

  await runCommand('git', ['push', 'origin', 'main'], { dryRun });
  await runCommand('git', ['push', 'origin', `v${version}`], { dryRun });
}

async function release(config: ReleaseConfig): Promise<void> {
  const { version, prerelease, dryRun, skipTests, skipBuild } = config;

  console.log(`\nReleasing Tachikoma v${version}`);
  console.log(`Prerelease: ${prerelease}`);
  console.log(`Dry run: ${dryRun}`);
  console.log('');

  // Verify clean working directory
  const statusResult = await new Promise<string>((resolve) => {
    const proc = spawn('git', ['status', '--porcelain'], { shell: true });
    let output = '';
    proc.stdout.on('data', (data) => { output += data; });
    proc.on('close', () => resolve(output));
  });

  if (statusResult.trim() && !dryRun) {
    console.error('Working directory is not clean. Commit or stash changes first.');
    process.exit(1);
  }

  // Run tests
  if (!skipTests) {
    console.log('Running tests...');
    await runCommand('npm', ['test'], { dryRun });
  }

  // Build
  if (!skipBuild) {
    console.log('Building...');
    await runCommand('npm', ['run', 'build'], { dryRun });
  }

  // Update version
  await updateVersion(version, dryRun);

  // Generate changelog
  await generateChangelog(dryRun);

  // Create and push tag
  await createGitTag(version, dryRun);
  await pushRelease(version, dryRun);

  console.log(`\nRelease v${version} created successfully!`);
  console.log('GitHub Actions will now build and publish the release.');
}

// CLI
async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const config: ReleaseConfig = {
    version: '',
    prerelease: false,
    dryRun: args.includes('--dry-run'),
    skipTests: args.includes('--skip-tests'),
    skipBuild: args.includes('--skip-build'),
  };

  // Get version from args or prompt
  const versionArg = args.find((a) => !a.startsWith('--'));
  if (versionArg) {
    config.version = versionArg;
  } else {
    const currentVersion = await getCurrentVersion();
    console.log(`Current version: ${currentVersion}`);
    config.version = await prompt('New version: ');
  }

  // Validate version
  if (!/^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$/.test(config.version)) {
    console.error('Invalid version format');
    process.exit(1);
  }

  config.prerelease = config.version.includes('-');

  // Confirm
  const confirm = await prompt(`Release v${config.version}? (y/N) `);
  if (confirm.toLowerCase() !== 'y') {
    console.log('Aborted');
    process.exit(0);
  }

  await release(config);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
```

### Appcast Generator (scripts/generate-appcast.ts)

```typescript
// scripts/generate-appcast.ts
import * as fs from 'fs';
import * as path from 'path';
import { createHash } from 'crypto';

interface AppcastItem {
  version: string;
  buildNumber: string;
  releaseNotes: string;
  pubDate: Date;
  downloadURL: string;
  signature: string;
  length: number;
  minimumSystemVersion: string;
}

function generateAppcastItem(item: AppcastItem): string {
  return `
    <item>
      <title>Version ${item.version}</title>
      <sparkle:releaseNotesLink>https://tachikoma.io/releases/v${item.version}/notes</sparkle:releaseNotesLink>
      <pubDate>${item.pubDate.toUTCString()}</pubDate>
      <enclosure
        url="${item.downloadURL}"
        sparkle:version="${item.buildNumber}"
        sparkle:shortVersionString="${item.version}"
        sparkle:edSignature="${item.signature}"
        length="${item.length}"
        type="application/octet-stream"
      />
      <sparkle:minimumSystemVersion>${item.minimumSystemVersion}</sparkle:minimumSystemVersion>
    </item>`;
}

function generateAppcast(items: AppcastItem[]): string {
  const itemsXml = items.map(generateAppcastItem).join('\n');

  return `<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Tachikoma Updates</title>
    <link>https://tachikoma.io</link>
    <description>Updates for Tachikoma</description>
    <language>en</language>
    ${itemsXml}
  </channel>
</rss>`;
}

// CLI
const args = process.argv.slice(2);
const version = args.find((a) => a.startsWith('--version='))?.split('=')[1];
const changelogPath = args.find((a) => a.startsWith('--changelog='))?.split('=')[1];
const outputPath = args.find((a) => a.startsWith('--output='))?.split('=')[1] || 'appcast.xml';

if (!version) {
  console.error('Version required');
  process.exit(1);
}

const changelog = changelogPath
  ? fs.readFileSync(changelogPath, 'utf-8')
  : '';

const item: AppcastItem = {
  version,
  buildNumber: version.replace(/\./g, ''),
  releaseNotes: changelog,
  pubDate: new Date(),
  downloadURL: `https://github.com/tachikoma/tachikoma/releases/download/v${version}/Tachikoma-${version}-universal.dmg`,
  signature: '', // Would be signed with EdDSA key
  length: 0, // Would be actual file size
  minimumSystemVersion: '10.15',
};

const appcast = generateAppcast([item]);
fs.writeFileSync(outputPath, appcast);
console.log(`Appcast written to ${outputPath}`);
```

---

## Testing Requirements

### Release Script Tests

```typescript
// scripts/__tests__/release.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as fs from 'fs';

vi.mock('fs');
vi.mock('child_process');

describe('Release Script', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should validate version format', () => {
    const validVersions = ['1.0.0', '1.0.0-alpha.1', '1.0.0-beta', '0.1.0'];
    const invalidVersions = ['1.0', '1', 'v1.0.0', '1.0.0.0'];

    const versionRegex = /^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$/;

    for (const v of validVersions) {
      expect(versionRegex.test(v)).toBe(true);
    }

    for (const v of invalidVersions) {
      expect(versionRegex.test(v)).toBe(false);
    }
  });

  it('should detect prerelease versions', () => {
    expect('1.0.0-alpha.1'.includes('-')).toBe(true);
    expect('1.0.0'.includes('-')).toBe(false);
  });
});
```

### Workflow Tests

```typescript
// .github/__tests__/release.test.ts
import { describe, it, expect } from 'vitest';
import * as yaml from 'yaml';
import * as fs from 'fs';
import * as path from 'path';

describe('Release Workflow', () => {
  const workflowPath = path.join(__dirname, '..', 'workflows', 'release.yml');

  it('should have valid release workflow', () => {
    expect(fs.existsSync(workflowPath)).toBe(true);

    const content = fs.readFileSync(workflowPath, 'utf-8');
    const workflow = yaml.parse(content);

    expect(workflow.name).toBe('Release');
    expect(workflow.on.push.tags).toContain('v*');
    expect(workflow.jobs.validate).toBeDefined();
    expect(workflow.jobs.build).toBeDefined();
    expect(workflow.jobs.release).toBeDefined();
  });

  it('should have signing job', () => {
    const content = fs.readFileSync(workflowPath, 'utf-8');
    const workflow = yaml.parse(content);

    expect(workflow.jobs.sign).toBeDefined();
  });

  it('should have publish job', () => {
    const content = fs.readFileSync(workflowPath, 'utf-8');
    const workflow = yaml.parse(content);

    expect(workflow.jobs.publish).toBeDefined();
  });
});
```

---

## Related Specs

- Spec 502: CI Integration
- Spec 498: Code Signing
- Spec 504: Version Management
- Spec 494: Electron Packaging
