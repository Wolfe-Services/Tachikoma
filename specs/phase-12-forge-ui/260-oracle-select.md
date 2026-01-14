# Spec 260: Oracle Select

## Header
- **Spec ID**: 260
- **Phase**: 12 - Forge UI
- **Component**: Oracle Select
- **Dependencies**: Spec 257 (Session Creation), Spec 259 (Participant Select)
- **Status**: Draft

## Objective
Create an oracle selection interface that allows users to choose and configure the decision-making AI that will evaluate deliberations, synthesize consensus, and make final determinations during forge sessions.

## Acceptance Criteria
- [x] Display available oracles with capability indicators
- [x] Show compatibility with selected participants
- [x] Provide recommendations based on session goal and participants
- [x] Allow oracle configuration (temperature, reasoning style)
- [x] Support custom oracle creation from available brains
- [x] Display cost implications of oracle selection
- [x] Validate oracle selection against session requirements
- [x] Show oracle performance history and ratings

## Implementation

### OracleSelect.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import OracleCard from './OracleCard.svelte';
  import OracleConfig from './OracleConfig.svelte';
  import OracleRecommendations from './OracleRecommendations.svelte';
  import { oracleStore } from '$lib/stores/oracles';
  import { getOracleRecommendations } from '$lib/services/oracleService';
  import type { Oracle, OracleConfig as Config, Participant, OracleRecommendation } from '$lib/types/forge';

  export let selected: Oracle | null = null;
  export let participants: Participant[] = [];
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    change: Oracle | null;
  }>();

  let showConfig = writable<boolean>(false);
  let showRecommendations = writable<boolean>(true);
  let recommendations = writable<OracleRecommendation[]>([]);
  let isLoadingRecommendations = writable<boolean>(false);
  let searchQuery = writable<string>('');

  const availableOracles = derived(
    [oracleStore, searchQuery],
    ([$store, $query]) => {
      let oracles = $store.available;

      if ($query) {
        const lowerQuery = $query.toLowerCase();
        oracles = oracles.filter(oracle =>
          oracle.name.toLowerCase().includes(lowerQuery) ||
          oracle.description.toLowerCase().includes(lowerQuery)
        );
      }

      return oracles;
    }
  );

  const compatibilityScores = derived(
    [availableOracles, () => participants],
    ([$oracles]) => {
      const scores = new Map<string, number>();

      for (const oracle of $oracles) {
        scores.set(oracle.id, calculateCompatibility(oracle));
      }

      return scores;
    }
  );

  function calculateCompatibility(oracle: Oracle): number {
    if (participants.length === 0) return 0.8; // Default good compatibility

    let score = 0.5;

    // Check if oracle can handle all participant providers
    const participantProviders = new Set(participants.map(p => p.provider));
    const handlesAllProviders = [...participantProviders].every(
      provider => oracle.supportedProviders?.includes(provider) ?? true
    );
    if (handlesAllProviders) score += 0.2;

    // Check capability overlap
    const participantCaps = new Set(participants.flatMap(p => p.capabilities));
    const oracleUnderstandsCaps = [...participantCaps].filter(
      cap => oracle.capabilities?.includes(cap)
    ).length / participantCaps.size;
    score += oracleUnderstandsCaps * 0.2;

    // Bonus for being from different provider (diversity)
    const oracleIsFromDifferentProvider = !participantProviders.has(oracle.provider);
    if (oracleIsFromDifferentProvider) score += 0.1;

    return Math.min(1, score);
  }

  async function loadRecommendations() {
    if (participants.length < 2) return;

    isLoadingRecommendations.set(true);
    try {
      const recs = await getOracleRecommendations(participants);
      recommendations.set(recs);
    } catch (error) {
      console.error('Failed to load recommendations:', error);
    } finally {
      isLoadingRecommendations.set(false);
    }
  }

  function selectOracle(oracle: Oracle) {
    const oracleWithDefaults: Oracle = {
      ...oracle,
      config: oracle.config || getDefaultConfig(oracle)
    };
    dispatch('change', oracleWithDefaults);
  }

  function deselectOracle() {
    dispatch('change', null);
  }

  function getDefaultConfig(oracle: Oracle): Config {
    return {
      temperature: 0.7,
      maxTokens: 4096,
      reasoningStyle: 'balanced',
      requireConsensus: true,
      consensusThreshold: 0.7,
      allowDissent: true,
      synthesisMode: 'comprehensive'
    };
  }

  function updateOracleConfig(config: Partial<Config>) {
    if (!selected) return;

    const updated: Oracle = {
      ...selected,
      config: { ...selected.config, ...config }
    };
    dispatch('change', updated);
  }

  function applyRecommendation(rec: OracleRecommendation) {
    const oracle = $availableOracles.find(o => o.id === rec.oracleId);
    if (oracle) {
      selectOracle({
        ...oracle,
        config: rec.suggestedConfig || getDefaultConfig(oracle)
      });
    }
  }

  onMount(() => {
    oracleStore.loadAvailable();
    loadRecommendations();
  });

  $: if (participants.length >= 2) {
    loadRecommendations();
  }
</script>

<div class="oracle-select" data-testid="oracle-select">
  <div class="select-header">
    <h2>Select Oracle</h2>
    <p class="subtitle">
      The oracle evaluates deliberations, synthesizes viewpoints, and makes final determinations.
    </p>
  </div>

  {#if $showRecommendations && $recommendations.length > 0}
    <OracleRecommendations
      recommendations={$recommendations}
      isLoading={$isLoadingRecommendations}
      on:select={(e) => applyRecommendation(e.detail)}
      on:dismiss={() => showRecommendations.set(false)}
    />
  {/if}

  <div class="search-section">
    <input
      type="search"
      placeholder="Search oracles..."
      bind:value={$searchQuery}
      class="search-input"
    />
  </div>

  <div class="oracle-grid">
    {#each $availableOracles as oracle (oracle.id)}
      <OracleCard
        {oracle}
        selected={selected?.id === oracle.id}
        compatibility={$compatibilityScores.get(oracle.id) || 0}
        on:click={() => selectOracle(oracle)}
      />
    {/each}

    {#if $availableOracles.length === 0}
      <div class="empty-state">
        <p>No oracles found</p>
      </div>
    {/if}
  </div>

  {#if selected}
    <div class="selected-oracle">
      <div class="selected-header">
        <h3>Selected Oracle</h3>
        <button
          type="button"
          class="configure-btn"
          on:click={() => showConfig.update(v => !v)}
        >
          {$showConfig ? 'Hide Config' : 'Configure'}
        </button>
      </div>

      <div class="selected-details">
        <div class="oracle-summary">
          <div class="oracle-icon">
            {#if selected.icon}
              <img src={selected.icon} alt="" />
            {:else}
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
              </svg>
            {/if}
          </div>
          <div class="oracle-info">
            <span class="oracle-name">{selected.name}</span>
            <span class="oracle-provider">{selected.provider}</span>
          </div>
          <div class="oracle-cost">
            ~${selected.estimatedCostPerRound?.toFixed(4)}/round
          </div>
          <button
            type="button"
            class="remove-btn"
            on:click={deselectOracle}
            aria-label="Remove oracle"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path d="M18 6L6 18M6 6l12 12" stroke-width="2" stroke-linecap="round" />
            </svg>
          </button>
        </div>

        <div class="oracle-capabilities">
          {#each selected.capabilities || [] as capability}
            <span class="capability-tag">{capability}</span>
          {/each}
        </div>

        <div class="compatibility-display">
          <span class="compat-label">Participant Compatibility:</span>
          <div class="compat-bar">
            <div
              class="compat-fill"
              style="width: {($compatibilityScores.get(selected.id) || 0) * 100}%"
            ></div>
          </div>
          <span class="compat-value">
            {(($compatibilityScores.get(selected.id) || 0) * 100).toFixed(0)}%
          </span>
        </div>
      </div>

      {#if $showConfig}
        <OracleConfig
          config={selected.config}
          on:change={(e) => updateOracleConfig(e.detail)}
        />
      {/if}
    </div>
  {/if}

  {#if errors.length > 0}
    <div class="error-messages" role="alert">
      {#each errors as error}
        <p class="error-message">{error}</p>
      {/each}
    </div>
  {/if}
</div>

<style>
  .oracle-select {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
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

  .search-section {
    margin-bottom: 0.5rem;
  }

  .search-input {
    width: 100%;
    max-width: 300px;
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
  }

  .oracle-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .selected-oracle {
    background: var(--card-bg);
    border: 2px solid var(--primary-color);
    border-radius: 8px;
    padding: 1.25rem;
  }

  .selected-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .selected-header h3 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .configure-btn {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .configure-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .oracle-summary {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .oracle-icon {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    background: var(--secondary-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--primary-color);
  }

  .oracle-icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 8px;
  }

  .oracle-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .oracle-name {
    font-weight: 600;
    font-size: 1rem;
  }

  .oracle-provider {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .oracle-cost {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .remove-btn {
    padding: 0.5rem;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 4px;
  }

  .remove-btn:hover {
    background: var(--error-bg);
    color: var(--error-color);
  }

  .oracle-capabilities {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .capability-tag {
    padding: 0.25rem 0.5rem;
    background: var(--tag-bg);
    color: var(--tag-color);
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .compatibility-display {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .compat-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    white-space: nowrap;
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
    background: var(--success-color);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .compat-value {
    font-size: 0.75rem;
    font-weight: 500;
    min-width: 35px;
  }

  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .error-messages {
    margin-top: 0.5rem;
  }

  .error-message {
    font-size: 0.875rem;
    color: var(--error-color);
  }
</style>
```

### OracleConfig.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { OracleConfig as Config } from '$lib/types/forge';

  export let config: Config;

  const dispatch = createEventDispatcher<{
    change: Partial<Config>;
  }>();

  function handleChange(field: keyof Config, value: unknown) {
    dispatch('change', { [field]: value });
  }
</script>

<div class="oracle-config">
  <h4>Oracle Configuration</h4>

  <div class="config-grid">
    <div class="config-field">
      <label for="temperature">Temperature</label>
      <input
        id="temperature"
        type="range"
        min="0"
        max="1"
        step="0.1"
        value={config.temperature}
        on:input={(e) => handleChange('temperature', parseFloat((e.target as HTMLInputElement).value))}
      />
      <span class="field-value">{config.temperature}</span>
    </div>

    <div class="config-field">
      <label for="reasoning-style">Reasoning Style</label>
      <select
        id="reasoning-style"
        value={config.reasoningStyle}
        on:change={(e) => handleChange('reasoningStyle', (e.target as HTMLSelectElement).value)}
      >
        <option value="analytical">Analytical</option>
        <option value="balanced">Balanced</option>
        <option value="creative">Creative</option>
        <option value="conservative">Conservative</option>
      </select>
    </div>

    <div class="config-field">
      <label for="consensus-threshold">Consensus Threshold</label>
      <input
        id="consensus-threshold"
        type="range"
        min="0.5"
        max="1"
        step="0.05"
        value={config.consensusThreshold}
        on:input={(e) => handleChange('consensusThreshold', parseFloat((e.target as HTMLInputElement).value))}
      />
      <span class="field-value">{(config.consensusThreshold * 100).toFixed(0)}%</span>
    </div>

    <div class="config-field checkbox-field">
      <label>
        <input
          type="checkbox"
          checked={config.requireConsensus}
          on:change={(e) => handleChange('requireConsensus', (e.target as HTMLInputElement).checked)}
        />
        Require Consensus
      </label>
    </div>

    <div class="config-field checkbox-field">
      <label>
        <input
          type="checkbox"
          checked={config.allowDissent}
          on:change={(e) => handleChange('allowDissent', (e.target as HTMLInputElement).checked)}
        />
        Allow Recorded Dissent
      </label>
    </div>

    <div class="config-field">
      <label for="synthesis-mode">Synthesis Mode</label>
      <select
        id="synthesis-mode"
        value={config.synthesisMode}
        on:change={(e) => handleChange('synthesisMode', (e.target as HTMLSelectElement).value)}
      >
        <option value="comprehensive">Comprehensive</option>
        <option value="concise">Concise</option>
        <option value="detailed">Detailed</option>
      </select>
    </div>
  </div>
</div>

<style>
  .oracle-config {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .oracle-config h4 {
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 1rem;
  }

  .config-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }

  .config-field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .config-field label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .config-field input[type="range"] {
    width: 100%;
  }

  .config-field select {
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
  }

  .field-value {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: right;
  }

  .checkbox-field {
    flex-direction: row;
    align-items: center;
  }

  .checkbox-field label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test oracle selection and configuration updates
2. **Integration Tests**: Verify compatibility scoring with participants
3. **Recommendation Tests**: Validate recommendation algorithm
4. **Configuration Tests**: Ensure config changes persist correctly
5. **Accessibility Tests**: Test keyboard navigation and screen reader support

## Related Specs
- Spec 257: Session Creation
- Spec 259: Participant Select
- Spec 279: Think Tank Selection (Settings)
