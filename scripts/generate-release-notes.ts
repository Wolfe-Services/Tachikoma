#!/usr/bin/env npx ts-node
/**
 * Generate release notes from config and changelog
 */

import * as fs from 'fs';
import { execSync } from 'child_process';

interface ReleaseConfig {
  version: string;
  date: string;
  type: 'major' | 'minor' | 'patch';
  highlights: string;
  features: Array<{ title: string; description: string; image?: string }>;
  improvements: string[];
  bugfixes: Array<{ description: string; issue?: string }>;
  breakingChanges: Array<{ 
    title: string; 
    what: string; 
    why: string; 
    migration: string;
    codeDiff?: string;
  }>;
  knownIssues: string[];
  upgradeSteps: string[];
  configChanges?: string[];
  contributors?: string[];
}

function loadConfig(path: string): ReleaseConfig {
  if (!fs.existsSync(path)) {
    throw new Error(`Config file not found: ${path}`);
  }
  return JSON.parse(fs.readFileSync(path, 'utf-8'));
}

function loadTemplate(): string {
  const templatePath = 'docs/release-notes/template.md';
  if (!fs.existsSync(templatePath)) {
    throw new Error(`Template not found: ${templatePath}`);
  }
  return fs.readFileSync(templatePath, 'utf-8');
}

function getPreviousVersion(): string | null {
  try {
    const tags = execSync('git tag -l --sort=-version:refname', { encoding: 'utf-8' });
    const tagList = tags.split('\n').filter(t => t.match(/^v?\d+\.\d+\.\d+$/));
    return tagList[0]?.replace(/^v/, '') || null;
  } catch {
    return null;
  }
}

function generateReleaseNotes(config: ReleaseConfig): string {
  const template = loadTemplate();
  const prevVersion = getPreviousVersion();

  let notes = template
    .replace(/{VERSION}/g, config.version)
    .replace(/{DATE}/g, config.date)
    .replace(/{Major\|Minor\|Patch}/g, capitalize(config.type))
    .replace(/{PREV_VERSION}/g, prevVersion || '0.0.0');

  // Replace highlights
  notes = notes.replace(
    /Brief summary of the most exciting features in this release \(2-3 sentences\)\./,
    config.highlights
  );

  // Generate features section
  const featuresSection = config.features.length > 0 
    ? config.features
        .map(f => {
          let section = `### ${f.title}\n${f.description}`;
          if (f.image) {
            section += `\n\n![${f.title}](${f.image})`;
          }
          return section;
        })
        .join('\n\n')
    : '### No major new features in this release';

  // Generate improvements
  const improvementsSection = config.improvements.length > 0
    ? config.improvements.map(i => `- ${i}`).join('\n')
    : '- General stability improvements';

  // Generate bug fixes
  const bugfixesSection = config.bugfixes.length > 0
    ? config.bugfixes
        .map(b => `- ${b.description}${b.issue ? ` (#${b.issue})` : ''}`)
        .join('\n')
    : '- No bug fixes in this release';

  // Generate breaking changes
  let breakingSection = '';
  if (config.breakingChanges.length > 0) {
    breakingSection = config.breakingChanges
      .map(bc => {
        let section = `### ${bc.title}\n**What changed:** ${bc.what}\n\n**Why:** ${bc.why}\n\n**Migration:** ${bc.migration}`;
        if (bc.codeDiff) {
          section += `\n\n\`\`\`diff\n${bc.codeDiff}\n\`\`\``;
        }
        return section;
      })
      .join('\n\n');
  } else {
    breakingSection = 'No breaking changes in this release.';
  }

  // Generate known issues
  const knownIssuesSection = config.knownIssues.length > 0
    ? config.knownIssues.map(i => `- ${i}`).join('\n')
    : '- No known issues at this time';

  // Generate upgrade steps
  const upgradeSection = config.upgradeSteps.length > 0
    ? config.upgradeSteps.map((step, i) => `${i + 1}. ${step}`).join('\n')
    : '1. Download the new version\n2. Your settings will be automatically migrated';

  // Generate config changes section
  let configSection = '';
  if (config.configChanges && config.configChanges.length > 0) {
    configSection = config.configChanges.join('\n\n');
  } else {
    configSection = 'No configuration changes required.';
  }

  // Generate contributors section
  let contributorsSection = '';
  if (config.contributors && config.contributors.length > 0) {
    contributorsSection = config.contributors.map(c => `@${c}`).join(', ');
  } else {
    contributorsSection = 'Thanks to all contributors who made this release possible!';
  }

  // Replace sections
  notes = notes
    .replace(/### Feature Name[\s\S]*?### Another Feature[\s\S]*?\.\.\./, featuresSection)
    .replace(/- Improvement 1 with brief description[\s\S]*?- Performance improvement details/, improvementsSection)
    .replace(/- Fixed issue where \[description\] \(#123\)[\s\S]*?- Fixed crash when \[description\] \(#124\)/, bugfixesSection)
    .replace(/### Change Title[\s\S]*?```diff[\s\S]*?```/, breakingSection)
    .replace(/- Issue description and workaround if available[\s\S]*?- Link to tracking issue if applicable/, knownIssuesSection)
    .replace(/1\. Step 1[\s\S]*?3\. Step 3/, upgradeSection)
    .replace(/Any configuration file changes needed\./, configSection)
    .replace(/@contributor1, @contributor2, \.\.\./, contributorsSection);

  return notes;
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

// Generate checksums for build artifacts
function generateChecksums(dir: string): string {
  if (!fs.existsSync(dir)) {
    return 'Build artifacts not found. Checksums will be added when artifacts are built.';
  }

  const files = fs.readdirSync(dir).filter(f => 
    !f.endsWith('.yml') && 
    !f.endsWith('.blockmap') &&
    !f.endsWith('.sig') &&
    f.includes('Tachikoma')
  );
  
  if (files.length === 0) {
    return 'No build artifacts found. Checksums will be added when artifacts are built.';
  }

  const checksums = files.map(f => {
    try {
      const hash = execSync(`shasum -a 256 "${dir}/${f}"`, { encoding: 'utf-8' }).split(' ')[0];
      return `${hash}  ${f}`;
    } catch {
      return `<hash>  ${f}`;
    }
  });
  
  return checksums.join('\n');
}

// CLI
const configPath = process.argv[2] || 'release-config.json';

if (process.argv.includes('--help') || process.argv.includes('-h')) {
  console.log(`
Usage: generate-release-notes.ts [config-file] [options]

Options:
  --help, -h     Show this help message
  --checksums    Generate checksums from dist/ directory
  --output       Specify output file (default: docs/release-notes/v{VERSION}.md)

Example:
  npx ts-node scripts/generate-release-notes.ts release-config.json
  npx ts-node scripts/generate-release-notes.ts --checksums
`);
  process.exit(0);
}

try {
  const config = loadConfig(configPath);
  let notes = generateReleaseNotes(config);

  // Generate checksums if requested
  if (process.argv.includes('--checksums')) {
    const checksums = generateChecksums('dist');
    notes = notes.replace(/SHA256 checksums:[\s\S]*?```/, `SHA256 checksums:\n${checksums}\n\`\`\``);
  }

  const outputIndex = process.argv.indexOf('--output');
  const outputPath = outputIndex !== -1 && process.argv[outputIndex + 1]
    ? process.argv[outputIndex + 1]
    : `docs/release-notes/v${config.version}.md`;

  fs.writeFileSync(outputPath, notes);
  console.log(`Release notes written to ${outputPath}`);

} catch (error) {
  console.error('Error generating release notes:', error);
  process.exit(1);
}