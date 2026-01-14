# Spec 565: Specs Browser Route

## Header
- **Spec ID**: 565
- **Phase**: 26 - Hotfix Critical UI
- **Priority**: P0 - CRITICAL
- **Dependencies**: 562
- **Estimated Time**: 20 minutes

## Objective
Create the /specs route that shows the spec browser with tree navigation.

## Acceptance Criteria
- [ ] File `web/src/routes/specs/+page.svelte` exists
- [ ] Page uses SpecBrowserLayout component
- [ ] Shows spec tree navigation on left
- [ ] Shows spec content on right
- [ ] Search functionality works

## Implementation

### Create specs folder and page
```bash
mkdir -p web/src/routes/specs
```

### specs/+page.svelte
```svelte
<script lang="ts">
  import SpecBrowserLayout from '$lib/components/spec-browser/SpecBrowserLayout.svelte';
  import SpecSearchUI from '$lib/components/spec-browser/SpecSearchUI.svelte';
  import { onMount } from 'svelte';
  import { ipc } from '$lib/ipc';
  
  let specs: any[] = [];
  let selectedSpec: any = null;
  let searchQuery = '';
  
  onMount(async () => {
    try {
      specs = await ipc.invoke('spec:list', {});
    } catch (e) {
      console.log('Could not load specs:', e);
      // Mock data for dev
      specs = [
        { id: 1, name: 'Project Structure', phase: 0, status: 'complete' },
        { id: 2, name: 'Rust Workspace', phase: 0, status: 'complete' },
        { id: 3, name: 'Electron Shell', phase: 0, status: 'in-progress' },
      ];
    }
  });
  
  function selectSpec(spec: any) {
    selectedSpec = spec;
  }
</script>

<div class="specs-page">
  <aside class="specs-sidebar">
    <div class="search-container">
      <SpecSearchUI bind:query={searchQuery} />
    </div>
    
    <nav class="spec-tree">
      {#each specs as spec}
        <button 
          class="spec-item"
          class:active={selectedSpec?.id === spec.id}
          on:click={() => selectSpec(spec)}
        >
          <span class="spec-status" class:complete={spec.status === 'complete'}>
            {spec.status === 'complete' ? '✓' : '○'}
          </span>
          <span class="spec-name">{spec.name}</span>
        </button>
      {/each}
    </nav>
  </aside>
  
  <main class="spec-content">
    {#if selectedSpec}
      <h1>{selectedSpec.name}</h1>
      <p>Spec #{selectedSpec.id} • Phase {selectedSpec.phase}</p>
      <!-- Spec content would load here -->
    {:else}
      <div class="select-prompt">
        <p>Select a spec from the sidebar to view details</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .specs-page {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 1.5rem;
    height: calc(100vh - 3rem);
  }
  
  .specs-sidebar {
    background: var(--bg-secondary);
    border-radius: 12px;
    padding: 1rem;
    overflow-y: auto;
  }
  
  .search-container {
    margin-bottom: 1rem;
  }
  
  .spec-tree {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .spec-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    background: transparent;
    border: none;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    border-radius: 6px;
    transition: background 0.15s;
  }
  
  .spec-item:hover {
    background: var(--bg-tertiary);
  }
  
  .spec-item.active {
    background: rgba(59, 130, 246, 0.2);
  }
  
  .spec-status {
    color: var(--text-muted);
  }
  
  .spec-status.complete {
    color: var(--accent-success);
  }
  
  .spec-content {
    background: var(--bg-secondary);
    border-radius: 12px;
    padding: 1.5rem;
    overflow-y: auto;
  }
  
  .select-prompt {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }
</style>
```

## Verification
Navigate to /specs - should show spec browser layout
