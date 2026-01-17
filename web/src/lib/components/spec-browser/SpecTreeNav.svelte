<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { ipc } from '$lib/ipc';

  export let selectedId: string | null = null;

  const dispatch = createEventDispatcher<{
    select: string;
  }>();

  let specs: any[] = [];
  let loading = true;

  onMount(async () => {
    try {
      specs = await ipc.invoke('spec:list', {});
    } catch (e) {
      console.log('Could not load specs:', e);
      // Mock data for dev
      specs = [
        { id: '1', name: 'Project Structure', phase: 0, status: 'complete' },
        { id: '2', name: 'Rust Workspace', phase: 0, status: 'complete' },
        { id: '3', name: 'Electron Shell', phase: 0, status: 'in-progress' },
        { id: '4', name: 'Spec Browser', phase: 26, status: 'in-progress' },
      ];
    } finally {
      loading = false;
    }
  });

  function selectSpec(specId: string) {
    dispatch('select', specId);
  }
</script>

<div class="spec-tree-nav">
  <div class="spec-tree-header">
    <h3>Specifications</h3>
  </div>

  <div class="spec-tree-content">
    {#if loading}
      <div class="loading">Loading specs...</div>
    {:else if specs.length === 0}
      <div class="empty">No specs found</div>
    {:else}
      <nav class="spec-tree">
        {#each specs as spec}
          <button 
            class="spec-item"
            class:active={selectedId === spec.id}
            on:click={() => selectSpec(spec.id)}
          >
            <span class="spec-status" class:complete={spec.status === 'complete'}>
              {spec.status === 'complete' ? '✓' : '○'}
            </span>
            <span class="spec-name">{spec.name}</span>
            <span class="spec-phase">#{spec.phase}</span>
          </button>
        {/each}
      </nav>
    {/if}
  </div>
</div>

<style>
  .spec-tree-nav {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .spec-tree-header {
    padding: 16px;
    border-bottom: 1px solid var(--color-border-default);
  }

  .spec-tree-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-default);
  }

  .spec-tree-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .loading, .empty {
    padding: 20px;
    text-align: center;
    color: var(--color-fg-muted);
    font-size: 13px;
  }

  .spec-tree {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .spec-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--color-fg-default);
    text-align: left;
    cursor: pointer;
    border-radius: 6px;
    transition: background 0.15s;
    font-size: 13px;
  }

  .spec-item:hover {
    background: var(--color-bg-hover);
  }

  .spec-item.active {
    background: var(--color-accent-subtle);
    color: var(--color-accent-fg);
  }

  .spec-status {
    color: var(--color-fg-muted);
    font-size: 12px;
    width: 12px;
    text-align: center;
  }

  .spec-status.complete {
    color: var(--color-success-fg);
  }

  .spec-name {
    flex: 1;
  }

  .spec-phase {
    color: var(--color-fg-muted);
    font-size: 11px;
  }
</style>