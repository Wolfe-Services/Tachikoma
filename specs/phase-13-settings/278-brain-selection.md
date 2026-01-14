# Spec 278: Brain Selection

## Header
- **Spec ID**: 278
- **Phase**: 13 - Settings UI
- **Component**: Brain Selection
- **Dependencies**: Spec 276 (Settings Layout)
- **Status**: Draft

## Objective
Create a settings interface for configuring default brains, managing brain preferences, setting up brain-specific parameters, and organizing brain collections for different use cases.

## Acceptance Criteria
- [x] Browse and search available brains with detailed info
- [x] Set default brain for chat and forge sessions
- [x] Configure brain-specific parameters (temperature, tokens)
- [x] Create brain presets for different tasks
- [x] Manage brain favorites and recent history
- [x] Display brain capabilities and costs
- [x] Test brain responses with sample prompts
- [x] Sync brain preferences across devices

## Implementation

### BrainSelection.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import BrainCard from './BrainCard.svelte';
  import BrainDetails from './BrainDetails.svelte';
  import BrainConfig from './BrainConfig.svelte';
  import BrainPresets from './BrainPresets.svelte';
  import BrainTest from './BrainTest.svelte';
  import { brainsStore } from '$lib/stores/brains';
  import { brainPreferencesStore } from '$lib/stores/brainPreferences';
  import type { Brain, BrainConfig as Config, BrainPreset } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    change: { defaultBrain: string };
    presetCreate: BrainPreset;
  }>();

  let searchQuery = writable<string>('');
  let filterProvider = writable<string>('all');
  let filterCapability = writable<string>('all');
  let selectedBrainId = writable<string | null>(null);
  let showConfig = writable<boolean>(false);
  let showPresets = writable<boolean>(false);
  let showTest = writable<boolean>(false);
  let viewMode = writable<'grid' | 'list'>('grid');

  const brains = derived(brainsStore, ($store) => $store.available);

  const filteredBrains = derived(
    [brains, searchQuery, filterProvider, filterCapability],
    ([$brains, $query, $provider, $capability]) => {
      return $brains.filter(brain => {
        // Search filter
        if ($query) {
          const query = $query.toLowerCase();
          const matches =
            brain.name.toLowerCase().includes(query) ||
            brain.description.toLowerCase().includes(query) ||
            brain.provider.toLowerCase().includes(query);
          if (!matches) return false;
        }

        // Provider filter
        if ($provider !== 'all' && brain.provider !== $provider) {
          return false;
        }

        // Capability filter
        if ($capability !== 'all' && !brain.capabilities.includes($capability)) {
          return false;
        }

        return true;
      });
    }
  );

  const providers = derived(brains, ($brains) => {
    const providerSet = new Set($brains.map(b => b.provider));
    return Array.from(providerSet).sort();
  });

  const capabilities = derived(brains, ($brains) => {
    const capSet = new Set($brains.flatMap(b => b.capabilities));
    return Array.from(capSet).sort();
  });

  const defaultBrain = derived(brainPreferencesStore, ($prefs) => $prefs.defaultBrainId);

  const defaultForgeBrain = derived(brainPreferencesStore, ($prefs) => $prefs.defaultForgeBrainId);

  const favorites = derived(brainPreferencesStore, ($prefs) => $prefs.favorites);

  const recentBrains = derived(brainPreferencesStore, ($prefs) => $prefs.recentlyUsed);

  const selectedBrain = derived(
    [brains, selectedBrainId],
    ([$brains, $id]) => $brains.find(b => b.id === $id) || null
  );

  const brainConfig = derived(
    [brainPreferencesStore, selectedBrainId],
    ([$prefs, $id]) => $id ? $prefs.configs[$id] || getDefaultConfig() : null
  );

  function getDefaultConfig(): Config {
    return {
      temperature: 0.7,
      maxTokens: 4096,
      topP: 1,
      frequencyPenalty: 0,
      presencePenalty: 0,
      systemPrompt: ''
    };
  }

  function selectBrain(brainId: string) {
    selectedBrainId.set(brainId);
  }

  function setAsDefault(brainId: string) {
    brainPreferencesStore.setDefault(brainId);
    dispatch('change', { defaultBrain: brainId });
  }

  function setAsForgeDefault(brainId: string) {
    brainPreferencesStore.setForgeDefault(brainId);
  }

  function toggleFavorite(brainId: string) {
    brainPreferencesStore.toggleFavorite(brainId);
  }

  function updateBrainConfig(brainId: string, config: Partial<Config>) {
    brainPreferencesStore.updateConfig(brainId, config);
  }

  function createPreset(name: string, brainIds: string[], configs: Record<string, Config>) {
    const preset: BrainPreset = {
      id: crypto.randomUUID(),
      name,
      brainIds,
      configs,
      createdAt: new Date()
    };
    brainPreferencesStore.addPreset(preset);
    dispatch('presetCreate', preset);
  }

  onMount(() => {
    brainsStore.loadAvailable();
    brainPreferencesStore.load();
  });
</script>

<div class="brain-selection" data-testid="brain-selection">
  <header class="selection-header">
    <div class="header-title">
      <h2>Brain Selection</h2>
      <p class="description">Configure default AI models and preferences</p>
    </div>
  </header>

  <div class="defaults-section">
    <h3>Default Brains</h3>
    <div class="defaults-grid">
      <div class="default-card">
        <span class="default-label">Chat Default</span>
        {#if $defaultBrain}
          {@const brain = $brains.find(b => b.id === $defaultBrain)}
          {#if brain}
            <div class="default-brain">
              <span class="brain-name">{brain.name}</span>
              <span class="brain-provider">{brain.provider}</span>
            </div>
          {/if}
        {:else}
          <span class="no-default">Not set</span>
        {/if}
      </div>

      <div class="default-card">
        <span class="default-label">Forge Default</span>
        {#if $defaultForgeBrain}
          {@const brain = $brains.find(b => b.id === $defaultForgeBrain)}
          {#if brain}
            <div class="default-brain">
              <span class="brain-name">{brain.name}</span>
              <span class="brain-provider">{brain.provider}</span>
            </div>
          {/if}
        {:else}
          <span class="no-default">Not set</span>
        {/if}
      </div>
    </div>
  </div>

  {#if $recentBrains.length > 0}
    <div class="recent-section">
      <h3>Recently Used</h3>
      <div class="recent-list">
        {#each $recentBrains.slice(0, 5) as brainId}
          {@const brain = $brains.find(b => b.id === brainId)}
          {#if brain}
            <button
              class="recent-item"
              on:click={() => selectBrain(brain.id)}
            >
              <span class="brain-name">{brain.name}</span>
              <span class="brain-provider">{brain.provider}</span>
            </button>
          {/if}
        {/each}
      </div>
    </div>
  {/if}

  <div class="browser-section">
    <div class="browser-toolbar">
      <div class="search-box">
        <input
          type="search"
          placeholder="Search brains..."
          bind:value={$searchQuery}
          class="search-input"
        />
      </div>

      <div class="filters">
        <select bind:value={$filterProvider} class="filter-select">
          <option value="all">All Providers</option>
          {#each $providers as provider}
            <option value={provider}>{provider}</option>
          {/each}
        </select>

        <select bind:value={$filterCapability} class="filter-select">
          <option value="all">All Capabilities</option>
          {#each $capabilities as capability}
            <option value={capability}>{capability}</option>
          {/each}
        </select>
      </div>

      <div class="toolbar-actions">
        <button
          class="action-btn"
          on:click={() => showPresets.set(true)}
        >
          Presets
        </button>

        <div class="view-toggle">
          <button
            class:active={$viewMode === 'grid'}
            on:click={() => viewMode.set('grid')}
          >
            Grid
          </button>
          <button
            class:active={$viewMode === 'list'}
            on:click={() => viewMode.set('list')}
          >
            List
          </button>
        </div>
      </div>
    </div>

    <div class="brain-browser">
      <div class="brain-{$viewMode}">
        {#each $filteredBrains as brain (brain.id)}
          <BrainCard
            {brain}
            selected={$selectedBrainId === brain.id}
            isDefault={$defaultBrain === brain.id}
            isFavorite={$favorites.includes(brain.id)}
            viewMode={$viewMode}
            on:click={() => selectBrain(brain.id)}
            on:setDefault={() => setAsDefault(brain.id)}
            on:toggleFavorite={() => toggleFavorite(brain.id)}
          />
        {/each}

        {#if $filteredBrains.length === 0}
          <div class="empty-state">
            <p>No brains match your filters</p>
          </div>
        {/if}
      </div>

      {#if $selectedBrain}
        <aside class="brain-detail-panel" transition:slide={{ axis: 'x' }}>
          <BrainDetails
            brain={$selectedBrain}
            config={$brainConfig}
            isDefault={$defaultBrain === $selectedBrain.id}
            isForgeDefault={$defaultForgeBrain === $selectedBrain.id}
            isFavorite={$favorites.includes($selectedBrain.id)}
            on:close={() => selectedBrainId.set(null)}
            on:setDefault={() => setAsDefault($selectedBrain.id)}
            on:setForgeDefault={() => setAsForgeDefault($selectedBrain.id)}
            on:toggleFavorite={() => toggleFavorite($selectedBrain.id)}
            on:configure={() => showConfig.set(true)}
            on:test={() => showTest.set(true)}
          />
        </aside>
      {/if}
    </div>
  </div>

  {#if $showConfig && $selectedBrain}
    <div class="modal-overlay" transition:fade on:click={() => showConfig.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <BrainConfig
          brain={$selectedBrain}
          config={$brainConfig}
          on:save={(e) => {
            updateBrainConfig($selectedBrain.id, e.detail);
            showConfig.set(false);
          }}
          on:close={() => showConfig.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showTest && $selectedBrain}
    <div class="modal-overlay" transition:fade on:click={() => showTest.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <BrainTest
          brain={$selectedBrain}
          config={$brainConfig}
          on:close={() => showTest.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showPresets}
    <div class="modal-overlay" transition:fade on:click={() => showPresets.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <BrainPresets
          brains={$brains}
          on:create={(e) => createPreset(e.detail.name, e.detail.brainIds, e.detail.configs)}
          on:close={() => showPresets.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .brain-selection {
    max-width: 1200px;
  }

  .selection-header {
    margin-bottom: 2rem;
  }

  .selection-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .defaults-section,
  .recent-section {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1.5rem;
  }

  .defaults-section h3,
  .recent-section h3 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .defaults-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }

  .default-card {
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .default-label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
  }

  .default-brain {
    display: flex;
    flex-direction: column;
  }

  .brain-name {
    font-weight: 500;
  }

  .brain-provider {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .no-default {
    color: var(--text-muted);
    font-style: italic;
  }

  .recent-list {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .recent-item {
    display: flex;
    flex-direction: column;
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
  }

  .recent-item:hover {
    border-color: var(--primary-color);
  }

  .browser-toolbar {
    display: flex;
    gap: 1rem;
    align-items: center;
    margin-bottom: 1rem;
  }

  .search-box {
    flex: 1;
    max-width: 300px;
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
  }

  .filters {
    display: flex;
    gap: 0.5rem;
  }

  .filter-select {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .toolbar-actions {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    margin-left: auto;
  }

  .action-btn {
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .view-toggle {
    display: flex;
    background: var(--secondary-bg);
    border-radius: 4px;
    overflow: hidden;
  }

  .view-toggle button {
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .view-toggle button.active {
    background: var(--primary-color);
    color: white;
  }

  .brain-browser {
    display: flex;
    gap: 1.5rem;
  }

  .brain-grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
  }

  .brain-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .brain-detail-panel {
    width: 400px;
    flex-shrink: 0;
  }

  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 3rem;
    color: var(--text-muted);
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
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .modal-content.large {
    max-width: 800px;
  }

  @media (max-width: 768px) {
    .defaults-grid {
      grid-template-columns: 1fr;
    }

    .browser-toolbar {
      flex-wrap: wrap;
    }

    .brain-browser {
      flex-direction: column;
    }

    .brain-detail-panel {
      width: 100%;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test filtering, search, and selection logic
2. **Integration Tests**: Verify preference persistence
3. **Config Tests**: Test brain configuration updates
4. **Preset Tests**: Validate preset creation and application
5. **Performance Tests**: Test with many brains

## Related Specs
- Spec 276: Settings Layout
- Spec 279: Think Tank Selection
- Spec 259: Participant Select (Forge UI)
