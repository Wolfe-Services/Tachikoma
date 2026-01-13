<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SessionConfig } from '$lib/types/forge';

  export let config: SessionConfig;
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    change: SessionConfig;
  }>();

  function updateConfig(field: keyof SessionConfig, value: unknown) {
    const updatedConfig = { ...config };
    updatedConfig[field] = value as never;
    config = updatedConfig;
    dispatch('change', updatedConfig);
  }

  function formatTime(minutes: number): string {
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
  }
</script>

<div class="session-config" data-testid="session-config">
  <div class="config-header">
    <h2>Session Configuration</h2>
    <p class="config-description">
      Configure the parameters that will govern your deliberation session.
    </p>
  </div>

  {#if errors.length > 0}
    <div class="error-list" role="alert">
      {#each errors as error}
        <p class="error-message">{error}</p>
      {/each}
    </div>
  {/if}

  <div class="config-form">
    <div class="form-group">
      <label for="maxRounds" class="form-label">
        Maximum Rounds
        <span class="help-text">How many deliberation rounds to allow before concluding</span>
      </label>
      <div class="input-group">
        <input
          id="maxRounds"
          type="range"
          min="1"
          max="20"
          step="1"
          value={config.maxRounds}
          on:input={(e) => updateConfig('maxRounds', parseInt(e.currentTarget.value))}
          class="range-input"
        />
        <span class="input-value">{config.maxRounds} rounds</span>
      </div>
      <div class="range-labels">
        <span>1</span>
        <span>20</span>
      </div>
    </div>

    <div class="form-group">
      <label for="convergenceThreshold" class="form-label">
        Convergence Threshold
        <span class="help-text">How much agreement is needed to conclude (0.5 = 50%, 1.0 = 100%)</span>
      </label>
      <div class="input-group">
        <input
          id="convergenceThreshold"
          type="range"
          min="0.5"
          max="1.0"
          step="0.05"
          value={config.convergenceThreshold}
          on:input={(e) => updateConfig('convergenceThreshold', parseFloat(e.currentTarget.value))}
          class="range-input"
        />
        <span class="input-value">{Math.round(config.convergenceThreshold * 100)}%</span>
      </div>
      <div class="range-labels">
        <span>50%</span>
        <span>100%</span>
      </div>
    </div>

    <div class="form-group">
      <label for="timeoutMinutes" class="form-label">
        Session Timeout
        <span class="help-text">Maximum time allowed for the entire session</span>
      </label>
      <div class="input-group">
        <input
          id="timeoutMinutes"
          type="range"
          min="5"
          max="480"
          step="5"
          value={config.timeoutMinutes}
          on:input={(e) => updateConfig('timeoutMinutes', parseInt(e.currentTarget.value))}
          class="range-input"
        />
        <span class="input-value">{formatTime(config.timeoutMinutes)}</span>
      </div>
      <div class="range-labels">
        <span>5m</span>
        <span>8h</span>
      </div>
    </div>

    <div class="form-group">
      <label for="autoSaveInterval" class="form-label">
        Auto-save Interval
        <span class="help-text">How often to automatically save session progress</span>
      </label>
      <select
        id="autoSaveInterval"
        value={config.autoSaveInterval}
        on:change={(e) => updateConfig('autoSaveInterval', parseInt(e.currentTarget.value))}
        class="select-input"
      >
        <option value={15000}>Every 15 seconds</option>
        <option value={30000}>Every 30 seconds</option>
        <option value={60000}>Every minute</option>
        <option value={300000}>Every 5 minutes</option>
        <option value={0}>Disabled</option>
      </select>
    </div>

    <div class="form-group">
      <label class="checkbox-label">
        <input
          type="checkbox"
          checked={config.allowHumanIntervention}
          on:change={(e) => updateConfig('allowHumanIntervention', e.currentTarget.checked)}
          class="checkbox-input"
        />
        <span class="checkbox-text">
          Allow Human Intervention
          <span class="help-text">Permit humans to interject during AI-driven deliberations</span>
        </span>
      </label>
    </div>
  </div>

  <div class="config-preview">
    <h3>Configuration Summary</h3>
    <div class="preview-grid">
      <div class="preview-item">
        <span class="preview-label">Max Rounds:</span>
        <span class="preview-value">{config.maxRounds}</span>
      </div>
      <div class="preview-item">
        <span class="preview-label">Convergence:</span>
        <span class="preview-value">{Math.round(config.convergenceThreshold * 100)}%</span>
      </div>
      <div class="preview-item">
        <span class="preview-label">Timeout:</span>
        <span class="preview-value">{formatTime(config.timeoutMinutes)}</span>
      </div>
      <div class="preview-item">
        <span class="preview-label">Auto-save:</span>
        <span class="preview-value">
          {config.autoSaveInterval === 0 ? 'Disabled' : `${config.autoSaveInterval / 1000}s`}
        </span>
      </div>
      <div class="preview-item">
        <span class="preview-label">Human Intervention:</span>
        <span class="preview-value">{config.allowHumanIntervention ? 'Allowed' : 'Disabled'}</span>
      </div>
    </div>
  </div>
</div>

<style>
  .session-config {
    max-width: 600px;
    margin: 0 auto;
  }

  .config-header {
    margin-bottom: 2rem;
    text-align: center;
  }

  .config-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .config-description {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .error-list {
    background: var(--error-bg, #fef2f2);
    border: 1px solid var(--error-border, #fecaca);
    border-radius: 6px;
    padding: 1rem;
    margin-bottom: 1.5rem;
  }

  .error-message {
    color: var(--error-text, #dc2626);
    font-size: 0.875rem;
    margin: 0;
  }

  .error-message:not(:last-child) {
    margin-bottom: 0.5rem;
  }

  .config-form {
    space-y: 2rem;
  }

  .form-group {
    margin-bottom: 2rem;
  }

  .form-label {
    display: block;
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 0.5rem;
  }

  .help-text {
    display: block;
    font-weight: normal;
    color: var(--text-secondary);
    font-size: 0.75rem;
    margin-top: 0.25rem;
  }

  .input-group {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }

  .range-input {
    flex: 1;
    height: 6px;
    background: var(--secondary-bg);
    border-radius: 3px;
    outline: none;
    cursor: pointer;
    -webkit-appearance: none;
  }

  .range-input::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    background: var(--primary-color);
    border-radius: 50%;
    cursor: pointer;
  }

  .range-input::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: var(--primary-color);
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .input-value {
    min-width: 80px;
    text-align: right;
    font-weight: 600;
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .range-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .select-input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg, white);
    color: var(--text-primary);
    font-size: 0.875rem;
    cursor: pointer;
  }

  .select-input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px var(--primary-color-alpha, rgba(59, 130, 246, 0.1));
  }

  .checkbox-label {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    cursor: pointer;
  }

  .checkbox-input {
    margin-top: 0.125rem;
    width: 1rem;
    height: 1rem;
    accent-color: var(--primary-color);
    cursor: pointer;
  }

  .checkbox-text {
    flex: 1;
    font-weight: 500;
    color: var(--text-primary);
  }

  .config-preview {
    margin-top: 3rem;
    padding: 1.5rem;
    background: var(--secondary-bg);
    border-radius: 8px;
    border: 1px solid var(--border-color);
  }

  .config-preview h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1rem;
    color: var(--text-primary);
  }

  .preview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 0.75rem;
  }

  .preview-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-color);
  }

  .preview-item:last-child {
    border-bottom: none;
  }

  .preview-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .preview-value {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  @media (max-width: 640px) {
    .config-form {
      padding: 0 1rem;
    }

    .input-group {
      flex-direction: column;
      align-items: stretch;
      gap: 0.5rem;
    }

    .input-value {
      text-align: left;
      min-width: auto;
    }

    .preview-grid {
      grid-template-columns: 1fr;
    }
  }
</style>