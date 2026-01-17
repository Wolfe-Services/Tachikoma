# 506 - Changelog Generation

**Phase:** 23 - Build & Distribution
**Spec ID:** 506
**Status:** Planned
**Dependencies:** 505-release-tagging
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement automated changelog generation from git commit history following the Keep a Changelog format, with support for conventional commits.

---

## Acceptance Criteria

- [x] Changelog follows Keep a Changelog format
- [x] Conventional commits parsed automatically
- [x] Breaking changes highlighted
- [x] Links to PRs and issues included
- [x] Manual entries supported
- [x] CI validates changelog on release

---

## Implementation Details

### 1. Changelog Format

Create/update `CHANGELOG.md`:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New feature descriptions go here

### Changed
- Changes in existing functionality

### Deprecated
- Soon-to-be removed features

### Removed
- Removed features

### Fixed
- Bug fixes

### Security
- Security fixes

## [1.0.0] - 2024-01-15

### Added
- Initial release of Tachikoma
- Multi-model backend support (Claude, Codex, Gemini, Ollama)
- Autonomous loop runner for unattended operation
- Spec-driven development workflow
- Spec Forge for multi-model brainstorming

[Unreleased]: https://github.com/tachikoma/tachikoma/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/tachikoma/tachikoma/releases/tag/v1.0.0
```

### 2. Changelog Generation Script

Create `scripts/generate-changelog.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Generate changelog from git commits following conventional commits
 */

import { execSync } from 'child_process';
import * as fs from 'fs';

interface Commit {
  hash: string;
  type: string;
  scope: string | null;
  subject: string;
  body: string;
  breaking: boolean;
  pr: string | null;
  issues: string[];
}

const COMMIT_TYPES: Record<string, string> = {
  feat: 'Added',
  fix: 'Fixed',
  docs: 'Documentation',
  style: 'Changed',
  refactor: 'Changed',
  perf: 'Changed',
  test: 'Changed',
  build: 'Changed',
  ci: 'Changed',
  chore: 'Changed',
  revert: 'Removed',
};

function getCommitsSince(tag: string | null): Commit[] {
  const range = tag ? `${tag}..HEAD` : 'HEAD';
  const format = '%H|%s|%b';

  try {
    const output = execSync(`git log ${range} --format="${format}" --no-merges`, {
      encoding: 'utf-8',
    });

    return output
      .split('\n')
      .filter(Boolean)
      .map(line => parseCommit(line))
      .filter((c): c is Commit => c !== null);
  } catch {
    return [];
  }
}

function parseCommit(raw: string): Commit | null {
  const [hash, subject, body = ''] = raw.split('|');

  // Parse conventional commit format: type(scope): subject
  const match = subject.match(/^(\w+)(?:\(([^)]+)\))?!?:\s*(.+)$/);
  if (!match) return null;

  const [, type, scope, subjectText] = match;

  // Extract PR number
  const prMatch = subject.match(/#(\d+)/);
  const pr = prMatch ? prMatch[1] : null;

  // Extract issue references
  const issueMatches = (body + subject).matchAll(/(close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s+#(\d+)/gi);
  const issues = Array.from(issueMatches).map(m => m[2]);

  // Check for breaking change
  const breaking = subject.includes('!:') || body.includes('BREAKING CHANGE');

  return {
    hash: hash.slice(0, 7),
    type,
    scope: scope || null,
    subject: subjectText,
    body,
    breaking,
    pr,
    issues,
  };
}

function groupByType(commits: Commit[]): Record<string, Commit[]> {
  const groups: Record<string, Commit[]> = {};

  for (const commit of commits) {
    const section = COMMIT_TYPES[commit.type] || 'Changed';
    if (!groups[section]) groups[section] = [];
    groups[section].push(commit);
  }

  return groups;
}

function formatCommit(commit: Commit): string {
  let line = `- ${commit.subject}`;

  if (commit.scope) {
    line = `- **${commit.scope}**: ${commit.subject}`;
  }

  if (commit.pr) {
    line += ` ([#${commit.pr}](https://github.com/tachikoma/tachikoma/pull/${commit.pr}))`;
  }

  return line;
}

function generateChangelogSection(version: string, date: string, commits: Commit[]): string {
  const groups = groupByType(commits);
  const sections: string[] = [];

  // Breaking changes first
  const breaking = commits.filter(c => c.breaking);
  if (breaking.length > 0) {
    sections.push('### Breaking Changes\n');
    sections.push(breaking.map(formatCommit).join('\n'));
    sections.push('');
  }

  // Other sections in order
  const sectionOrder = ['Added', 'Changed', 'Deprecated', 'Removed', 'Fixed', 'Security', 'Documentation'];

  for (const section of sectionOrder) {
    const sectionCommits = groups[section]?.filter(c => !c.breaking);
    if (sectionCommits?.length) {
      sections.push(`### ${section}\n`);
      sections.push(sectionCommits.map(formatCommit).join('\n'));
      sections.push('');
    }
  }

  return `## [${version}] - ${date}\n\n${sections.join('\n')}`;
}

function updateChangelog(newSection: string, version: string): void {
  const changelogPath = 'CHANGELOG.md';
  let content = fs.readFileSync(changelogPath, 'utf-8');

  // Insert after [Unreleased] section
  const unreleasedMatch = content.match(/## \[Unreleased\][\s\S]*?(?=## \[|$)/);
  if (unreleasedMatch) {
    const insertPoint = unreleasedMatch.index! + unreleasedMatch[0].length;
    content = content.slice(0, insertPoint) + '\n' + newSection + '\n' + content.slice(insertPoint);
  }

  // Update links at bottom
  const prevVersion = content.match(/## \[(\d+\.\d+\.\d+)\]/)?.[1];
  if (prevVersion) {
    const compareLink = `[${version}]: https://github.com/tachikoma/tachikoma/compare/v${prevVersion}...v${version}`;
    content = content.replace(
      /\[Unreleased\]: .*/,
      `[Unreleased]: https://github.com/tachikoma/tachikoma/compare/v${version}...HEAD\n${compareLink}`
    );
  }

  fs.writeFileSync(changelogPath, content);
}

// CLI
const version = process.argv[2];
const date = process.argv[3] || new Date().toISOString().split('T')[0];

if (!version) {
  console.error('Usage: generate-changelog.ts <version> [date]');
  process.exit(1);
}

const lastTag = execSync('git describe --tags --abbrev=0 2>/dev/null || echo ""', {
  encoding: 'utf-8',
}).trim();

const commits = getCommitsSince(lastTag || null);

if (commits.length === 0) {
  console.log('No commits found since last tag');
  process.exit(0);
}

const section = generateChangelogSection(version, date, commits);
console.log('Generated changelog section:\n');
console.log(section);

if (process.argv.includes('--write')) {
  updateChangelog(section, version);
  console.log('\nChangelog updated!');
}
```

### 3. Commit Message Validation

Create `.commitlintrc.json`:

```json
{
  "extends": ["@commitlint/config-conventional"],
  "rules": {
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert"]
    ],
    "scope-empty": [0],
    "subject-case": [0]
  }
}
```

### 4. Git Hooks

Create `.husky/commit-msg`:

```bash
#!/bin/sh
npx --no -- commitlint --edit "$1"
```

---

## Testing Requirements

1. Changelog follows Keep a Changelog format
2. Conventional commits are parsed correctly
3. Breaking changes are highlighted
4. Links to PRs/issues work
5. Changelog validates on release

---

## Related Specs

- Depends on: [505-release-tagging.md](505-release-tagging.md)
- Next: [507-release-notes.md](507-release-notes.md)
- Related: [504-version-management.md](504-version-management.md)
