# 221 - Backend Selector Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 221
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~11% of Sonnet window

---

## Objective

Create a backend selector component that allows users to choose which AI backend to use for a mission, displaying available backends with their status, capabilities, and pricing information.

---

## Acceptance Criteria

- [ ] List of configured backends with status indicators
- [ ] Backend capability badges (context size, features)
- [ ] Real-time availability checking
- [ ] Pricing/cost estimation display
- [ ] Default backend selection
- [ ] Backend configuration link
- [ ] Keyboard accessible selection

---

## Implementation Details

### 1. Types (src/lib/types/backend.ts)

```typescript
/**
 * Types for backend selection and management.
 */

export interface Backend {
  id: string;
  name: string;
  provider: BackendProvider;
  model: string;
  status: BackendStatus;
  capabilities: BackendCapabilities;
  pricing: BackendPricing;
  isDefault: boolean;
  lastChecked: string;
}

export type BackendProvider =
  | 'anthropic'
  | 'openai'
  | 'google'
  | 'local'
  | 'custom';

export type BackendStatus =
  | 'available'
  | 'degraded'
  | 'unavailable'
  | 'checking'
  | 'unconfigured';

export interface BackendCapabilities {
  maxContextTokens: number;
  maxOutputTokens: number;
  supportsVision: boolean;
  supportsTools: boolean;
  supportsStreaming: boolean;
  supportsJson: boolean;
}

export interface BackendPricing {
  inputCostPer1k: number;
  outputCostPer1k: number;
  currency: string;
}

export interface BackendHealth {
  status: BackendStatus;
  latencyMs: number;
  errorMessage?: string;
  checkedAt: string;
}

export const PROVIDER_ICONS: Record<BackendProvider, string> = {
  anthropic: 'A',
  openai: 'O',
  google: 'G',
  local: 'L',
  custom: 'C',
};

export const PROVIDER_COLORS: Record<BackendProvider, string> = {
  anthropic: '#D97757',
  openai: '#10A37F',
  google: '#4285F4',
  local: '#6B7280',
  custom: '#8B5CF6',
};
```

### 2. Backend Store (src/lib/stores/backend-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { Backend, BackendHealth, BackendStatus } from '$lib/types/backend';
import { ipcRenderer } from '$lib/ipc';

interface BackendStoreState {
  backends: Map<string, Backend>;
  health: Map<string, BackendHealth>;
  loading: boolean;
  error: string | null;
  checkingHealth: Set<string>;
}

function createBackendStore() {
  const initialState: BackendStoreState = {
    backends: new Map(),
    health: new Map(),
    loading: false,
    error: null,
    checkingHealth: new Set(),
  };

  const { subscribe, set, update } = writable<BackendStoreState>(initialState);

  // Listen for backend status updates
  ipcRenderer.on('backend:status', (_event, { backendId, health }: { backendId: string; health: BackendHealth }) => {
    update(state => {
      const healthMap = new Map(state.health);
      healthMap.set(backendId, health);

      const backends = new Map(state.backends);
      const backend = backends.get(backendId);
      if (backend) {
        backends.set(backendId, { ...backend, status: health.status, lastChecked: health.checkedAt });
      }

      const checkingHealth = new Set(state.checkingHealth);
      checkingHealth.delete(backendId);

      return { ...state, health: healthMap, backends, checkingHealth };
    });
  });

  return {
    subscribe,

    async loadBackends(): Promise<void> {
      update(s => ({ ...s, loading: true, error: null }));

      try {
        const backends: Backend[] = await ipcRenderer.invoke('backend:list');
        update(s => ({
          ...s,
          backends: new Map(backends.map(b => [b.id, b])),
          loading: false,
        }));
      } catch (error) {
        update(s => ({
          ...s,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to load backends',
        }));
      }
    },

    async checkHealth(backendId: string): Promise<void> {
      update(s => {
        const checkingHealth = new Set(s.checkingHealth);
        checkingHealth.add(backendId);
        return { ...s, checkingHealth };
      });

      try {
        await ipcRenderer.invoke('backend:check-health', backendId);
      } catch (error) {
        update(s => {
          const checkingHealth = new Set(s.checkingHealth);
          checkingHealth.delete(backendId);

          const healthMap = new Map(s.health);
          healthMap.set(backendId, {
            status: 'unavailable',
            latencyMs: 0,
            errorMessage: error instanceof Error ? error.message : 'Health check failed',
            checkedAt: new Date().toISOString(),
          });

          return { ...s, checkingHealth, health: healthMap };
        });
      }
    },

    async checkAllHealth(): Promise<void> {
      const state = await new Promise<BackendStoreState>(resolve => {
        subscribe(s => resolve(s))();
      });

      for (const backendId of state.backends.keys()) {
        await this.checkHealth(backendId);
      }
    },

    async setDefault(backendId: string): Promise<void> {
      try {
        await ipcRenderer.invoke('backend:set-default', backendId);
        update(s => {
          const backends = new Map(s.backends);
          backends.forEach((backend, id) => {
            backends.set(id, { ...backend, isDefault: id === backendId });
          });
          return { ...s, backends };
        });
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to set default backend',
        }));
      }
    },
  };
}

export const backendStore = createBackendStore();

export const availableBackends = derived(backendStore, $state =>
  Array.from($state.backends.values()).filter(b => b.status === 'available')
);

export const defaultBackend = derived(backendStore, $state =>
  Array.from($state.backends.values()).find(b => b.isDefault)
);

export const backendList = derived(backendStore, $state =>
  Array.from($state.backends.values()).sort((a, b) => {
    // Sort by: default first, then available, then by name
    if (a.isDefault && !b.isDefault) return -1;
    if (!a.isDefault && b.isDefault) return 1;
    if (a.status === 'available' && b.status !== 'available') return -1;
    if (a.status !== 'available' && b.status === 'available') return 1;
    return a.name.localeCompare(b.name);
  })
);
```

### 3. Backend Selector Component (src/lib/components/mission/BackendSelector.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { backendStore, backendList, defaultBackend } from '$lib/stores/backend-store';
  import type { Backend } from '$lib/types/backend';
  import { PROVIDER_ICONS, PROVIDER_COLORS } from '$lib/types/backend';
  import BackendCard from './BackendCard.svelte';

  export let selectedId = '';
  export let disabled = false;

  const dispatch = createEventDispatcher<{
    change: string;
  }>();

  let expandedBackendId: string | null = null;

  // Default to the default backend if none selected
  $: if (!selectedId && $defaultBackend) {
    selectedId = $defaultBackend.id;
    dispatch('change', selectedId);
  }

  function selectBackend(backendId: string) {
    if (disabled) return;

    const backend = $backendStore.backends.get(backendId);
    if (backend && backend.status === 'available') {
      selectedId = backendId;
      dispatch('change', backendId);
    }
  }

  function toggleExpanded(backendId: string) {
    expandedBackendId = expandedBackendId === backendId ? null : backendId;
  }

  function handleKeyDown(event: KeyboardEvent, backend: Backend) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      selectBackend(backend.id);
    }
  }

  onMount(() => {
    backendStore.loadBackends();
  });
</script>

<div class="backend-selector" class:backend-selector--disabled={disabled}>
  {#if $backendStore.loading}
    <div class="backend-selector__loading">
      <div class="loading-spinner"></div>
      <span>Loading backends...</span>
    </div>
  {:else if $backendStore.error}
    <div class="backend-selector__error">
      <span>{$backendStore.error}</span>
      <button on:click={() => backendStore.loadBackends()}>Retry</button>
    </div>
  {:else if $backendList.length === 0}
    <div class="backend-selector__empty">
      <p>No backends configured</p>
      <a href="/settings/backends">Configure backends</a>
    </div>
  {:else}
    <div class="backend-list" role="radiogroup" aria-label="Select backend">
      {#each $backendList as backend}
        <BackendCard
          {backend}
          selected={selectedId === backend.id}
          expanded={expandedBackendId === backend.id}
          checking={$backendStore.checkingHealth.has(backend.id)}
          health={$backendStore.health.get(backend.id)}
          on:select={() => selectBackend(backend.id)}
          on:toggle={() => toggleExpanded(backend.id)}
          on:checkHealth={() => backendStore.checkHealth(backend.id)}
          on:keydown={(e) => handleKeyDown(e.detail, backend)}
        />
      {/each}
    </div>

    <div class="backend-selector__footer">
      <button
        class="refresh-btn"
        on:click={() => backendStore.checkAllHealth()}
        disabled={$backendStore.checkingHealth.size > 0}
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M7 1a6 6 0 00-6 6h1.5A4.5 4.5 0 017 2.5V1zm0 12a6 6 0 006-6h-1.5A4.5 4.5 0 017 11.5V13z"/>
        </svg>
        Refresh All
      </button>
      <a href="/settings/backends" class="config-link">
        Configure Backends
      </a>
    </div>
  {/if}
</div>

<style>
  .backend-selector {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .backend-selector--disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .backend-selector__loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 32px;
    color: var(--color-text-secondary);
  }

  .loading-spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .backend-selector__error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 24px;
    color: var(--color-error);
    text-align: center;
  }

  .backend-selector__error button {
    padding: 8px 16px;
    border: 1px solid var(--color-error);
    background: transparent;
    color: var(--color-error);
    border-radius: 4px;
    cursor: pointer;
  }

  .backend-selector__empty {
    text-align: center;
    padding: 32px;
    color: var(--color-text-secondary);
  }

  .backend-selector__empty a {
    color: var(--color-primary);
    text-decoration: none;
  }

  .backend-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .backend-selector__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border: none;
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
    font-size: 13px;
    border-radius: 4px;
    cursor: pointer;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--color-bg-active);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .config-link {
    font-size: 13px;
    color: var(--color-primary);
    text-decoration: none;
  }

  .config-link:hover {
    text-decoration: underline;
  }
</style>
```

### 4. Backend Card Component (src/lib/components/mission/BackendCard.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Backend, BackendHealth } from '$lib/types/backend';
  import { PROVIDER_ICONS, PROVIDER_COLORS } from '$lib/types/backend';

  export let backend: Backend;
  export let selected = false;
  export let expanded = false;
  export let checking = false;
  export let health: BackendHealth | undefined;

  const dispatch = createEventDispatcher<{
    select: void;
    toggle: void;
    checkHealth: void;
    keydown: KeyboardEvent;
  }>();

  const statusLabels = {
    available: 'Available',
    degraded: 'Degraded',
    unavailable: 'Unavailable',
    checking: 'Checking...',
    unconfigured: 'Not Configured',
  };

  const statusColors = {
    available: 'var(--color-success)',
    degraded: 'var(--color-warning)',
    unavailable: 'var(--color-error)',
    checking: 'var(--color-text-muted)',
    unconfigured: 'var(--color-text-muted)',
  };

  function formatTokens(tokens: number): string {
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(1)}M`;
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(0)}k`;
    return tokens.toString();
  }

  function formatCost(cost: number): string {
    return `$${cost.toFixed(4)}`;
  }
</script>

<div
  class="backend-card"
  class:backend-card--selected={selected}
  class:backend-card--unavailable={backend.status !== 'available'}
  role="radio"
  aria-checked={selected}
  tabindex="0"
  on:click={() => dispatch('select')}
  on:keydown={(e) => dispatch('keydown', e)}
>
  <!-- Provider Icon -->
  <div
    class="backend-card__icon"
    style="background-color: {PROVIDER_COLORS[backend.provider]}"
  >
    {PROVIDER_ICONS[backend.provider]}
  </div>

  <!-- Main Info -->
  <div class="backend-card__content">
    <div class="backend-card__header">
      <h4 class="backend-card__name">
        {backend.name}
        {#if backend.isDefault}
          <span class="default-badge">Default</span>
        {/if}
      </h4>
      <span
        class="backend-card__status"
        style="color: {statusColors[checking ? 'checking' : backend.status]}"
      >
        {#if checking}
          <span class="status-spinner"></span>
        {/if}
        {statusLabels[checking ? 'checking' : backend.status]}
      </span>
    </div>

    <div class="backend-card__model">{backend.model}</div>

    <div class="backend-card__capabilities">
      <span class="capability" title="Max context tokens">
        {formatTokens(backend.capabilities.maxContextTokens)} ctx
      </span>
      {#if backend.capabilities.supportsVision}
        <span class="capability capability--feature" title="Supports vision">
          Vision
        </span>
      {/if}
      {#if backend.capabilities.supportsTools}
        <span class="capability capability--feature" title="Supports tools">
          Tools
        </span>
      {/if}
    </div>
  </div>

  <!-- Expand Button -->
  <button
    class="backend-card__expand"
    on:click|stopPropagation={() => dispatch('toggle')}
    aria-expanded={expanded}
    aria-label="Show details"
  >
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="currentColor"
      class:rotated={expanded}
    >
      <path d="M4 6l4 4 4-4"/>
    </svg>
  </button>

  <!-- Selected Indicator -->
  {#if selected}
    <div class="backend-card__selected-indicator">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z"/>
      </svg>
    </div>
  {/if}
</div>

<!-- Expanded Details -->
{#if expanded}
  <div class="backend-details">
    <div class="backend-details__section">
      <h5>Pricing</h5>
      <div class="pricing-row">
        <span>Input:</span>
        <span>{formatCost(backend.pricing.inputCostPer1k)}/1k tokens</span>
      </div>
      <div class="pricing-row">
        <span>Output:</span>
        <span>{formatCost(backend.pricing.outputCostPer1k)}/1k tokens</span>
      </div>
    </div>

    <div class="backend-details__section">
      <h5>Capabilities</h5>
      <ul class="capabilities-list">
        <li>Max context: {formatTokens(backend.capabilities.maxContextTokens)}</li>
        <li>Max output: {formatTokens(backend.capabilities.maxOutputTokens)}</li>
        <li>Vision: {backend.capabilities.supportsVision ? 'Yes' : 'No'}</li>
        <li>Tools: {backend.capabilities.supportsTools ? 'Yes' : 'No'}</li>
        <li>Streaming: {backend.capabilities.supportsStreaming ? 'Yes' : 'No'}</li>
      </ul>
    </div>

    {#if health}
      <div class="backend-details__section">
        <h5>Health</h5>
        <div class="health-info">
          <span>Latency: {health.latencyMs}ms</span>
          <span>Checked: {new Date(health.checkedAt).toLocaleTimeString()}</span>
          {#if health.errorMessage}
            <span class="health-error">{health.errorMessage}</span>
          {/if}
        </div>
      </div>
    {/if}

    <button
      class="check-health-btn"
      on:click|stopPropagation={() => dispatch('checkHealth')}
      disabled={checking}
    >
      {checking ? 'Checking...' : 'Check Health'}
    </button>
  </div>
{/if}

<style>
  .backend-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: var(--color-bg-primary);
    cursor: pointer;
    transition: all 0.15s ease;
    position: relative;
  }

  .backend-card:hover {
    border-color: var(--color-primary);
  }

  .backend-card:focus {
    outline: none;
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .backend-card--selected {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .backend-card--unavailable {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .backend-card__icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    color: white;
    font-weight: 700;
    font-size: 18px;
    flex-shrink: 0;
  }

  .backend-card__content {
    flex: 1;
    min-width: 0;
  }

  .backend-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  .backend-card__name {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .default-badge {
    padding: 2px 6px;
    background: var(--color-primary);
    color: white;
    font-size: 10px;
    font-weight: 500;
    border-radius: 4px;
    text-transform: uppercase;
  }

  .backend-card__status {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
  }

  .status-spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .backend-card__model {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin-bottom: 8px;
  }

  .backend-card__capabilities {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .capability {
    padding: 2px 6px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    font-size: 11px;
    color: var(--color-text-secondary);
  }

  .capability--feature {
    background: rgba(33, 150, 243, 0.1);
    color: var(--color-primary);
  }

  .backend-card__expand {
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 4px;
  }

  .backend-card__expand:hover {
    background: var(--color-bg-hover);
  }

  .backend-card__expand svg {
    transition: transform 0.15s ease;
  }

  .backend-card__expand svg.rotated {
    transform: rotate(180deg);
  }

  .backend-card__selected-indicator {
    position: absolute;
    top: 8px;
    right: 8px;
    color: var(--color-primary);
  }

  .backend-details {
    padding: 16px;
    margin-top: -1px;
    border: 1px solid var(--color-border);
    border-top: none;
    border-radius: 0 0 8px 8px;
    background: var(--color-bg-secondary);
  }

  .backend-details__section {
    margin-bottom: 16px;
  }

  .backend-details__section h5 {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 8px 0;
    text-transform: uppercase;
  }

  .pricing-row {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    padding: 4px 0;
  }

  .capabilities-list {
    list-style: none;
    padding: 0;
    margin: 0;
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .capabilities-list li {
    padding: 4px 0;
  }

  .health-info {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .health-info span {
    display: block;
    padding: 2px 0;
  }

  .health-error {
    color: var(--color-error);
  }

  .check-health-btn {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 13px;
    border-radius: 4px;
    cursor: pointer;
  }

  .check-health-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .check-health-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

---

## Testing Requirements

1. Backends load correctly from IPC
2. Selection emits change event
3. Unavailable backends cannot be selected
4. Health check updates status
5. Default backend is pre-selected
6. Keyboard navigation works
7. Expanded details show correctly

### Test File (src/lib/components/mission/__tests__/BackendSelector.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import BackendSelector from '../BackendSelector.svelte';
import { backendStore } from '$lib/stores/backend-store';

vi.mock('$lib/ipc', () => ({
  ipcRenderer: {
    invoke: vi.fn().mockResolvedValue([
      {
        id: 'claude-sonnet',
        name: 'Claude Sonnet',
        provider: 'anthropic',
        model: 'claude-3-sonnet',
        status: 'available',
        isDefault: true,
        capabilities: { maxContextTokens: 200000, supportsVision: true, supportsTools: true },
        pricing: { inputCostPer1k: 0.003, outputCostPer1k: 0.015, currency: 'USD' },
      },
      {
        id: 'gpt-4',
        name: 'GPT-4',
        provider: 'openai',
        model: 'gpt-4-turbo',
        status: 'available',
        isDefault: false,
        capabilities: { maxContextTokens: 128000, supportsVision: true, supportsTools: true },
        pricing: { inputCostPer1k: 0.01, outputCostPer1k: 0.03, currency: 'USD' },
      },
    ]),
    on: vi.fn(),
  },
}));

describe('BackendSelector', () => {
  beforeEach(async () => {
    await backendStore.loadBackends();
  });

  it('renders available backends', async () => {
    render(BackendSelector);

    expect(screen.getByText('Claude Sonnet')).toBeInTheDocument();
    expect(screen.getByText('GPT-4')).toBeInTheDocument();
  });

  it('marks default backend', () => {
    render(BackendSelector);

    expect(screen.getByText('Default')).toBeInTheDocument();
  });

  it('emits change on selection', async () => {
    const { component } = render(BackendSelector);
    const handler = vi.fn();
    component.$on('change', handler);

    const gpt4Card = screen.getByText('GPT-4').closest('[role="radio"]');
    await fireEvent.click(gpt4Card!);

    expect(handler).toHaveBeenCalledWith(expect.objectContaining({ detail: 'gpt-4' }));
  });

  it('shows expanded details on toggle', async () => {
    render(BackendSelector);

    const expandBtn = screen.getAllByLabelText('Show details')[0];
    await fireEvent.click(expandBtn);

    expect(screen.getByText('Pricing')).toBeInTheDocument();
    expect(screen.getByText('Capabilities')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [222-mode-toggle.md](222-mode-toggle.md)
- Used by: [218-mission-creation.md](218-mission-creation.md)
