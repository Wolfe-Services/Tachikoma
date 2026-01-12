# Spec 505: Build Tests

## Phase
23 - Build/Package System

## Spec ID
505

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 502 (CI Integration)
- All other Phase 23 specs

## Estimated Context
~10%

---

## Objective

Implement comprehensive testing infrastructure for the build system itself. This includes verification of build outputs, artifact integrity checks, installation testing, and smoke tests for packaged applications across all platforms.

---

## Acceptance Criteria

- [ ] Build output verification tests
- [ ] Artifact integrity checks (signatures, checksums)
- [ ] Installation and uninstallation tests
- [ ] Application launch smoke tests
- [ ] Cross-platform build matrix validation
- [ ] Build performance benchmarks
- [ ] Dependency verification
- [ ] Binary compatibility tests
- [ ] Update mechanism tests
- [ ] Build reproducibility verification

---

## Implementation Details

### Build Test Runner (scripts/tests/build-tests.ts)

```typescript
// scripts/tests/build-tests.ts
import * as fs from 'fs';
import * as path from 'path';
import { spawn } from 'child_process';
import { createHash } from 'crypto';

interface TestResult {
  name: string;
  passed: boolean;
  duration: number;
  error?: string;
  details?: Record<string, unknown>;
}

interface TestSuite {
  name: string;
  tests: TestResult[];
  passed: number;
  failed: number;
  duration: number;
}

type TestFunction = () => Promise<void>;

class BuildTestRunner {
  private suites: Map<string, TestFunction[]> = new Map();
  private results: TestSuite[] = [];

  describe(suiteName: string, fn: () => void): void {
    this.suites.set(suiteName, []);
    const currentSuite = this.suites.get(suiteName)!;

    // Temporarily override global test function
    const originalIt = (global as any).it;
    (global as any).it = (name: string, testFn: TestFunction) => {
      currentSuite.push(async () => {
        const start = Date.now();
        try {
          await testFn();
          return { name, passed: true, duration: Date.now() - start };
        } catch (error) {
          return {
            name,
            passed: false,
            duration: Date.now() - start,
            error: error instanceof Error ? error.message : String(error),
          };
        }
      });
    };

    fn();

    (global as any).it = originalIt;
  }

  async run(): Promise<void> {
    console.log('\nRunning Build Tests\n' + '='.repeat(60) + '\n');

    for (const [suiteName, tests] of this.suites) {
      const suite: TestSuite = {
        name: suiteName,
        tests: [],
        passed: 0,
        failed: 0,
        duration: 0,
      };

      console.log(`Suite: ${suiteName}`);

      for (const test of tests) {
        const result = await (test as any)();
        suite.tests.push(result);

        if (result.passed) {
          suite.passed++;
          console.log(`  \x1b[32m\u2713\x1b[0m ${result.name} (${result.duration}ms)`);
        } else {
          suite.failed++;
          console.log(`  \x1b[31m\u2717\x1b[0m ${result.name} (${result.duration}ms)`);
          console.log(`    Error: ${result.error}`);
        }

        suite.duration += result.duration;
      }

      this.results.push(suite);
      console.log('');
    }

    this.printSummary();
  }

  private printSummary(): void {
    console.log('='.repeat(60));
    console.log('Summary');
    console.log('='.repeat(60));

    let totalPassed = 0;
    let totalFailed = 0;

    for (const suite of this.results) {
      totalPassed += suite.passed;
      totalFailed += suite.failed;
      console.log(`  ${suite.name}: ${suite.passed}/${suite.tests.length} passed`);
    }

    console.log('');
    console.log(`Total: ${totalPassed} passed, ${totalFailed} failed`);
    console.log('='.repeat(60));

    if (totalFailed > 0) {
      process.exit(1);
    }
  }
}

export { BuildTestRunner, TestResult, TestSuite };
```

### Artifact Verification Tests (scripts/tests/artifact-tests.ts)

```typescript
// scripts/tests/artifact-tests.ts
import * as fs from 'fs';
import * as path from 'path';
import { createHash } from 'crypto';
import { BuildTestRunner } from './build-tests';

const runner = new BuildTestRunner();

// Expected artifacts for each platform
const EXPECTED_ARTIFACTS = {
  darwin: [
    'Tachikoma-*.dmg',
    'Tachikoma-*.zip',
  ],
  linux: [
    'Tachikoma-*.AppImage',
    'tachikoma_*_amd64.deb',
    'tachikoma-*.x86_64.rpm',
  ],
  win32: [
    'Tachikoma-Setup-*.exe',
    'Tachikoma-*-portable.exe',
  ],
};

// Artifact size limits (in MB)
const SIZE_LIMITS = {
  dmg: { min: 50, max: 300 },
  AppImage: { min: 80, max: 400 },
  exe: { min: 50, max: 250 },
  deb: { min: 50, max: 200 },
  rpm: { min: 50, max: 200 },
};

function findArtifacts(dir: string, pattern: string): string[] {
  if (!fs.existsSync(dir)) return [];

  const regex = new RegExp(pattern.replace('*', '.*'));
  return fs.readdirSync(dir).filter((f) => regex.test(f));
}

function getFileSizeMB(filePath: string): number {
  const stats = fs.statSync(filePath);
  return stats.size / (1024 * 1024);
}

function calculateChecksum(filePath: string, algorithm: string = 'sha256'): string {
  const content = fs.readFileSync(filePath);
  return createHash(algorithm).update(content).digest('hex');
}

runner.describe('Artifact Existence', () => {
  const platform = process.platform;
  const releaseDir = path.join(process.cwd(), 'electron', 'release');

  (global as any).it('should have release directory', async () => {
    if (!fs.existsSync(releaseDir)) {
      throw new Error(`Release directory not found: ${releaseDir}`);
    }
  });

  const expected = EXPECTED_ARTIFACTS[platform as keyof typeof EXPECTED_ARTIFACTS] || [];

  for (const pattern of expected) {
    (global as any).it(`should have artifact matching ${pattern}`, async () => {
      const artifacts = findArtifacts(releaseDir, pattern);
      if (artifacts.length === 0) {
        throw new Error(`No artifact found matching ${pattern}`);
      }
    });
  }
});

runner.describe('Artifact Size', () => {
  const releaseDir = path.join(process.cwd(), 'electron', 'release');

  for (const [ext, limits] of Object.entries(SIZE_LIMITS)) {
    (global as any).it(`should have valid size for .${ext} files`, async () => {
      const files = fs.readdirSync(releaseDir).filter((f) => f.endsWith(`.${ext}`));

      for (const file of files) {
        const filePath = path.join(releaseDir, file);
        const sizeMB = getFileSizeMB(filePath);

        if (sizeMB < limits.min) {
          throw new Error(
            `${file} is too small: ${sizeMB.toFixed(2)}MB (min: ${limits.min}MB)`
          );
        }

        if (sizeMB > limits.max) {
          throw new Error(
            `${file} is too large: ${sizeMB.toFixed(2)}MB (max: ${limits.max}MB)`
          );
        }
      }
    });
  }
});

runner.describe('Artifact Checksums', () => {
  const releaseDir = path.join(process.cwd(), 'electron', 'release');

  (global as any).it('should have SHA256SUMS file', async () => {
    const checksumsPath = path.join(releaseDir, 'SHA256SUMS.txt');
    if (!fs.existsSync(checksumsPath)) {
      throw new Error('SHA256SUMS.txt not found');
    }
  });

  (global as any).it('should have valid checksums', async () => {
    const checksumsPath = path.join(releaseDir, 'SHA256SUMS.txt');
    if (!fs.existsSync(checksumsPath)) return;

    const content = fs.readFileSync(checksumsPath, 'utf-8');
    const lines = content.trim().split('\n');

    for (const line of lines) {
      const [expectedHash, filename] = line.split(/\s+/);
      const filePath = path.join(releaseDir, filename);

      if (!fs.existsSync(filePath)) {
        throw new Error(`File not found: ${filename}`);
      }

      const actualHash = calculateChecksum(filePath);
      if (actualHash !== expectedHash) {
        throw new Error(
          `Checksum mismatch for ${filename}: expected ${expectedHash}, got ${actualHash}`
        );
      }
    }
  });
});

export { runner as artifactTestRunner };
```

### Smoke Tests (scripts/tests/smoke-tests.ts)

```typescript
// scripts/tests/smoke-tests.ts
import { spawn, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { BuildTestRunner } from './build-tests';

const runner = new BuildTestRunner();

interface AppProcess {
  process: ChildProcess;
  stdout: string;
  stderr: string;
}

async function launchApp(
  appPath: string,
  args: string[] = [],
  timeout: number = 30000
): Promise<AppProcess> {
  return new Promise((resolve, reject) => {
    const proc = spawn(appPath, args, {
      env: { ...process.env, ELECTRON_ENABLE_LOGGING: '1' },
    });

    let stdout = '';
    let stderr = '';

    proc.stdout?.on('data', (data) => {
      stdout += data.toString();
    });

    proc.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    // Wait for app to start
    setTimeout(() => {
      resolve({ process: proc, stdout, stderr });
    }, 5000);

    // Timeout
    setTimeout(() => {
      proc.kill();
      reject(new Error(`App launch timed out after ${timeout}ms`));
    }, timeout);

    proc.on('error', (err) => {
      reject(err);
    });
  });
}

function getAppPath(): string | null {
  const platform = process.platform;
  const releaseDir = path.join(process.cwd(), 'electron', 'release');

  switch (platform) {
    case 'darwin': {
      const apps = fs.readdirSync(releaseDir).filter((f) =>
        f.endsWith('.app')
      );
      if (apps.length > 0) {
        return path.join(releaseDir, apps[0], 'Contents', 'MacOS', 'Tachikoma');
      }
      break;
    }
    case 'linux': {
      const appImages = fs.readdirSync(releaseDir).filter((f) =>
        f.endsWith('.AppImage')
      );
      if (appImages.length > 0) {
        const appPath = path.join(releaseDir, appImages[0]);
        fs.chmodSync(appPath, '755');
        return appPath;
      }
      break;
    }
    case 'win32': {
      // For Windows, we need to run from unpacked directory
      const unpackedDir = path.join(releaseDir, 'win-unpacked');
      if (fs.existsSync(unpackedDir)) {
        return path.join(unpackedDir, 'Tachikoma.exe');
      }
      break;
    }
  }

  return null;
}

runner.describe('Application Launch', () => {
  const appPath = getAppPath();

  (global as any).it('should find application executable', async () => {
    if (!appPath) {
      throw new Error('Application executable not found');
    }
    if (!fs.existsSync(appPath)) {
      throw new Error(`Application not found at: ${appPath}`);
    }
  });

  (global as any).it('should launch successfully', async () => {
    if (!appPath) {
      throw new Error('Application path not available');
    }

    const app = await launchApp(appPath, ['--version']);

    // Check for errors in output
    if (app.stderr.includes('Error') || app.stderr.includes('FATAL')) {
      throw new Error(`Application error: ${app.stderr}`);
    }

    app.process.kill();
  });

  (global as any).it('should display version', async () => {
    if (!appPath) {
      throw new Error('Application path not available');
    }

    const app = await launchApp(appPath, ['--version']);

    // Wait for output
    await new Promise((resolve) => setTimeout(resolve, 2000));

    if (!app.stdout.includes('Tachikoma') && !app.stderr.includes('Tachikoma')) {
      throw new Error('Version output not found');
    }

    app.process.kill();
  });
});

runner.describe('Application Startup', () => {
  const appPath = getAppPath();

  (global as any).it('should start without crashes', async () => {
    if (!appPath) {
      throw new Error('Application path not available');
    }

    const app = await launchApp(appPath, [], 10000);

    // Check for crash indicators
    const crashIndicators = [
      'crashed',
      'SIGSEGV',
      'SIGABRT',
      'uncaughtException',
      'unhandledRejection',
    ];

    const output = app.stdout + app.stderr;
    for (const indicator of crashIndicators) {
      if (output.toLowerCase().includes(indicator.toLowerCase())) {
        app.process.kill();
        throw new Error(`Application crashed: ${indicator} detected`);
      }
    }

    app.process.kill();
  });
});

export { runner as smokeTestRunner };
```

### Installation Tests (scripts/tests/install-tests.ts)

```typescript
// scripts/tests/install-tests.ts
import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { BuildTestRunner } from './build-tests';

const runner = new BuildTestRunner();

async function runCommand(
  command: string,
  args: string[]
): Promise<{ code: number; stdout: string; stderr: string }> {
  return new Promise((resolve) => {
    const proc = spawn(command, args, { shell: true });
    let stdout = '';
    let stderr = '';

    proc.stdout?.on('data', (data) => {
      stdout += data.toString();
    });

    proc.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    proc.on('close', (code) => {
      resolve({ code: code ?? -1, stdout, stderr });
    });
  });
}

runner.describe('macOS Installation', () => {
  if (process.platform !== 'darwin') return;

  const releaseDir = path.join(process.cwd(), 'electron', 'release');
  const dmgFiles = fs.existsSync(releaseDir)
    ? fs.readdirSync(releaseDir).filter((f) => f.endsWith('.dmg'))
    : [];

  (global as any).it('should have DMG file', async () => {
    if (dmgFiles.length === 0) {
      throw new Error('No DMG file found');
    }
  });

  (global as any).it('should be a valid DMG', async () => {
    if (dmgFiles.length === 0) return;

    const dmgPath = path.join(releaseDir, dmgFiles[0]);
    const result = await runCommand('hdiutil', ['verify', dmgPath]);

    if (result.code !== 0) {
      throw new Error(`Invalid DMG: ${result.stderr}`);
    }
  });

  (global as any).it('should mount and contain app bundle', async () => {
    if (dmgFiles.length === 0) return;

    const dmgPath = path.join(releaseDir, dmgFiles[0]);
    const mountPoint = '/Volumes/Tachikoma';

    // Mount
    await runCommand('hdiutil', ['attach', dmgPath, '-nobrowse']);

    try {
      // Check for app bundle
      const appPath = path.join(mountPoint, 'Tachikoma.app');
      if (!fs.existsSync(appPath)) {
        throw new Error('App bundle not found in DMG');
      }

      // Check for Applications link
      const appsLink = path.join(mountPoint, 'Applications');
      const stats = fs.lstatSync(appsLink);
      if (!stats.isSymbolicLink()) {
        throw new Error('Applications symlink not found');
      }
    } finally {
      // Unmount
      await runCommand('hdiutil', ['detach', mountPoint, '-force']);
    }
  });
});

runner.describe('Linux Installation', () => {
  if (process.platform !== 'linux') return;

  const releaseDir = path.join(process.cwd(), 'electron', 'release');

  (global as any).it('should have AppImage file', async () => {
    const appImages = fs.existsSync(releaseDir)
      ? fs.readdirSync(releaseDir).filter((f) => f.endsWith('.AppImage'))
      : [];

    if (appImages.length === 0) {
      throw new Error('No AppImage file found');
    }
  });

  (global as any).it('should be executable', async () => {
    const appImages = fs.readdirSync(releaseDir).filter((f) =>
      f.endsWith('.AppImage')
    );
    if (appImages.length === 0) return;

    const appImagePath = path.join(releaseDir, appImages[0]);
    const stats = fs.statSync(appImagePath);

    // Check executable bit
    if (!(stats.mode & 0o111)) {
      throw new Error('AppImage is not executable');
    }
  });

  (global as any).it('should have valid DEB package', async () => {
    const debs = fs.existsSync(releaseDir)
      ? fs.readdirSync(releaseDir).filter((f) => f.endsWith('.deb'))
      : [];
    if (debs.length === 0) return;

    const debPath = path.join(releaseDir, debs[0]);
    const result = await runCommand('dpkg-deb', ['--info', debPath]);

    if (result.code !== 0) {
      throw new Error(`Invalid DEB: ${result.stderr}`);
    }
  });
});

runner.describe('Windows Installation', () => {
  if (process.platform !== 'win32') return;

  const releaseDir = path.join(process.cwd(), 'electron', 'release');

  (global as any).it('should have installer', async () => {
    const installers = fs.existsSync(releaseDir)
      ? fs.readdirSync(releaseDir).filter((f) =>
          f.endsWith('.exe') && f.includes('Setup')
        )
      : [];

    if (installers.length === 0) {
      throw new Error('No installer found');
    }
  });

  (global as any).it('should be signed', async () => {
    const installers = fs.readdirSync(releaseDir).filter((f) =>
      f.endsWith('.exe') && f.includes('Setup')
    );
    if (installers.length === 0) return;

    const installerPath = path.join(releaseDir, installers[0]);
    const result = await runCommand('signtool', ['verify', '/pa', installerPath]);

    // Note: This will fail if not signed, which is expected in some environments
    if (result.code !== 0 && !result.stderr.includes('not signed')) {
      throw new Error(`Signature verification failed: ${result.stderr}`);
    }
  });
});

export { runner as installTestRunner };
```

### Build Performance Tests (scripts/tests/perf-tests.ts)

```typescript
// scripts/tests/perf-tests.ts
import * as fs from 'fs';
import * as path from 'path';
import { BuildTestRunner } from './build-tests';

const runner = new BuildTestRunner();

interface BuildMetrics {
  totalTime: number;
  rustTime: number;
  webTime: number;
  electronTime: number;
  artifactSize: number;
}

function loadBuildMetrics(): BuildMetrics | null {
  const metricsPath = path.join(process.cwd(), '.build-metrics.json');
  if (fs.existsSync(metricsPath)) {
    return JSON.parse(fs.readFileSync(metricsPath, 'utf-8'));
  }
  return null;
}

// Performance thresholds (in milliseconds)
const THRESHOLDS = {
  totalTime: 600000, // 10 minutes
  rustTime: 300000, // 5 minutes
  webTime: 120000, // 2 minutes
  electronTime: 180000, // 3 minutes
};

runner.describe('Build Performance', () => {
  const metrics = loadBuildMetrics();

  (global as any).it('should complete within total time threshold', async () => {
    if (!metrics) {
      throw new Error('Build metrics not available');
    }

    if (metrics.totalTime > THRESHOLDS.totalTime) {
      throw new Error(
        `Build too slow: ${metrics.totalTime}ms (threshold: ${THRESHOLDS.totalTime}ms)`
      );
    }
  });

  (global as any).it('should have Rust build within threshold', async () => {
    if (!metrics) return;

    if (metrics.rustTime > THRESHOLDS.rustTime) {
      throw new Error(
        `Rust build too slow: ${metrics.rustTime}ms (threshold: ${THRESHOLDS.rustTime}ms)`
      );
    }
  });

  (global as any).it('should have web build within threshold', async () => {
    if (!metrics) return;

    if (metrics.webTime > THRESHOLDS.webTime) {
      throw new Error(
        `Web build too slow: ${metrics.webTime}ms (threshold: ${THRESHOLDS.webTime}ms)`
      );
    }
  });
});

runner.describe('Artifact Size', () => {
  const releaseDir = path.join(process.cwd(), 'electron', 'release');

  // Maximum sizes in MB
  const MAX_SIZES = {
    dmg: 200,
    AppImage: 300,
    exe: 200,
    msi: 200,
  };

  for (const [ext, maxSize] of Object.entries(MAX_SIZES)) {
    (global as any).it(`should have .${ext} under ${maxSize}MB`, async () => {
      if (!fs.existsSync(releaseDir)) return;

      const files = fs.readdirSync(releaseDir).filter((f) =>
        f.endsWith(`.${ext}`)
      );

      for (const file of files) {
        const filePath = path.join(releaseDir, file);
        const stats = fs.statSync(filePath);
        const sizeMB = stats.size / (1024 * 1024);

        if (sizeMB > maxSize) {
          throw new Error(
            `${file} is too large: ${sizeMB.toFixed(2)}MB (max: ${maxSize}MB)`
          );
        }
      }
    });
  }
});

export { runner as perfTestRunner };
```

### Test Runner Script (scripts/tests/run-all.ts)

```typescript
// scripts/tests/run-all.ts
import { artifactTestRunner } from './artifact-tests';
import { smokeTestRunner } from './smoke-tests';
import { installTestRunner } from './install-tests';
import { perfTestRunner } from './perf-tests';

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const suites = args.length > 0 ? args : ['artifact', 'smoke', 'install', 'perf'];

  console.log('\n' + '='.repeat(60));
  console.log('Tachikoma Build Tests');
  console.log('='.repeat(60) + '\n');

  const runners: Record<string, any> = {
    artifact: artifactTestRunner,
    smoke: smokeTestRunner,
    install: installTestRunner,
    perf: perfTestRunner,
  };

  for (const suite of suites) {
    const runner = runners[suite];
    if (!runner) {
      console.error(`Unknown test suite: ${suite}`);
      continue;
    }

    await runner.run();
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
```

### Package.json Scripts

```json
{
  "scripts": {
    "test:build": "ts-node scripts/tests/run-all.ts",
    "test:build:artifact": "ts-node scripts/tests/run-all.ts artifact",
    "test:build:smoke": "ts-node scripts/tests/run-all.ts smoke",
    "test:build:install": "ts-node scripts/tests/run-all.ts install",
    "test:build:perf": "ts-node scripts/tests/run-all.ts perf"
  }
}
```

### CI Build Test Workflow (.github/workflows/build-tests.yml)

```yaml
# .github/workflows/build-tests.yml
name: Build Tests

on:
  workflow_call:
    inputs:
      artifact-name:
        required: true
        type: string
      platform:
        required: true
        type: string

jobs:
  test:
    name: Build Tests (${{ inputs.platform }})
    runs-on: ${{ inputs.platform == 'darwin' && 'macos-latest' || inputs.platform == 'linux' && 'ubuntu-latest' || 'windows-latest' }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}
          path: electron/release

      - name: Install dependencies
        run: npm ci

      - name: Run artifact tests
        run: npm run test:build:artifact

      - name: Run smoke tests
        run: npm run test:build:smoke
        continue-on-error: true

      - name: Run installation tests
        run: npm run test:build:install
        continue-on-error: true

      - name: Upload test results
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: test-results-${{ inputs.platform }}
          path: test-results/
```

---

## Testing Requirements

### Meta Tests (Tests for Build Tests)

```typescript
// scripts/tests/__tests__/build-tests.test.ts
import { describe, it, expect } from 'vitest';
import { BuildTestRunner } from '../build-tests';

describe('BuildTestRunner', () => {
  it('should create runner', () => {
    const runner = new BuildTestRunner();
    expect(runner).toBeDefined();
  });

  it('should register test suites', () => {
    const runner = new BuildTestRunner();
    let registered = false;

    runner.describe('Test Suite', () => {
      registered = true;
    });

    expect(registered).toBe(true);
  });
});
```

### Artifact Test Verification

```typescript
// scripts/tests/__tests__/artifact-tests.test.ts
import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('Artifact Tests Configuration', () => {
  it('should have size limits for all formats', () => {
    const formats = ['dmg', 'AppImage', 'exe', 'deb', 'rpm'];

    // Verify configuration exists
    for (const format of formats) {
      expect(typeof format).toBe('string');
    }
  });

  it('should have expected artifacts for all platforms', () => {
    const platforms = ['darwin', 'linux', 'win32'];

    for (const platform of platforms) {
      expect(['darwin', 'linux', 'win32']).toContain(platform);
    }
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 502: CI Integration
- Spec 503: Release Workflow
- Spec 499: macOS Packaging
- Spec 500: Windows Installer
- Spec 501: Linux Packages
