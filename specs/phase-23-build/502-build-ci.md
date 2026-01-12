# Spec 502: CI Integration

## Phase
23 - Build/Package System

## Spec ID
502

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 009 (CI Pipeline Setup)
- Spec 493 (Rust Compilation)

## Estimated Context
~12%

---

## Objective

Implement comprehensive CI/CD integration for the build system using GitHub Actions. This includes automated builds for all platforms, artifact generation, caching strategies, and integration with the release workflow.

---

## Acceptance Criteria

- [ ] Matrix builds for all supported platforms
- [ ] Efficient caching for Rust, Node.js, and Electron
- [ ] Parallel job execution where possible
- [ ] Build artifact upload and retention
- [ ] Pull request verification builds
- [ ] Nightly and release builds
- [ ] Build status notifications
- [ ] Build metrics and performance tracking
- [ ] Dependency vulnerability scanning
- [ ] Build artifact signing in CI

---

## Implementation Details

### Main CI Workflow (.github/workflows/ci.yml)

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
  workflow_dispatch:
    inputs:
      debug:
        description: 'Enable debug mode'
        required: false
        default: 'false'

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  SCCACHE_GHA_ENABLED: 'true'
  RUSTC_WRAPPER: sccache

jobs:
  # Determine what changed
  changes:
    name: Detect Changes
    runs-on: ubuntu-latest
    outputs:
      rust: ${{ steps.filter.outputs.rust }}
      web: ${{ steps.filter.outputs.web }}
      electron: ${{ steps.filter.outputs.electron }}
      docs: ${{ steps.filter.outputs.docs }}
    steps:
      - uses: actions/checkout@v4

      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            rust:
              - 'crates/**'
              - 'Cargo.toml'
              - 'Cargo.lock'
            web:
              - 'web/**'
              - 'package.json'
            electron:
              - 'electron/**'
            docs:
              - 'docs/**'
              - '*.md'

  # Rust checks and tests
  rust:
    name: Rust (${{ matrix.os }})
    needs: changes
    if: needs.changes.outputs.rust == 'true' || github.event_name == 'workflow_dispatch'
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
          targets: ${{ matrix.target }}

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.4

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.os }}-${{ matrix.target }}
          cache-targets: true

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Build
        run: cargo build --workspace --target ${{ matrix.target }}

      - name: Test
        run: cargo test --workspace --target ${{ matrix.target }} -- --test-threads=1

      - name: Build release
        if: github.ref == 'refs/heads/main'
        run: cargo build --workspace --release --target ${{ matrix.target }}

      - name: Upload artifacts
        if: github.ref == 'refs/heads/main'
        uses: actions/upload-artifact@v4
        with:
          name: rust-${{ matrix.os }}-${{ matrix.target }}
          path: |
            target/${{ matrix.target }}/release/tachikoma*
            !target/${{ matrix.target }}/release/*.d
            !target/${{ matrix.target }}/release/*.pdb
          retention-days: 7

  # Web checks and tests
  web:
    name: Web
    needs: changes
    if: needs.changes.outputs.web == 'true' || github.event_name == 'workflow_dispatch'
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: web/package-lock.json

      - name: Install dependencies
        run: cd web && npm ci

      - name: Lint
        run: cd web && npm run lint

      - name: Type check
        run: cd web && npm run check

      - name: Test
        run: cd web && npm test -- --run --coverage

      - name: Build
        run: cd web && npm run build

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          directory: web/coverage
          flags: web

      - name: Upload build
        uses: actions/upload-artifact@v4
        with:
          name: web-build
          path: web/dist
          retention-days: 7

  # Electron build
  electron:
    name: Electron (${{ matrix.os }})
    needs: [rust, web]
    if: always() && !cancelled() && (needs.rust.result == 'success' || needs.rust.result == 'skipped') && (needs.web.result == 'success' || needs.web.result == 'skipped')
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            platform: linux
          - os: macos-latest
            platform: darwin
          - os: windows-latest
            platform: win32

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.4

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Download web build
        uses: actions/download-artifact@v4
        with:
          name: web-build
          path: web/dist

      - name: Install dependencies
        run: |
          npm ci
          cd electron && npm ci

      - name: Build native module
        run: cd crates/tachikoma-native && npm run build

      - name: Build Electron
        run: cd electron && npm run build

      - name: Package (dry run)
        run: cd electron && npm run build:unpack

      - name: Upload package
        uses: actions/upload-artifact@v4
        with:
          name: electron-${{ matrix.platform }}
          path: |
            electron/dist/**/!(*.map)
          retention-days: 7

  # Security scanning
  security:
    name: Security Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Rust audit
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: npm audit
        run: |
          npm audit --audit-level=high || true
          cd web && npm audit --audit-level=high || true
          cd ../electron && npm audit --audit-level=high || true

  # Summary job
  ci-success:
    name: CI Success
    needs: [rust, web, electron, security]
    if: always()
    runs-on: ubuntu-latest
    steps:
      - name: Check job results
        run: |
          if [[ "${{ needs.rust.result }}" == "failure" ]] || \
             [[ "${{ needs.web.result }}" == "failure" ]] || \
             [[ "${{ needs.electron.result }}" == "failure" ]]; then
            echo "One or more jobs failed"
            exit 1
          fi
          echo "All jobs passed!"
```

### Build Workflow (.github/workflows/build.yml)

```yaml
# .github/workflows/build.yml
name: Build

on:
  push:
    branches: [main]
    tags: ['v*']
  workflow_dispatch:
    inputs:
      platforms:
        description: 'Platforms to build (comma-separated: linux,darwin,win32)'
        required: false
        default: 'linux,darwin,win32'
      sign:
        description: 'Sign the builds'
        required: false
        default: 'false'

jobs:
  build-matrix:
    name: Build Matrix
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.set-matrix.outputs.matrix }}
    steps:
      - id: set-matrix
        run: |
          if [[ "${{ github.event_name }}" == "workflow_dispatch" ]]; then
            platforms="${{ github.event.inputs.platforms }}"
          else
            platforms="linux,darwin,win32"
          fi

          matrix="{\"include\":["
          first=true

          IFS=',' read -ra PLATFORMS <<< "$platforms"
          for platform in "${PLATFORMS[@]}"; do
            if [ "$first" = true ]; then
              first=false
            else
              matrix+=","
            fi

            case $platform in
              linux)
                matrix+="{\"os\":\"ubuntu-latest\",\"platform\":\"linux\",\"arch\":[\"x64\",\"arm64\"]}"
                ;;
              darwin)
                matrix+="{\"os\":\"macos-latest\",\"platform\":\"darwin\",\"arch\":[\"x64\",\"arm64\",\"universal\"]}"
                ;;
              win32)
                matrix+="{\"os\":\"windows-latest\",\"platform\":\"win32\",\"arch\":[\"x64\"]}"
                ;;
            esac
          done

          matrix+="]}"
          echo "matrix=$matrix" >> $GITHUB_OUTPUT

  build:
    name: Build (${{ matrix.platform }})
    needs: build-matrix
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix: ${{ fromJson(needs.build-matrix.outputs.matrix) }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: |
            x86_64-unknown-linux-gnu
            aarch64-unknown-linux-gnu
            x86_64-apple-darwin
            aarch64-apple-darwin
            x86_64-pc-windows-msvc

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install Linux dependencies
        if: matrix.platform == 'linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libgtk-3-dev \
            libwebkit2gtk-4.1-dev \
            librsvg2-dev \
            patchelf

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.4

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          key: build-${{ matrix.platform }}

      - name: Install dependencies
        run: |
          npm ci
          cd web && npm ci
          cd ../electron && npm ci

      - name: Build Rust (all architectures)
        run: |
          npm run build:rust

      - name: Build web
        run: cd web && npm run build

      - name: Build native module
        run: cd crates/tachikoma-native && npm run build

      - name: Build Electron
        run: cd electron && npm run build

      - name: Package
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          cd electron
          npm run package:${{ matrix.platform }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: build-${{ matrix.platform }}
          path: |
            electron/release/**/*.dmg
            electron/release/**/*.zip
            electron/release/**/*.exe
            electron/release/**/*.msi
            electron/release/**/*.AppImage
            electron/release/**/*.deb
            electron/release/**/*.rpm
            electron/release/**/*.snap
          retention-days: 14

  sign:
    name: Sign (${{ matrix.platform }})
    needs: build
    if: github.event.inputs.sign == 'true' || startsWith(github.ref, 'refs/tags/')
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: macos-latest
            platform: darwin
          - os: windows-latest
            platform: win32
          - os: ubuntu-latest
            platform: linux

    steps:
      - uses: actions/checkout@v4

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          name: build-${{ matrix.platform }}
          path: release

      - name: Sign (macOS)
        if: matrix.platform == 'darwin'
        env:
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_ID_PASSWORD: ${{ secrets.APPLE_ID_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          # Import certificate and sign
          npx ts-node scripts/signing/macos-sign.ts release

      - name: Sign (Windows)
        if: matrix.platform == 'win32'
        env:
          WINDOWS_CERTIFICATE_PATH: certificate.pfx
          WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        run: |
          echo "${{ secrets.WINDOWS_CERTIFICATE_BASE64 }}" | base64 -d > certificate.pfx
          npx ts-node scripts/signing/windows-sign.ts release
          rm certificate.pfx

      - name: Sign (Linux)
        if: matrix.platform == 'linux'
        env:
          GPG_PRIVATE_KEY: ${{ secrets.GPG_PRIVATE_KEY }}
          GPG_PASSPHRASE: ${{ secrets.GPG_PASSPHRASE }}
        run: |
          echo "$GPG_PRIVATE_KEY" | gpg --batch --import
          npx ts-node scripts/signing/linux-sign.ts release

      - name: Upload signed artifacts
        uses: actions/upload-artifact@v4
        with:
          name: signed-${{ matrix.platform }}
          path: release
          retention-days: 14
```

### Nightly Build Workflow (.github/workflows/nightly.yml)

```yaml
# .github/workflows/nightly.yml
name: Nightly Build

on:
  schedule:
    - cron: '0 2 * * *' # 2 AM UTC
  workflow_dispatch:

jobs:
  check-changes:
    name: Check for Changes
    runs-on: ubuntu-latest
    outputs:
      should_build: ${{ steps.check.outputs.should_build }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 2

      - name: Check for changes since last nightly
        id: check
        run: |
          # Get the last nightly release date
          last_nightly=$(gh release list --json tagName,publishedAt \
            --jq '.[] | select(.tagName | startswith("nightly-")) | .publishedAt' \
            | head -1)

          if [ -z "$last_nightly" ]; then
            echo "should_build=true" >> $GITHUB_OUTPUT
          else
            # Check if there are commits since then
            commits=$(git log --since="$last_nightly" --oneline | wc -l)
            if [ "$commits" -gt 0 ]; then
              echo "should_build=true" >> $GITHUB_OUTPUT
            else
              echo "should_build=false" >> $GITHUB_OUTPUT
            fi
          fi
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  build:
    name: Nightly Build
    needs: check-changes
    if: needs.check-changes.outputs.should_build == 'true' || github.event_name == 'workflow_dispatch'
    uses: ./.github/workflows/build.yml
    with:
      platforms: linux,darwin,win32
    secrets: inherit

  release:
    name: Create Nightly Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Generate release notes
        id: notes
        run: |
          echo "# Nightly Build $(date +%Y-%m-%d)" > RELEASE_NOTES.md
          echo "" >> RELEASE_NOTES.md
          echo "This is an automated nightly build. Use at your own risk." >> RELEASE_NOTES.md
          echo "" >> RELEASE_NOTES.md
          echo "## Changes since last nightly" >> RELEASE_NOTES.md
          git log --oneline -20 >> RELEASE_NOTES.md

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: nightly-${{ github.run_number }}
          name: Nightly Build ${{ github.run_number }}
          body_path: RELEASE_NOTES.md
          prerelease: true
          files: |
            artifacts/**/*.dmg
            artifacts/**/*.zip
            artifacts/**/*.exe
            artifacts/**/*.AppImage
            artifacts/**/*.deb
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Cleanup old nightlies
        run: |
          # Keep only the last 7 nightly releases
          gh release list --json tagName \
            --jq '.[] | select(.tagName | startswith("nightly-")) | .tagName' \
            | tail -n +8 \
            | xargs -I {} gh release delete {} --yes || true
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Reusable Workflow for Tests (.github/workflows/test.yml)

```yaml
# .github/workflows/test.yml
name: Test

on:
  workflow_call:
    inputs:
      rust:
        description: 'Run Rust tests'
        type: boolean
        default: true
      web:
        description: 'Run web tests'
        type: boolean
        default: true
      e2e:
        description: 'Run E2E tests'
        type: boolean
        default: false

jobs:
  rust-test:
    if: inputs.rust
    name: Rust Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Run doc tests
        run: cargo test --doc --workspace

  web-test:
    if: inputs.web
    name: Web Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: web/package-lock.json

      - name: Install dependencies
        run: cd web && npm ci

      - name: Run tests
        run: cd web && npm test -- --run --coverage

  e2e-test:
    if: inputs.e2e
    name: E2E Tests
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Build
        run: npm run build

      - name: Run E2E tests
        run: npm run test:e2e
```

---

## Testing Requirements

### CI Configuration Tests

```typescript
// .github/__tests__/ci.test.ts
import { describe, it, expect } from 'vitest';
import * as yaml from 'yaml';
import * as fs from 'fs';
import * as path from 'path';

describe('CI Workflows', () => {
  const workflowsDir = path.join(__dirname, '..', 'workflows');

  it('should have valid CI workflow', () => {
    const ciPath = path.join(workflowsDir, 'ci.yml');
    expect(fs.existsSync(ciPath)).toBe(true);

    const content = fs.readFileSync(ciPath, 'utf-8');
    const workflow = yaml.parse(content);

    expect(workflow.name).toBe('CI');
    expect(workflow.on.push).toBeDefined();
    expect(workflow.on.pull_request).toBeDefined();
    expect(workflow.jobs).toBeDefined();
  });

  it('should have valid build workflow', () => {
    const buildPath = path.join(workflowsDir, 'build.yml');
    expect(fs.existsSync(buildPath)).toBe(true);

    const content = fs.readFileSync(buildPath, 'utf-8');
    const workflow = yaml.parse(content);

    expect(workflow.name).toBe('Build');
    expect(workflow.jobs.build).toBeDefined();
  });

  it('should cache Rust dependencies', () => {
    const ciPath = path.join(workflowsDir, 'ci.yml');
    const content = fs.readFileSync(ciPath, 'utf-8');

    expect(content).toContain('Swatinem/rust-cache');
  });

  it('should cache Node.js dependencies', () => {
    const ciPath = path.join(workflowsDir, 'ci.yml');
    const content = fs.readFileSync(ciPath, 'utf-8');

    expect(content).toContain("cache: 'npm'");
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 009: CI Pipeline Setup
- Spec 503: Release Workflow
- Spec 498: Code Signing
