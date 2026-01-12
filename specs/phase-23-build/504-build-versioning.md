# Spec 504: Version Management

## Phase
23 - Build/Package System

## Spec ID
504

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 503 (Release Workflow)

## Estimated Context
~8%

---

## Objective

Implement comprehensive version management across all components of the Tachikoma project. This includes semantic versioning, automatic version bumping, version synchronization across packages, and version embedding in builds.

---

## Acceptance Criteria

- [ ] Semantic versioning (SemVer) compliance
- [ ] Synchronized versions across all packages
- [ ] Automatic version bumping based on commits
- [ ] Version embedded in all build artifacts
- [ ] Pre-release version support (alpha, beta, rc)
- [ ] Version history tracking
- [ ] Version validation in CI
- [ ] Changelog generation from version history
- [ ] Version comparison utilities
- [ ] API version compatibility tracking

---

## Implementation Details

### Version Configuration (version.config.ts)

```typescript
// version.config.ts
import * as fs from 'fs';
import * as path from 'path';

interface VersionConfig {
  // Current version
  version: string;

  // Pre-release identifier
  prerelease?: string;

  // Build metadata
  build?: string;

  // Files to update
  files: VersionFile[];

  // Git configuration
  git: {
    tagPrefix: string;
    commitMessage: string;
    signTags: boolean;
  };

  // Commit types that trigger version bumps
  commitTypes: {
    major: string[];
    minor: string[];
    patch: string[];
  };
}

interface VersionFile {
  path: string;
  type: 'json' | 'toml' | 'yaml' | 'text';
  pattern?: string; // For text files
  key?: string; // For structured files (e.g., "package.version" for nested)
}

const defaultConfig: VersionConfig = {
  version: '0.0.0',
  files: [
    { path: 'package.json', type: 'json', key: 'version' },
    { path: 'web/package.json', type: 'json', key: 'version' },
    { path: 'electron/package.json', type: 'json', key: 'version' },
    { path: 'Cargo.toml', type: 'toml', key: 'workspace.package.version' },
    { path: 'crates/tachikoma-cli/Cargo.toml', type: 'toml', key: 'package.version' },
  ],
  git: {
    tagPrefix: 'v',
    commitMessage: 'chore(release): v${version}',
    signTags: true,
  },
  commitTypes: {
    major: ['BREAKING CHANGE', '!:'],
    minor: ['feat'],
    patch: ['fix', 'perf', 'refactor'],
  },
};

export { VersionConfig, VersionFile, defaultConfig };
```

### Version Manager (scripts/version/manager.ts)

```typescript
// scripts/version/manager.ts
import * as fs from 'fs';
import * as path from 'path';
import * as semver from 'semver';
import * as toml from '@iarna/toml';
import * as yaml from 'yaml';
import { VersionConfig, VersionFile, defaultConfig } from '../../version.config';

type ReleaseType = 'major' | 'minor' | 'patch' | 'premajor' | 'preminor' | 'prepatch' | 'prerelease';

interface VersionBump {
  type: ReleaseType;
  preid?: string;
}

interface VersionInfo {
  version: string;
  major: number;
  minor: number;
  patch: number;
  prerelease: string[];
  build: string[];
}

class VersionManager {
  private config: VersionConfig;
  private projectRoot: string;

  constructor(projectRoot: string = process.cwd(), config?: Partial<VersionConfig>) {
    this.projectRoot = projectRoot;
    this.config = { ...defaultConfig, ...config };
    this.loadCurrentVersion();
  }

  private loadCurrentVersion(): void {
    // Load from package.json as source of truth
    const packageJsonPath = path.join(this.projectRoot, 'package.json');
    if (fs.existsSync(packageJsonPath)) {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
      this.config.version = packageJson.version;
    }
  }

  getCurrentVersion(): string {
    return this.config.version;
  }

  getVersionInfo(): VersionInfo {
    const parsed = semver.parse(this.config.version);
    if (!parsed) {
      throw new Error(`Invalid version: ${this.config.version}`);
    }

    return {
      version: parsed.version,
      major: parsed.major,
      minor: parsed.minor,
      patch: parsed.patch,
      prerelease: parsed.prerelease.map(String),
      build: parsed.build,
    };
  }

  bump(type: ReleaseType, preid?: string): string {
    const newVersion = semver.inc(this.config.version, type, preid);
    if (!newVersion) {
      throw new Error(`Failed to bump version from ${this.config.version}`);
    }
    return newVersion;
  }

  setVersion(version: string): void {
    if (!semver.valid(version)) {
      throw new Error(`Invalid version: ${version}`);
    }
    this.config.version = version;
  }

  async updateAllFiles(version: string): Promise<void> {
    console.log(`Updating version to ${version} in all files...`);

    for (const file of this.config.files) {
      await this.updateFile(file, version);
    }

    this.config.version = version;
    console.log('All files updated successfully');
  }

  private async updateFile(file: VersionFile, version: string): Promise<void> {
    const filePath = path.join(this.projectRoot, file.path);

    if (!fs.existsSync(filePath)) {
      console.warn(`File not found: ${filePath}`);
      return;
    }

    const content = fs.readFileSync(filePath, 'utf-8');
    let updated: string;

    switch (file.type) {
      case 'json':
        updated = this.updateJson(content, version, file.key);
        break;
      case 'toml':
        updated = this.updateToml(content, version, file.key);
        break;
      case 'yaml':
        updated = this.updateYaml(content, version, file.key);
        break;
      case 'text':
        updated = this.updateText(content, version, file.pattern!);
        break;
      default:
        throw new Error(`Unknown file type: ${file.type}`);
    }

    fs.writeFileSync(filePath, updated);
    console.log(`Updated: ${file.path}`);
  }

  private updateJson(content: string, version: string, key?: string): string {
    const json = JSON.parse(content);
    if (key) {
      this.setNestedValue(json, key, version);
    } else {
      json.version = version;
    }
    return JSON.stringify(json, null, 2) + '\n';
  }

  private updateToml(content: string, version: string, key?: string): string {
    const parsed = toml.parse(content);
    if (key) {
      this.setNestedValue(parsed, key, version);
    } else {
      (parsed as any).version = version;
    }
    return toml.stringify(parsed as any);
  }

  private updateYaml(content: string, version: string, key?: string): string {
    const parsed = yaml.parse(content);
    if (key) {
      this.setNestedValue(parsed, key, version);
    } else {
      parsed.version = version;
    }
    return yaml.stringify(parsed);
  }

  private updateText(content: string, version: string, pattern: string): string {
    const regex = new RegExp(pattern, 'g');
    return content.replace(regex, version);
  }

  private setNestedValue(obj: any, path: string, value: any): void {
    const keys = path.split('.');
    let current = obj;

    for (let i = 0; i < keys.length - 1; i++) {
      if (!(keys[i] in current)) {
        current[keys[i]] = {};
      }
      current = current[keys[i]];
    }

    current[keys[keys.length - 1]] = value;
  }

  validateSync(): string[] {
    const errors: string[] = [];
    const expectedVersion = this.config.version;

    for (const file of this.config.files) {
      const filePath = path.join(this.projectRoot, file.path);

      if (!fs.existsSync(filePath)) {
        continue;
      }

      const content = fs.readFileSync(filePath, 'utf-8');
      let actualVersion: string | undefined;

      try {
        switch (file.type) {
          case 'json':
            const json = JSON.parse(content);
            actualVersion = file.key
              ? this.getNestedValue(json, file.key)
              : json.version;
            break;
          case 'toml':
            const parsed = toml.parse(content);
            actualVersion = file.key
              ? this.getNestedValue(parsed, file.key)
              : (parsed as any).version;
            break;
        }

        if (actualVersion && actualVersion !== expectedVersion) {
          errors.push(
            `Version mismatch in ${file.path}: expected ${expectedVersion}, got ${actualVersion}`
          );
        }
      } catch (e) {
        errors.push(`Failed to parse ${file.path}: ${e}`);
      }
    }

    return errors;
  }

  private getNestedValue(obj: any, path: string): any {
    const keys = path.split('.');
    let current = obj;

    for (const key of keys) {
      if (current === undefined || current === null) {
        return undefined;
      }
      current = current[key];
    }

    return current;
  }

  compare(v1: string, v2: string): -1 | 0 | 1 {
    return semver.compare(v1, v2);
  }

  isPrerelease(version?: string): boolean {
    const v = version ?? this.config.version;
    const prerelease = semver.prerelease(v);
    return prerelease !== null && prerelease.length > 0;
  }

  satisfies(version: string, range: string): boolean {
    return semver.satisfies(version, range);
  }
}

export { VersionManager, ReleaseType, VersionBump, VersionInfo };
```

### Conventional Commits Parser (scripts/version/commits.ts)

```typescript
// scripts/version/commits.ts
import { spawn } from 'child_process';
import { ReleaseType } from './manager';

interface ConventionalCommit {
  hash: string;
  type: string;
  scope?: string;
  subject: string;
  body?: string;
  footer?: string;
  breaking: boolean;
  raw: string;
}

interface VersionRecommendation {
  releaseType: ReleaseType;
  commits: ConventionalCommit[];
  breaking: ConventionalCommit[];
  features: ConventionalCommit[];
  fixes: ConventionalCommit[];
}

const COMMIT_PATTERN = /^(\w+)(?:\(([^)]+)\))?(!)?:\s*(.+)$/;

async function getCommitsSinceTag(tag?: string): Promise<string[]> {
  return new Promise((resolve, reject) => {
    const args = tag
      ? ['log', `${tag}..HEAD`, '--format=%H %s']
      : ['log', '--format=%H %s'];

    const proc = spawn('git', args, { shell: true });
    let output = '';

    proc.stdout.on('data', (data) => {
      output += data.toString();
    });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve(output.trim().split('\n').filter(Boolean));
      } else {
        reject(new Error(`git log failed with code ${code}`));
      }
    });
  });
}

async function getLatestTag(): Promise<string | undefined> {
  return new Promise((resolve) => {
    const proc = spawn('git', ['describe', '--tags', '--abbrev=0'], { shell: true });
    let output = '';

    proc.stdout.on('data', (data) => {
      output += data.toString();
    });

    proc.on('close', (code) => {
      if (code === 0 && output.trim()) {
        resolve(output.trim());
      } else {
        resolve(undefined);
      }
    });
  });
}

function parseCommit(raw: string): ConventionalCommit | null {
  const [hash, ...messageParts] = raw.split(' ');
  const message = messageParts.join(' ');
  const match = message.match(COMMIT_PATTERN);

  if (!match) {
    return null;
  }

  const [, type, scope, breaking, subject] = match;

  return {
    hash,
    type,
    scope,
    subject,
    breaking: breaking === '!' || message.includes('BREAKING CHANGE'),
    raw: message,
  };
}

async function analyzeCommits(): Promise<VersionRecommendation> {
  const latestTag = await getLatestTag();
  const rawCommits = await getCommitsSinceTag(latestTag);

  const commits: ConventionalCommit[] = [];
  const breaking: ConventionalCommit[] = [];
  const features: ConventionalCommit[] = [];
  const fixes: ConventionalCommit[] = [];

  for (const raw of rawCommits) {
    const commit = parseCommit(raw);
    if (!commit) continue;

    commits.push(commit);

    if (commit.breaking) {
      breaking.push(commit);
    }

    if (commit.type === 'feat') {
      features.push(commit);
    }

    if (commit.type === 'fix') {
      fixes.push(commit);
    }
  }

  let releaseType: ReleaseType = 'patch';

  if (breaking.length > 0) {
    releaseType = 'major';
  } else if (features.length > 0) {
    releaseType = 'minor';
  }

  return {
    releaseType,
    commits,
    breaking,
    features,
    fixes,
  };
}

function generateChangelog(recommendation: VersionRecommendation): string {
  const lines: string[] = [];

  if (recommendation.breaking.length > 0) {
    lines.push('### BREAKING CHANGES\n');
    for (const commit of recommendation.breaking) {
      lines.push(`- ${commit.subject} (${commit.hash.slice(0, 7)})`);
    }
    lines.push('');
  }

  if (recommendation.features.length > 0) {
    lines.push('### Features\n');
    for (const commit of recommendation.features) {
      const scope = commit.scope ? `**${commit.scope}:** ` : '';
      lines.push(`- ${scope}${commit.subject} (${commit.hash.slice(0, 7)})`);
    }
    lines.push('');
  }

  if (recommendation.fixes.length > 0) {
    lines.push('### Bug Fixes\n');
    for (const commit of recommendation.fixes) {
      const scope = commit.scope ? `**${commit.scope}:** ` : '';
      lines.push(`- ${scope}${commit.subject} (${commit.hash.slice(0, 7)})`);
    }
    lines.push('');
  }

  return lines.join('\n');
}

export {
  ConventionalCommit,
  VersionRecommendation,
  getCommitsSinceTag,
  getLatestTag,
  parseCommit,
  analyzeCommits,
  generateChangelog,
};
```

### Version CLI (scripts/version/cli.ts)

```typescript
// scripts/version/cli.ts
import { VersionManager, ReleaseType } from './manager';
import { analyzeCommits, generateChangelog } from './commits';

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];

  const manager = new VersionManager();

  switch (command) {
    case 'get':
      console.log(manager.getCurrentVersion());
      break;

    case 'info':
      const info = manager.getVersionInfo();
      console.log(JSON.stringify(info, null, 2));
      break;

    case 'bump': {
      const type = args[1] as ReleaseType;
      const preid = args[2];

      if (!type) {
        console.error('Usage: version bump <type> [preid]');
        console.error('Types: major, minor, patch, premajor, preminor, prepatch, prerelease');
        process.exit(1);
      }

      const newVersion = manager.bump(type, preid);
      console.log(`Bumped version: ${manager.getCurrentVersion()} -> ${newVersion}`);

      if (!args.includes('--dry-run')) {
        await manager.updateAllFiles(newVersion);
      }
      break;
    }

    case 'set': {
      const version = args[1];

      if (!version) {
        console.error('Usage: version set <version>');
        process.exit(1);
      }

      manager.setVersion(version);

      if (!args.includes('--dry-run')) {
        await manager.updateAllFiles(version);
      }
      break;
    }

    case 'validate': {
      const errors = manager.validateSync();

      if (errors.length > 0) {
        console.error('Version validation failed:');
        for (const error of errors) {
          console.error(`  - ${error}`);
        }
        process.exit(1);
      }

      console.log('All versions are in sync');
      break;
    }

    case 'recommend': {
      const recommendation = await analyzeCommits();
      console.log(`Recommended release type: ${recommendation.releaseType}`);
      console.log(`  Breaking changes: ${recommendation.breaking.length}`);
      console.log(`  Features: ${recommendation.features.length}`);
      console.log(`  Fixes: ${recommendation.fixes.length}`);

      if (args.includes('--changelog')) {
        console.log('\nChangelog:\n');
        console.log(generateChangelog(recommendation));
      }
      break;
    }

    default:
      console.log(`
Version Management CLI

Commands:
  get                     Get current version
  info                    Get detailed version info
  bump <type> [preid]     Bump version (major, minor, patch, etc.)
  set <version>           Set specific version
  validate                Validate version sync across files
  recommend               Recommend next version based on commits

Options:
  --dry-run               Don't write changes
  --changelog             Show changelog (with recommend)
      `);
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
    "version:get": "ts-node scripts/version/cli.ts get",
    "version:info": "ts-node scripts/version/cli.ts info",
    "version:bump": "ts-node scripts/version/cli.ts bump",
    "version:set": "ts-node scripts/version/cli.ts set",
    "version:validate": "ts-node scripts/version/cli.ts validate",
    "version:recommend": "ts-node scripts/version/cli.ts recommend"
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// scripts/version/__tests__/manager.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { VersionManager } from '../manager';

vi.mock('fs');

describe('VersionManager', () => {
  let manager: VersionManager;

  beforeEach(() => {
    vi.clearAllMocks();
    manager = new VersionManager('/test', { version: '1.0.0', files: [] });
  });

  it('should get current version', () => {
    expect(manager.getCurrentVersion()).toBe('1.0.0');
  });

  it('should bump major version', () => {
    expect(manager.bump('major')).toBe('2.0.0');
  });

  it('should bump minor version', () => {
    expect(manager.bump('minor')).toBe('1.1.0');
  });

  it('should bump patch version', () => {
    expect(manager.bump('patch')).toBe('1.0.1');
  });

  it('should bump prerelease version', () => {
    expect(manager.bump('prerelease', 'alpha')).toBe('1.0.1-alpha.0');
  });

  it('should validate version format', () => {
    expect(() => manager.setVersion('invalid')).toThrow();
    expect(() => manager.setVersion('1.0.0')).not.toThrow();
  });

  it('should detect prerelease', () => {
    expect(manager.isPrerelease('1.0.0')).toBe(false);
    expect(manager.isPrerelease('1.0.0-alpha.1')).toBe(true);
  });

  it('should compare versions', () => {
    expect(manager.compare('1.0.0', '2.0.0')).toBe(-1);
    expect(manager.compare('2.0.0', '1.0.0')).toBe(1);
    expect(manager.compare('1.0.0', '1.0.0')).toBe(0);
  });
});
```

### Commit Parser Tests

```typescript
// scripts/version/__tests__/commits.test.ts
import { describe, it, expect } from 'vitest';
import { parseCommit, generateChangelog } from '../commits';

describe('Commit Parser', () => {
  it('should parse conventional commit', () => {
    const commit = parseCommit('abc1234 feat(ui): add new button');

    expect(commit).toBeDefined();
    expect(commit?.type).toBe('feat');
    expect(commit?.scope).toBe('ui');
    expect(commit?.subject).toBe('add new button');
    expect(commit?.breaking).toBe(false);
  });

  it('should detect breaking change with !', () => {
    const commit = parseCommit('abc1234 feat!: breaking change');

    expect(commit?.breaking).toBe(true);
  });

  it('should handle commit without scope', () => {
    const commit = parseCommit('abc1234 fix: bug fix');

    expect(commit?.type).toBe('fix');
    expect(commit?.scope).toBeUndefined();
  });

  it('should return null for non-conventional commits', () => {
    const commit = parseCommit('abc1234 random commit message');

    expect(commit).toBeNull();
  });
});

describe('Changelog Generator', () => {
  it('should generate changelog from commits', () => {
    const recommendation = {
      releaseType: 'minor' as const,
      commits: [],
      breaking: [],
      features: [
        { hash: 'abc1234', type: 'feat', scope: 'ui', subject: 'new button', breaking: false, raw: '' },
      ],
      fixes: [
        { hash: 'def5678', type: 'fix', subject: 'bug fix', breaking: false, raw: '' },
      ],
    };

    const changelog = generateChangelog(recommendation);

    expect(changelog).toContain('Features');
    expect(changelog).toContain('new button');
    expect(changelog).toContain('Bug Fixes');
    expect(changelog).toContain('bug fix');
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 503: Release Workflow
- Spec 502: CI Integration
