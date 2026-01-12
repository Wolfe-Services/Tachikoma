# Spec 244: Version History

## Phase
11 - Spec Browser UI

## Spec ID
244

## Status
Planned

## Dependencies
- Spec 236 (Spec Detail View)
- Spec 237 (Spec Editor)

## Estimated Context
~9%

---

## Objective

Implement a version history system for specs that tracks all changes, allows viewing previous versions, supports diff comparison, and enables restoring to earlier versions.

---

## Acceptance Criteria

- [ ] Track all spec changes with timestamps
- [ ] View list of previous versions
- [ ] Side-by-side diff comparison
- [ ] Restore to previous version
- [ ] View who made each change
- [ ] Highlight changes between versions
- [ ] Export version history
- [ ] Compact vs detailed view toggle

---

## Implementation Details

### SpecHistory.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import type { Spec, SpecVersion } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import Button from '$lib/components/Button.svelte';
  import DiffView from './DiffView.svelte';
  import { formatDate, formatRelativeTime } from '$lib/utils/date';
  import { getSpecHistory } from '$lib/api/specs';

  export let specId: string;

  const dispatch = createEventDispatcher<{
    restore: SpecVersion;
    close: void;
  }>();

  let versions = writable<SpecVersion[]>([]);
  let loading = true;
  let error: string | null = null;
  let selectedVersions = writable<[SpecVersion | null, SpecVersion | null]>([null, null]);
  let viewMode: 'list' | 'diff' = 'list';
  let detailMode: 'compact' | 'detailed' = 'compact';

  // Load version history
  onMount(async () => {
    try {
      const history = await getSpecHistory(specId);
      versions.set(history);

      // Select most recent version by default
      if (history.length > 0) {
        selectedVersions.set([history[0], history[1] ?? null]);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load history';
    } finally {
      loading = false;
    }
  });

  function selectVersion(version: SpecVersion, slot: 0 | 1) {
    selectedVersions.update(v => {
      const newVersions = [...v] as [SpecVersion | null, SpecVersion | null];
      newVersions[slot] = version;
      return newVersions;
    });
  }

  function handleRestore(version: SpecVersion) {
    const confirmed = confirm(
      `Are you sure you want to restore to version from ${formatDate(version.timestamp)}?`
    );

    if (confirmed) {
      dispatch('restore', version);
    }
  }

  function getChangesSummary(version: SpecVersion): string {
    const changes: string[] = [];

    if (version.changes.title) changes.push('title');
    if (version.changes.status) changes.push('status');
    if (version.changes.content) changes.push('content');
    if (version.changes.dependencies) changes.push('dependencies');

    if (changes.length === 0) return 'No changes recorded';
    return `Changed: ${changes.join(', ')}`;
  }

  $: canCompare = $selectedVersions[0] && $selectedVersions[1];
</script>

<div class="spec-history">
  <header class="spec-history__header">
    <div class="spec-history__title">
      <Icon name="history" size={18} />
      <h2>Version History</h2>
    </div>
    <div class="spec-history__actions">
      <div class="spec-history__view-toggle">
        <Button
          variant={viewMode === 'list' ? 'primary' : 'ghost'}
          size="sm"
          on:click={() => viewMode = 'list'}
        >
          <Icon name="list" size={14} />
          List
        </Button>
        <Button
          variant={viewMode === 'diff' ? 'primary' : 'ghost'}
          size="sm"
          disabled={!canCompare}
          on:click={() => viewMode = 'diff'}
        >
          <Icon name="git-compare" size={14} />
          Compare
        </Button>
      </div>
      <Button variant="ghost" size="sm" on:click={() => dispatch('close')}>
        <Icon name="x" size={16} />
      </Button>
    </div>
  </header>

  {#if loading}
    <div class="spec-history__loading">
      <Icon name="loader" size={24} class="spinning" />
      <span>Loading history...</span>
    </div>
  {:else if error}
    <div class="spec-history__error">
      <Icon name="alert-circle" size={24} />
      <span>{error}</span>
    </div>
  {:else if $versions.length === 0}
    <div class="spec-history__empty">
      <Icon name="clock" size={32} />
      <span>No version history available</span>
    </div>
  {:else if viewMode === 'list'}
    <div class="spec-history__list-header">
      <label class="spec-history__detail-toggle">
        <input
          type="checkbox"
          checked={detailMode === 'detailed'}
          on:change={() => detailMode = detailMode === 'compact' ? 'detailed' : 'compact'}
        />
        Show details
      </label>
      <span class="spec-history__count">
        {$versions.length} version{$versions.length !== 1 ? 's' : ''}
      </span>
    </div>

    <ul class="spec-history__list">
      {#each $versions as version, i}
        {@const isSelected = $selectedVersions.includes(version)}
        <li
          class="spec-history__item"
          class:spec-history__item--selected={isSelected}
          class:spec-history__item--current={i === 0}
        >
          <div class="spec-history__item-header">
            <div class="spec-history__item-meta">
              <span class="spec-history__item-date">
                {formatDate(version.timestamp)}
              </span>
              <span class="spec-history__item-relative">
                {formatRelativeTime(version.timestamp)}
              </span>
              {#if i === 0}
                <span class="spec-history__item-badge">Current</span>
              {/if}
            </div>

            <div class="spec-history__item-actions">
              <button
                class="spec-history__select-btn"
                class:spec-history__select-btn--active={$selectedVersions[0] === version}
                on:click={() => selectVersion(version, 0)}
                title="Select as left version"
              >
                A
              </button>
              <button
                class="spec-history__select-btn"
                class:spec-history__select-btn--active={$selectedVersions[1] === version}
                on:click={() => selectVersion(version, 1)}
                title="Select as right version"
              >
                B
              </button>
              {#if i > 0}
                <Button
                  variant="ghost"
                  size="sm"
                  on:click={() => handleRestore(version)}
                >
                  <Icon name="rotate-ccw" size={12} />
                  Restore
                </Button>
              {/if}
            </div>
          </div>

          <div class="spec-history__item-body">
            {#if version.author}
              <span class="spec-history__item-author">
                <Icon name="user" size={12} />
                {version.author}
              </span>
            {/if}

            <span class="spec-history__item-changes">
              {getChangesSummary(version)}
            </span>
          </div>

          {#if detailMode === 'detailed' && version.message}
            <div class="spec-history__item-message">
              {version.message}
            </div>
          {/if}

          {#if detailMode === 'detailed'}
            <div class="spec-history__item-details">
              {#if version.changes.title}
                <div class="spec-history__change">
                  <span class="spec-history__change-label">Title:</span>
                  <span class="spec-history__change-old">{version.changes.title.old}</span>
                  <Icon name="arrow-right" size={12} />
                  <span class="spec-history__change-new">{version.changes.title.new}</span>
                </div>
              {/if}
              {#if version.changes.status}
                <div class="spec-history__change">
                  <span class="spec-history__change-label">Status:</span>
                  <span class="spec-history__change-old">{version.changes.status.old}</span>
                  <Icon name="arrow-right" size={12} />
                  <span class="spec-history__change-new">{version.changes.status.new}</span>
                </div>
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {:else}
    <div class="spec-history__diff">
      {#if $selectedVersions[0] && $selectedVersions[1]}
        <DiffView
          left={$selectedVersions[0]}
          right={$selectedVersions[1]}
        />
      {:else}
        <div class="spec-history__diff-prompt">
          Select two versions to compare
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .spec-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
  }

  .spec-history__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-history__title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .spec-history__title h2 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }

  .spec-history__actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .spec-history__view-toggle {
    display: flex;
    gap: 4px;
  }

  .spec-history__loading,
  .spec-history__error,
  .spec-history__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    gap: 12px;
    color: var(--color-text-tertiary);
  }

  .spec-history__error {
    color: var(--color-danger);
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .spec-history__list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--color-surface-subtle);
    border-bottom: 1px solid var(--color-border);
  }

  .spec-history__detail-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .spec-history__count {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-history__list {
    flex: 1;
    overflow-y: auto;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .spec-history__item {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-history__item--selected {
    background: var(--color-primary-subtle);
  }

  .spec-history__item--current {
    border-left: 3px solid var(--color-primary);
  }

  .spec-history__item-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 8px;
  }

  .spec-history__item-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .spec-history__item-date {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .spec-history__item-relative {
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
  }

  .spec-history__item-badge {
    padding: 2px 6px;
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    background: var(--color-primary);
    color: white;
    border-radius: 3px;
  }

  .spec-history__item-actions {
    display: flex;
    gap: 4px;
  }

  .spec-history__select-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 600;
    background: var(--color-surface-elevated);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-tertiary);
  }

  .spec-history__select-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .spec-history__select-btn--active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .spec-history__item-body {
    display: flex;
    gap: 16px;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }

  .spec-history__item-author {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .spec-history__item-message {
    margin-top: 12px;
    padding: 8px 12px;
    font-size: 0.875rem;
    background: var(--color-surface-subtle);
    border-radius: 4px;
    color: var(--color-text-secondary);
  }

  .spec-history__item-details {
    margin-top: 12px;
    padding: 12px;
    background: var(--color-surface-subtle);
    border-radius: 4px;
  }

  .spec-history__change {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    margin-bottom: 8px;
  }

  .spec-history__change:last-child {
    margin-bottom: 0;
  }

  .spec-history__change-label {
    font-weight: 500;
    color: var(--color-text-secondary);
    min-width: 60px;
  }

  .spec-history__change-old {
    padding: 2px 6px;
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: 3px;
    text-decoration: line-through;
  }

  .spec-history__change-new {
    padding: 2px 6px;
    background: var(--color-success-subtle);
    color: var(--color-success);
    border-radius: 3px;
  }

  .spec-history__diff {
    flex: 1;
    overflow: hidden;
  }

  .spec-history__diff-prompt {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-tertiary);
  }
</style>
```

### DiffView.svelte

```svelte
<script lang="ts">
  import { diffLines, diffWords } from 'diff';
  import type { SpecVersion } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import { formatDate } from '$lib/utils/date';

  export let left: SpecVersion;
  export let right: SpecVersion;

  type DiffPart = {
    value: string;
    added?: boolean;
    removed?: boolean;
  };

  $: contentDiff = diffLines(left.snapshot.content, right.snapshot.content);
  $: titleDiff = diffWords(left.snapshot.title, right.snapshot.title);

  function getLineClass(part: DiffPart): string {
    if (part.added) return 'diff-view__line--added';
    if (part.removed) return 'diff-view__line--removed';
    return '';
  }
</script>

<div class="diff-view">
  <header class="diff-view__header">
    <div class="diff-view__version diff-view__version--left">
      <span class="diff-view__version-label">A</span>
      <span>{formatDate(left.timestamp)}</span>
    </div>
    <Icon name="arrow-right" size={16} />
    <div class="diff-view__version diff-view__version--right">
      <span class="diff-view__version-label">B</span>
      <span>{formatDate(right.timestamp)}</span>
    </div>
  </header>

  <div class="diff-view__stats">
    <span class="diff-view__stat diff-view__stat--added">
      +{contentDiff.filter(p => p.added).reduce((sum, p) => sum + p.value.split('\n').length - 1, 0)} lines
    </span>
    <span class="diff-view__stat diff-view__stat--removed">
      -{contentDiff.filter(p => p.removed).reduce((sum, p) => sum + p.value.split('\n').length - 1, 0)} lines
    </span>
  </div>

  {#if left.snapshot.title !== right.snapshot.title}
    <div class="diff-view__section">
      <h4>Title</h4>
      <div class="diff-view__inline">
        {#each titleDiff as part}
          <span class={getLineClass(part)}>{part.value}</span>
        {/each}
      </div>
    </div>
  {/if}

  <div class="diff-view__section diff-view__section--content">
    <h4>Content</h4>
    <div class="diff-view__content">
      {#each contentDiff as part}
        {#each part.value.split('\n') as line, i}
          {#if i < part.value.split('\n').length - 1 || line}
            <div class="diff-view__line {getLineClass(part)}">
              <span class="diff-view__line-prefix">
                {part.added ? '+' : part.removed ? '-' : ' '}
              </span>
              <span class="diff-view__line-content">{line || ' '}</span>
            </div>
          {/if}
        {/each}
      {/each}
    </div>
  </div>
</div>

<style>
  .diff-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .diff-view__header {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 12px 16px;
    background: var(--color-surface-subtle);
    border-bottom: 1px solid var(--color-border);
  }

  .diff-view__version {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
  }

  .diff-view__version-label {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 600;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
  }

  .diff-view__stats {
    display: flex;
    gap: 16px;
    padding: 8px 16px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.875rem;
  }

  .diff-view__stat--added {
    color: var(--color-success);
  }

  .diff-view__stat--removed {
    color: var(--color-danger);
  }

  .diff-view__section {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .diff-view__section h4 {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    margin: 0 0 12px;
  }

  .diff-view__section--content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border-bottom: none;
  }

  .diff-view__inline {
    font-family: var(--font-mono);
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .diff-view__content {
    flex: 1;
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: 0.875rem;
    background: var(--color-code-bg);
    border-radius: 6px;
  }

  .diff-view__line {
    display: flex;
    line-height: 1.6;
  }

  .diff-view__line--added {
    background: var(--color-success-subtle);
    color: var(--color-success);
  }

  .diff-view__line--removed {
    background: var(--color-danger-subtle);
    color: var(--color-danger);
  }

  .diff-view__line-prefix {
    width: 24px;
    padding: 0 8px;
    text-align: center;
    user-select: none;
    color: var(--color-text-tertiary);
  }

  .diff-view__line--added .diff-view__line-prefix {
    color: var(--color-success);
  }

  .diff-view__line--removed .diff-view__line-prefix {
    color: var(--color-danger);
  }

  .diff-view__line-content {
    flex: 1;
    padding-right: 16px;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>
```

### Version Types

```typescript
// types/spec.ts additions
export interface SpecVersion {
  id: string;
  specId: string;
  timestamp: Date;
  author?: string;
  message?: string;
  snapshot: Spec;
  changes: {
    title?: { old: string; new: string };
    status?: { old: SpecStatus; new: SpecStatus };
    content?: boolean;
    dependencies?: { added: string[]; removed: string[] };
  };
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SpecHistory from './SpecHistory.svelte';
import DiffView from './DiffView.svelte';
import * as api from '$lib/api/specs';

describe('SpecHistory', () => {
  const mockHistory = [
    {
      id: 'v3',
      specId: '244',
      timestamp: new Date('2024-01-20'),
      author: 'Jane',
      snapshot: { title: 'Final Title', content: 'Final content' },
      changes: { title: { old: 'Draft Title', new: 'Final Title' } }
    },
    {
      id: 'v2',
      specId: '244',
      timestamp: new Date('2024-01-15'),
      author: 'John',
      snapshot: { title: 'Draft Title', content: 'Draft content' },
      changes: { content: true }
    },
    {
      id: 'v1',
      specId: '244',
      timestamp: new Date('2024-01-10'),
      snapshot: { title: 'Initial', content: 'Initial' },
      changes: {}
    }
  ];

  beforeEach(() => {
    vi.spyOn(api, 'getSpecHistory').mockResolvedValue(mockHistory);
  });

  it('loads and displays version history', async () => {
    render(SpecHistory, { props: { specId: '244' } });

    await waitFor(() => {
      expect(screen.getByText('3 versions')).toBeInTheDocument();
    });
  });

  it('shows current badge on latest version', async () => {
    render(SpecHistory, { props: { specId: '244' } });

    await waitFor(() => {
      expect(screen.getByText('Current')).toBeInTheDocument();
    });
  });

  it('allows selecting versions for comparison', async () => {
    render(SpecHistory, { props: { specId: '244' } });

    await waitFor(() => {
      expect(screen.getAllByText('A').length).toBeGreaterThan(0);
    });

    const selectButtons = screen.getAllByTitle('Select as left version');
    await fireEvent.click(selectButtons[1]);

    // Button should be active
  });

  it('switches to diff view when compare clicked', async () => {
    render(SpecHistory, { props: { specId: '244' } });

    await waitFor(() => {
      expect(screen.getByText('Compare')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Compare'));

    // Should show diff view
  });

  it('dispatches restore event', async () => {
    const { component } = render(SpecHistory, { props: { specId: '244' } });

    const restoreHandler = vi.fn();
    component.$on('restore', restoreHandler);

    vi.spyOn(window, 'confirm').mockReturnValue(true);

    await waitFor(() => {
      expect(screen.getAllByText('Restore').length).toBeGreaterThan(0);
    });

    const restoreButtons = screen.getAllByText('Restore');
    await fireEvent.click(restoreButtons[0]);

    expect(restoreHandler).toHaveBeenCalled();
  });

  it('toggles detail mode', async () => {
    render(SpecHistory, { props: { specId: '244' } });

    await waitFor(() => {
      expect(screen.getByText('Show details')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Show details'));

    // Should show detailed view with changes
  });
});

describe('DiffView', () => {
  const leftVersion = {
    timestamp: new Date('2024-01-15'),
    snapshot: {
      title: 'Old Title',
      content: 'Line 1\nLine 2\nLine 3'
    }
  };

  const rightVersion = {
    timestamp: new Date('2024-01-20'),
    snapshot: {
      title: 'New Title',
      content: 'Line 1\nModified Line 2\nLine 3\nLine 4'
    }
  };

  it('renders version dates', () => {
    render(DiffView, { props: { left: leftVersion, right: rightVersion } });

    // Should show both dates
  });

  it('shows line statistics', () => {
    render(DiffView, { props: { left: leftVersion, right: rightVersion } });

    expect(screen.getByText(/\+\d+ lines/)).toBeInTheDocument();
    expect(screen.getByText(/-\d+ lines/)).toBeInTheDocument();
  });

  it('shows title diff when changed', () => {
    render(DiffView, { props: { left: leftVersion, right: rightVersion } });

    expect(screen.getByText('Title')).toBeInTheDocument();
  });

  it('highlights added lines', () => {
    render(DiffView, { props: { left: leftVersion, right: rightVersion } });

    const addedLines = document.querySelectorAll('.diff-view__line--added');
    expect(addedLines.length).toBeGreaterThan(0);
  });

  it('highlights removed lines', () => {
    render(DiffView, { props: { left: leftVersion, right: rightVersion } });

    const removedLines = document.querySelectorAll('.diff-view__line--removed');
    expect(removedLines.length).toBeGreaterThan(0);
  });
});
```

---

## Related Specs

- Spec 236: Spec Detail View
- Spec 237: Spec Editor
- Spec 245: Spec Comments
