# Spec 274: Comparison View

## Header
- **Spec ID**: 274
- **Phase**: 12 - Forge UI
- **Component**: Comparison View
- **Dependencies**: Spec 273 (History Browser)
- **Status**: Draft

## Objective
Create a side-by-side comparison view for analyzing multiple forge sessions, highlighting differences in approaches, outcomes, participant performance, and convergence patterns.

## Acceptance Criteria
1. Support comparison of 2-4 sessions simultaneously
2. Display side-by-side metrics and outcomes
3. Highlight key differences and similarities
4. Show participant performance across sessions
5. Visualize convergence pattern differences
6. Compare decision logs and rationales
7. Enable metric-by-metric detailed comparison
8. Export comparison reports

## Implementation

### ComparisonView.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import ComparisonHeader from './ComparisonHeader.svelte';
  import MetricsComparison from './MetricsComparison.svelte';
  import OutcomeComparison from './OutcomeComparison.svelte';
  import ParticipantComparison from './ParticipantComparison.svelte';
  import ConvergenceComparison from './ConvergenceComparison.svelte';
  import DecisionComparison from './DecisionComparison.svelte';
  import TimelineComparison from './TimelineComparison.svelte';
  import { sessionHistoryStore } from '$lib/stores/sessionHistory';
  import type { SessionSummary, ComparisonMetric } from '$lib/types/forge';

  export let sessionIds: string[];

  const dispatch = createEventDispatcher<{
    close: void;
    export: { format: string };
    removeSession: { sessionId: string };
  }>();

  let activeTab = writable<'overview' | 'metrics' | 'participants' | 'convergence' | 'decisions' | 'timeline'>('overview');
  let highlightDifferences = writable<boolean>(true);
  let normalizeMetrics = writable<boolean>(false);

  const sessions = derived(
    [sessionHistoryStore, () => sessionIds],
    ([$store]) =>
      sessionIds.map(id => $store.sessions.find(s => s.id === id)).filter(Boolean) as SessionSummary[]
  );

  const comparisonMetrics = derived(sessions, ($sessions) => {
    if ($sessions.length === 0) return null;

    const metrics: ComparisonMetric[] = [
      {
        name: 'Total Rounds',
        key: 'totalRounds',
        values: $sessions.map(s => s.totalRounds),
        best: 'min',
        format: 'number'
      },
      {
        name: 'Duration',
        key: 'durationMs',
        values: $sessions.map(s => s.durationMs || 0),
        best: 'min',
        format: 'duration'
      },
      {
        name: 'Final Convergence',
        key: 'finalConvergence',
        values: $sessions.map(s => s.finalConvergence || 0),
        best: 'max',
        format: 'percent'
      },
      {
        name: 'Participant Count',
        key: 'participantCount',
        values: $sessions.map(s => s.participants.length),
        best: null,
        format: 'number'
      },
      {
        name: 'Decisions Made',
        key: 'decisionCount',
        values: $sessions.map(s => s.decisionCount || 0),
        best: null,
        format: 'number'
      },
      {
        name: 'Dissent Count',
        key: 'dissentCount',
        values: $sessions.map(s => s.dissentCount || 0),
        best: 'min',
        format: 'number'
      },
      {
        name: 'Human Interventions',
        key: 'interventionCount',
        values: $sessions.map(s => s.interventionCount || 0),
        best: null,
        format: 'number'
      },
      {
        name: 'Result Confidence',
        key: 'resultConfidence',
        values: $sessions.map(s => s.resultConfidence || 0),
        best: 'max',
        format: 'percent'
      }
    ];

    return metrics;
  });

  const participantOverlap = derived(sessions, ($sessions) => {
    if ($sessions.length < 2) return { common: [], unique: [] };

    const allParticipants = $sessions.map(s => new Set(s.participants.map(p => p.id)));

    // Find common participants
    const common = [...allParticipants[0]].filter(id =>
      allParticipants.every(set => set.has(id))
    );

    // Find unique participants per session
    const unique = $sessions.map((s, i) => ({
      sessionId: s.id,
      sessionName: s.name,
      participants: s.participants.filter(p =>
        !allParticipants.filter((_, j) => j !== i).some(set => set.has(p.id))
      )
    }));

    return { common, unique };
  });

  const convergenceData = derived(sessions, ($sessions) => {
    return $sessions.map(session => ({
      sessionId: session.id,
      sessionName: session.name,
      data: session.convergenceHistory || []
    }));
  });

  function getBestValue(metric: ComparisonMetric): number | null {
    if (!metric.best) return null;

    const values = metric.values.filter(v => v !== null && v !== undefined);
    if (values.length === 0) return null;

    return metric.best === 'max' ? Math.max(...values) : Math.min(...values);
  }

  function isBestValue(metric: ComparisonMetric, value: number): boolean {
    const best = getBestValue(metric);
    return best !== null && value === best;
  }

  function formatValue(value: number, format: string): string {
    switch (format) {
      case 'percent':
        return `${(value * 100).toFixed(1)}%`;
      case 'duration':
        const minutes = Math.floor(value / 60000);
        const hours = Math.floor(minutes / 60);
        if (hours > 0) return `${hours}h ${minutes % 60}m`;
        return `${minutes}m`;
      default:
        return value.toLocaleString();
    }
  }

  function removeSession(sessionId: string) {
    dispatch('removeSession', { sessionId });
  }

  async function exportComparison(format: string) {
    dispatch('export', { format });
  }
</script>

<div class="comparison-view" data-testid="comparison-view">
  <ComparisonHeader
    sessions={$sessions}
    on:close={() => dispatch('close')}
    on:removeSession={(e) => removeSession(e.detail)}
    on:export={(e) => exportComparison(e.detail)}
  />

  <div class="comparison-tabs">
    <button
      class:active={$activeTab === 'overview'}
      on:click={() => activeTab.set('overview')}
    >
      Overview
    </button>
    <button
      class:active={$activeTab === 'metrics'}
      on:click={() => activeTab.set('metrics')}
    >
      Metrics
    </button>
    <button
      class:active={$activeTab === 'participants'}
      on:click={() => activeTab.set('participants')}
    >
      Participants
    </button>
    <button
      class:active={$activeTab === 'convergence'}
      on:click={() => activeTab.set('convergence')}
    >
      Convergence
    </button>
    <button
      class:active={$activeTab === 'decisions'}
      on:click={() => activeTab.set('decisions')}
    >
      Decisions
    </button>
    <button
      class:active={$activeTab === 'timeline'}
      on:click={() => activeTab.set('timeline')}
    >
      Timeline
    </button>
  </div>

  <div class="comparison-options">
    <label class="option">
      <input type="checkbox" bind:checked={$highlightDifferences} />
      Highlight differences
    </label>
    <label class="option">
      <input type="checkbox" bind:checked={$normalizeMetrics} />
      Normalize metrics
    </label>
  </div>

  <div class="comparison-content">
    {#if $activeTab === 'overview'}
      <div class="overview-grid">
        {#each $sessions as session, i (session.id)}
          <div class="session-overview" transition:fade>
            <div class="session-header" style="border-left-color: var(--color-{i + 1})">
              <h3>{session.name}</h3>
              <span class="session-date">
                {new Date(session.createdAt).toLocaleDateString()}
              </span>
            </div>

            <div class="session-goal">
              <p>{session.goal.slice(0, 200)}{session.goal.length > 200 ? '...' : ''}</p>
            </div>

            <div class="session-stats">
              <div class="stat">
                <span class="stat-value">{session.totalRounds}</span>
                <span class="stat-label">Rounds</span>
              </div>
              <div class="stat">
                <span class="stat-value">{(session.finalConvergence * 100).toFixed(0)}%</span>
                <span class="stat-label">Convergence</span>
              </div>
              <div class="stat">
                <span class="stat-value">{session.participants.length}</span>
                <span class="stat-label">Participants</span>
              </div>
            </div>

            <div class="session-outcome">
              <span class="outcome-label">Status:</span>
              <span class="outcome-status {session.status}">{session.status}</span>
            </div>

            {#if session.hasResult}
              <div class="result-preview">
                <h4>Result Preview</h4>
                <p>{session.resultSummary?.slice(0, 150)}...</p>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {:else if $activeTab === 'metrics'}
      <MetricsComparison
        sessions={$sessions}
        metrics={$comparisonMetrics || []}
        highlightDifferences={$highlightDifferences}
        normalize={$normalizeMetrics}
      />
    {:else if $activeTab === 'participants'}
      <ParticipantComparison
        sessions={$sessions}
        overlap={$participantOverlap}
      />
    {:else if $activeTab === 'convergence'}
      <ConvergenceComparison
        data={$convergenceData}
        highlightDifferences={$highlightDifferences}
      />
    {:else if $activeTab === 'decisions'}
      <DecisionComparison
        sessions={$sessions}
      />
    {:else if $activeTab === 'timeline'}
      <TimelineComparison
        sessions={$sessions}
      />
    {/if}
  </div>

  {#if $comparisonMetrics}
    <div class="metrics-summary" transition:slide>
      <h4>Key Differences</h4>
      <div class="difference-list">
        {#each $comparisonMetrics.filter(m => {
          const values = m.values.filter(v => v !== null);
          if (values.length < 2) return false;
          const min = Math.min(...values);
          const max = Math.max(...values);
          return max > min * 1.2; // 20% difference threshold
        }) as metric}
          <div class="difference-item">
            <span class="diff-name">{metric.name}</span>
            <span class="diff-range">
              {formatValue(Math.min(...metric.values), metric.format)} -
              {formatValue(Math.max(...metric.values), metric.format)}
            </span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .comparison-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
  }

  .comparison-tabs {
    display: flex;
    gap: 0.25rem;
    padding: 0 1.5rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .comparison-tabs button {
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .comparison-tabs button.active {
    color: var(--primary-color);
    border-bottom-color: var(--primary-color);
  }

  .comparison-tabs button:hover:not(.active) {
    color: var(--text-primary);
  }

  .comparison-options {
    display: flex;
    gap: 1.5rem;
    padding: 0.75rem 1.5rem;
    background: var(--card-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .comparison-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .session-overview {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
  }

  .session-header {
    padding: 1rem 1.25rem;
    border-left: 4px solid;
    background: var(--secondary-bg);
  }

  .session-header h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .session-date {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .session-goal {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .session-goal p {
    font-size: 0.875rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .session-stats {
    display: flex;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .stat {
    flex: 1;
    text-align: center;
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

  .session-outcome {
    padding: 0.75rem 1.25rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .outcome-label {
    color: var(--text-muted);
  }

  .outcome-status {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    text-transform: capitalize;
  }

  .outcome-status.completed {
    background: var(--success-alpha);
    color: var(--success-color);
  }

  .outcome-status.stopped {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .result-preview {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
    background: var(--secondary-bg);
  }

  .result-preview h4 {
    font-size: 0.8125rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
    color: var(--text-secondary);
  }

  .result-preview p {
    font-size: 0.8125rem;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .metrics-summary {
    padding: 1rem 1.5rem;
    background: var(--card-bg);
    border-top: 1px solid var(--border-color);
  }

  .metrics-summary h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
  }

  .difference-list {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .difference-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border-radius: 4px;
  }

  .diff-name {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .diff-range {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--primary-color);
  }

  :root {
    --color-1: #4a9eff;
    --color-2: #ff6b6b;
    --color-3: #4ecdc4;
    --color-4: #ffe66d;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test metric comparison calculations
2. **Integration Tests**: Verify multi-session loading
3. **Visual Tests**: Validate side-by-side display accuracy
4. **Difference Tests**: Test difference highlighting algorithm
5. **Export Tests**: Validate comparison report generation

## Related Specs
- Spec 273: History Browser
- Spec 261: Round Visualization
- Spec 267: Convergence Indicator
