# 249 - Spec Diff Viewer

**Phase:** 11 - Spec Browser UI
**Spec ID:** 249
**Status:** Planned
**Dependencies:** 248-spec-version-history
**Estimated Context:** ~11% of Sonnet window

---

## Objective

Create a comprehensive diff viewer component for comparing spec content changes with syntax highlighting, side-by-side or unified views, and inline commenting capability.

---

## Acceptance Criteria

- [ ] Unified and split diff views
- [ ] Syntax highlighting for code blocks
- [ ] Line-by-line navigation
- [ ] Inline comments on changes
- [ ] Word-level diff highlighting
- [ ] Collapse unchanged sections
- [ ] Copy diff to clipboard

---

## Implementation Details

### 1. Types (src/lib/types/spec-diff.ts)

```typescript
export type DiffViewMode = 'unified' | 'split';

export interface DiffOptions {
  viewMode: DiffViewMode;
  contextLines: number;
  showWordDiff: boolean;
  collapseUnchanged: boolean;
  syntaxHighlight: boolean;
}

export interface DiffFile {
  oldPath: string;
  newPath: string;
  oldContent: string;
  newContent: string;
  hunks: DiffHunk[];
  stats: DiffStats;
}

export interface DiffHunk {
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  changes: DiffChange[];
}

export interface DiffChange {
  type: 'add' | 'del' | 'normal';
  content: string;
  oldLine?: number;
  newLine?: number;
  wordDiff?: WordDiff[];
}

export interface WordDiff {
  type: 'add' | 'del' | 'normal';
  value: string;
}

export interface DiffStats {
  additions: number;
  deletions: number;
  changes: number;
}

export interface DiffComment {
  id: string;
  lineNumber: number;
  side: 'old' | 'new';
  author: string;
  content: string;
  timestamp: string;
  resolved: boolean;
}
```

### 2. Spec Diff Viewer Component (src/lib/components/spec-browser/SpecDiffViewer.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type {
    DiffFile,
    DiffHunk,
    DiffChange,
    DiffOptions,
    DiffComment,
    DiffViewMode,
  } from '$lib/types/spec-diff';
  import { ipcRenderer } from '$lib/ipc';

  export let oldContent: string;
  export let newContent: string;
  export let oldPath = 'Original';
  export let newPath = 'Modified';

  const dispatch = createEventDispatcher<{
    addComment: { line: number; side: 'old' | 'new' };
  }>();

  let diff: DiffFile | null = null;
  let options: DiffOptions = {
    viewMode: 'unified',
    contextLines: 3,
    showWordDiff: true,
    collapseUnchanged: true,
    syntaxHighlight: true,
  };
  let comments: DiffComment[] = [];
  let expandedHunks: Set<number> = new Set();
  let activeCommentLine: { line: number; side: 'old' | 'new' } | null = null;
  let newCommentText = '';

  async function computeDiff() {
    diff = await ipcRenderer.invoke('diff:compute', {
      oldContent,
      newContent,
      oldPath,
      newPath,
      options,
    });
  }

  function toggleViewMode() {
    options.viewMode = options.viewMode === 'unified' ? 'split' : 'unified';
  }

  function toggleHunk(index: number) {
    if (expandedHunks.has(index)) {
      expandedHunks.delete(index);
    } else {
      expandedHunks.add(index);
    }
    expandedHunks = new Set(expandedHunks);
  }

  function expandAllHunks() {
    if (diff) {
      expandedHunks = new Set(diff.hunks.map((_, i) => i));
    }
  }

  function collapseAllHunks() {
    expandedHunks = new Set();
  }

  function startComment(line: number, side: 'old' | 'new') {
    activeCommentLine = { line, side };
    newCommentText = '';
  }

  async function submitComment() {
    if (!activeCommentLine || !newCommentText.trim()) return;

    const comment: DiffComment = {
      id: crypto.randomUUID(),
      lineNumber: activeCommentLine.line,
      side: activeCommentLine.side,
      author: 'Current User',
      content: newCommentText.trim(),
      timestamp: new Date().toISOString(),
      resolved: false,
    };

    comments = [...comments, comment];
    activeCommentLine = null;
    newCommentText = '';
  }

  function cancelComment() {
    activeCommentLine = null;
    newCommentText = '';
  }

  async function copyDiff() {
    if (!diff) return;
    const text = formatDiffAsText(diff);
    await navigator.clipboard.writeText(text);
  }

  function formatDiffAsText(diff: DiffFile): string {
    let output = `--- ${diff.oldPath}\n+++ ${diff.newPath}\n`;
    for (const hunk of diff.hunks) {
      output += `${hunk.header}\n`;
      for (const change of hunk.changes) {
        const prefix = change.type === 'add' ? '+' : change.type === 'del' ? '-' : ' ';
        output += `${prefix}${change.content}\n`;
      }
    }
    return output;
  }

  function getLineComments(line: number, side: 'old' | 'new'): DiffComment[] {
    return comments.filter(c => c.lineNumber === line && c.side === side);
  }

  function renderWordDiff(change: DiffChange): string {
    if (!change.wordDiff || !options.showWordDiff) {
      return escapeHtml(change.content);
    }

    return change.wordDiff.map(word => {
      const escaped = escapeHtml(word.value);
      if (word.type === 'add') {
        return `<span class="word-add">${escaped}</span>`;
      } else if (word.type === 'del') {
        return `<span class="word-del">${escaped}</span>`;
      }
      return escaped;
    }).join('');
  }

  function escapeHtml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  onMount(computeDiff);

  $: if (oldContent !== undefined || newContent !== undefined) {
    computeDiff();
  }
</script>

<div class="diff-viewer">
  <header class="diff-viewer__header">
    <div class="diff-stats">
      {#if diff}
        <span class="stat additions">+{diff.stats.additions}</span>
        <span class="stat deletions">-{diff.stats.deletions}</span>
      {/if}
    </div>

    <div class="diff-options">
      <button
        class="option-btn"
        class:active={options.viewMode === 'split'}
        on:click={toggleViewMode}
        title="Toggle view mode"
      >
        {options.viewMode === 'unified' ? 'Split' : 'Unified'}
      </button>

      <button
        class="option-btn"
        class:active={options.showWordDiff}
        on:click={() => { options.showWordDiff = !options.showWordDiff; }}
        title="Toggle word diff"
      >
        Word Diff
      </button>

      <button
        class="option-btn"
        on:click={expandAllHunks}
        title="Expand all"
      >
        Expand All
      </button>

      <button
        class="option-btn"
        on:click={collapseAllHunks}
        title="Collapse all"
      >
        Collapse All
      </button>

      <button
        class="option-btn"
        on:click={copyDiff}
        title="Copy diff"
      >
        Copy
      </button>
    </div>
  </header>

  <div class="diff-viewer__paths">
    <div class="path old-path">{oldPath}</div>
    {#if options.viewMode === 'split'}
      <div class="path new-path">{newPath}</div>
    {/if}
  </div>

  {#if !diff}
    <div class="diff-viewer__loading">Computing diff...</div>
  {:else if diff.hunks.length === 0}
    <div class="diff-viewer__empty">No changes detected</div>
  {:else}
    <div class="diff-content" class:split-view={options.viewMode === 'split'}>
      {#each diff.hunks as hunk, hunkIndex}
        <div class="diff-hunk">
          <button
            class="hunk-header"
            on:click={() => toggleHunk(hunkIndex)}
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 12 12"
              class:expanded={expandedHunks.has(hunkIndex)}
            >
              <path d="M4 2l4 4-4 4" stroke="currentColor" stroke-width="1.5" fill="none"/>
            </svg>
            <span class="hunk-info">{hunk.header}</span>
          </button>

          {#if expandedHunks.has(hunkIndex) || !options.collapseUnchanged}
            <div class="hunk-content">
              {#if options.viewMode === 'unified'}
                {#each hunk.changes as change}
                  <div class="diff-line diff-line--{change.type}">
                    <span class="line-number old">{change.oldLine || ''}</span>
                    <span class="line-number new">{change.newLine || ''}</span>
                    <span class="line-prefix">
                      {change.type === 'add' ? '+' : change.type === 'del' ? '-' : ' '}
                    </span>
                    <span class="line-content">
                      {@html renderWordDiff(change)}
                    </span>
                    <button
                      class="comment-btn"
                      on:click={() => startComment(
                        change.newLine || change.oldLine || 0,
                        change.type === 'del' ? 'old' : 'new'
                      )}
                      title="Add comment"
                    >
                      +
                    </button>
                  </div>

                  {#each getLineComments(change.newLine || change.oldLine || 0, change.type === 'del' ? 'old' : 'new') as comment}
                    <div class="line-comment">
                      <div class="comment-header">
                        <span class="comment-author">{comment.author}</span>
                        <span class="comment-time">
                          {new Date(comment.timestamp).toLocaleString()}
                        </span>
                      </div>
                      <div class="comment-content">{comment.content}</div>
                    </div>
                  {/each}

                  {#if activeCommentLine?.line === (change.newLine || change.oldLine) &&
                       activeCommentLine?.side === (change.type === 'del' ? 'old' : 'new')}
                    <div class="comment-input">
                      <textarea
                        bind:value={newCommentText}
                        placeholder="Add a comment..."
                        rows="2"
                      />
                      <div class="comment-actions">
                        <button class="cancel-btn" on:click={cancelComment}>
                          Cancel
                        </button>
                        <button class="submit-btn" on:click={submitComment}>
                          Comment
                        </button>
                      </div>
                    </div>
                  {/if}
                {/each}
              {:else}
                <!-- Split view -->
                <div class="split-container">
                  <div class="split-pane old-pane">
                    {#each hunk.changes.filter(c => c.type !== 'add') as change}
                      <div class="diff-line diff-line--{change.type}">
                        <span class="line-number">{change.oldLine || ''}</span>
                        <span class="line-content">
                          {@html renderWordDiff(change)}
                        </span>
                      </div>
                    {/each}
                  </div>
                  <div class="split-pane new-pane">
                    {#each hunk.changes.filter(c => c.type !== 'del') as change}
                      <div class="diff-line diff-line--{change.type}">
                        <span class="line-number">{change.newLine || ''}</span>
                        <span class="line-content">
                          {@html renderWordDiff(change)}
                        </span>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .diff-viewer {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
  }

  .diff-viewer__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .diff-stats {
    display: flex;
    gap: 12px;
    font-family: monospace;
    font-size: 13px;
  }

  .stat.additions {
    color: var(--color-success);
  }

  .stat.deletions {
    color: var(--color-error);
  }

  .diff-options {
    display: flex;
    gap: 8px;
  }

  .option-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .option-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .option-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .diff-viewer__paths {
    display: flex;
    border-bottom: 1px solid var(--color-border);
  }

  .path {
    flex: 1;
    padding: 8px 16px;
    font-family: monospace;
    font-size: 12px;
    color: var(--color-text-muted);
    background: var(--color-bg-secondary);
  }

  .old-path {
    border-right: 1px solid var(--color-border);
  }

  .diff-viewer__loading,
  .diff-viewer__empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }

  .diff-content {
    flex: 1;
    overflow-y: auto;
  }

  .diff-hunk {
    border-bottom: 1px solid var(--color-border);
  }

  .diff-hunk:last-child {
    border-bottom: none;
  }

  .hunk-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px;
    border: none;
    background: var(--color-bg-secondary);
    color: var(--color-text-muted);
    font-family: monospace;
    font-size: 12px;
    cursor: pointer;
    text-align: left;
  }

  .hunk-header:hover {
    background: var(--color-bg-hover);
  }

  .hunk-header svg {
    transition: transform 0.15s ease;
  }

  .hunk-header svg.expanded {
    transform: rotate(90deg);
  }

  .hunk-content {
    font-family: monospace;
    font-size: 13px;
    line-height: 20px;
  }

  .diff-line {
    display: flex;
    align-items: stretch;
  }

  .diff-line--add {
    background: rgba(76, 175, 80, 0.1);
  }

  .diff-line--del {
    background: rgba(244, 67, 54, 0.1);
  }

  .line-number {
    width: 50px;
    padding: 0 8px;
    text-align: right;
    color: var(--color-text-muted);
    background: var(--color-bg-secondary);
    user-select: none;
    border-right: 1px solid var(--color-border);
  }

  .diff-line .line-number.old {
    width: 40px;
  }

  .diff-line .line-number.new {
    width: 40px;
  }

  .line-prefix {
    width: 20px;
    text-align: center;
    user-select: none;
  }

  .diff-line--add .line-prefix {
    color: var(--color-success);
  }

  .diff-line--del .line-prefix {
    color: var(--color-error);
  }

  .line-content {
    flex: 1;
    padding: 0 8px;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .line-content :global(.word-add) {
    background: rgba(76, 175, 80, 0.3);
    padding: 1px 2px;
    border-radius: 2px;
  }

  .line-content :global(.word-del) {
    background: rgba(244, 67, 54, 0.3);
    padding: 1px 2px;
    border-radius: 2px;
    text-decoration: line-through;
  }

  .comment-btn {
    width: 24px;
    opacity: 0;
    border: none;
    background: var(--color-primary);
    color: white;
    cursor: pointer;
    transition: opacity 0.1s ease;
  }

  .diff-line:hover .comment-btn {
    opacity: 1;
  }

  .line-comment {
    margin: 8px 16px;
    padding: 12px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
  }

  .comment-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 12px;
  }

  .comment-author {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .comment-time {
    color: var(--color-text-muted);
  }

  .comment-content {
    font-size: 13px;
    color: var(--color-text-secondary);
    font-family: inherit;
  }

  .comment-input {
    margin: 8px 16px;
    padding: 12px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-primary);
    border-radius: 6px;
  }

  .comment-input textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-family: inherit;
    font-size: 13px;
    resize: vertical;
  }

  .comment-input textarea:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .comment-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }

  .cancel-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .submit-btn {
    padding: 6px 12px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  /* Split view */
  .split-container {
    display: flex;
  }

  .split-pane {
    flex: 1;
    overflow-x: auto;
  }

  .split-pane.old-pane {
    border-right: 1px solid var(--color-border);
  }

  .split-pane .diff-line {
    padding-right: 8px;
  }

  .split-pane .line-number {
    width: 50px;
  }
</style>
```

---

## Testing Requirements

1. Unified diff displays correctly
2. Split view works
3. Word diff highlighting works
4. Comments can be added
5. Copy exports correct format
6. Hunks expand/collapse

---

## Related Specs

- Depends on: [248-spec-version-history.md](248-spec-version-history.md)
- Next: [250-spec-creation.md](250-spec-creation.md)
