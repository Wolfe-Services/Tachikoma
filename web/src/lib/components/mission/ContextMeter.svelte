<script lang="ts">
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import type { ContextUsage, ContextZone } from '$lib/types/context';
  import { CONTEXT_THRESHOLDS, getContextZone } from '$lib/types/context';

  export let usage: ContextUsage;
  export let compact = false;

  const animatedPercent = tweened(0, { duration: 400, easing: cubicOut });

  $: animatedPercent.set(usage.usagePercent);
  $: zone = getContextZone($animatedPercent);

  const zoneColors: Record<ContextZone, string> = {
    safe: 'var(--color-success)',
    warning: 'var(--color-warning)',
    danger: 'var(--orange-500)',
    critical: 'var(--color-error)',
  };

  function formatTokens(tokens: number): string {
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}k`;
    return tokens.toString();
  }
</script>

<div
  class="context-meter"
  class:context-meter--compact={compact}
  class:context-meter--critical={zone === 'critical'}
  role="meter"
  aria-valuenow={usage.usagePercent}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-label="Context window usage"
>
  <div class="context-meter__bar">
    <div
      class="context-meter__fill"
      style="width: {$animatedPercent}%; background-color: {zoneColors[zone]}"
    ></div>

    <!-- Threshold markers -->
    <div
      class="context-meter__threshold"
      style="left: {CONTEXT_THRESHOLDS.warning}%"
      title="Warning threshold"
    ></div>
    <div
      class="context-meter__threshold context-meter__threshold--danger"
      style="left: {CONTEXT_THRESHOLDS.danger}%"
      title="Danger threshold"
    ></div>
    <div
      class="context-meter__threshold context-meter__threshold--critical"
      style="left: {CONTEXT_THRESHOLDS.critical}%"
      title="Critical threshold"
    ></div>
  </div>

  <div class="context-meter__info">
    <span class="context-meter__percent" style="color: {zoneColors[zone]}">
      {Math.round($animatedPercent)}%
    </span>

    {#if !compact}
      <span class="context-meter__tokens">
        {formatTokens(usage.totalTokens)} / {formatTokens(usage.maxTokens)}
      </span>
    {/if}

    {#if zone === 'critical'}
      <span class="context-meter__warning">REDLINE</span>
    {/if}
  </div>

  {#if !compact}
    <div class="context-meter__breakdown">
      <span class="breakdown-item">
        <span class="breakdown-label">In:</span>
        {formatTokens(usage.inputTokens)}
      </span>
      <span class="breakdown-item">
        <span class="breakdown-label">Out:</span>
        {formatTokens(usage.outputTokens)}
      </span>
    </div>
  {/if}
</div>

<style>
  .context-meter {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .context-meter--compact {
    flex-direction: row;
    align-items: center;
    gap: var(--space-3);
  }

  .context-meter--critical {
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .context-meter__bar {
    position: relative;
    height: 12px;
    background: var(--color-bg-hover);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .context-meter--compact .context-meter__bar {
    flex: 1;
    height: 8px;
  }

  .context-meter__fill {
    height: 100%;
    border-radius: var(--radius-md);
    transition: width var(--duration-300) var(--ease-out);
  }

  .context-meter__threshold {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: rgba(255, 255, 255, 0.3);
  }

  .context-meter__threshold--danger {
    background: rgba(247, 147, 22, 0.5);
  }

  .context-meter__threshold--critical {
    background: rgba(239, 68, 68, 0.5);
  }

  .context-meter__info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .context-meter__percent {
    font-size: var(--text-base);
    font-weight: var(--font-bold);
  }

  .context-meter--compact .context-meter__percent {
    font-size: var(--text-sm);
  }

  .context-meter__tokens {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .context-meter__warning {
    padding: var(--space-1) var(--space-2);
    background: var(--color-error);
    color: white;
    font-size: var(--text-xs);
    font-weight: var(--font-bold);
    border-radius: var(--radius-sm);
    animation: blink 0.5s ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .context-meter__breakdown {
    display: flex;
    gap: var(--space-4);
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  .breakdown-label {
    color: var(--color-text-muted);
    margin-right: var(--space-1);
  }
</style>