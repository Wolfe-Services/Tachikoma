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
    padding: var(--space-4);
    background: var(--color-bg-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
  }

  .cost-tracking__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-4);
  }

  .cost-tracking__title {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
    margin: 0;
  }

  .cost-tracking__total {
    font-size: var(--text-lg);
    font-weight: var(--font-bold);
    color: var(--color-text-primary);
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  }

  .cost-tracking__total.near-budget {
    color: var(--color-warning);
  }

  .cost-tracking__total.over-budget {
    color: var(--color-error);
  }

  .cost-tracking__budget {
    margin-top: var(--space-4);
  }

  .budget-bar {
    height: 8px;
    background: var(--color-bg-hover);
    border-radius: var(--radius-sm);
    overflow: hidden;
    margin-bottom: var(--space-2);
  }

  .budget-bar__fill {
    height: 100%;
    background: var(--color-success);
    transition: width var(--duration-300) var(--ease-out);
  }

  .budget-bar__fill--warning {
    background: var(--color-warning);
  }

  .budget-bar__fill--danger {
    background: var(--color-error);
  }

  .budget-label {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .cost-tracking__projection {
    margin-top: var(--space-3);
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  .projection-warning {
    color: var(--color-error);
    margin-left: var(--space-2);
  }
</style>