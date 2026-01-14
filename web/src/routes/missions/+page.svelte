<script lang="ts">
  import type { MissionHistoryEntry } from '$lib/types/history';
  import HistoryView from '$lib/components/mission/HistoryView.svelte';
  import PageHeader from '$lib/components/common/PageHeader.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  
  // Section 9 deployment terminology
  let missions: MissionHistoryEntry[] = [];
  
  // Tactical readiness stats
  let readyUnits = 9;
  let deploymentsToday = 0;
  let successRate = 98.2;
</script>

<div class="deploy-page">
  <PageHeader 
    title="DEPLOY UNITS"
    subtitle="Task Tachikoma units to execute missions autonomously"
    tag="TACTICAL OPERATIONS"
    icon="play-circle"
  >
    <svelte:fragment slot="actions">
      <a href="/missions/new" class="btn-primary">
        <Icon name="zap" size={16} />
        <span>DEPLOY UNIT</span>
      </a>
    </svelte:fragment>
  </PageHeader>
  
  <!-- Tactical Overview -->
  <section class="tactical-overview">
    <div class="overview-card">
      <div class="overview-icon ready">
        <Icon name="robot" size={24} glow />
      </div>
      <div class="overview-data">
        <span class="overview-value">{readyUnits}</span>
        <span class="overview-label">UNITS READY</span>
      </div>
      <div class="overview-bar">
        <div class="bar-fill" style="width: 100%"></div>
      </div>
    </div>
    
    <div class="overview-card">
      <div class="overview-icon">
        <Icon name="activity" size={24} />
      </div>
      <div class="overview-data">
        <span class="overview-value">{deploymentsToday}</span>
        <span class="overview-label">DEPLOYMENTS TODAY</span>
      </div>
      <div class="overview-bar">
        <div class="bar-fill" style="width: 0%"></div>
      </div>
    </div>
    
    <div class="overview-card">
      <div class="overview-icon success">
        <Icon name="target" size={24} />
      </div>
      <div class="overview-data">
        <span class="overview-value">{successRate}%</span>
        <span class="overview-label">SUCCESS RATE</span>
      </div>
      <div class="overview-bar">
        <div class="bar-fill success" style="width: {successRate}%"></div>
      </div>
    </div>
  </section>
  
  {#if missions.length > 0}
    <section class="missions-section">
      <div class="section-header">
        <h2 class="section-title">
          <Icon name="clock" size={18} />
          <span>MISSION HISTORY</span>
        </h2>
        <span class="section-count">{missions.length} missions</span>
      </div>
      <HistoryView entries={missions} />
    </section>
  {:else}
    <!-- Empty State - First Deployment -->
    <div class="empty-state">
      <div class="empty-container">
        <div class="empty-visual">
          <div class="tachi-silhouette">
            <div class="eye left"></div>
            <div class="eye right"></div>
          </div>
          <div class="pulse-ring ring-1"></div>
          <div class="pulse-ring ring-2"></div>
          <div class="pulse-ring ring-3"></div>
        </div>
        
        <div class="empty-content">
          <h2 class="empty-title">UNITS STANDING BY</h2>
          <p class="empty-text">
            All 9 Tachikoma units are ready for tasking. Deploy your first unit to begin 
            automated spec implementation.
          </p>
          
          <div class="empty-features">
            <div class="feature">
              <Icon name="zap" size={18} />
              <span>Autonomous coding</span>
            </div>
            <div class="feature">
              <Icon name="shield" size={18} />
              <span>Safe execution</span>
            </div>
            <div class="feature">
              <Icon name="refresh-cw" size={18} />
              <span>Auto-recovery</span>
            </div>
          </div>
          
          <a href="/missions/new" class="deploy-btn">
            <div class="btn-glow"></div>
            <Icon name="play" size={20} />
            <span>DEPLOY FIRST UNIT</span>
          </a>
        </div>
      </div>
      
      <!-- Quick Actions -->
      <div class="quick-actions">
        <a href="/specs" class="action-link">
          <Icon name="file-text" size={16} />
          <span>Browse specs to deploy</span>
          <Icon name="chevron-right" size={14} />
        </a>
        <a href="/forge" class="action-link">
          <Icon name="brain" size={16} />
          <span>Create new spec with Think Tank</span>
          <Icon name="chevron-right" size={14} />
        </a>
      </div>
    </div>
  {/if}
</div>

<style>
  .deploy-page {
    max-width: 1200px;
    margin: 0 auto;
  }
  
  /* Tactical Overview */
  .tactical-overview {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }
  
  .overview-card {
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    padding: 1.25rem;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 1rem;
    transition: all 0.3s ease;
    position: relative;
    overflow: hidden;
  }
  
  .overview-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, transparent, var(--tachi-cyan, #4ecdc4), transparent);
    opacity: 0;
    transition: opacity 0.3s ease;
  }
  
  .overview-card:hover {
    border-color: var(--tachi-cyan, #4ecdc4);
    box-shadow: 0 0 20px rgba(78, 205, 196, 0.15);
  }
  
  .overview-card:hover::before {
    opacity: 1;
  }
  
  .overview-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(78, 205, 196, 0.1);
    border: 1px solid rgba(78, 205, 196, 0.2);
    border-radius: 10px;
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .overview-icon.ready {
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.2), rgba(78, 205, 196, 0.05));
    animation: readyPulse 2s ease-in-out infinite;
  }
  
  .overview-icon.success {
    color: var(--success-color, #3fb950);
    background: rgba(63, 185, 80, 0.1);
    border-color: rgba(63, 185, 80, 0.2);
  }
  
  @keyframes readyPulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(78, 205, 196, 0.4); }
    50% { box-shadow: 0 0 20px 5px rgba(78, 205, 196, 0.2); }
  }
  
  .overview-data {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  
  .overview-value {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    line-height: 1;
  }
  
  .overview-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 1px;
    margin-top: 0.25rem;
  }
  
  .overview-bar {
    width: 100%;
    height: 3px;
    background: var(--bg-tertiary, #1c2128);
    border-radius: 2px;
    overflow: hidden;
  }
  
  .bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    border-radius: 2px;
    transition: width 0.5s ease;
  }
  
  .bar-fill.success {
    background: linear-gradient(90deg, #2d7a4a, var(--success-color, #3fb950));
  }
  
  /* Missions Section */
  .missions-section {
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    overflow: hidden;
  }
  
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    background: linear-gradient(90deg, rgba(78, 205, 196, 0.08), transparent);
  }
  
  .section-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 1.5px;
    margin: 0;
  }
  
  .section-count {
    font-size: 0.8rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }
  
  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2rem;
  }
  
  .empty-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 3rem 2rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 16px;
    width: 100%;
    max-width: 600px;
    position: relative;
    overflow: hidden;
  }
  
  .empty-container::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, transparent, var(--tachi-cyan, #4ecdc4), transparent);
  }
  
  .empty-visual {
    position: relative;
    width: 120px;
    height: 120px;
    margin-bottom: 1.5rem;
  }
  
  .tachi-silhouette {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 60px;
    height: 60px;
    background: linear-gradient(135deg, var(--bg-tertiary, #1c2128), var(--bg-primary, #0d1117));
    border: 2px solid rgba(78, 205, 196, 0.5);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    z-index: 1;
  }
  
  .eye {
    width: 8px;
    height: 8px;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    box-shadow: 0 0 10px var(--tachi-cyan, #4ecdc4);
    animation: eyeBlink 4s ease-in-out infinite;
  }
  
  .eye.right {
    animation-delay: 0.1s;
  }
  
  @keyframes eyeBlink {
    0%, 45%, 55%, 100% { opacity: 1; }
    48%, 52% { opacity: 0.3; }
  }
  
  .pulse-ring {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    border: 1px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    animation: pulseOut 3s ease-out infinite;
  }
  
  .ring-1 { width: 80px; height: 80px; animation-delay: 0s; }
  .ring-2 { width: 100px; height: 100px; animation-delay: 1s; }
  .ring-3 { width: 120px; height: 120px; animation-delay: 2s; }
  
  @keyframes pulseOut {
    0% { opacity: 0.6; transform: translate(-50%, -50%) scale(0.8); }
    100% { opacity: 0; transform: translate(-50%, -50%) scale(1.2); }
  }
  
  .empty-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }
  
  .empty-title {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 2px;
    margin: 0;
    text-shadow: 0 0 20px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.3));
  }
  
  .empty-text {
    font-size: 1rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    max-width: 400px;
    margin: 0;
    line-height: 1.6;
  }
  
  .empty-features {
    display: flex;
    gap: 2rem;
    margin: 1rem 0;
    flex-wrap: wrap;
    justify-content: center;
  }
  
  .feature {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }
  
  .feature :global(svg) {
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .deploy-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 2rem;
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    border: 1px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 10px;
    color: var(--bg-primary, #0d1117);
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.9rem;
    font-weight: 700;
    letter-spacing: 1px;
    text-decoration: none;
    cursor: pointer;
    transition: all 0.3s ease;
    position: relative;
    overflow: hidden;
    margin-top: 1rem;
  }
  
  .deploy-btn .btn-glow {
    position: absolute;
    inset: -50%;
    background: radial-gradient(circle, rgba(255,255,255,0.3) 0%, transparent 50%);
    opacity: 0;
    transition: opacity 0.3s ease;
  }
  
  .deploy-btn:hover {
    background: linear-gradient(135deg, var(--tachi-cyan, #4ecdc4), var(--tachi-cyan-bright, #6ee7df));
    box-shadow: 0 0 30px rgba(78, 205, 196, 0.5), 0 10px 40px rgba(78, 205, 196, 0.3);
    transform: translateY(-3px);
  }
  
  .deploy-btn:hover .btn-glow {
    opacity: 1;
    animation: btnGlowMove 1s ease-in-out infinite;
  }
  
  @keyframes btnGlowMove {
    0%, 100% { transform: translateX(-100%); }
    50% { transform: translateX(100%); }
  }
  
  /* Quick Actions */
  .quick-actions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
    max-width: 400px;
  }
  
  .action-link {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.875rem 1rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 8px;
    color: var(--text-primary, #e6edf3);
    text-decoration: none;
    transition: all 0.2s ease;
  }
  
  .action-link:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    border-color: var(--tachi-cyan, #4ecdc4);
    transform: translateX(4px);
  }
  
  .action-link :global(svg:first-child) {
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .action-link span {
    flex: 1;
  }
  
  .action-link :global(svg:last-child) {
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    transition: all 0.2s ease;
  }
  
  .action-link:hover :global(svg:last-child) {
    color: var(--tachi-cyan, #4ecdc4);
    transform: translateX(4px);
  }
</style>
