<script lang="ts">
  export let inputTokens: number;
  export let outputTokens: number;
  export let inputCost: number;
  export let outputCost: number;

  function formatTokens(tokens: number): string {
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(2)}M`;
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}k`;
    return tokens.toString();
  }

  function formatCost(amount: number, currency = 'USD'): string {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency,
      minimumFractionDigits: 4,
      maximumFractionDigits: 4,
    }).format(amount);
  }

  $: totalTokens = inputTokens + outputTokens;
  $: totalCost = inputCost + outputCost;
</script>

<div class="cost-breakdown">
  <div class="cost-breakdown__row">
    <span class="cost-breakdown__label">Input</span>
    <span class="cost-breakdown__tokens">{formatTokens(inputTokens)} tokens</span>
    <span class="cost-breakdown__cost">{formatCost(inputCost)}</span>
  </div>
  
  <div class="cost-breakdown__row">
    <span class="cost-breakdown__label">Output</span>
    <span class="cost-breakdown__tokens">{formatTokens(outputTokens)} tokens</span>
    <span class="cost-breakdown__cost">{formatCost(outputCost)}</span>
  </div>
  
  <div class="cost-breakdown__separator"></div>
  
  <div class="cost-breakdown__row cost-breakdown__row--total">
    <span class="cost-breakdown__label">Total</span>
    <span class="cost-breakdown__tokens">{formatTokens(totalTokens)} tokens</span>
    <span class="cost-breakdown__cost">{formatCost(totalCost)}</span>
  </div>
</div>

<style>
  .cost-breakdown {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .cost-breakdown__row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 12px;
    align-items: center;
  }

  .cost-breakdown__row--total {
    font-weight: 600;
    color: var(--text);
  }

  .cost-breakdown__label {
    font-size: 13px;
    color: var(--text-muted);
  }

  .cost-breakdown__row--total .cost-breakdown__label {
    color: var(--text);
  }

  .cost-breakdown__tokens {
    font-size: 12px;
    color: var(--text-muted);
    text-align: right;
    min-width: 80px;
  }

  .cost-breakdown__cost {
    font-size: 13px;
    color: var(--text);
    text-align: right;
    min-width: 70px;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  }

  .cost-breakdown__separator {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
</style>