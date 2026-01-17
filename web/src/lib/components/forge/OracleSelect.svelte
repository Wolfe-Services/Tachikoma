<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
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
          <div class="oracle-title">
            <div
              class="oracle-icon"
              style="--oracle-accent: {oracle.type === 'llm' ? 'rgba(88, 166, 255, 0.9)' : oracle.type === 'consensus' ? 'rgba(63, 185, 80, 0.9)' : 'rgba(78, 205, 196, 0.9)'}"
            >
              <Icon
                name={oracle.type === 'llm' ? 'brain' : oracle.type === 'consensus' ? 'check-circle' : 'refresh-cw'}
                size={18}
                glow
              />
            </div>
            <h3 class="oracle-name">{oracle.name}</h3>
          </div>
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
            <Icon name="alert-triangle" size={14} />
            <span>Also a participant</span>
          </div>
        {/if}

        {#if selected?.id === oracle.id}
          <div class="selected-indicator" aria-hidden="true">
            <Icon name="check-circle" size={16} glow />
            <span>Selected</span>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  {#if selected}
    <div class="selection-summary">
      <div class="summary-header">
        <div class="summary-title">
          <Icon name="brain" size={16} glow />
          <h3>Selected Oracle</h3>
        </div>
        <div class="summary-pill">{selected.name}</div>
      </div>
      <p class="summary-text">
        Estimated <strong>${selected.estimatedCostPerRound.toFixed(4)} / round</strong> •
        Fit: <strong>{getRecommendation(selected).toLowerCase()}</strong>
      </p>
    </div>
  {/if}
</div>

<style>
  .oracle-select {
    max-width: 960px;
    margin: 0 auto;
  }

  .step-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 1px;
    text-transform: uppercase;
  }

  .step-description {
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    margin-bottom: 2rem;
    line-height: 1.5;
  }

  .error-messages {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: rgba(255, 107, 107, 0.08);
    border: 1px solid rgba(255, 107, 107, 0.25);
    border-radius: 12px;
  }

  .error-message {
    color: rgba(230, 237, 243, 0.85);
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
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 12px;
    background:
      linear-gradient(135deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.01)),
      rgba(13, 17, 23, 0.35);
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.25) inset,
      0 12px 35px rgba(0, 0, 0, 0.25);
    -webkit-backdrop-filter: blur(12px) saturate(1.1);
    backdrop-filter: blur(12px) saturate(1.1);
  }

  .oracle-card:hover {
    border-color: rgba(78, 205, 196, 0.5);
    transform: translateY(-2px);
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.25) inset,
      0 18px 50px rgba(0, 0, 0, 0.3),
      0 0 22px rgba(78, 205, 196, 0.12);
  }

  .oracle-card.selected {
    border-color: rgba(78, 205, 196, 0.75);
    box-shadow:
      0 0 0 3px rgba(78, 205, 196, 0.14),
      0 18px 55px rgba(0, 0, 0, 0.35);
  }

  .oracle-card.incompatible {
    opacity: 0.7;
    border-color: rgba(255, 217, 61, 0.5);
  }

  .oracle-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .oracle-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
  }

  .oracle-icon {
    width: 38px;
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.18), rgba(78, 205, 196, 0.05));
    border: 1px solid rgba(78, 205, 196, 0.22);
    color: var(--oracle-accent, rgba(78, 205, 196, 0.9));
    flex-shrink: 0;
  }

  .oracle-name {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .oracle-type {
    background: rgba(13, 17, 23, 0.35);
    color: rgba(230, 237, 243, 0.65);
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
    border: 1px solid rgba(78, 205, 196, 0.14);
    letter-spacing: 1px;
    font-family: var(--font-display, 'Orbitron', sans-serif);
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
    color: rgba(230, 237, 243, 0.65);
    font-size: 0.875rem;
  }

  .cost-value {
    font-weight: 600;
    color: rgba(230, 237, 243, 0.9);
    font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
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
    height: 10px;
    background: rgba(78, 205, 196, 0.1);
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 999px;
    overflow: hidden;
  }

  .score-fill {
    height: 100%;
    background: linear-gradient(90deg, rgba(45, 122, 122, 0.9), rgba(78, 205, 196, 0.95));
    transition: width 0.3s ease;
    box-shadow: 0 0 18px rgba(78, 205, 196, 0.25);
  }

  .score-text {
    font-size: 0.875rem;
    font-weight: 500;
    color: rgba(230, 237, 243, 0.7);
    min-width: 3rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 1px;
  }

  .recommendation {
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.9px;
    border: 1px solid rgba(78, 205, 196, 0.14);
    background: rgba(13, 17, 23, 0.25);
    color: rgba(230, 237, 243, 0.7);
    border-radius: 999px;
    padding: 0.25rem 0.65rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
  }

  .recommendation-high {
    border-color: rgba(63, 185, 80, 0.35);
    color: rgba(63, 185, 80, 0.95);
    background: rgba(63, 185, 80, 0.08);
  }

  .recommendation-good {
    border-color: rgba(78, 205, 196, 0.35);
    color: rgba(78, 205, 196, 0.95);
    background: rgba(78, 205, 196, 0.08);
  }

  .recommendation-fair {
    border-color: rgba(255, 217, 61, 0.35);
    color: rgba(255, 217, 61, 0.95);
    background: rgba(255, 217, 61, 0.08);
  }

  .recommendation-low {
    border-color: rgba(255, 107, 107, 0.35);
    color: rgba(255, 107, 107, 0.95);
    background: rgba(255, 107, 107, 0.08);
  }

  .oracle-config details {
    margin-top: 0.5rem;
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 12px;
    background: rgba(13, 17, 23, 0.18);
    padding: 0.65rem 0.75rem;
  }

  .oracle-config summary {
    font-size: 0.875rem;
    color: rgba(230, 237, 243, 0.75);
    cursor: pointer;
    user-select: none;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 0.8px;
    text-transform: uppercase;
    font-size: 0.75rem;
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
    color: rgba(230, 237, 243, 0.65);
    text-transform: capitalize;
  }

  .config-value {
    color: rgba(230, 237, 243, 0.9);
    font-weight: 500;
  }

  .warning-badge {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: rgba(255, 217, 61, 0.08);
    border: 1px solid rgba(255, 217, 61, 0.22);
    color: rgba(255, 217, 61, 0.95);
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 0.7px;
    font-size: 0.65rem;
    padding: 0.3rem 0.55rem;
    border-radius: 999px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .selected-indicator {
    position: absolute;
    bottom: 0.75rem;
    right: 0.75rem;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: rgba(78, 205, 196, 0.12);
    border: 1px solid rgba(78, 205, 196, 0.22);
    color: rgba(78, 205, 196, 0.95);
    font-size: 0.65rem;
    padding: 0.3rem 0.6rem;
    border-radius: 999px;
    font-weight: 700;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 0.8px;
    text-transform: uppercase;
  }

  .selection-summary {
    padding: 1rem 1.1rem;
    background: rgba(13, 17, 23, 0.25);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 14px;
  }

  .summary-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }

  .summary-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: rgba(78, 205, 196, 0.95);
  }

  .summary-title h3 {
    margin: 0;
    font-size: 0.8rem;
    letter-spacing: 1px;
    text-transform: uppercase;
    font-family: var(--font-display, 'Orbitron', sans-serif);
  }

  .summary-pill {
    padding: 0.25rem 0.6rem;
    border-radius: 999px;
    border: 1px solid rgba(78, 205, 196, 0.18);
    background: rgba(78, 205, 196, 0.08);
    color: rgba(230, 237, 243, 0.85);
    font-size: 0.8rem;
    white-space: nowrap;
  }

  .summary-text {
    margin: 0;
    color: rgba(230, 237, 243, 0.7);
  }

  .oracle-card:focus-visible {
    outline: 2px solid rgba(78, 205, 196, 0.85);
    outline-offset: 2px;
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