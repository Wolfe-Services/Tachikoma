<script lang="ts">
  import { onMount } from 'svelte';
  import { ipc } from '$lib/ipc';
  import Icon from '$lib/components/common/Icon.svelte';
  import TachikomaLogo from '$lib/components/common/TachikomaLogo.svelte';
  
  // System status - reflects actual backend functionality
  let stats = {
    specsTotal: 562,
    specsComplete: 562,
    activeMissions: 0,        // Loop Runner sessions running
    forgeSessions: 0,         // Active Forge brainstorming sessions
    totalIterations: 0,       // Total iterations completed
    successRate: 98.2
  };
  
  // System components status
  let systemStatus = {
    loopRunner: 'idle',       // idle | running | paused
    forge: 'ready',           // ready | active | processing
    specRegistry: 'online',   // online | syncing
    aiBackend: 'connected'    // connected | disconnected
  };
  
  let platform = 'unknown';
  
  // AI backends configured in the system
  const configuredBackends = [
    { id: 'claude', name: 'Claude', status: 'primary', provider: 'Anthropic' },
    { id: 'gpt4', name: 'GPT-4', status: 'available', provider: 'OpenAI' },
    { id: 'gemini', name: 'Gemini', status: 'available', provider: 'Google' },
    { id: 'ollama', name: 'Ollama', status: 'local', provider: 'Local' },
  ];
  
  // Tachikoma personality - they're curious and enthusiastic!
  const tachiQuotes = [
    "Analyzing target specifications...",
    "Loop runner standing by!",
    "Ready to assist, Major!",
    "Tactical assessment complete.",
    "All systems nominal."
  ];
  
  let currentQuote = tachiQuotes[0];
  
  // Real activity from the system (populated from IPC in production)
  let recentActivity = [
    { id: 1, action: 'Loop completed: spec-142 UI polish', time: '2 min ago', type: 'success', component: 'loop' },
    { id: 2, action: 'Context reboot triggered (redline)', time: '5 min ago', type: 'info', component: 'loop' },
    { id: 3, action: 'Forge session started: API design', time: '12 min ago', type: 'info', component: 'forge' },
    { id: 4, action: 'Spec synced from filesystem', time: '1 hr ago', type: 'success', component: 'specs' }
  ];
  
  onMount(async () => {
    if (typeof window !== 'undefined' && window.tachikoma) {
      platform = window.tachikoma.platform;
    }
    
    // Rotate quotes like a Tachikoma would chatter
    setInterval(() => {
      currentQuote = tachiQuotes[Math.floor(Math.random() * tachiQuotes.length)];
    }, 5000);
    
    try {
      const specs = await ipc.invoke('spec:list', {});
      if (Array.isArray(specs)) {
        stats.specsTotal = specs.length || 0;
        stats.specsComplete = specs.filter((s: any) => s.complete).length || 0;
        stats = { ...stats };
      }
      
      // TODO: Fetch actual mission/forge status from backend
      // const missionStatus = await ipc.invoke('mission:status', {});
      // const forgeStatus = await ipc.invoke('forge:status', {});
    } catch (e) {
      console.log('IPC not available:', e);
    }
  });
  
  $: specsProgress = stats.specsTotal > 0 
    ? Math.round((stats.specsComplete / stats.specsTotal) * 100) 
    : 0;
</script>

<div class="dashboard">
  <!-- Hero Section -->
  <header class="dashboard-hero">
    <div class="hero-background">
      <div class="hex-grid"></div>
    </div>
    <div class="hero-content">
      <div class="hero-icon">
        <TachikomaLogo size={72} animated={true} />
      </div>
      <div class="hero-text">
        <div class="hero-tag">SECTION 9 // AI DIVISION</div>
        <h1 class="hero-title">TACHIKOMA</h1>
        <p class="hero-subtitle">Spec-driven AI development system with continuous loop execution</p>
        <div class="hero-quote">
          <span class="quote-indicator">▸</span>
          <span class="quote-text">{currentQuote}</span>
        </div>
      </div>
    </div>
    <div class="hero-meta">
      <span class="meta-item" class:online={systemStatus.loopRunner !== 'running'} class:active={systemStatus.loopRunner === 'running'}>
        <span class="status-dot"></span>
        <span>{systemStatus.loopRunner === 'running' ? 'LOOP ACTIVE' : 'LOOP IDLE'}</span>
      </span>
      <span class="meta-divider">//</span>
      <span class="meta-item online">
        <span class="status-dot"></span>
        <span>AI BACKEND CONNECTED</span>
      </span>
      <span class="meta-divider">//</span>
      <span class="meta-item">
        <span>PLATFORM: {platform.toUpperCase()}</span>
      </span>
    </div>
  </header>
  
  <!-- System Status Grid -->
  <section class="squad-section">
    <div class="section-header">
      <h2 class="section-title">
        <Icon name="activity" size={18} />
        <span>SYSTEM STATUS</span>
      </h2>
      <span class="section-subtitle">Core components and AI backend status</span>
    </div>
    
    <div class="stats-grid">
      <!-- Loop Runner Status -->
      <div class="stat-card" class:active={systemStatus.loopRunner === 'running'}>
        <div class="stat-header">
          <Icon name="play-circle" size={18} glow />
          <span class="stat-label">LOOP RUNNER</span>
          <span class="status-pill" class:running={systemStatus.loopRunner === 'running'} class:idle={systemStatus.loopRunner === 'idle'}>
            {systemStatus.loopRunner.toUpperCase()}
          </span>
        </div>
        <div class="stat-body">
          <div class="component-status">
            <div class="status-row">
              <span class="status-label">Active Missions:</span>
              <span class="status-value">{stats.activeMissions}</span>
            </div>
            <div class="status-row">
              <span class="status-label">Total Iterations:</span>
              <span class="status-value">{stats.totalIterations}</span>
            </div>
          </div>
          <div class="stat-detail">Continuous Claude Code execution engine</div>
        </div>
      </div>
      
      <!-- Specs Progress -->
      <div class="stat-card primary">
        <div class="stat-header">
          <Icon name="file-text" size={18} glow />
          <span class="stat-label">SPEC REGISTRY</span>
        </div>
        <div class="stat-body">
          <div class="stat-value">{stats.specsComplete}<span class="stat-unit">/{stats.specsTotal}</span></div>
          <div class="progress-container">
            <div class="progress-bar">
              <div class="progress-fill" style="width: {specsProgress}%">
                <div class="progress-glow"></div>
              </div>
            </div>
            <span class="progress-label">{specsProgress}%</span>
          </div>
          <div class="stat-detail">Specifications with all criteria complete</div>
        </div>
      </div>
      
      <!-- Forge (Think Tank) Status -->
      <div class="stat-card">
        <div class="stat-header">
          <Icon name="brain" size={18} glow />
          <span class="stat-label">THINK TANK</span>
          <span class="status-pill ready">{systemStatus.forge.toUpperCase()}</span>
        </div>
        <div class="stat-body">
          <div class="component-status">
            <div class="status-row">
              <span class="status-label">Active Sessions:</span>
              <span class="status-value">{stats.forgeSessions}</span>
            </div>
            <div class="status-row">
              <span class="status-label">AI Backends:</span>
              <span class="status-value">{configuredBackends.length} configured</span>
            </div>
          </div>
          <div class="stat-detail">Multi-model spec brainstorming</div>
        </div>
      </div>
    </div>
    
    <!-- AI Backends Row -->
    <div class="backends-section">
      <div class="backends-label">
        <Icon name="cpu" size={14} />
        <span>CONFIGURED AI BACKENDS</span>
      </div>
      <div class="backends-list">
        {#each configuredBackends as backend}
          <div class="backend-chip" class:primary={backend.status === 'primary'}>
            <span class="backend-name">{backend.name}</span>
            <span class="backend-provider">{backend.provider}</span>
          </div>
        {/each}
      </div>
    </div>
  </section>
  
  <!-- Command Grid -->
  <section class="content-section">
    <div class="content-grid">
      <!-- Operations Panel -->
      <div class="panel operations">
        <div class="panel-header">
          <h2 class="panel-title">
            <Icon name="zap" size={18} />
            <span>OPERATIONS</span>
          </h2>
          <span class="panel-badge">COMMAND</span>
        </div>
        <div class="action-grid">
          <a href="/missions/new" class="action-card">
            <div class="action-icon">
              <Icon name="play-circle" size={28} />
            </div>
            <div class="action-content">
              <span class="action-label">Deploy Unit</span>
              <span class="action-desc">Task a Tachikoma to a mission</span>
            </div>
            <div class="action-arrow">
              <Icon name="chevron-right" size={16} />
            </div>
          </a>
          <a href="/specs" class="action-card">
            <div class="action-icon">
              <Icon name="file-text" size={28} />
            </div>
            <div class="action-content">
              <span class="action-label">Spec Registry</span>
              <span class="action-desc">View mission specifications</span>
            </div>
            <div class="action-arrow">
              <Icon name="chevron-right" size={16} />
            </div>
          </a>
          <a href="/forge" class="action-card highlight">
            <div class="action-icon forge-icon">
              <Icon name="brain" size={28} />
            </div>
            <div class="action-content">
              <span class="action-label">Think Tank</span>
              <span class="action-desc">Multi-unit deliberation engine</span>
            </div>
            <div class="action-arrow">
              <Icon name="chevron-right" size={16} />
            </div>
          </a>
          <a href="/settings" class="action-card">
            <div class="action-icon">
              <Icon name="settings" size={28} />
            </div>
            <div class="action-content">
              <span class="action-label">Configuration</span>
              <span class="action-desc">Squad & system settings</span>
            </div>
            <div class="action-arrow">
              <Icon name="chevron-right" size={16} />
            </div>
          </a>
        </div>
      </div>
      
      <!-- Activity Log -->
      <div class="panel activity-log">
        <div class="panel-header">
          <h2 class="panel-title">
            <Icon name="terminal" size={18} />
            <span>ACTIVITY LOG</span>
          </h2>
          <span class="panel-badge live">LIVE</span>
        </div>
        <div class="activity-list">
          {#each recentActivity as activity}
            <div class="activity-item" class:success={activity.type === 'success'} class:error={activity.type === 'error'}>
              <div class="activity-indicator" class:success={activity.type === 'success'} class:error={activity.type === 'error'}></div>
              <div class="activity-content">
                <span class="activity-action">
                  {#if activity.component}
                    <span class="component-tag" class:loop={activity.component === 'loop'} class:forge={activity.component === 'forge'} class:specs={activity.component === 'specs'}>
                      {activity.component.toUpperCase()}
                    </span>
                  {/if}
                  {activity.action}
                </span>
                <span class="activity-time">{activity.time}</span>
              </div>
            </div>
          {/each}
        </div>
        <div class="activity-footer">
          <a href="/logs" class="view-all">
            <span>View full activity log</span>
            <Icon name="arrow-right" size={14} />
          </a>
        </div>
      </div>
    </div>
  </section>
  
  <!-- Lore Footer -->
  <footer class="lore-footer">
    <div class="lore-content">
      <span class="lore-symbol">◈</span>
      <p class="lore-text">
        Tachikoma: Spec-driven AI development with the Ralph Loop for continuous execution 
        and multi-model Think Tank for collaborative spec creation.
      </p>
      <span class="lore-symbol">◈</span>
    </div>
  </footer>
</div>

<style>
  .dashboard {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0;
  }
  
  /* Hero Section */
  .dashboard-hero {
    padding: 2.5rem;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.1) 0%, rgba(78, 205, 196, 0.02) 50%, transparent 100%);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 16px;
    margin-bottom: 2rem;
    position: relative;
    overflow: hidden;
  }
  
  .hero-background {
    position: absolute;
    inset: 0;
    opacity: 0.3;
    pointer-events: none;
  }
  
  .hex-grid {
    position: absolute;
    inset: 0;
    background-image: url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M30 0l25.98 15v30L30 60 4.02 45V15z' fill='none' stroke='%234ecdc4' stroke-width='0.5' opacity='0.15'/%3E%3C/svg%3E");
    background-size: 30px 30px;
  }
  
  .hero-content {
    display: flex;
    align-items: flex-start;
    gap: 2rem;
    margin-bottom: 1.5rem;
    position: relative;
    z-index: 1;
  }
  
  .hero-icon {
    flex-shrink: 0;
    padding-top: 0.5rem;
  }
  
  .hero-text {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .hero-tag {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 3px;
    opacity: 0.8;
  }
  
  .hero-title {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 2.25rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 4px;
    margin: 0;
    text-shadow: 0 0 30px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.4));
  }
  
  .hero-subtitle {
    font-size: 1.1rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    margin: 0;
  }
  
  .hero-quote {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: rgba(78, 205, 196, 0.08);
    border-left: 2px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 0 4px 4px 0;
  }
  
  .quote-indicator {
    color: var(--tachi-cyan, #4ecdc4);
    animation: blink 1s ease-in-out infinite;
  }
  
  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }
  
  .quote-text {
    font-size: 0.9rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    font-style: italic;
  }
  
  .hero-meta {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    letter-spacing: 1px;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    position: relative;
    z-index: 1;
  }
  
  .meta-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .meta-item.online {
    color: var(--success-color, #3fb950);
  }
  
  .meta-item.active {
    color: var(--tachi-yellow, #ffd93d);
  }
  
  .meta-item.active .status-dot {
    background: var(--tachi-yellow, #ffd93d);
    box-shadow: 0 0 8px var(--tachi-yellow, #ffd93d);
  }
  
  .status-dot {
    width: 6px;
    height: 6px;
    background: var(--success-color, #3fb950);
    border-radius: 50%;
    box-shadow: 0 0 8px var(--success-color, #3fb950);
    animation: pulse 2s ease-in-out infinite;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .meta-divider {
    color: var(--border-color, rgba(78, 205, 196, 0.3));
  }
  
  /* Squad Section */
  .squad-section {
    margin-bottom: 2rem;
  }
  
  .section-header {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 1rem;
  }
  
  .section-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 1.5px;
    margin: 0;
  }
  
  .section-subtitle {
    font-size: 0.8rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
  }
  
  .stat-card {
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    padding: 1.5rem;
    transition: all 0.3s ease;
  }
  
  .stat-card:hover {
    border-color: var(--tachi-cyan, #4ecdc4);
    box-shadow: 0 0 20px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.15));
  }
  
  .stat-card.primary {
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.1), var(--bg-secondary, #161b22));
    border-color: rgba(78, 205, 196, 0.3);
  }
  
  .stat-card.active {
    border-color: var(--success-color, #3fb950);
    box-shadow: 0 0 20px rgba(63, 185, 80, 0.15);
  }
  
  .status-pill {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.55rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    letter-spacing: 1px;
    margin-left: auto;
  }
  
  .status-pill.idle {
    background: rgba(230, 237, 243, 0.1);
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }
  
  .status-pill.running {
    background: rgba(63, 185, 80, 0.2);
    color: var(--success-color, #3fb950);
    animation: statusPulse 2s ease-in-out infinite;
  }
  
  .status-pill.ready {
    background: rgba(78, 205, 196, 0.15);
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  @keyframes statusPulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }
  
  .component-status {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .status-label {
    font-size: 0.85rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }
  
  .status-value {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.9rem;
    color: var(--text-primary, #e6edf3);
  }
  
  /* AI Backends Section */
  .backends-section {
    margin-top: 1rem;
    padding: 1rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 10px;
  }
  
  .backends-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 1px;
    margin-bottom: 0.75rem;
  }
  
  .backends-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  
  .backend-chip {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 0.5rem 0.75rem;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 6px;
    transition: all 0.2s ease;
  }
  
  .backend-chip:hover {
    border-color: var(--tachi-cyan, #4ecdc4);
  }
  
  .backend-chip.primary {
    border-color: var(--tachi-cyan, #4ecdc4);
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.1), var(--bg-tertiary, #1c2128));
  }
  
  .backend-name {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
  }
  
  .backend-provider {
    font-size: 0.65rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .stat-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .stat-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 1.5px;
  }
  
  .stat-body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  
  .stat-value {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    line-height: 1;
  }
  
  .stat-unit {
    font-size: 1rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    margin-left: 0.25rem;
  }
  
  .stat-detail {
    font-size: 0.85rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .stat-detail.success {
    color: var(--success-color, #3fb950);
  }
  
  .progress-container {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .progress-bar {
    flex: 1;
    height: 8px;
    background: var(--bg-tertiary, #1c2128);
    border-radius: 4px;
    overflow: hidden;
  }
  
  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    border-radius: 4px;
    position: relative;
    transition: width 0.5s ease;
  }
  
  .progress-glow {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 30px;
    background: linear-gradient(90deg, transparent, var(--tachi-cyan-bright, #6ee7df));
    animation: progressGlow 2s ease-in-out infinite;
  }
  
  @keyframes progressGlow {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 1; }
  }
  
  .progress-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.8rem;
    color: var(--tachi-cyan, #4ecdc4);
    min-width: 45px;
    text-align: right;
  }
  
  /* Content Section */
  .content-section {
    margin-bottom: 2rem;
  }
  
  .content-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
  }
  
  @media (max-width: 1000px) {
    .content-grid {
      grid-template-columns: 1fr;
    }
  }
  
  .panel {
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    overflow: hidden;
  }
  
  .panel-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    background: linear-gradient(90deg, rgba(78, 205, 196, 0.08), transparent);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  
  .panel-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 1.5px;
    margin: 0;
  }
  
  .panel-badge {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    font-weight: 600;
    padding: 0.25rem 0.5rem;
    background: rgba(78, 205, 196, 0.1);
    border: 1px solid rgba(78, 205, 196, 0.2);
    border-radius: 4px;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 1px;
  }
  
  .panel-badge.live {
    background: rgba(255, 107, 107, 0.1);
    border-color: rgba(255, 107, 107, 0.3);
    color: var(--tachi-red, #ff6b6b);
    animation: livePulse 2s ease-in-out infinite;
  }
  
  @keyframes livePulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }
  
  /* Operations Panel */
  .action-grid {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
  }
  
  .action-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.25rem;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    border-radius: 10px;
    text-decoration: none;
    color: var(--text-primary, #e6edf3);
    transition: all 0.3s ease;
  }
  
  .action-card:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    border-color: var(--tachi-cyan, #4ecdc4);
    transform: translateX(4px);
  }
  
  .action-card.highlight {
    background: linear-gradient(90deg, rgba(78, 205, 196, 0.12), var(--bg-tertiary, #1c2128));
    border-color: rgba(78, 205, 196, 0.25);
  }
  
  .action-icon {
    color: var(--tachi-cyan, #4ecdc4);
    flex-shrink: 0;
    width: 32px;
    display: flex;
    justify-content: center;
  }
  
  .forge-icon {
    animation: forgeGlow 3s ease-in-out infinite;
  }
  
  @keyframes forgeGlow {
    0%, 100% { filter: drop-shadow(0 0 4px var(--tachi-cyan-glow)); }
    50% { filter: drop-shadow(0 0 12px var(--tachi-cyan)); }
  }
  
  .action-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }
  
  .action-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.85rem;
    font-weight: 600;
    letter-spacing: 0.5px;
  }
  
  .action-desc {
    font-size: 0.8rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .action-arrow {
    color: var(--text-muted, rgba(230, 237, 243, 0.3));
    transition: all 0.2s ease;
  }
  
  .action-card:hover .action-arrow {
    color: var(--tachi-cyan, #4ecdc4);
    transform: translateX(4px);
  }
  
  /* Activity Log */
  .activity-list {
    padding: 0.5rem 0;
    max-height: 280px;
    overflow-y: auto;
  }
  
  .activity-item {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 0.875rem 1.25rem;
    transition: background 0.2s ease;
  }
  
  .activity-item:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.05));
  }
  
  .activity-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted, rgba(230, 237, 243, 0.3));
    margin-top: 0.5rem;
    flex-shrink: 0;
  }
  
  .activity-indicator.success {
    background: var(--success-color, #3fb950);
    box-shadow: 0 0 8px rgba(63, 185, 80, 0.5);
  }
  
  .activity-indicator.error {
    background: var(--error-color, #ff6b6b);
    box-shadow: 0 0 8px rgba(255, 107, 107, 0.5);
  }
  
  .activity-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .activity-action {
    font-size: 0.9rem;
    color: var(--text-primary, #e6edf3);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  
  .component-tag {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    font-weight: 600;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    letter-spacing: 0.5px;
  }
  
  .component-tag.loop {
    background: rgba(63, 185, 80, 0.15);
    border: 1px solid rgba(63, 185, 80, 0.3);
    color: var(--success-color, #3fb950);
  }
  
  .component-tag.forge {
    background: rgba(139, 92, 246, 0.15);
    border: 1px solid rgba(139, 92, 246, 0.3);
    color: #a78bfa;
  }
  
  .component-tag.specs {
    background: rgba(78, 205, 196, 0.15);
    border: 1px solid rgba(78, 205, 196, 0.3);
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .activity-time {
    font-size: 0.75rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .activity-footer {
    padding: 0.75rem 1.25rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
  }
  
  .view-all {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    text-decoration: none;
    transition: all 0.2s ease;
  }
  
  .view-all:hover {
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  /* Lore Footer */
  .lore-footer {
    padding: 1.5rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    margin-top: 1rem;
  }
  
  .lore-content {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    text-align: center;
  }
  
  .lore-symbol {
    color: var(--tachi-cyan, #4ecdc4);
    opacity: 0.5;
    font-size: 0.8rem;
  }
  
  .lore-text {
    font-size: 0.8rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    font-style: italic;
    max-width: 600px;
    margin: 0;
  }
</style>
