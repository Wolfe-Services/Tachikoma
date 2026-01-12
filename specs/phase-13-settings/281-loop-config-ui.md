# Spec 281: Loop Config UI

## Header
- **Spec ID**: 281
- **Phase**: 13 - Settings UI
- **Component**: Loop Config UI
- **Dependencies**: Spec 276 (Settings Layout)
- **Status**: Draft

## Objective
Create a configuration interface for managing deliberation loop settings, including round limits, iteration parameters, and convergence behaviors for forge sessions.

## Acceptance Criteria
1. Configure maximum rounds per session
2. Set iteration limits for each phase
3. Configure automatic retry behavior
4. Define loop timeout settings
5. Set up parallel execution options
6. Configure checkpoint frequency
7. Define resource limits per loop
8. Visualize loop flow configuration

## Implementation

### LoopConfigUI.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide } from 'svelte/transition';
  import LoopFlowVisualizer from './LoopFlowVisualizer.svelte';
  import PhaseConfig from './PhaseConfig.svelte';
  import TimeoutConfig from './TimeoutConfig.svelte';
  import ResourceLimits from './ResourceLimits.svelte';
  import { loopConfigStore } from '$lib/stores/loopConfig';
  import type { LoopConfig, PhaseConfig as Phase, TimeoutSettings } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: LoopConfig;
    reset: void;
  }>();

  let showAdvanced = writable<boolean>(false);
  let activePhase = writable<string | null>(null);

  const config = derived(loopConfigStore, ($store) => $store.config);

  const phases = derived(config, ($config) => [
    { id: 'drafting', name: 'Drafting', config: $config.phases.drafting },
    { id: 'critiquing', name: 'Critiquing', config: $config.phases.critiquing },
    { id: 'deliberating', name: 'Deliberating', config: $config.phases.deliberating },
    { id: 'converging', name: 'Converging', config: $config.phases.converging }
  ]);

  function updateConfig(field: keyof LoopConfig, value: unknown) {
    loopConfigStore.update(field, value);
  }

  function updatePhase(phaseId: string, phaseConfig: Phase) {
    loopConfigStore.updatePhase(phaseId, phaseConfig);
  }

  function updateTimeout(settings: TimeoutSettings) {
    loopConfigStore.updateTimeout(settings);
  }

  async function saveConfig() {
    await loopConfigStore.save();
    dispatch('save', $config);
  }

  function resetToDefaults() {
    if (confirm('Reset all loop settings to defaults?')) {
      loopConfigStore.resetToDefaults();
      dispatch('reset');
    }
  }

  onMount(() => {
    loopConfigStore.load();
  });
</script>

<div class="loop-config-ui" data-testid="loop-config-ui">
  <header class="config-header">
    <div class="header-title">
      <h2>Loop Configuration</h2>
      <p class="description">Configure deliberation loop behavior and limits</p>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={resetToDefaults}>
        Reset to Defaults
      </button>
      <button class="btn primary" on:click={saveConfig}>
        Save Configuration
      </button>
    </div>
  </header>

  <section class="visualizer-section">
    <LoopFlowVisualizer
      config={$config}
      activePhase={$activePhase}
      on:phaseClick={(e) => activePhase.set(e.detail)}
    />
  </section>

  <section class="basic-config">
    <h3>Basic Settings</h3>

    <div class="config-grid">
      <div class="form-group">
        <label for="max-rounds">Maximum Rounds</label>
        <input
          id="max-rounds"
          type="number"
          value={$config.maxRounds}
          on:input={(e) => updateConfig('maxRounds', parseInt((e.target as HTMLInputElement).value))}
          min="1"
          max="50"
        />
        <span class="help-text">Maximum number of deliberation rounds per session</span>
      </div>

      <div class="form-group">
        <label for="min-rounds">Minimum Rounds</label>
        <input
          id="min-rounds"
          type="number"
          value={$config.minRounds}
          on:input={(e) => updateConfig('minRounds', parseInt((e.target as HTMLInputElement).value))}
          min="1"
          max="20"
        />
        <span class="help-text">Minimum rounds before checking convergence</span>
      </div>

      <div class="form-group">
        <label for="checkpoint-frequency">Checkpoint Frequency</label>
        <select
          id="checkpoint-frequency"
          value={$config.checkpointFrequency}
          on:change={(e) => updateConfig('checkpointFrequency', (e.target as HTMLSelectElement).value)}
        >
          <option value="every_round">Every Round</option>
          <option value="every_2_rounds">Every 2 Rounds</option>
          <option value="every_5_rounds">Every 5 Rounds</option>
          <option value="on_phase_change">On Phase Change</option>
          <option value="manual">Manual Only</option>
        </select>
        <span class="help-text">How often to save session checkpoints</span>
      </div>

      <div class="form-group">
        <label for="parallel-participants">Parallel Participants</label>
        <input
          id="parallel-participants"
          type="number"
          value={$config.parallelParticipants}
          on:input={(e) => updateConfig('parallelParticipants', parseInt((e.target as HTMLInputElement).value))}
          min="1"
          max="10"
        />
        <span class="help-text">Max participants processing in parallel</span>
      </div>
    </div>

    <div class="toggle-options">
      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.autoRetry}
          on:change={(e) => updateConfig('autoRetry', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-label">Auto-retry on failures</span>
        <span class="toggle-description">Automatically retry failed operations</span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.allowEarlyTermination}
          on:change={(e) => updateConfig('allowEarlyTermination', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-label">Allow early termination</span>
        <span class="toggle-description">Stop before max rounds if converged</span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.preservePartialResults}
          on:change={(e) => updateConfig('preservePartialResults', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-label">Preserve partial results</span>
        <span class="toggle-description">Save results even if session fails</span>
      </label>
    </div>
  </section>

  <section class="phase-configs">
    <h3>Phase Configuration</h3>

    <div class="phase-tabs">
      {#each $phases as phase}
        <button
          class="phase-tab"
          class:active={$activePhase === phase.id}
          on:click={() => activePhase.set(phase.id)}
        >
          {phase.name}
        </button>
      {/each}
    </div>

    {#each $phases as phase (phase.id)}
      {#if $activePhase === phase.id}
        <div class="phase-content" transition:slide>
          <PhaseConfig
            phaseId={phase.id}
            phaseName={phase.name}
            config={phase.config}
            on:change={(e) => updatePhase(phase.id, e.detail)}
          />
        </div>
      {/if}
    {/each}
  </section>

  <button
    class="toggle-advanced"
    on:click={() => showAdvanced.update(v => !v)}
  >
    {$showAdvanced ? 'Hide' : 'Show'} Advanced Settings
  </button>

  {#if $showAdvanced}
    <section class="advanced-config" transition:slide>
      <h3>Timeout Configuration</h3>
      <TimeoutConfig
        settings={$config.timeouts}
        on:change={(e) => updateTimeout(e.detail)}
      />

      <h3>Resource Limits</h3>
      <ResourceLimits
        limits={$config.resourceLimits}
        on:change={(e) => updateConfig('resourceLimits', e.detail)}
      />

      <div class="retry-config">
        <h3>Retry Configuration</h3>

        <div class="config-grid">
          <div class="form-group">
            <label for="retry-attempts">Max Retry Attempts</label>
            <input
              id="retry-attempts"
              type="number"
              value={$config.retryConfig.maxAttempts}
              on:input={(e) => updateConfig('retryConfig', {
                ...$config.retryConfig,
                maxAttempts: parseInt((e.target as HTMLInputElement).value)
              })}
              min="0"
              max="10"
            />
          </div>

          <div class="form-group">
            <label for="retry-delay">Retry Delay (ms)</label>
            <input
              id="retry-delay"
              type="number"
              value={$config.retryConfig.delayMs}
              on:input={(e) => updateConfig('retryConfig', {
                ...$config.retryConfig,
                delayMs: parseInt((e.target as HTMLInputElement).value)
              })}
              min="100"
              max="30000"
              step="100"
            />
          </div>

          <div class="form-group">
            <label for="backoff-multiplier">Backoff Multiplier</label>
            <input
              id="backoff-multiplier"
              type="number"
              value={$config.retryConfig.backoffMultiplier}
              on:input={(e) => updateConfig('retryConfig', {
                ...$config.retryConfig,
                backoffMultiplier: parseFloat((e.target as HTMLInputElement).value)
              })}
              min="1"
              max="5"
              step="0.1"
            />
          </div>
        </div>
      </div>
    </section>
  {/if}
</div>

<style>
  .loop-config-ui {
    max-width: 900px;
  }

  .config-header {
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

  .visualizer-section {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .basic-config,
  .phase-configs,
  .advanced-config {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1.25rem;
  }

  .config-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1.25rem;
    margin-bottom: 1.5rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  .form-group label {
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  .form-group input,
  .form-group select {
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .help-text {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.375rem;
  }

  .toggle-options {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .toggle-option {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    gap: 0.25rem 0.75rem;
    align-items: center;
    cursor: pointer;
  }

  .toggle-option input {
    grid-row: span 2;
    width: 18px;
    height: 18px;
  }

  .toggle-label {
    font-size: 0.875rem;
    font-weight: 500;
  }

  .toggle-description {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .phase-tabs {
    display: flex;
    gap: 0.25rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    padding: 0.25rem;
    margin-bottom: 1rem;
  }

  .phase-tab {
    flex: 1;
    padding: 0.625rem 1rem;
    background: transparent;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .phase-tab:hover {
    background: var(--hover-bg);
  }

  .phase-tab.active {
    background: var(--primary-color);
    color: white;
  }

  .phase-content {
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .toggle-advanced {
    display: block;
    width: 100%;
    padding: 0.75rem;
    background: transparent;
    border: 1px dashed var(--border-color);
    border-radius: 6px;
    color: var(--text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    margin-bottom: 1.5rem;
  }

  .toggle-advanced:hover {
    border-color: var(--primary-color);
    color: var(--primary-color);
  }

  .retry-config {
    margin-top: 1.5rem;
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

  @media (max-width: 768px) {
    .config-grid {
      grid-template-columns: 1fr;
    }

    .config-header {
      flex-direction: column;
      gap: 1rem;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test config validation and updates
2. **Integration Tests**: Verify config persistence
3. **Visualization Tests**: Test flow visualizer accuracy
4. **Phase Tests**: Test phase-specific configurations
5. **Reset Tests**: Verify defaults restoration

## Related Specs
- Spec 276: Settings Layout
- Spec 282: Stop Conditions UI
- Spec 267: Convergence Indicator
