# Spec 492: Build Configuration

## Phase
23 - Build/Package System

## Spec ID
492

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 007 (Build System Configuration)

## Estimated Context
~8%

---

## Objective

Define and implement the build configuration system that manages environment-specific settings, feature flags, optimization levels, and platform-specific configurations across all build targets.

---

## Acceptance Criteria

- [ ] Centralized configuration file for all build settings
- [ ] Environment-specific config overrides (dev/staging/prod)
- [ ] Feature flags system for conditional compilation
- [ ] Platform-specific configuration sections
- [ ] Build profile definitions (debug/release/test)
- [ ] Configuration validation with clear error messages
- [ ] Environment variable interpolation
- [ ] Configuration inheritance and merging
- [ ] Type-safe configuration access in TypeScript
- [ ] Documentation generation from config schema

---

## Implementation Details

### Main Configuration File (tachikoma.build.yaml)

```yaml
# tachikoma.build.yaml - Master Build Configuration

version: "1.0"

# Project metadata
project:
  name: tachikoma
  version: "${npm_package_version}"
  description: "AI-Powered Development Assistant"
  author: "Tachikoma Team"
  license: "MIT"
  repository: "https://github.com/tachikoma/tachikoma"

# Build profiles
profiles:
  development:
    optimize: false
    sourcemaps: true
    debug: true
    minify: false
    env:
      NODE_ENV: development
      RUST_LOG: debug
      VITE_DEV_MODE: "true"

  production:
    optimize: true
    sourcemaps: false
    debug: false
    minify: true
    env:
      NODE_ENV: production
      RUST_LOG: info
      VITE_DEV_MODE: "false"

  test:
    optimize: false
    sourcemaps: true
    debug: true
    minify: false
    env:
      NODE_ENV: test
      RUST_LOG: trace
      VITE_DEV_MODE: "true"

# Feature flags
features:
  telemetry:
    default: true
    description: "Enable anonymous telemetry"
    envVar: TACHIKOMA_TELEMETRY

  autoUpdate:
    default: true
    description: "Enable automatic updates"
    envVar: TACHIKOMA_AUTO_UPDATE

  experimental:
    default: false
    description: "Enable experimental features"
    envVar: TACHIKOMA_EXPERIMENTAL

  debugMenu:
    default: false
    description: "Show debug menu in UI"
    profiles: [development, test]

# Platform configurations
platforms:
  darwin:
    identifier: "com.tachikoma.app"
    category: "public.app-category.developer-tools"
    entitlements:
      - com.apple.security.cs.allow-unsigned-executable-memory
      - com.apple.security.cs.disable-library-validation
    codesign:
      identity: "${APPLE_SIGNING_IDENTITY}"
      notarize: true
    architectures: [x64, arm64]
    universalBinary: true

  linux:
    categories:
      - Development
      - IDE
    desktop:
      name: Tachikoma
      comment: AI-Powered Development Assistant
      terminal: false
    formats: [deb, rpm, AppImage, snap]
    architectures: [x64, arm64]

  win32:
    appId: "com.tachikoma.app"
    publisherName: "Tachikoma Team"
    certificateFile: "${WINDOWS_CERTIFICATE}"
    formats: [nsis, portable, msi]
    architectures: [x64, arm64]

# Rust build configuration
rust:
  profile:
    development:
      opt-level: 0
      debug: true
      lto: false

    production:
      opt-level: 3
      debug: false
      lto: "thin"
      codegen-units: 1
      panic: "abort"

  targets:
    darwin-x64: x86_64-apple-darwin
    darwin-arm64: aarch64-apple-darwin
    linux-x64: x86_64-unknown-linux-gnu
    linux-arm64: aarch64-unknown-linux-gnu
    win32-x64: x86_64-pc-windows-msvc
    win32-arm64: aarch64-pc-windows-msvc

  features:
    default: [logging, metrics]
    production: [logging, metrics, telemetry]
    development: [logging, metrics, debug-assertions]

# Svelte/Vite build configuration
web:
  outDir: dist
  assetsDir: assets
  publicDir: public

  build:
    development:
      target: esnext
      minify: false
      sourcemap: true

    production:
      target: es2020
      minify: esbuild
      sourcemap: false
      rollupOptions:
        output:
          manualChunks:
            vendor: [svelte, @sveltejs/kit]

  define:
    __APP_VERSION__: "${npm_package_version}"
    __GIT_COMMIT__: "${GIT_COMMIT}"
    __BUILD_TIME__: "${BUILD_TIME}"

# Electron build configuration
electron:
  appId: "com.tachikoma.app"
  productName: Tachikoma
  copyright: "Copyright (c) 2024 Tachikoma Team"

  directories:
    output: release
    buildResources: resources

  files:
    - dist/**/*
    - package.json
    - "!**/*.map"

  extraResources:
    - from: ../target/release
      to: bin
      filter:
        - tachikoma*
        - "!*.pdb"

# Optimization settings
optimization:
  bundleAnalysis: false
  treeshaking: true
  deadCodeElimination: true
  compressionLevel: 9

# Cache configuration
cache:
  enabled: true
  directory: .build-cache
  maxAge: 604800 # 7 days in seconds
  hashInputs:
    - Cargo.lock
    - package-lock.json
    - web/package-lock.json
```

### Configuration Parser (scripts/config/parser.ts)

```typescript
// scripts/config/parser.ts
import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'yaml';

interface BuildProfile {
  optimize: boolean;
  sourcemaps: boolean;
  debug: boolean;
  minify: boolean;
  env: Record<string, string>;
}

interface FeatureFlag {
  default: boolean;
  description: string;
  envVar?: string;
  profiles?: string[];
}

interface PlatformConfig {
  identifier?: string;
  architectures: string[];
  [key: string]: unknown;
}

interface BuildConfig {
  version: string;
  project: {
    name: string;
    version: string;
    description: string;
    author: string;
    license: string;
    repository: string;
  };
  profiles: Record<string, BuildProfile>;
  features: Record<string, FeatureFlag>;
  platforms: Record<string, PlatformConfig>;
  rust: Record<string, unknown>;
  web: Record<string, unknown>;
  electron: Record<string, unknown>;
  optimization: Record<string, unknown>;
  cache: Record<string, unknown>;
}

class ConfigParser {
  private config: BuildConfig;
  private profile: string;
  private platform: string;

  constructor(configPath: string = 'tachikoma.build.yaml') {
    const rawConfig = fs.readFileSync(configPath, 'utf-8');
    const parsed = yaml.parse(rawConfig);
    this.config = this.interpolateEnvVars(parsed);
    this.profile = process.env.BUILD_PROFILE ?? 'development';
    this.platform = process.platform;
  }

  private interpolateEnvVars(obj: any): any {
    if (typeof obj === 'string') {
      return obj.replace(/\$\{([^}]+)\}/g, (match, varName) => {
        return process.env[varName] ?? match;
      });
    }
    if (Array.isArray(obj)) {
      return obj.map((item) => this.interpolateEnvVars(item));
    }
    if (typeof obj === 'object' && obj !== null) {
      const result: Record<string, unknown> = {};
      for (const [key, value] of Object.entries(obj)) {
        result[key] = this.interpolateEnvVars(value);
      }
      return result;
    }
    return obj;
  }

  getProfile(): BuildProfile {
    const profile = this.config.profiles[this.profile];
    if (!profile) {
      throw new Error(`Unknown build profile: ${this.profile}`);
    }
    return profile;
  }

  getPlatformConfig(): PlatformConfig {
    const platform = this.config.platforms[this.platform];
    if (!platform) {
      throw new Error(`Unsupported platform: ${this.platform}`);
    }
    return platform;
  }

  isFeatureEnabled(featureName: string): boolean {
    const feature = this.config.features[featureName];
    if (!feature) return false;

    // Check environment variable override
    if (feature.envVar && process.env[feature.envVar] !== undefined) {
      return process.env[feature.envVar] === 'true';
    }

    // Check profile-specific enablement
    if (feature.profiles && !feature.profiles.includes(this.profile)) {
      return false;
    }

    return feature.default;
  }

  getEnabledFeatures(): string[] {
    return Object.keys(this.config.features).filter((name) =>
      this.isFeatureEnabled(name)
    );
  }

  getRustConfig(): Record<string, unknown> {
    const base = this.config.rust;
    const profileConfig = base.profile?.[this.profile] ?? {};
    return { ...base, profile: profileConfig };
  }

  getWebConfig(): Record<string, unknown> {
    const base = this.config.web;
    const buildConfig = base.build?.[this.profile] ?? {};
    return { ...base, build: buildConfig };
  }

  getElectronConfig(): Record<string, unknown> {
    return this.config.electron;
  }

  getFullConfig(): BuildConfig {
    return this.config;
  }

  validate(): string[] {
    const errors: string[] = [];

    // Validate required fields
    if (!this.config.version) {
      errors.push('Missing required field: version');
    }
    if (!this.config.project?.name) {
      errors.push('Missing required field: project.name');
    }

    // Validate profiles
    for (const [name, profile] of Object.entries(this.config.profiles)) {
      if (typeof profile.optimize !== 'boolean') {
        errors.push(`Profile ${name}: optimize must be a boolean`);
      }
    }

    // Validate platform configs
    for (const [platform, config] of Object.entries(this.config.platforms)) {
      if (!config.architectures || config.architectures.length === 0) {
        errors.push(`Platform ${platform}: architectures must be specified`);
      }
    }

    return errors;
  }

  exportForRust(): string {
    const profile = this.getProfile();
    const features = this.getEnabledFeatures();
    const rustConfig = this.getRustConfig();

    return `
// Auto-generated build configuration
// Do not edit manually

pub const PROFILE: &str = "${this.profile}";
pub const DEBUG: bool = ${profile.debug};
pub const OPTIMIZE: bool = ${profile.optimize};

pub const FEATURES: &[&str] = &[${features.map((f) => `"${f}"`).join(', ')}];

pub fn is_feature_enabled(feature: &str) -> bool {
    FEATURES.contains(&feature)
}
`.trim();
  }

  exportForTypeScript(): string {
    const profile = this.getProfile();
    const features = this.getEnabledFeatures();

    return `
// Auto-generated build configuration
// Do not edit manually

export const BUILD_CONFIG = {
  profile: '${this.profile}',
  debug: ${profile.debug},
  optimize: ${profile.optimize},
  sourcemaps: ${profile.sourcemaps},
  minify: ${profile.minify},
  features: ${JSON.stringify(features)},
  platform: '${this.platform}',
  env: ${JSON.stringify(profile.env)},
} as const;

export type BuildProfile = typeof BUILD_CONFIG['profile'];
export type Feature = typeof BUILD_CONFIG['features'][number];

export function isFeatureEnabled(feature: string): boolean {
  return BUILD_CONFIG.features.includes(feature as Feature);
}
`.trim();
  }
}

export { ConfigParser, BuildConfig, BuildProfile, FeatureFlag };
```

### Configuration Validator (scripts/config/validator.ts)

```typescript
// scripts/config/validator.ts
import Ajv, { JSONSchemaType } from 'ajv';

const configSchema = {
  type: 'object',
  required: ['version', 'project', 'profiles'],
  properties: {
    version: { type: 'string' },
    project: {
      type: 'object',
      required: ['name', 'version'],
      properties: {
        name: { type: 'string', minLength: 1 },
        version: { type: 'string' },
        description: { type: 'string' },
        author: { type: 'string' },
        license: { type: 'string' },
        repository: { type: 'string' },
      },
    },
    profiles: {
      type: 'object',
      additionalProperties: {
        type: 'object',
        required: ['optimize', 'debug'],
        properties: {
          optimize: { type: 'boolean' },
          sourcemaps: { type: 'boolean' },
          debug: { type: 'boolean' },
          minify: { type: 'boolean' },
          env: {
            type: 'object',
            additionalProperties: { type: 'string' },
          },
        },
      },
    },
    features: {
      type: 'object',
      additionalProperties: {
        type: 'object',
        required: ['default'],
        properties: {
          default: { type: 'boolean' },
          description: { type: 'string' },
          envVar: { type: 'string' },
          profiles: {
            type: 'array',
            items: { type: 'string' },
          },
        },
      },
    },
  },
};

class ConfigValidator {
  private ajv: Ajv;

  constructor() {
    this.ajv = new Ajv({ allErrors: true });
  }

  validate(config: unknown): { valid: boolean; errors: string[] } {
    const validate = this.ajv.compile(configSchema);
    const valid = validate(config);

    return {
      valid: valid as boolean,
      errors: validate.errors?.map((e) => `${e.instancePath}: ${e.message}`) ?? [],
    };
  }
}

export { ConfigValidator, configSchema };
```

---

## Testing Requirements

### Unit Tests

```typescript
// scripts/config/__tests__/parser.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ConfigParser } from '../parser';

describe('ConfigParser', () => {
  beforeEach(() => {
    vi.stubEnv('BUILD_PROFILE', 'development');
  });

  it('should parse valid configuration', () => {
    const parser = new ConfigParser('test-config.yaml');
    expect(parser.getFullConfig()).toBeDefined();
  });

  it('should interpolate environment variables', () => {
    vi.stubEnv('TEST_VAR', 'test-value');
    const parser = new ConfigParser('test-config.yaml');
    const config = parser.getFullConfig();
    expect(config.project.version).not.toContain('${');
  });

  it('should return correct profile', () => {
    const parser = new ConfigParser('test-config.yaml');
    const profile = parser.getProfile();
    expect(profile.debug).toBe(true);
  });

  it('should check feature flags', () => {
    const parser = new ConfigParser('test-config.yaml');
    expect(typeof parser.isFeatureEnabled('telemetry')).toBe('boolean');
  });

  it('should export TypeScript config', () => {
    const parser = new ConfigParser('test-config.yaml');
    const ts = parser.exportForTypeScript();
    expect(ts).toContain('export const BUILD_CONFIG');
  });

  it('should validate configuration', () => {
    const parser = new ConfigParser('test-config.yaml');
    const errors = parser.validate();
    expect(errors).toHaveLength(0);
  });
});
```

### Validation Tests

```typescript
// scripts/config/__tests__/validator.test.ts
import { describe, it, expect } from 'vitest';
import { ConfigValidator } from '../validator';

describe('ConfigValidator', () => {
  const validator = new ConfigValidator();

  it('should validate correct config', () => {
    const config = {
      version: '1.0',
      project: { name: 'test', version: '1.0.0' },
      profiles: {
        development: { optimize: false, debug: true },
      },
    };

    const result = validator.validate(config);
    expect(result.valid).toBe(true);
  });

  it('should reject missing required fields', () => {
    const config = {
      version: '1.0',
    };

    const result = validator.validate(config);
    expect(result.valid).toBe(false);
    expect(result.errors.length).toBeGreaterThan(0);
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 493: Rust Compilation
- Spec 494: Electron Packaging
- Spec 504: Version Management
