<script lang="ts">
  import { missionStore, isRunning, progress } from '$lib/stores/mission';
  import { onMount } from 'svelte';
  import MissionComparison from '$lib/components/mission/MissionComparison.svelte';
  import ContextMeter from '$lib/components/mission/ContextMeter.svelte';
  import CostTracking from '$lib/components/mission/CostTracking.svelte';
  import { ipc } from '$lib/ipc';

  let activeMission = null;
  let missionSpec = '';
  let backend = 'claude';
  let mode = 'development';
  let isStarting = false;

  onMount(async () => {
    // Load active mission if any
    if ($missionStore.current) {
      activeMission = $missionStore.current;
    }
  });

  async function startMission() {
    if (!missionSpec.trim()) {
      alert('Please select a spec file');
      return;
    }

    isStarting = true;
    try {
      const result = await ipc.invoke('mission:start', {
        specPath: missionSpec,
        backend,
        mode
      });
      
      console.log('Mission started:', result.missionId);
      // The mission store should automatically update via IPC events
    } catch (error) {
      console.error('Failed to start mission:', error);
      alert('Failed to start mission: ' + error.message);
    } finally {
      isStarting = false;
    }
  }

  async function stopMission() {
    if (!activeMission) return;

    try {
      const result = await ipc.invoke('mission:stop', {
        missionId: activeMission.id
      });
      
      if (result.success) {
        console.log('Mission stopped');
      }
    } catch (error) {
      console.error('Failed to stop mission:', error);
      alert('Failed to stop mission: ' + error.message);
    }
  }

  $: currentMission = $missionStore.current;
</script>

<div class="mission-control">
  <div class="mission-header">
    <h1>Mission Control</h1>
    <p class="subtitle">Start and manage AI-driven development missions</p>
  </div>

  {#if $isRunning && currentMission}
    <!-- Active Mission View -->
    <section class="active-mission">
      <div class="mission-card">
        <div class="mission-status">
          <div class="status-indicator running">
            <div class="pulse-dot"></div>
            <span>Mission Active</span>
          </div>
          <button class="btn btn--danger" on:click={stopMission}>
            Stop Mission
          </button>
        </div>

        <div class="mission-details">
          <h2>{currentMission.title}</h2>
          <div class="mission-meta">
            <span>Backend: {currentMission.backend || backend}</span>
            <span>Mode: {currentMission.mode || mode}</span>
            <span>Started: {new Date(currentMission.startedAt).toLocaleString()}</span>
          </div>
        </div>

        <div class="progress-section">
          <div class="progress-header">
            <span>Progress: {$progress}%</span>
            <span>{currentMission.currentStep || 'Processing...'}</span>
          </div>
          <div class="progress-bar">
            <div class="progress-fill" style="width: {$progress}%"></div>
          </div>
        </div>
      </div>

      <div class="mission-metrics">
        <div class="metric-card">
          <h3>Context Usage</h3>
          <ContextMeter 
            usage={currentMission.contextUsage || 0}
            maxTokens={100000}
            warningThreshold={75000}
            criticalThreshold={90000}
          />
        </div>

        <div class="metric-card">
          <h3>Cost Tracking</h3>
          <CostTracking 
            totalCost={currentMission.cost || 0}
            todayCost={currentMission.cost || 0}
            weekCost={currentMission.cost || 0}
            showDetailed={false}
          />
        </div>
      </div>

      {#if currentMission.comparison}
        <div class="comparison-section">
          <h3>Mission Comparison</h3>
          <MissionComparison comparison={currentMission.comparison} />
        </div>
      {/if}
    </section>
  {:else}
    <!-- Mission Setup View -->
    <section class="mission-setup">
      <div class="setup-card">
        <h2>Start New Mission</h2>
        <p>Configure and launch a new development mission</p>

        <form on:submit|preventDefault={startMission}>
          <div class="form-group">
            <label for="spec-select">Spec File</label>
            <div class="input-group">
              <input
                id="spec-select"
                type="text"
                bind:value={missionSpec}
                placeholder="/path/to/spec.md"
                required
              />
              <button type="button" class="btn btn--secondary btn--sm">Browse</button>
            </div>
            <small>Select the specification file to implement</small>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="backend-select">AI Backend</label>
              <select id="backend-select" bind:value={backend}>
                <option value="claude">Claude (Anthropic)</option>
                <option value="gpt4">GPT-4 (OpenAI)</option>
                <option value="gemini">Gemini (Google)</option>
              </select>
            </div>

            <div class="form-group">
              <label for="mode-select">Execution Mode</label>
              <select id="mode-select" bind:value={mode}>
                <option value="development">Development</option>
                <option value="production">Production</option>
                <option value="testing">Testing</option>
              </select>
            </div>
          </div>

          <div class="form-actions">
            <button type="submit" class="btn btn--primary" disabled={isStarting}>
              {isStarting ? 'Starting Mission...' : 'Start Mission'}
            </button>
          </div>
        </form>
      </div>

      <div class="recent-missions">
        <h3>Recent Missions</h3>
        <div class="mission-list">
          <div class="empty-state">
            <p>No recent missions</p>
            <small>Completed missions will appear here</small>
          </div>
        </div>
      </div>
    </section>
  {/if}
</div>

<style>
  .mission-control {
    max-width: 1200px;
    margin: 0 auto;
  }

  .mission-header {
    margin-bottom: 2rem;
    text-align: center;
  }

  .mission-header h1 {
    margin: 0;
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text);
    margin-bottom: 0.5rem;
  }

  .subtitle {
    color: var(--text-muted);
    font-size: 1.125rem;
    margin: 0;
  }

  .active-mission {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .mission-card {
    background: var(--bg-secondary);
    border: 1px solid var(--accent);
    border-radius: 12px;
    padding: 2rem;
  }

  .mission-status {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-weight: 600;
    color: var(--accent);
  }

  .status-indicator.running {
    color: var(--accent);
  }

  .pulse-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0% { transform: scale(1); opacity: 1; }
    50% { transform: scale(1.2); opacity: 0.7; }
    100% { transform: scale(1); opacity: 1; }
  }

  .mission-details h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1.5rem;
    color: var(--text);
  }

  .mission-meta {
    display: flex;
    gap: 2rem;
    color: var(--text-muted);
    font-size: 0.875rem;
    margin-bottom: 1.5rem;
  }

  .progress-section {
    margin-top: 1rem;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
    color: var(--text);
  }

  .progress-bar {
    width: 100%;
    height: 12px;
    background: var(--bg);
    border-radius: 6px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.3s ease;
  }

  .mission-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .metric-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .metric-card h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    color: var(--text);
  }

  .comparison-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .comparison-section h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    color: var(--text);
  }

  .mission-setup {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 2rem;
  }

  .setup-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 2rem;
  }

  .setup-card h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
    color: var(--text);
  }

  .setup-card p {
    color: var(--text-muted);
    margin: 0 0 2rem 0;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: var(--text);
  }

  .input-group {
    display: flex;
    gap: 0.5rem;
  }

  .input-group input {
    flex: 1;
  }

  input, select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font-size: 0.875rem;
  }

  input:focus, select:focus {
    outline: none;
    border-color: var(--accent);
  }

  small {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .form-actions {
    margin-top: 2rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border);
  }

  .btn {
    padding: 0.75rem 1.5rem;
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

  .btn--primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn--primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn--secondary {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn--secondary:hover {
    background: var(--bg);
  }

  .btn--danger {
    background: #ef4444;
    color: white;
  }

  .btn--danger:hover {
    background: #dc2626;
  }

  .btn--sm {
    padding: 0.5rem 1rem;
    font-size: 0.75rem;
  }

  .recent-missions {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .recent-missions h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    color: var(--text);
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .empty-state p {
    margin: 0 0 0.25rem 0;
    font-size: 0.875rem;
  }

  .empty-state small {
    font-size: 0.75rem;
  }

  @media (max-width: 768px) {
    .mission-header h1 {
      font-size: 2rem;
    }

    .mission-setup {
      grid-template-columns: 1fr;
      gap: 1.5rem;
    }

    .form-row {
      grid-template-columns: 1fr;
    }

    .mission-meta {
      flex-direction: column;
      gap: 0.5rem;
    }

    .mission-status {
      flex-direction: column;
      gap: 1rem;
      align-items: flex-start;
    }
  }
</style>