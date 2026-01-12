# 491 - Build System Overview

**Phase:** 23 - Build & Distribution
**Spec ID:** 491
**Status:** Planned
**Dependencies:** 007-build-system, 175-build-config
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Provide a comprehensive overview of the Tachikoma build system, documenting the complete build pipeline from source to distributable packages across all supported platforms.

---

## Acceptance Criteria

- [ ] Build pipeline architecture documented
- [ ] Platform-specific build processes defined
- [ ] Dependencies and prerequisites listed
- [ ] Build artifacts and outputs specified
- [ ] CI/CD integration points identified
- [ ] Build optimization strategies documented

---

## Implementation Details

### 1. Build Pipeline Architecture

```
Source Code
    │
    ├─► Rust Workspace ─► cargo build ─► Native Libraries
    │                                      │
    │                                      ▼
    │                              tachikoma-native.node
    │                                      │
    ├─► TypeScript/Svelte ─► vite build ─► web/dist/
    │                                      │
    │                                      ▼
    │                              Bundled Frontend
    │                                      │
    ├─► Electron Shell ─► electron-builder ─┬─► macOS (.dmg, .app)
    │                                       ├─► Windows (.exe, .msi)
    │                                       └─► Linux (.AppImage, .deb)
    │
    └─► Server Binary ─► cargo build ─► tachikoma-server
```

### 2. Build Configuration

Create `build.config.ts`:

```typescript
/**
 * Tachikoma Build Configuration
 *
 * Central configuration for all build processes.
 */

export interface BuildConfig {
  // Version info
  version: string;
  buildNumber: string;
  gitCommit: string;

  // Paths
  rootDir: string;
  outputDir: string;
  cacheDir: string;

  // Platform targets
  platforms: PlatformConfig[];

  // Build options
  options: BuildOptions;
}

export interface PlatformConfig {
  name: 'darwin' | 'win32' | 'linux';
  arch: 'x64' | 'arm64';
  enabled: boolean;
}

export interface BuildOptions {
  // Optimization level
  release: boolean;

  // Code signing
  sign: boolean;
  notarize: boolean;

  // Output formats
  formats: {
    dmg: boolean;
    pkg: boolean;
    nsis: boolean;
    msi: boolean;
    appimage: boolean;
    deb: boolean;
    rpm: boolean;
  };

  // Features
  includeSourceMaps: boolean;
  stripDebugSymbols: boolean;
  compressAssets: boolean;
}

export function loadBuildConfig(): BuildConfig {
  const pkg = require('../package.json');

  return {
    version: pkg.version,
    buildNumber: process.env.BUILD_NUMBER || 'dev',
    gitCommit: process.env.GIT_COMMIT || 'unknown',

    rootDir: process.cwd(),
    outputDir: 'dist',
    cacheDir: '.build-cache',

    platforms: [
      { name: 'darwin', arch: 'x64', enabled: true },
      { name: 'darwin', arch: 'arm64', enabled: true },
      { name: 'win32', arch: 'x64', enabled: true },
      { name: 'linux', arch: 'x64', enabled: true },
    ],

    options: {
      release: process.env.NODE_ENV === 'production',
      sign: !!process.env.CSC_LINK,
      notarize: !!process.env.APPLE_ID,

      formats: {
        dmg: true,
        pkg: false,
        nsis: true,
        msi: false,
        appimage: true,
        deb: true,
        rpm: false,
      },

      includeSourceMaps: process.env.NODE_ENV !== 'production',
      stripDebugSymbols: process.env.NODE_ENV === 'production',
      compressAssets: true,
    },
  };
}
```

### 3. Master Build Script

Create `scripts/build.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Master build script for Tachikoma
 */

import { spawn } from 'child_process';
import { loadBuildConfig, BuildConfig } from '../build.config';
import * as path from 'path';

const config = loadBuildConfig();

interface BuildStep {
  name: string;
  command: string;
  args: string[];
  cwd?: string;
  env?: Record<string, string>;
  condition?: () => boolean;
}

const buildSteps: BuildStep[] = [
  {
    name: 'Clean previous build',
    command: 'rm',
    args: ['-rf', 'dist', 'electron/out'],
    condition: () => process.argv.includes('--clean'),
  },
  {
    name: 'Build Rust workspace',
    command: 'cargo',
    args: ['build', config.options.release ? '--release' : ''].filter(Boolean),
    env: {
      RUSTFLAGS: config.options.stripDebugSymbols ? '-C strip=symbols' : '',
    },
  },
  {
    name: 'Build native module',
    command: 'npm',
    args: ['run', 'build'],
    cwd: 'crates/tachikoma-native',
  },
  {
    name: 'Build web frontend',
    command: 'npm',
    args: ['run', 'build'],
    cwd: 'web',
    env: {
      NODE_ENV: config.options.release ? 'production' : 'development',
    },
  },
  {
    name: 'Build Electron main',
    command: 'npm',
    args: ['run', 'build'],
    cwd: 'electron',
  },
  {
    name: 'Package Electron app',
    command: 'npm',
    args: ['run', 'package'],
    cwd: 'electron',
    condition: () => process.argv.includes('--package'),
  },
];

async function runStep(step: BuildStep): Promise<void> {
  if (step.condition && !step.condition()) {
    console.log(`⏭️  Skipping: ${step.name}`);
    return;
  }

  console.log(`\n🔨 ${step.name}...`);

  return new Promise((resolve, reject) => {
    const proc = spawn(step.command, step.args, {
      cwd: step.cwd ? path.join(config.rootDir, step.cwd) : config.rootDir,
      env: { ...process.env, ...step.env },
      stdio: 'inherit',
    });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${step.name} failed with code ${code}`));
      }
    });
  });
}

async function build(): Promise<void> {
  console.log('🚀 Starting Tachikoma build');
  console.log(`   Version: ${config.version}`);
  console.log(`   Release: ${config.options.release}`);
  console.log(`   Platforms: ${config.platforms.filter(p => p.enabled).map(p => `${p.name}-${p.arch}`).join(', ')}`);

  const startTime = Date.now();

  for (const step of buildSteps) {
    await runStep(step);
  }

  const duration = ((Date.now() - startTime) / 1000).toFixed(1);
  console.log(`\n✅ Build completed in ${duration}s`);
}

build().catch((err) => {
  console.error('\n❌ Build failed:', err.message);
  process.exit(1);
});
```

### 4. Makefile Integration

Create/update `Makefile`:

```makefile
.PHONY: all build build-release clean test package dist

# Default target
all: build

# Development build
build:
	@echo "Building Tachikoma (development)..."
	cargo build --workspace
	cd crates/tachikoma-native && npm run build
	cd web && npm run build
	cd electron && npm run build

# Release build
build-release:
	@echo "Building Tachikoma (release)..."
	NODE_ENV=production cargo build --workspace --release
	cd crates/tachikoma-native && npm run build:release
	cd web && npm run build
	cd electron && npm run build

# Clean all build artifacts
clean:
	cargo clean
	rm -rf web/dist
	rm -rf electron/dist
	rm -rf electron/out
	rm -rf dist
	rm -rf .build-cache

# Run all tests
test:
	cargo test --workspace
	cd web && npm test

# Package for distribution
package: build-release
	cd electron && npm run package

# Create distribution packages
dist: build-release
	@echo "Creating distribution packages..."
	cd electron && npm run dist

# Platform-specific builds
dist-mac: build-release
	cd electron && npm run dist -- --mac

dist-win: build-release
	cd electron && npm run dist -- --win

dist-linux: build-release
	cd electron && npm run dist -- --linux

# Install dependencies
install:
	npm install
	cd web && npm install
	cd electron && npm install
	cd crates/tachikoma-native && npm install

# Development server
dev:
	npm run dev

# CI build
ci: lint test build-release

# Linting
lint:
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings
	cd web && npm run lint
```

### 5. Build Prerequisites Documentation

Create `docs/building/prerequisites.md`:

```markdown
# Build Prerequisites

## Required Software

### All Platforms
- Node.js 20.x or later
- npm 10.x or later
- Rust 1.75 or later (via rustup)
- Git

### macOS
- Xcode Command Line Tools
- Apple Developer ID (for signing)

### Windows
- Visual Studio Build Tools 2022
- Windows SDK 10.0
- Code signing certificate (for signing)

### Linux
- GCC 11 or later
- libgtk-3-dev
- libwebkit2gtk-4.0-dev
- libayatana-appindicator3-dev

## Installation

### macOS
```bash
# Install Xcode CLT
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (via nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
```

### Windows
```powershell
# Install Rust
winget install Rustlang.Rustup

# Install Node.js
winget install OpenJS.NodeJS.LTS

# Install Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools
```

### Linux (Ubuntu/Debian)
```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  libgtk-3-dev \
  libwebkit2gtk-4.0-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```
```

---

## Testing Requirements

1. `make build` completes without errors
2. `make build-release` produces optimized artifacts
3. `make package` creates distributable packages
4. Build works on all target platforms
5. Prerequisites documentation is accurate

---

## Related Specs

- Depends on: [007-build-system.md](../phase-00-setup/007-build-system.md), [175-build-config.md](../phase-08-electron/175-build-config.md)
- Next: [492-rust-build.md](492-rust-build.md)
- Related: [494-electron-packaging.md](494-electron-packaging.md)
