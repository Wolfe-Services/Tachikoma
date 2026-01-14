# 300 - Cost Summary

**Phase:** 14 - Dashboard
**Spec ID:** 300
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 011-common-core-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create a cost summary component that displays API usage costs, budget tracking, and spending trends with visual breakdowns by model, mission, and time period.

---

## Acceptance Criteria

- [x] `CostSummary.svelte` component created
- [x] Total cost display with currency formatting
- [x] Cost breakdown by model (Claude, etc.)
- [x] Cost breakdown by mission
- [x] Budget tracking with limits
- [x] Period comparison (daily, weekly, monthly)
- [x] Cost projection estimates
- [x] Alert thresholds configuration

---

## Implementation Details

### 1. Cost Summary Component (web/src/lib/components/cost/CostSummary.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import type { CostData, CostBreakdown, BudgetConfig } from '$lib/types/cost';
  import { costStore } from '$lib/stores/cost';
  import Icon from '$lib/components/common/Icon.svelte';
  import DonutChart from '$lib/components/charts/DonutChart.svelte';
  import SparkLine from '$lib/components/charts/SparkLine.svelte';

  export let period: 'day' | 'week' | 'month' = 'month';
  export let showBreakdown: boolean = true;
  export let showProjection: boolean = true;

  let expanded = false;

  $: data = $costStore[period];
  $: budgetPercent = data?.budget ? (data.totalCost / data.budget.limit) * 100 : 0;
  $: isOverBudget = budgetPercent > 100;
  $: isNearBudget = budgetPercent > 80 && budgetPercent <= 100;

  const animatedCost = tweened(0, {
    duration: 800,
    easing: cubicOut
  });

  $: if (data?.totalCost) {
    animatedCost.set(data.totalCost);
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount);
  }

  function formatCompact(amount: number): string {
    if (amount >= 1000) {
      return `$${(amount / 1000).toFixed(1)}k`;
    }
    return formatCurrency(amount);
  }

  function getChangeIndicator(change: number): { icon: string; color: string; text: string } {
    if (change > 0) {
      return { icon: 'trending-up', color: 'var(--red-500)', text: `+${change.toFixed(1)}%` };
    } else if (change < 0) {
      return { icon: 'trending-down', color: 'var(--green-500)', text: `${change.toFixed(1)}%` };
    }
    return { icon: 'minus', color: 'var(--gray-500)', text: '0%' };
  }

  $: changeIndicator = data ? getChangeIndicator(data.changePercent) : null;
</script>

<div class="cost-summary" class:expanded class:over-budget={isOverBudget} class:near-budget={isNearBudget}>
  <div class="summary-header">
    <div class="header-left">
      <Icon name="dollar-sign" size={20} />
      <h3 class="title">Cost Summary</h3>
    </div>

    <div class="period-selector">
      <button
        class:active={period === 'day'}
        on:click={() => period = 'day'}
      >Day</button>
      <button
        class:active={period === 'week'}
        on:click={() => period = 'week'}
      >Week</button>
      <button
        class:active={period === 'month'}
        on:click={() => period = 'month'}
      >Month</button>
    </div>
  </div>

  <div class="main-cost">
    <div class="cost-value">
      <span class="amount">{formatCurrency($animatedCost)}</span>
      {#if changeIndicator}
        <span class="change" style="color: {changeIndicator.color}">
          <Icon name={changeIndicator.icon} size={14} />
          {changeIndicator.text}
        </span>
      {/if}
    </div>

    {#if data?.budget}
      <div class="budget-bar">
        <div class="budget-progress">
          <div
            class="budget-fill"
            class:warning={isNearBudget}
            class:danger={isOverBudget}
            style="width: {Math.min(budgetPercent, 100)}%"
          />
        </div>
        <span class="budget-label">
          {formatCompact(data.budget.limit - data.totalCost)} remaining of {formatCompact(data.budget.limit)}
        </span>
      </div>
    {/if}
  </div>

  {#if data?.sparklineData}
    <div class="trend-chart">
      <SparkLine
        data={data.sparklineData}
        height={40}
        color={isOverBudget ? 'var(--red-500)' : 'var(--accent-color)'}
      />
    </div>
  {/if}

  {#if showBreakdown && data?.breakdown}
    <button
      class="expand-toggle"
      on:click={() => expanded = !expanded}
      aria-expanded={expanded}
    >
      <span>View Breakdown</span>
      <Icon name={expanded ? 'chevron-up' : 'chevron-down'} size={16} />
    </button>

    {#if expanded}
      <div class="breakdown-section" transition:fly={{ y: -10, duration: 200 }}>
        <div class="breakdown-grid">
          <div class="breakdown-chart">
            <DonutChart
              data={data.breakdown.byModel}
              size={120}
              showLegend={false}
            />
          </div>

          <div class="breakdown-list">
            <h4>By Model</h4>
            {#each data.breakdown.byModel as item}
              <div class="breakdown-item">
                <span class="item-color" style="background: {item.color}" />
                <span class="item-label">{item.label}</span>
                <span class="item-value">{formatCurrency(item.value)}</span>
                <span class="item-percent">{item.percent.toFixed(1)}%</span>
              </div>
            {/each}
          </div>
        </div>

        <div class="breakdown-missions">
          <h4>Top Missions by Cost</h4>
          <ul class="mission-cost-list">
            {#each data.breakdown.topMissions.slice(0, 5) as mission}
              <li class="mission-cost-item">
                <span class="mission-name">{mission.title}</span>
                <span class="mission-cost">{formatCurrency(mission.cost)}</span>
              </li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  {/if}

  {#if showProjection && data?.projection}
    <div class="projection" class:warning={data.projection.projectedCost > (data.budget?.limit || Infinity)}>
      <Icon name="activity" size={14} />
      <span>
        Projected {period === 'month' ? 'monthly' : period === 'week' ? 'weekly' : 'daily'} total:
        <strong>{formatCurrency(data.projection.projectedCost)}</strong>
      </span>
    </div>
  {/if}
</div>

<style>
  .cost-summary {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .cost-summary.over-budget {
    border-color: var(--red-300);
  }

  .cost-summary.near-budget {
    border-color: var(--yellow-300);
  }

  .summary-header {
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

  .title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .period-selector {
    display: flex;
    gap: 0.25rem;
    padding: 0.25rem;
    background: var(--bg-secondary);
    border-radius: 0.5rem;
  }

  .period-selector button {
    padding: 0.375rem 0.75rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .period-selector button.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    box-shadow: var(--shadow-sm);
  }

  .main-cost {
    padding: 1.25rem;
  }

  .cost-value {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
  }

  .amount {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .change {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .budget-bar {
    margin-top: 1rem;
  }

  .budget-progress {
    height: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
    overflow: hidden;
  }

  .budget-fill {
    height: 100%;
    background: var(--accent-color);
    border-radius: 9999px;
    transition: width 0.3s ease;
  }

  .budget-fill.warning {
    background: var(--yellow-500);
  }

  .budget-fill.danger {
    background: var(--red-500);
  }

  .budget-label {
    display: block;
    margin-top: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .trend-chart {
    padding: 0 1.25rem;
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
    background: var(--bg-primary);
  }

  .breakdown-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 1.5rem;
    align-items: center;
  }

  .breakdown-list h4,
  .breakdown-missions h4 {
    margin: 0 0 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .breakdown-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0;
  }

  .item-color {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 0.25rem;
  }

  .item-label {
    flex: 1;
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .item-value {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .item-percent {
    width: 3rem;
    text-align: right;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .breakdown-missions {
    margin-top: 1.25rem;
    padding-top: 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .mission-cost-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .mission-cost-item {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    font-size: 0.8125rem;
    border-bottom: 1px solid var(--border-color);
  }

  .mission-cost-item:last-child {
    border-bottom: none;
  }

  .mission-name {
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }

  .mission-cost {
    font-weight: 500;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .projection {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .projection.warning {
    background: var(--red-50);
    color: var(--red-700);
  }

  .projection strong {
    color: inherit;
  }
</style>
```

### 2. Cost Types (web/src/lib/types/cost.ts)

```typescript
export interface CostBreakdownItem {
  label: string;
  value: number;
  percent: number;
  color: string;
}

export interface MissionCost {
  id: string;
  title: string;
  specId: string;
  cost: number;
  tokenUsage: {
    input: number;
    output: number;
  };
}

export interface CostBreakdown {
  byModel: CostBreakdownItem[];
  topMissions: MissionCost[];
}

export interface BudgetConfig {
  limit: number;
  alertThreshold: number;
  period: 'day' | 'week' | 'month';
}

export interface CostProjection {
  projectedCost: number;
  confidence: number;
  basedOnDays: number;
}

export interface CostData {
  totalCost: number;
  changePercent: number;
  breakdown: CostBreakdown;
  budget: BudgetConfig | null;
  sparklineData: number[];
  projection: CostProjection;
}

export interface CostStore {
  day: CostData | null;
  week: CostData | null;
  month: CostData | null;
}
```

### 3. Cost Store (web/src/lib/stores/cost.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { CostStore, CostData } from '$lib/types/cost';

const initialState: CostStore = {
  day: null,
  week: null,
  month: null
};

function createCostStore() {
  const { subscribe, set, update } = writable<CostStore>(initialState);

  return {
    subscribe,
    setCostData: (period: 'day' | 'week' | 'month', data: CostData) =>
      update(s => ({ ...s, [period]: data })),
    reset: () => set(initialState)
  };
}

export const costStore = createCostStore();

export const totalMonthlyCost = derived(
  costStore,
  $store => $store.month?.totalCost ?? 0
);

export const isOverBudget = derived(
  costStore,
  $store => {
    const monthData = $store.month;
    if (!monthData?.budget) return false;
    return monthData.totalCost > monthData.budget.limit;
  }
);
```

---

## Testing Requirements

1. Cost values format correctly with currency
2. Budget progress bar reflects usage
3. Period switching loads correct data
4. Sparkline renders trend correctly
5. Breakdown percentages sum to 100%
6. Over-budget state triggers warnings
7. Projection calculates correctly

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [301-token-charts.md](301-token-charts.md)
- Used by: Dashboard overview, cost management views
