# 308 - Performance Trend

**Phase:** 14 - Dashboard
**Spec ID:** 308
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 303-time-series
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create performance trend visualization components that display mission execution times, throughput metrics, and performance degradation analysis over time.

---

## Acceptance Criteria

- [x] `PerformanceTrend.svelte` component created
- [x] Execution time trend charts
- [x] Throughput metrics display
- [x] P50/P90/P99 latency percentiles
- [x] Performance comparison periods
- [x] Anomaly detection highlights
- [x] Performance score calculation
- [x] Baseline comparison view

---

## Implementation Details

### 1. Performance Trend Component (web/src/lib/components/performance/PerformanceTrend.svelte)

```svelte
<script lang="ts">
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { fly, fade } from 'svelte/transition';
  import type { PerformanceData, PercentileData } from '$lib/types/performance';
  import Icon from '$lib/components/common/Icon.svelte';
  import TimeSeriesChart from '$lib/components/charts/TimeSeriesChart.svelte';
  import SparkLine from '$lib/components/charts/SparkLine.svelte';

  export let data: PerformanceData;
  export let showPercentiles: boolean = true;
  export let showThroughput: boolean = true;
  export let baseline: PerformanceData | null = null;

  let selectedMetric: 'latency' | 'throughput' = 'latency';
  let selectedPercentile: 'p50' | 'p90' | 'p99' = 'p50';

  $: performanceScore = calculateScore(data);
  $: scoreStatus = getScoreStatus(performanceScore);
  $: comparisonDiff = baseline ? data.avgLatency - baseline.avgLatency : null;

  const animatedScore = tweened(0, {
    duration: 1000,
    easing: cubicOut
  });

  $: animatedScore.set(performanceScore);

  function calculateScore(data: PerformanceData): number {
    // Score based on latency, throughput, and error rate
    const latencyScore = Math.max(0, 100 - (data.avgLatency / 100)); // Lower is better
    const throughputScore = Math.min(100, data.throughput / 10); // Higher is better
    const errorScore = Math.max(0, 100 - (data.errorRate * 10)); // Lower is better

    return (latencyScore * 0.4 + throughputScore * 0.3 + errorScore * 0.3);
  }

  function getScoreStatus(score: number): {
    label: string;
    color: string;
    icon: string;
  } {
    if (score >= 90) return { label: 'Excellent', color: 'var(--green-500)', icon: 'zap' };
    if (score >= 70) return { label: 'Good', color: 'var(--blue-500)', icon: 'thumbs-up' };
    if (score >= 50) return { label: 'Fair', color: 'var(--yellow-500)', icon: 'minus' };
    return { label: 'Poor', color: 'var(--red-500)', icon: 'alert-triangle' };
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms.toFixed(0)}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
    return `${(ms / 60000).toFixed(2)}m`;
  }

  function formatThroughput(val: number): string {
    if (val >= 1000) return `${(val / 1000).toFixed(1)}K/min`;
    return `${val.toFixed(1)}/min`;
  }

  $: chartData = selectedMetric === 'latency'
    ? [{
        id: 'latency',
        label: `${selectedPercentile.toUpperCase()} Latency`,
        color: 'var(--accent-color)',
        points: data.latencyTrend.map(d => ({
          timestamp: d.timestamp,
          value: d[selectedPercentile]
        }))
      }]
    : [{
        id: 'throughput',
        label: 'Throughput',
        color: 'var(--green-500)',
        points: data.throughputTrend.map(d => ({
          timestamp: d.timestamp,
          value: d.value
        }))
      }];
</script>

<div class="performance-trend">
  <div class="trend-header">
    <div class="header-left">
      <Icon name="activity" size={20} />
      <h3>Performance</h3>
    </div>

    <div class="metric-toggle">
      <button
        class:active={selectedMetric === 'latency'}
        on:click={() => selectedMetric = 'latency'}
      >
        Latency
      </button>
      <button
        class:active={selectedMetric === 'throughput'}
        on:click={() => selectedMetric = 'throughput'}
      >
        Throughput
      </button>
    </div>
  </div>

  <div class="score-section">
    <div class="score-display">
      <div class="score-ring" style="--color: {scoreStatus.color}">
        <svg viewBox="0 0 100 100">
          <circle
            class="score-bg"
            cx="50"
            cy="50"
            r="45"
            fill="none"
            stroke="var(--bg-secondary)"
            stroke-width="10"
          />
          <circle
            class="score-progress"
            cx="50"
            cy="50"
            r="45"
            fill="none"
            stroke={scoreStatus.color}
            stroke-width="10"
            stroke-dasharray="{($animatedScore / 100) * 283} 283"
            stroke-linecap="round"
            transform="rotate(-90 50 50)"
          />
        </svg>
        <div class="score-value">
          <span class="value">{$animatedScore.toFixed(0)}</span>
          <span class="label">Score</span>
        </div>
      </div>

      <div class="score-status" style="color: {scoreStatus.color}">
        <Icon name={scoreStatus.icon} size={16} />
        <span>{scoreStatus.label}</span>
      </div>
    </div>

    <div class="metrics-summary">
      <div class="metric-item">
        <span class="metric-label">Avg Latency</span>
        <span class="metric-value">{formatDuration(data.avgLatency)}</span>
        {#if comparisonDiff !== null}
          <span class="metric-diff" class:negative={comparisonDiff > 0}>
            {comparisonDiff > 0 ? '+' : ''}{formatDuration(comparisonDiff)}
          </span>
        {/if}
      </div>

      <div class="metric-item">
        <span class="metric-label">Throughput</span>
        <span class="metric-value">{formatThroughput(data.throughput)}</span>
        <SparkLine
          data={data.throughputTrend.map(d => d.value)}
          height={20}
          width={60}
          color="var(--green-500)"
        />
      </div>

      <div class="metric-item">
        <span class="metric-label">Error Rate</span>
        <span class="metric-value" class:danger={data.errorRate > 5}>
          {data.errorRate.toFixed(2)}%
        </span>
      </div>
    </div>
  </div>

  {#if showPercentiles && selectedMetric === 'latency'}
    <div class="percentiles-section">
      <div class="percentile-tabs">
        <button
          class:active={selectedPercentile === 'p50'}
          on:click={() => selectedPercentile = 'p50'}
        >
          P50
        </button>
        <button
          class:active={selectedPercentile === 'p90'}
          on:click={() => selectedPercentile = 'p90'}
        >
          P90
        </button>
        <button
          class:active={selectedPercentile === 'p99'}
          on:click={() => selectedPercentile = 'p99'}
        >
          P99
        </button>
      </div>

      <div class="percentile-values">
        <div class="percentile-item">
          <span class="percentile-label">P50</span>
          <span class="percentile-value">{formatDuration(data.percentiles.p50)}</span>
        </div>
        <div class="percentile-item">
          <span class="percentile-label">P90</span>
          <span class="percentile-value">{formatDuration(data.percentiles.p90)}</span>
        </div>
        <div class="percentile-item">
          <span class="percentile-label">P99</span>
          <span class="percentile-value">{formatDuration(data.percentiles.p99)}</span>
        </div>
      </div>
    </div>
  {/if}

  <div class="chart-section">
    <TimeSeriesChart
      data={chartData}
      height={200}
      showLegend={false}
      showBrush={false}
    />
  </div>

  {#if data.anomalies && data.anomalies.length > 0}
    <div class="anomalies-section">
      <h4>
        <Icon name="alert-circle" size={14} />
        Performance Anomalies
      </h4>
      <ul class="anomaly-list">
        {#each data.anomalies as anomaly}
          <li class="anomaly-item">
            <span class="anomaly-time">{new Date(anomaly.timestamp).toLocaleString()}</span>
            <span class="anomaly-desc">{anomaly.description}</span>
            <span class="anomaly-impact" style="color: {anomaly.severity === 'high' ? 'var(--red-500)' : 'var(--yellow-500)'}">
              {anomaly.impact}
            </span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>

<style>
  .performance-trend {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .trend-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .header-left h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .metric-toggle {
    display: flex;
    gap: 0.25rem;
    padding: 0.25rem;
    background: var(--bg-secondary);
    border-radius: 0.5rem;
  }

  .metric-toggle button {
    padding: 0.375rem 0.75rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .metric-toggle button.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    box-shadow: var(--shadow-sm);
  }

  .score-section {
    display: flex;
    gap: 2rem;
    padding: 1.25rem;
    align-items: center;
  }

  .score-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .score-ring {
    position: relative;
    width: 100px;
    height: 100px;
  }

  .score-ring svg {
    width: 100%;
    height: 100%;
  }

  .score-progress {
    transition: stroke-dasharray 0.5s ease;
  }

  .score-value {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .score-value .value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .score-value .label {
    font-size: 0.625rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
  }

  .score-status {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .metrics-summary {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .metric-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .metric-label {
    width: 80px;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .metric-value {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .metric-value.danger {
    color: var(--red-500);
  }

  .metric-diff {
    font-size: 0.75rem;
    color: var(--green-500);
  }

  .metric-diff.negative {
    color: var(--red-500);
  }

  .percentiles-section {
    padding: 0 1.25rem 1rem;
  }

  .percentile-tabs {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
  }

  .percentile-tabs button {
    padding: 0.25rem 0.625rem;
    border: 1px solid var(--border-color);
    background: transparent;
    border-radius: 0.375rem;
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .percentile-tabs button.active {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
  }

  .percentile-values {
    display: flex;
    gap: 1.5rem;
  }

  .percentile-item {
    display: flex;
    flex-direction: column;
  }

  .percentile-label {
    font-size: 0.625rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
  }

  .percentile-value {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
    font-family: monospace;
  }

  .chart-section {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .anomalies-section {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .anomalies-section h4 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0 0 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--yellow-600);
  }

  .anomaly-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .anomaly-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0;
    font-size: 0.75rem;
    border-bottom: 1px solid var(--border-color);
  }

  .anomaly-item:last-child {
    border-bottom: none;
  }

  .anomaly-time {
    color: var(--text-tertiary);
    white-space: nowrap;
  }

  .anomaly-desc {
    flex: 1;
    color: var(--text-primary);
  }

  .anomaly-impact {
    font-weight: 500;
  }
</style>
```

### 2. Performance Types (web/src/lib/types/performance.ts)

```typescript
export interface PercentileData {
  p50: number;
  p90: number;
  p99: number;
}

export interface LatencyTrendPoint {
  timestamp: string;
  p50: number;
  p90: number;
  p99: number;
}

export interface ThroughputTrendPoint {
  timestamp: string;
  value: number;
}

export interface PerformanceAnomaly {
  timestamp: string;
  description: string;
  severity: 'low' | 'medium' | 'high';
  impact: string;
}

export interface PerformanceData {
  avgLatency: number;
  throughput: number;
  errorRate: number;
  percentiles: PercentileData;
  latencyTrend: LatencyTrendPoint[];
  throughputTrend: ThroughputTrendPoint[];
  anomalies?: PerformanceAnomaly[];
}
```

---

## Testing Requirements

1. Score calculation is accurate
2. Percentile selection updates chart
3. Metric toggle switches correctly
4. Anomaly list displays properly
5. Baseline comparison calculates diff
6. Animation on score change is smooth
7. Duration formatting is accurate

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [309-summaries.md](309-summaries.md)
- Used by: Performance monitoring views
