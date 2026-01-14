<script lang="ts">
  import { onMount } from 'svelte';
  import { ipc } from '$lib/ipc';

  let specs = [];
  let loading = true;
  let error = null;
  let selectedSpec = null;

  onMount(async () => {
    await loadSpecs();
  });

  async function loadSpecs() {
    try {
      loading = true;
      specs = await ipc.invoke('spec:list', {});
      error = null;
    } catch (err) {
      error = err.message || 'Failed to load specs';
      console.error('Failed to load specs:', err);
    } finally {
      loading = false;
    }
  }

  async function viewSpec(spec) {
    try {
      const result = await ipc.invoke('spec:read', { path: spec.path });
      selectedSpec = {
        ...spec,
        content: result.content,
        metadata: result.metadata
      };
    } catch (err) {
      console.error('Failed to load spec content:', err);
    }
  }

  function closeSpecView() {
    selectedSpec = null;
  }

  function getStatusColor(status) {
    switch (status) {
      case 'completed': return '#22c55e';
      case 'in-progress': return '#f59e0b';
      case 'planned': return '#6b7280';
      case 'blocked': return '#ef4444';
      default: return '#6b7280';
    }
  }
</script>

<div class="spec-browser">
  <div class="spec-header">
    <h1>Specification Browser</h1>
    <p class="subtitle">Browse and manage development specifications</p>
    <button class="btn btn--secondary" on:click={loadSpecs}>
      Refresh
    </button>
  </div>

  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading specifications...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <p>Error loading specs: {error}</p>
      <button class="btn btn--primary" on:click={loadSpecs}>
        Try Again
      </button>
    </div>
  {:else if specs.length === 0}
    <div class="empty-state">
      <h2>No Specifications Found</h2>
      <p>No specification files were found in the project.</p>
    </div>
  {:else}
    <div class="spec-grid">
      {#each specs as spec}
        <div class="spec-card">
          <div class="spec-header-info">
            <div class="spec-title">
              <h3>{spec.title || spec.name}</h3>
              <span 
                class="status-badge" 
                style="background-color: {getStatusColor(spec.status)}"
              >
                {spec.status || 'unknown'}
              </span>
            </div>
            {#if spec.phase}
              <div class="spec-phase">Phase {spec.phase}</div>
            {/if}
          </div>

          <div class="spec-content">
            <p class="spec-description">
              {spec.description || 'No description available'}
            </p>
            
            {#if spec.dependencies && spec.dependencies.length > 0}
              <div class="spec-dependencies">
                <strong>Dependencies:</strong>
                <div class="dependency-list">
                  {#each spec.dependencies as dep}
                    <span class="dependency">{dep}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <div class="spec-actions">
            <button class="btn btn--secondary btn--sm" on:click={() => viewSpec(spec)}>
              View Details
            </button>
            <a href="/mission?spec={encodeURIComponent(spec.path)}" class="btn btn--primary btn--sm">
              Start Mission
            </a>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if selectedSpec}
  <!-- Spec Detail Modal -->
  <div class="modal-overlay" on:click={closeSpecView}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h2>{selectedSpec.title || selectedSpec.name}</h2>
        <button class="close-btn" on:click={closeSpecView}>×</button>
      </div>

      <div class="modal-body">
        <div class="spec-metadata">
          <div class="metadata-item">
            <strong>Status:</strong>
            <span 
              class="status-badge" 
              style="background-color: {getStatusColor(selectedSpec.status)}"
            >
              {selectedSpec.status || 'unknown'}
            </span>
          </div>
          {#if selectedSpec.metadata.phase}
            <div class="metadata-item">
              <strong>Phase:</strong> {selectedSpec.metadata.phase}
            </div>
          {/if}
          <div class="metadata-item">
            <strong>Path:</strong> 
            <code>{selectedSpec.path}</code>
          </div>
        </div>

        <div class="spec-content-view">
          <h3>Content</h3>
          <div class="content-preview">
            {selectedSpec.content}
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn--secondary" on:click={closeSpecView}>
          Close
        </button>
        <a 
          href="/mission?spec={encodeURIComponent(selectedSpec.path)}" 
          class="btn btn--primary"
        >
          Start Mission with this Spec
        </a>
      </div>
    </div>
  </div>
{/if}

<style>
  .spec-browser {
    max-width: 1200px;
    margin: 0 auto;
  }

  .spec-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
  }

  .spec-header h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 600;
    color: var(--text);
  }

  .subtitle {
    margin: 0.5rem 0 0 0;
    color: var(--text-muted);
    font-size: 1rem;
  }

  .loading-state, .error-state, .empty-state {
    text-align: center;
    padding: 4rem 2rem;
    color: var(--text-muted);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--border);
    border-top: 4px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 1rem;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .error-state p {
    color: #ef4444;
    margin-bottom: 1rem;
  }

  .spec-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .spec-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    transition: all 0.2s ease;
  }

  .spec-card:hover {
    border-color: var(--accent);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .spec-header-info {
    margin-bottom: 1rem;
  }

  .spec-title {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }

  .spec-title h3 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
    flex: 1;
  }

  .status-badge {
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .spec-phase {
    font-size: 0.875rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .spec-content {
    margin-bottom: 1.5rem;
  }

  .spec-description {
    color: var(--text-muted);
    line-height: 1.5;
    margin: 0 0 1rem 0;
  }

  .spec-dependencies {
    font-size: 0.875rem;
  }

  .spec-dependencies strong {
    color: var(--text);
    display: block;
    margin-bottom: 0.5rem;
  }

  .dependency-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .dependency {
    background: var(--bg);
    color: var(--text-muted);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    border: 1px solid var(--border);
  }

  .spec-actions {
    display: flex;
    gap: 0.75rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    text-decoration: none;
    display: inline-block;
    text-align: center;
    transition: all 0.2s ease;
  }

  .btn--primary {
    background: var(--accent);
    color: white;
  }

  .btn--primary:hover {
    background: var(--accent-hover);
  }

  .btn--secondary {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn--secondary:hover {
    background: var(--bg);
  }

  .btn--sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
  }

  /* Modal Styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }

  .modal-content {
    background: var(--bg-secondary);
    border-radius: 12px;
    width: 100%;
    max-width: 800px;
    max-height: 90vh;
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.5rem;
    color: var(--text);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.25rem;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text);
  }

  .modal-body {
    padding: 1.5rem;
    max-height: 60vh;
    overflow-y: auto;
  }

  .spec-metadata {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: var(--bg);
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .metadata-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .metadata-item strong {
    color: var(--text);
  }

  .metadata-item code {
    background: var(--bg-secondary);
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-size: 0.75rem;
    border: 1px solid var(--border);
  }

  .content-preview {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 1rem;
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.75rem;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-x: auto;
    max-height: 400px;
    overflow-y: auto;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1.5rem;
    border-top: 1px solid var(--border);
  }

  @media (max-width: 768px) {
    .spec-grid {
      grid-template-columns: 1fr;
    }

    .spec-header {
      flex-direction: column;
      gap: 1rem;
      align-items: flex-start;
    }

    .spec-title {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.5rem;
    }

    .spec-actions {
      flex-direction: column;
    }

    .modal-content {
      margin: 1rem;
      max-height: calc(100vh - 2rem);
    }
  }
</style>