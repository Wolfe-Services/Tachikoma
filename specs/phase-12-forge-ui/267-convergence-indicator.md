# Spec 267: Convergence Indicator

## Header
- **Spec ID**: 267
- **Phase**: 12 - Forge UI
- **Component**: Convergence Indicator
- **Dependencies**: Spec 261 (Round Visualization)
- **Status**: Draft

## Objective
Create a visual indicator system that displays real-time convergence metrics during deliberation sessions, showing progress toward consensus and highlighting areas of agreement and remaining disagreement.

## Acceptance Criteria
1. Display overall convergence score as animated progress indicator
2. Show per-topic convergence breakdown
3. Visualize convergence trend across rounds
4. Highlight threshold proximity with warnings
5. Animate transitions as convergence changes
6. Display participant alignment matrix
7. Provide convergence velocity metrics
8. Support multiple convergence algorithms

## Implementation

### ConvergenceIndicator.svelte
```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import ConvergenceChart from './ConvergenceChart.svelte';
  import TopicConvergence from './TopicConvergence.svelte';
  import AlignmentMatrix from './AlignmentMatrix.svelte';
  import ConvergenceHistory from './ConvergenceHistory.svelte';
  import { convergenceService } from '$lib/services/convergenceService';
  import type {
    ConvergenceMetrics,
    TopicScore,
    ParticipantAlignment,
    ConvergenceHistory as History
  } from '$lib/types/forge';

  export let sessionId: string;
  export let threshold: number = 0.8;
  export let compact: boolean = false;

  let metrics = writable<ConvergenceMetrics | null>(null);
  let history = writable<History[]>([]);
  let viewMode = writable<'overview' | 'topics' | 'alignment' | 'history'>('overview');
  let unsubscribe: (() => void) | null = null;

  const overallScore = tweened(0, {
    duration: 800,
    easing: cubicOut
  });

  const velocity = derived(history, ($history) => {
    if ($history.length < 2) return 0;

    const recent = $history.slice(-3);
    const deltas = [];

    for (let i = 1; i < recent.length; i++) {
      deltas.push(recent[i].score - recent[i - 1].score);
    }

    return deltas.reduce((sum, d) => sum + d, 0) / deltas.length;
  });

  const estimatedRoundsToConverge = derived(
    [metrics, velocity],
    ([$metrics, $velocity]) => {
      if (!$metrics || $velocity <= 0) return null;

      const remaining = threshold - $metrics.overall;
      if (remaining <= 0) return 0;

      return Math.ceil(remaining / $velocity);
    }
  );

  const statusColor = derived(metrics, ($metrics) => {
    if (!$metrics) return 'var(--text-muted)';

    const ratio = $metrics.overall / threshold;

    if (ratio >= 1) return 'var(--success-color)';
    if (ratio >= 0.9) return 'var(--info-color)';
    if (ratio >= 0.7) return 'var(--warning-color)';
    return 'var(--error-color)';
  });

  const statusLabel = derived(metrics, ($metrics) => {
    if (!$metrics) return 'Calculating...';

    const ratio = $metrics.overall / threshold;

    if (ratio >= 1) return 'Converged';
    if (ratio >= 0.9) return 'Near Convergence';
    if (ratio >= 0.7) return 'Progressing';
    if (ratio >= 0.5) return 'Moderate Agreement';
    return 'Low Agreement';
  });

  function subscribeToUpdates() {
    unsubscribe = convergenceService.subscribe(sessionId, (update) => {
      metrics.set(update.metrics);
      overallScore.set(update.metrics.overall);

      history.update(h => [
        ...h,
        {
          roundNumber: update.roundNumber,
          score: update.metrics.overall,
          timestamp: new Date()
        }
      ]);
    });
  }

  async function loadInitialData() {
    const data = await convergenceService.getMetrics(sessionId);
    metrics.set(data.metrics);
    history.set(data.history);
    overallScore.set(data.metrics.overall);
  }

  function getScoreArcPath(score: number, radius: number = 60): string {
    const angle = score * 270 - 135; // -135 to 135 degrees
    const radians = (angle * Math.PI) / 180;
    const endX = radius * Math.cos(radians);
    const endY = radius * Math.sin(radians);
    const largeArc = score > 0.5 ? 1 : 0;

    const startRadians = (-135 * Math.PI) / 180;
    const startX = radius * Math.cos(startRadians);
    const startY = radius * Math.sin(startRadians);

    return `M ${startX} ${startY} A ${radius} ${radius} 0 ${largeArc} 1 ${endX} ${endY}`;
  }

  onMount(() => {
    loadInitialData();
    subscribeToUpdates();
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });
</script>

<div
  class="convergence-indicator"
  class:compact
  data-testid="convergence-indicator"
>
  {#if compact}
    <div class="compact-view">
      <div class="compact-gauge">
        <svg viewBox="-70 -70 140 140" class="gauge-svg">
          <path
            class="gauge-bg"
            d={getScoreArcPath(1, 60)}
            stroke-width="8"
            fill="none"
          />
          <path
            class="gauge-fill"
            d={getScoreArcPath($overallScore, 60)}
            stroke={$statusColor}
            stroke-width="8"
            fill="none"
            stroke-linecap="round"
          />
          <line
            class="threshold-marker"
            x1="0"
            y1="0"
            x2={60 * Math.cos((threshold * 270 - 135) * Math.PI / 180)}
            y2={60 * Math.sin((threshold * 270 - 135) * Math.PI / 180)}
            stroke="var(--warning-color)"
            stroke-width="2"
          />
        </svg>
        <div class="gauge-value">
          <span class="value">{($overallScore * 100).toFixed(0)}%</span>
          <span class="label">{$statusLabel}</span>
        </div>
      </div>

      {#if $velocity !== 0}
        <div class="compact-velocity" class:positive={$velocity > 0}>
          <span class="velocity-arrow">{$velocity > 0 ? '↑' : '↓'}</span>
          <span class="velocity-value">{Math.abs($velocity * 100).toFixed(1)}%</span>
        </div>
      {/if}
    </div>
  {:else}
    <div class="full-view">
      <header class="indicator-header">
        <h3>Convergence</h3>
        <div class="header-tabs">
          <button
            class:active={$viewMode === 'overview'}
            on:click={() => viewMode.set('overview')}
          >
            Overview
          </button>
          <button
            class:active={$viewMode === 'topics'}
            on:click={() => viewMode.set('topics')}
          >
            Topics
          </button>
          <button
            class:active={$viewMode === 'alignment'}
            on:click={() => viewMode.set('alignment')}
          >
            Alignment
          </button>
          <button
            class:active={$viewMode === 'history'}
            on:click={() => viewMode.set('history')}
          >
            History
          </button>
        </div>
      </header>

      <div class="indicator-content">
        {#if $viewMode === 'overview'}
          <div class="overview-view">
            <div class="main-gauge">
              <svg viewBox="-80 -80 160 160" class="gauge-svg large">
                <defs>
                  <linearGradient id="gaugeGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" style="stop-color: var(--error-color)" />
                    <stop offset="50%" style="stop-color: var(--warning-color)" />
                    <stop offset="100%" style="stop-color: var(--success-color)" />
                  </linearGradient>
                </defs>

                <path
                  class="gauge-bg"
                  d={getScoreArcPath(1, 70)}
                  stroke-width="12"
                  fill="none"
                />
                <path
                  class="gauge-fill animated"
                  d={getScoreArcPath($overallScore, 70)}
                  stroke={$statusColor}
                  stroke-width="12"
                  fill="none"
                  stroke-linecap="round"
                />

                <!-- Threshold marker -->
                <g class="threshold-group">
                  <line
                    x1="0"
                    y1="0"
                    x2={70 * Math.cos((threshold * 270 - 135) * Math.PI / 180)}
                    y2={70 * Math.sin((threshold * 270 - 135) * Math.PI / 180)}
                    stroke="var(--warning-color)"
                    stroke-width="2"
                    stroke-dasharray="4,2"
                  />
                  <circle
                    cx={70 * Math.cos((threshold * 270 - 135) * Math.PI / 180)}
                    cy={70 * Math.sin((threshold * 270 - 135) * Math.PI / 180)}
                    r="4"
                    fill="var(--warning-color)"
                  />
                </g>
              </svg>

              <div class="gauge-center">
                <span class="score-value">{($overallScore * 100).toFixed(0)}%</span>
                <span class="score-label">{$statusLabel}</span>
                <span class="threshold-label">Threshold: {(threshold * 100).toFixed(0)}%</span>
              </div>
            </div>

            <div class="metrics-sidebar">
              <div class="metric-card">
                <span class="metric-label">Velocity</span>
                <span class="metric-value" class:positive={$velocity > 0} class:negative={$velocity < 0}>
                  {$velocity > 0 ? '+' : ''}{($velocity * 100).toFixed(1)}% / round
                </span>
              </div>

              <div class="metric-card">
                <span class="metric-label">Est. Rounds to Converge</span>
                <span class="metric-value">
                  {#if $estimatedRoundsToConverge === null}
                    --
                  {:else if $estimatedRoundsToConverge === 0}
                    Converged
                  {:else}
                    ~{$estimatedRoundsToConverge} rounds
                  {/if}
                </span>
              </div>

              {#if $metrics}
                <div class="metric-card">
                  <span class="metric-label">Agreement Areas</span>
                  <span class="metric-value">{$metrics.agreementCount}</span>
                </div>

                <div class="metric-card">
                  <span class="metric-label">Disagreement Areas</span>
                  <span class="metric-value">{$metrics.disagreementCount}</span>
                </div>
              {/if}
            </div>
          </div>
        {:else if $viewMode === 'topics'}
          <TopicConvergence
            topics={$metrics?.topicScores || []}
            {threshold}
          />
        {:else if $viewMode === 'alignment'}
          <AlignmentMatrix
            alignments={$metrics?.participantAlignments || []}
          />
        {:else if $viewMode === 'history'}
          <ConvergenceHistory
            history={$history}
            {threshold}
          />
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .convergence-indicator {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
  }

  .compact {
    padding: 0.75rem;
  }

  .compact-view {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .compact-gauge {
    position: relative;
    width: 80px;
    height: 80px;
  }

  .gauge-svg {
    width: 100%;
    height: 100%;
  }

  .gauge-svg.large {
    width: 200px;
    height: 200px;
  }

  .gauge-bg {
    stroke: var(--border-color);
  }

  .gauge-fill {
    transition: stroke 0.3s ease;
  }

  .gauge-fill.animated {
    animation: pulse-glow 2s ease-in-out infinite;
  }

  @keyframes pulse-glow {
    0%, 100% {
      filter: drop-shadow(0 0 2px currentColor);
    }
    50% {
      filter: drop-shadow(0 0 8px currentColor);
    }
  }

  .gauge-value {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
  }

  .gauge-value .value {
    display: block;
    font-size: 1.25rem;
    font-weight: 700;
  }

  .gauge-value .label {
    font-size: 0.625rem;
    color: var(--text-muted);
  }

  .compact-velocity {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .compact-velocity.positive {
    color: var(--success-color);
  }

  .velocity-arrow {
    font-size: 0.875rem;
  }

  .full-view {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .indicator-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .indicator-header h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .header-tabs {
    display: flex;
    gap: 0.25rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    padding: 0.25rem;
  }

  .header-tabs button {
    padding: 0.375rem 0.75rem;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
    border-radius: 3px;
  }

  .header-tabs button.active {
    background: var(--primary-color);
    color: white;
  }

  .indicator-content {
    flex: 1;
    padding: 1.25rem;
    overflow-y: auto;
  }

  .overview-view {
    display: flex;
    gap: 2rem;
    align-items: flex-start;
  }

  .main-gauge {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .gauge-center {
    position: absolute;
    text-align: center;
  }

  .score-value {
    display: block;
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .score-label {
    display: block;
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-top: 0.25rem;
  }

  .threshold-label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.5rem;
  }

  .metrics-sidebar {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    flex: 1;
  }

  .metric-card {
    padding: 0.75rem 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .metric-label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.25rem;
  }

  .metric-value {
    font-size: 1rem;
    font-weight: 600;
  }

  .metric-value.positive {
    color: var(--success-color);
  }

  .metric-value.negative {
    color: var(--error-color);
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test score calculations and threshold comparisons
2. **Animation Tests**: Verify smooth transitions and tweening
3. **Real-time Tests**: Test subscription updates
4. **Visual Tests**: Validate gauge rendering accuracy
5. **Performance Tests**: Measure render performance with rapid updates

## Related Specs
- Spec 261: Round Visualization
- Spec 264: Conflict Highlights
- Spec 271: Result Preview
