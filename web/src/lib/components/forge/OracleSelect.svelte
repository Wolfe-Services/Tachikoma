<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Oracle, Participant } from '$lib/types/forge';

  export let selected: Oracle | null = null;
  export let participants: Participant[] = [];
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    change: Oracle | null;
  }>();

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
      name: 'GPT-4 Oracle',
      type: 'llm',
      config: { model: 'gpt-4', temperature: 0.3 },
      estimatedCostPerRound: 0.02
    },
    {
      id: 'claude-oracle-1',
      name: 'Claude Oracle',
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

  function handleSelect(oracle: Oracle) {
    selected = oracle;
    dispatch('change', oracle);
  }

  function getCompatibilityScore(oracle: Oracle): number {
    // Simple compatibility scoring based on participant types and count
    const aiParticipants = participants.filter(p => p.type === 'ai').length;
    const humanParticipants = participants.filter(p => p.type === 'human').length;
    
    switch (oracle.type) {
      case 'consensus':
        // Better with more participants
        return Math.min(0.9, 0.6 + (participants.length * 0.05));
      case 'llm':
        // Better with AI participants
        return Math.min(0.95, 0.7 + (aiParticipants * 0.1));
      case 'hybrid':
        // Balanced approach, good with mixed groups
        return Math.min(0.9, 0.8 + (Math.min(aiParticipants, humanParticipants) * 0.05));
      default:
        return 0.5;
    }
  }

  function getRecommendation(oracle: Oracle): string {
    const score = getCompatibilityScore(oracle);
    if (score >= 0.8) return 'Highly Recommended';
    if (score >= 0.6) return 'Good Fit';
    if (score >= 0.4) return 'May Work';
    return 'Not Recommended';
  }

  function getRecommendationClass(oracle: Oracle): string {
    const score = getCompatibilityScore(oracle);
    if (score >= 0.8) return 'recommendation-high';
    if (score >= 0.6) return 'recommendation-good';
    if (score >= 0.4) return 'recommendation-fair';
    return 'recommendation-low';
  }

  function isParticipantOracle(oracle: Oracle): boolean {
    return participants.some(p => p.id === oracle.id);
  }

  $: sortedOracles = availableOracles
    .map(oracle => ({
      ...oracle,
      compatibility: getCompatibilityScore(oracle),
      recommendation: getRecommendation(oracle)
    }))
    .sort((a, b) => b.compatibility - a.compatibility);
</script>

<div class="oracle-select" data-testid="oracle-select">
  <div class="step-header">
    <h2>Choose Oracle</h2>
    <p class="step-description">
      The oracle will evaluate contributions and guide the deliberation process toward consensus.
    </p>
  </div>

  {#if errors.length > 0}
    <div class="error-messages" role="alert">
      {#each errors as error}
        <p class="error-message">{error}</p>
      {/each}
    </div>
  {/if}

  <div class="oracle-grid">
    {#each sortedOracles as oracle}
      <div
        class="oracle-card"
        class:selected={selected?.id === oracle.id}
        class:incompatible={isParticipantOracle(oracle)}
        role="button"
        tabindex="0"
        aria-pressed={selected?.id === oracle.id}
        data-testid="oracle-option-{oracle.id}"
        on:click={() => handleSelect(oracle)}
        on:keydown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleSelect(oracle);
          }
        }}
      >
        <div class="oracle-header">
          <h3 class="oracle-name">{oracle.name}</h3>
          <div class="oracle-type">{oracle.type}</div>
        </div>

        <div class="oracle-details">
          <div class="cost-info">
            <span class="cost-label">Cost per round:</span>
            <span class="cost-value">${oracle.estimatedCostPerRound.toFixed(4)}</span>
          </div>

          <div class="compatibility-info">
            <div class="compatibility-score">
              <div class="score-bar">
                <div
                  class="score-fill"
                  style="width: {oracle.compatibility * 100}%"
                ></div>
              </div>
              <span class="score-text">{Math.round(oracle.compatibility * 100)}%</span>
            </div>
            <div class="recommendation {getRecommendationClass(oracle)}">
              {oracle.recommendation}
            </div>
          </div>

          <div class="oracle-config">
            <details>
              <summary>Configuration</summary>
              <ul class="config-list">
                {#each Object.entries(oracle.config) as [key, value]}
                  <li>
                    <span class="config-key">{key.replace(/_/g, ' ')}:</span>
                    <span class="config-value">{value}</span>
                  </li>
                {/each}
              </ul>
            </details>
          </div>
        </div>

        {#if isParticipantOracle(oracle)}
          <div class="warning-badge">
            ⚠️ Also a participant
          </div>
        {/if}

        {#if selected?.id === oracle.id}
          <div class="selected-indicator" aria-hidden="true">
            ✓ Selected
          </div>
        {/if}
      </div>
    {/each}
  </div>

  {#if selected}
    <div class="selection-summary">
      <h3>Selected Oracle: {selected.name}</h3>
      <p>This oracle will cost approximately <strong>${selected.estimatedCostPerRound.toFixed(4)} per round</strong> and is rated as <strong>{getRecommendation(selected).toLowerCase()}</strong> for your participant configuration.</p>
    </div>
  {/if}
</div>

<style>
  .oracle-select {
    max-width: 800px;
  }

  .step-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .step-description {
    color: var(--text-secondary);
    margin-bottom: 2rem;
    line-height: 1.5;
  }

  .error-messages {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: var(--error-bg, #fef2f2);
    border: 1px solid var(--error-border, #fecaca);
    border-radius: 6px;
  }

  .error-message {
    color: var(--error-color, #dc2626);
    font-size: 0.875rem;
    margin: 0;
  }

  .oracle-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .oracle-card {
    padding: 1.5rem;
    border: 2px solid var(--border-color, #e5e7eb);
    border-radius: 12px;
    background: var(--card-bg, #ffffff);
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
  }

  .oracle-card:hover {
    border-color: var(--primary-color, #3b82f6);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .oracle-card.selected {
    border-color: var(--primary-color, #3b82f6);
    background: var(--primary-bg-light, #f0f7ff);
  }

  .oracle-card.incompatible {
    opacity: 0.7;
    border-color: var(--warning-color, #f59e0b);
  }

  .oracle-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .oracle-name {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .oracle-type {
    background: var(--secondary-bg, #f3f4f6);
    color: var(--text-secondary);
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
  }

  .oracle-details {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .cost-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .cost-label {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .cost-value {
    font-weight: 600;
    color: var(--text-primary);
  }

  .compatibility-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .compatibility-score {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .score-bar {
    flex: 1;
    height: 6px;
    background: var(--border-color, #e5e7eb);
    border-radius: 3px;
    overflow: hidden;
  }

  .score-fill {
    height: 100%;
    background: var(--success-color, #10b981);
    transition: width 0.3s ease;
  }

  .score-text {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
    min-width: 3rem;
  }

  .recommendation {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .recommendation-high {
    color: var(--success-color, #10b981);
  }

  .recommendation-good {
    color: var(--primary-color, #3b82f6);
  }

  .recommendation-fair {
    color: var(--warning-color, #f59e0b);
  }

  .recommendation-low {
    color: var(--error-color, #ef4444);
  }

  .oracle-config details {
    margin-top: 0.5rem;
  }

  .oracle-config summary {
    font-size: 0.875rem;
    color: var(--text-secondary);
    cursor: pointer;
    user-select: none;
  }

  .config-list {
    list-style: none;
    margin: 0.5rem 0 0 0;
    padding: 0;
    font-size: 0.75rem;
  }

  .config-list li {
    display: flex;
    justify-content: space-between;
    padding: 0.25rem 0;
  }

  .config-key {
    color: var(--text-secondary);
    text-transform: capitalize;
  }

  .config-value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .warning-badge {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    background: var(--warning-color, #f59e0b);
    color: white;
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-weight: 500;
  }

  .selected-indicator {
    position: absolute;
    bottom: 0.75rem;
    right: 0.75rem;
    background: var(--success-color, #10b981);
    color: white;
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-weight: 500;
  }

  .selection-summary {
    padding: 1.5rem;
    background: var(--info-bg, #f0f9ff);
    border: 1px solid var(--info-border, #bae6fd);
    border-radius: 8px;
  }

  .selection-summary h3 {
    font-size: 1.125rem;
    margin: 0 0 0.5rem 0;
    color: var(--text-primary);
  }

  .selection-summary p {
    margin: 0;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  /* Focus styles for accessibility */
  .oracle-card:focus {
    outline: 2px solid var(--primary-color, #3b82f6);
    outline-offset: 2px;
  }
</style>