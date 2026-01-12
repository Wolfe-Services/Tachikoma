# Spec 174: NAPI-RS Setup

## Phase
8 - Electron Shell

## Spec ID
174

## Status
Planned

## Dependencies
- Spec 173 (Rust Native Modules)

## Estimated Context
~9%

---

## Objective

Configure the NAPI-RS toolchain for building, testing, and packaging Rust native modules that integrate with Electron. This includes cross-compilation setup, CI/CD integration, and platform-specific build configurations.

---

## Acceptance Criteria

- [x] NAPI-RS project scaffolding complete
- [x] Cross-compilation for macOS, Windows, and Linux
- [x] ARM64 and x64 architecture support
- [x] Automated builds in CI/CD pipeline
- [x] Pre-built binaries distribution
- [x] Local development workflow documented
- [x] Version management with npm package
- [x] TypeScript type generation

---

## Implementation Details

### Project Structure

```
native/
  Cargo.toml
  build.rs
  package.json
  npm/
    darwin-arm64/
      package.json
    darwin-x64/
      package.json
    linux-arm64-gnu/
      package.json
    linux-x64-gnu/
      package.json
    win32-arm64-msvc/
      package.json
    win32-x64-msvc/
      package.json
  src/
    lib.rs
    crypto.rs
    compression.rs
    hash.rs
  index.d.ts
  index.js
```

### Main package.json

```json
{
  "name": "@tachikoma/native",
  "version": "0.1.0",
  "main": "index.js",
  "types": "index.d.ts",
  "napi": {
    "name": "tachikoma-native",
    "triples": {
      "defaults": true,
      "additional": [
        "aarch64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "aarch64-pc-windows-msvc"
      ]
    }
  },
  "license": "MIT",
  "devDependencies": {
    "@napi-rs/cli": "^2.17.0",
    "typescript": "^5.3.0"
  },
  "engines": {
    "node": ">= 18"
  },
  "scripts": {
    "artifacts": "napi artifacts",
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "prepublishOnly": "napi prepublish -t npm",
    "test": "cargo test",
    "universal": "napi universal",
    "version": "napi version"
  },
  "optionalDependencies": {
    "@tachikoma/native-darwin-arm64": "0.1.0",
    "@tachikoma/native-darwin-x64": "0.1.0",
    "@tachikoma/native-linux-arm64-gnu": "0.1.0",
    "@tachikoma/native-linux-x64-gnu": "0.1.0",
    "@tachikoma/native-win32-arm64-msvc": "0.1.0",
    "@tachikoma/native-win32-x64-msvc": "0.1.0"
  }
}
```

### Platform-specific package.json (macOS ARM64)

```json
{
  "name": "@tachikoma/native-darwin-arm64",
  "version": "0.1.0",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "main": "tachikoma-native.darwin-arm64.node",
  "files": ["tachikoma-native.darwin-arm64.node"],
  "license": "MIT",
  "engines": {
    "node": ">= 18"
  }
}
```

### Index.js Loader

```javascript
// native/index.js
const { existsSync, readFileSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;

let nativeBinding = null;
let localFileExisted = false;
let loadError = null;

function isMusl() {
  // For linux, determine if system uses musl or glibc
  if (platform !== 'linux') return false;
  try {
    const report = process.report?.getReport();
    if (report?.header?.glibcVersionRuntime) return false;
    const lddOutput = require('child_process')
      .execSync('ldd --version 2>&1 || true')
      .toString();
    return lddOutput.includes('musl');
  } catch {
    return false;
  }
}

switch (platform) {
  case 'android':
    switch (arch) {
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'tachikoma-native.android-arm64.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./tachikoma-native.android-arm64.node');
          } else {
            nativeBinding = require('@tachikoma/native-android-arm64');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm':
        localFileExisted = existsSync(
          join(__dirname, 'tachikoma-native.android-arm-eabi.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./tachikoma-native.android-arm-eabi.node');
          } else {
            nativeBinding = require('@tachikoma/native-android-arm-eabi');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`);
    }
    break;
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'tachikoma-native.win32-x64-msvc.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./tachikoma-native.win32-x64-msvc.node');
          } else {
            nativeBinding = require('@tachikoma/native-win32-x64-msvc');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case 'ia32':
        localFileExisted = existsSync(
          join(__dirname, 'tachikoma-native.win32-ia32-msvc.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./tachikoma-native.win32-ia32-msvc.node');
          } else {
            nativeBinding = require('@tachikoma/native-win32-ia32-msvc');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'tachikoma-native.win32-arm64-msvc.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./tachikoma-native.win32-arm64-msvc.node');
          } else {
            nativeBinding = require('@tachikoma/native-win32-arm64-msvc');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }
    break;
  case 'darwin':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'tachikoma-native.darwin-x64.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./tachikoma-native.darwin-x64.node');
          } else {
            nativeBinding = require('@tachikoma/native-darwin-x64');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'tachikoma-native.darwin-arm64.node')
        );
        try {
          if (localFileExisted) {
            nativeBinding = require('./tachikoma-native.darwin-arm64.node');
          } else {
            nativeBinding = require('@tachikoma/native-darwin-arm64');
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }
    break;
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'tachikoma-native.linux-x64-musl.node')
          );
          try {
            if (localFileExisted) {
              nativeBinding = require('./tachikoma-native.linux-x64-musl.node');
            } else {
              nativeBinding = require('@tachikoma/native-linux-x64-musl');
            }
          } catch (e) {
            loadError = e;
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'tachikoma-native.linux-x64-gnu.node')
          );
          try {
            if (localFileExisted) {
              nativeBinding = require('./tachikoma-native.linux-x64-gnu.node');
            } else {
              nativeBinding = require('@tachikoma/native-linux-x64-gnu');
            }
          } catch (e) {
            loadError = e;
          }
        }
        break;
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'tachikoma-native.linux-arm64-musl.node')
          );
          try {
            if (localFileExisted) {
              nativeBinding = require('./tachikoma-native.linux-arm64-musl.node');
            } else {
              nativeBinding = require('@tachikoma/native-linux-arm64-musl');
            }
          } catch (e) {
            loadError = e;
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'tachikoma-native.linux-arm64-gnu.node')
          );
          try {
            if (localFileExisted) {
              nativeBinding = require('./tachikoma-native.linux-arm64-gnu.node');
            } else {
              nativeBinding = require('@tachikoma/native-linux-arm64-gnu');
            }
          } catch (e) {
            loadError = e;
          }
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }
    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError;
  }
  throw new Error(`Failed to load native binding`);
}

module.exports = nativeBinding;
```

### TypeScript Declarations

```typescript
// native/index.d.ts
/* auto-generated by NAPI-RS */

export const enum HashAlgorithm {
  Sha256 = 'Sha256',
  Sha512 = 'Sha512',
  Sha3_256 = 'Sha3_256',
  Sha3_512 = 'Sha3_512',
  Blake3 = 'Blake3'
}

export const enum CompressionAlgorithm {
  Gzip = 'Gzip',
  Zstd = 'Zstd'
}

export const enum CompressionLevel {
  Fast = 'Fast',
  Default = 'Default',
  Best = 'Best'
}

export interface FileHash {
  path: string
  hash: string
}

export interface EncryptedData {
  ciphertext: Buffer
  nonce: Buffer
  tag: Buffer
}

export interface KeyDerivationResult {
  hash: string
  salt: string
}

/** Initialize the native module */
export function init(): void

/** Get version information */
export function getVersion(): string

/** Hash a string */
export function hashString(data: string, algorithm: HashAlgorithm): string

/** Hash a buffer */
export function hashBuffer(data: Buffer, algorithm: HashAlgorithm): string

/** Hash a file asynchronously */
export function hashFile(path: string, algorithm: HashAlgorithm): Promise<string>

/** Hash multiple files in parallel */
export function hashFiles(paths: Array<string>, algorithm: HashAlgorithm): Promise<Array<FileHash>>

/** Encrypt data using AES-256-GCM */
export function encrypt(data: Buffer, key: Buffer): EncryptedData

/** Decrypt data using AES-256-GCM */
export function decrypt(encrypted: EncryptedData, key: Buffer): Buffer

/** Generate a random encryption key */
export function generateKey(length: number): Buffer

/** Hash a password using Argon2id */
export function hashPassword(password: string): Promise<KeyDerivationResult>

/** Verify a password against a hash */
export function verifyPassword(password: string, hash: string): Promise<boolean>

/** Compress data */
export function compress(data: Buffer, algorithm: CompressionAlgorithm, level?: CompressionLevel | undefined | null): Buffer

/** Decompress data */
export function decompress(data: Buffer, algorithm: CompressionAlgorithm): Buffer

/** Compress a file asynchronously */
export function compressFile(inputPath: string, outputPath: string, algorithm: CompressionAlgorithm): Promise<void>
```

### GitHub Actions CI/CD

```yaml
# .github/workflows/native.yml
name: Build Native Module

on:
  push:
    branches: [main]
    paths:
      - 'native/**'
      - '.github/workflows/native.yml'
  pull_request:
    branches: [main]
    paths:
      - 'native/**'

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
          - target: aarch64-pc-windows-msvc
            os: windows-latest

    runs-on: ${{ matrix.os }}
    name: Build - ${{ matrix.target }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install dependencies (Linux ARM64)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-aarch64-linux-gnu

      - name: Install NAPI-RS CLI
        run: npm install -g @napi-rs/cli

      - name: Build
        working-directory: native
        run: |
          npm install
          napi build --platform --release --target ${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: bindings-${{ matrix.target }}
          path: native/*.node
          if-no-files-found: error

  test:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: bindings-x86_64-unknown-linux-gnu
          path: native

      - name: Run tests
        working-directory: native
        run: |
          npm install
          npm test

  publish:
    needs: [build, test]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: native/artifacts

      - name: Move artifacts
        working-directory: native
        run: |
          npm install -g @napi-rs/cli
          napi artifacts

      - name: Publish
        working-directory: native
        run: |
          npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### Local Development Setup

```bash
#!/bin/bash
# scripts/setup-native.sh

set -e

echo "Setting up native module development environment..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "Node.js not found. Please install Node.js 18 or later."
    exit 1
fi

# Install NAPI-RS CLI
npm install -g @napi-rs/cli

# Navigate to native directory
cd native

# Install npm dependencies
npm install

# Build the native module
npm run build:debug

echo "Native module setup complete!"
echo "Run 'npm run build' for a release build."
```

### Electron Builder Integration

```typescript
// electron-builder.config.ts (partial)
export default {
  // ... other config
  extraResources: [
    {
      from: 'native/tachikoma-native.${platform}-${arch}.node',
      to: 'native/',
    },
  ],
  beforeBuild: async (context) => {
    // Build native module for target platform
    const { execSync } = require('child_process');
    const target = `${context.platform.nodeName}-${context.arch}`;

    console.log(`Building native module for ${target}...`);

    execSync(`cd native && napi build --platform --release --target ${target}`, {
      stdio: 'inherit',
    });
  },
};
```

---

## Testing Requirements

### Unit Tests

```typescript
// native/__tests__/setup.test.ts
import { describe, it, expect, beforeAll } from 'vitest';
import * as native from '../index';

describe('NAPI-RS Setup', () => {
  beforeAll(() => {
    native.init();
  });

  it('should load the native module', () => {
    expect(native).toBeDefined();
    expect(typeof native.init).toBe('function');
  });

  it('should return version', () => {
    const version = native.getVersion();
    expect(version).toMatch(/^\d+\.\d+\.\d+$/);
  });

  it('should export all expected functions', () => {
    expect(typeof native.hashString).toBe('function');
    expect(typeof native.hashBuffer).toBe('function');
    expect(typeof native.hashFile).toBe('function');
    expect(typeof native.encrypt).toBe('function');
    expect(typeof native.decrypt).toBe('function');
    expect(typeof native.compress).toBe('function');
    expect(typeof native.decompress).toBe('function');
  });
});
```

### Cross-Platform Tests

```typescript
// native/__tests__/cross-platform.test.ts
import { describe, it, expect } from 'vitest';
import { platform, arch } from 'os';

describe('Cross-Platform Compatibility', () => {
  it('should detect current platform', () => {
    const supportedPlatforms = ['darwin', 'linux', 'win32'];
    expect(supportedPlatforms).toContain(platform());
  });

  it('should detect current architecture', () => {
    const supportedArchs = ['x64', 'arm64'];
    expect(supportedArchs).toContain(arch());
  });

  it('should load correct binary', () => {
    // The loader should have loaded the correct binary
    const native = require('../index');
    expect(native).toBeDefined();
  });
});
```

---

## Related Specs

- Spec 173: Rust Native Modules
- Spec 175: Build Configuration
- Spec 177: macOS Build
- Spec 178: Windows Build
- Spec 179: Linux Build
