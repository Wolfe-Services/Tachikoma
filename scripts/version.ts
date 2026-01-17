#!/usr/bin/env ts-node
/**
 * Version management script
 * Keeps all version numbers in sync across the project
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const ROOT_DIR = path.join(__dirname, '..');
const VERSION_FILE = path.join(ROOT_DIR, 'version.json');

interface VersionConfig {
  version: string;
  channel: 'stable' | 'beta' | 'alpha';
  prerelease: string | null;
  build: string | null;
}

function loadVersion(): VersionConfig {
  return JSON.parse(fs.readFileSync(VERSION_FILE, 'utf-8'));
}

function saveVersion(config: VersionConfig): void {
  fs.writeFileSync(VERSION_FILE, JSON.stringify(config, null, 2) + '\n');
}

function getFullVersion(config: VersionConfig): string {
  let version = config.version;
  if (config.prerelease) {
    version += `-${config.prerelease}`;
  }
  if (config.build) {
    version += `+${config.build}`;
  }
  return version;
}

function updatePackageJson(filePath: string, version: string): void {
  const pkg = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
  pkg.version = version;
  fs.writeFileSync(filePath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`Updated ${filePath}`);
}

function updateCargoToml(filePath: string, version: string): void {
  let content = fs.readFileSync(filePath, 'utf-8');

  // Update workspace version
  content = content.replace(
    /^version = ".*"/m,
    `version = "${version}"`
  );

  // Update workspace.package version
  content = content.replace(
    /(\[workspace\.package\][\s\S]*?)version = ".*"/m,
    `$1version = "${version}"`
  );

  fs.writeFileSync(filePath, content);
  console.log(`Updated ${filePath}`);
}

function syncVersions(): void {
  const config = loadVersion();
  const version = config.version; // Use base version for files

  console.log(`Syncing version: ${getFullVersion(config)}`);

  // Update root package.json
  updatePackageJson(path.join(ROOT_DIR, 'package.json'), version);

  // Update web package.json
  updatePackageJson(path.join(ROOT_DIR, 'web/package.json'), version);

  // Update electron package.json
  updatePackageJson(path.join(ROOT_DIR, 'electron/package.json'), version);

  // Update root Cargo.toml
  updateCargoToml(path.join(ROOT_DIR, 'Cargo.toml'), version);

  console.log('Version sync complete!');
}

function bumpVersion(type: 'major' | 'minor' | 'patch' | 'prerelease'): void {
  const config = loadVersion();
  const [major, minor, patch] = config.version.split('.').map(Number);

  switch (type) {
    case 'major':
      config.version = `${major + 1}.0.0`;
      config.prerelease = null;
      break;
    case 'minor':
      config.version = `${major}.${minor + 1}.0`;
      config.prerelease = null;
      break;
    case 'patch':
      config.version = `${major}.${minor}.${patch + 1}`;
      config.prerelease = null;
      break;
    case 'prerelease':
      if (config.prerelease) {
        const match = config.prerelease.match(/^(\w+)\.(\d+)$/);
        if (match) {
          config.prerelease = `${match[1]}.${Number(match[2]) + 1}`;
        }
      } else {
        config.prerelease = 'beta.1';
      }
      break;
  }

  saveVersion(config);
  syncVersions();

  console.log(`Version bumped to ${getFullVersion(config)}`);
}

function setBuildMetadata(build: string | null): void {
  const config = loadVersion();
  config.build = build;
  saveVersion(config);
  console.log(`Build metadata set to: ${build || '(none)'}`);
}

// CLI
const command = process.argv[2];
const arg = process.argv[3];

switch (command) {
  case 'sync':
    syncVersions();
    break;
  case 'bump':
    if (!['major', 'minor', 'patch', 'prerelease'].includes(arg)) {
      console.error('Usage: version.ts bump <major|minor|patch|prerelease>');
      process.exit(1);
    }
    bumpVersion(arg as 'major' | 'minor' | 'patch' | 'prerelease');
    break;
  case 'set':
    if (!arg) {
      console.error('Usage: version.ts set <version>');
      process.exit(1);
    }
    const config = loadVersion();
    config.version = arg;
    saveVersion(config);
    syncVersions();
    break;
  case 'build':
    setBuildMetadata(arg || null);
    break;
  case 'get':
    console.log(getFullVersion(loadVersion()));
    break;
  default:
    console.log('Usage: version.ts <sync|bump|set|build|get>');
    process.exit(1);
}