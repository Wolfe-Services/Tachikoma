# Spec 491: Build System Orchestration

## Phase
23 - Build/Package System

## Spec ID
491

## Status
Planned

## Dependencies
- Spec 007 (Build System Configuration)
- Spec 002 (Rust Workspace)
- Spec 003 (Electron Shell)

## Estimated Context
~10%

---

## Objective

Implement the master build orchestration system that coordinates all build steps across Rust, Electron, Svelte, and native modules. This system manages build order, parallelization, caching, and provides a unified interface for development and production builds.

---

## Acceptance Criteria

- [ ] Unified build command orchestrates all subsystems
- [ ] Build steps execute in correct dependency order
- [ ] Parallel execution where dependencies allow
- [ ] Incremental builds for development efficiency
- [ ] Build progress reporting with detailed output
- [ ] Error recovery and partial rebuild support
- [ ] Build artifact management and cleanup
- [ ] Cross-platform build script compatibility
- [ ] Environment-specific build configurations
- [ ] Build timing and performance metrics

---

## Implementation Details

### Build Orchestrator (scripts/build-orchestrator.ts)

```typescript
// scripts/build-orchestrator.ts
import { spawn, SpawnOptions } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

interface BuildStep {
  name: string;
  command: string;
  args: string[];
  cwd?: string;
  env?: Record<string, string>;
  dependsOn?: string[];
  parallel?: boolean;
  skipIf?: () => boolean;
  retries?: number;
}

interface BuildConfig {
  mode: 'development' | 'production' | 'test';
  platform: 'darwin' | 'linux' | 'win32';
  arch: 'x64' | 'arm64';
  verbose: boolean;
  clean: boolean;
  incremental: boolean;
  parallel: boolean;
}

interface BuildResult {
  step: string;
  success: boolean;
  duration: number;
  output: string;
  error?: string;
}

interface BuildMetrics {
  totalDuration: number;
  stepMetrics: Map<string, { duration: number; cached: boolean }>;
  cacheHitRate: number;
}

const COLORS = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  dim: '\x1b[2m',
};

class BuildOrchestrator {
  private config: BuildConfig;
  private results: BuildResult[] = [];
  private startTime: number = 0;
  private cache: Map<string, string> = new Map();
  private cacheDir: string;

  constructor(config: Partial<BuildConfig> = {}) {
    this.config = {
      mode: config.mode ?? 'development',
      platform: config.platform ?? (process.platform as BuildConfig['platform']),
      arch: config.arch ?? (process.arch as BuildConfig['arch']),
      verbose: config.verbose ?? false,
      clean: config.clean ?? false,
      incremental: config.incremental ?? true,
      parallel: config.parallel ?? true,
    };
    this.cacheDir = path.join(os.tmpdir(), 'tachikoma-build-cache');
  }

  private log(level: 'info' | 'warn' | 'error' | 'success' | 'step', message: string): void {
    const timestamp = new Date().toISOString().split('T')[1].slice(0, 8);
    const colors: Record<string, string> = {
      info: COLORS.blue,
      warn: COLORS.yellow,
      error: COLORS.red,
      success: COLORS.green,
      step: COLORS.cyan,
    };
    console.log(`${COLORS.dim}[${timestamp}]${COLORS.reset} ${colors[level]}[${level.toUpperCase()}]${COLORS.reset} ${message}`);
  }

  private async runCommand(step: BuildStep): Promise<BuildResult> {
    const startTime = Date.now();
    const cwd = step.cwd ?? process.cwd();

    return new Promise((resolve) => {
      const env = {
        ...process.env,
        ...step.env,
        FORCE_COLOR: '1',
        NODE_ENV: this.config.mode,
      };

      const options: SpawnOptions = {
        cwd,
        env,
        shell: true,
        stdio: this.config.verbose ? 'inherit' : 'pipe',
      };

      this.log('step', `Starting: ${step.name}`);

      const proc = spawn(step.command, step.args, options);
      let output = '';
      let errorOutput = '';

      if (!this.config.verbose) {
        proc.stdout?.on('data', (data) => {
          output += data.toString();
        });
        proc.stderr?.on('data', (data) => {
          errorOutput += data.toString();
        });
      }

      proc.on('close', (code) => {
        const duration = Date.now() - startTime;
        const success = code === 0;

        if (success) {
          this.log('success', `Completed: ${step.name} (${(duration / 1000).toFixed(2)}s)`);
        } else {
          this.log('error', `Failed: ${step.name} (exit code: ${code})`);
          if (errorOutput) {
            console.error(errorOutput);
          }
        }

        resolve({
          step: step.name,
          success,
          duration,
          output,
          error: errorOutput || undefined,
        });
      });

      proc.on('error', (err) => {
        const duration = Date.now() - startTime;
        this.log('error', `Error in ${step.name}: ${err.message}`);
        resolve({
          step: step.name,
          success: false,
          duration,
          output: '',
          error: err.message,
        });
      });
    });
  }

  private async runWithRetry(step: BuildStep): Promise<BuildResult> {
    const maxRetries = step.retries ?? 0;
    let lastResult: BuildResult | null = null;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      if (attempt > 0) {
        this.log('warn', `Retrying ${step.name} (attempt ${attempt + 1}/${maxRetries + 1})`);
      }
      lastResult = await this.runCommand(step);
      if (lastResult.success) {
        return lastResult;
      }
    }

    return lastResult!;
  }

  private getBuildSteps(): BuildStep[] {
    const isProduction = this.config.mode === 'production';
    const cargoProfile = isProduction ? '--release' : '';

    return [
      {
        name: 'clean',
        command: 'npm',
        args: ['run', 'clean'],
        skipIf: () => !this.config.clean,
      },
      {
        name: 'rust-check',
        command: 'cargo',
        args: ['check', '--workspace', cargoProfile].filter(Boolean),
        parallel: true,
      },
      {
        name: 'rust-build',
        command: 'cargo',
        args: ['build', '--workspace', cargoProfile].filter(Boolean),
        dependsOn: ['rust-check'],
      },
      {
        name: 'native-module',
        command: 'npm',
        args: ['run', 'build'],
        cwd: 'crates/tachikoma-native',
        dependsOn: ['rust-build'],
        env: {
          CARGO_BUILD_TARGET: this.getNativeTarget(),
        },
      },
      {
        name: 'svelte-check',
        command: 'npm',
        args: ['run', 'check'],
        cwd: 'web',
        parallel: true,
      },
      {
        name: 'svelte-build',
        command: 'npm',
        args: ['run', 'build'],
        cwd: 'web',
        dependsOn: ['svelte-check'],
        env: {
          BUILD_MODE: this.config.mode,
        },
      },
      {
        name: 'electron-build',
        command: 'npm',
        args: ['run', 'build'],
        cwd: 'electron',
        dependsOn: ['native-module', 'svelte-build'],
      },
      {
        name: 'verify',
        command: 'npm',
        args: ['run', 'verify'],
        dependsOn: ['electron-build'],
        skipIf: () => this.config.mode !== 'production',
      },
    ];
  }

  private getNativeTarget(): string {
    const targets: Record<string, Record<string, string>> = {
      darwin: {
        x64: 'x86_64-apple-darwin',
        arm64: 'aarch64-apple-darwin',
      },
      linux: {
        x64: 'x86_64-unknown-linux-gnu',
        arm64: 'aarch64-unknown-linux-gnu',
      },
      win32: {
        x64: 'x86_64-pc-windows-msvc',
        arm64: 'aarch64-pc-windows-msvc',
      },
    };
    return targets[this.config.platform]?.[this.config.arch] ?? '';
  }

  private async executeSteps(steps: BuildStep[]): Promise<void> {
    const completed = new Set<string>();
    const pending = [...steps];

    while (pending.length > 0) {
      // Find steps that can run (dependencies satisfied)
      const runnable = pending.filter((step) => {
        if (step.skipIf?.()) {
          completed.add(step.name);
          return false;
        }
        const deps = step.dependsOn ?? [];
        return deps.every((dep) => completed.has(dep));
      });

      if (runnable.length === 0 && pending.length > 0) {
        this.log('error', 'Deadlock detected: no runnable steps');
        throw new Error('Build deadlock');
      }

      // Separate parallel and sequential steps
      const parallelSteps = this.config.parallel
        ? runnable.filter((s) => s.parallel !== false)
        : [];
      const sequentialSteps = runnable.filter(
        (s) => !this.config.parallel || s.parallel === false
      );

      // Run parallel steps concurrently
      if (parallelSteps.length > 0) {
        const results = await Promise.all(
          parallelSteps.map((step) => this.runWithRetry(step))
        );

        for (const result of results) {
          this.results.push(result);
          if (!result.success) {
            throw new Error(`Build step failed: ${result.step}`);
          }
          completed.add(result.step);
          const idx = pending.findIndex((s) => s.name === result.step);
          if (idx !== -1) pending.splice(idx, 1);
        }
      }

      // Run sequential steps one at a time
      for (const step of sequentialSteps) {
        const result = await this.runWithRetry(step);
        this.results.push(result);

        if (!result.success) {
          throw new Error(`Build step failed: ${step.name}`);
        }

        completed.add(step.name);
        const idx = pending.findIndex((s) => s.name === step.name);
        if (idx !== -1) pending.splice(idx, 1);
      }
    }
  }

  async build(): Promise<BuildMetrics> {
    this.startTime = Date.now();
    this.log('info', `Starting ${this.config.mode} build for ${this.config.platform}-${this.config.arch}`);

    try {
      const steps = this.getBuildSteps();
      await this.executeSteps(steps);

      const totalDuration = Date.now() - this.startTime;
      this.log('success', `Build completed in ${(totalDuration / 1000).toFixed(2)}s`);

      return this.getMetrics();
    } catch (error) {
      const totalDuration = Date.now() - this.startTime;
      this.log('error', `Build failed after ${(totalDuration / 1000).toFixed(2)}s`);
      throw error;
    }
  }

  private getMetrics(): BuildMetrics {
    const stepMetrics = new Map<string, { duration: number; cached: boolean }>();

    for (const result of this.results) {
      stepMetrics.set(result.step, {
        duration: result.duration,
        cached: false, // TODO: Implement cache detection
      });
    }

    return {
      totalDuration: Date.now() - this.startTime,
      stepMetrics,
      cacheHitRate: 0,
    };
  }

  printSummary(): void {
    console.log('\n' + '='.repeat(60));
    console.log('BUILD SUMMARY');
    console.log('='.repeat(60));

    for (const result of this.results) {
      const status = result.success ? `${COLORS.green}PASS${COLORS.reset}` : `${COLORS.red}FAIL${COLORS.reset}`;
      const duration = `${(result.duration / 1000).toFixed(2)}s`;
      console.log(`  ${status} ${result.step.padEnd(20)} ${duration}`);
    }

    const total = this.results.reduce((sum, r) => sum + r.duration, 0);
    console.log('='.repeat(60));
    console.log(`Total: ${(total / 1000).toFixed(2)}s`);
    console.log('='.repeat(60) + '\n');
  }
}

// CLI Entry Point
async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const config: Partial<BuildConfig> = {};

  for (const arg of args) {
    if (arg === '--production' || arg === '-p') config.mode = 'production';
    if (arg === '--development' || arg === '-d') config.mode = 'development';
    if (arg === '--verbose' || arg === '-v') config.verbose = true;
    if (arg === '--clean' || arg === '-c') config.clean = true;
    if (arg === '--no-parallel') config.parallel = false;
  }

  const orchestrator = new BuildOrchestrator(config);

  try {
    await orchestrator.build();
    orchestrator.printSummary();
    process.exit(0);
  } catch (error) {
    orchestrator.printSummary();
    console.error(error);
    process.exit(1);
  }
}

main();

export { BuildOrchestrator, BuildConfig, BuildStep, BuildResult };
```

### Build Configuration (build.config.ts)

```typescript
// build.config.ts
import type { BuildConfig } from './scripts/build-orchestrator';

export const defaultConfig: BuildConfig = {
  mode: 'development',
  platform: process.platform as BuildConfig['platform'],
  arch: process.arch as BuildConfig['arch'],
  verbose: false,
  clean: false,
  incremental: true,
  parallel: true,
};

export const productionConfig: BuildConfig = {
  ...defaultConfig,
  mode: 'production',
  clean: true,
  incremental: false,
};

export const testConfig: BuildConfig = {
  ...defaultConfig,
  mode: 'test',
  verbose: true,
};
```

### Package.json Scripts

```json
{
  "scripts": {
    "build": "ts-node scripts/build-orchestrator.ts",
    "build:dev": "ts-node scripts/build-orchestrator.ts --development",
    "build:prod": "ts-node scripts/build-orchestrator.ts --production",
    "build:verbose": "ts-node scripts/build-orchestrator.ts --verbose",
    "build:clean": "ts-node scripts/build-orchestrator.ts --clean",
    "clean": "npm-run-all clean:*",
    "clean:rust": "cargo clean",
    "clean:web": "rimraf web/dist web/.svelte-kit",
    "clean:electron": "rimraf electron/dist electron/out",
    "clean:native": "rimraf crates/tachikoma-native/build",
    "verify": "ts-node scripts/verify-build.ts"
  }
}
```

### Build Verification Script

```typescript
// scripts/verify-build.ts
import * as fs from 'fs';
import * as path from 'path';

interface VerificationResult {
  name: string;
  passed: boolean;
  message: string;
}

function verifyArtifact(artifactPath: string, description: string): VerificationResult {
  const exists = fs.existsSync(artifactPath);
  return {
    name: description,
    passed: exists,
    message: exists ? `Found: ${artifactPath}` : `Missing: ${artifactPath}`,
  };
}

function verifyBuild(): VerificationResult[] {
  const results: VerificationResult[] = [];

  // Verify Rust binaries
  const rustTarget = process.platform === 'win32' ? 'tachikoma.exe' : 'tachikoma';
  results.push(verifyArtifact(
    path.join('target', 'release', rustTarget),
    'Rust CLI binary'
  ));

  // Verify web build
  results.push(verifyArtifact(
    path.join('web', 'dist', 'index.html'),
    'Web build output'
  ));

  // Verify native module
  results.push(verifyArtifact(
    path.join('crates', 'tachikoma-native', 'index.node'),
    'Native Node module'
  ));

  // Verify Electron build
  results.push(verifyArtifact(
    path.join('electron', 'dist', 'main', 'index.js'),
    'Electron main process'
  ));

  return results;
}

const results = verifyBuild();
let allPassed = true;

console.log('\nBuild Verification Results:');
console.log('='.repeat(50));

for (const result of results) {
  const status = result.passed ? '\x1b[32mPASS\x1b[0m' : '\x1b[31mFAIL\x1b[0m';
  console.log(`[${status}] ${result.name}`);
  console.log(`       ${result.message}`);
  if (!result.passed) allPassed = false;
}

console.log('='.repeat(50));
process.exit(allPassed ? 0 : 1);
```

---

## Testing Requirements

### Unit Tests

```typescript
// scripts/__tests__/build-orchestrator.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BuildOrchestrator, BuildConfig } from '../build-orchestrator';

describe('BuildOrchestrator', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create orchestrator with default config', () => {
    const orchestrator = new BuildOrchestrator();
    expect(orchestrator).toBeDefined();
  });

  it('should accept custom configuration', () => {
    const config: Partial<BuildConfig> = {
      mode: 'production',
      verbose: true,
    };
    const orchestrator = new BuildOrchestrator(config);
    expect(orchestrator).toBeDefined();
  });

  it('should detect correct native target for platform', () => {
    const orchestrator = new BuildOrchestrator({
      platform: 'darwin',
      arch: 'arm64',
    });
    // Access private method via reflection for testing
    const target = (orchestrator as any).getNativeTarget();
    expect(target).toBe('aarch64-apple-darwin');
  });

  it('should order steps by dependencies', () => {
    const orchestrator = new BuildOrchestrator();
    const steps = (orchestrator as any).getBuildSteps();

    const electronIdx = steps.findIndex((s: any) => s.name === 'electron-build');
    const svelteIdx = steps.findIndex((s: any) => s.name === 'svelte-build');

    expect(steps[electronIdx].dependsOn).toContain('svelte-build');
  });
});
```

### Integration Tests

```typescript
// scripts/__tests__/build-orchestrator.integration.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { BuildOrchestrator } from '../build-orchestrator';
import * as fs from 'fs';
import * as path from 'path';

describe('BuildOrchestrator Integration', () => {
  const testDir = path.join(__dirname, '.test-build');

  beforeAll(() => {
    fs.mkdirSync(testDir, { recursive: true });
  });

  afterAll(() => {
    fs.rmSync(testDir, { recursive: true, force: true });
  });

  it('should complete a dry-run build', async () => {
    const orchestrator = new BuildOrchestrator({
      mode: 'test',
      verbose: false,
    });

    // Mock the actual build commands
    const metrics = await orchestrator.build();
    expect(metrics.totalDuration).toBeGreaterThan(0);
  });
});
```

---

## Related Specs

- Spec 492: Build Configuration
- Spec 493: Rust Compilation
- Spec 494: Electron Packaging
- Spec 495: Svelte Bundling
- Spec 502: CI Integration
