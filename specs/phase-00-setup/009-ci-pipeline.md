# 009 - CI Pipeline Setup

**Phase:** 0 - Setup
**Spec ID:** 009
**Status:** Planned
**Dependencies:** 007-build-system, 008-test-infrastructure
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Configure GitHub Actions CI/CD pipeline for automated testing, linting, and build verification on every push and pull request.

---

## Acceptance Criteria

- [ ] CI runs on push to main and PRs
- [ ] Rust tests and clippy run
- [ ] TypeScript tests and linting run
- [ ] Build verification for all platforms
- [ ] Cache configuration for fast builds
- [ ] Status badges for README

---

## Implementation Details

### 1. Main CI Workflow (.github/workflows/ci.yml)

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  rust:
    name: Rust (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.os }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Test
        run: cargo test --workspace --no-fail-fast

  web:
    name: Web
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
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
        run: cd web && npm test -- --run

  electron:
    name: Electron Build
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    needs: [rust, web]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install dependencies
        run: |
          npm ci
          cd web && npm ci
          cd ../electron && npm ci

      - name: Build
        run: npm run build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: build-${{ matrix.os }}
          path: electron/dist/
          retention-days: 7
```

### 2. Release Workflow (.github/workflows/release.yml)

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: linux
          - os: macos-latest
            target: darwin
          - os: windows-latest
            target: win32

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: |
          npm ci
          cd web && npm ci
          cd ../electron && npm ci

      - name: Build release
        run: npm run build
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Package
        run: cd electron && npm run package
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Upload release assets
        uses: softprops/action-gh-release@v1
        with:
          files: |
            electron/release/*.dmg
            electron/release/*.exe
            electron/release/*.AppImage
            electron/release/*.deb
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 3. Dependabot Configuration (.github/dependabot.yml)

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
    groups:
      rust-dependencies:
        patterns:
          - "*"

  - package-ecosystem: npm
    directory: /web
    schedule:
      interval: weekly
    groups:
      npm-web:
        patterns:
          - "*"

  - package-ecosystem: npm
    directory: /electron
    schedule:
      interval: weekly
    groups:
      npm-electron:
        patterns:
          - "*"

  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
```

### 4. Branch Protection (document for manual setup)

Configure in GitHub Settings > Branches > main:

- Require pull request reviews before merging
- Require status checks to pass:
  - `Rust (ubuntu-latest)`
  - `Web`
- Require branches to be up to date
- Include administrators

---

## Testing Requirements

1. Push to feature branch triggers CI
2. All checks pass on clean code
3. Failed tests block merge
4. Release workflow triggers on tag

---

## Related Specs

- Depends on: [007-build-system.md](007-build-system.md), [008-test-infrastructure.md](008-test-infrastructure.md)
- Next: [010-documentation-setup.md](010-documentation-setup.md)
