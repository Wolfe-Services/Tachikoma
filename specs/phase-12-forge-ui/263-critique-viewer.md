# Spec 263: Critique Viewer

## Header
- **Spec ID**: 263
- **Phase**: 12 - Forge UI
- **Component**: Critique Viewer
- **Dependencies**: Spec 262 (Draft Viewer)
- **Status**: Draft

## Objective
Create a specialized viewer for displaying and analyzing critiques provided by AI participants during deliberation rounds, with sentiment analysis, categorization, and response tracking.

## Acceptance Criteria
- [x] Display critiques with attribution to source participant
- [x] Show critique target (which draft/response is being critiqued)
- [x] Categorize critiques by type (technical, logical, stylistic, etc.)
- [x] Visualize sentiment and severity of critiques
- [x] Track critique responses and rebuttals
- [x] Enable filtering by category, severity, and participant
- [x] Link critiques to specific sections of drafts
- [x] Aggregate critique patterns across rounds

## Implementation

### CritiqueViewer.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import CritiqueCard from './CritiqueCard.svelte';
  import CritiqueSummary from './CritiqueSummary.svelte';
  import CritiqueFilters from './CritiqueFilters.svelte';
  import CritiqueThread from './CritiqueThread.svelte';
  import SentimentBadge from './SentimentBadge.svelte';
  import type {
    Critique,
    CritiqueCategory,
    CritiqueSeverity,
    CritiqueFilters as Filters
  } from '$lib/types/forge';

  export let critiques: Critique[] = [];
  export let roundNumber: number;
  export let participants: { id: string; name: string }[] = [];

  const dispatch = createEventDispatcher<{
    respond: { critiqueId: string; response: string };
    highlight: { critiqueId: string; targetId: string };
  }>();

  let selectedCritiqueId = writable<string | null>(null);
  let filters = writable<Filters>({
    categories: [],
    severities: [],
    authors: [],
    targets: [],
    hasResponse: null,
    searchQuery: ''
  });
  let sortBy = writable<'time' | 'severity' | 'category'>('time');
  let viewMode = writable<'list' | 'grouped' | 'threaded'>('list');

  const filteredCritiques = derived(
    [() => critiques, filters],
    ([critiqueList, $filters]) => {
      return critiqueList.filter(critique => {
        // Category filter
        if ($filters.categories.length > 0 &&
            !$filters.categories.includes(critique.category)) {
          return false;
        }

        // Severity filter
        if ($filters.severities.length > 0 &&
            !$filters.severities.includes(critique.severity)) {
          return false;
        }

        // Author filter
        if ($filters.authors.length > 0 &&
            !$filters.authors.includes(critique.authorId)) {
          return false;
        }

        // Target filter
        if ($filters.targets.length > 0 &&
            !$filters.targets.includes(critique.targetId)) {
          return false;
        }

        // Response filter
        if ($filters.hasResponse !== null) {
          const hasResponse = critique.responses && critique.responses.length > 0;
          if ($filters.hasResponse !== hasResponse) {
            return false;
          }
        }

        // Search filter
        if ($filters.searchQuery) {
          const query = $filters.searchQuery.toLowerCase();
          return critique.content.toLowerCase().includes(query) ||
                 critique.authorName.toLowerCase().includes(query);
        }

        return true;
      });
    }
  );

  const sortedCritiques = derived(
    [filteredCritiques, sortBy],
    ([$critiques, $sortBy]) => {
      const sorted = [...$critiques];

      switch ($sortBy) {
        case 'severity':
          const severityOrder = { critical: 0, major: 1, minor: 2, suggestion: 3 };
          sorted.sort((a, b) => severityOrder[a.severity] - severityOrder[b.severity]);
          break;
        case 'category':
          sorted.sort((a, b) => a.category.localeCompare(b.category));
          break;
        case 'time':
        default:
          sorted.sort((a, b) =>
            new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
          );
      }

      return sorted;
    }
  );

  const groupedCritiques = derived(sortedCritiques, ($critiques) => {
    const groups = new Map<string, Critique[]>();

    for (const critique of $critiques) {
      const key = critique.targetId;
      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(critique);
    }

    return groups;
  });

  const critiqueSummary = derived(
    [() => critiques],
    ([critiqueList]) => {
      const byCategory = new Map<CritiqueCategory, number>();
      const bySeverity = new Map<CritiqueSeverity, number>();
      const byAuthor = new Map<string, number>();
      let totalResponded = 0;

      for (const critique of critiqueList) {
        byCategory.set(
          critique.category,
          (byCategory.get(critique.category) || 0) + 1
        );
        bySeverity.set(
          critique.severity,
          (bySeverity.get(critique.severity) || 0) + 1
        );
        byAuthor.set(
          critique.authorId,
          (byAuthor.get(critique.authorId) || 0) + 1
        );

        if (critique.responses && critique.responses.length > 0) {
          totalResponded++;
        }
      }

      return {
        total: critiqueList.length,
        byCategory,
        bySeverity,
        byAuthor,
        responseRate: critiqueList.length > 0
          ? totalResponded / critiqueList.length
          : 0
      };
    }
  );

  const selectedCritique = derived(
    [() => critiques, selectedCritiqueId],
    ([critiqueList, $id]) => critiqueList.find(c => c.id === $id) || null
  );

  function selectCritique(id: string) {
    selectedCritiqueId.set(id);
    const critique = critiques.find(c => c.id === id);
    if (critique) {
      dispatch('highlight', { critiqueId: id, targetId: critique.targetId });
    }
  }

  function handleRespond(critiqueId: string, response: string) {
    dispatch('respond', { critiqueId, response });
  }

  function getSeverityColor(severity: CritiqueSeverity): string {
    switch (severity) {
      case 'critical': return 'var(--error-color)';
      case 'major': return 'var(--warning-color)';
      case 'minor': return 'var(--info-color)';
      case 'suggestion': return 'var(--success-color)';
      default: return 'var(--text-muted)';
    }
  }

  function getCategoryIcon(category: CritiqueCategory): string {
    switch (category) {
      case 'technical': return 'code';
      case 'logical': return 'logic';
      case 'stylistic': return 'pen';
      case 'factual': return 'check-circle';
      case 'structural': return 'layers';
      case 'completeness': return 'list';
      default: return 'message';
    }
  }
</script>

<div class="critique-viewer" data-testid="critique-viewer">
  <header class="viewer-header">
    <h3>Round {roundNumber} Critiques</h3>
    <div class="header-stats">
      <span class="stat">{$critiqueSummary.total} critiques</span>
      <span class="stat">{($critiqueSummary.responseRate * 100).toFixed(0)}% responded</span>
    </div>
  </header>

  <CritiqueSummary summary={$critiqueSummary} />

  <div class="viewer-toolbar">
    <CritiqueFilters
      {participants}
      bind:filters={$filters}
    />

    <div class="toolbar-right">
      <select bind:value={$sortBy} class="sort-select">
        <option value="time">Sort by Time</option>
        <option value="severity">Sort by Severity</option>
        <option value="category">Sort by Category</option>
      </select>

      <div class="view-toggle">
        <button
          class:active={$viewMode === 'list'}
          on:click={() => viewMode.set('list')}
          title="List view"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <rect x="3" y="4" width="18" height="4" />
            <rect x="3" y="10" width="18" height="4" />
            <rect x="3" y="16" width="18" height="4" />
          </svg>
        </button>
        <button
          class:active={$viewMode === 'grouped'}
          on:click={() => viewMode.set('grouped')}
          title="Grouped view"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <rect x="3" y="3" width="8" height="8" />
            <rect x="13" y="3" width="8" height="8" />
            <rect x="3" y="13" width="8" height="8" />
            <rect x="13" y="13" width="8" height="8" />
          </svg>
        </button>
        <button
          class:active={$viewMode === 'threaded'}
          on:click={() => viewMode.set('threaded')}
          title="Threaded view"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke-width="2"/>
          </svg>
        </button>
      </div>
    </div>
  </div>

  <div class="critique-content">
    {#if $sortedCritiques.length === 0}
      <div class="empty-state">
        <p>No critiques match your filters</p>
      </div>
    {:else if $viewMode === 'list'}
      <div class="critique-list">
        {#each $sortedCritiques as critique (critique.id)}
          <CritiqueCard
            {critique}
            selected={$selectedCritiqueId === critique.id}
            on:click={() => selectCritique(critique.id)}
            on:respond={(e) => handleRespond(critique.id, e.detail)}
          />
        {/each}
      </div>
    {:else if $viewMode === 'grouped'}
      <div class="critique-groups">
        {#each [...$groupedCritiques.entries()] as [targetId, targetCritiques] (targetId)}
          <div class="critique-group" transition:fade>
            <div class="group-header">
              <span class="group-target">
                {participants.find(p => p.id === targetId)?.name || 'Unknown'}
              </span>
              <span class="group-count">{targetCritiques.length} critiques</span>
            </div>
            <div class="group-critiques">
              {#each targetCritiques as critique (critique.id)}
                <CritiqueCard
                  {critique}
                  compact
                  selected={$selectedCritiqueId === critique.id}
                  on:click={() => selectCritique(critique.id)}
                />
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {:else if $viewMode === 'threaded'}
      <div class="critique-threads">
        {#each $sortedCritiques.filter(c => !c.parentId) as rootCritique (rootCritique.id)}
          <CritiqueThread
            critique={rootCritique}
            allCritiques={critiques}
            selected={$selectedCritiqueId === rootCritique.id}
            on:select={(e) => selectCritique(e.detail)}
            on:respond={(e) => handleRespond(e.detail.critiqueId, e.detail.response)}
          />
        {/each}
      </div>
    {/if}
  </div>

  {#if $selectedCritique}
    <aside class="critique-detail" transition:slide={{ axis: 'x' }}>
      <div class="detail-header">
        <h4>Critique Details</h4>
        <button
          class="close-btn"
          on:click={() => selectedCritiqueId.set(null)}
          aria-label="Close details"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M18 6L6 18M6 6l12 12" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="detail-content">
        <div class="detail-meta">
          <div class="meta-row">
            <span class="meta-label">From:</span>
            <span class="meta-value">{$selectedCritique.authorName}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">To:</span>
            <span class="meta-value">
              {participants.find(p => p.id === $selectedCritique.targetId)?.name || 'Unknown'}
            </span>
          </div>
          <div class="meta-row">
            <span class="meta-label">Category:</span>
            <span class="category-badge">{$selectedCritique.category}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">Severity:</span>
            <SentimentBadge
              severity={$selectedCritique.severity}
              sentiment={$selectedCritique.sentiment}
            />
          </div>
        </div>

        <div class="detail-body">
          <p>{$selectedCritique.content}</p>
        </div>

        {#if $selectedCritique.quotedText}
          <div class="quoted-section">
            <span class="quote-label">Referenced text:</span>
            <blockquote>{$selectedCritique.quotedText}</blockquote>
          </div>
        {/if}

        {#if $selectedCritique.responses?.length}
          <div class="responses-section">
            <h5>Responses ({$selectedCritique.responses.length})</h5>
            {#each $selectedCritique.responses as response}
              <div class="response-item">
                <div class="response-header">
                  <span class="responder">{response.authorName}</span>
                  <span class="response-time">
                    {new Date(response.createdAt).toLocaleTimeString()}
                  </span>
                </div>
                <p class="response-content">{response.content}</p>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </aside>
  {/if}
</div>

<style>
  .critique-viewer {
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

  .header-stats {
    display: flex;
    gap: 1rem;
  }

  .stat {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .viewer-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1.25rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .toolbar-right {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .sort-select {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.8125rem;
  }

  .view-toggle {
    display: flex;
    background: var(--card-bg);
    border-radius: 4px;
    overflow: hidden;
  }

  .view-toggle button {
    padding: 0.5rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .view-toggle button.active {
    background: var(--primary-color);
    color: white;
  }

  .critique-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .critique-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .critique-groups {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .critique-group {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    overflow: hidden;
  }

  .group-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .group-target {
    font-weight: 500;
  }

  .group-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .group-critiques {
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .critique-threads {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .critique-detail {
    width: 350px;
    border-left: 1px solid var(--border-color);
    background: var(--card-bg);
    overflow-y: auto;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .detail-header h4 {
    font-size: 0.9375rem;
    font-weight: 600;
  }

  .close-btn {
    padding: 0.25rem;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .detail-content {
    padding: 1rem;
  }

  .detail-meta {
    margin-bottom: 1rem;
  }

  .meta-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
    font-size: 0.8125rem;
  }

  .meta-label {
    color: var(--text-muted);
  }

  .meta-value {
    color: var(--text-primary);
  }

  .category-badge {
    padding: 0.125rem 0.5rem;
    background: var(--secondary-bg);
    border-radius: 3px;
    font-size: 0.75rem;
    text-transform: capitalize;
  }

  .detail-body {
    font-size: 0.9375rem;
    line-height: 1.6;
    margin-bottom: 1rem;
  }

  .quoted-section {
    margin-bottom: 1rem;
  }

  .quote-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    display: block;
    margin-bottom: 0.5rem;
  }

  .quoted-section blockquote {
    margin: 0;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-left: 3px solid var(--primary-color);
    font-size: 0.875rem;
    font-style: italic;
  }

  .responses-section h5 {
    font-size: 0.8125rem;
    font-weight: 500;
    margin-bottom: 0.75rem;
    color: var(--text-secondary);
  }

  .response-item {
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    margin-bottom: 0.5rem;
  }

  .response-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .responder {
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .response-time {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .response-content {
    font-size: 0.875rem;
    line-height: 1.5;
    margin: 0;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test filtering, sorting, and grouping logic
2. **Integration Tests**: Verify critique-draft linking works correctly
3. **Sentiment Tests**: Validate sentiment analysis display
4. **Thread Tests**: Test threaded view hierarchy
5. **Response Tests**: Verify response submission flow

## Related Specs
- Spec 262: Draft Viewer
- Spec 264: Conflict Highlights
- Spec 266: Dissent Log UI
