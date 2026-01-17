#!/usr/bin/env ts-node
/**
 * Verify auto-update packages and manifests
 */

import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'js-yaml';
import * as crypto from 'crypto';

interface UpdateManifest {
  version: string;
  files: Array<{
    url: string;
    sha512: string;
    size: number;
  }>;
  path: string;
  sha512: string;
  releaseDate: string;
}

function loadManifest(filePath: string): UpdateManifest {
  const content = fs.readFileSync(filePath, 'utf-8');
  return yaml.load(content) as UpdateManifest;
}

function calculateSha512(filePath: string): string {
  const content = fs.readFileSync(filePath);
  return crypto.createHash('sha512').update(content).digest('base64');
}

function verifyManifest(manifestPath: string, artifactsDir: string): boolean {
  console.log(`\nVerifying: ${path.basename(manifestPath)}`);

  const manifest = loadManifest(manifestPath);
  let errors = 0;

  // Check version format
  if (!/^\d+\.\d+\.\d+/.test(manifest.version)) {
    console.log(`ERROR: Invalid version format: ${manifest.version}`);
    errors++;
  } else {
    console.log(`OK: Version ${manifest.version}`);
  }

  // Check release date
  if (manifest.releaseDate) {
    const date = new Date(manifest.releaseDate);
    if (isNaN(date.getTime())) {
      console.log(`ERROR: Invalid release date: ${manifest.releaseDate}`);
      errors++;
    } else {
      console.log(`OK: Release date ${manifest.releaseDate}`);
    }
  }

  // Verify files
  for (const file of manifest.files) {
    const filename = path.basename(file.url);
    const filePath = path.join(artifactsDir, filename);

    if (!fs.existsSync(filePath)) {
      console.log(`ERROR: Referenced file not found: ${filename}`);
      errors++;
      continue;
    }

    // Verify size
    const stat = fs.statSync(filePath);
    if (stat.size !== file.size) {
      console.log(`ERROR: Size mismatch for ${filename}: expected ${file.size}, got ${stat.size}`);
      errors++;
    } else {
      console.log(`OK: ${filename} size matches (${file.size} bytes)`);
    }

    // Verify hash
    const actualHash = calculateSha512(filePath);
    if (actualHash !== file.sha512) {
      console.log(`ERROR: SHA512 mismatch for ${filename}`);
      console.log(`  Expected: ${file.sha512.substring(0, 20)}...`);
      console.log(`  Actual:   ${actualHash.substring(0, 20)}...`);
      errors++;
    } else {
      console.log(`OK: ${filename} SHA512 matches`);
    }
  }

  return errors === 0;
}

// Main
const artifactsDir = process.argv[2] || 'electron/out';

console.log('=== Auto-Update Package Verification ===');
console.log(`Artifacts directory: ${artifactsDir}`);

const manifests = [
  'latest.yml',
  'latest-mac.yml',
  'latest-linux.yml',
];

let allPassed = true;

for (const manifest of manifests) {
  const manifestPath = path.join(artifactsDir, manifest);
  if (fs.existsSync(manifestPath)) {
    if (!verifyManifest(manifestPath, artifactsDir)) {
      allPassed = false;
    }
  } else {
    console.log(`\nWARNING: ${manifest} not found`);
  }
}

console.log('\n=== Verification Complete ===');

if (allPassed) {
  console.log('PASSED: All update packages verified');
  process.exit(0);
} else {
  console.log('FAILED: Some verifications failed');
  process.exit(1);
}