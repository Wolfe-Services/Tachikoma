# 304 - Context Visualization

**Phase:** 14 - Dashboard
**Spec ID:** 304
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 301-token-charts
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create context window visualization components that display token usage within the context window, showing utilization levels, token composition, and warning indicators for approaching limits.

---

## Acceptance Criteria

- [ ] `ContextWindow.svelte` component created
- [ ] Visual representation of context capacity
- [ ] Token breakdown by category (system, user, assistant)
- [ ] Redline threshold warnings
- [ ] Real-time usage updates
- [ ] Historical context usage patterns
- [ ] Tooltip with detailed breakdown
- [ ] Animation on usage changes

---

## Implementation Details

### 1. Context Window Component (web/src/lib/components/context/ContextWindow.svelte)

```svelte
<script lang="ts">
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { fade, fly } from 'svelte/transition';
  import type { ContextUsage } from '$lib/types/context';
  import Icon from '$lib/components/common/Icon.svelte';

  export let usage: ContextUsage;
  export let maxTokens: number = 200000;
  export let redlineThreshold: number = 0.85;
  export let warningThreshold: number = 0.7;
  export let showBreakdown: boolean = true;
  export let showHistory: boolean = false;

  let showTooltip = false;
  let tooltipPosition = { x: 0, y: 0 };

  $: totalUsed = usage.system + usage.user + usage.assistant + usage.tools;
  $: usagePercent = (totalUsed / maxTokens) * 100;
  $: remaining = maxTokens - totalUsed;

  $: status = getStatus(usagePercent);

  const animatedUsage = tweened(0, {
    duration: 500,
    easing: cubicOut
  });

  $: animatedUsage.set(usagePercent);

  function getStatus(percent: number): {
    level: 'normal' | 'warning' | 'critical';
    color: string;
    message: string;
  } {
    if (percent >= redlineThreshold * 100) {
      return {
        level: 'critical',
        color: 'var(--red-500)',
        message: 'Context window critical - consider rebooting'
      };
    } else if (percent >= warningThreshold * 100) {
      return {
        level: 'warning',
        color: 'var(--yellow-500)',
        message: 'Approaching context limit'
      };
    }
    return {
      level: 'normal',
      color: 'var(--green-500)',
      message: 'Context usage normal'
    };
  }

  function formatTokens(n: number): string {
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  }

  function handleMouseMove(event: MouseEvent) {
    tooltipPosition = { x: event.clientX, y: event.clientY };
  }

  const segments = [
    { key: 'system', label: 'System', color: 'var(--blue-500)' },
    { key: 'user', label: 'User', color: 'var(--green-500)' },
    { key: 'assistant', label: 'Assistant', color: 'var(--purple-500)' },
    { key: 'tools', label: 'Tools', color: 'var(--orange-500)' }
  ];
</script>

<div class="context-window" class:warning={status.level === 'warning'} class:critical={status.level === 'critical'}>
  <div class="header">
    <div class="title">
      <Icon name="layers" size={18} />
      <h3>Context Window</h3>
    </div>
    <span class="status-indicator" style="color: {status.color}">
      <Icon name={status.level === 'critical' ? 'alert-triangle' : status.level === 'warning' ? 'alert-circle' : 'check-circle'} size={16} />
      {status.message}
    </span>
  </div>

  <div class="usage-display">
    <div class="usage-text">
      <span class="current">{formatTokens(totalUsed)}</span>
      <span class="separator">/</span>
      <span class="max">{formatTokens(maxTokens)}</span>
      <span class="label">tokens</span>
    </div>
    <span class="percent" style="color: {status.color}">
      {$animatedUsage.toFixed(1)}%
    </span>
  </div>

  <div
    class="usage-bar"
    on:mouseenter={() => showTooltip = true}
    on:mouseleave={() => showTooltip = false}
    on:mousemove={handleMouseMove}
    role="progressbar"
    aria-valuenow={totalUsed}
    aria-valuemin={0}
    aria-valuemax={maxTokens}
  >
    <div class="bar-background">
      <!-- Warning threshold marker -->
      <div
        class="threshold-marker warning"
        style="left: {warningThreshold * 100}%"
      />
      <!-- Redline threshold marker -->
      <div
        class="threshold-marker critical"
        style="left: {redlineThreshold * 100}%"
      />

      <!-- Stacked usage segments -->
      <div class="bar-segments">
        {#each segments as segment, i}
          {@const value = usage[segment.key as keyof ContextUsage]}
          {@const prevTotal = segments.slice(0, i).reduce((sum, s) => sum + (usage[s.key as keyof ContextUsage] || 0), 0)}
          {#if value > 0}
            <div
              class="bar-segment"
              style="
                left: {(prevTotal / maxTokens) * 100}%;
                width: {(value / maxTokens) * 100}%;
                background: {segment.color};
              "
              title="{segment.label}: {formatTokens(value)}"
            />
          {/if}
        {/each}
      </div>
    </div>
  </div>

  {#if showBreakdown}
    <div class="breakdown">
      {#each segments as segment}
        {@const value = usage[segment.key as keyof ContextUsage]}
        {#if value > 0}
          <div class="breakdown-item">
            <span class="item-color" style="background: {segment.color}" />
            <span class="item-label">{segment.label}</span>
            <span class="item-value">{formatTokens(value)}</span>
            <span class="item-percent">{((value / totalUsed) * 100).toFixed(1)}%</span>
          </div>
        {/if}
      {/each}
    </div>
  {/if}

  <div class="footer">
    <span class="remaining">
      <Icon name="battery" size={14} />
      {formatTokens(remaining)} remaining
    </span>
    {#if usage.estimatedCost}
      <span class="cost">
        Est. cost: ${usage.estimatedCost.toFixed(4)}
      </span>
    {/if}
  </div>

  {#if showTooltip}
    <div
      class="tooltip"
      style="left: {tooltipPosition.x}px; top: {tooltipPosition.y}px;"
      transition:fade={{ duration: 100 }}
    >
      <div class="tooltip-header">Context Breakdown</div>
      <div class="tooltip-content">
        {#each segments as segment}
          {@const value = usage[segment.key as keyof ContextUsage]}
          <div class="tooltip-row">
            <span class="tooltip-color" style="background: {segment.color}" />
            <span class="tooltip-label">{segment.label}</span>
            <span class="tooltip-value">{formatTokens(value)}</span>
          </div>
        {/each}
        <div class="tooltip-divider" />
        <div class="tooltip-row total">
          <span class="tooltip-label">Total</span>
          <span class="tooltip-value">{formatTokens(totalUsed)}</span>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .context-window {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .context-window.warning {
    border-color: var(--yellow-300);
  }

  .context-window.critical {
    border-color: var(--red-300);
    animation: pulse-border 2s ease-in-out infinite;
  }

  @keyframes pulse-border {
    0%, 100% { border-color: var(--red-300); }
    50% { border-color: var(--red-500); }
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .title h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .usage-display {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 1rem 1.25rem 0.5rem;
  }

  .usage-text {
    display: flex;
    align-items: baseline;
    gap: 0.25rem;
  }

  .current {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .separator {
    color: var(--text-tertiary);
  }

  .max {
    font-size: 1rem;
    color: var(--text-secondary);
  }

  .label {
    margin-left: 0.25rem;
    font-size: 0.875rem;
    color: var(--text-tertiary);
  }

  .percent {
    font-size: 1.25rem;
    font-weight: 600;
  }

  .usage-bar {
    position: relative;
    margin: 0 1.25rem 1rem;
    cursor: pointer;
  }

  .bar-background {
    position: relative;
    height: 1rem;
    background: var(--bg-secondary);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .threshold-marker {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    z-index: 2;
  }

  .threshold-marker.warning {
    background: var(--yellow-400);
    opacity: 0.5;
  }

  .threshold-marker.critical {
    background: var(--red-400);
  }

  .bar-segments {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }

  .bar-segment {
    position: absolute;
    top: 0;
    bottom: 0;
    transition: width 0.3s ease;
  }

  .bar-segment:first-child {
    border-radius: 0.5rem 0 0 0.5rem;
  }

  .bar-segment:last-child {
    border-radius: 0 0.5rem 0.5rem 0;
  }

  .breakdown {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
    padding: 0 1.25rem 1rem;
  }

  .breakdown-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 0.375rem;
  }

  .item-color {
    width: 0.625rem;
    height: 0.625rem;
    border-radius: 0.125rem;
    flex-shrink: 0;
  }

  .item-label {
    flex: 1;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .item-value {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .item-percent {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
    width: 2.5rem;
    text-align: right;
  }

  .footer {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem 1.25rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 0.75rem;
  }

  .remaining {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    color: var(--text-secondary);
  }

  .cost {
    color: var(--text-tertiary);
  }

  .tooltip {
    position: fixed;
    transform: translate(-50%, -100%);
    margin-top: -10px;
    padding: 0.75rem;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    pointer-events: none;
    z-index: 1000;
    min-width: 180px;
  }

  .tooltip-header {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.5rem;
  }

  .tooltip-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0;
  }

  .tooltip-color {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 0.125rem;
  }

  .tooltip-label {
    flex: 1;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .tooltip-value {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .tooltip-divider {
    height: 1px;
    background: var(--border-color);
    margin: 0.375rem 0;
  }

  .tooltip-row.total {
    font-weight: 500;
  }
</style>
```

### 2. Context Usage Types (web/src/lib/types/context.ts)

```typescript
export interface ContextUsage {
  system: number;
  user: number;
  assistant: number;
  tools: number;
  estimatedCost?: number;
}

export interface ContextHistory {
  timestamp: string;
  usage: ContextUsage;
  total: number;
}

export interface ContextConfig {
  maxTokens: number;
  redlineThreshold: number;
  warningThreshold: number;
  autoReboot: boolean;
}
```

### 3. Context Store (web/src/lib/stores/context.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { ContextUsage, ContextHistory, ContextConfig } from '$lib/types/context';

interface ContextState {
  current: ContextUsage;
  config: ContextConfig;
  history: ContextHistory[];
}

const defaultConfig: ContextConfig = {
  maxTokens: 200000,
  redlineThreshold: 0.85,
  warningThreshold: 0.7,
  autoReboot: false
};

function createContextStore() {
  const { subscribe, set, update } = writable<ContextState>({
    current: { system: 0, user: 0, assistant: 0, tools: 0 },
    config: defaultConfig,
    history: []
  });

  return {
    subscribe,
    updateUsage: (usage: ContextUsage) =>
      update(s => ({
        ...s,
        current: usage,
        history: [...s.history.slice(-99), { timestamp: new Date().toISOString(), usage, total: usage.system + usage.user + usage.assistant + usage.tools }]
      })),
    setConfig: (config: Partial<ContextConfig>) =>
      update(s => ({ ...s, config: { ...s.config, ...config } })),
    reset: () => set({
      current: { system: 0, user: 0, assistant: 0, tools: 0 },
      config: defaultConfig,
      history: []
    })
  };
}

export const contextStore = createContextStore();

export const contextUsage = derived(
  contextStore,
  $store => $store.current
);

export const isNearRedline = derived(
  contextStore,
  $store => {
    const total = $store.current.system + $store.current.user + $store.current.assistant + $store.current.tools;
    return total / $store.config.maxTokens >= $store.config.warningThreshold;
  }
);
```

---

## Testing Requirements

1. Usage bar accurately reflects token counts
2. Threshold markers position correctly
3. Warning/critical states trigger at thresholds
4. Tooltip shows correct breakdown
5. Animation on usage changes is smooth
6. Stacked segments sum correctly
7. Responsive layout works

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [305-test-charts.md](305-test-charts.md)
- Used by: Mission view, status panels
