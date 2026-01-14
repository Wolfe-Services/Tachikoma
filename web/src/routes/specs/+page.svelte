<script lang="ts">
  import PageHeader from '$lib/components/common/PageHeader.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import { onMount } from 'svelte';
  import { ipc } from '$lib/ipc';
  
  interface Spec {
    id: number;
    name: string;
    phase: number;
    status: 'complete' | 'in-progress' | 'pending';
    file?: string;
    description?: string;
  }
  
  let specs: Spec[] = [];
  let selectedSpec: Spec | null = null;
  let searchQuery = '';
  let filterPhase = 'all';
  let filterStatus = 'all';
  let viewMode: 'grid' | 'list' = 'list';
  
  // Phase metadata for Section 9 terminology
  const phases = [
    { id: 0, name: 'SETUP', color: '#58a6ff' },
    { id: 1, name: 'COMMON', color: '#a371f7' },
    { id: 2, name: 'PRIMITIVES', color: '#4ecdc4' },
    { id: 3, name: 'BACKENDS', color: '#ffd93d' },
    { id: 4, name: 'CLI', color: '#ff6b6b' },
    { id: 5, name: 'LOOP', color: '#3fb950' },
    { id: 6, name: 'SPECS', color: '#f778ba' },
    { id: 7, name: 'FORGE', color: '#ff9f43' },
  ];
  
  onMount(async () => {
    try {
      specs = await ipc.invoke('spec:list', {});
    } catch (e) {
      console.log('Could not load specs:', e);
      // Mock data for development
      specs = [
        { id: 1, name: 'Project Structure', phase: 0, status: 'complete', description: 'Initialize project scaffold and directory structure' },
        { id: 2, name: 'Rust Workspace', phase: 0, status: 'complete', description: 'Setup Cargo workspace with crate organization' },
        { id: 3, name: 'Electron Shell', phase: 0, status: 'in-progress', description: 'Main process and window management' },
        { id: 4, name: 'Svelte Integration', phase: 0, status: 'pending', description: 'SvelteKit frontend with Vite' },
        { id: 5, name: 'IPC Bridge', phase: 0, status: 'pending', description: 'Native bindings between Rust and Electron' },
        { id: 31, name: 'Primitives Crate', phase: 2, status: 'complete', description: 'Core primitive operations setup' },
        { id: 32, name: 'read_file Implementation', phase: 2, status: 'complete', description: 'File reading primitive' },
        { id: 56, name: 'Claude API Client', phase: 3, status: 'in-progress', description: 'Anthropic Claude integration' },
      ];
    }
  });

  function selectSpec(spec: Spec) {
    selectedSpec = selectedSpec?.id === spec.id ? null : spec;
  }
  
  function getPhaseColor(phaseId: number): string {
    return phases.find(p => p.id === phaseId)?.color || '#4ecdc4';
  }

  function getPhaseName(phaseId: number): string {
    return phases.find(p => p.id === phaseId)?.name || `PHASE ${phaseId}`;
  }

  $: filteredSpecs = specs.filter(spec => {
    const matchesSearch = spec.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesPhase = filterPhase === 'all' || spec.phase === parseInt(filterPhase);
    const matchesStatus = filterStatus === 'all' || spec.status === filterStatus;
    return matchesSearch && matchesPhase && matchesStatus;
  });
  
  $: groupedSpecs = filteredSpecs.reduce((acc, spec) => {
    if (!acc[spec.phase]) acc[spec.phase] = [];
    acc[spec.phase].push(spec);
    return acc;
  }, {} as Record<number, Spec[]>);
  
  $: stats = {
    total: specs.length,
    complete: specs.filter(s => s.status === 'complete').length,
    inProgress: specs.filter(s => s.status === 'in-progress').length,
    pending: specs.filter(s => s.status === 'pending').length,
  };
</script>

<div class="specs-page">
  <PageHeader 
    title="SPEC REGISTRY"
    subtitle="Mission specifications and implementation plans"
    tag="INTELLIGENCE DATABASE"
    icon="file-text"
  >
    <svelte:fragment slot="actions">
      <a href="/forge" class="btn-secondary">
        <Icon name="brain" size={16} />
        <span>Create Spec</span>
      </a>
    </svelte:fragment>
  </PageHeader>
  
  <!-- Stats Bar -->
  <div class="stats-bar">
    <div class="stat-item total">
      <span class="stat-value">{stats.total}</span>
      <span class="stat-label">TOTAL</span>
    </div>
    <div class="stat-divider"></div>
    <div class="stat-item complete">
      <span class="stat-value">{stats.complete}</span>
      <span class="stat-label">COMPLETE</span>
    </div>
    <div class="stat-item progress">
      <span class="stat-value">{stats.inProgress}</span>
      <span class="stat-label">IN PROGRESS</span>
    </div>
    <div class="stat-item pending">
      <span class="stat-value">{stats.pending}</span>
      <span class="stat-label">PENDING</span>
    </div>
    <div class="stat-bar-visual">
      <div class="bar-segment complete" style="width: {(stats.complete / stats.total) * 100}%"></div>
      <div class="bar-segment progress" style="width: {(stats.inProgress / stats.total) * 100}%"></div>
      <div class="bar-segment pending" style="width: {(stats.pending / stats.total) * 100}%"></div>
    </div>
  </div>
  
  <!-- Search & Filters -->
  <div class="controls-bar">
    <div class="search-box">
      <Icon name="search" size={18} />
      <input 
        type="text" 
        placeholder="Search specifications..." 
        bind:value={searchQuery}
      />
    </div>
    
    <div class="filters">
      <select bind:value={filterPhase}>
        <option value="all">All Phases</option>
        {#each phases as phase}
          <option value={phase.id}>Phase {phase.id}: {phase.name}</option>
        {/each}
      </select>
      
      <select bind:value={filterStatus}>
        <option value="all">All Status</option>
        <option value="complete">Complete</option>
        <option value="in-progress">In Progress</option>
        <option value="pending">Pending</option>
      </select>
      
      <div class="view-toggle">
        <button class:active={viewMode === 'list'} on:click={() => viewMode = 'list'}>
          <Icon name="menu" size={16} />
        </button>
        <button class:active={viewMode === 'grid'} on:click={() => viewMode = 'grid'}>
          <Icon name="cpu" size={16} />
        </button>
      </div>
    </div>
  </div>
  
  <!-- Spec List -->
  <div class="specs-container" class:grid-view={viewMode === 'grid'}>
    {#each Object.entries(groupedSpecs).sort(([a], [b]) => parseInt(a) - parseInt(b)) as [phaseId, phaseSpecs]}
      <div class="phase-group">
        <div class="phase-header" style="--phase-color: {getPhaseColor(parseInt(phaseId))}">
          <div class="phase-marker"></div>
          <span class="phase-number">PHASE {phaseId}</span>
          <span class="phase-name">{getPhaseName(parseInt(phaseId))}</span>
          <span class="phase-count">{phaseSpecs.length} specs</span>
        </div>
        
        <div class="specs-list" class:grid={viewMode === 'grid'}>
          {#each phaseSpecs as spec}
            <button 
              class="spec-card"
              class:selected={selectedSpec?.id === spec.id}
              class:complete={spec.status === 'complete'}
              class:in-progress={spec.status === 'in-progress'}
              on:click={() => selectSpec(spec)}
              style="--phase-color: {getPhaseColor(spec.phase)}"
            >
              <div class="spec-status-indicator">
                {#if spec.status === 'complete'}
                  <Icon name="check-circle" size={18} />
                {:else if spec.status === 'in-progress'}
                  <div class="status-spinner"></div>
                {:else}
                  <div class="status-pending"></div>
                {/if}
              </div>
              
              <div class="spec-info">
                <span class="spec-id">#{String(spec.id).padStart(3, '0')}</span>
                <span class="spec-name">{spec.name}</span>
                {#if spec.description && viewMode === 'list'}
                  <span class="spec-desc">{spec.description}</span>
                {/if}
              </div>
              
              <div class="spec-actions">
                <div class="spec-badge" class:complete={spec.status === 'complete'} class:progress={spec.status === 'in-progress'}>
                  {spec.status.toUpperCase().replace('-', ' ')}
                </div>
                <Icon name="chevron-right" size={16} />
              </div>
            </button>
          {/each}
        </div>
      </div>
    {/each}
    
    {#if filteredSpecs.length === 0}
      <div class="no-results">
        <Icon name="search" size={32} />
        <span>No specifications match your search</span>
      </div>
    {/if}
  </div>
  
  <!-- Spec Detail Panel -->
  {#if selectedSpec}
    <div class="spec-detail-panel">
      <div class="detail-header">
        <div class="detail-meta">
          <span class="detail-phase" style="color: {getPhaseColor(selectedSpec.phase)}">
            PHASE {selectedSpec.phase}
          </span>
          <span class="detail-id">SPEC #{String(selectedSpec.id).padStart(3, '0')}</span>
        </div>
        <button class="close-btn" on:click={() => selectedSpec = null}>
          <Icon name="x" size={18} />
        </button>
      </div>
      
      <h2 class="detail-title">{selectedSpec.name}</h2>
      
      {#if selectedSpec.description}
        <p class="detail-desc">{selectedSpec.description}</p>
      {/if}
      
      <div class="detail-status">
        <span class="status-label">STATUS</span>
        <div class="status-badge" class:complete={selectedSpec.status === 'complete'} class:progress={selectedSpec.status === 'in-progress'}>
          {#if selectedSpec.status === 'complete'}
            <Icon name="check-circle" size={14} />
          {:else if selectedSpec.status === 'in-progress'}
            <Icon name="loader" size={14} />
          {:else}
            <Icon name="clock" size={14} />
          {/if}
          <span>{selectedSpec.status.toUpperCase().replace('-', ' ')}</span>
        </div>
      </div>
      
      <div class="detail-actions">
        <a href="/missions/new?spec={selectedSpec.id}" class="btn-primary">
          <Icon name="play" size={16} />
          <span>DEPLOY</span>
        </a>
        <button class="btn-secondary">
          <Icon name="eye" size={16} />
          <span>View Full Spec</span>
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .specs-page {
    max-width: 1400px;
    margin: 0 auto;
    position: relative;
  }
  
  /* Stats Bar */
  .stats-bar {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1rem 1.25rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
  }
  
  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }
  
  .stat-value {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
  }
  
  .stat-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 1px;
  }
  
  .stat-item.total .stat-value { color: var(--tachi-cyan, #4ecdc4); }
  .stat-item.complete .stat-value { color: var(--success-color, #3fb950); }
  .stat-item.progress .stat-value { color: var(--warning-color, #ffd93d); }
  .stat-item.pending .stat-value { color: var(--text-muted, rgba(230, 237, 243, 0.5)); }
  
  .stat-divider {
    width: 1px;
    height: 30px;
    background: var(--border-color, rgba(78, 205, 196, 0.2));
  }
  
  .stat-bar-visual {
    flex: 1;
    min-width: 200px;
    height: 6px;
    background: var(--bg-tertiary, #1c2128);
    border-radius: 3px;
    display: flex;
    overflow: hidden;
  }
  
  .bar-segment {
    height: 100%;
    transition: width 0.5s ease;
  }
  
  .bar-segment.complete { background: var(--success-color, #3fb950); }
  .bar-segment.progress { background: var(--warning-color, #ffd93d); }
  .bar-segment.pending { background: var(--text-muted, rgba(230, 237, 243, 0.3)); }
  
  /* Controls */
  .controls-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
  }
  
  .search-box {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    min-width: 250px;
    max-width: 400px;
    padding: 0.625rem 1rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 8px;
    transition: all 0.2s ease;
  }
  
  .search-box:focus-within {
    border-color: var(--tachi-cyan, #4ecdc4);
    box-shadow: 0 0 10px rgba(78, 205, 196, 0.2);
  }
  
  .search-box :global(svg) {
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .search-box input {
    flex: 1;
    background: transparent !important;
    border: none !important;
    color: var(--text-primary, #e6edf3) !important;
    font-size: 0.9rem;
    outline: none;
  }
  
  .filters {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .filters select {
    padding: 0.5rem 2rem 0.5rem 0.75rem;
    background: var(--bg-secondary, #161b22) !important;
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2)) !important;
    border-radius: 6px;
    color: var(--text-primary, #e6edf3) !important;
    font-size: 0.85rem;
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%234ecdc4' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E") !important;
    background-repeat: no-repeat !important;
    background-position: right 0.75rem center !important;
  }
  
  .view-toggle {
    display: flex;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 6px;
    overflow: hidden;
  }
  
  .view-toggle button {
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .view-toggle button:hover {
    color: var(--text-primary, #e6edf3);
  }
  
  .view-toggle button.active {
    background: rgba(78, 205, 196, 0.15);
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  /* Phase Groups */
  .specs-container {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding-right: 0;
    transition: padding-right 0.3s ease;
  }
  
  .specs-container:has(~ .spec-detail-panel) {
    padding-right: 380px;
  }
  
  .phase-group {
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    overflow: hidden;
  }
  
  .phase-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.875rem 1.25rem;
    background: linear-gradient(90deg, rgba(78, 205, 196, 0.08), transparent);
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
  }
  
  .phase-marker {
    width: 4px;
    height: 20px;
    background: var(--phase-color);
    border-radius: 2px;
    box-shadow: 0 0 8px var(--phase-color);
  }
  
  .phase-number {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--phase-color);
    letter-spacing: 1px;
  }
  
  .phase-name {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    letter-spacing: 1px;
  }
  
  .phase-count {
    margin-left: auto;
    font-size: 0.75rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  /* Spec Cards */
  .specs-list {
    padding: 0.5rem;
  }
  
  .specs-list.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.75rem;
  }
  
  .spec-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    width: 100%;
    padding: 1rem 1.25rem;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
    position: relative;
  }
  
  .spec-card::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--phase-color);
    opacity: 0;
    transition: opacity 0.2s ease;
    border-radius: 3px 0 0 3px;
  }
  
  .spec-card:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    border-color: var(--border-color, rgba(78, 205, 196, 0.2));
  }
  
  .spec-card:hover::before {
    opacity: 1;
  }
  
  .spec-card.selected {
    background: rgba(78, 205, 196, 0.1);
    border-color: var(--tachi-cyan, #4ecdc4);
  }
  
  .spec-card.selected::before {
    opacity: 1;
    background: var(--tachi-cyan, #4ecdc4);
  }
  
  .spec-status-indicator {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .spec-card.complete .spec-status-indicator {
    color: var(--success-color, #3fb950);
  }
  
  .spec-card.in-progress .spec-status-indicator {
    color: var(--warning-color, #ffd93d);
  }
  
  .status-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 217, 61, 0.3);
    border-top-color: var(--warning-color, #ffd93d);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .status-pending {
    width: 12px;
    height: 12px;
    border: 2px solid var(--text-muted, rgba(230, 237, 243, 0.4));
    border-radius: 50%;
  }
  
  .spec-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    min-width: 0;
  }
  
  .spec-id {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 0.5px;
    opacity: 0.8;
  }
  
  .spec-name {
    font-family: var(--font-body, 'Rajdhani', sans-serif);
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .spec-desc {
    font-size: 0.8rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .spec-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
  }
  
  .spec-badge {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.55rem;
    font-weight: 600;
    padding: 0.25rem 0.5rem;
    background: var(--bg-secondary, #161b22);
    border-radius: 4px;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 0.5px;
  }
  
  .spec-badge.complete {
    background: rgba(63, 185, 80, 0.15);
    color: var(--success-color, #3fb950);
  }
  
  .spec-badge.progress {
    background: rgba(255, 217, 61, 0.15);
    color: var(--warning-color, #ffd93d);
  }
  
  .spec-card :global(svg:last-child) {
    color: var(--text-muted, rgba(230, 237, 243, 0.3));
    transition: all 0.2s ease;
  }
  
  .spec-card:hover :global(svg:last-child) {
    color: var(--tachi-cyan, #4ecdc4);
    transform: translateX(2px);
  }
  
  /* No Results */
  .no-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 4rem 2rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    text-align: center;
  }
  
  /* Detail Panel */
  .spec-detail-panel {
    position: fixed;
    top: 48px; /* Header height */
    right: 0;
    width: 360px;
    height: calc(100vh - 48px);
    background: var(--bg-secondary, #161b22);
    border-left: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    z-index: 50;
    animation: slideIn 0.3s ease;
    overflow-y: auto;
  }
  
  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
  
  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }
  
  .detail-meta {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .detail-phase {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    font-weight: 600;
    letter-spacing: 1px;
  }
  
  .detail-id {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 1px;
  }
  
  .close-btn {
    padding: 0.5rem;
    background: transparent;
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 6px;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .close-btn:hover {
    background: rgba(255, 107, 107, 0.1);
    border-color: var(--tachi-red, #ff6b6b);
    color: var(--tachi-red, #ff6b6b);
  }
  
  .detail-title {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
    margin: 0;
  }
  
  .detail-desc {
    font-size: 0.9rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    line-height: 1.6;
    margin: 0;
  }
  
  .detail-status {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .status-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 1px;
  }
  
  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.875rem;
    background: var(--bg-tertiary, #1c2128);
    border-radius: 6px;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--text-muted, rgba(230, 237, 243, 0.6));
    letter-spacing: 0.5px;
    width: fit-content;
  }
  
  .status-badge.complete {
    background: rgba(63, 185, 80, 0.15);
    color: var(--success-color, #3fb950);
  }
  
  .status-badge.progress {
    background: rgba(255, 217, 61, 0.15);
    color: var(--warning-color, #ffd93d);
  }
  
  .detail-actions {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: auto;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
  }
  
  .detail-actions .btn-primary,
  .detail-actions .btn-secondary {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-decoration: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .detail-actions .btn-primary {
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    border: 1px solid var(--tachi-cyan, #4ecdc4);
    color: var(--bg-primary, #0d1117);
  }
  
  .detail-actions .btn-primary:hover {
    box-shadow: 0 0 20px rgba(78, 205, 196, 0.4);
    transform: translateY(-1px);
  }
  
  .detail-actions .btn-secondary {
    background: transparent;
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.3));
    color: var(--text-primary, #e6edf3);
  }
  
  .detail-actions .btn-secondary:hover {
    background: rgba(78, 205, 196, 0.1);
    border-color: var(--tachi-cyan, #4ecdc4);
  }
</style>
