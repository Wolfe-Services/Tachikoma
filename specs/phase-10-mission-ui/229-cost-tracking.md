# 229 - Cost Tracking Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 229
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a cost tracking component that displays real-time cost information for mission execution, including input/output token costs, cumulative spending, and cost projections.

---

## Acceptance Criteria

- [ ] Real-time cost display with input/output breakdown
- [ ] Cumulative cost tracking across sessions
- [ ] Cost projection based on current usage
- [ ] Budget alerts and warnings
- [ ] Cost history graph
- [ ] Export cost reports

---

## Implementation Details

### 1. Types (src/lib/types/cost.ts)

```typescript
export interface CostInfo {
  inputTokens: number;
  outputTokens: number;
  inputCost: number;
  outputCost: number;
  totalCost: number;
  currency: string;
}

export interface CostHistory {
  timestamp: string;
  cost: number;
  tokens: number;
  action: string;
}

export interface CostBudget {
  daily: number;
  weekly: number;
  monthly: number;
  perMission: number;
}

export interface CostProjection {
  estimatedTotal: number;
  remainingBudget: number;
  projectedOverage: number;
  confidence: number;
}
```

### 2. Cost Tracking Component (src/lib/components/mission/CostTracking.svelte)

```svelte
<script lang="ts">
  import type { CostInfo, CostBudget, CostProjection } from '$lib/types/cost';
  import CostBreakdown from './CostBreakdown.svelte';
  import CostGraph from './CostGraph.svelte';

  export let cost: CostInfo;
  export let budget: CostBudget | null = null;
  export let projection: CostProjection | null = null;
  export let history: { timestamp: string; cost: number }[] = [];

  function formatCost(amount: number, currency = 'USD'): string {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency,
      minimumFractionDigits: 4,
      maximumFractionDigits: 4,
    }).format(amount);
  }

  function formatTokens(tokens: number): string {
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(2)}M`;
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}k`;
    return tokens.toString();
  }

  $: budgetUsage = budget?.perMission
    ? (cost.totalCost / budget.perMission) * 100
    : 0;

  $: isOverBudget = budgetUsage > 100;
  $: isNearBudget = budgetUsage > 80 && !isOverBudget;
</script>

<div class="cost-tracking">
  <div class="cost-tracking__header">
    <h3 class="cost-tracking__title">Cost</h3>
    <span
      class="cost-tracking__total"
      class:over-budget={isOverBudget}
      class:near-budget={isNearBudget}
    >
      {formatCost(cost.totalCost, cost.currency)}
    </span>
  </div>

  <CostBreakdown
    inputTokens={cost.inputTokens}
    outputTokens={cost.outputTokens}
    inputCost={cost.inputCost}
    outputCost={cost.outputCost}
  />

  {#if budget?.perMission}
    <div class="cost-tracking__budget">
      <div class="budget-bar">
        <div
          class="budget-bar__fill"
          class:budget-bar__fill--warning={isNearBudget}
          class:budget-bar__fill--danger={isOverBudget}
          style="width: {Math.min(budgetUsage, 100)}%"
        ></div>
      </div>
      <span class="budget-label">
        {budgetUsage.toFixed(1)}% of {formatCost(budget.perMission)} budget
      </span>
    </div>
  {/if}

  {#if projection}
    <div class="cost-tracking__projection">
      <span>Projected: {formatCost(projection.estimatedTotal)}</span>
      {#if projection.projectedOverage > 0}
        <span class="projection-warning">
          +{formatCost(projection.projectedOverage)} over budget
        </span>
      {/if}
    </div>
  {/if}

  {#if history.length > 0}
    <CostGraph data={history} />
  {/if}
</div>

<style>
  .cost-tracking {
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }

  .cost-tracking__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .cost-tracking__title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .cost-tracking__total {
    font-size: 18px;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .cost-tracking__total.near-budget {
    color: var(--color-warning);
  }

  .cost-tracking__total.over-budget {
    color: var(--color-error);
  }

  .cost-tracking__budget {
    margin-top: 16px;
  }

  .budget-bar {
    height: 8px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .budget-bar__fill {
    height: 100%;
    background: var(--color-success);
    transition: width 0.3s ease;
  }

  .budget-bar__fill--warning {
    background: var(--color-warning);
  }

  .budget-bar__fill--danger {
    background: var(--color-error);
  }

  .budget-label {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .cost-tracking__projection {
    margin-top: 12px;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .projection-warning {
    color: var(--color-error);
    margin-left: 8px;
  }
</style>
```

---

## Testing Requirements

1. Cost displays with correct formatting
2. Budget bar shows correct percentage
3. Warning states trigger appropriately
4. Projection calculates correctly

---

## Related Specs

- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [230-context-meter.md](230-context-meter.md)
