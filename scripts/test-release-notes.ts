#!/usr/bin/env npx ts-node
/**
 * Test release notes generation functionality
 */

import * as fs from 'fs';
import { execSync } from 'child_process';

interface TestResult {
  name: string;
  passed: boolean;
  message: string;
}

const tests: TestResult[] = [];

function test(name: string, condition: boolean, message: string) {
  tests.push({ name, passed: condition, message });
  console.log(`${condition ? '✅' : '❌'} ${name}: ${message}`);
}

function runTests() {
  console.log('🧪 Testing Release Notes Functionality\n');

  // Test 1: Template exists
  const templateExists = fs.existsSync('docs/release-notes/template.md');
  test('Template exists', templateExists, templateExists ? 'Found template.md' : 'Template missing');

  // Test 2: Template has required sections
  if (templateExists) {
    const template = fs.readFileSync('docs/release-notes/template.md', 'utf-8');
    const requiredSections = [
      '## Highlights',
      '## What\'s New', 
      '## Improvements',
      '## Bug Fixes',
      '## Breaking Changes',
      '## Known Issues',
      '## Upgrade Instructions',
      '## Downloads',
      '## Checksums'
    ];

    requiredSections.forEach(section => {
      const hasSection = template.includes(section);
      test(`Template has ${section}`, hasSection, hasSection ? 'Section found' : 'Section missing');
    });
  }

  // Test 3: Generator script exists and is executable
  const generatorExists = fs.existsSync('scripts/generate-release-notes.ts');
  test('Generator script exists', generatorExists, generatorExists ? 'Found generate-release-notes.ts' : 'Generator missing');

  // Test 4: GitHub release body script exists and is executable
  const githubScriptExists = fs.existsSync('scripts/github-release-body.sh');
  const isExecutable = githubScriptExists ? (fs.statSync('scripts/github-release-body.sh').mode & 0o111) !== 0 : false;
  test('GitHub release script executable', isExecutable, isExecutable ? 'Script is executable' : 'Script not executable or missing');

  // Test 5: Example config exists
  const configExists = fs.existsSync('release-config.example.json');
  test('Example config exists', configExists, configExists ? 'Found release-config.example.json' : 'Example config missing');

  // Test 6: Can generate release notes from example config
  if (generatorExists && configExists) {
    try {
      const testOutput = 'docs/release-notes/test-output.md';
      execSync(`npx ts-node scripts/generate-release-notes.ts release-config.example.json --output ${testOutput}`, { 
        stdio: 'pipe',
        timeout: 30000 
      });
      const generated = fs.existsSync(testOutput);
      test('Can generate release notes', generated, generated ? 'Successfully generated from example config' : 'Generation failed');
      
      if (generated) {
        // Clean up test file
        fs.unlinkSync(testOutput);
      }
    } catch (error) {
      test('Can generate release notes', false, `Generation failed: ${error}`);
    }
  }

  // Test 7: Can generate GitHub release body
  if (githubScriptExists) {
    try {
      const output = execSync('./scripts/github-release-body.sh 1.0.0', { 
        encoding: 'utf-8',
        timeout: 10000
      });
      const hasContent = output.includes('## Tachikoma v1.0.0') && output.includes('### Quick Links');
      test('Can generate GitHub release body', hasContent, hasContent ? 'Generated valid release body' : 'Invalid release body format');
    } catch (error) {
      test('Can generate GitHub release body', false, `GitHub body generation failed: ${error}`);
    }
  }

  // Test 8: Download links use correct format
  if (templateExists) {
    const template = fs.readFileSync('docs/release-notes/template.md', 'utf-8');
    const hasDownloadLinks = template.includes('github.com/tachikoma/tachikoma/releases/download/v{VERSION}');
    const hasPlatforms = ['macOS (Intel)', 'macOS (Apple Silicon)', 'Windows', 'Linux (AppImage)', 'Linux (deb)']
      .every(platform => template.includes(platform));
    test('Download links properly formatted', hasDownloadLinks && hasPlatforms, 
      hasDownloadLinks && hasPlatforms ? 'All platforms and correct format' : 'Missing platforms or incorrect format');
  }

  // Summary
  const passed = tests.filter(t => t.passed).length;
  const total = tests.length;
  
  console.log(`\n📊 Test Results: ${passed}/${total} passed`);
  
  if (passed === total) {
    console.log('🎉 All tests passed! Release notes functionality is working correctly.');
    process.exit(0);
  } else {
    console.log('❌ Some tests failed. Please check the issues above.');
    process.exit(1);
  }
}

runTests();