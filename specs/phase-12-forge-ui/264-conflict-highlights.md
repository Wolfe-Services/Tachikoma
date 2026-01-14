# Spec 264: Conflict Highlights

## Header
- **Spec ID**: 264
- **Phase**: 12 - Forge UI
- **Component**: Conflict Highlights
- **Dependencies**: Spec 262 (Draft Viewer), Spec 263 (Critique Viewer)
- **Status**: Draft

## Objective
Create a visual system for highlighting and navigating conflicts between participant responses, including contradictory statements, incompatible recommendations, and areas of disagreement requiring resolution.

## Acceptance Criteria
- [x] Automatically detect and highlight conflicting statements across drafts
- [x] Categorize conflicts by type (factual, opinion, approach, priority)
- [x] Provide visual indicators showing conflict severity and scope
- [x] Enable navigation between related conflicts
- [x] Display conflict resolution suggestions from oracle
- [x] Track conflict resolution status through rounds
- [x] Support manual conflict flagging by users
- [x] Generate conflict summary reports

## Implementation

### ConflictHighlights.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, fly } from 'svelte/transition';
  import ConflictCard from './ConflictCard.svelte';
  import ConflictDetail from './ConflictDetail.svelte';
  import ConflictMap from './ConflictMap.svelte';
  import ConflictResolution from './ConflictResolution.svelte';
  import { conflictDetectionService } from '$lib/services/conflictDetection';
  import type {
    Conflict,
    ConflictType,
    ConflictSeverity,
    ConflictStatus,
    ConflictStatement
  } from '$lib/types/forge';

  export let drafts: { id: string; participantName: string; content: string }[] = [];
  export let roundNumber: number;
  export let previousConflicts: Conflict[] = [];

  const dispatch = createEventDispatcher<{
    highlight: { draftId: string; start: number; end: number };
    resolve: { conflictId: string; resolution: string };
    flag: { statement: ConflictStatement };
  }>();

  let conflicts = writable<Conflict[]>([]);
  let selectedConflictId = writable<string | null>(null);
  let isAnalyzing = writable<boolean>(false);
  let viewMode = writable<'list' | 'map' | 'timeline'>('list');
  let filterType = writable<ConflictType | 'all'>('all');
  let filterStatus = writable<ConflictStatus | 'all'>('all');
  let showResolved = writable<boolean>(false);

  const filteredConflicts = derived(
    [conflicts, filterType, filterStatus, showResolved],
    ([$conflicts, $type, $status, $showResolved]) => {
      return $conflicts.filter(conflict => {
        if ($type !== 'all' && conflict.type !== $type) return false;
        if ($status !== 'all' && conflict.status !== $status) return false;
        if (!$showResolved && conflict.status === 'resolved') return false;
        return true;
      });
    }
  );

  const conflictStats = derived(conflicts, ($conflicts) => ({
    total: $conflicts.length,
    byType: {
      factual: $conflicts.filter(c => c.type === 'factual').length,
      opinion: $conflicts.filter(c => c.type === 'opinion').length,
      approach: $conflicts.filter(c => c.type === 'approach').length,
      priority: $conflicts.filter(c => c.type === 'priority').length
    },
    bySeverity: {
      critical: $conflicts.filter(c => c.severity === 'critical').length,
      major: $conflicts.filter(c => c.severity === 'major').length,
      minor: $conflicts.filter(c => c.severity === 'minor').length
    },
    resolved: $conflicts.filter(c => c.status === 'resolved').length,
    pending: $conflicts.filter(c => c.status === 'pending').length
  }));

  const selectedConflict = derived(
    [conflicts, selectedConflictId],
    ([$conflicts, $id]) => $conflicts.find(c => c.id === $id) || null
  );

  async function analyzeConflicts() {
    isAnalyzing.set(true);

    try {
      const detected = await conflictDetectionService.analyze(drafts);

      // Merge with previous conflicts to track resolution
      const merged = mergeWithPrevious(detected, previousConflicts);
      conflicts.set(merged);
    } catch (error) {
      console.error('Conflict analysis failed:', error);
    } finally {
      isAnalyzing.set(false);
    }
  }

  function mergeWithPrevious(current: Conflict[], previous: Conflict[]): Conflict[] {
    const merged: Conflict[] = [];
    const previousMap = new Map(previous.map(c => [getConflictKey(c), c]));

    for (const conflict of current) {
      const key = getConflictKey(conflict);
      const prev = previousMap.get(key);

      if (prev) {
        merged.push({
          ...conflict,
          previousRounds: [...(prev.previousRounds || []), prev.roundNumber],
          resolutionAttempts: (prev.resolutionAttempts || 0) + (prev.status === 'pending' ? 1 : 0)
        });
        previousMap.delete(key);
      } else {
        merged.push(conflict);
      }
    }

    // Add resolved conflicts from previous
    for (const prev of previousMap.values()) {
      if (prev.status === 'resolved') {
        merged.push({ ...prev, status: 'resolved' });
      }
    }

    return merged;
  }

  function getConflictKey(conflict: Conflict): string {
    return `${conflict.statements.map(s => s.participantId).sort().join('-')}-${conflict.type}`;
  }

  function selectConflict(id: string) {
    selectedConflictId.set(id);
  }

  function highlightStatement(statement: ConflictStatement) {
    dispatch('highlight', {
      draftId: statement.draftId,
      start: statement.startOffset,
      end: statement.endOffset
    });
  }

  function handleResolve(conflictId: string, resolution: string) {
    conflicts.update(list =>
      list.map(c =>
        c.id === conflictId
          ? { ...c, status: 'resolved' as ConflictStatus, resolution }
          : c
      )
    );
    dispatch('resolve', { conflictId, resolution });
  }

  function flagAsConflict(statement: ConflictStatement) {
    dispatch('flag', { statement });
  }

  function getSeverityColor(severity: ConflictSeverity): string {
    switch (severity) {
      case 'critical': return 'var(--error-color)';
      case 'major': return 'var(--warning-color)';
      case 'minor': return 'var(--info-color)';
      default: return 'var(--text-muted)';
    }
  }

  $: if (drafts.length >= 2) {
    analyzeConflicts();
  }
</script>

<div class="conflict-highlights" data-testid="conflict-highlights">
  <header class="viewer-header">
    <div class="header-title">
      <h3>Conflicts</h3>
      {#if $isAnalyzing}
        <span class="analyzing-badge">Analyzing...</span>
      {/if}
    </div>

    <div class="header-actions">
      <button
        class="refresh-btn"
        on:click={analyzeConflicts}
        disabled={$isAnalyzing}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" stroke-width="2" stroke-linecap="round"/>
        </svg>
        Re-analyze
      </button>
    </div>
  </header>

  <div class="stats-bar">
    <div class="stat-item">
      <span class="stat-value">{$conflictStats.total}</span>
      <span class="stat-label">Total</span>
    </div>
    <div class="stat-item critical">
      <span class="stat-value">{$conflictStats.bySeverity.critical}</span>
      <span class="stat-label">Critical</span>
    </div>
    <div class="stat-item major">
      <span class="stat-value">{$conflictStats.bySeverity.major}</span>
      <span class="stat-label">Major</span>
    </div>
    <div class="stat-item minor">
      <span class="stat-value">{$conflictStats.bySeverity.minor}</span>
      <span class="stat-label">Minor</span>
    </div>
    <div class="stat-item resolved">
      <span class="stat-value">{$conflictStats.resolved}</span>
      <span class="stat-label">Resolved</span>
    </div>
  </div>

  <div class="toolbar">
    <div class="filters">
      <select bind:value={$filterType} class="filter-select">
        <option value="all">All Types</option>
        <option value="factual">Factual</option>
        <option value="opinion">Opinion</option>
        <option value="approach">Approach</option>
        <option value="priority">Priority</option>
      </select>

      <select bind:value={$filterStatus} class="filter-select">
        <option value="all">All Status</option>
        <option value="pending">Pending</option>
        <option value="acknowledged">Acknowledged</option>
        <option value="resolved">Resolved</option>
      </select>

      <label class="show-resolved">
        <input type="checkbox" bind:checked={$showResolved} />
        Show resolved
      </label>
    </div>

    <div class="view-toggle">
      <button
        class:active={$viewMode === 'list'}
        on:click={() => viewMode.set('list')}
        title="List view"
      >
        List
      </button>
      <button
        class:active={$viewMode === 'map'}
        on:click={() => viewMode.set('map')}
        title="Relationship map"
      >
        Map
      </button>
      <button
        class:active={$viewMode === 'timeline'}
        on:click={() => viewMode.set('timeline')}
        title="Timeline view"
      >
        Timeline
      </button>
    </div>
  </div>

  <div class="content-area">
    {#if $filteredConflicts.length === 0}
      <div class="empty-state" transition:fade>
        {#if $isAnalyzing}
          <div class="loading-spinner"></div>
          <p>Analyzing drafts for conflicts...</p>
        {:else}
          <p>No conflicts detected</p>
          <p class="hint">
            {drafts.length < 2
              ? 'Need at least 2 drafts to detect conflicts'
              : 'All participants appear to be in agreement'}
          </p>
        {/if}
      </div>
    {:else if $viewMode === 'list'}
      <div class="conflict-list">
        {#each $filteredConflicts as conflict (conflict.id)}
          <ConflictCard
            {conflict}
            selected={$selectedConflictId === conflict.id}
            on:click={() => selectConflict(conflict.id)}
            on:highlight={(e) => highlightStatement(e.detail)}
          />
        {/each}
      </div>
    {:else if $viewMode === 'map'}
      <ConflictMap
        conflicts={$filteredConflicts}
        participants={drafts.map(d => ({ id: d.id, name: d.participantName }))}
        selectedId={$selectedConflictId}
        on:select={(e) => selectConflict(e.detail)}
      />
    {:else if $viewMode === 'timeline'}
      <div class="conflict-timeline">
        {#each Array.from({ length: roundNumber }, (_, i) => i + 1) as round}
          <div class="timeline-round">
            <div class="round-marker">R{round}</div>
            <div class="round-conflicts">
              {#each $filteredConflicts.filter(c =>
                c.roundNumber === round || c.previousRounds?.includes(round)
              ) as conflict (conflict.id)}
                <div
                  class="timeline-conflict"
                  class:selected={$selectedConflictId === conflict.id}
                  class:resolved={conflict.status === 'resolved'}
                  style="border-left-color: {getSeverityColor(conflict.severity)}"
                  on:click={() => selectConflict(conflict.id)}
                  transition:fly={{ x: -20, duration: 200 }}
                >
                  <span class="conflict-type">{conflict.type}</span>
                  <span class="conflict-parties">
                    {conflict.statements.map(s => s.participantName).join(' vs ')}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}

    {#if $selectedConflict}
      <aside class="conflict-detail" transition:fly={{ x: 300, duration: 200 }}>
        <ConflictDetail
          conflict={$selectedConflict}
          on:close={() => selectedConflictId.set(null)}
          on:highlight={(e) => highlightStatement(e.detail)}
        />

        <ConflictResolution
          conflict={$selectedConflict}
          on:resolve={(e) => handleResolve($selectedConflict.id, e.detail)}
        />
      </aside>
    {/if}
  </div>
</div>

<style>
  .conflict-highlights {
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

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .header-title h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .analyzing-badge {
    padding: 0.25rem 0.5rem;
    background: var(--primary-alpha);
    color: var(--primary-color);
    border-radius: 4px;
    font-size: 0.75rem;
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .refresh-btn {
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

  .refresh-btn:hover:not(:disabled) {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .stats-bar {
    display: flex;
    gap: 1rem;
    padding: 0.75rem 1.25rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .stat-item {
    text-align: center;
    padding: 0.5rem 1rem;
  }

  .stat-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .stat-item.critical .stat-value {
    color: var(--error-color);
  }

  .stat-item.major .stat-value {
    color: var(--warning-color);
  }

  .stat-item.minor .stat-value {
    color: var(--info-color);
  }

  .stat-item.resolved .stat-value {
    color: var(--success-color);
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .filters {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .filter-select {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.8125rem;
  }

  .show-resolved {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .view-toggle {
    display: flex;
    background: var(--secondary-bg);
    border-radius: 4px;
    overflow: hidden;
  }

  .view-toggle button {
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.75rem;
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

  .conflict-list {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .conflict-timeline {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .timeline-round {
    display: flex;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .round-marker {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--primary-color);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.875rem;
    font-weight: 600;
    flex-shrink: 0;
  }

  .round-conflicts {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .timeline-conflict {
    padding: 0.75rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-left-width: 3px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .timeline-conflict:hover {
    background: var(--hover-bg);
  }

  .timeline-conflict.selected {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .timeline-conflict.resolved {
    opacity: 0.6;
  }

  .conflict-type {
    font-size: 0.75rem;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .conflict-parties {
    display: block;
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .conflict-detail {
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

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border-color);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test conflict detection algorithm accuracy
2. **Integration Tests**: Verify highlighting syncs with draft viewer
3. **Resolution Tests**: Test conflict resolution workflow
4. **Performance Tests**: Measure analysis time with large drafts
5. **Visual Tests**: Validate conflict visualization accuracy

## Related Specs
- Spec 262: Draft Viewer
- Spec 263: Critique Viewer
- Spec 267: Convergence Indicator
