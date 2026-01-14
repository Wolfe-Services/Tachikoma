<script lang="ts">
  import { missionStore, isRunning, progress } from '$lib/stores/mission';
  import { onMount } from 'svelte';
  import MissionComparison from '$lib/components/mission/MissionComparison.svelte';
  import ContextMeter from '$lib/components/mission/ContextMeter.svelte';
  import CostTracking from '$lib/components/mission/CostTracking.svelte';
  import HistoryView from '$lib/components/mission/HistoryView.svelte';
  import { ipc } from '$lib/ipc';
  
  let activeMission = null;
  let recentMissions = [];
  let systemStatus = 'Ready';
  let showQuickTest = false;
  let testResults: string[] = [];

  onMount(async () => {
    // Load recent mission history
    try {
      // For now, mock some data - this will be replaced with real data later
      recentMissions = [
        {
          id: 'mission-001',
          title: 'Update API Documentation',
          status: 'completed',
          progress: 100,
          startedAt: new Date(Date.now() - 3600000).toISOString(),
          duration: 2400000,
          cost: 0.0234
        },
        {
          id: 'mission-002',  
          title: 'Fix Authentication Bug',
          status: 'running',
          progress: 67,
          startedAt: new Date(Date.now() - 1800000).toISOString(),
          duration: 1800000,
          cost: 0.0156
        }
      ];
    } catch (error) {
      console.error('Failed to load recent missions:', error);
    }
  });

  async function quickTest() {
    showQuickTest = true;
    testResults = ['Testing system connectivity...'];
    
    try {
      // Test config
      const config = await ipc.invoke('config:get', {});
      testResults = [...testResults, `✓ Config loaded: ${config.backend.brain}`];
      
      // Test spec list
      const specs = await ipc.invoke('spec:list', {});
      testResults = [...testResults, `✓ Specs accessible: ${specs.length} found`];
      
      testResults = [...testResults, '✅ System ready!'];
      systemStatus = 'Ready';
    } catch (error) {
      testResults = [...testResults, `❌ System error: ${error}`];
      systemStatus = 'Error';
    }
  }

  function formatDuration(ms) {
    if (ms < 60000) return `${Math.round(ms / 1000)}s`;
    return `${Math.round(ms / 60000)}m`;
  }

  $: currentMission = $missionStore.current;
</script>

<div class="dashboard">
  <header class="dashboard-header">
    <div class="header-content">
      <h1>Mission Control Dashboard</h1>
      <p class="subtitle">System Status: <span class="status status--{systemStatus.toLowerCase()}">{systemStatus}</span></p>
    </div>
    <div class="header-actions">
      <button class="btn btn--secondary" on:click={quickTest}>
        System Check
      </button>
      <a href="/mission" class="btn btn--primary">
        Start Mission
      </a>
    </div>
  </header>

  <div class="dashboard-grid">
    <!-- Current Mission Status -->
    <section class="dashboard-card card--primary">
      <h2>Active Mission</h2>
      {#if $isRunning && currentMission}
        <div class="mission-status">
          <h3>{currentMission.title}</h3>
          <div class="progress-bar">
            <div class="progress-fill" style="width: {$progress}%"></div>
          </div>
          <div class="mission-meta">
            <span>Progress: {$progress}%</span>
            <span>Running: {formatDuration(Date.now() - new Date(currentMission.startedAt).getTime())}</span>
          </div>
        </div>
      {:else}
        <div class="empty-state">
          <p>No active mission</p>
          <a href="/mission" class="btn btn--primary btn--sm">Start Mission</a>
        </div>
      {/if}
    </section>

    <!-- Context Usage -->
    <section class="dashboard-card">
      <h2>Context Usage</h2>
      <div class="context-container">
        <ContextMeter 
          usage={$missionStore.current?.contextUsage || 0}
          maxTokens={100000}
          warningThreshold={75000}
          criticalThreshold={90000}
        />
      </div>
    </section>

    <!-- Cost Tracking -->
    <section class="dashboard-card">
      <h2>Cost Overview</h2>
      <div class="cost-container">
        <CostTracking 
          totalCost={recentMissions.reduce((sum, m) => sum + m.cost, 0)}
          todayCost={recentMissions.filter(m => 
            new Date(m.startedAt).toDateString() === new Date().toDateString()
          ).reduce((sum, m) => sum + m.cost, 0)}
          weekCost={recentMissions.reduce((sum, m) => sum + m.cost, 0)}
          showDetailed={false}
        />
      </div>
    </section>

    <!-- Recent Missions -->
    <section class="dashboard-card card--span-2">
      <div class="card-header">
        <h2>Recent Missions</h2>
        <a href="/history" class="link">View All</a>
      </div>
      
      {#if recentMissions.length > 0}
        <div class="mission-list">
          {#each recentMissions as mission}
            <div class="mission-item">
              <div class="mission-info">
                <h4>{mission.title}</h4>
                <div class="mission-meta">
                  <span class="status status--{mission.status}">{mission.status}</span>
                  <span>{new Date(mission.startedAt).toLocaleDateString()}</span>
                  <span>${mission.cost.toFixed(4)}</span>
                </div>
              </div>
              <div class="mission-progress">
                {#if mission.status === 'running'}
                  <div class="progress-circle">
                    <span>{mission.progress}%</span>
                  </div>
                {:else}
                  <div class="mission-duration">{formatDuration(mission.duration)}</div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="empty-state">
          <p>No recent missions</p>
        </div>
      {/if}
    </section>

    <!-- Quick Actions -->
    <section class="dashboard-card">
      <h2>Quick Actions</h2>
      <div class="action-grid">
        <a href="/mission" class="action-button">
          <div class="action-icon">🎯</div>
          <span>New Mission</span>
        </a>
        <a href="/specs" class="action-button">
          <div class="action-icon">📋</div>
          <span>Browse Specs</span>
        </a>
        <a href="/forge" class="action-button">
          <div class="action-icon">⚡</div>
          <span>AI Forge</span>
        </a>
        <a href="/settings" class="action-button">
          <div class="action-icon">⚙️</div>
          <span>Settings</span>
        </a>
      </div>
    </section>
  </div>

  <!-- System Check Results -->
  {#if showQuickTest && testResults.length > 0}
    <section class="test-results">
      <h3>System Check Results</h3>
      <div class="test-output">
        {#each testResults as result}
          <div class="test-line">{result}</div>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .dashboard {
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
  }

  .header-content h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 600;
    color: var(--text);
  }

  .subtitle {
    margin: 0.5rem 0 0 0;
    color: var(--text-muted);
  }

  .status {
    font-weight: 600;
    text-transform: uppercase;
    font-size: 0.875rem;
  }

  .status--ready { color: #22c55e; }
  .status--error { color: #ef4444; }
  .status--running { color: var(--accent); }
  .status--completed { color: #22c55e; }

  .header-actions {
    display: flex;
    gap: 1rem;
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

  .dashboard-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .dashboard-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .card--primary {
    border-color: var(--accent);
    background: linear-gradient(135deg, var(--bg-secondary), rgba(99, 102, 241, 0.05));
  }

  .card--span-2 {
    grid-column: span 2;
  }

  .dashboard-card h2 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .link {
    color: var(--accent);
    text-decoration: none;
    font-size: 0.875rem;
  }

  .link:hover {
    text-decoration: underline;
  }

  .mission-status h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    color: var(--text);
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: var(--bg);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.3s ease;
  }

  .mission-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.875rem;
    color: var(--text-muted);
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .context-container,
  .cost-container {
    display: flex;
    justify-content: center;
  }

  .mission-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .mission-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--bg);
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .mission-info h4 {
    margin: 0 0 0.25rem 0;
    font-size: 0.875rem;
    color: var(--text);
  }

  .mission-info .mission-meta {
    gap: 0.75rem;
    font-size: 0.75rem;
  }

  .progress-circle {
    width: 40px;
    height: 40px;
    border: 2px solid var(--accent);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--accent);
  }

  .mission-duration {
    font-size: 0.875rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .action-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  .action-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    text-decoration: none;
    color: var(--text);
    transition: all 0.2s ease;
  }

  .action-button:hover {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .action-icon {
    font-size: 1.5rem;
  }

  .test-results {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    margin-top: 1rem;
  }

  .test-results h3 {
    margin: 0 0 1rem 0;
    color: var(--text);
  }

  .test-output {
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.875rem;
    background: var(--bg);
    padding: 1rem;
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .test-line {
    margin-bottom: 0.25rem;
    color: var(--text);
  }

  @media (max-width: 768px) {
    .dashboard {
      padding: 1rem;
    }

    .dashboard-header {
      flex-direction: column;
      gap: 1rem;
      align-items: stretch;
    }

    .dashboard-grid {
      grid-template-columns: 1fr;
    }

    .card--span-2 {
      grid-column: span 1;
    }

    .action-grid {
      grid-template-columns: 1fr;
    }

    .mission-item {
      flex-direction: column;
      gap: 0.75rem;
      align-items: flex-start;
    }
  }
</style>