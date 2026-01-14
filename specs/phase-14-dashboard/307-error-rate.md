# 307 - Error Rate

**Phase:** 14 - Dashboard
**Spec ID:** 307
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 303-time-series
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create error rate visualization components that display error frequency, error type distribution, error trends over time, and error impact analysis.

---

## Acceptance Criteria

- [x] `ErrorRateCard.svelte` component created
- [x] Error frequency over time chart
- [x] Error type distribution breakdown
- [x] Top errors list with details
- [x] Error severity indicators
- [x] Trend comparison with baseline
- [x] Alert threshold visualization
- [x] Error correlation analysis

---

## Implementation Details

### 1. Error Rate Card (web/src/lib/components/errors/ErrorRateCard.svelte)

```svelte
<script lang="ts">
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { fly, fade } from 'svelte/transition';
  import type { ErrorStats, ErrorItem } from '$lib/types/errors';
  import Icon from '$lib/components/common/Icon.svelte';
  import TimeSeriesChart from '$lib/components/charts/TimeSeriesChart.svelte';
  import DonutChart from '$lib/components/charts/DonutChart.svelte';

  export let stats: ErrorStats;
  export let errors: ErrorItem[] = [];
  export let alertThreshold: number = 5; // errors per minute
  export let showTrend: boolean = true;
  export let showBreakdown: boolean = true;

  let expanded = false;
  let selectedError: ErrorItem | null = null;

  const animatedRate = tweened(0, {
    duration: 800,
    easing: cubicOut
  });

  $: animatedRate.set(stats.currentRate);
  $: isAboveThreshold = stats.currentRate > alertThreshold;
  $: trendDirection = stats.changePercent > 0 ? 'up' : stats.changePercent < 0 ? 'down' : 'flat';

  $: errorTypeData = Object.entries(stats.byType).map(([type, count]) => ({
    label: type,
    value: count,
    color: getErrorTypeColor(type)
  }));

  function getErrorTypeColor(type: string): string {
    const colors: Record<string, string> = {
      'API Error': 'var(--red-500)',
      'Timeout': 'var(--yellow-500)',
      'Validation': 'var(--orange-500)',
      'Auth': 'var(--purple-500)',
      'Rate Limit': 'var(--blue-500)',
      'Unknown': 'var(--gray-500)'
    };
    return colors[type] || 'var(--gray-500)';
  }

  function getSeverityColor(severity: ErrorItem['severity']): string {
    switch (severity) {
      case 'critical': return 'var(--red-500)';
      case 'high': return 'var(--orange-500)';
      case 'medium': return 'var(--yellow-500)';
      case 'low': return 'var(--blue-500)';
      default: return 'var(--gray-500)';
    }
  }

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  }
</script>

<div class="error-rate-card" class:alert={isAboveThreshold}>
  <div class="card-header">
    <div class="header-left">
      <Icon name="alert-triangle" size={18} />
      <h3>Error Rate</h3>
    </div>

    {#if isAboveThreshold}
      <span class="alert-badge" transition:fade>
        <Icon name="alert-circle" size={14} />
        Above Threshold
      </span>
    {/if}
  </div>

  <div class="rate-display">
    <div class="current-rate">
      <span class="rate-value" class:critical={isAboveThreshold}>
        {$animatedRate.toFixed(2)}
      </span>
      <span class="rate-unit">errors/min</span>
    </div>

    <div class="rate-change" class:up={trendDirection === 'up'} class:down={trendDirection === 'down'}>
      <Icon
        name={trendDirection === 'up' ? 'trending-up' : trendDirection === 'down' ? 'trending-down' : 'minus'}
        size={16}
      />
      <span>{Math.abs(stats.changePercent).toFixed(1)}%</span>
      <span class="change-period">vs last hour</span>
    </div>
  </div>

  <div class="threshold-bar">
    <div class="bar-background">
      <div
        class="threshold-marker"
        style="left: {(alertThreshold / (alertThreshold * 2)) * 100}%"
      >
        <span class="threshold-label">Threshold</span>
      </div>
      <div
        class="bar-fill"
        class:danger={isAboveThreshold}
        style="width: {Math.min((stats.currentRate / (alertThreshold * 2)) * 100, 100)}%"
      />
    </div>
  </div>

  {#if showTrend}
    <div class="trend-section">
      <h4>Error Trend (24h)</h4>
      <TimeSeriesChart
        data={[{
          id: 'errors',
          label: 'Errors',
          color: isAboveThreshold ? 'var(--red-500)' : 'var(--accent-color)',
          points: stats.trendData.map(d => ({ timestamp: d.timestamp, value: d.count }))
        }]}
        height={150}
        showLegend={false}
        showBrush={false}
      />
    </div>
  {/if}

  {#if showBreakdown}
    <button
      class="expand-toggle"
      on:click={() => expanded = !expanded}
      aria-expanded={expanded}
    >
      <span>Error Breakdown</span>
      <Icon name={expanded ? 'chevron-up' : 'chevron-down'} size={16} />
    </button>

    {#if expanded}
      <div class="breakdown-section" transition:fly={{ y: -10, duration: 200 }}>
        <div class="breakdown-grid">
          <div class="type-chart">
            <DonutChart
              data={errorTypeData}
              size={120}
              showLabels={false}
            />
          </div>

          <div class="type-list">
            <h4>By Type</h4>
            {#each errorTypeData as item}
              <div class="type-item">
                <span class="type-color" style="background: {item.color}" />
                <span class="type-label">{item.label}</span>
                <span class="type-value">{item.value}</span>
                <span class="type-percent">
                  {((item.value / stats.totalErrors) * 100).toFixed(1)}%
                </span>
              </div>
            {/each}
          </div>
        </div>

        <div class="top-errors">
          <h4>Top Errors</h4>
          <ul class="error-list">
            {#each errors.slice(0, 5) as error (error.id)}
              <li
                class="error-item"
                class:selected={selectedError?.id === error.id}
                on:click={() => selectedError = selectedError?.id === error.id ? null : error}
                on:keypress
                role="button"
                tabindex="0"
              >
                <span class="error-severity" style="background: {getSeverityColor(error.severity)}" />
                <div class="error-info">
                  <span class="error-message">{error.message}</span>
                  <span class="error-meta">
                    {error.count} occurrences - Last: {formatTimestamp(error.lastSeen)}
                  </span>
                </div>
                <Icon name="chevron-right" size={14} />
              </li>

              {#if selectedError?.id === error.id}
                <li class="error-details" transition:fly={{ y: -5, duration: 150 }}>
                  <div class="detail-row">
                    <span class="detail-label">Type</span>
                    <span class="detail-value">{error.type}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">First Seen</span>
                    <span class="detail-value">{new Date(error.firstSeen).toLocaleString()}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Affected Missions</span>
                    <span class="detail-value">{error.affectedMissions}</span>
                  </div>
                  {#if error.stackTrace}
                    <div class="stack-trace">
                      <pre>{error.stackTrace}</pre>
                    </div>
                  {/if}
                </li>
              {/if}
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  {/if}

  <div class="card-footer">
    <span class="total-errors">
      Total: {stats.totalErrors.toLocaleString()} errors (24h)
    </span>
    <a href="/errors" class="view-all">
      View All
      <Icon name="arrow-right" size={14} />
    </a>
  </div>
</div>

<style>
  .error-rate-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .error-rate-card.alert {
    border-color: var(--red-300);
    animation: pulse-border 2s ease-in-out infinite;
  }

  @keyframes pulse-border {
    0%, 100% { border-color: var(--red-300); }
    50% { border-color: var(--red-500); }
  }

  .card-header {
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

  .alert-badge {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.625rem;
    background: var(--red-100);
    color: var(--red-700);
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 9999px;
  }

  .rate-display {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    padding: 1.25rem;
  }

  .current-rate {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  .rate-value {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .rate-value.critical {
    color: var(--red-500);
  }

  .rate-unit {
    font-size: 0.875rem;
    color: var(--text-tertiary);
  }

  .rate-change {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .rate-change.up {
    color: var(--red-500);
  }

  .rate-change.down {
    color: var(--green-500);
  }

  .change-period {
    color: var(--text-tertiary);
    font-size: 0.75rem;
  }

  .threshold-bar {
    padding: 0 1.25rem 1.25rem;
  }

  .bar-background {
    position: relative;
    height: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
  }

  .threshold-marker {
    position: absolute;
    top: -1.5rem;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .threshold-marker::after {
    content: '';
    position: absolute;
    top: 1rem;
    width: 2px;
    height: calc(100% + 0.5rem);
    background: var(--text-tertiary);
  }

  .threshold-label {
    font-size: 0.625rem;
    color: var(--text-tertiary);
    white-space: nowrap;
  }

  .bar-fill {
    height: 100%;
    background: var(--accent-color);
    border-radius: 9999px;
    transition: width 0.3s ease;
  }

  .bar-fill.danger {
    background: var(--red-500);
  }

  .trend-section {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .trend-section h4,
  .breakdown-section h4 {
    margin: 0 0 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .expand-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.75rem;
    border: none;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .expand-toggle:hover {
    background: var(--bg-hover);
  }

  .breakdown-section {
    padding: 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .breakdown-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 1.5rem;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .type-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0;
  }

  .type-color {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 0.125rem;
    flex-shrink: 0;
  }

  .type-label {
    flex: 1;
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .type-value {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .type-percent {
    width: 3rem;
    text-align: right;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .top-errors h4 {
    margin-top: 1rem;
  }

  .error-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .error-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .error-item:hover {
    background: var(--bg-hover);
  }

  .error-severity {
    width: 0.375rem;
    height: 2rem;
    border-radius: 9999px;
    flex-shrink: 0;
  }

  .error-info {
    flex: 1;
    min-width: 0;
  }

  .error-message {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .error-meta {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
  }

  .error-details {
    padding: 0.75rem;
    margin: 0 0.75rem 0.5rem 1.5rem;
    background: var(--bg-secondary);
    border-radius: 0.375rem;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 0.25rem 0;
    font-size: 0.75rem;
  }

  .detail-label {
    color: var(--text-tertiary);
  }

  .detail-value {
    color: var(--text-primary);
  }

  .stack-trace {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-primary);
    border-radius: 0.25rem;
    max-height: 100px;
    overflow: auto;
  }

  .stack-trace pre {
    margin: 0;
    font-size: 0.625rem;
    color: var(--text-secondary);
    white-space: pre-wrap;
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1.25rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
  }

  .total-errors {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .view-all {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.8125rem;
    color: var(--accent-color);
    text-decoration: none;
  }

  .view-all:hover {
    text-decoration: underline;
  }
</style>
```

### 2. Error Types (web/src/lib/types/errors.ts)

```typescript
export interface ErrorItem {
  id: string;
  type: string;
  message: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  count: number;
  firstSeen: string;
  lastSeen: string;
  affectedMissions: number;
  stackTrace?: string;
}

export interface ErrorTrendPoint {
  timestamp: string;
  count: number;
}

export interface ErrorStats {
  currentRate: number;
  changePercent: number;
  totalErrors: number;
  byType: Record<string, number>;
  trendData: ErrorTrendPoint[];
}
```

---

## Testing Requirements

1. Rate display updates correctly
2. Threshold alerts trigger appropriately
3. Trend chart shows accurate data
4. Type breakdown sums to total
5. Error details expand/collapse
6. Severity colors are correct
7. Animation on rate changes is smooth

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [308-performance-trend.md](308-performance-trend.md)
- Used by: Error monitoring views
