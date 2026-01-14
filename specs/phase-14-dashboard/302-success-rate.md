# 302 - Success Rate

**Phase:** 14 - Dashboard
**Spec ID:** 302
**Status:** Planned
**Dependencies:** 296-dashboard-layout
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create success rate visualization components that display mission completion rates, failure analysis, and trend indicators for monitoring overall system health.

---

## Acceptance Criteria

- [x] `SuccessRateCard.svelte` component created
- [x] `SuccessRateGauge.svelte` circular gauge
- [x] Historical success rate trends
- [x] Failure reason breakdown
- [x] Comparison to previous periods
- [x] Target threshold indicators
- [x] Animated value transitions
- [x] Color-coded status levels

---

## Implementation Details

### 1. Success Rate Card (web/src/lib/components/metrics/SuccessRateCard.svelte)

```svelte
<script lang="ts">
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { fly, fade } from 'svelte/transition';
  import type { SuccessRateData } from '$lib/types/metrics';
  import Icon from '$lib/components/common/Icon.svelte';
  import SuccessRateGauge from './SuccessRateGauge.svelte';
  import SparkLine from '$lib/components/charts/SparkLine.svelte';

  export let data: SuccessRateData;
  export let showTrend: boolean = true;
  export let showBreakdown: boolean = true;
  export let target: number = 95;

  let expanded = false;

  const animatedRate = tweened(0, {
    duration: 1000,
    easing: cubicOut
  });

  $: {
    animatedRate.set(data.rate);
  }

  $: status = getStatus(data.rate, target);
  $: changeIndicator = getChangeIndicator(data.change);

  function getStatus(rate: number, threshold: number): {
    level: 'good' | 'warning' | 'critical';
    color: string;
    label: string;
  } {
    if (rate >= threshold) {
      return { level: 'good', color: 'var(--green-500)', label: 'On Target' };
    } else if (rate >= threshold - 10) {
      return { level: 'warning', color: 'var(--yellow-500)', label: 'Below Target' };
    }
    return { level: 'critical', color: 'var(--red-500)', label: 'Critical' };
  }

  function getChangeIndicator(change: number) {
    if (change > 0) {
      return { icon: 'trending-up', color: 'var(--green-500)', text: `+${change.toFixed(1)}%` };
    } else if (change < 0) {
      return { icon: 'trending-down', color: 'var(--red-500)', text: `${change.toFixed(1)}%` };
    }
    return { icon: 'minus', color: 'var(--gray-500)', text: '0%' };
  }
</script>

<div class="success-rate-card" class:expanded>
  <div class="card-header">
    <div class="header-title">
      <Icon name="target" size={18} />
      <h3>Success Rate</h3>
    </div>
    <span class="status-badge {status.level}">
      {status.label}
    </span>
  </div>

  <div class="card-body">
    <div class="gauge-section">
      <SuccessRateGauge
        value={$animatedRate}
        {target}
        size={140}
        strokeWidth={12}
      />
    </div>

    <div class="stats-section">
      <div class="main-stat">
        <span class="rate-value">{$animatedRate.toFixed(1)}%</span>
        {#if changeIndicator}
          <span class="rate-change" style="color: {changeIndicator.color}">
            <Icon name={changeIndicator.icon} size={14} />
            {changeIndicator.text}
          </span>
        {/if}
      </div>

      <div class="stat-row">
        <div class="stat-item">
          <span class="stat-value success">{data.successful}</span>
          <span class="stat-label">Successful</span>
        </div>
        <div class="stat-item">
          <span class="stat-value failed">{data.failed}</span>
          <span class="stat-label">Failed</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{data.total}</span>
          <span class="stat-label">Total</span>
        </div>
      </div>
    </div>
  </div>

  {#if showTrend && data.trendData}
    <div class="trend-section">
      <span class="trend-label">Last 7 days</span>
      <SparkLine
        data={data.trendData}
        height={40}
        color={status.color}
        showArea
      />
    </div>
  {/if}

  {#if showBreakdown}
    <button
      class="expand-toggle"
      on:click={() => expanded = !expanded}
      aria-expanded={expanded}
    >
      <span>Failure Breakdown</span>
      <Icon name={expanded ? 'chevron-up' : 'chevron-down'} size={16} />
    </button>

    {#if expanded && data.failureReasons}
      <div class="breakdown-section" transition:fly={{ y: -10, duration: 200 }}>
        <h4>Failure Reasons</h4>
        <ul class="failure-list">
          {#each data.failureReasons as reason}
            <li class="failure-item">
              <div class="failure-bar-container">
                <div
                  class="failure-bar"
                  style="width: {reason.percent}%"
                />
              </div>
              <span class="failure-label">{reason.reason}</span>
              <span class="failure-count">{reason.count}</span>
              <span class="failure-percent">{reason.percent.toFixed(1)}%</span>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}
</div>

<style>
  .success-rate-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .header-title h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .status-badge {
    padding: 0.25rem 0.625rem;
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 9999px;
  }

  .status-badge.good {
    background: var(--green-100);
    color: var(--green-700);
  }

  .status-badge.warning {
    background: var(--yellow-100);
    color: var(--yellow-700);
  }

  .status-badge.critical {
    background: var(--red-100);
    color: var(--red-700);
  }

  .card-body {
    display: flex;
    gap: 1.5rem;
    padding: 1.25rem;
  }

  .gauge-section {
    flex-shrink: 0;
  }

  .stats-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .main-stat {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .rate-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .rate-change {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .stat-row {
    display: flex;
    gap: 1.5rem;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .stat-value.success {
    color: var(--green-600);
  }

  .stat-value.failed {
    color: var(--red-600);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .trend-section {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .trend-label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.75rem;
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
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .breakdown-section h4 {
    margin: 0 0 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .failure-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .failure-item {
    display: grid;
    grid-template-columns: 100px 1fr auto auto;
    gap: 0.75rem;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-color);
  }

  .failure-item:last-child {
    border-bottom: none;
  }

  .failure-bar-container {
    height: 0.375rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
    overflow: hidden;
  }

  .failure-bar {
    height: 100%;
    background: var(--red-400);
    border-radius: 9999px;
  }

  .failure-label {
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .failure-count {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .failure-percent {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    width: 3rem;
    text-align: right;
  }
</style>
```

### 2. Success Rate Gauge (web/src/lib/components/metrics/SuccessRateGauge.svelte)

```svelte
<script lang="ts">
  export let value: number = 0;
  export let target: number = 95;
  export let size: number = 120;
  export let strokeWidth: number = 10;

  $: radius = (size - strokeWidth) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: progress = (value / 100) * circumference;
  $: targetProgress = (target / 100) * circumference;

  $: color = value >= target
    ? 'var(--green-500)'
    : value >= target - 10
      ? 'var(--yellow-500)'
      : 'var(--red-500)';

  $: bgColor = value >= target
    ? 'var(--green-100)'
    : value >= target - 10
      ? 'var(--yellow-100)'
      : 'var(--red-100)';
</script>

<svg
  class="success-gauge"
  width={size}
  height={size}
  viewBox="0 0 {size} {size}"
>
  <!-- Background circle -->
  <circle
    class="gauge-bg"
    cx={size / 2}
    cy={size / 2}
    r={radius}
    fill="none"
    stroke={bgColor}
    stroke-width={strokeWidth}
  />

  <!-- Target indicator -->
  <circle
    class="gauge-target"
    cx={size / 2}
    cy={size / 2}
    r={radius}
    fill="none"
    stroke="var(--border-color)"
    stroke-width={strokeWidth}
    stroke-dasharray="{targetProgress} {circumference}"
    stroke-dashoffset={circumference / 4}
    stroke-linecap="round"
  />

  <!-- Progress arc -->
  <circle
    class="gauge-progress"
    cx={size / 2}
    cy={size / 2}
    r={radius}
    fill="none"
    stroke={color}
    stroke-width={strokeWidth}
    stroke-dasharray="{progress} {circumference}"
    stroke-dashoffset={circumference / 4}
    stroke-linecap="round"
  />

  <!-- Center content -->
  <text
    class="gauge-value"
    x={size / 2}
    y={size / 2 - 4}
    text-anchor="middle"
    dominant-baseline="middle"
    fill={color}
  >
    {value.toFixed(0)}%
  </text>

  <text
    class="gauge-label"
    x={size / 2}
    y={size / 2 + 14}
    text-anchor="middle"
    dominant-baseline="middle"
  >
    of {target}% target
  </text>
</svg>

<style>
  .success-gauge {
    transform: rotate(-90deg);
  }

  .gauge-progress {
    transition: stroke-dasharray 0.5s ease;
  }

  .gauge-value {
    transform: rotate(90deg);
    transform-origin: center;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .gauge-label {
    transform: rotate(90deg);
    transform-origin: center;
    font-size: 0.625rem;
    fill: var(--text-tertiary);
  }
</style>
```

### 3. Success Rate Types (web/src/lib/types/metrics.ts)

```typescript
export interface FailureReason {
  reason: string;
  count: number;
  percent: number;
}

export interface SuccessRateData {
  rate: number;
  change: number;
  successful: number;
  failed: number;
  total: number;
  trendData: number[];
  failureReasons: FailureReason[];
}
```

---

## Testing Requirements

1. Gauge renders at correct percentage
2. Color changes based on target threshold
3. Trend sparkline shows correct data
4. Failure breakdown percentages sum correctly
5. Animated transitions are smooth
6. Status badge reflects current status
7. Responsive layout works at all sizes

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [303-time-series.md](303-time-series.md)
- Used by: Dashboard overview, metrics views
