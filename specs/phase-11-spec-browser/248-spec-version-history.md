# 248 - Spec Version History

**Phase:** 11 - Spec Browser UI
**Spec ID:** 248
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 247-spec-metadata
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a version history component that displays spec revision history, allows viewing previous versions, comparing versions, and restoring from history.

---

## Acceptance Criteria

- [x] Display version history timeline
- [x] Show change summary per version
- [x] View previous versions
- [x] Compare two versions
- [x] Restore previous version
- [x] Author and timestamp display
- [x] Diff highlighting between versions

---

## Implementation Details

### 1. Types (src/lib/types/spec-history.ts)

```typescript
export interface SpecVersion {
  id: string;
  specId: string;
  version: number;
  content: string;
  frontmatter: Record<string, unknown>;
  author: string;
  timestamp: string;
  message: string;
  changes: VersionChanges;
}

export interface VersionChanges {
  additions: number;
  deletions: number;
  sections: string[];
}

export interface VersionComparison {
  base: SpecVersion;
  compare: SpecVersion;
  hunks: DiffHunk[];
}

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  type: 'context' | 'addition' | 'deletion';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}
```

### 2. Spec Version History Component (src/lib/components/spec-browser/SpecVersionHistory.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { SpecVersion, VersionComparison } from '$lib/types/spec-history';
  import { ipcRenderer } from '$lib/ipc';
  import { fade, slide } from 'svelte/transition';

  export let specId: string;

  const dispatch = createEventDispatcher<{
    viewVersion: SpecVersion;
    restore: SpecVersion;
    compare: { base: string; compare: string };
  }>();

  let versions: SpecVersion[] = [];
  let selectedVersions: Set<string> = new Set();
  let isLoading = true;
  let expandedVersion: string | null = null;
  let comparison: VersionComparison | null = null;
  let showCompare = false;

  async function loadVersions() {
    isLoading = true;
    try {
      versions = await ipcRenderer.invoke('spec:get-versions', specId);
    } finally {
      isLoading = false;
    }
  }

  function toggleVersionSelect(versionId: string) {
    if (selectedVersions.has(versionId)) {
      selectedVersions.delete(versionId);
    } else if (selectedVersions.size < 2) {
      selectedVersions.add(versionId);
    }
    selectedVersions = new Set(selectedVersions);
  }

  async function compareVersions() {
    if (selectedVersions.size !== 2) return;

    const [baseId, compareId] = Array.from(selectedVersions);
    comparison = await ipcRenderer.invoke('spec:compare-versions', {
      specId,
      baseVersion: baseId,
      compareVersion: compareId,
    });
    showCompare = true;
  }

  async function restoreVersion(version: SpecVersion) {
    const confirmed = await ipcRenderer.invoke('dialog:confirm', {
      title: 'Restore Version',
      message: `Restore spec to version ${version.version}? This will create a new version with the previous content.`,
      confirmText: 'Restore',
      cancelText: 'Cancel',
    });

    if (confirmed) {
      await ipcRenderer.invoke('spec:restore-version', {
        specId,
        versionId: version.id,
      });
      dispatch('restore', version);
      await loadVersions();
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return date.toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
      });
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return `${diffDays} days ago`;
    } else {
      return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
      });
    }
  }

  function getChangeIcon(changes: VersionChanges): string {
    if (changes.additions > changes.deletions) return '+';
    if (changes.deletions > changes.additions) return '-';
    return '~';
  }

  onMount(loadVersions);

  $: if (specId) loadVersions();
</script>

<div class="version-history">
  <header class="version-history__header">
    <h3>Version History</h3>
    {#if selectedVersions.size === 2}
      <button class="compare-btn" on:click={compareVersions}>
        Compare Selected
      </button>
    {/if}
  </header>

  {#if isLoading}
    <div class="version-history__loading">Loading history...</div>
  {:else if versions.length === 0}
    <div class="version-history__empty">No version history available</div>
  {:else}
    <div class="version-timeline">
      {#each versions as version, index}
        <div
          class="version-item"
          class:version-item--current={index === 0}
          class:version-item--selected={selectedVersions.has(version.id)}
          transition:slide={{ duration: 200 }}
        >
          <div class="version-item__marker">
            <div class="marker-dot" />
            {#if index < versions.length - 1}
              <div class="marker-line" />
            {/if}
          </div>

          <div class="version-item__content">
            <div class="version-item__header">
              <button
                class="version-select"
                on:click={() => toggleVersionSelect(version.id)}
                title="Select for comparison"
              >
                {#if selectedVersions.has(version.id)}
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M8 0a8 8 0 100 16A8 8 0 008 0zm3.78 5.28l-4.5 6a.75.75 0 01-1.14.06l-2.25-2.25a.75.75 0 111.06-1.06l1.64 1.64 3.97-5.3a.75.75 0 011.22.91z"/>
                  </svg>
                {:else}
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                    <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1" fill="none"/>
                  </svg>
                {/if}
              </button>

              <span class="version-number">
                v{version.version}
                {#if index === 0}
                  <span class="current-badge">Current</span>
                {/if}
              </span>

              <span class="version-time">{formatDate(version.timestamp)}</span>
            </div>

            <div class="version-item__message">
              {version.message || 'No message'}
            </div>

            <div class="version-item__meta">
              <span class="version-author">{version.author}</span>
              <span class="version-changes">
                <span class="change-icon change-icon--{getChangeIcon(version.changes)}">
                  {getChangeIcon(version.changes)}
                </span>
                <span class="additions">+{version.changes.additions}</span>
                <span class="deletions">-{version.changes.deletions}</span>
              </span>
            </div>

            {#if version.changes.sections.length > 0}
              <button
                class="version-item__expand"
                on:click={() => {
                  expandedVersion = expandedVersion === version.id ? null : version.id;
                }}
              >
                {expandedVersion === version.id ? 'Hide' : 'Show'} changed sections
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 12 12"
                  fill="currentColor"
                  class:rotated={expandedVersion === version.id}
                >
                  <path d="M3 4.5l3 3 3-3" stroke="currentColor" stroke-width="1.5" fill="none"/>
                </svg>
              </button>

              {#if expandedVersion === version.id}
                <ul class="changed-sections" transition:slide={{ duration: 150 }}>
                  {#each version.changes.sections as section}
                    <li>{section}</li>
                  {/each}
                </ul>
              {/if}
            {/if}

            <div class="version-item__actions">
              <button
                class="action-btn"
                on:click={() => dispatch('viewVersion', version)}
              >
                View
              </button>
              {#if index > 0}
                <button
                  class="action-btn action-btn--restore"
                  on:click={() => restoreVersion(version)}
                >
                  Restore
                </button>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showCompare && comparison}
  <div
    class="compare-overlay"
    on:click={() => { showCompare = false; }}
    transition:fade={{ duration: 150 }}
  >
    <div class="compare-panel" on:click|stopPropagation>
      <header class="compare-panel__header">
        <h3>Version Comparison</h3>
        <span class="compare-versions">
          v{comparison.base.version} → v{comparison.compare.version}
        </span>
        <button class="close-btn" on:click={() => { showCompare = false; }}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4.293 4.293a1 1 0 011.414 0L8 6.586l2.293-2.293a1 1 0 111.414 1.414L9.414 8l2.293 2.293a1 1 0 01-1.414 1.414L8 9.414l-2.293 2.293a1 1 0 01-1.414-1.414L6.586 8 4.293 5.707a1 1 0 010-1.414z"/>
          </svg>
        </button>
      </header>

      <div class="compare-panel__content">
        {#each comparison.hunks as hunk}
          <div class="diff-hunk">
            <div class="hunk-header">
              @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
            </div>
            {#each hunk.lines as line}
              <div class="diff-line diff-line--{line.type}">
                <span class="line-number old">{line.oldLineNumber || ''}</span>
                <span class="line-number new">{line.newLineNumber || ''}</span>
                <span class="line-prefix">
                  {line.type === 'addition' ? '+' : line.type === 'deletion' ? '-' : ' '}
                </span>
                <span class="line-content">{line.content}</span>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .version-history {
    height: 100%;
    overflow-y: auto;
    padding: 16px;
  }

  .version-history__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .version-history__header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .compare-btn {
    padding: 6px 12px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .version-history__loading,
  .version-history__empty {
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .version-timeline {
    position: relative;
  }

  .version-item {
    display: flex;
    gap: 12px;
    padding-bottom: 16px;
  }

  .version-item__marker {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 20px;
  }

  .marker-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--color-border);
    border: 2px solid var(--color-bg-primary);
    z-index: 1;
  }

  .version-item--current .marker-dot {
    background: var(--color-primary);
  }

  .version-item--selected .marker-dot {
    background: var(--color-success);
  }

  .marker-line {
    flex: 1;
    width: 2px;
    background: var(--color-border);
    margin-top: 4px;
  }

  .version-item__content {
    flex: 1;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    padding: 12px;
  }

  .version-item--selected .version-item__content {
    outline: 2px solid var(--color-primary);
  }

  .version-item__header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .version-select {
    padding: 2px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .version-select:hover {
    color: var(--color-primary);
  }

  .version-number {
    font-family: monospace;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .current-badge {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    text-transform: uppercase;
    font-weight: 500;
  }

  .version-time {
    margin-left: auto;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .version-item__message {
    font-size: 13px;
    color: var(--color-text-primary);
    margin-bottom: 8px;
  }

  .version-item__meta {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
    color: var(--color-text-muted);
    margin-bottom: 8px;
  }

  .version-changes {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .change-icon {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    font-size: 11px;
    font-weight: bold;
  }

  .change-icon--\+ {
    background: rgba(76, 175, 80, 0.1);
    color: var(--color-success);
  }

  .change-icon---  {
    background: rgba(244, 67, 54, 0.1);
    color: var(--color-error);
  }

  .change-icon--~ {
    background: rgba(255, 193, 7, 0.1);
    color: var(--color-warning);
  }

  .additions {
    color: var(--color-success);
  }

  .deletions {
    color: var(--color-error);
  }

  .version-item__expand {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 0;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 12px;
    cursor: pointer;
  }

  .version-item__expand:hover {
    color: var(--color-primary);
  }

  .version-item__expand svg {
    transition: transform 0.15s ease;
  }

  .version-item__expand svg.rotated {
    transform: rotate(180deg);
  }

  .changed-sections {
    margin: 8px 0;
    padding-left: 20px;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .changed-sections li {
    padding: 2px 0;
  }

  .version-item__actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .action-btn {
    padding: 4px 10px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    color: var(--color-text-secondary);
  }

  .action-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .action-btn--restore:hover {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  /* Compare Overlay */
  .compare-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .compare-panel {
    width: 90%;
    max-width: 800px;
    max-height: 80vh;
    background: var(--color-bg-primary);
    border-radius: 12px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .compare-panel__header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .compare-panel__header h3 {
    margin: 0;
    font-size: 16px;
  }

  .compare-versions {
    font-family: monospace;
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .close-btn {
    margin-left: auto;
    padding: 6px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: 4px;
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .compare-panel__content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .diff-hunk {
    margin-bottom: 16px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    overflow: hidden;
  }

  .hunk-header {
    padding: 8px 12px;
    background: var(--color-bg-secondary);
    font-family: monospace;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .diff-line {
    display: flex;
    font-family: monospace;
    font-size: 12px;
    line-height: 20px;
  }

  .diff-line--addition {
    background: rgba(76, 175, 80, 0.1);
  }

  .diff-line--deletion {
    background: rgba(244, 67, 54, 0.1);
  }

  .line-number {
    width: 40px;
    padding: 0 8px;
    text-align: right;
    color: var(--color-text-muted);
    user-select: none;
    border-right: 1px solid var(--color-border);
  }

  .line-prefix {
    width: 20px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .diff-line--addition .line-prefix {
    color: var(--color-success);
  }

  .diff-line--deletion .line-prefix {
    color: var(--color-error);
  }

  .line-content {
    flex: 1;
    padding: 0 8px;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>
```

---

## Testing Requirements

1. Version history loads correctly
2. Version selection works
3. Comparison shows diff
4. Restore creates new version
5. Timeline renders properly

---

## Related Specs

- Depends on: [247-spec-metadata.md](247-spec-metadata.md)
- Next: [249-spec-diff-viewer.md](249-spec-diff-viewer.md)
