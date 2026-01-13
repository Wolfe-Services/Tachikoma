<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Participant } from '$lib/types/forge';

  export let selected: Participant[] = [];
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    change: Participant[];
  }>();

  // Mock available participants - in real implementation would come from API
  const availableParticipants: Participant[] = [
    {
      id: 'human-1',
      name: 'Alice Johnson',
      type: 'human',
      role: 'Product Manager',
      status: 'active',
      avatar: '👩‍💼',
      estimatedCostPerRound: 0
    },
    {
      id: 'human-2', 
      name: 'Bob Smith',
      type: 'human',
      role: 'Engineering Lead',
      status: 'active',
      avatar: '👨‍💻',
      estimatedCostPerRound: 0
    },
    {
      id: 'ai-1',
      name: 'Claude Sonnet',
      type: 'ai',
      role: 'Strategic Advisor',
      status: 'active',
      avatar: '🤖',
      estimatedCostPerRound: 0.05
    },
    {
      id: 'ai-2',
      name: 'GPT-4',
      type: 'ai', 
      role: 'Creative Ideator',
      status: 'active',
      avatar: '🧠',
      estimatedCostPerRound: 0.08
    },
    {
      id: 'ai-3',
      name: 'Gemini Pro',
      type: 'ai',
      role: 'Technical Analyst', 
      status: 'active',
      avatar: '💎',
      estimatedCostPerRound: 0.06
    },
    {
      id: 'human-3',
      name: 'Carol Davis',
      type: 'human',
      role: 'UX Designer',
      status: 'active',
      avatar: '👩‍🎨',
      estimatedCostPerRound: 0
    }
  ];

  let searchQuery = '';
  let filterType: 'all' | 'human' | 'ai' = 'all';

  $: filteredParticipants = availableParticipants.filter(p => {
    const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         p.role.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFilter = filterType === 'all' || p.type === filterType;
    return matchesSearch && matchesFilter;
  });

  $: selectedIds = new Set(selected.map(p => p.id));

  function toggleParticipant(participant: Participant) {
    let newSelected: Participant[];
    
    if (selectedIds.has(participant.id)) {
      newSelected = selected.filter(p => p.id !== participant.id);
    } else {
      newSelected = [...selected, participant];
    }
    
    dispatch('change', newSelected);
  }

  function removeParticipant(participantId: string) {
    const newSelected = selected.filter(p => p.id !== participantId);
    dispatch('change', newSelected);
  }

  function clearSearch() {
    searchQuery = '';
  }

  $: totalCost = selected.reduce((sum, p) => sum + (p.estimatedCostPerRound || 0), 0);
</script>

<div class="participant-select-step" data-testid="participant-select-step">
  <div class="step-header">
    <h2>Select Participants</h2>
    <p class="step-description">
      Choose who will participate in this deliberation session. You need at least 2 participants for meaningful discussion.
    </p>
  </div>

  <div class="selected-section">
    <h3>Selected Participants ({selected.length})</h3>
    {#if selected.length > 0}
      <div class="selected-list">
        {#each selected as participant (participant.id)}
          <div class="selected-item" data-testid="selected-participant-{participant.id}">
            <div class="participant-avatar">
              {participant.avatar || '👤'}
            </div>
            <div class="participant-info">
              <div class="participant-name">{participant.name}</div>
              <div class="participant-role">{participant.role}</div>
            </div>
            <div class="participant-meta">
              <span class="participant-type" class:human={participant.type === 'human'} class:ai={participant.type === 'ai'}>
                {participant.type.toUpperCase()}
              </span>
              {#if participant.estimatedCostPerRound}
                <span class="participant-cost">${participant.estimatedCostPerRound.toFixed(3)}/round</span>
              {/if}
            </div>
            <button
              type="button"
              class="remove-btn"
              on:click={() => removeParticipant(participant.id)}
              aria-label="Remove {participant.name}"
            >
              ×
            </button>
          </div>
        {/each}
        
        {#if totalCost > 0}
          <div class="cost-summary">
            <span class="cost-label">Total estimated cost per round:</span>
            <span class="cost-value">${totalCost.toFixed(3)}</span>
          </div>
        {/if}
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">👥</div>
        <p>No participants selected yet</p>
        <p class="empty-hint">Choose from the available participants below</p>
      </div>
    {/if}
  </div>

  <div class="search-section">
    <div class="search-controls">
      <div class="search-input-container">
        <input
          type="text"
          class="search-input"
          placeholder="Search participants by name or role..."
          bind:value={searchQuery}
          data-testid="participant-search"
        />
        {#if searchQuery}
          <button 
            type="button" 
            class="search-clear"
            on:click={clearSearch}
            aria-label="Clear search"
          >
            ×
          </button>
        {/if}
      </div>

      <div class="filter-controls">
        <button
          type="button"
          class="filter-btn"
          class:active={filterType === 'all'}
          on:click={() => filterType = 'all'}
        >
          All
        </button>
        <button
          type="button"
          class="filter-btn"
          class:active={filterType === 'human'}
          on:click={() => filterType = 'human'}
        >
          Human
        </button>
        <button
          type="button"
          class="filter-btn"
          class:active={filterType === 'ai'}
          on:click={() => filterType = 'ai'}
        >
          AI
        </button>
      </div>
    </div>
  </div>

  <div class="available-section">
    <h3>Available Participants</h3>
    {#if filteredParticipants.length > 0}
      <div class="participant-grid">
        {#each filteredParticipants as participant (participant.id)}
          <button
            type="button"
            class="participant-card"
            class:selected={selectedIds.has(participant.id)}
            on:click={() => toggleParticipant(participant)}
            data-testid="participant-{participant.id}"
          >
            <div class="card-header">
              <div class="participant-avatar">
                {participant.avatar || '👤'}
              </div>
              <div class="participant-type" class:human={participant.type === 'human'} class:ai={participant.type === 'ai'}>
                {participant.type.toUpperCase()}
              </div>
            </div>
            <div class="card-body">
              <div class="participant-name">{participant.name}</div>
              <div class="participant-role">{participant.role}</div>
              {#if participant.estimatedCostPerRound}
                <div class="participant-cost">${participant.estimatedCostPerRound.toFixed(3)}/round</div>
              {/if}
            </div>
            {#if selectedIds.has(participant.id)}
              <div class="selected-indicator">✓</div>
            {/if}
          </button>
        {/each}
      </div>
    {:else}
      <div class="no-results">
        <div class="no-results-icon">🔍</div>
        <p>No participants found</p>
        <p class="no-results-hint">Try adjusting your search or filter</p>
      </div>
    {/if}
  </div>

  {#if errors.length > 0}
    <div class="error-list" role="alert">
      {#each errors as error}
        <div class="error-item">
          <span class="error-icon">⚠️</span>
          {error}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .participant-select-step {
    max-width: 800px;
    margin: 0 auto;
  }

  .step-header {
    margin-bottom: 2rem;
  }

  .step-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--text-primary);
  }

  .step-description {
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .selected-section,
  .available-section {
    margin-bottom: 2rem;
  }

  .selected-section h3,
  .available-section h3 {
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 1rem;
    color: var(--text-primary);
  }

  .selected-list {
    border: 2px dashed var(--border-color);
    border-radius: 8px;
    padding: 1rem;
    background: var(--secondary-bg);
  }

  .selected-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem;
    background: white;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  .selected-item:last-child {
    margin-bottom: 0;
  }

  .participant-avatar {
    font-size: 1.5rem;
    width: 2.5rem;
    height: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--avatar-bg, #f3f4f6);
    border-radius: 50%;
  }

  .participant-info {
    flex: 1;
  }

  .participant-name {
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 0.25rem;
  }

  .participant-role {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .participant-meta {
    display: flex;
    flex-direction: column;
    align-items: end;
    gap: 0.25rem;
  }

  .participant-type {
    font-size: 0.75rem;
    font-weight: 500;
    padding: 0.125rem 0.5rem;
    border-radius: 12px;
    text-transform: uppercase;
  }

  .participant-type.human {
    background: var(--human-bg, #ecfdf5);
    color: var(--human-color, #059669);
  }

  .participant-type.ai {
    background: var(--ai-bg, #eff6ff);
    color: var(--ai-color, #2563eb);
  }

  .participant-cost {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: monospace;
  }

  .remove-btn {
    width: 1.5rem;
    height: 1.5rem;
    border: none;
    background: var(--error-bg, #fef2f2);
    color: var(--error-color, #dc2626);
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    transition: background-color 0.15s ease;
  }

  .remove-btn:hover {
    background: var(--error-color, #dc2626);
    color: white;
  }

  .cost-summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-color);
    font-weight: 500;
  }

  .cost-label {
    color: var(--text-secondary);
  }

  .cost-value {
    color: var(--text-primary);
    font-family: monospace;
  }

  .empty-state,
  .no-results {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .empty-icon,
  .no-results-icon {
    font-size: 2rem;
    margin-bottom: 1rem;
  }

  .empty-hint,
  .no-results-hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .search-section {
    margin-bottom: 1.5rem;
  }

  .search-controls {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .search-input-container {
    position: relative;
    flex: 1;
    min-width: 250px;
  }

  .search-input {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    font-size: 0.875rem;
    background: white;
    color: var(--text-primary);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px var(--primary-color-alpha, rgba(59, 130, 246, 0.1));
  }

  .search-clear {
    position: absolute;
    right: 0.5rem;
    top: 50%;
    transform: translateY(-50%);
    width: 1.5rem;
    height: 1.5rem;
    border: none;
    background: var(--secondary-bg);
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    color: var(--text-muted);
  }

  .search-clear:hover {
    background: var(--border-color);
    color: var(--text-primary);
  }

  .filter-controls {
    display: flex;
    gap: 0.25rem;
  }

  .filter-btn {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    background: var(--secondary-bg);
    color: var(--text-secondary);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.15s ease;
  }

  .filter-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .filter-btn.active {
    background: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  .participant-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .participant-card {
    position: relative;
    padding: 1rem;
    border: 2px solid var(--border-color);
    border-radius: 8px;
    background: white;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }

  .participant-card:hover {
    border-color: var(--primary-color);
    box-shadow: 0 4px 12px var(--shadow-color, rgba(0, 0, 0, 0.1));
  }

  .participant-card.selected {
    border-color: var(--success-color, #10b981);
    background: var(--success-bg, #f0fdf4);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .card-body .participant-name {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .card-body .participant-role {
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
  }

  .card-body .participant-cost {
    font-size: 0.75rem;
    font-family: monospace;
  }

  .selected-indicator {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    width: 1.5rem;
    height: 1.5rem;
    background: var(--success-color, #10b981);
    color: white;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: bold;
  }

  .error-list {
    margin-top: 1rem;
  }

  .error-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: var(--error-bg, #fef2f2);
    color: var(--error-color, #dc2626);
    border-radius: 6px;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
  }

  .error-icon {
    flex-shrink: 0;
  }

  @media (max-width: 768px) {
    .search-controls {
      flex-direction: column;
      align-items: stretch;
    }

    .participant-grid {
      grid-template-columns: 1fr;
    }

    .selected-item {
      flex-wrap: wrap;
      gap: 0.5rem;
    }

    .participant-meta {
      flex-direction: row;
      align-items: center;
    }
  }
</style>