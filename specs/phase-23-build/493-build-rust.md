# Spec 493: Rust Compilation

## Phase
23 - Build/Package System

## Spec ID
493

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 492 (Build Configuration)
- Spec 002 (Rust Workspace)

## Estimated Context
~10%

---

## Objective

Implement the Rust compilation pipeline that handles workspace builds, cross-compilation, optimization, and binary distribution. This includes managing release profiles, target architectures, and integration with the native module system.

---

## Acceptance Criteria

- [ ] Workspace-wide compilation with correct dependency order
- [ ] Release profile optimization (LTO, codegen-units, panic=abort)
- [ ] Cross-compilation for all target platforms
- [ ] Universal binary generation for macOS
- [ ] Cargo feature flag management
- [ ] Binary stripping and size optimization
- [ ] Build artifact caching with sccache
- [ ] Parallel compilation configuration
- [ ] Build metadata embedding (version, git hash)
- [ ] Integration test compilation

---

## Implementation Details

### Rust Build Script (scripts/build-rust.ts)

```typescript
// scripts/build-rust.ts
import { spawn, SpawnOptions } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

interface RustBuildConfig {
  profile: 'debug' | 'release' | 'test';
  target?: string;
  features: string[];
  workspace: boolean;
  packages?: string[];
  jobs?: number;
  verbose: boolean;
  sccache: boolean;
}

interface RustTarget {
  triple: string;
  platform: string;
  arch: string;
  tier: 1 | 2 | 3;
}

const SUPPORTED_TARGETS: RustTarget[] = [
  { triple: 'x86_64-apple-darwin', platform: 'darwin', arch: 'x64', tier: 1 },
  { triple: 'aarch64-apple-darwin', platform: 'darwin', arch: 'arm64', tier: 1 },
  { triple: 'x86_64-unknown-linux-gnu', platform: 'linux', arch: 'x64', tier: 1 },
  { triple: 'aarch64-unknown-linux-gnu', platform: 'linux', arch: 'arm64', tier: 1 },
  { triple: 'x86_64-pc-windows-msvc', platform: 'win32', arch: 'x64', tier: 1 },
  { triple: 'aarch64-pc-windows-msvc', platform: 'win32', arch: 'arm64', tier: 2 },
];

class RustBuilder {
  private config: RustBuildConfig;
  private projectRoot: string;

  constructor(config: Partial<RustBuildConfig> = {}) {
    this.config = {
      profile: config.profile ?? 'release',
      target: config.target,
      features: config.features ?? [],
      workspace: config.workspace ?? true,
      packages: config.packages,
      jobs: config.jobs,
      verbose: config.verbose ?? false,
      sccache: config.sccache ?? true,
    };
    this.projectRoot = process.cwd();
  }

  private getCargoArgs(): string[] {
    const args = ['build'];

    // Profile
    if (this.config.profile === 'release') {
      args.push('--release');
    }

    // Target
    if (this.config.target) {
      args.push('--target', this.config.target);
    }

    // Workspace or specific packages
    if (this.config.workspace && !this.config.packages) {
      args.push('--workspace');
    } else if (this.config.packages) {
      for (const pkg of this.config.packages) {
        args.push('-p', pkg);
      }
    }

    // Features
    if (this.config.features.length > 0) {
      args.push('--features', this.config.features.join(','));
    }

    // Jobs
    if (this.config.jobs) {
      args.push('-j', this.config.jobs.toString());
    }

    // Verbose
    if (this.config.verbose) {
      args.push('-v');
    }

    return args;
  }

  private getEnv(): Record<string, string> {
    const env: Record<string, string> = {
      ...process.env as Record<string, string>,
      CARGO_TERM_COLOR: 'always',
    };

    // Enable sccache if available
    if (this.config.sccache) {
      env.RUSTC_WRAPPER = 'sccache';
    }

    // Set optimization flags for release
    if (this.config.profile === 'release') {
      env.CARGO_PROFILE_RELEASE_LTO = 'thin';
      env.CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '1';
      env.CARGO_PROFILE_RELEASE_OPT_LEVEL = '3';
    }

    return env;
  }

  async build(): Promise<void> {
    const args = this.getCargoArgs();
    const env = this.getEnv();

    console.log(`[RUST] Building with: cargo ${args.join(' ')}`);

    return new Promise((resolve, reject) => {
      const proc = spawn('cargo', args, {
        cwd: this.projectRoot,
        env,
        stdio: 'inherit',
        shell: true,
      });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error(`Cargo build failed with code ${code}`));
        }
      });

      proc.on('error', (err) => {
        reject(err);
      });
    });
  }

  async buildForTarget(target: RustTarget): Promise<void> {
    console.log(`[RUST] Building for ${target.triple}`);

    // Install target if needed
    await this.installTarget(target.triple);

    this.config.target = target.triple;
    await this.build();
  }

  private async installTarget(target: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const proc = spawn('rustup', ['target', 'add', target], {
        stdio: 'inherit',
        shell: true,
      });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error(`Failed to install target ${target}`));
        }
      });
    });
  }

  async buildUniversalBinary(binaryName: string): Promise<void> {
    if (process.platform !== 'darwin') {
      throw new Error('Universal binaries are only supported on macOS');
    }

    const x64Target = 'x86_64-apple-darwin';
    const arm64Target = 'aarch64-apple-darwin';

    // Build for both architectures
    await this.buildForTarget(
      SUPPORTED_TARGETS.find((t) => t.triple === x64Target)!
    );
    await this.buildForTarget(
      SUPPORTED_TARGETS.find((t) => t.triple === arm64Target)!
    );

    // Create universal binary with lipo
    const profile = this.config.profile === 'release' ? 'release' : 'debug';
    const x64Path = path.join('target', x64Target, profile, binaryName);
    const arm64Path = path.join('target', arm64Target, profile, binaryName);
    const universalPath = path.join('target', 'universal', profile, binaryName);

    // Ensure output directory exists
    fs.mkdirSync(path.dirname(universalPath), { recursive: true });

    return new Promise((resolve, reject) => {
      const proc = spawn('lipo', [
        '-create',
        x64Path,
        arm64Path,
        '-output',
        universalPath,
      ], {
        stdio: 'inherit',
      });

      proc.on('close', (code) => {
        if (code === 0) {
          console.log(`[RUST] Created universal binary: ${universalPath}`);
          resolve();
        } else {
          reject(new Error('Failed to create universal binary'));
        }
      });
    });
  }

  async stripBinary(binaryPath: string): Promise<void> {
    const stripCmd = process.platform === 'darwin' ? 'strip' : 'strip';
    const stripArgs = process.platform === 'darwin'
      ? ['-x', binaryPath]
      : ['--strip-all', binaryPath];

    return new Promise((resolve, reject) => {
      const proc = spawn(stripCmd, stripArgs, { stdio: 'inherit' });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error('Failed to strip binary'));
        }
      });
    });
  }

  getOutputPath(binaryName: string): string {
    const profile = this.config.profile === 'release' ? 'release' : 'debug';
    const target = this.config.target;
    const ext = process.platform === 'win32' ? '.exe' : '';

    if (target) {
      return path.join('target', target, profile, binaryName + ext);
    }
    return path.join('target', profile, binaryName + ext);
  }

  static getSupportedTargets(): RustTarget[] {
    return SUPPORTED_TARGETS;
  }
}

export { RustBuilder, RustBuildConfig, RustTarget };
```

### Cargo Configuration (Cargo.toml profiles)

```toml
# Cargo.toml - Root workspace configuration

[workspace]
resolver = "2"
members = [
    "crates/tachikoma-cli",
    "crates/tachikoma-common",
    "crates/tachikoma-primitives",
    "crates/tachikoma-backends",
    "crates/tachikoma-forge",
    "crates/tachikoma-specs",
    "crates/tachikoma-loop",
    "crates/tachikoma-database",
    "crates/tachikoma-native",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
rust-version = "1.75"
authors = ["Tachikoma Team"]
license = "MIT"
repository = "https://github.com/tachikoma/tachikoma"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# HTTP
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging/tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# CLI
clap = { version = "4.4", features = ["derive", "env"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }

# Native module
napi = { version = "2.14", features = ["async", "serde-json"] }
napi-derive = "2.14"

# Testing
mockall = "0.12"

# Profile for development builds
[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"
lto = false
incremental = true
codegen-units = 256

# Profile for release builds
[profile.release]
opt-level = 3
debug = false
split-debuginfo = "off"
lto = "thin"
incremental = false
codegen-units = 1
panic = "abort"
strip = "symbols"

# Profile for CI/test builds
[profile.ci]
inherits = "release"
lto = false
codegen-units = 16

# Profile for profiling
[profile.profiling]
inherits = "release"
debug = true
strip = "none"

# Profile for smaller binary size
[profile.release-small]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = "symbols"
```

### Build Script for Version Embedding

```rust
// build.rs - Root build script
use std::process::Command;

fn main() {
    // Re-run if build script changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Get git information
    let git_hash = get_git_hash();
    let git_branch = get_git_branch();
    let git_dirty = is_git_dirty();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    println!("cargo:rustc-env=GIT_DIRTY={}", git_dirty);

    // Build timestamp
    let build_time = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    // Build profile
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

    // Target triple
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);
}

fn get_git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn get_git_branch() -> String {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn is_git_dirty() -> bool {
    Command::new("git")
        .args(["diff", "--quiet", "HEAD"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(false)
}
```

### Version Information Module

```rust
// crates/tachikoma-common/src/version.rs

/// Build-time version information
pub struct BuildInfo {
    pub version: &'static str,
    pub git_hash: &'static str,
    pub git_branch: &'static str,
    pub git_dirty: bool,
    pub build_time: &'static str,
    pub build_profile: &'static str,
    pub build_target: &'static str,
    pub rust_version: &'static str,
}

impl BuildInfo {
    pub const fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_hash: env!("GIT_HASH"),
            git_branch: env!("GIT_BRANCH"),
            git_dirty: matches!(env!("GIT_DIRTY"), "true"),
            build_time: env!("BUILD_TIME"),
            build_profile: env!("BUILD_PROFILE"),
            build_target: env!("BUILD_TARGET"),
            rust_version: env!("CARGO_PKG_RUST_VERSION"),
        }
    }

    pub fn version_string(&self) -> String {
        let dirty = if self.git_dirty { "-dirty" } else { "" };
        format!(
            "{} ({}{}) built {} [{}]",
            self.version,
            self.git_hash,
            dirty,
            self.build_time,
            self.build_profile
        )
    }

    pub fn user_agent(&self) -> String {
        format!("Tachikoma/{} ({})", self.version, self.build_target)
    }
}

pub static BUILD_INFO: BuildInfo = BuildInfo::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info() {
        let info = BuildInfo::new();
        assert!(!info.version.is_empty());
        assert!(!info.git_hash.is_empty());
    }

    #[test]
    fn test_version_string() {
        let info = BuildInfo::new();
        let version = info.version_string();
        assert!(version.contains(info.version));
    }
}
```

### Cross-Compilation Script

```bash
#!/usr/bin/env bash
# scripts/cross-compile.sh

set -euo pipefail

TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-pc-windows-msvc"
)

PROFILE="${1:-release}"

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."

    # Add target if not installed
    rustup target add "$target" 2>/dev/null || true

    # Build
    if [[ "$PROFILE" == "release" ]]; then
        cargo build --release --target "$target" --workspace
    else
        cargo build --target "$target" --workspace
    fi

    echo "Built for $target successfully"
done

# Create universal binary on macOS
if [[ "$(uname)" == "Darwin" ]]; then
    echo "Creating universal binary..."
    mkdir -p target/universal/$PROFILE

    lipo -create \
        "target/x86_64-apple-darwin/$PROFILE/tachikoma" \
        "target/aarch64-apple-darwin/$PROFILE/tachikoma" \
        -output "target/universal/$PROFILE/tachikoma"

    echo "Universal binary created"
fi
```

---

## Testing Requirements

### Unit Tests

```typescript
// scripts/__tests__/build-rust.test.ts
import { describe, it, expect, vi } from 'vitest';
import { RustBuilder, RustTarget } from '../build-rust';

describe('RustBuilder', () => {
  it('should create builder with default config', () => {
    const builder = new RustBuilder();
    expect(builder).toBeDefined();
  });

  it('should generate correct cargo args for release', () => {
    const builder = new RustBuilder({ profile: 'release' });
    const args = (builder as any).getCargoArgs();
    expect(args).toContain('--release');
    expect(args).toContain('--workspace');
  });

  it('should include features in cargo args', () => {
    const builder = new RustBuilder({
      features: ['telemetry', 'metrics'],
    });
    const args = (builder as any).getCargoArgs();
    expect(args).toContain('--features');
    expect(args).toContain('telemetry,metrics');
  });

  it('should return supported targets', () => {
    const targets = RustBuilder.getSupportedTargets();
    expect(targets.length).toBeGreaterThan(0);
    expect(targets.every((t: RustTarget) => t.triple && t.platform)).toBe(true);
  });

  it('should get correct output path', () => {
    const builder = new RustBuilder({ profile: 'release' });
    const path = builder.getOutputPath('tachikoma');
    expect(path).toContain('release');
    expect(path).toContain('tachikoma');
  });
});
```

### Build Tests

```rust
// crates/tachikoma-common/src/version_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info_not_empty() {
        assert!(!BUILD_INFO.version.is_empty());
        assert!(!BUILD_INFO.git_hash.is_empty());
        assert!(!BUILD_INFO.build_time.is_empty());
    }

    #[test]
    fn test_version_string_format() {
        let version = BUILD_INFO.version_string();
        assert!(version.contains(BUILD_INFO.version));
        assert!(version.contains("built"));
    }

    #[test]
    fn test_user_agent_format() {
        let ua = BUILD_INFO.user_agent();
        assert!(ua.starts_with("Tachikoma/"));
    }
}
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 492: Build Configuration
- Spec 497: Native Modules
- Spec 499: macOS Packaging
- Spec 500: Windows Installer
- Spec 501: Linux Packages
