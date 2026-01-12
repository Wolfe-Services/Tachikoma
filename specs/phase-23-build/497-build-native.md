# Spec 497: Native Modules

## Phase
23 - Build/Package System

## Spec ID
497

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 493 (Rust Compilation)
- Spec 494 (Electron Packaging)

## Estimated Context
~10%

---

## Objective

Implement the native module build system using napi-rs to create Node.js native addons from Rust code. This enables high-performance Rust functionality to be called directly from the Electron main and renderer processes.

---

## Acceptance Criteria

- [ ] napi-rs project structure and configuration
- [ ] Cross-platform native module compilation
- [ ] Electron version-specific builds
- [ ] TypeScript type generation from Rust
- [ ] Native module loading in Electron
- [ ] Platform-specific binary distribution
- [ ] Debug and release build profiles
- [ ] Native module testing infrastructure
- [ ] Prebuild generation for CI
- [ ] Universal binary support for macOS

---

## Implementation Details

### Native Module Crate (crates/tachikoma-native/Cargo.toml)

```toml
# crates/tachikoma-native/Cargo.toml
[package]
name = "tachikoma-native"
version = "1.0.0"
edition = "2021"
license = "MIT"

[lib]
crate-type = ["cdylib"]

[dependencies]
# NAPI-RS
napi = { version = "2.14", default-features = false, features = [
    "napi9",
    "async",
    "serde-json",
    "tokio_rt",
    "error_anyhow",
] }
napi-derive = "2.14"

# Workspace dependencies
tachikoma-common = { path = "../tachikoma-common" }
tachikoma-primitives = { path = "../tachikoma-primitives" }
tachikoma-backends = { path = "../tachikoma-backends" }
tachikoma-loop = { path = "../tachikoma-loop" }

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[build-dependencies]
napi-build = "2.1"

[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
```

### Build Script (crates/tachikoma-native/build.rs)

```rust
// crates/tachikoma-native/build.rs
extern crate napi_build;

fn main() {
    napi_build::setup();
}
```

### Native Module Implementation (crates/tachikoma-native/src/lib.rs)

```rust
// crates/tachikoma-native/src/lib.rs
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

mod file_system;
mod loop_runner;
mod primitives;
mod backends;

// Re-export modules
pub use file_system::*;
pub use loop_runner::*;
pub use primitives::*;
pub use backends::*;

/// Initialize the native module
#[napi]
pub fn init() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Tachikoma native module initialized");
    Ok(())
}

/// Get version information
#[napi(object)]
pub struct VersionInfo {
    pub version: String,
    pub git_hash: String,
    pub build_time: String,
    pub rust_version: String,
}

#[napi]
pub fn get_version() -> VersionInfo {
    VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_hash: env!("GIT_HASH").to_string(),
        build_time: env!("BUILD_TIME").to_string(),
        rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
    }
}

/// File system operations
#[napi(object)]
pub struct FileInfo {
    pub path: String,
    pub size: i64,
    pub is_dir: bool,
    pub modified: i64,
    pub created: i64,
}

#[napi]
pub async fn read_file(path: String) -> Result<Buffer> {
    let content = tokio::fs::read(&path)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to read file: {}", e)))?;

    Ok(Buffer::from(content))
}

#[napi]
pub async fn write_file(path: String, content: Buffer) -> Result<()> {
    tokio::fs::write(&path, content.as_ref())
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to write file: {}", e)))
}

#[napi]
pub async fn list_directory(path: String, recursive: bool) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();

    if recursive {
        collect_entries_recursive(&path, &mut entries).await?;
    } else {
        collect_entries(&path, &mut entries).await?;
    }

    Ok(entries)
}

async fn collect_entries(dir: &str, entries: &mut Vec<FileInfo>) -> Result<()> {
    let mut read_dir = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    while let Some(entry) = read_dir.next_entry().await.map_err(|e| {
        Error::new(Status::GenericFailure, e.to_string())
    })? {
        let metadata = entry.metadata().await.map_err(|e| {
            Error::new(Status::GenericFailure, e.to_string())
        })?;

        entries.push(FileInfo {
            path: entry.path().to_string_lossy().to_string(),
            size: metadata.len() as i64,
            is_dir: metadata.is_dir(),
            modified: metadata.modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64)
                .unwrap_or(0),
            created: metadata.created()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64)
                .unwrap_or(0),
        });
    }

    Ok(())
}

async fn collect_entries_recursive(dir: &str, entries: &mut Vec<FileInfo>) -> Result<()> {
    collect_entries(dir, entries).await?;

    let dirs: Vec<String> = entries
        .iter()
        .filter(|e| e.is_dir)
        .map(|e| e.path.clone())
        .collect();

    for subdir in dirs {
        Box::pin(collect_entries_recursive(&subdir, entries)).await?;
    }

    Ok(())
}

/// Execute shell command
#[napi(object)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[napi]
pub async fn execute_command(
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
    timeout_ms: Option<u32>,
) -> Result<CommandResult> {
    use tokio::process::Command;
    use tokio::time::{timeout, Duration};

    let mut cmd = Command::new(&command);
    cmd.args(&args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(120000) as u64);

    let output = timeout(timeout_duration, cmd.output())
        .await
        .map_err(|_| Error::new(Status::GenericFailure, "Command timed out"))?
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to execute command: {}", e)))?;

    Ok(CommandResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Backend session management
#[napi]
pub struct NativeSession {
    inner: Arc<RwLock<SessionState>>,
}

struct SessionState {
    config: HashMap<String, String>,
    messages: Vec<String>,
}

#[napi]
impl NativeSession {
    #[napi(constructor)]
    pub fn new() -> Self {
        NativeSession {
            inner: Arc::new(RwLock::new(SessionState {
                config: HashMap::new(),
                messages: Vec::new(),
            })),
        }
    }

    #[napi]
    pub async fn configure(&self, key: String, value: String) -> Result<()> {
        let mut state = self.inner.write().await;
        state.config.insert(key, value);
        Ok(())
    }

    #[napi]
    pub async fn get_config(&self, key: String) -> Result<Option<String>> {
        let state = self.inner.read().await;
        Ok(state.config.get(&key).cloned())
    }

    #[napi]
    pub async fn add_message(&self, content: String) -> Result<()> {
        let mut state = self.inner.write().await;
        state.messages.push(content);
        Ok(())
    }

    #[napi]
    pub async fn get_messages(&self) -> Result<Vec<String>> {
        let state = self.inner.read().await;
        Ok(state.messages.clone())
    }

    #[napi]
    pub async fn clear(&self) -> Result<()> {
        let mut state = self.inner.write().await;
        state.config.clear();
        state.messages.clear();
        Ok(())
    }
}
```

### Package.json for Native Module

```json
{
  "name": "@tachikoma/native",
  "version": "1.0.0",
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
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "build:universal": "napi build --platform --release --target universal-apple-darwin",
    "prepublishOnly": "napi prepublish -t npm",
    "test": "cargo test",
    "artifacts": "napi artifacts"
  },
  "devDependencies": {
    "@napi-rs/cli": "^2.18.0"
  },
  "files": [
    "index.js",
    "index.d.ts",
    "*.node"
  ],
  "optionalDependencies": {
    "@tachikoma/native-darwin-x64": "1.0.0",
    "@tachikoma/native-darwin-arm64": "1.0.0",
    "@tachikoma/native-linux-x64-gnu": "1.0.0",
    "@tachikoma/native-linux-arm64-gnu": "1.0.0",
    "@tachikoma/native-win32-x64-msvc": "1.0.0",
    "@tachikoma/native-win32-arm64-msvc": "1.0.0"
  }
}
```

### TypeScript Loader (crates/tachikoma-native/index.js)

```javascript
// crates/tachikoma-native/index.js
const { existsSync, readFileSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;

let nativeBinding = null;
let localFileExisted = false;
let loadError = null;

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process')
        .execSync('which ldd')
        .toString()
        .trim();
      return readFileSync(lddPath, 'utf8').includes('musl');
    } catch {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

switch (platform) {
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
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
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

### Build Script for Native Module

```typescript
// scripts/build-native.ts
import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

interface NativeBuildConfig {
  profile: 'debug' | 'release';
  target?: string;
  universal?: boolean;
}

async function buildNativeModule(config: NativeBuildConfig): Promise<void> {
  const cwd = path.join(process.cwd(), 'crates', 'tachikoma-native');

  console.log(`Building native module (${config.profile})...`);

  const args = ['build', '--platform'];

  if (config.profile === 'release') {
    args.push('--release');
  }

  if (config.target) {
    args.push('--target', config.target);
  }

  if (config.universal && process.platform === 'darwin') {
    args.push('--target', 'universal-apple-darwin');
  }

  return new Promise((resolve, reject) => {
    const proc = spawn('napi', args, {
      cwd,
      stdio: 'inherit',
      shell: true,
    });

    proc.on('close', (code) => {
      if (code === 0) {
        console.log('Native module build complete');
        resolve();
      } else {
        reject(new Error(`Native module build failed with code ${code}`));
      }
    });

    proc.on('error', (err) => {
      reject(err);
    });
  });
}

export { buildNativeModule, NativeBuildConfig };
```

---

## Testing Requirements

### Rust Tests

```rust
// crates/tachikoma-native/src/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version = get_version();
        assert!(!version.version.is_empty());
    }

    #[tokio::test]
    async fn test_native_session() {
        let session = NativeSession::new();

        session.configure("key".to_string(), "value".to_string()).await.unwrap();
        let value = session.get_config("key".to_string()).await.unwrap();
        assert_eq!(value, Some("value".to_string()));

        session.add_message("test message".to_string()).await.unwrap();
        let messages = session.get_messages().await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "test message");

        session.clear().await.unwrap();
        let messages = session.get_messages().await.unwrap();
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn test_file_operations() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        let content = read_file(path.clone()).await.unwrap();
        assert_eq!(content.as_ref(), b"test content");
    }
}
```

### TypeScript Tests

```typescript
// crates/tachikoma-native/__tests__/native.test.ts
import { describe, it, expect, beforeAll } from 'vitest';
import * as native from '../index';

describe('Native Module', () => {
  beforeAll(() => {
    native.init();
  });

  it('should get version info', () => {
    const version = native.getVersion();
    expect(version.version).toBeDefined();
    expect(version.gitHash).toBeDefined();
  });

  it('should read files', async () => {
    const content = await native.readFile(__filename);
    expect(content).toBeInstanceOf(Buffer);
    expect(content.length).toBeGreaterThan(0);
  });

  it('should list directory', async () => {
    const entries = await native.listDirectory(__dirname, false);
    expect(Array.isArray(entries)).toBe(true);
    expect(entries.length).toBeGreaterThan(0);
  });

  it('should execute commands', async () => {
    const result = await native.executeCommand('echo', ['hello'], undefined, 5000);
    expect(result.exitCode).toBe(0);
    expect(result.stdout.trim()).toBe('hello');
  });

  it('should manage sessions', async () => {
    const session = new native.NativeSession();

    await session.configure('testKey', 'testValue');
    const value = await session.getConfig('testKey');
    expect(value).toBe('testValue');

    await session.addMessage('test message');
    const messages = await session.getMessages();
    expect(messages).toContain('test message');

    await session.clear();
    const clearedMessages = await session.getMessages();
    expect(clearedMessages.length).toBe(0);
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 493: Rust Compilation
- Spec 494: Electron Packaging
- Spec 005: IPC Bridge
