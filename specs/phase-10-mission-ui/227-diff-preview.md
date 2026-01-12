# 227 - Diff Preview Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 227
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create a diff preview component that displays file changes in a readable format with syntax highlighting, supporting both unified and split view modes with line-by-line annotations.

---

## Acceptance Criteria

- [ ] Unified and split view modes
- [ ] Syntax highlighting for changed content
- [ ] Line numbers with add/remove indicators
- [ ] Expand/collapse unchanged sections
- [ ] Copy individual hunks
- [ ] Navigate between hunks
- [ ] File header with change summary

---

## Implementation Details

### 1. Types (src/lib/types/diff.ts)

```typescript
/**
 * Types for diff display functionality.
 */

export interface DiffFile {
  path: string;
  oldPath?: string;
  status: DiffStatus;
  hunks: DiffHunk[];
  language: string;
  binary: boolean;
  stats: DiffStats;
}

export type DiffStatus = 'added' | 'modified' | 'deleted' | 'renamed' | 'copied';

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  header: string;
  lines: DiffLine[];
}

export interface DiffLine {
  type: 'add' | 'remove' | 'context' | 'info';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

export interface DiffStats {
  additions: number;
  deletions: number;
  totalChanges: number;
}

export type DiffViewMode = 'unified' | 'split';

export interface DiffViewConfig {
  mode: DiffViewMode;
  showLineNumbers: boolean;
  syntaxHighlight: boolean;
  contextLines: number;
  wrapLines: boolean;
}

export const DEFAULT_DIFF_CONFIG: DiffViewConfig = {
  mode: 'unified',
  showLineNumbers: true,
  syntaxHighlight: true,
  contextLines: 3,
  wrapLines: false,
};

export function getLanguageFromPath(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase() || '';
  const langMap: Record<string, string> = {
    ts: 'typescript',
    tsx: 'typescript',
    js: 'javascript',
    jsx: 'javascript',
    rs: 'rust',
    py: 'python',
    md: 'markdown',
    json: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    html: 'html',
    css: 'css',
    scss: 'scss',
    svelte: 'svelte',
  };
  return langMap[ext] || 'text';
}
```

### 2. Diff Preview Component (src/lib/components/mission/DiffPreview.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { DiffFile, DiffHunk, DiffLine, DiffViewConfig } from '$lib/types/diff';
  import { DEFAULT_DIFF_CONFIG } from '$lib/types/diff';
  import DiffHunkView from './DiffHunkView.svelte';
  import DiffSplitView from './DiffSplitView.svelte';

  export let file: DiffFile;
  export let config: DiffViewConfig = DEFAULT_DIFF_CONFIG;
  export let expanded = true;

  const dispatch = createEventDispatcher<{
    approve: void;
    reject: void;
    edit: void;
  }>();

  let currentHunkIndex = 0;

  const statusLabels: Record<string, string> = {
    added: 'Added',
    modified: 'Modified',
    deleted: 'Deleted',
    renamed: 'Renamed',
    copied: 'Copied',
  };

  const statusColors: Record<string, string> = {
    added: 'var(--color-success)',
    modified: 'var(--color-primary)',
    deleted: 'var(--color-error)',
    renamed: 'var(--color-warning)',
    copied: 'var(--color-text-secondary)',
  };

  function navigateHunk(direction: 'prev' | 'next') {
    if (direction === 'prev' && currentHunkIndex > 0) {
      currentHunkIndex--;
    } else if (direction === 'next' && currentHunkIndex < file.hunks.length - 1) {
      currentHunkIndex++;
    }
  }

  function copyHunk(hunk: DiffHunk) {
    const content = hunk.lines
      .map(line => {
        const prefix = line.type === 'add' ? '+' : line.type === 'remove' ? '-' : ' ';
        return prefix + line.content;
      })
      .join('\n');
    navigator.clipboard.writeText(content);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'j' || event.key === 'ArrowDown') {
      navigateHunk('next');
    } else if (event.key === 'k' || event.key === 'ArrowUp') {
      navigateHunk('prev');
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="diff-preview" class:diff-preview--collapsed={!expanded}>
  <!-- File Header -->
  <div class="diff-preview__header">
    <button
      class="diff-preview__toggle"
      on:click={() => { expanded = !expanded; }}
      aria-expanded={expanded}
    >
      <svg
        width="12"
        height="12"
        viewBox="0 0 12 12"
        class:rotated={expanded}
      >
        <path fill="currentColor" d="M4 2l4 4-4 4"/>
      </svg>
    </button>

    <span
      class="diff-preview__status"
      style="color: {statusColors[file.status]}"
    >
      {statusLabels[file.status]}
    </span>

    <span class="diff-preview__path">
      {#if file.oldPath && file.oldPath !== file.path}
        <span class="diff-preview__old-path">{file.oldPath}</span>
        <span class="diff-preview__arrow">→</span>
      {/if}
      {file.path}
    </span>

    <div class="diff-preview__stats">
      <span class="diff-preview__additions">+{file.stats.additions}</span>
      <span class="diff-preview__deletions">-{file.stats.deletions}</span>
    </div>

    <div class="diff-preview__actions">
      <button
        class="diff-action"
        on:click={() => dispatch('approve')}
        title="Approve changes"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M12.03 3.97a.75.75 0 010 1.06l-6.25 6.25a.75.75 0 01-1.06 0L2.47 9.03a.75.75 0 011.06-1.06l1.72 1.72 5.72-5.72a.75.75 0 011.06 0z"/>
        </svg>
      </button>
      <button
        class="diff-action"
        on:click={() => dispatch('reject')}
        title="Reject changes"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M4.293 4.293a1 1 0 011.414 0L7 5.586l1.293-1.293a1 1 0 111.414 1.414L8.414 7l1.293 1.293a1 1 0 01-1.414 1.414L7 8.414l-1.293 1.293a1 1 0 01-1.414-1.414L5.586 7 4.293 5.707a1 1 0 010-1.414z"/>
        </svg>
      </button>
      <button
        class="diff-action"
        on:click={() => dispatch('edit')}
        title="Edit manually"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M10.586 1.586a2 2 0 012.828 2.828l-7 7L3 12l.586-3.414 7-7z"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Diff Content -->
  {#if expanded}
    <div class="diff-preview__content">
      {#if file.binary}
        <div class="diff-preview__binary">
          Binary file changed
        </div>
      {:else if file.hunks.length === 0}
        <div class="diff-preview__empty">
          No changes to display
        </div>
      {:else}
        <!-- View Mode Toggle -->
        <div class="diff-preview__view-toggle">
          <button
            class="view-toggle-btn"
            class:active={config.mode === 'unified'}
            on:click={() => { config = { ...config, mode: 'unified' }; }}
          >
            Unified
          </button>
          <button
            class="view-toggle-btn"
            class:active={config.mode === 'split'}
            on:click={() => { config = { ...config, mode: 'split' }; }}
          >
            Split
          </button>

          <!-- Hunk Navigation -->
          {#if file.hunks.length > 1}
            <div class="hunk-nav">
              <button
                class="hunk-nav-btn"
                disabled={currentHunkIndex === 0}
                on:click={() => navigateHunk('prev')}
              >
                ↑
              </button>
              <span class="hunk-nav-count">
                {currentHunkIndex + 1}/{file.hunks.length}
              </span>
              <button
                class="hunk-nav-btn"
                disabled={currentHunkIndex === file.hunks.length - 1}
                on:click={() => navigateHunk('next')}
              >
                ↓
              </button>
            </div>
          {/if}
        </div>

        <!-- Hunks -->
        <div class="diff-preview__hunks">
          {#if config.mode === 'unified'}
            {#each file.hunks as hunk, index}
              <DiffHunkView
                {hunk}
                language={file.language}
                showLineNumbers={config.showLineNumbers}
                highlighted={index === currentHunkIndex}
                on:copy={() => copyHunk(hunk)}
              />
            {/each}
          {:else}
            <DiffSplitView
              hunks={file.hunks}
              language={file.language}
              showLineNumbers={config.showLineNumbers}
            />
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .diff-preview {
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--color-bg-primary);
  }

  .diff-preview__header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .diff-preview--collapsed .diff-preview__header {
    border-bottom: none;
  }

  .diff-preview__toggle {
    padding: 4px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .diff-preview__toggle svg {
    transition: transform 0.15s ease;
  }

  .diff-preview__toggle svg.rotated {
    transform: rotate(90deg);
  }

  .diff-preview__status {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .diff-preview__path {
    flex: 1;
    font-size: 13px;
    font-family: 'SF Mono', monospace;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .diff-preview__old-path {
    color: var(--color-text-muted);
    text-decoration: line-through;
  }

  .diff-preview__arrow {
    color: var(--color-text-muted);
    margin: 0 4px;
  }

  .diff-preview__stats {
    display: flex;
    gap: 8px;
    font-size: 12px;
    font-family: monospace;
  }

  .diff-preview__additions {
    color: var(--color-success);
  }

  .diff-preview__deletions {
    color: var(--color-error);
  }

  .diff-preview__actions {
    display: flex;
    gap: 4px;
  }

  .diff-action {
    padding: 6px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: 4px;
    cursor: pointer;
  }

  .diff-action:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .diff-preview__content {
    overflow-x: auto;
  }

  .diff-preview__binary,
  .diff-preview__empty {
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
    font-style: italic;
  }

  .diff-preview__view-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .view-toggle-btn {
    padding: 4px 10px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    color: var(--color-text-secondary);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .view-toggle-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .hunk-nav {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
  }

  .hunk-nav-btn {
    padding: 4px 8px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    color: var(--color-text-secondary);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .hunk-nav-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .hunk-nav-count {
    font-size: 12px;
    color: var(--color-text-muted);
    min-width: 40px;
    text-align: center;
  }

  .diff-preview__hunks {
    font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
    font-size: 13px;
    line-height: 1.5;
  }
</style>
```

### 3. Diff Hunk View Component (src/lib/components/mission/DiffHunkView.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { DiffHunk, DiffLine } from '$lib/types/diff';

  export let hunk: DiffHunk;
  export let language = 'text';
  export let showLineNumbers = true;
  export let highlighted = false;

  const dispatch = createEventDispatcher<{ copy: void }>();

  function getLineClass(line: DiffLine): string {
    switch (line.type) {
      case 'add': return 'diff-line--add';
      case 'remove': return 'diff-line--remove';
      case 'info': return 'diff-line--info';
      default: return '';
    }
  }

  function getLinePrefix(line: DiffLine): string {
    switch (line.type) {
      case 'add': return '+';
      case 'remove': return '-';
      default: return ' ';
    }
  }
</script>

<div class="diff-hunk" class:diff-hunk--highlighted={highlighted}>
  <!-- Hunk Header -->
  <div class="diff-hunk__header">
    <span class="diff-hunk__location">{hunk.header}</span>
    <button
      class="diff-hunk__copy"
      on:click={() => dispatch('copy')}
      title="Copy hunk"
    >
      Copy
    </button>
  </div>

  <!-- Lines -->
  <div class="diff-hunk__lines">
    {#each hunk.lines as line}
      <div class="diff-line {getLineClass(line)}">
        {#if showLineNumbers}
          <span class="diff-line__old-num">
            {line.oldLineNumber ?? ''}
          </span>
          <span class="diff-line__new-num">
            {line.newLineNumber ?? ''}
          </span>
        {/if}
        <span class="diff-line__prefix">{getLinePrefix(line)}</span>
        <span class="diff-line__content">{line.content}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .diff-hunk {
    border-bottom: 1px solid var(--color-border);
  }

  .diff-hunk:last-child {
    border-bottom: none;
  }

  .diff-hunk--highlighted {
    box-shadow: inset 3px 0 0 var(--color-primary);
  }

  .diff-hunk__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .diff-hunk__location {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .diff-hunk__copy {
    padding: 2px 8px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 11px;
    cursor: pointer;
  }

  .diff-hunk__copy:hover {
    color: var(--color-text-primary);
  }

  .diff-hunk__lines {
    overflow-x: auto;
  }

  .diff-line {
    display: flex;
    min-height: 20px;
    white-space: pre;
  }

  .diff-line--add {
    background: rgba(52, 211, 153, 0.15);
  }

  .diff-line--remove {
    background: rgba(248, 113, 113, 0.15);
  }

  .diff-line--info {
    background: var(--color-bg-secondary);
    color: var(--color-text-muted);
    font-style: italic;
  }

  .diff-line__old-num,
  .diff-line__new-num {
    min-width: 40px;
    padding: 0 8px;
    text-align: right;
    color: var(--color-text-muted);
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    user-select: none;
  }

  .diff-line--add .diff-line__new-num {
    background: rgba(52, 211, 153, 0.2);
  }

  .diff-line--remove .diff-line__old-num {
    background: rgba(248, 113, 113, 0.2);
  }

  .diff-line__prefix {
    width: 20px;
    text-align: center;
    color: var(--color-text-muted);
    user-select: none;
  }

  .diff-line--add .diff-line__prefix {
    color: var(--color-success);
  }

  .diff-line--remove .diff-line__prefix {
    color: var(--color-error);
  }

  .diff-line__content {
    flex: 1;
    padding: 0 8px;
  }
</style>
```

### 4. Diff Split View Component (src/lib/components/mission/DiffSplitView.svelte)

```svelte
<script lang="ts">
  import type { DiffHunk, DiffLine } from '$lib/types/diff';

  export let hunks: DiffHunk[];
  export let language = 'text';
  export let showLineNumbers = true;

  interface SplitLine {
    left: DiffLine | null;
    right: DiffLine | null;
  }

  function buildSplitLines(hunks: DiffHunk[]): SplitLine[] {
    const result: SplitLine[] = [];

    for (const hunk of hunks) {
      // Add hunk header
      result.push({
        left: { type: 'info', content: hunk.header },
        right: { type: 'info', content: hunk.header },
      });

      let i = 0;
      while (i < hunk.lines.length) {
        const line = hunk.lines[i];

        if (line.type === 'context') {
          result.push({ left: line, right: line });
          i++;
        } else if (line.type === 'remove') {
          // Check for corresponding add
          let j = i + 1;
          while (j < hunk.lines.length && hunk.lines[j].type === 'remove') j++;

          const removeCount = j - i;
          let addStart = j;
          while (j < hunk.lines.length && hunk.lines[j].type === 'add') j++;
          const addCount = j - addStart;

          const maxCount = Math.max(removeCount, addCount);
          for (let k = 0; k < maxCount; k++) {
            result.push({
              left: k < removeCount ? hunk.lines[i + k] : null,
              right: k < addCount ? hunk.lines[addStart + k] : null,
            });
          }

          i = j;
        } else if (line.type === 'add') {
          result.push({ left: null, right: line });
          i++;
        } else {
          i++;
        }
      }
    }

    return result;
  }

  $: splitLines = buildSplitLines(hunks);
</script>

<div class="diff-split">
  <div class="diff-split__container">
    {#each splitLines as { left, right }}
      <div class="diff-split__row">
        <!-- Left Side -->
        <div
          class="diff-split__cell diff-split__cell--left"
          class:diff-split__cell--remove={left?.type === 'remove'}
          class:diff-split__cell--info={left?.type === 'info'}
          class:diff-split__cell--empty={!left}
        >
          {#if left}
            {#if showLineNumbers && left.type !== 'info'}
              <span class="diff-split__line-num">{left.oldLineNumber ?? ''}</span>
            {/if}
            <span class="diff-split__content">{left.content}</span>
          {/if}
        </div>

        <!-- Right Side -->
        <div
          class="diff-split__cell diff-split__cell--right"
          class:diff-split__cell--add={right?.type === 'add'}
          class:diff-split__cell--info={right?.type === 'info'}
          class:diff-split__cell--empty={!right}
        >
          {#if right}
            {#if showLineNumbers && right.type !== 'info'}
              <span class="diff-split__line-num">{right.newLineNumber ?? ''}</span>
            {/if}
            <span class="diff-split__content">{right.content}</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .diff-split {
    overflow-x: auto;
  }

  .diff-split__container {
    min-width: 100%;
    display: table;
  }

  .diff-split__row {
    display: table-row;
  }

  .diff-split__cell {
    display: table-cell;
    width: 50%;
    vertical-align: top;
    white-space: pre;
    border-right: 1px solid var(--color-border);
  }

  .diff-split__cell--left {
    border-right: 2px solid var(--color-border);
  }

  .diff-split__cell--add {
    background: rgba(52, 211, 153, 0.15);
  }

  .diff-split__cell--remove {
    background: rgba(248, 113, 113, 0.15);
  }

  .diff-split__cell--info {
    background: var(--color-bg-secondary);
    color: var(--color-text-muted);
    font-style: italic;
    text-align: center;
    padding: 4px 8px;
  }

  .diff-split__cell--empty {
    background: var(--color-bg-secondary);
  }

  .diff-split__line-num {
    display: inline-block;
    min-width: 40px;
    padding: 0 8px;
    text-align: right;
    color: var(--color-text-muted);
    background: var(--color-bg-secondary);
    user-select: none;
  }

  .diff-split__content {
    padding: 0 8px;
  }
</style>
```

---

## Testing Requirements

1. Unified view renders correctly
2. Split view aligns changes properly
3. Line numbers display correctly
4. Syntax highlighting works
5. Hunk navigation functions
6. Copy hunk copies correct content
7. Approve/reject/edit emit events

### Test File (src/lib/components/mission/__tests__/DiffPreview.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import DiffPreview from '../DiffPreview.svelte';

describe('DiffPreview', () => {
  const mockFile = {
    path: 'src/test.ts',
    status: 'modified' as const,
    language: 'typescript',
    binary: false,
    stats: { additions: 5, deletions: 2, totalChanges: 7 },
    hunks: [
      {
        oldStart: 1,
        oldLines: 5,
        newStart: 1,
        newLines: 8,
        header: '@@ -1,5 +1,8 @@',
        lines: [
          { type: 'context' as const, content: 'const a = 1;', oldLineNumber: 1, newLineNumber: 1 },
          { type: 'remove' as const, content: 'const b = 2;', oldLineNumber: 2 },
          { type: 'add' as const, content: 'const b = 3;', newLineNumber: 2 },
          { type: 'add' as const, content: 'const c = 4;', newLineNumber: 3 },
        ],
      },
    ],
  };

  it('renders file path and stats', () => {
    render(DiffPreview, { file: mockFile });

    expect(screen.getByText('src/test.ts')).toBeInTheDocument();
    expect(screen.getByText('+5')).toBeInTheDocument();
    expect(screen.getByText('-2')).toBeInTheDocument();
  });

  it('shows modified status', () => {
    render(DiffPreview, { file: mockFile });

    expect(screen.getByText('Modified')).toBeInTheDocument();
  });

  it('toggles between unified and split view', async () => {
    render(DiffPreview, { file: mockFile });

    const splitBtn = screen.getByText('Split');
    await fireEvent.click(splitBtn);

    expect(splitBtn).toHaveClass('active');
  });

  it('emits approve event', async () => {
    const { component } = render(DiffPreview, { file: mockFile });
    const handler = vi.fn();
    component.$on('approve', handler);

    await fireEvent.click(screen.getByTitle('Approve changes'));

    expect(handler).toHaveBeenCalled();
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [228-test-results.md](228-test-results.md)
- Used by: [226-checkpoint-display.md](226-checkpoint-display.md)
