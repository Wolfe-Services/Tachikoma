# Spec 564: Missions Route Page

## Header
- **Spec ID**: 564
- **Phase**: 26 - Hotfix Critical UI
- **Priority**: P0 - CRITICAL
- **Dependencies**: 562
- **Estimated Time**: 20 minutes

## Objective
Create the /missions route that shows mission history and allows starting new missions.

## Acceptance Criteria
- [x] File `web/src/routes/missions/+page.svelte` exists
- [x] Page shows list of past missions (or empty state)
- [x] "New Mission" button opens mission creation flow
- [x] Uses existing mission components from `$lib/components/mission/`

## Implementation

### Create missions folder and page
```bash
mkdir -p web/src/routes/missions
```

### missions/+page.svelte
```svelte
<script lang="ts">
  import HistoryView from '$lib/components/mission/HistoryView.svelte';
  
  let missions: any[] = [];
</script>

<div class="missions-page">
  <header class="page-header">
    <h1>Missions</h1>
    <a href="/missions/new" class="btn-primary">
      + New Mission
    </a>
  </header>
  
  {#if missions.length > 0}
    <HistoryView {missions} />
  {:else}
    <div class="empty-state">
      <div class="empty-icon">🚀</div>
      <h2>No missions yet</h2>
      <p>Start your first mission to let Tachikoma implement specs automatically.</p>
      <a href="/missions/new" class="btn-primary">Start First Mission</a>
    </div>
  {/if}
</div>

<style>
  .missions-page {
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }
  
  .page-header h1 {
    font-size: 1.75rem;
    margin: 0;
  }
  
  .btn-primary {
    background: var(--accent-primary);
    color: white;
    padding: 0.75rem 1.25rem;
    border-radius: 8px;
    text-decoration: none;
    font-weight: 500;
    transition: opacity 0.2s;
  }
  
  .btn-primary:hover {
    opacity: 0.9;
  }
  
  .empty-state {
    text-align: center;
    padding: 4rem 2rem;
    background: var(--bg-secondary);
    border-radius: 12px;
  }
  
  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }
  
  .empty-state h2 {
    margin: 0 0 0.5rem 0;
  }
  
  .empty-state p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
  }
</style>
```

## Verification
Navigate to /missions - should show empty state or mission list
