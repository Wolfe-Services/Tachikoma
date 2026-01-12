# Spec 261: Round Visualization

## Header
- **Spec ID**: 261
- **Phase**: 12 - Forge UI
- **Component**: Round Visualization
- **Dependencies**: Spec 256 (Forge Layout)
- **Status**: Draft

## Objective
Create a comprehensive visualization system for displaying deliberation rounds, including participant responses, voting patterns, convergence metrics, and round progression in real-time.

## Acceptance Criteria
1. Display current and historical rounds in a timeline view
2. Show participant contributions with attribution
3. Visualize voting and agreement patterns
4. Display convergence metrics with progress indicators
5. Support real-time updates during active deliberation
6. Provide detailed drill-down for individual responses
7. Enable round comparison across sessions
8. Animate transitions between round states

## Implementation

### RoundVisualization.svelte
```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide, fly } from 'svelte/transition';
  import RoundTimeline from './RoundTimeline.svelte';
  import RoundCard from './RoundCard.svelte';
  import ConvergenceChart from './ConvergenceChart.svelte';
  import ParticipantResponses from './ParticipantResponses.svelte';
  import VotingPattern from './VotingPattern.svelte';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import { subscribeToRoundUpdates } from '$lib/services/realtimeService';
  import type { Round, RoundState, ParticipantResponse } from '$lib/types/forge';

  export let sessionId: string;

  let selectedRound = writable<number | null>(null);
  let expandedResponses = writable<Set<string>>(new Set());
  let viewMode = writable<'timeline' | 'grid' | 'detailed'>('timeline');
  let unsubscribe: (() => void) | null = null;

  const rounds = derived(forgeSessionStore, ($store) => {
    return $store.activeSession?.rounds || [];
  });

  const currentRound = derived(
    [rounds, selectedRound],
    ([$rounds, $selected]) => {
      if ($selected !== null && $rounds[$selected]) {
        return $rounds[$selected];
      }
      return $rounds[$rounds.length - 1] || null;
    }
  );

  const convergenceHistory = derived(rounds, ($rounds) => {
    return $rounds.map((round, index) => ({
      round: index + 1,
      score: round.convergenceScore,
      timestamp: round.completedAt || round.startedAt
    }));
  });

  const roundStats = derived(currentRound, ($round) => {
    if (!$round) return null;

    const responses = $round.responses || [];
    const totalParticipants = $round.participants?.length || 0;
    const completedResponses = responses.filter(r => r.status === 'completed').length;

    return {
      participantCount: totalParticipants,
      responseCount: completedResponses,
      pendingCount: totalParticipants - completedResponses,
      averageConfidence: responses.reduce((sum, r) => sum + (r.confidence || 0), 0) / responses.length || 0,
      consensusPoints: $round.consensusPoints || 0,
      dissents: $round.dissents?.length || 0
    };
  });

  function selectRound(index: number) {
    selectedRound.set(index);
  }

  function toggleResponseExpand(responseId: string) {
    expandedResponses.update(set => {
      const newSet = new Set(set);
      if (newSet.has(responseId)) {
        newSet.delete(responseId);
      } else {
        newSet.add(responseId);
      }
      return newSet;
    });
  }

  function getRoundStateClass(state: RoundState): string {
    switch (state) {
      case 'pending': return 'state-pending';
      case 'in_progress': return 'state-active';
      case 'completed': return 'state-completed';
      case 'converged': return 'state-converged';
      case 'failed': return 'state-failed';
      default: return '';
    }
  }

  function formatDuration(startTime: Date, endTime?: Date): string {
    const end = endTime || new Date();
    const duration = Math.floor((end.getTime() - new Date(startTime).getTime()) / 1000);

    if (duration < 60) return `${duration}s`;
    if (duration < 3600) return `${Math.floor(duration / 60)}m ${duration % 60}s`;
    return `${Math.floor(duration / 3600)}h ${Math.floor((duration % 3600) / 60)}m`;
  }

  onMount(() => {
    unsubscribe = subscribeToRoundUpdates(sessionId, (update) => {
      forgeSessionStore.applyRoundUpdate(update);
    });
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });
</script>

<div class="round-visualization" data-testid="round-visualization">
  <header class="viz-header">
    <h2>Deliberation Rounds</h2>
    <div class="view-controls">
      <button
        class:active={$viewMode === 'timeline'}
        on:click={() => viewMode.set('timeline')}
      >
        Timeline
      </button>
      <button
        class:active={$viewMode === 'grid'}
        on:click={() => viewMode.set('grid')}
      >
        Grid
      </button>
      <button
        class:active={$viewMode === 'detailed'}
        on:click={() => viewMode.set('detailed')}
      >
        Detailed
      </button>
    </div>
  </header>

  {#if $rounds.length > 0}
    <RoundTimeline
      rounds={$rounds}
      selectedIndex={$selectedRound}
      on:select={(e) => selectRound(e.detail)}
    />

    <div class="convergence-section">
      <ConvergenceChart
        data={$convergenceHistory}
        threshold={$forgeSessionStore.activeSession?.config.convergenceThreshold || 0.8}
      />
    </div>

    {#if $currentRound}
      <div
        class="current-round"
        class:is-active={$currentRound.state === 'in_progress'}
        transition:fade={{ duration: 200 }}
      >
        <div class="round-header">
          <div class="round-title">
            <span class="round-number">Round {$currentRound.number}</span>
            <span class="round-state {getRoundStateClass($currentRound.state)}">
              {$currentRound.state.replace('_', ' ')}
            </span>
          </div>
          <div class="round-meta">
            <span class="duration">
              {formatDuration($currentRound.startedAt, $currentRound.completedAt)}
            </span>
            {#if $currentRound.state === 'in_progress'}
              <span class="live-indicator">
                <span class="pulse"></span>
                Live
              </span>
            {/if}
          </div>
        </div>

        {#if $roundStats}
          <div class="round-stats">
            <div class="stat">
              <span class="stat-value">{$roundStats.responseCount}/{$roundStats.participantCount}</span>
              <span class="stat-label">Responses</span>
            </div>
            <div class="stat">
              <span class="stat-value">{($currentRound.convergenceScore * 100).toFixed(0)}%</span>
              <span class="stat-label">Convergence</span>
            </div>
            <div class="stat">
              <span class="stat-value">{($roundStats.averageConfidence * 100).toFixed(0)}%</span>
              <span class="stat-label">Avg Confidence</span>
            </div>
            <div class="stat">
              <span class="stat-value">{$roundStats.consensusPoints}</span>
              <span class="stat-label">Consensus Points</span>
            </div>
            <div class="stat" class:has-dissent={$roundStats.dissents > 0}>
              <span class="stat-value">{$roundStats.dissents}</span>
              <span class="stat-label">Dissents</span>
            </div>
          </div>
        {/if}

        <div class="round-content" class:detailed={$viewMode === 'detailed'}>
          {#if $viewMode === 'detailed' || $viewMode === 'timeline'}
            <ParticipantResponses
              responses={$currentRound.responses || []}
              expandedIds={$expandedResponses}
              on:toggle={(e) => toggleResponseExpand(e.detail)}
            />
          {/if}

          {#if $viewMode === 'grid'}
            <div class="response-grid">
              {#each $currentRound.responses || [] as response (response.id)}
                <div
                  class="response-card"
                  class:expanded={$expandedResponses.has(response.id)}
                  on:click={() => toggleResponseExpand(response.id)}
                  transition:fly={{ y: 20, duration: 300 }}
                >
                  <div class="response-header">
                    <span class="participant-name">{response.participantName}</span>
                    <span class="confidence-badge">
                      {(response.confidence * 100).toFixed(0)}%
                    </span>
                  </div>
                  <p class="response-preview">
                    {response.content.slice(0, 150)}...
                  </p>
                  {#if response.votes}
                    <VotingPattern votes={response.votes} compact />
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>

        {#if $currentRound.oracleAnalysis}
          <div class="oracle-analysis" transition:slide>
            <h4>Oracle Analysis</h4>
            <div class="analysis-content">
              {$currentRound.oracleAnalysis}
            </div>
            {#if $currentRound.convergencePoints}
              <div class="convergence-points">
                <h5>Points of Convergence</h5>
                <ul>
                  {#each $currentRound.convergencePoints as point}
                    <li>{point}</li>
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="empty-state">
      <p>No deliberation rounds yet</p>
      <p class="hint">Rounds will appear here once the session starts</p>
    </div>
  {/if}
</div>

<style>
  .round-visualization {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding: 1.5rem;
    height: 100%;
    overflow-y: auto;
  }

  .viz-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .viz-header h2 {
    font-size: 1.25rem;
    font-weight: 600;
  }

  .view-controls {
    display: flex;
    gap: 0.25rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    padding: 0.25rem;
  }

  .view-controls button {
    padding: 0.5rem 0.75rem;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .view-controls button.active {
    background: var(--primary-color);
    color: white;
  }

  .convergence-section {
    background: var(--card-bg);
    border-radius: 8px;
    padding: 1rem;
  }

  .current-round {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.25rem;
  }

  .current-round.is-active {
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px var(--primary-alpha);
  }

  .round-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .round-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .round-number {
    font-size: 1.125rem;
    font-weight: 600;
  }

  .round-state {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    text-transform: capitalize;
  }

  .state-pending {
    background: var(--muted-bg);
    color: var(--text-muted);
  }

  .state-active {
    background: var(--primary-alpha);
    color: var(--primary-color);
  }

  .state-completed {
    background: var(--success-bg);
    color: var(--success-color);
  }

  .state-converged {
    background: var(--info-bg);
    color: var(--info-color);
  }

  .state-failed {
    background: var(--error-bg);
    color: var(--error-color);
  }

  .round-meta {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .duration {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .live-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--success-color);
    font-weight: 500;
  }

  .pulse {
    width: 8px;
    height: 8px;
    background: var(--success-color);
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 rgba(var(--success-rgb), 0.4);
    }
    70% {
      box-shadow: 0 0 0 10px rgba(var(--success-rgb), 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(var(--success-rgb), 0);
    }
  }

  .round-stats {
    display: flex;
    gap: 1.5rem;
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    margin-bottom: 1rem;
  }

  .stat {
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .stat.has-dissent .stat-value {
    color: var(--warning-color);
  }

  .response-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
  }

  .response-card {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 1rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .response-card:hover {
    border-color: var(--primary-color);
  }

  .response-card.expanded {
    grid-column: span 2;
  }

  .response-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .participant-name {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .confidence-badge {
    padding: 0.125rem 0.375rem;
    background: var(--primary-alpha);
    color: var(--primary-color);
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .response-preview {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .oracle-analysis {
    margin-top: 1.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border-color);
  }

  .oracle-analysis h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--oracle-color);
  }

  .analysis-content {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .convergence-points h5 {
    font-size: 0.8125rem;
    font-weight: 500;
    margin-top: 1rem;
    margin-bottom: 0.5rem;
  }

  .convergence-points ul {
    list-style: disc;
    padding-left: 1.25rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test round state transitions and stats calculations
2. **Integration Tests**: Verify real-time updates display correctly
3. **Animation Tests**: Ensure transitions are smooth and performant
4. **Visual Tests**: Validate responsive layouts across screen sizes
5. **Stress Tests**: Test with many rounds and participants

## Related Specs
- Spec 256: Forge Layout
- Spec 262: Draft Viewer
- Spec 263: Critique Viewer
- Spec 267: Convergence Indicator
