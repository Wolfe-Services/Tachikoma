# Spec 563: Dashboard Home Page

## Header
- **Spec ID**: 563
- **Phase**: 26 - Hotfix Critical UI
- **Priority**: P0 - CRITICAL
- **Dependencies**: 562
- **Estimated Time**: 30 minutes

## Objective
Replace the test placeholder page with an actual dashboard showing project status, recent missions, and quick actions.

## Acceptance Criteria
- [ ] `web/src/routes/+page.svelte` shows dashboard content (not test buttons)
- [ ] Dashboard displays: Welcome message, Project stats, Recent activity
- [ ] Stats cards show: Specs progress, Active missions, Success rate
- [ ] Quick action buttons: New Mission, Browse Specs, Start Forge
- [ ] Responsive grid layout for cards

## Implementation

### Update +page.svelte
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { ipc } from '$lib/ipc';
  
  let stats = {
    specsTotal: 0,
    specsComplete: 0,
    activeMissions: 0,
    successRate: 0
  };
  
  let platform = 'unknown';
  
  onMount(async () => {
    if (typeof window !== 'undefined' && window.tachikoma) {
      platform = window.tachikoma.platform;
    }
    
    try {
      const config = await ipc.invoke('config:get', {});
      const specs = await ipc.invoke('spec:list', {});
      stats.specsTotal = specs.length || 0;
      stats.specsComplete = specs.filter((s: any) => s.complete).length || 0;
    } catch (e) {
      console.log('IPC not available:', e);
    }
  });
  
  $: specsProgress = stats.specsTotal > 0 
    ? Math.round((stats.specsComplete / stats.specsTotal) * 100) 
    : 0;
</script>

<div class="dashboard">
  <header class="dashboard-header">
    <h1>Welcome to Tachikoma</h1>
    <p class="subtitle">Your squad of tireless AI coders • Running on {platform}</p>
  </header>
  
  <section class="stats-grid">
    <div class="stat-card">
      <div class="stat-icon">📋</div>
      <div class="stat-content">
        <div class="stat-value">{stats.specsComplete}/{stats.specsTotal}</div>
        <div class="stat-label">Specs Complete</div>
        <div class="progress-bar">
          <div class="progress-fill" style="width: {specsProgress}%"></div>
        </div>
      </div>
    </div>
    
    <div class="stat-card">
      <div class="stat-icon">🚀</div>
      <div class="stat-content">
        <div class="stat-value">{stats.activeMissions}</div>
        <div class="stat-label">Active Missions</div>
      </div>
    </div>
    
    <div class="stat-card">
      <div class="stat-icon">✅</div>
      <div class="stat-content">
        <div class="stat-value">{stats.successRate}%</div>
        <div class="stat-label">Success Rate</div>
      </div>
    </div>
  </section>
  
  <section class="quick-actions">
    <h2>Quick Actions</h2>
    <div class="action-grid">
      <a href="/missions/new" class="action-card">
        <span class="action-icon">▶️</span>
        <span class="action-label">New Mission</span>
      </a>
      <a href="/specs" class="action-card">
        <span class="action-icon">📄</span>
        <span class="action-label">Browse Specs</span>
      </a>
      <a href="/forge" class="action-card">
        <span class="action-icon">🔥</span>
        <span class="action-label">Start Forge</span>
      </a>
      <a href="/settings" class="action-card">
        <span class="action-icon">⚙️</span>
        <span class="action-label">Settings</span>
      </a>
    </div>
  </section>
</div>

<style>
  .dashboard {
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .dashboard-header {
    margin-bottom: 2rem;
  }
  
  .dashboard-header h1 {
    font-size: 2rem;
    margin: 0 0 0.5rem 0;
  }
  
  .subtitle {
    color: var(--text-secondary);
    margin: 0;
  }
  
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }
  
  .stat-card {
    background: var(--bg-secondary);
    border-radius: 12px;
    padding: 1.5rem;
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }
  
  .stat-icon {
    font-size: 2rem;
  }
  
  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
  }
  
  .stat-label {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }
  
  .progress-bar {
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    margin-top: 0.5rem;
    overflow: hidden;
  }
  
  .progress-fill {
    height: 100%;
    background: var(--accent-primary);
    transition: width 0.3s ease;
  }
  
  .quick-actions h2 {
    font-size: 1.25rem;
    margin-bottom: 1rem;
  }
  
  .action-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
  }
  
  .action-card {
    background: var(--bg-secondary);
    border-radius: 12px;
    padding: 1.5rem;
    text-align: center;
    text-decoration: none;
    color: var(--text-primary);
    transition: all 0.2s ease;
    border: 1px solid transparent;
  }
  
  .action-card:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent-primary);
    transform: translateY(-2px);
  }
  
  .action-icon {
    font-size: 2rem;
    display: block;
    margin-bottom: 0.5rem;
  }
  
  .action-label {
    font-weight: 500;
  }
</style>
```

## Verification
1. Run `npm run dev`
2. Dashboard shows stats cards and quick actions
3. No "Test IPC Bridge" button visible
4. Quick action links navigate correctly
