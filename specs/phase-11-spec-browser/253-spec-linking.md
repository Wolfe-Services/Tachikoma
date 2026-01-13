# 253 - Spec Linking

**Phase:** 11 - Spec Browser UI
**Spec ID:** 253
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 247-spec-metadata
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a spec linking system that enables bi-directional links between specs, visualizes dependency graphs, and provides link management with auto-detection of references.

---

## Acceptance Criteria

- [x] Create links between specs
- [x] Visualize dependency graph
- [x] Auto-detect spec references in content
- [x] Broken link detection
- [x] Link preview on hover
- [x] Bi-directional link tracking
- [x] Link type categorization
- [x] Bulk link operations

---

## Implementation Details

### 1. Types (src/lib/types/spec-linking.ts)

```typescript
export type LinkType =
  | 'depends_on'
  | 'blocks'
  | 'related'
  | 'implements'
  | 'extends'
  | 'references';

export interface SpecLink {
  id: string;
  sourceSpecId: string;
  targetSpecId: string;
  type: LinkType;
  isAutoDetected: boolean;
  context?: string;
  createdAt: string;
  createdBy: string;
}

export interface LinkGraph {
  nodes: LinkNode[];
  edges: LinkEdge[];
}

export interface LinkNode {
  id: string;
  specId: string;
  title: string;
  phase: number;
  status: string;
  x?: number;
  y?: number;
}

export interface LinkEdge {
  id: string;
  source: string;
  target: string;
  type: LinkType;
  isAutoDetected: boolean;
}

export interface LinkSuggestion {
  targetSpecId: string;
  targetTitle: string;
  type: LinkType;
  confidence: number;
  reason: string;
  context: string;
}

export interface BrokenLink {
  specId: string;
  specTitle: string;
  linkText: string;
  targetReference: string;
  lineNumber: number;
}
```

### 2. Spec Linking Panel Component (src/lib/components/spec-browser/SpecLinkingPanel.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type {
    SpecLink,
    LinkType,
    LinkGraph,
    LinkSuggestion,
    BrokenLink,
  } from '$lib/types/spec-linking';
  import { ipcRenderer } from '$lib/ipc';
  import { fade, slide } from 'svelte/transition';

  export let specId: string;

  const dispatch = createEventDispatcher<{
    navigate: string;
    linkCreated: SpecLink;
    linkRemoved: string;
  }>();

  let incomingLinks: SpecLink[] = [];
  let outgoingLinks: SpecLink[] = [];
  let suggestions: LinkSuggestion[] = [];
  let brokenLinks: BrokenLink[] = [];
  let isLoading = true;
  let showAddLink = false;
  let showGraph = false;
  let linkGraph: LinkGraph | null = null;
  let activeTab: 'outgoing' | 'incoming' | 'suggestions' | 'broken' = 'outgoing';

  // New link form
  let newLinkTargetId = '';
  let newLinkType: LinkType = 'depends_on';
  let searchQuery = '';
  let searchResults: { id: string; title: string }[] = [];

  const linkTypeLabels: Record<LinkType, string> = {
    depends_on: 'Depends On',
    blocks: 'Blocks',
    related: 'Related To',
    implements: 'Implements',
    extends: 'Extends',
    references: 'References',
  };

  const linkTypeIcons: Record<LinkType, string> = {
    depends_on: '→',
    blocks: '⛔',
    related: '↔',
    implements: '⚙️',
    extends: '📦',
    references: '📎',
  };

  async function loadLinks() {
    isLoading = true;
    try {
      const [incoming, outgoing, suggested, broken] = await Promise.all([
        ipcRenderer.invoke('spec:get-incoming-links', specId),
        ipcRenderer.invoke('spec:get-outgoing-links', specId),
        ipcRenderer.invoke('spec:get-link-suggestions', specId),
        ipcRenderer.invoke('spec:get-broken-links', specId),
      ]);
      incomingLinks = incoming;
      outgoingLinks = outgoing;
      suggestions = suggested;
      brokenLinks = broken;
    } finally {
      isLoading = false;
    }
  }

  async function loadGraph() {
    linkGraph = await ipcRenderer.invoke('spec:get-link-graph', specId);
    showGraph = true;
  }

  async function searchSpecs() {
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }
    searchResults = await ipcRenderer.invoke('spec:search-for-linking', {
      query: searchQuery,
      excludeSpecId: specId,
    });
  }

  async function createLink() {
    if (!newLinkTargetId) return;

    const link = await ipcRenderer.invoke('spec:create-link', {
      sourceSpecId: specId,
      targetSpecId: newLinkTargetId,
      type: newLinkType,
    });

    dispatch('linkCreated', link);
    resetLinkForm();
    await loadLinks();
  }

  async function removeLink(linkId: string) {
    const confirmed = await ipcRenderer.invoke('dialog:confirm', {
      title: 'Remove Link',
      message: 'Are you sure you want to remove this link?',
      confirmText: 'Remove',
      cancelText: 'Cancel',
    });

    if (confirmed) {
      await ipcRenderer.invoke('spec:remove-link', linkId);
      dispatch('linkRemoved', linkId);
      await loadLinks();
    }
  }

  async function acceptSuggestion(suggestion: LinkSuggestion) {
    const link = await ipcRenderer.invoke('spec:create-link', {
      sourceSpecId: specId,
      targetSpecId: suggestion.targetSpecId,
      type: suggestion.type,
    });

    dispatch('linkCreated', link);
    await loadLinks();
  }

  async function dismissSuggestion(suggestion: LinkSuggestion) {
    await ipcRenderer.invoke('spec:dismiss-link-suggestion', {
      specId,
      targetSpecId: suggestion.targetSpecId,
    });
    suggestions = suggestions.filter(s => s.targetSpecId !== suggestion.targetSpecId);
  }

  function resetLinkForm() {
    showAddLink = false;
    newLinkTargetId = '';
    newLinkType = 'depends_on';
    searchQuery = '';
    searchResults = [];
  }

  function selectSearchResult(result: { id: string; title: string }) {
    newLinkTargetId = result.id;
    searchQuery = `${result.id} - ${result.title}`;
    searchResults = [];
  }

  onMount(loadLinks);

  $: if (specId) loadLinks();
</script>

<div class="linking-panel">
  <header class="linking-panel__header">
    <h3>Spec Links</h3>
    <div class="header-actions">
      <button class="graph-btn" on:click={loadGraph} title="View dependency graph">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="4" cy="8" r="2"/>
          <circle cx="12" cy="4" r="2"/>
          <circle cx="12" cy="12" r="2"/>
          <path d="M6 8l4-3M6 8l4 3" stroke="currentColor" stroke-width="1.5" fill="none"/>
        </svg>
      </button>
      <button class="add-btn" on:click={() => { showAddLink = !showAddLink; }}>
        {showAddLink ? 'Cancel' : '+ Add Link'}
      </button>
    </div>
  </header>

  {#if showAddLink}
    <div class="add-link-form" transition:slide={{ duration: 150 }}>
      <div class="form-group">
        <label>Link Type</label>
        <select bind:value={newLinkType}>
          {#each Object.entries(linkTypeLabels) as [type, label]}
            <option value={type}>{linkTypeIcons[type]} {label}</option>
          {/each}
        </select>
      </div>

      <div class="form-group">
        <label>Target Spec</label>
        <div class="search-input-container">
          <input
            type="text"
            placeholder="Search specs..."
            bind:value={searchQuery}
            on:input={searchSpecs}
          />
          {#if searchResults.length > 0}
            <ul class="search-results">
              {#each searchResults as result}
                <li>
                  <button on:click={() => selectSearchResult(result)}>
                    <span class="result-id">{result.id}</span>
                    <span class="result-title">{result.title}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      </div>

      <div class="form-actions">
        <button class="cancel-btn" on:click={resetLinkForm}>Cancel</button>
        <button
          class="create-btn"
          on:click={createLink}
          disabled={!newLinkTargetId}
        >
          Create Link
        </button>
      </div>
    </div>
  {/if}

  <nav class="tab-nav">
    <button
      class:active={activeTab === 'outgoing'}
      on:click={() => { activeTab = 'outgoing'; }}
    >
      Outgoing ({outgoingLinks.length})
    </button>
    <button
      class:active={activeTab === 'incoming'}
      on:click={() => { activeTab = 'incoming'; }}
    >
      Incoming ({incomingLinks.length})
    </button>
    <button
      class:active={activeTab === 'suggestions'}
      on:click={() => { activeTab = 'suggestions'; }}
    >
      Suggestions ({suggestions.length})
    </button>
    {#if brokenLinks.length > 0}
      <button
        class="broken-tab"
        class:active={activeTab === 'broken'}
        on:click={() => { activeTab = 'broken'; }}
      >
        Broken ({brokenLinks.length})
      </button>
    {/if}
  </nav>

  <div class="link-content">
    {#if isLoading}
      <div class="loading">Loading links...</div>
    {:else if activeTab === 'outgoing'}
      {#if outgoingLinks.length === 0}
        <div class="empty-state">No outgoing links</div>
      {:else}
        <ul class="link-list">
          {#each outgoingLinks as link (link.id)}
            <li class="link-item" transition:slide={{ duration: 150 }}>
              <button
                class="link-target"
                on:click={() => dispatch('navigate', link.targetSpecId)}
              >
                <span class="link-type-icon">{linkTypeIcons[link.type]}</span>
                <span class="link-spec-id">{link.targetSpecId}</span>
                <span class="link-type-label">{linkTypeLabels[link.type]}</span>
              </button>
              {#if link.isAutoDetected}
                <span class="auto-badge" title="Auto-detected">Auto</span>
              {/if}
              <button
                class="remove-btn"
                on:click={() => removeLink(link.id)}
                title="Remove link"
              >
                ×
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {:else if activeTab === 'incoming'}
      {#if incomingLinks.length === 0}
        <div class="empty-state">No incoming links</div>
      {:else}
        <ul class="link-list">
          {#each incomingLinks as link (link.id)}
            <li class="link-item incoming" transition:slide={{ duration: 150 }}>
              <button
                class="link-target"
                on:click={() => dispatch('navigate', link.sourceSpecId)}
              >
                <span class="link-type-icon">←</span>
                <span class="link-spec-id">{link.sourceSpecId}</span>
                <span class="link-type-label">{linkTypeLabels[link.type]}</span>
              </button>
              {#if link.isAutoDetected}
                <span class="auto-badge" title="Auto-detected">Auto</span>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    {:else if activeTab === 'suggestions'}
      {#if suggestions.length === 0}
        <div class="empty-state">No suggestions</div>
      {:else}
        <ul class="suggestion-list">
          {#each suggestions as suggestion (suggestion.targetSpecId)}
            <li class="suggestion-item" transition:slide={{ duration: 150 }}>
              <div class="suggestion-header">
                <span class="suggestion-icon">{linkTypeIcons[suggestion.type]}</span>
                <span class="suggestion-target">{suggestion.targetSpecId}</span>
                <span class="suggestion-title">{suggestion.targetTitle}</span>
              </div>
              <div class="suggestion-reason">
                {suggestion.reason}
              </div>
              <div class="suggestion-confidence">
                <div
                  class="confidence-bar"
                  style="width: {suggestion.confidence * 100}%"
                />
                <span>{Math.round(suggestion.confidence * 100)}% match</span>
              </div>
              <div class="suggestion-actions">
                <button
                  class="dismiss-btn"
                  on:click={() => dismissSuggestion(suggestion)}
                >
                  Dismiss
                </button>
                <button
                  class="accept-btn"
                  on:click={() => acceptSuggestion(suggestion)}
                >
                  Accept
                </button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    {:else if activeTab === 'broken'}
      <ul class="broken-list">
        {#each brokenLinks as broken (broken.targetReference)}
          <li class="broken-item" transition:slide={{ duration: 150 }}>
            <div class="broken-header">
              <span class="broken-icon">⚠️</span>
              <span class="broken-text">{broken.linkText}</span>
            </div>
            <div class="broken-details">
              <span class="broken-target">Target: {broken.targetReference}</span>
              <span class="broken-line">Line {broken.lineNumber}</span>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

{#if showGraph && linkGraph}
  <div
    class="graph-overlay"
    on:click={() => { showGraph = false; }}
    transition:fade={{ duration: 150 }}
  >
    <div class="graph-container" on:click|stopPropagation>
      <header class="graph-header">
        <h3>Dependency Graph</h3>
        <button class="close-btn" on:click={() => { showGraph = false; }}>×</button>
      </header>
      <div class="graph-canvas">
        <svg width="100%" height="100%">
          <!-- Edges -->
          {#each linkGraph.edges as edge}
            {@const source = linkGraph.nodes.find(n => n.id === edge.source)}
            {@const target = linkGraph.nodes.find(n => n.id === edge.target)}
            {#if source && target && source.x && source.y && target.x && target.y}
              <line
                x1={source.x}
                y1={source.y}
                x2={target.x}
                y2={target.y}
                class="graph-edge"
                class:auto-detected={edge.isAutoDetected}
              />
            {/if}
          {/each}

          <!-- Nodes -->
          {#each linkGraph.nodes as node}
            {#if node.x && node.y}
              <g
                class="graph-node"
                class:current={node.specId === specId}
                transform="translate({node.x}, {node.y})"
              >
                <circle r="20" />
                <text dy="4" text-anchor="middle">{node.specId}</text>
              </g>
            {/if}
          {/each}
        </svg>
      </div>
      <div class="graph-legend">
        <span class="legend-item">
          <span class="legend-dot current" /> Current Spec
        </span>
        <span class="legend-item">
          <span class="legend-dot" /> Related Spec
        </span>
        <span class="legend-item">
          <span class="legend-line" /> Manual Link
        </span>
        <span class="legend-item">
          <span class="legend-line auto" /> Auto-detected
        </span>
      </div>
    </div>
  </div>
{/if}

<style>
  .linking-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-primary);
  }

  .linking-panel__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .linking-panel__header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .graph-btn {
    padding: 6px 10px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-muted);
  }

  .graph-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .add-btn {
    padding: 6px 12px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .add-link-form {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .form-group {
    margin-bottom: 12px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .form-group select,
  .form-group input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 13px;
  }

  .search-input-container {
    position: relative;
  }

  .search-results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    max-height: 200px;
    overflow-y: auto;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    list-style: none;
    padding: 0;
    margin: 4px 0 0 0;
    z-index: 10;
  }

  .search-results button {
    display: flex;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
  }

  .search-results button:hover {
    background: var(--color-bg-hover);
  }

  .result-id {
    font-family: monospace;
    font-size: 12px;
    color: var(--color-primary);
  }

  .result-title {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .cancel-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .create-btn {
    padding: 6px 12px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .create-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab-nav {
    display: flex;
    border-bottom: 1px solid var(--color-border);
  }

  .tab-nav button {
    flex: 1;
    padding: 10px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 12px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }

  .tab-nav button:hover {
    background: var(--color-bg-hover);
  }

  .tab-nav button.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }

  .tab-nav button.broken-tab {
    color: var(--color-warning);
  }

  .link-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .loading,
  .empty-state {
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .link-list,
  .suggestion-list,
  .broken-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .link-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    border-radius: 6px;
    margin-bottom: 4px;
  }

  .link-item:hover {
    background: var(--color-bg-hover);
  }

  .link-target {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    padding: 4px;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
  }

  .link-type-icon {
    font-size: 14px;
  }

  .link-spec-id {
    font-family: monospace;
    font-size: 13px;
    color: var(--color-primary);
  }

  .link-type-label {
    font-size: 11px;
    color: var(--color-text-muted);
    margin-left: auto;
  }

  .auto-badge {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    color: var(--color-text-muted);
  }

  .remove-btn {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 16px;
    opacity: 0;
  }

  .link-item:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    color: var(--color-error);
  }

  .suggestion-item {
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .suggestion-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .suggestion-icon {
    font-size: 14px;
  }

  .suggestion-target {
    font-family: monospace;
    font-size: 13px;
    color: var(--color-primary);
  }

  .suggestion-title {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .suggestion-reason {
    font-size: 12px;
    color: var(--color-text-muted);
    margin-bottom: 8px;
  }

  .suggestion-confidence {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .confidence-bar {
    height: 4px;
    background: var(--color-success);
    border-radius: 2px;
    max-width: 100px;
  }

  .suggestion-confidence span {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .suggestion-actions {
    display: flex;
    gap: 8px;
  }

  .dismiss-btn {
    padding: 4px 10px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .accept-btn {
    padding: 4px 10px;
    border: none;
    background: var(--color-success);
    color: white;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .broken-item {
    padding: 12px;
    background: rgba(255, 193, 7, 0.1);
    border: 1px solid var(--color-warning);
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .broken-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .broken-text {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .broken-details {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  /* Graph Overlay */
  .graph-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .graph-container {
    width: 80%;
    max-width: 900px;
    height: 70vh;
    background: var(--color-bg-primary);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .graph-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .graph-header h3 {
    margin: 0;
  }

  .close-btn {
    padding: 6px 12px;
    border: none;
    background: transparent;
    font-size: 20px;
    cursor: pointer;
    color: var(--color-text-muted);
  }

  .graph-canvas {
    flex: 1;
    overflow: hidden;
  }

  .graph-edge {
    stroke: var(--color-border);
    stroke-width: 2;
  }

  .graph-edge.auto-detected {
    stroke-dasharray: 4 4;
  }

  .graph-node circle {
    fill: var(--color-bg-secondary);
    stroke: var(--color-border);
    stroke-width: 2;
  }

  .graph-node.current circle {
    fill: var(--color-primary);
    stroke: var(--color-primary);
  }

  .graph-node text {
    font-size: 10px;
    fill: var(--color-text-primary);
  }

  .graph-node.current text {
    fill: white;
  }

  .graph-legend {
    display: flex;
    gap: 16px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .legend-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-bg-secondary);
    border: 2px solid var(--color-border);
  }

  .legend-dot.current {
    background: var(--color-primary);
    border-color: var(--color-primary);
  }

  .legend-line {
    width: 20px;
    height: 2px;
    background: var(--color-border);
  }

  .legend-line.auto {
    background: repeating-linear-gradient(
      90deg,
      var(--color-border) 0,
      var(--color-border) 4px,
      transparent 4px,
      transparent 8px
    );
  }
</style>
```

---

## Testing Requirements

1. Links load correctly
2. Link creation works
3. Link removal works
4. Suggestions display
5. Auto-detection works
6. Graph visualization renders
7. Broken links detected

---

## Related Specs

- Depends on: [247-spec-metadata.md](247-spec-metadata.md)
- Next: [254-spec-export.md](254-spec-export.md)
