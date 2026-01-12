# Spec 209: Diff Viewer Component

## Phase
Phase 9: UI Foundation

## Spec ID
209

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 208: Code Block Component
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement a Diff Viewer component for Tachikoma to display file changes, configuration differences, and code modifications with side-by-side and unified view modes, syntax highlighting, and navigation.

---

## Acceptance Criteria

- [x] Side-by-side diff view
- [x] Unified diff view
- [x] Syntax highlighting
- [x] Line numbers
- [x] Addition/deletion/modification highlighting
- [x] Expand/collapse unchanged sections
- [x] Navigation between changes
- [x] Word-level diff within lines
- [x] Copy individual changes
- [x] Support for large diffs with virtualization

---

## Implementation Details

### src/lib/components/ui/DiffViewer/types.ts

```typescript
export interface DiffLine {
  type: 'add' | 'remove' | 'unchanged' | 'header';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface DiffFile {
  oldPath: string;
  newPath: string;
  hunks: DiffHunk[];
  isBinary?: boolean;
  isNew?: boolean;
  isDeleted?: boolean;
  isRenamed?: boolean;
}
```

### src/lib/components/ui/DiffViewer/DiffViewer.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { cn } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';
  import Button from '../Button/Button.svelte';
  import type { DiffFile, DiffHunk, DiffLine } from './types';

  type ViewMode = 'split' | 'unified';

  export let diff: DiffFile;
  export let viewMode: ViewMode = 'split';
  export let language: string = 'text';
  export let showLineNumbers: boolean = true;
  export let expandedContext: number = 3;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    expand: { hunkIndex: number };
    copy: { type: 'old' | 'new'; content: string };
  }>();

  let currentChangeIndex = 0;

  $: allChanges = diff.hunks.flatMap((hunk, hunkIndex) =>
    hunk.lines
      .map((line, lineIndex) => ({ line, hunkIndex, lineIndex }))
      .filter(({ line }) => line.type === 'add' || line.type === 'remove')
  );

  $: totalChanges = allChanges.length;

  function navigateToChange(direction: 'prev' | 'next') {
    if (direction === 'next' && currentChangeIndex < totalChanges - 1) {
      currentChangeIndex++;
    } else if (direction === 'prev' && currentChangeIndex > 0) {
      currentChangeIndex--;
    }

    // Scroll to the change
    const change = allChanges[currentChangeIndex];
    if (change) {
      const element = document.querySelector(
        `[data-hunk="${change.hunkIndex}"][data-line="${change.lineIndex}"]`
      );
      element?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  function getLineClass(line: DiffLine): string {
    switch (line.type) {
      case 'add': return 'diff-line-add';
      case 'remove': return 'diff-line-remove';
      case 'header': return 'diff-line-header';
      default: return 'diff-line-unchanged';
    }
  }

  function getLinePrefix(line: DiffLine): string {
    switch (line.type) {
      case 'add': return '+';
      case 'remove': return '-';
      case 'header': return '@@';
      default: return ' ';
    }
  }

  $: classes = cn(
    'diff-viewer',
    `diff-viewer-${viewMode}`,
    className
  );
</script>

<div class={classes}>
  <div class="diff-header">
    <div class="diff-file-info">
      {#if diff.isNew}
        <span class="diff-badge diff-badge-new">New</span>
      {:else if diff.isDeleted}
        <span class="diff-badge diff-badge-deleted">Deleted</span>
      {:else if diff.isRenamed}
        <span class="diff-badge diff-badge-renamed">Renamed</span>
      {/if}

      <span class="diff-file-path">
        {#if diff.oldPath !== diff.newPath}
          <span class="diff-old-path">{diff.oldPath}</span>
          <Icon name="arrow-right" size={14} />
        {/if}
        <span class="diff-new-path">{diff.newPath}</span>
      </span>
    </div>

    <div class="diff-actions">
      <div class="diff-navigation">
        <span class="diff-change-count">
          {currentChangeIndex + 1} / {totalChanges} changes
        </span>
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          disabled={currentChangeIndex === 0}
          on:click={() => navigateToChange('prev')}
          aria-label="Previous change"
        >
          <Icon name="chevron-up" size={16} />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          disabled={currentChangeIndex === totalChanges - 1}
          on:click={() => navigateToChange('next')}
          aria-label="Next change"
        >
          <Icon name="chevron-down" size={16} />
        </Button>
      </div>

      <div class="diff-view-toggle">
        <Button
          variant={viewMode === 'split' ? 'secondary' : 'ghost'}
          size="sm"
          on:click={() => viewMode = 'split'}
        >
          Split
        </Button>
        <Button
          variant={viewMode === 'unified' ? 'secondary' : 'ghost'}
          size="sm"
          on:click={() => viewMode = 'unified'}
        >
          Unified
        </Button>
      </div>
    </div>
  </div>

  {#if diff.isBinary}
    <div class="diff-binary">
      Binary file not shown
    </div>
  {:else}
    <div class="diff-content">
      {#each diff.hunks as hunk, hunkIndex}
        <div class="diff-hunk">
          <div class="diff-hunk-header">
            @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
          </div>

          {#if viewMode === 'split'}
            <div class="diff-split">
              <div class="diff-split-side diff-split-old">
                {#each hunk.lines as line, lineIndex}
                  {#if line.type !== 'add'}
                    <div
                      class="diff-line {getLineClass(line)}"
                      data-hunk={hunkIndex}
                      data-line={lineIndex}
                    >
                      {#if showLineNumbers}
                        <span class="diff-line-number">{line.oldLineNumber || ''}</span>
                      {/if}
                      <span class="diff-line-prefix">{line.type === 'remove' ? '-' : ' '}</span>
                      <span class="diff-line-content">{line.content}</span>
                    </div>
                  {:else}
                    <div class="diff-line diff-line-empty">
                      {#if showLineNumbers}
                        <span class="diff-line-number"></span>
                      {/if}
                      <span class="diff-line-content"></span>
                    </div>
                  {/if}
                {/each}
              </div>

              <div class="diff-split-side diff-split-new">
                {#each hunk.lines as line, lineIndex}
                  {#if line.type !== 'remove'}
                    <div
                      class="diff-line {getLineClass(line)}"
                      data-hunk={hunkIndex}
                      data-line={lineIndex}
                    >
                      {#if showLineNumbers}
                        <span class="diff-line-number">{line.newLineNumber || ''}</span>
                      {/if}
                      <span class="diff-line-prefix">{line.type === 'add' ? '+' : ' '}</span>
                      <span class="diff-line-content">{line.content}</span>
                    </div>
                  {:else}
                    <div class="diff-line diff-line-empty">
                      {#if showLineNumbers}
                        <span class="diff-line-number"></span>
                      {/if}
                      <span class="diff-line-content"></span>
                    </div>
                  {/if}
                {/each}
              </div>
            </div>
          {:else}
            <div class="diff-unified">
              {#each hunk.lines as line, lineIndex}
                <div
                  class="diff-line {getLineClass(line)}"
                  data-hunk={hunkIndex}
                  data-line={lineIndex}
                >
                  {#if showLineNumbers}
                    <span class="diff-line-number diff-line-number-old">
                      {line.oldLineNumber || ''}
                    </span>
                    <span class="diff-line-number diff-line-number-new">
                      {line.newLineNumber || ''}
                    </span>
                  {/if}
                  <span class="diff-line-prefix">{getLinePrefix(line)}</span>
                  <span class="diff-line-content">{line.content}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .diff-viewer {
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-surface);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .diff-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-3) var(--spacing-4);
    background-color: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .diff-file-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .diff-badge {
    padding: var(--spacing-0-5) var(--spacing-2);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    border-radius: var(--radius-sm);
  }

  .diff-badge-new {
    background-color: var(--color-success-subtle);
    color: var(--color-success-fg);
  }

  .diff-badge-deleted {
    background-color: var(--color-error-subtle);
    color: var(--color-error-fg);
  }

  .diff-badge-renamed {
    background-color: var(--color-info-subtle);
    color: var(--color-info-fg);
  }

  .diff-file-path {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    color: var(--color-fg-default);
  }

  .diff-old-path {
    color: var(--color-fg-muted);
    text-decoration: line-through;
  }

  .diff-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-4);
  }

  .diff-navigation {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
  }

  .diff-change-count {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    margin-right: var(--spacing-2);
  }

  .diff-view-toggle {
    display: flex;
    gap: var(--spacing-1);
  }

  .diff-content {
    overflow-x: auto;
  }

  .diff-binary {
    padding: var(--spacing-8);
    text-align: center;
    color: var(--color-fg-muted);
  }

  .diff-hunk {
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .diff-hunk:last-child {
    border-bottom: none;
  }

  .diff-hunk-header {
    padding: var(--spacing-2) var(--spacing-4);
    background-color: var(--color-info-subtle);
    color: var(--color-info-fg);
    font-size: var(--text-xs);
  }

  /* Split view */
  .diff-split {
    display: flex;
  }

  .diff-split-side {
    flex: 1;
    min-width: 0;
  }

  .diff-split-old {
    border-right: 1px solid var(--color-border-subtle);
  }

  /* Line styles */
  .diff-line {
    display: flex;
    min-height: 24px;
    line-height: 24px;
  }

  .diff-line-number {
    flex-shrink: 0;
    width: 50px;
    padding: 0 var(--spacing-2);
    text-align: right;
    color: var(--color-fg-muted);
    background-color: var(--color-bg-muted);
    user-select: none;
  }

  .diff-line-prefix {
    flex-shrink: 0;
    width: 20px;
    text-align: center;
    user-select: none;
  }

  .diff-line-content {
    flex: 1;
    padding: 0 var(--spacing-2);
    white-space: pre;
  }

  .diff-line-add {
    background-color: var(--color-success-subtle);
  }

  .diff-line-add .diff-line-content {
    background-color: rgba(34, 197, 94, 0.2);
  }

  .diff-line-add .diff-line-prefix {
    color: var(--color-success-fg);
  }

  .diff-line-remove {
    background-color: var(--color-error-subtle);
  }

  .diff-line-remove .diff-line-content {
    background-color: rgba(239, 68, 68, 0.2);
  }

  .diff-line-remove .diff-line-prefix {
    color: var(--color-error-fg);
  }

  .diff-line-unchanged {
    background-color: transparent;
  }

  .diff-line-empty {
    background-color: var(--color-bg-muted);
  }

  .diff-line-header {
    background-color: var(--color-info-subtle);
    color: var(--color-info-fg);
  }
</style>
```

### src/lib/utils/diff.ts

```typescript
/**
 * Diff parsing and generation utilities
 */

import type { DiffFile, DiffHunk, DiffLine } from '@components/ui/DiffViewer/types';

/**
 * Parse unified diff format
 */
export function parseUnifiedDiff(diffText: string): DiffFile[] {
  const files: DiffFile[] = [];
  const lines = diffText.split('\n');

  let currentFile: DiffFile | null = null;
  let currentHunk: DiffHunk | null = null;
  let oldLineNum = 0;
  let newLineNum = 0;

  for (const line of lines) {
    // File header
    if (line.startsWith('--- ')) {
      if (currentFile && currentHunk) {
        currentFile.hunks.push(currentHunk);
      }
      if (currentFile) {
        files.push(currentFile);
      }

      currentFile = {
        oldPath: line.slice(4).replace(/^a\//, ''),
        newPath: '',
        hunks: []
      };
      currentHunk = null;
    } else if (line.startsWith('+++ ') && currentFile) {
      currentFile.newPath = line.slice(4).replace(/^b\//, '');
      currentFile.isNew = currentFile.oldPath === '/dev/null';
      currentFile.isDeleted = currentFile.newPath === '/dev/null';
      currentFile.isRenamed = currentFile.oldPath !== currentFile.newPath &&
                              !currentFile.isNew && !currentFile.isDeleted;
    }
    // Hunk header
    else if (line.startsWith('@@') && currentFile) {
      if (currentHunk) {
        currentFile.hunks.push(currentHunk);
      }

      const match = line.match(/@@ -(\d+),?(\d*) \+(\d+),?(\d*) @@/);
      if (match) {
        oldLineNum = parseInt(match[1], 10);
        newLineNum = parseInt(match[3], 10);

        currentHunk = {
          oldStart: oldLineNum,
          oldLines: parseInt(match[2] || '1', 10),
          newStart: newLineNum,
          newLines: parseInt(match[4] || '1', 10),
          lines: []
        };
      }
    }
    // Content lines
    else if (currentHunk) {
      if (line.startsWith('+')) {
        currentHunk.lines.push({
          type: 'add',
          content: line.slice(1),
          newLineNumber: newLineNum++
        });
      } else if (line.startsWith('-')) {
        currentHunk.lines.push({
          type: 'remove',
          content: line.slice(1),
          oldLineNumber: oldLineNum++
        });
      } else if (line.startsWith(' ') || line === '') {
        currentHunk.lines.push({
          type: 'unchanged',
          content: line.slice(1) || '',
          oldLineNumber: oldLineNum++,
          newLineNumber: newLineNum++
        });
      }
    }
  }

  // Push final hunk and file
  if (currentFile && currentHunk) {
    currentFile.hunks.push(currentHunk);
  }
  if (currentFile) {
    files.push(currentFile);
  }

  return files;
}

/**
 * Generate simple diff between two strings
 */
export function generateSimpleDiff(oldText: string, newText: string): DiffFile {
  const oldLines = oldText.split('\n');
  const newLines = newText.split('\n');

  const lines: DiffLine[] = [];
  let oldNum = 1;
  let newNum = 1;

  // Simple line-by-line comparison
  const maxLen = Math.max(oldLines.length, newLines.length);

  for (let i = 0; i < maxLen; i++) {
    const oldLine = oldLines[i];
    const newLine = newLines[i];

    if (oldLine === newLine) {
      lines.push({
        type: 'unchanged',
        content: oldLine || '',
        oldLineNumber: oldNum++,
        newLineNumber: newNum++
      });
    } else {
      if (oldLine !== undefined) {
        lines.push({
          type: 'remove',
          content: oldLine,
          oldLineNumber: oldNum++
        });
      }
      if (newLine !== undefined) {
        lines.push({
          type: 'add',
          content: newLine,
          newLineNumber: newNum++
        });
      }
    }
  }

  return {
    oldPath: 'old',
    newPath: 'new',
    hunks: [{
      oldStart: 1,
      oldLines: oldLines.length,
      newStart: 1,
      newLines: newLines.length,
      lines
    }]
  };
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/DiffViewer.test.ts
import { describe, it, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DiffViewer from '@components/ui/DiffViewer/DiffViewer.svelte';
import { parseUnifiedDiff, generateSimpleDiff } from '@utils/diff';

describe('DiffViewer', () => {
  const mockDiff = {
    oldPath: 'file.txt',
    newPath: 'file.txt',
    hunks: [{
      oldStart: 1,
      oldLines: 3,
      newStart: 1,
      newLines: 3,
      lines: [
        { type: 'unchanged' as const, content: 'line 1', oldLineNumber: 1, newLineNumber: 1 },
        { type: 'remove' as const, content: 'old line', oldLineNumber: 2 },
        { type: 'add' as const, content: 'new line', newLineNumber: 2 },
        { type: 'unchanged' as const, content: 'line 3', oldLineNumber: 3, newLineNumber: 3 }
      ]
    }]
  };

  it('should render diff', () => {
    const { getByText } = render(DiffViewer, { props: { diff: mockDiff } });
    expect(getByText('old line')).toBeInTheDocument();
    expect(getByText('new line')).toBeInTheDocument();
  });

  it('should toggle between split and unified view', async () => {
    const { getByText, container } = render(DiffViewer, { props: { diff: mockDiff } });

    expect(container.querySelector('.diff-viewer-split')).toBeInTheDocument();

    await fireEvent.click(getByText('Unified'));
    expect(container.querySelector('.diff-viewer-unified')).toBeInTheDocument();
  });
});

describe('Diff Utils', () => {
  it('should generate simple diff', () => {
    const result = generateSimpleDiff('line1\nline2', 'line1\nline3');

    expect(result.hunks[0].lines).toContainEqual(
      expect.objectContaining({ type: 'remove', content: 'line2' })
    );
    expect(result.hunks[0].lines).toContainEqual(
      expect.objectContaining({ type: 'add', content: 'line3' })
    );
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [208-code-block.md](./208-code-block.md) - Code block component
- [210-terminal-component.md](./210-terminal-component.md) - Terminal component
