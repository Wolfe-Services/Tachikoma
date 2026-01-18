<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Participant } from '$lib/types/forge';

  export let selected: Participant[] = [];
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    change: Participant[];
  }>();

  const brokenAvatarIds = new Set<string>();

  function parseAvatar(avatar?: string): { src?: string; emoji?: string } {
    if (!avatar) return {};
    if (avatar.startsWith('asset:')) {
      const raw = avatar.slice('asset:'.length);
      const [src, emoji] = raw.split('|');
      return { src, emoji };
    }
    if (avatar.startsWith('/') || avatar.startsWith('http')) {
      return { src: avatar };
    }
    return { emoji: avatar };
  }

  function markAvatarBroken(id: string) {
    brokenAvatarIds.add(id);
  }

  // Mock available participants - in real implementation would come from API
  // Theme: Ghost in the Shell / Public Security Section 9
  // Icons from web/static/icons/ (Iconfactory GitS set)
  const modelOptions: Array<{ id: string; label: string }> = [
    { id: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4' },
    { id: 'claude-3-5-sonnet-20241022', label: 'Claude 3.5 Sonnet' },
    { id: 'gpt-4-turbo', label: 'GPT-4 Turbo' },
    { id: 'ollama/llama3:latest', label: 'Ollama Llama 3' },
  ];

  const availableParticipants: Participant[] = [
    {
      id: 'human-kusanagi',
      name: 'Major Motoko Kusanagi',
      type: 'human',
      role: 'Visionary / Systems Architect',
      status: 'active',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Motoko-Kusanagi.32.png|🕶️',
      estimatedCostPerRound: 0
    },
    {
      id: 'human-batou',
      name: 'Batou',
      type: 'human',
      role: 'Deployment Specialist / Reliability',
      status: 'active',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Bateau.32.png|🦾',
      estimatedCostPerRound: 0
    },
    {
      id: 'human-togusa',
      name: 'Togusa',
      type: 'human',
      role: 'QA Lead / Product Critic',
      status: 'active',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Togusa.32.png|🕵️',
      estimatedCostPerRound: 0
    },
    {
      id: 'human-aramaki',
      name: 'Daisuke Aramaki',
      type: 'human',
      role: 'Stakeholder / Strategy & Risk',
      status: 'active',
      avatar: '🎖️',
      estimatedCostPerRound: 0
    },
    {
      id: 'human-ishikawa',
      name: 'Ishikawa',
      type: 'human',
      role: 'Observability / Telemetry',
      status: 'active',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Ishikawa.32.png|📡',
      estimatedCostPerRound: 0
    },
    {
      id: 'human-saito',
      name: 'Saito',
      type: 'human',
      role: 'Security / Threat Modeling',
      status: 'active',
      avatar: '🎯',
      estimatedCostPerRound: 0
    },

    // AI participants (multi-agent think tanks)
    {
      id: 'ai-fuchikoma-blue',
      name: 'Fuchikoma Blue',
      type: 'ai',
      role: 'Implementation Agent',
      status: 'active',
      modelId: 'claude-sonnet-4-20250514',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Blue.32.png|🕷️',
      estimatedCostPerRound: 0.03
    },
    {
      id: 'ai-fuchikoma-red',
      name: 'Fuchikoma Red',
      type: 'ai',
      role: 'Testing & Validation Agent',
      status: 'active',
      modelId: 'gpt-4-turbo',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Red.32.png|🔴',
      estimatedCostPerRound: 0.03
    },
    {
      id: 'ai-fuchikoma-purple',
      name: 'Fuchikoma Purple',
      type: 'ai',
      role: 'Research & Analysis Agent',
      status: 'active',
      modelId: 'ollama/llama3:latest',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Purple.32.png|🟣',
      estimatedCostPerRound: 0.03
    },
    {
      id: 'ai-arakone',
      name: 'Arakone Unit',
      type: 'ai',
      role: 'Heavy Processing / Synthesis',
      status: 'active',
      modelId: 'claude-3-5-sonnet-20241022',
      avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Arakone-Unit.32.png|🦂',
      estimatedCostPerRound: 0.06
    },
    {
      id: 'ai-laughing-man',
      name: 'Laughing Man',
      type: 'ai',
      role: 'Adversarial Reviewer',
      status: 'active',
      modelId: 'gpt-4-turbo',
      avatar: '🌀',
      estimatedCostPerRound: 0.06
    },
    {
      id: 'ai-puppet-master',
      name: 'Puppet Master',
      type: 'ai',
      role: 'Orchestration & Arbitration',
      status: 'active',
      modelId: 'claude-sonnet-4-20250514',
      avatar: '🧬',
      estimatedCostPerRound: 0.08
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

  function setParticipantModel(participantId: string, modelId: string) {
    const newSelected = selected.map(p => (p.id === participantId ? { ...p, modelId } : p));
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
          {@const a = parseAvatar(participant.avatar)}
          <div class="selected-item" data-testid="selected-participant-{participant.id}">
            <div class="participant-avatar">
              {#if a.src && !brokenAvatarIds.has(participant.id)}
                <img
                  src={a.src}
                  alt=""
                  on:error={() => markAvatarBroken(participant.id)}
                />
              {:else}
                <span class="avatar-emoji" aria-hidden="true">{a.emoji || '👤'}</span>
              {/if}
            </div>
            <div class="participant-info">
              <div class="participant-name">{participant.name}</div>
              <div class="participant-role">{participant.role}</div>
            </div>
            <div class="participant-meta">
              <span class="participant-type" class:human={participant.type === 'human'} class:ai={participant.type === 'ai'}>
                {participant.type.toUpperCase()}
              </span>
              {#if participant.type === 'ai'}
                <select
                  class="model-select"
                  value={participant.modelId || 'claude-sonnet-4-20250514'}
                  on:change={(e) => setParticipantModel(participant.id, e.currentTarget.value)}
                  aria-label="Model for {participant.name}"
                >
                  {#each modelOptions as opt}
                    <option value={opt.id}>{opt.label}</option>
                  {/each}
                </select>
              {/if}
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
          {@const a = parseAvatar(participant.avatar)}
          <button
            type="button"
            class="participant-card"
            class:selected={selectedIds.has(participant.id)}
            on:click={() => toggleParticipant(participant)}
            data-testid="participant-{participant.id}"
          >
            <div class="card-header">
              <div class="participant-avatar">
                {#if a.src && !brokenAvatarIds.has(participant.id)}
                  <img
                    src={a.src}
                    alt=""
                    on:error={() => markAvatarBroken(participant.id)}
                  />
                {:else}
                  <span class="avatar-emoji" aria-hidden="true">{a.emoji || '👤'}</span>
                {/if}
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
    max-width: 960px;
    margin: 0 auto;
  }

  .step-header {
    margin-bottom: 2rem;
  }

  .step-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 1px;
    text-transform: uppercase;
  }

  .step-description {
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
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
    color: var(--text-primary, #e6edf3);
  }

  .selected-list {
    border: 1px dashed rgba(78, 205, 196, 0.28);
    border-radius: 14px;
    padding: 1rem;
    background: rgba(13, 17, 23, 0.22);
    -webkit-backdrop-filter: blur(12px) saturate(1.15);
    backdrop-filter: blur(12px) saturate(1.15);
  }

  .selected-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem;
    background: rgba(22, 27, 34, 0.45);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 12px;
    margin-bottom: 0.5rem;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.25) inset;
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
    background: rgba(13, 17, 23, 0.55);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 50%;
    color: var(--text-primary, #e6edf3);
    overflow: hidden;
  }

  .participant-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 50%;
    filter: saturate(1.05) contrast(1.05);
  }

  .avatar-emoji {
    line-height: 1;
  }

  .participant-info {
    flex: 1;
  }

  .participant-name {
    font-weight: 500;
    color: var(--text-primary, #e6edf3);
    margin-bottom: 0.25rem;
  }

  .participant-role {
    font-size: 0.875rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
  }

  .participant-meta {
    display: flex;
    flex-direction: column;
    align-items: end;
    gap: 0.25rem;
  }

  .model-select {
    width: 100%;
    max-width: 200px;
    padding: 0.35rem 0.5rem;
    border-radius: 10px;
    border: 1px solid rgba(78, 205, 196, 0.18);
    background: rgba(13, 17, 23, 0.45);
    color: rgba(230, 237, 243, 0.85);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .model-select:focus {
    outline: none;
    border-color: rgba(78, 205, 196, 0.5);
    box-shadow: 0 0 0 2px rgba(78, 205, 196, 0.1);
  }

  .participant-type {
    font-size: 0.75rem;
    font-weight: 500;
    padding: 0.125rem 0.5rem;
    border-radius: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    border: 1px solid rgba(78, 205, 196, 0.18);
    background: rgba(13, 17, 23, 0.35);
  }

  .participant-type.human {
    background: rgba(63, 185, 80, 0.12);
    color: rgba(63, 185, 80, 0.95);
    border-color: rgba(63, 185, 80, 0.35);
  }

  .participant-type.ai {
    background: rgba(88, 166, 255, 0.12);
    color: rgba(88, 166, 255, 0.95);
    border-color: rgba(88, 166, 255, 0.35);
  }

  .participant-cost {
    font-size: 0.75rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .remove-btn {
    width: 1.5rem;
    height: 1.5rem;
    border: none;
    background: rgba(255, 107, 107, 0.12);
    color: rgba(255, 107, 107, 0.95);
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    transition: background-color 0.15s ease;
  }

  .remove-btn:hover {
    background: rgba(255, 107, 107, 0.25);
    color: rgba(255, 255, 255, 0.9);
  }

  .cost-summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid rgba(78, 205, 196, 0.14);
    font-weight: 500;
  }

  .cost-label {
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
  }

  .cost-value {
    color: var(--text-primary, #e6edf3);
    font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .empty-state,
  .no-results {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.45));
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
    border-radius: 12px;
    font-size: 0.875rem;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.14);
    color: var(--text-primary, #e6edf3);
  }

  .search-input:focus {
    outline: none;
    border-color: rgba(78, 205, 196, 0.55);
    box-shadow: 0 0 0 3px rgba(78, 205, 196, 0.12);
  }

  .search-clear {
    position: absolute;
    right: 0.5rem;
    top: 50%;
    transform: translateY(-50%);
    width: 1.5rem;
    height: 1.5rem;
    border: none;
    background: rgba(13, 17, 23, 0.45);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }

  .search-clear:hover {
    background: rgba(78, 205, 196, 0.12);
    color: rgba(230, 237, 243, 0.85);
  }

  .filter-controls {
    display: flex;
    gap: 0.25rem;
  }

  .filter-btn {
    padding: 0.5rem 1rem;
    border: 1px solid rgba(78, 205, 196, 0.14);
    background: rgba(13, 17, 23, 0.25);
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    border-radius: 12px;
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.15s ease;
  }

  .filter-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary, #e6edf3);
    border-color: rgba(78, 205, 196, 0.28);
  }

  .filter-btn.active {
    background: linear-gradient(135deg, rgba(45, 122, 122, 0.9), rgba(78, 205, 196, 0.9));
    color: rgba(13, 17, 23, 0.95);
    border-color: rgba(78, 205, 196, 0.65);
    box-shadow: 0 0 18px rgba(78, 205, 196, 0.2);
  }

  .participant-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .participant-card {
    position: relative;
    padding: 1rem;
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 14px;
    background:
      linear-gradient(135deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.01)),
      rgba(13, 17, 23, 0.35);
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.25) inset,
      0 12px 35px rgba(0, 0, 0, 0.25);
    -webkit-backdrop-filter: blur(12px) saturate(1.1);
    backdrop-filter: blur(12px) saturate(1.1);
  }

  .participant-card:hover {
    border-color: rgba(78, 205, 196, 0.5);
    transform: translateY(-1px);
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.25) inset,
      0 18px 50px rgba(0, 0, 0, 0.3),
      0 0 22px rgba(78, 205, 196, 0.12);
  }

  .participant-card.selected {
    border-color: rgba(78, 205, 196, 0.75);
    box-shadow:
      0 0 0 3px rgba(78, 205, 196, 0.14),
      0 18px 55px rgba(0, 0, 0, 0.35);
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
    background: rgba(78, 205, 196, 0.9);
    color: rgba(13, 17, 23, 0.95);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: bold;
    box-shadow: 0 0 16px rgba(78, 205, 196, 0.25);
  }

  .error-list {
    margin-top: 1rem;
  }

  .error-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: rgba(255, 107, 107, 0.08);
    color: rgba(230, 237, 243, 0.85);
    border-radius: 12px;
    border: 1px solid rgba(255, 107, 107, 0.25);
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
  }

  .participant-card:focus-visible,
  .filter-btn:focus-visible,
  .remove-btn:focus-visible,
  .search-clear:focus-visible {
    outline: 2px solid rgba(78, 205, 196, 0.85);
    outline-offset: 2px;
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