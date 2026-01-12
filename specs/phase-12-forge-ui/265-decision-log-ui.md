# Spec 265: Decision Log UI

## Header
- **Spec ID**: 265
- **Phase**: 12 - Forge UI
- **Component**: Decision Log UI
- **Dependencies**: Spec 261 (Round Visualization)
- **Status**: Draft

## Objective
Create a comprehensive decision logging interface that records and displays all decisions made during deliberation sessions, including oracle rulings, consensus points, and human interventions with full audit trail capabilities.

## Acceptance Criteria
1. Display chronological log of all decisions made during session
2. Categorize decisions by type (consensus, oracle ruling, human override)
3. Show decision rationale and supporting evidence
4. Track decision dependencies and implications
5. Enable search and filtering of decision history
6. Export decision logs for compliance and review
7. Support decision annotation and tagging
8. Visualize decision impact on session progression

## Implementation

### DecisionLogUI.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import DecisionCard from './DecisionCard.svelte';
  import DecisionDetail from './DecisionDetail.svelte';
  import DecisionTimeline from './DecisionTimeline.svelte';
  import DecisionExport from './DecisionExport.svelte';
  import { decisionLogStore } from '$lib/stores/decisionLog';
  import type {
    Decision,
    DecisionType,
    DecisionStatus,
    DecisionFilter
  } from '$lib/types/forge';

  export let sessionId: string;

  const dispatch = createEventDispatcher<{
    annotate: { decisionId: string; note: string };
    tag: { decisionId: string; tags: string[] };
    export: { format: string; decisions: Decision[] };
  }>();

  let selectedDecisionId = writable<string | null>(null);
  let searchQuery = writable<string>('');
  let filterType = writable<DecisionType | 'all'>('all');
  let filterRound = writable<number | 'all'>('all');
  let viewMode = writable<'list' | 'timeline' | 'tree'>('list');
  let showExportDialog = writable<boolean>(false);

  const decisions = derived(decisionLogStore, ($store) =>
    $store.decisions.filter(d => d.sessionId === sessionId)
  );

  const filteredDecisions = derived(
    [decisions, searchQuery, filterType, filterRound],
    ([$decisions, $query, $type, $round]) => {
      return $decisions.filter(decision => {
        // Type filter
        if ($type !== 'all' && decision.type !== $type) return false;

        // Round filter
        if ($round !== 'all' && decision.roundNumber !== $round) return false;

        // Search filter
        if ($query) {
          const query = $query.toLowerCase();
          return (
            decision.title.toLowerCase().includes(query) ||
            decision.description.toLowerCase().includes(query) ||
            decision.rationale?.toLowerCase().includes(query) ||
            decision.tags?.some(t => t.toLowerCase().includes(query))
          );
        }

        return true;
      });
    }
  );

  const groupedByRound = derived(filteredDecisions, ($decisions) => {
    const groups = new Map<number, Decision[]>();

    for (const decision of $decisions) {
      const round = decision.roundNumber;
      if (!groups.has(round)) {
        groups.set(round, []);
      }
      groups.get(round)!.push(decision);
    }

    return groups;
  });

  const decisionStats = derived(decisions, ($decisions) => ({
    total: $decisions.length,
    byType: {
      consensus: $decisions.filter(d => d.type === 'consensus').length,
      oracle: $decisions.filter(d => d.type === 'oracle_ruling').length,
      human: $decisions.filter(d => d.type === 'human_override').length,
      automated: $decisions.filter(d => d.type === 'automated').length
    },
    impactful: $decisions.filter(d => d.impact === 'high').length,
    withDissent: $decisions.filter(d => d.dissents && d.dissents.length > 0).length
  }));

  const rounds = derived(decisions, ($decisions) => {
    const roundSet = new Set($decisions.map(d => d.roundNumber));
    return Array.from(roundSet).sort((a, b) => a - b);
  });

  const selectedDecision = derived(
    [decisions, selectedDecisionId],
    ([$decisions, $id]) => $decisions.find(d => d.id === $id) || null
  );

  function selectDecision(id: string) {
    selectedDecisionId.set(id);
  }

  function getTypeIcon(type: DecisionType): string {
    switch (type) {
      case 'consensus': return 'check-circle';
      case 'oracle_ruling': return 'gavel';
      case 'human_override': return 'user';
      case 'automated': return 'cpu';
      default: return 'circle';
    }
  }

  function getTypeColor(type: DecisionType): string {
    switch (type) {
      case 'consensus': return 'var(--success-color)';
      case 'oracle_ruling': return 'var(--primary-color)';
      case 'human_override': return 'var(--warning-color)';
      case 'automated': return 'var(--info-color)';
      default: return 'var(--text-muted)';
    }
  }

  function handleAnnotate(decisionId: string, note: string) {
    dispatch('annotate', { decisionId, note });
  }

  function handleTag(decisionId: string, tags: string[]) {
    dispatch('tag', { decisionId, tags });
  }

  function handleExport(format: string) {
    dispatch('export', { format, decisions: $filteredDecisions });
    showExportDialog.set(false);
  }

  onMount(() => {
    decisionLogStore.loadForSession(sessionId);
  });
</script>

<div class="decision-log-ui" data-testid="decision-log-ui">
  <header class="log-header">
    <div class="header-title">
      <h3>Decision Log</h3>
      <span class="decision-count">{$decisionStats.total} decisions</span>
    </div>

    <div class="header-actions">
      <button
        class="export-btn"
        on:click={() => showExportDialog.set(true)}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" stroke-width="2" stroke-linecap="round"/>
        </svg>
        Export
      </button>
    </div>
  </header>

  <div class="stats-bar">
    <div class="stat-item">
      <span class="stat-icon consensus">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
      </span>
      <span class="stat-value">{$decisionStats.byType.consensus}</span>
      <span class="stat-label">Consensus</span>
    </div>
    <div class="stat-item">
      <span class="stat-icon oracle">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/>
        </svg>
      </span>
      <span class="stat-value">{$decisionStats.byType.oracle}</span>
      <span class="stat-label">Oracle</span>
    </div>
    <div class="stat-item">
      <span class="stat-icon human">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
        </svg>
      </span>
      <span class="stat-value">{$decisionStats.byType.human}</span>
      <span class="stat-label">Human</span>
    </div>
    <div class="stat-item highlight">
      <span class="stat-value">{$decisionStats.impactful}</span>
      <span class="stat-label">High Impact</span>
    </div>
    <div class="stat-item warning">
      <span class="stat-value">{$decisionStats.withDissent}</span>
      <span class="stat-label">With Dissent</span>
    </div>
  </div>

  <div class="toolbar">
    <div class="search-box">
      <input
        type="search"
        placeholder="Search decisions..."
        bind:value={$searchQuery}
        class="search-input"
      />
    </div>

    <div class="filters">
      <select bind:value={$filterType} class="filter-select">
        <option value="all">All Types</option>
        <option value="consensus">Consensus</option>
        <option value="oracle_ruling">Oracle Ruling</option>
        <option value="human_override">Human Override</option>
        <option value="automated">Automated</option>
      </select>

      <select bind:value={$filterRound} class="filter-select">
        <option value="all">All Rounds</option>
        {#each $rounds as round}
          <option value={round}>Round {round}</option>
        {/each}
      </select>
    </div>

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
        class:active={$viewMode === 'timeline'}
        on:click={() => viewMode.set('timeline')}
        title="Timeline view"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" stroke-width="2"/>
        </svg>
      </button>
      <button
        class:active={$viewMode === 'tree'}
        on:click={() => viewMode.set('tree')}
        title="Dependency tree"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M4 4v7h7M20 20v-7h-7M4 20l7-7M20 4l-7 7" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="content-area">
    {#if $filteredDecisions.length === 0}
      <div class="empty-state" transition:fade>
        <p>No decisions found</p>
        <p class="hint">
          {$searchQuery || $filterType !== 'all' || $filterRound !== 'all'
            ? 'Try adjusting your filters'
            : 'Decisions will appear here as the session progresses'}
        </p>
      </div>
    {:else if $viewMode === 'list'}
      <div class="decision-list">
        {#each [...$groupedByRound.entries()].sort((a, b) => b[0] - a[0]) as [round, roundDecisions] (round)}
          <div class="round-group" transition:slide>
            <div class="round-header">
              <span class="round-label">Round {round}</span>
              <span class="round-count">{roundDecisions.length} decisions</span>
            </div>
            <div class="round-decisions">
              {#each roundDecisions as decision (decision.id)}
                <DecisionCard
                  {decision}
                  selected={$selectedDecisionId === decision.id}
                  on:click={() => selectDecision(decision.id)}
                />
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {:else if $viewMode === 'timeline'}
      <DecisionTimeline
        decisions={$filteredDecisions}
        selectedId={$selectedDecisionId}
        on:select={(e) => selectDecision(e.detail)}
      />
    {:else if $viewMode === 'tree'}
      <div class="decision-tree">
        <!-- Tree visualization component -->
        <p class="coming-soon">Dependency tree view coming soon</p>
      </div>
    {/if}

    {#if $selectedDecision}
      <aside class="decision-detail" transition:slide={{ axis: 'x' }}>
        <DecisionDetail
          decision={$selectedDecision}
          on:close={() => selectedDecisionId.set(null)}
          on:annotate={(e) => handleAnnotate($selectedDecision.id, e.detail)}
          on:tag={(e) => handleTag($selectedDecision.id, e.detail)}
        />
      </aside>
    {/if}
  </div>

  {#if $showExportDialog}
    <DecisionExport
      decisions={$filteredDecisions}
      on:export={(e) => handleExport(e.detail)}
      on:close={() => showExportDialog.set(false)}
    />
  {/if}
</div>

<style>
  .decision-log-ui {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
    border-radius: 8px;
    overflow: hidden;
  }

  .log-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .header-title h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .decision-count {
    padding: 0.25rem 0.5rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .export-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .export-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .stats-bar {
    display: flex;
    gap: 1rem;
    padding: 0.75rem 1.25rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .stat-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--card-bg);
    border-radius: 4px;
  }

  .stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 4px;
  }

  .stat-icon.consensus {
    background: var(--success-alpha);
    color: var(--success-color);
  }

  .stat-icon.oracle {
    background: var(--primary-alpha);
    color: var(--primary-color);
  }

  .stat-icon.human {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .stat-value {
    font-weight: 600;
    font-size: 0.9375rem;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .stat-item.highlight .stat-value {
    color: var(--primary-color);
  }

  .stat-item.warning .stat-value {
    color: var(--warning-color);
  }

  .toolbar {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .search-box {
    flex: 1;
    max-width: 300px;
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
  }

  .filters {
    display: flex;
    gap: 0.5rem;
  }

  .filter-select {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.8125rem;
  }

  .view-toggle {
    display: flex;
    background: var(--secondary-bg);
    border-radius: 4px;
    overflow: hidden;
    margin-left: auto;
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

  .content-area {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .decision-list {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .round-group {
    margin-bottom: 1.5rem;
  }

  .round-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .round-label {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .round-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .round-decisions {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .decision-detail {
    width: 400px;
    border-left: 1px solid var(--border-color);
    background: var(--card-bg);
    overflow-y: auto;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .coming-soon {
    text-align: center;
    padding: 3rem;
    color: var(--text-muted);
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test filtering, sorting, and grouping logic
2. **Integration Tests**: Verify decision logging during session
3. **Export Tests**: Validate export formats (JSON, CSV, PDF)
4. **Search Tests**: Test search functionality accuracy
5. **Accessibility Tests**: Ensure keyboard navigation works

## Related Specs
- Spec 261: Round Visualization
- Spec 266: Dissent Log UI
- Spec 270: Human Intervention
