# Spec 279: Think Tank Selection

## Header
- **Spec ID**: 279
- **Phase**: 13 - Settings UI
- **Component**: Think Tank Selection
- **Dependencies**: Spec 278 (Brain Selection)
- **Status**: Draft

## Objective
Create a settings interface for configuring default think tanks (oracle + participant combinations), managing saved configurations, and setting up quick-access deliberation teams.

## Acceptance Criteria
- [x] Configure default think tank compositions
- [x] Save and name custom think tank configurations
- [x] Set default oracle for deliberations
- [x] Quick-select from saved think tanks
- [x] Import/export think tank configurations
- [x] Display estimated costs per configuration
- [x] Validate think tank compositions
- [x] Support think tank templates

## Implementation

### ThinkTankSelection.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, fly } from 'svelte/transition';
  import ThinkTankCard from './ThinkTankCard.svelte';
  import ThinkTankEditor from './ThinkTankEditor.svelte';
  import ThinkTankTemplates from './ThinkTankTemplates.svelte';
  import OracleSelector from './OracleSelector.svelte';
  import ParticipantSelector from './ParticipantSelector.svelte';
  import { thinkTankStore } from '$lib/stores/thinkTank';
  import { brainsStore } from '$lib/stores/brains';
  import type { ThinkTank, Oracle, Participant, ThinkTankTemplate } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    change: ThinkTank;
    create: ThinkTank;
    delete: { id: string };
  }>();

  let showEditor = writable<boolean>(false);
  let showTemplates = writable<boolean>(false);
  let editingTankId = writable<string | null>(null);
  let selectedTankId = writable<string | null>(null);

  const thinkTanks = derived(thinkTankStore, ($store) => $store.tanks);

  const defaultTankId = derived(thinkTankStore, ($store) => $store.defaultTankId);

  const defaultOracle = derived(thinkTankStore, ($store) => $store.defaultOracleId);

  const selectedTank = derived(
    [thinkTanks, selectedTankId],
    ([$tanks, $id]) => $tanks.find(t => t.id === $id) || null
  );

  const brains = derived(brainsStore, ($store) => $store.available);

  const totalCosts = derived(thinkTanks, ($tanks) => {
    const costs = new Map<string, number>();

    for (const tank of $tanks) {
      let cost = tank.oracle?.estimatedCostPerRound || 0;
      cost += tank.participants.reduce((sum, p) => sum + (p.estimatedCostPerRound || 0), 0);
      costs.set(tank.id, cost);
    }

    return costs;
  });

  function createNewTank() {
    editingTankId.set(null);
    showEditor.set(true);
  }

  function editTank(tankId: string) {
    editingTankId.set(tankId);
    showEditor.set(true);
  }

  function saveTank(tank: ThinkTank) {
    if ($editingTankId) {
      thinkTankStore.update(tank);
    } else {
      thinkTankStore.add(tank);
      dispatch('create', tank);
    }
    showEditor.set(false);
    editingTankId.set(null);
  }

  function deleteTank(tankId: string) {
    if (confirm('Are you sure you want to delete this think tank?')) {
      thinkTankStore.remove(tankId);
      dispatch('delete', { id: tankId });

      if ($selectedTankId === tankId) {
        selectedTankId.set(null);
      }
    }
  }

  function setAsDefault(tankId: string) {
    thinkTankStore.setDefault(tankId);
    dispatch('change', $thinkTanks.find(t => t.id === tankId)!);
  }

  function duplicateTank(tankId: string) {
    const tank = $thinkTanks.find(t => t.id === tankId);
    if (tank) {
      const duplicate: ThinkTank = {
        ...tank,
        id: crypto.randomUUID(),
        name: `${tank.name} (Copy)`,
        createdAt: new Date()
      };
      thinkTankStore.add(duplicate);
    }
  }

  function applyTemplate(template: ThinkTankTemplate) {
    const tank: ThinkTank = {
      id: crypto.randomUUID(),
      name: template.name,
      description: template.description,
      oracle: template.oracle,
      participants: template.participants,
      config: template.config,
      createdAt: new Date()
    };

    thinkTankStore.add(tank);
    showTemplates.set(false);
  }

  async function exportTank(tankId: string) {
    const tank = $thinkTanks.find(t => t.id === tankId);
    if (tank) {
      const data = JSON.stringify(tank, null, 2);
      const blob = new Blob([data], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `think-tank-${tank.name.toLowerCase().replace(/\s+/g, '-')}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  }

  function importTank() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const text = await file.text();
        const tank = JSON.parse(text);
        tank.id = crypto.randomUUID();
        tank.createdAt = new Date();
        thinkTankStore.add(tank);
      }
    };
    input.click();
  }

  onMount(() => {
    thinkTankStore.load();
    brainsStore.loadAvailable();
  });
</script>

<div class="think-tank-selection" data-testid="think-tank-selection">
  <header class="selection-header">
    <div class="header-title">
      <h2>Think Tank Configuration</h2>
      <p class="description">Configure default deliberation teams and oracle settings</p>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={() => showTemplates.set(true)}>
        Templates
      </button>
      <button class="btn secondary" on:click={importTank}>
        Import
      </button>
      <button class="btn primary" on:click={createNewTank}>
        Create Think Tank
      </button>
    </div>
  </header>

  <section class="defaults-section">
    <h3>Default Oracle</h3>
    <OracleSelector
      selectedId={$defaultOracle}
      brains={$brains.filter(b => b.capabilities.includes('oracle'))}
      on:change={(e) => thinkTankStore.setDefaultOracle(e.detail)}
    />
  </section>

  <section class="tanks-section">
    <div class="section-header">
      <h3>Saved Think Tanks</h3>
      <span class="tank-count">{$thinkTanks.length} configurations</span>
    </div>

    {#if $thinkTanks.length === 0}
      <div class="empty-state">
        <p>No think tanks configured yet</p>
        <p class="hint">Create a think tank to save your favorite deliberation team</p>
        <button class="btn primary" on:click={createNewTank}>
          Create First Think Tank
        </button>
      </div>
    {:else}
      <div class="tanks-grid">
        {#each $thinkTanks as tank (tank.id)}
          <ThinkTankCard
            {tank}
            isDefault={$defaultTankId === tank.id}
            estimatedCost={$totalCosts.get(tank.id) || 0}
            on:click={() => selectedTankId.set(tank.id)}
            on:edit={() => editTank(tank.id)}
            on:delete={() => deleteTank(tank.id)}
            on:setDefault={() => setAsDefault(tank.id)}
            on:duplicate={() => duplicateTank(tank.id)}
            on:export={() => exportTank(tank.id)}
          />
        {/each}
      </div>
    {/if}
  </section>

  {#if $selectedTank}
    <aside class="tank-preview" transition:fly={{ x: 300, duration: 200 }}>
      <div class="preview-header">
        <h3>{$selectedTank.name}</h3>
        <button
          class="close-btn"
          on:click={() => selectedTankId.set(null)}
          aria-label="Close preview"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M18 6L6 18M6 6l12 12" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="preview-content">
        {#if $selectedTank.description}
          <p class="tank-description">{$selectedTank.description}</p>
        {/if}

        <div class="preview-section">
          <h4>Oracle</h4>
          {#if $selectedTank.oracle}
            <div class="oracle-info">
              <span class="name">{$selectedTank.oracle.name}</span>
              <span class="provider">{$selectedTank.oracle.provider}</span>
            </div>
          {:else}
            <span class="not-set">Default oracle will be used</span>
          {/if}
        </div>

        <div class="preview-section">
          <h4>Participants ({$selectedTank.participants.length})</h4>
          <div class="participant-list">
            {#each $selectedTank.participants as participant}
              <div class="participant-item">
                <span class="name">{participant.name}</span>
                <span class="role">{participant.role || 'Participant'}</span>
              </div>
            {/each}
          </div>
        </div>

        <div class="preview-section">
          <h4>Estimated Cost</h4>
          <span class="cost-value">
            ~${($totalCosts.get($selectedTank.id) || 0).toFixed(4)} / round
          </span>
        </div>

        <div class="preview-actions">
          <button class="btn secondary" on:click={() => editTank($selectedTank.id)}>
            Edit
          </button>
          {#if $defaultTankId !== $selectedTank.id}
            <button class="btn primary" on:click={() => setAsDefault($selectedTank.id)}>
              Set as Default
            </button>
          {/if}
        </div>
      </div>
    </aside>
  {/if}

  {#if $showEditor}
    <div class="modal-overlay" transition:fade on:click={() => showEditor.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <ThinkTankEditor
          tank={$editingTankId ? $thinkTanks.find(t => t.id === $editingTankId) : null}
          brains={$brains}
          on:save={(e) => saveTank(e.detail)}
          on:close={() => {
            showEditor.set(false);
            editingTankId.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showTemplates}
    <div class="modal-overlay" transition:fade on:click={() => showTemplates.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ThinkTankTemplates
          on:select={(e) => applyTemplate(e.detail)}
          on:close={() => showTemplates.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .think-tank-selection {
    max-width: 1200px;
  }

  .selection-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 2rem;
  }

  .header-title h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .header-actions {
    display: flex;
    gap: 0.75rem;
  }

  .defaults-section {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1.5rem;
  }

  .defaults-section h3 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .tanks-section {
    margin-bottom: 1.5rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .section-header h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .tank-count {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .tanks-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }

  .empty-state {
    background: var(--card-bg);
    border: 1px dashed var(--border-color);
    border-radius: 8px;
    padding: 3rem;
    text-align: center;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
    margin-bottom: 1.5rem;
  }

  .tank-preview {
    position: fixed;
    right: 0;
    top: 0;
    bottom: 0;
    width: 400px;
    background: var(--card-bg);
    border-left: 1px solid var(--border-color);
    z-index: 100;
    overflow-y: auto;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .preview-header h3 {
    font-size: 1.125rem;
    font-weight: 600;
  }

  .close-btn {
    padding: 0.25rem;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
  }

  .preview-content {
    padding: 1.25rem;
  }

  .tank-description {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin-bottom: 1.5rem;
    line-height: 1.5;
  }

  .preview-section {
    margin-bottom: 1.5rem;
  }

  .preview-section h4 {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-muted);
    margin-bottom: 0.75rem;
  }

  .oracle-info,
  .participant-item {
    display: flex;
    flex-direction: column;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  .name {
    font-weight: 500;
  }

  .provider,
  .role {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .not-set {
    color: var(--text-muted);
    font-style: italic;
  }

  .cost-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--primary-color);
  }

  .preview-actions {
    display: flex;
    gap: 0.75rem;
    margin-top: 2rem;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn.primary {
    background: var(--primary-color);
    color: white;
  }

  .btn.secondary {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--card-bg);
    border-radius: 8px;
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .modal-content.large {
    max-width: 900px;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test tank CRUD operations
2. **Integration Tests**: Verify persistence and sync
3. **Template Tests**: Test template application
4. **Cost Tests**: Validate cost calculations
5. **Import/Export Tests**: Test portability

## Related Specs
- Spec 278: Brain Selection
- Spec 260: Oracle Select (Forge UI)
- Spec 259: Participant Select (Forge UI)
