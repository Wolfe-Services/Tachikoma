# 230 - Context Meter Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 230
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a context meter component that visualizes the current context window usage, showing token consumption with color-coded zones and warnings as usage approaches limits.

---

## Acceptance Criteria

- [x] Visual meter showing context usage percentage
- [x] Color zones (green/yellow/orange/red)
- [x] Token count display (used/max)
- [x] Animated transitions on updates
- [x] Redline warning threshold indicator
- [x] Tooltip with detailed breakdown

---

## Implementation Details

### 1. Types (src/lib/types/context.ts)

```typescript
export interface ContextUsage {
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  maxTokens: number;
  usagePercent: number;
  zone: ContextZone;
}

export type ContextZone = 'safe' | 'warning' | 'danger' | 'critical';

export const CONTEXT_THRESHOLDS = {
  warning: 60,
  danger: 80,
  critical: 95,
};

export function getContextZone(percent: number): ContextZone {
  if (percent >= CONTEXT_THRESHOLDS.critical) return 'critical';
  if (percent >= CONTEXT_THRESHOLDS.danger) return 'danger';
  if (percent >= CONTEXT_THRESHOLDS.warning) return 'warning';
  return 'safe';
}
```

### 2. Context Meter Component (src/lib/components/mission/ContextMeter.svelte)

```svelte
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
    danger: '#ff6b35',
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
    gap: 8px;
  }

  .context-meter--compact {
    flex-direction: row;
    align-items: center;
    gap: 12px;
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
    border-radius: 6px;
    overflow: hidden;
  }

  .context-meter--compact .context-meter__bar {
    flex: 1;
    height: 8px;
  }

  .context-meter__fill {
    height: 100%;
    border-radius: 6px;
    transition: width 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .context-meter__threshold {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: rgba(255, 255, 255, 0.3);
  }

  .context-meter__threshold--danger {
    background: rgba(255, 107, 53, 0.5);
  }

  .context-meter__threshold--critical {
    background: rgba(244, 67, 54, 0.5);
  }

  .context-meter__info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .context-meter__percent {
    font-size: 16px;
    font-weight: 700;
  }

  .context-meter--compact .context-meter__percent {
    font-size: 14px;
  }

  .context-meter__tokens {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .context-meter__warning {
    padding: 2px 6px;
    background: var(--color-error);
    color: white;
    font-size: 10px;
    font-weight: 700;
    border-radius: 4px;
    animation: blink 0.5s ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .context-meter__breakdown {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .breakdown-label {
    color: var(--color-text-muted);
    margin-right: 4px;
  }
</style>
```

---

## Testing Requirements

1. Meter displays correct percentage
2. Color zones change at thresholds
3. Animation works smoothly
4. Redline warning shows at critical
5. Token counts format correctly

---

## Related Specs

- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [231-redline-warning.md](231-redline-warning.md)
