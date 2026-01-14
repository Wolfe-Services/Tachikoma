# Spec 262: Draft Viewer

## Header
- **Spec ID**: 262
- **Phase**: 12 - Forge UI
- **Component**: Draft Viewer
- **Dependencies**: Spec 261 (Round Visualization)
- **Status**: Draft

## Objective
Create a specialized viewer for displaying participant draft responses during deliberation, with side-by-side comparison, diff highlighting, version tracking, and collaborative annotation support.

## Acceptance Criteria
- [x] Display individual drafts with full formatting support
- [x] Side-by-side comparison of multiple drafts
- [x] Diff highlighting between draft versions
- [x] Syntax highlighting for code blocks
- [x] Version history navigation per draft
- [x] Annotation and commenting on specific sections
- [x] Export drafts in multiple formats
- [x] Real-time updates as drafts are submitted

## Implementation

### DraftViewer.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import DraftContent from './DraftContent.svelte';
  import DraftComparison from './DraftComparison.svelte';
  import DraftVersions from './DraftVersions.svelte';
  import DraftAnnotations from './DraftAnnotations.svelte';
  import MarkdownRenderer from '$lib/components/MarkdownRenderer.svelte';
  import { diffLines, diffWords } from '$lib/utils/diff';
  import type { Draft, DraftVersion, Annotation, DiffResult } from '$lib/types/forge';

  export let drafts: Draft[] = [];
  export let roundNumber: number;
  export let allowAnnotations: boolean = true;

  const dispatch = createEventDispatcher<{
    annotate: { draftId: string; annotation: Partial<Annotation> };
    export: { draftId: string; format: string };
  }>();

  let selectedDraftId = writable<string | null>(null);
  let comparisonDraftId = writable<string | null>(null);
  let viewMode = writable<'single' | 'compare' | 'all'>('single');
  let showVersions = writable<boolean>(false);
  let showAnnotations = writable<boolean>(false);
  let diffMode = writable<'lines' | 'words'>('lines');
  let selectedVersion = writable<number | null>(null);

  const selectedDraft = derived(
    [() => drafts, selectedDraftId],
    ([drafts, $id]) => drafts.find(d => d.id === $id) || drafts[0] || null
  );

  const comparisonDraft = derived(
    [() => drafts, comparisonDraftId],
    ([drafts, $id]) => drafts.find(d => d.id === $id) || null
  );

  const diffResult = derived(
    [selectedDraft, comparisonDraft, diffMode],
    ([$selected, $comparison, $mode]) => {
      if (!$selected || !$comparison) return null;

      if ($mode === 'words') {
        return diffWords($selected.content, $comparison.content);
      }
      return diffLines($selected.content, $comparison.content);
    }
  );

  const currentVersionContent = derived(
    [selectedDraft, selectedVersion],
    ([$draft, $version]) => {
      if (!$draft) return null;
      if ($version === null) return $draft.content;
      return $draft.versions?.[$version]?.content || $draft.content;
    }
  );

  function selectDraft(draftId: string) {
    selectedDraftId.set(draftId);
    selectedVersion.set(null);
  }

  function setComparisonDraft(draftId: string | null) {
    comparisonDraftId.set(draftId);
    if (draftId) {
      viewMode.set('compare');
    }
  }

  function selectVersion(versionIndex: number) {
    selectedVersion.set(versionIndex);
  }

  function addAnnotation(selection: { start: number; end: number; text: string }) {
    if (!$selectedDraft || !allowAnnotations) return;

    const annotation: Partial<Annotation> = {
      draftId: $selectedDraft.id,
      selection,
      content: '',
      createdAt: new Date()
    };

    dispatch('annotate', { draftId: $selectedDraft.id, annotation });
  }

  function exportDraft(format: 'markdown' | 'html' | 'pdf' | 'json') {
    if (!$selectedDraft) return;
    dispatch('export', { draftId: $selectedDraft.id, format });
  }

  $: if (drafts.length > 0 && !$selectedDraftId) {
    selectedDraftId.set(drafts[0].id);
  }
</script>

<div class="draft-viewer" data-testid="draft-viewer">
  <header class="viewer-header">
    <h3>Round {roundNumber} Drafts</h3>
    <div class="header-actions">
      <div class="view-toggle">
        <button
          class:active={$viewMode === 'single'}
          on:click={() => viewMode.set('single')}
        >
          Single
        </button>
        <button
          class:active={$viewMode === 'compare'}
          on:click={() => viewMode.set('compare')}
          disabled={drafts.length < 2}
        >
          Compare
        </button>
        <button
          class:active={$viewMode === 'all'}
          on:click={() => viewMode.set('all')}
        >
          All
        </button>
      </div>

      <div class="action-buttons">
        <button
          class="icon-btn"
          class:active={$showVersions}
          on:click={() => showVersions.update(v => !v)}
          title="Show version history"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
        {#if allowAnnotations}
          <button
            class="icon-btn"
            class:active={$showAnnotations}
            on:click={() => showAnnotations.update(v => !v)}
            title="Show annotations"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        {/if}
        <div class="export-dropdown">
          <button class="icon-btn" title="Export draft">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
          <div class="dropdown-content">
            <button on:click={() => exportDraft('markdown')}>Markdown</button>
            <button on:click={() => exportDraft('html')}>HTML</button>
            <button on:click={() => exportDraft('pdf')}>PDF</button>
            <button on:click={() => exportDraft('json')}>JSON</button>
          </div>
        </div>
      </div>
    </div>
  </header>

  <div class="draft-tabs">
    {#each drafts as draft (draft.id)}
      <button
        class="draft-tab"
        class:active={$selectedDraftId === draft.id}
        class:comparison={$comparisonDraftId === draft.id}
        on:click={() => selectDraft(draft.id)}
        on:contextmenu|preventDefault={() => setComparisonDraft(draft.id)}
      >
        <span class="tab-participant">{draft.participantName}</span>
        {#if draft.status === 'in_progress'}
          <span class="tab-status writing">Writing...</span>
        {:else if draft.status === 'completed'}
          <span class="tab-status completed">Done</span>
        {/if}
        {#if draft.versions && draft.versions.length > 1}
          <span class="version-count">v{draft.versions.length}</span>
        {/if}
      </button>
    {/each}
  </div>

  {#if $showVersions && $selectedDraft?.versions}
    <div class="versions-panel" transition:slide>
      <DraftVersions
        versions={$selectedDraft.versions}
        selectedIndex={$selectedVersion}
        on:select={(e) => selectVersion(e.detail)}
      />
    </div>
  {/if}

  <div class="viewer-content" class:compare-mode={$viewMode === 'compare'}>
    {#if $viewMode === 'single' && $selectedDraft}
      <div class="single-view" transition:fade>
        <div class="draft-meta">
          <span class="participant-name">{$selectedDraft.participantName}</span>
          <span class="draft-timestamp">
            {new Date($selectedDraft.updatedAt).toLocaleTimeString()}
          </span>
          {#if $selectedDraft.wordCount}
            <span class="word-count">{$selectedDraft.wordCount} words</span>
          {/if}
        </div>

        <DraftContent
          content={$currentVersionContent || ''}
          annotations={$showAnnotations ? $selectedDraft.annotations : []}
          on:select={addAnnotation}
        />

        {#if $showAnnotations && $selectedDraft.annotations?.length}
          <DraftAnnotations
            annotations={$selectedDraft.annotations}
            on:reply={(e) => dispatch('annotate', e.detail)}
          />
        {/if}
      </div>
    {:else if $viewMode === 'compare'}
      <div class="compare-view">
        {#if $selectedDraft && $comparisonDraft}
          <div class="diff-controls">
            <span>Comparing:</span>
            <select bind:value={$diffMode}>
              <option value="lines">Line by line</option>
              <option value="words">Word by word</option>
            </select>
          </div>

          <DraftComparison
            leftDraft={$selectedDraft}
            rightDraft={$comparisonDraft}
            diff={$diffResult}
            mode={$diffMode}
          />
        {:else}
          <div class="compare-prompt">
            <p>Select a draft from the tabs above</p>
            <p class="hint">Right-click a tab to set it as comparison target</p>
          </div>
        {/if}
      </div>
    {:else if $viewMode === 'all'}
      <div class="all-view">
        {#each drafts as draft (draft.id)}
          <div class="draft-card">
            <div class="card-header">
              <span class="participant-name">{draft.participantName}</span>
              <span class="draft-status {draft.status}">{draft.status}</span>
            </div>
            <div class="card-content">
              <MarkdownRenderer content={draft.content.slice(0, 500)} />
              {#if draft.content.length > 500}
                <button
                  class="read-more"
                  on:click={() => {
                    selectDraft(draft.id);
                    viewMode.set('single');
                  }}
                >
                  Read more...
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .draft-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
    border-radius: 8px;
    overflow: hidden;
  }

  .viewer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .viewer-header h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .view-toggle {
    display: flex;
    background: var(--secondary-bg);
    border-radius: 4px;
    overflow: hidden;
  }

  .view-toggle button {
    padding: 0.375rem 0.75rem;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .view-toggle button.active {
    background: var(--primary-color);
    color: white;
  }

  .view-toggle button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .icon-btn {
    padding: 0.5rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .icon-btn:hover,
  .icon-btn.active {
    background: var(--secondary-bg);
    color: var(--text-primary);
  }

  .export-dropdown {
    position: relative;
  }

  .dropdown-content {
    display: none;
    position: absolute;
    right: 0;
    top: 100%;
    background: var(--dropdown-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    min-width: 120px;
    z-index: 10;
  }

  .export-dropdown:hover .dropdown-content {
    display: block;
  }

  .dropdown-content button {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .dropdown-content button:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .draft-tabs {
    display: flex;
    gap: 0.25rem;
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    overflow-x: auto;
  }

  .draft-tab {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .draft-tab:hover {
    background: var(--hover-bg);
  }

  .draft-tab.active {
    background: var(--card-bg);
    border-color: var(--primary-color);
    color: var(--text-primary);
  }

  .draft-tab.comparison {
    border-color: var(--warning-color);
  }

  .tab-status {
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
  }

  .tab-status.writing {
    background: var(--warning-bg);
    color: var(--warning-color);
  }

  .tab-status.completed {
    background: var(--success-bg);
    color: var(--success-color);
  }

  .version-count {
    font-size: 0.625rem;
    color: var(--text-muted);
  }

  .versions-panel {
    padding: 0.75rem 1rem;
    background: var(--card-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .viewer-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.25rem;
  }

  .single-view {
    max-width: 800px;
    margin: 0 auto;
  }

  .draft-meta {
    display: flex;
    gap: 1rem;
    align-items: center;
    margin-bottom: 1rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--border-color);
  }

  .participant-name {
    font-weight: 600;
  }

  .draft-timestamp {
    color: var(--text-muted);
    font-size: 0.8125rem;
  }

  .word-count {
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .compare-view {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .diff-controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    margin-bottom: 1rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .diff-controls select {
    padding: 0.375rem 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.8125rem;
  }

  .compare-prompt {
    text-align: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .all-view {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }

  .draft-card {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    overflow: hidden;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .draft-status {
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    border-radius: 3px;
  }

  .draft-status.completed {
    background: var(--success-bg);
    color: var(--success-color);
  }

  .draft-status.in_progress {
    background: var(--warning-bg);
    color: var(--warning-color);
  }

  .card-content {
    padding: 1rem;
    font-size: 0.875rem;
    line-height: 1.6;
  }

  .read-more {
    display: inline-block;
    margin-top: 0.75rem;
    padding: 0;
    background: none;
    border: none;
    color: var(--primary-color);
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .read-more:hover {
    text-decoration: underline;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test draft selection, version navigation, and comparison logic
2. **Integration Tests**: Verify diff highlighting accuracy
3. **Export Tests**: Validate export functionality for all formats
4. **Annotation Tests**: Test annotation creation and display
5. **Performance Tests**: Measure render performance with large drafts

## Related Specs
- Spec 261: Round Visualization
- Spec 263: Critique Viewer
- Spec 264: Conflict Highlights
