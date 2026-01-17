<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { slide } from 'svelte/transition';
  import Icon from '$lib/components/common/Icon.svelte';
  import type { Oracle, Participant, SessionConfig } from '$lib/types/forge';

  export let selectedOracle: Oracle | null = null;
  export let participants: Participant[] = [];
  export let config: SessionConfig;
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    oracleChange: Oracle | null;
    configChange: SessionConfig;
  }>();

  let showAdvanced = false;

  // Mock oracle data - in a real app this would come from a store
  const availableOracles: Oracle[] = [
    {
      id: 'consensus-oracle-1',
      name: 'Consensus Oracle',
      type: 'consensus',
      config: { threshold: 0.8, method: 'weighted_voting' },
      estimatedCostPerRound: 0.005
    },
    {
      id: 'gpt4-oracle-1',
      name: 'LLM Oracle (Analyst)',
      type: 'llm',
      config: { model: 'gpt-4', temperature: 0.3 },
      estimatedCostPerRound: 0.02
    },
    {
      id: 'claude-oracle-1',
      name: 'LLM Oracle (Architect)',
      type: 'llm',
      config: { model: 'claude-3-sonnet', temperature: 0.2 },
      estimatedCostPerRound: 0.015
    },
    {
      id: 'hybrid-oracle-1',
      name: 'Hybrid Oracle',
      type: 'hybrid',
      config: { llm_weight: 0.6, consensus_weight: 0.4 },
      estimatedCostPerRound: 0.012
    }
  ];

  // Oracle temperature slider
  let oracleTemperature = 0.3;
  
  // Convergence threshold presets
  const convergencePresets = [
    { value: 0.7, label: '70%' },
    { value: 0.8, label: '80%' },
    { value: 0.9, label: '90%' }
  ];

  // Max rounds options
  const maxRoundsOptions = [3, 5, 7, 10, 15, 20];
  
  // Timeout options (in minutes)
  const timeoutOptions = [
    { value: 15, label: '15 min' },
    { value: 30, label: '30 min' },
    { value: 60, label: '1 hr' },
    { value: 120, label: '2 hr' },
    { value: 240, label: '4 hr' },
    { value: 480, label: '8 hr' }
  ];

  // Auto-save options
  const autoSaveOptions = [
    { value: 15000, label: '15s' },
    { value: 30000, label: '30s' },
    { value: 60000, label: '1m' },
    { value: 300000, label: '5m' },
    { value: 0, label: 'Off' }
  ];

  function handleOracleSelect(oracle: Oracle) {
    selectedOracle = oracle;
    // Update temperature from oracle config if available
    if (oracle.config.temperature !== undefined) {
      oracleTemperature = oracle.config.temperature;
    }
    dispatch('oracleChange', oracle);
  }

  function handleTemperatureChange(value: number) {
    oracleTemperature = value;
    if (selectedOracle) {
      const updatedOracle = {
        ...selectedOracle,
        config: { ...selectedOracle.config, temperature: value }
      };
      selectedOracle = updatedOracle;
      dispatch('oracleChange', updatedOracle);
    }
  }

  function updateConfig(field: keyof SessionConfig, value: unknown) {
    const updatedConfig = { ...config, [field]: value };
    config = updatedConfig;
    dispatch('configChange', updatedConfig);
  }

  function getCompatibilityScore(oracle: Oracle): number {
    const aiParticipants = participants.filter(p => p.type === 'ai').length;
    const humanParticipants = participants.filter(p => p.type === 'human').length;
    
    switch (oracle.type) {
      case 'consensus':
        return Math.min(0.9, 0.6 + (participants.length * 0.05));
      case 'llm':
        return Math.min(0.95, 0.7 + (aiParticipants * 0.1));
      case 'hybrid':
        return Math.min(0.9, 0.8 + (Math.min(aiParticipants, humanParticipants) * 0.05));
      default:
        return 0.5;
    }
  }

  function getOracleIcon(type: string): string {
    switch (type) {
      case 'llm': return 'brain';
      case 'consensus': return 'check-circle';
      case 'hybrid': return 'refresh-cw';
      default: return 'cpu';
    }
  }

  function getOracleAccent(type: string): string {
    switch (type) {
      case 'llm': return 'rgba(88, 166, 255, 0.9)';
      case 'consensus': return 'rgba(63, 185, 80, 0.9)';
      case 'hybrid': return 'rgba(78, 205, 196, 0.9)';
      default: return 'rgba(139, 92, 246, 0.9)';
    }
  }

  $: sortedOracles = availableOracles
    .map(oracle => ({
      ...oracle,
      compatibility: getCompatibilityScore(oracle)
    }))
    .sort((a, b) => b.compatibility - a.compatibility);

  $: temperatureLabel = oracleTemperature <= 0.3 ? 'Precise' : oracleTemperature >= 0.7 ? 'Creative' : 'Balanced';
</script>

<div class="oracle-config-panel" data-testid="oracle-config-panel">
  <div class="panel-header">
    <div class="header-icon">
      <Icon name="sliders" size={18} glow />
    </div>
    <div class="header-text">
      <h2>Oracle & Session Configuration</h2>
      <p>Configure the AI orchestrator and session parameters</p>
    </div>
  </div>

  {#if errors.length > 0}
    <div class="error-banner" role="alert">
      <Icon name="alert-triangle" size={16} />
      {#each errors as error}
        <span>{error}</span>
      {/each}
    </div>
  {/if}

  <div class="two-column-layout">
    <!-- Left Column: Oracle Selection -->
    <div class="column oracle-column">
      <div class="section-header">
        <Icon name="brain" size={16} glow />
        <span>Oracle</span>
      </div>

      <!-- Oracle Selection Dropdown -->
      <div class="compact-group">
        <label class="compact-label">Model</label>
        <select
          class="compact-select"
          value={selectedOracle?.id || ''}
          on:change={(e) => {
            const oracle = sortedOracles.find(o => o.id === e.currentTarget.value);
            if (oracle) handleOracleSelect(oracle);
          }}
        >
          <option value="" disabled>Select an oracle...</option>
          {#each sortedOracles as oracle}
            <option value={oracle.id}>
              {oracle.name} ({oracle.type})
            </option>
          {/each}
        </select>
      </div>

      <!-- Temperature Slider (only for LLM oracles) -->
      {#if selectedOracle && (selectedOracle.type === 'llm' || selectedOracle.type === 'hybrid')}
        <div class="compact-group">
          <label class="compact-label">Temperature</label>
          <div class="temperature-control">
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              value={oracleTemperature}
              on:input={(e) => handleTemperatureChange(parseFloat(e.currentTarget.value))}
              class="vertical-slider"
            />
            <div class="temperature-info">
              <span class="temp-value">{oracleTemperature.toFixed(1)}</span>
              <span class="temp-label">{temperatureLabel}</span>
            </div>
          </div>
          <div class="slider-labels">
            <span>Precise</span>
            <span>Creative</span>
          </div>
        </div>
      {/if}

      <!-- Cost Estimate -->
      {#if selectedOracle}
        <div class="cost-display">
          <Icon name="dollar-sign" size={14} />
          <span>~${selectedOracle.estimatedCostPerRound?.toFixed(3) || '0.00'}/round</span>
        </div>
      {/if}

      <!-- Oracle Type Cards (compact radio buttons) -->
      <div class="oracle-types">
        {#each sortedOracles as oracle}
          <button
            type="button"
            class="oracle-chip"
            class:selected={selectedOracle?.id === oracle.id}
            style="--oracle-accent: {getOracleAccent(oracle.type)}"
            on:click={() => handleOracleSelect(oracle)}
          >
            <Icon name={getOracleIcon(oracle.type)} size={14} />
            <span class="chip-label">{oracle.type}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Right Column: Session Parameters -->
    <div class="column config-column">
      <div class="section-header">
        <Icon name="settings" size={16} glow />
        <span>Session Parameters</span>
      </div>

      <!-- Max Rounds -->
      <div class="compact-group inline-group">
        <label class="compact-label">Max Rounds</label>
        <select
          class="compact-select small"
          value={config.maxRounds}
          on:change={(e) => updateConfig('maxRounds', parseInt(e.currentTarget.value))}
        >
          {#each maxRoundsOptions as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>

      <!-- Convergence Threshold with Presets -->
      <div class="compact-group">
        <label class="compact-label">Convergence</label>
        <div class="preset-buttons">
          {#each convergencePresets as preset}
            <button
              type="button"
              class="preset-btn"
              class:active={config.convergenceThreshold === preset.value}
              on:click={() => updateConfig('convergenceThreshold', preset.value)}
            >
              {preset.label}
            </button>
          {/each}
        </div>
      </div>

      <!-- Timeout -->
      <div class="compact-group inline-group">
        <label class="compact-label">Timeout</label>
        <select
          class="compact-select small"
          value={config.timeoutMinutes}
          on:change={(e) => updateConfig('timeoutMinutes', parseInt(e.currentTarget.value))}
        >
          {#each timeoutOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </div>

      <!-- Advanced Options Toggle -->
      <button
        type="button"
        class="advanced-toggle"
        on:click={() => showAdvanced = !showAdvanced}
        aria-expanded={showAdvanced}
      >
        <Icon name={showAdvanced ? 'chevron-down' : 'chevron-right'} size={14} />
        <span>Advanced Options</span>
      </button>

      {#if showAdvanced}
        <div class="advanced-options" transition:slide={{ duration: 200 }}>
          <!-- Human Intervention -->
          <label class="checkbox-group">
            <input
              type="checkbox"
              checked={config.allowHumanIntervention}
              on:change={(e) => updateConfig('allowHumanIntervention', e.currentTarget.checked)}
            />
            <span class="checkbox-label">Human Intervention</span>
          </label>

          <!-- Auto-save Interval -->
          <div class="compact-group inline-group">
            <label class="compact-label">Auto-save</label>
            <select
              class="compact-select small"
              value={config.autoSaveInterval}
              on:change={(e) => updateConfig('autoSaveInterval', parseInt(e.currentTarget.value))}
            >
              {#each autoSaveOptions as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Selection Summary (compact) -->
  {#if selectedOracle}
    <div class="summary-bar">
      <div class="summary-item">
        <Icon name="brain" size={14} />
        <span>{selectedOracle.name}</span>
      </div>
      <div class="summary-divider">•</div>
      <div class="summary-item">
        <span>{config.maxRounds} rounds</span>
      </div>
      <div class="summary-divider">•</div>
      <div class="summary-item">
        <span>{Math.round(config.convergenceThreshold * 100)}% threshold</span>
      </div>
      <div class="summary-divider">•</div>
      <div class="summary-item">
        <span>{config.timeoutMinutes}m timeout</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .oracle-config-panel {
    max-width: 900px;
    margin: 0 auto;
  }

  .panel-header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
  }

  .header-icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 10px;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.18), rgba(78, 205, 196, 0.05));
    border: 1px solid rgba(78, 205, 196, 0.22);
    color: var(--tachi-cyan, #4ecdc4);
    flex-shrink: 0;
  }

  .header-text h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0 0 0.25rem 0;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .header-text p {
    margin: 0;
    color: rgba(230, 237, 243, 0.6);
    font-size: 0.875rem;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.65rem 0.85rem;
    margin-bottom: 1.25rem;
    border-radius: 10px;
    background: rgba(255, 107, 107, 0.08);
    border: 1px solid rgba(255, 107, 107, 0.22);
    color: rgba(255, 107, 107, 0.95);
    font-size: 0.875rem;
  }

  .two-column-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
  }

  .column {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 1.5px;
    text-transform: uppercase;
    color: rgba(78, 205, 196, 0.9);
    padding-bottom: 0.5rem;
    border-bottom: 1px solid rgba(78, 205, 196, 0.14);
    margin-bottom: 0.5rem;
  }

  .compact-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .inline-group {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }

  .compact-label {
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.8px;
    text-transform: uppercase;
    color: rgba(230, 237, 243, 0.6);
  }

  .compact-select {
    width: 100%;
    padding: 0.5rem 0.75rem;
    padding-right: 2rem;
    border: 1px solid rgba(78, 205, 196, 0.18);
    border-radius: 8px;
    background: rgba(13, 17, 23, 0.5);
    color: var(--text-primary, #e6edf3);
    font-size: 0.875rem;
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%234ecdc4' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
  }

  .compact-select.small {
    width: auto;
    min-width: 80px;
  }

  .compact-select:focus {
    outline: none;
    border-color: rgba(78, 205, 196, 0.5);
    box-shadow: 0 0 0 2px rgba(78, 205, 196, 0.1);
  }

  .temperature-control {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .vertical-slider {
    flex: 1;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: linear-gradient(90deg, rgba(88, 166, 255, 0.3), rgba(255, 107, 107, 0.3));
    border-radius: 999px;
    border: 1px solid rgba(78, 205, 196, 0.12);
    cursor: pointer;
  }

  .vertical-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    cursor: pointer;
    box-shadow: 0 0 10px rgba(78, 205, 196, 0.5);
  }

  .vertical-slider::-moz-range-thumb {
    width: 16px;
    height: 16px;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    cursor: pointer;
    border: none;
    box-shadow: 0 0 10px rgba(78, 205, 196, 0.5);
  }

  .temperature-info {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    min-width: 60px;
  }

  .temp-value {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
  }

  .temp-label {
    font-size: 0.65rem;
    color: rgba(230, 237, 243, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .slider-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.65rem;
    color: rgba(230, 237, 243, 0.4);
    margin-top: -0.25rem;
  }

  .cost-display {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.6rem;
    border-radius: 999px;
    background: rgba(63, 185, 80, 0.08);
    border: 1px solid rgba(63, 185, 80, 0.18);
    color: rgba(63, 185, 80, 0.95);
    font-size: 0.75rem;
    font-weight: 500;
    width: fit-content;
  }

  .oracle-types {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .oracle-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.65rem;
    border-radius: 999px;
    background: rgba(13, 17, 23, 0.5);
    border: 1px solid rgba(78, 205, 196, 0.14);
    color: rgba(230, 237, 243, 0.65);
    font-size: 0.7rem;
    font-weight: 500;
    text-transform: capitalize;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .oracle-chip:hover {
    border-color: var(--oracle-accent, rgba(78, 205, 196, 0.5));
    color: rgba(230, 237, 243, 0.85);
  }

  .oracle-chip.selected {
    background: rgba(78, 205, 196, 0.12);
    border-color: var(--oracle-accent, rgba(78, 205, 196, 0.5));
    color: var(--oracle-accent, rgba(78, 205, 196, 0.95));
  }

  .preset-buttons {
    display: flex;
    gap: 0.35rem;
  }

  .preset-btn {
    flex: 1;
    padding: 0.45rem 0.75rem;
    border-radius: 8px;
    background: rgba(13, 17, 23, 0.5);
    border: 1px solid rgba(78, 205, 196, 0.14);
    color: rgba(230, 237, 243, 0.65);
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .preset-btn:hover {
    border-color: rgba(78, 205, 196, 0.35);
    color: rgba(230, 237, 243, 0.85);
  }

  .preset-btn.active {
    background: rgba(78, 205, 196, 0.15);
    border-color: rgba(78, 205, 196, 0.5);
    color: rgba(78, 205, 196, 0.95);
  }

  .advanced-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.45rem 0.65rem;
    margin-top: 0.5rem;
    border-radius: 8px;
    background: transparent;
    border: 1px solid rgba(78, 205, 196, 0.1);
    color: rgba(230, 237, 243, 0.55);
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    width: fit-content;
  }

  .advanced-toggle:hover {
    border-color: rgba(78, 205, 196, 0.22);
    color: rgba(230, 237, 243, 0.75);
  }

  .advanced-options {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.75rem;
    background: rgba(13, 17, 23, 0.3);
    border-radius: 10px;
    border: 1px solid rgba(78, 205, 196, 0.08);
  }

  .checkbox-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .checkbox-group input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--tachi-cyan, #4ecdc4);
    cursor: pointer;
  }

  .checkbox-label {
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.75);
  }

  .summary-bar {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    margin-top: 1.5rem;
    padding: 0.65rem 0.85rem;
    border-radius: 10px;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.14);
  }

  .summary-item {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.75);
  }

  .summary-divider {
    color: rgba(78, 205, 196, 0.4);
  }

  /* Mobile: stack columns vertically */
  @media (max-width: 768px) {
    .two-column-layout {
      grid-template-columns: 1fr;
      gap: 1.5rem;
    }

    .summary-bar {
      flex-wrap: wrap;
      justify-content: center;
    }

    .preset-buttons {
      flex-wrap: wrap;
    }
  }
</style>
