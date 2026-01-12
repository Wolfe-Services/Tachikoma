# Spec 259: Participant Select

## Header
- **Spec ID**: 259
- **Phase**: 12 - Forge UI
- **Component**: Participant Select
- **Dependencies**: Spec 257 (Session Creation)
- **Status**: Draft

## Objective
Create a participant selection interface that allows users to choose and configure AI participants (brains) for deliberation sessions, with filtering, search, compatibility checking, and role assignment capabilities.

## Acceptance Criteria
1. Grid and list view options for browsing available brains
2. Search and filter by capabilities, provider, and cost tier
3. Drag-and-drop ordering for participant priority
4. Role assignment for specialized functions (advocate, critic, synthesizer)
5. Compatibility indicators between selected participants
6. Cost estimation per participant and total
7. Minimum/maximum participant validation
8. Preset participant groups for common configurations

## Implementation

### ParticipantSelect.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { flip } from 'svelte/animate';
  import { dndzone } from 'svelte-dnd-action';
  import ParticipantCard from './ParticipantCard.svelte';
  import ParticipantFilters from './ParticipantFilters.svelte';
  import ParticipantPresets from './ParticipantPresets.svelte';
  import CompatibilityMatrix from './CompatibilityMatrix.svelte';
  import { brainsStore } from '$lib/stores/brains';
  import type {
    Brain,
    Participant,
    ParticipantRole,
    ParticipantFilters as Filters,
    ParticipantPreset
  } from '$lib/types/forge';

  export let selected: Participant[] = [];
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    change: Participant[];
  }>();

  let searchQuery = writable<string>('');
  let filters = writable<Filters>({
    providers: [],
    capabilities: [],
    costTier: null,
    minRating: null
  });
  let viewMode = writable<'grid' | 'list'>('grid');
  let showCompatibility = writable<boolean>(false);
  let showPresets = writable<boolean>(false);

  const availableBrains = derived(
    [brainsStore, searchQuery, filters],
    ([$brains, $query, $filters]) => {
      let result = $brains.available;

      // Search filter
      if ($query) {
        const lowerQuery = $query.toLowerCase();
        result = result.filter(brain =>
          brain.name.toLowerCase().includes(lowerQuery) ||
          brain.description.toLowerCase().includes(lowerQuery) ||
          brain.provider.toLowerCase().includes(lowerQuery)
        );
      }

      // Provider filter
      if ($filters.providers.length > 0) {
        result = result.filter(brain =>
          $filters.providers.includes(brain.provider)
        );
      }

      // Capabilities filter
      if ($filters.capabilities.length > 0) {
        result = result.filter(brain =>
          $filters.capabilities.every(cap =>
            brain.capabilities.includes(cap)
          )
        );
      }

      // Cost tier filter
      if ($filters.costTier) {
        result = result.filter(brain =>
          brain.costTier === $filters.costTier
        );
      }

      // Rating filter
      if ($filters.minRating) {
        result = result.filter(brain =>
          brain.rating >= $filters.minRating
        );
      }

      return result;
    }
  );

  const selectedIds = derived(
    [() => selected],
    () => new Set(selected.map(p => p.brainId))
  );

  const totalCost = derived(
    [() => selected],
    () => selected.reduce((sum, p) => sum + (p.estimatedCostPerRound || 0), 0)
  );

  const compatibilityScore = derived(
    [() => selected],
    () => {
      if (selected.length < 2) return 1;

      // Calculate average compatibility between all pairs
      let totalScore = 0;
      let pairs = 0;

      for (let i = 0; i < selected.length; i++) {
        for (let j = i + 1; j < selected.length; j++) {
          totalScore += calculatePairCompatibility(selected[i], selected[j]);
          pairs++;
        }
      }

      return pairs > 0 ? totalScore / pairs : 1;
    }
  );

  function calculatePairCompatibility(a: Participant, b: Participant): number {
    // Simple compatibility score based on diverse but complementary capabilities
    const sharedCaps = a.capabilities.filter(c => b.capabilities.includes(c));
    const uniqueCaps = new Set([...a.capabilities, ...b.capabilities]);

    const diversity = 1 - (sharedCaps.length / uniqueCaps.size);
    const providerDiversity = a.provider !== b.provider ? 0.1 : 0;

    return Math.min(1, 0.5 + diversity * 0.4 + providerDiversity);
  }

  function addParticipant(brain: Brain) {
    if ($selectedIds.has(brain.id)) return;
    if (selected.length >= 10) return;

    const participant: Participant = {
      id: crypto.randomUUID(),
      brainId: brain.id,
      name: brain.name,
      provider: brain.provider,
      capabilities: brain.capabilities,
      role: 'participant',
      estimatedCostPerRound: brain.estimatedCostPerRound,
      config: {}
    };

    const newSelected = [...selected, participant];
    dispatch('change', newSelected);
  }

  function removeParticipant(participantId: string) {
    const newSelected = selected.filter(p => p.id !== participantId);
    dispatch('change', newSelected);
  }

  function updateParticipantRole(participantId: string, role: ParticipantRole) {
    const newSelected = selected.map(p =>
      p.id === participantId ? { ...p, role } : p
    );
    dispatch('change', newSelected);
  }

  function handleDndConsider(event: CustomEvent) {
    selected = event.detail.items;
  }

  function handleDndFinalize(event: CustomEvent) {
    selected = event.detail.items;
    dispatch('change', selected);
  }

  function applyPreset(preset: ParticipantPreset) {
    const participants: Participant[] = preset.brainIds.map((brainId, index) => {
      const brain = $brainsStore.available.find(b => b.id === brainId);
      if (!brain) return null;

      return {
        id: crypto.randomUUID(),
        brainId: brain.id,
        name: brain.name,
        provider: brain.provider,
        capabilities: brain.capabilities,
        role: preset.roles?.[index] || 'participant',
        estimatedCostPerRound: brain.estimatedCostPerRound,
        config: preset.configs?.[index] || {}
      };
    }).filter(Boolean) as Participant[];

    dispatch('change', participants);
    showPresets.set(false);
  }

  onMount(() => {
    brainsStore.loadAvailable();
  });
</script>

<div class="participant-select" data-testid="participant-select">
  <div class="select-header">
    <h2>Select Participants</h2>
    <p class="subtitle">
      Choose AI brains to participate in the deliberation.
      Minimum 2, maximum 10 participants.
    </p>
  </div>

  <div class="toolbar">
    <div class="search-box">
      <input
        type="search"
        placeholder="Search brains..."
        bind:value={$searchQuery}
        class="search-input"
      />
    </div>

    <div class="toolbar-actions">
      <button
        type="button"
        class="action-btn"
        class:active={$showPresets}
        on:click={() => showPresets.update(v => !v)}
      >
        Presets
      </button>
      <button
        type="button"
        class="action-btn"
        class:active={$showCompatibility}
        on:click={() => showCompatibility.update(v => !v)}
      >
        Compatibility
      </button>
      <div class="view-toggle">
        <button
          type="button"
          class:active={$viewMode === 'grid'}
          on:click={() => viewMode.set('grid')}
          aria-label="Grid view"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <rect x="3" y="3" width="7" height="7" />
            <rect x="14" y="3" width="7" height="7" />
            <rect x="3" y="14" width="7" height="7" />
            <rect x="14" y="14" width="7" height="7" />
          </svg>
        </button>
        <button
          type="button"
          class:active={$viewMode === 'list'}
          on:click={() => viewMode.set('list')}
          aria-label="List view"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <rect x="3" y="4" width="18" height="4" />
            <rect x="3" y="10" width="18" height="4" />
            <rect x="3" y="16" width="18" height="4" />
          </svg>
        </button>
      </div>
    </div>
  </div>

  {#if $showPresets}
    <ParticipantPresets on:apply={(e) => applyPreset(e.detail)} />
  {/if}

  <ParticipantFilters bind:filters={$filters} />

  <div class="content-area">
    <div class="available-section">
      <h3>Available Brains ({$availableBrains.length})</h3>
      <div class="brain-{$viewMode}">
        {#each $availableBrains as brain (brain.id)}
          <ParticipantCard
            {brain}
            selected={$selectedIds.has(brain.id)}
            disabled={selected.length >= 10 && !$selectedIds.has(brain.id)}
            viewMode={$viewMode}
            on:click={() => addParticipant(brain)}
          />
        {/each}

        {#if $availableBrains.length === 0}
          <div class="empty-state">
            <p>No brains match your filters</p>
            <button
              type="button"
              class="clear-filters-btn"
              on:click={() => {
                searchQuery.set('');
                filters.set({ providers: [], capabilities: [], costTier: null, minRating: null });
              }}
            >
              Clear Filters
            </button>
          </div>
        {/if}
      </div>
    </div>

    <div class="selected-section">
      <div class="selected-header">
        <h3>Selected ({selected.length}/10)</h3>
        <span class="cost-display">
          ~${$totalCost.toFixed(4)}/round
        </span>
      </div>

      {#if selected.length > 0}
        <div
          class="selected-list"
          use:dndzone={{ items: selected, flipDurationMs: 200 }}
          on:consider={handleDndConsider}
          on:finalize={handleDndFinalize}
        >
          {#each selected as participant (participant.id)}
            <div class="selected-item" animate:flip={{ duration: 200 }}>
              <div class="drag-handle">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                  <circle cx="9" cy="6" r="2" />
                  <circle cx="15" cy="6" r="2" />
                  <circle cx="9" cy="12" r="2" />
                  <circle cx="15" cy="12" r="2" />
                  <circle cx="9" cy="18" r="2" />
                  <circle cx="15" cy="18" r="2" />
                </svg>
              </div>

              <div class="participant-info">
                <span class="participant-name">{participant.name}</span>
                <span class="participant-provider">{participant.provider}</span>
              </div>

              <select
                class="role-select"
                value={participant.role}
                on:change={(e) => updateParticipantRole(
                  participant.id,
                  (e.target as HTMLSelectElement).value as ParticipantRole
                )}
              >
                <option value="participant">Participant</option>
                <option value="advocate">Advocate</option>
                <option value="critic">Critic</option>
                <option value="synthesizer">Synthesizer</option>
              </select>

              <button
                type="button"
                class="remove-btn"
                on:click={() => removeParticipant(participant.id)}
                aria-label="Remove {participant.name}"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                  <path d="M18 6L6 18M6 6l12 12" stroke-width="2" stroke-linecap="round" />
                </svg>
              </button>
            </div>
          {/each}
        </div>

        {#if $showCompatibility && selected.length >= 2}
          <CompatibilityMatrix participants={selected} />
        {/if}

        <div class="compatibility-indicator">
          <span class="compat-label">Team Compatibility:</span>
          <div class="compat-bar">
            <div
              class="compat-fill"
              style="width: {$compatibilityScore * 100}%"
              class:low={$compatibilityScore < 0.5}
              class:medium={$compatibilityScore >= 0.5 && $compatibilityScore < 0.7}
              class:high={$compatibilityScore >= 0.7}
            ></div>
          </div>
          <span class="compat-value">{($compatibilityScore * 100).toFixed(0)}%</span>
        </div>
      {:else}
        <div class="empty-selection">
          <p>No participants selected</p>
          <p class="hint">Click on brains to add them to the session</p>
        </div>
      {/if}
    </div>
  </div>

  {#if errors.length > 0}
    <div class="error-messages" role="alert">
      {#each errors as error}
        <p class="error-message">{error}</p>
      {/each}
    </div>
  {/if}
</div>

<style>
  .participant-select {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .select-header h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
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

  .toolbar-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .action-btn {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .action-btn.active {
    background: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  .view-toggle {
    display: flex;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    overflow: hidden;
  }

  .view-toggle button {
    padding: 0.5rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .view-toggle button.active {
    background: var(--secondary-bg);
    color: var(--text-primary);
  }

  .content-area {
    display: grid;
    grid-template-columns: 1fr 350px;
    gap: 1.5rem;
  }

  .available-section h3,
  .selected-section h3 {
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 0.75rem;
    color: var(--text-secondary);
  }

  .brain-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
  }

  .brain-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .selected-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .cost-display {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .selected-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-height: 100px;
  }

  .selected-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
  }

  .drag-handle {
    cursor: grab;
    color: var(--text-muted);
  }

  .participant-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .participant-name {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .participant-provider {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .role-select {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.75rem;
  }

  .remove-btn {
    padding: 0.25rem;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
  }

  .remove-btn:hover {
    color: var(--error-color);
  }

  .compatibility-indicator {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 1rem;
    padding: 0.75rem;
    background: var(--card-bg);
    border-radius: 6px;
  }

  .compat-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .compat-bar {
    flex: 1;
    height: 6px;
    background: var(--border-color);
    border-radius: 3px;
    overflow: hidden;
  }

  .compat-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .compat-fill.low {
    background: var(--error-color);
  }

  .compat-fill.medium {
    background: var(--warning-color);
  }

  .compat-fill.high {
    background: var(--success-color);
  }

  .compat-value {
    font-size: 0.75rem;
    font-weight: 500;
    min-width: 35px;
  }

  .empty-selection,
  .empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.75rem;
    margin-top: 0.5rem;
  }

  .error-messages {
    margin-top: 0.5rem;
  }

  .error-message {
    font-size: 0.875rem;
    color: var(--error-color);
  }

  @media (max-width: 900px) {
    .content-area {
      grid-template-columns: 1fr;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test filtering, search, and selection logic
2. **Integration Tests**: Verify drag-and-drop reordering works correctly
3. **Compatibility Tests**: Validate compatibility score calculations
4. **Preset Tests**: Ensure presets apply correctly
5. **Performance Tests**: Test with large brain lists

## Related Specs
- Spec 257: Session Creation
- Spec 258: Goal Input
- Spec 260: Oracle Select
- Spec 278: Brain Selection (Settings)
