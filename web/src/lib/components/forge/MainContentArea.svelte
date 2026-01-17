<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SessionPhase, ForgeSession } from '$lib/types/forge';
  import { marked } from 'marked';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import Icon from '$lib/components/common/Icon.svelte';
  import GlassPanel from '$lib/components/ui/GlassPanel.svelte';
  import DeliberationView from './DeliberationView.svelte';

  export let sessionId: string | null = null;
  export let phase: SessionPhase = 'idle';

  // Get the active session from the store
  $: session = $forgeSessionStore.activeSession;
  $: goalHtml = session?.goal ? marked(session.goal) : '';
  $: participantCount = session?.participants?.length || 0;
  $: aiCount = session?.participants?.filter(p => p.type === 'ai').length || 0;
  $: humanCount = participantCount - aiCount;

  function getPhaseConfig(phase: SessionPhase) {
    const configs: Record<SessionPhase, { icon: string; label: string; color: string; description: string }> = {
      idle: { icon: '⏸️', label: 'READY', color: 'slate', description: 'Select or create a session' },
      configuring: { icon: '⚙️', label: 'CONFIGURING', color: 'cyan', description: 'Session ready to begin deliberation' },
      drafting: { icon: '✏️', label: 'DRAFTING', color: 'blue', description: 'Participants are creating initial proposals' },
      critiquing: { icon: '🔍', label: 'CRITIQUE', color: 'amber', description: 'Reviewing and providing feedback' },
      deliberating: { icon: '💬', label: 'DELIBERATING', color: 'purple', description: 'Structured debate in progress' },
      converging: { icon: '🎯', label: 'CONVERGING', color: 'green', description: 'Working towards consensus' },
      completed: { icon: '✅', label: 'COMPLETE', color: 'emerald', description: 'Session concluded successfully' },
      paused: { icon: '⏸️', label: 'PAUSED', color: 'yellow', description: 'Session paused - resume when ready' },
      error: { icon: '⚠️', label: 'ERROR', color: 'red', description: 'An error occurred' }
    };
    return configs[phase] || configs.idle;
  }

  $: phaseConfig = getPhaseConfig(phase);

  const dispatch = createEventDispatcher<{
    startDeliberation: { sessionId: string };
    editSession: { sessionId: string };
  }>();

  function handleStartDeliberation() {
    if (!sessionId) return;
    // Update session phase to 'drafting' to begin the deliberation process
    forgeSessionStore.updateSessionPhase(sessionId, 'drafting');
  }

  function handleEditSession() {
    if (!sessionId) return;
    dispatch('editSession', { sessionId });
  }

  function formatDate(date: Date | undefined): string {
    if (!date) return 'Unknown';
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit'
    }).format(date);
  }
</script>

<div class="main-content-area" data-testid="main-content-area" data-phase={phase}>
  {#if session && phase !== 'idle'}
    <!-- Active Session View -->
    <div class="session-view">
      <!-- Session Header -->
      <header class="session-header">
        <div class="header-left">
          <div class="session-icon">
            <img 
              src="/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Blue.32.png" 
              alt="Tachikoma" 
              class="session-tachi-icon"
            />
          </div>
          <div class="session-meta">
            <h1 class="session-title">{session.name || 'Untitled Session'}</h1>
            <div class="session-details">
              <span class="phase-badge" data-color={phaseConfig.color}>
                {phaseConfig.icon} {phaseConfig.label}
              </span>
              <span class="meta-divider">•</span>
              <span class="meta-item">{humanCount} human{humanCount !== 1 ? 's' : ''}</span>
              <span class="meta-divider">•</span>
              <span class="meta-item">{aiCount} AI agent{aiCount !== 1 ? 's' : ''}</span>
              {#if session.createdAt}
                <span class="meta-divider">•</span>
                <span class="meta-item">Created {formatDate(session.createdAt)}</span>
              {/if}
            </div>
          </div>
        </div>
        <div class="header-actions">
          {#if phase === 'configuring'}
            <button type="button" class="btn btn-secondary" on:click={handleEditSession}>
              <Icon name="settings" size={16} />
              Edit
            </button>
            <button type="button" class="btn btn-primary" on:click={handleStartDeliberation}>
              <Icon name="zap" size={16} />
              Start Deliberation
            </button>
          {:else if phase === 'paused'}
            <button type="button" class="btn btn-primary" on:click={handleStartDeliberation}>
              <Icon name="play" size={16} />
              Resume Session
            </button>
          {/if}
        </div>
      </header>

      <!-- Main Content -->
      <div class="session-content">
        <!-- Goal Section - Hero -->
        <section class="goal-section">
          <div class="section-header">
            <div class="section-title">
              <span class="section-icon">🎯</span>
              <h2>Session Goal</h2>
            </div>
            <span class="section-tag">OBJECTIVE</span>
          </div>
          <div class="goal-body">
            {#if goalHtml}
              <div class="goal-text" data-testid="session-goal">
                {@html goalHtml}
              </div>
            {:else}
              <p class="goal-empty">No goal defined for this session.</p>
            {/if}
          </div>
        </section>

        <!-- Session Info Grid -->
        <div class="info-grid">
          <!-- Participants Summary -->
          <section class="info-card participants-card">
            <div class="card-header">
              <Icon name="users" size={18} />
              <h3>Participants</h3>
              <span class="card-count">{participantCount}</span>
            </div>
            <div class="card-body">
              {#if session.participants && session.participants.length > 0}
                <ul class="participant-list">
                  {#each session.participants.slice(0, 6) as participant}
                    <li class="participant-row">
                      <span class="participant-avatar" class:ai={participant.type === 'ai'}>
                        {participant.name.charAt(0).toUpperCase()}
                      </span>
                      <div class="participant-info">
                        <span class="participant-name">{participant.name}</span>
                        <span class="participant-role">{participant.role}</span>
                      </div>
                      <span class="participant-type" class:ai={participant.type === 'ai'}>
                        {participant.type.toUpperCase()}
                      </span>
                    </li>
                  {/each}
                  {#if session.participants.length > 6}
                    <li class="participant-more">
                      +{session.participants.length - 6} more participants
                    </li>
                  {/if}
                </ul>
              {:else}
                <p class="card-empty">No participants configured</p>
              {/if}
            </div>
          </section>

          <!-- Oracle -->
          <section class="info-card oracle-card">
            <div class="card-header">
              <Icon name="brain" size={18} />
              <h3>Oracle</h3>
            </div>
            <div class="card-body">
              {#if session.oracle}
                <div class="oracle-info">
                  <div class="oracle-name">{session.oracle.name}</div>
                  <div class="oracle-type">{session.oracle.type}</div>
                  {#if session.oracle.estimatedCostPerRound}
                    <div class="oracle-cost">
                      ~${session.oracle.estimatedCostPerRound.toFixed(4)}/round
                    </div>
                  {/if}
                </div>
              {:else}
                <p class="card-empty">No oracle selected</p>
              {/if}
            </div>
          </section>

          <!-- Session Config -->
          <section class="info-card config-card">
            <div class="card-header">
              <Icon name="settings" size={18} />
              <h3>Configuration</h3>
            </div>
            <div class="card-body">
              {#if session.config}
                <dl class="config-list">
                  <div class="config-item">
                    <dt>Max Rounds</dt>
                    <dd>{session.config.maxRounds || 5}</dd>
                  </div>
                  <div class="config-item">
                    <dt>Convergence</dt>
                    <dd>{((session.config.convergenceThreshold || 0.8) * 100).toFixed(0)}%</dd>
                  </div>
                  <div class="config-item">
                    <dt>Timeout</dt>
                    <dd>{session.config.timeoutMinutes || 60}min</dd>
                  </div>
                  <div class="config-item">
                    <dt>Human Override</dt>
                    <dd>{session.config.allowHumanIntervention ? 'Enabled' : 'Disabled'}</dd>
                  </div>
                </dl>
              {:else}
                <p class="card-empty">Default configuration</p>
              {/if}
            </div>
          </section>
        </div>

        <!-- Phase-specific content: Deliberation View -->
        {#if phase === 'drafting' || phase === 'critiquing' || phase === 'deliberating' || phase === 'converging'}
          <DeliberationView {session} {phase} />
        {/if}

        {#if phase === 'completed'}
          <section class="results-section">
            <div class="section-header">
              <div class="section-title">
                <span class="section-icon">📋</span>
                <h2>Session Results</h2>
              </div>
              <span class="section-tag">COMPLETE</span>
            </div>
            <div class="results-body">
              <p>Review the outcomes and decisions from this deliberation session.</p>
              <div class="results-placeholder">
                <Icon name="file-text" size={48} />
                <p>Results display coming soon</p>
              </div>
            </div>
          </section>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Welcome / Empty State -->
    <div class="welcome-view">
      <div class="welcome-content">
        <div class="welcome-icon">🔥</div>
        <h1>Spec Forge</h1>
        <p class="welcome-tagline">Multi-model deliberation engine for spec creation and refinement</p>
        
        <div class="welcome-actions">
          <button type="button" class="btn btn-primary btn-large">
            <Icon name="zap" size={20} />
            Create New Session
          </button>
        </div>

        <div class="welcome-features">
          <div class="feature-card">
            <div class="feature-icon">👥</div>
            <h3>Multi-Agent</h3>
            <p>Bring together AI models with different perspectives</p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">🎯</div>
            <h3>Goal-Driven</h3>
            <p>Define objectives and let agents deliberate solutions</p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">⚡</div>
            <h3>Convergence</h3>
            <p>Structured rounds to reach consensus on complex topics</p>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .main-content-area {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: linear-gradient(180deg, rgba(13, 17, 23, 0.02) 0%, rgba(13, 17, 23, 0.08) 100%);
  }

  /* Session View */
  .session-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .session-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.5rem;
    padding: 1.25rem 1.75rem;
    background: rgba(13, 17, 23, 0.45);
    border-bottom: 1px solid rgba(78, 205, 196, 0.14);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-width: 0;
  }

  .session-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.18), rgba(78, 205, 196, 0.05));
    border: 1px solid rgba(78, 205, 196, 0.28);
    color: var(--tachi-cyan, #4ecdc4);
    flex-shrink: 0;
  }
  
  .session-tachi-icon {
    width: 32px;
    height: 32px;
    object-fit: contain;
    filter: drop-shadow(0 0 8px var(--tachi-cyan, #4ecdc4));
    transition: filter 0.3s ease;
  }
  
  .session-icon:hover .session-tachi-icon {
    filter: drop-shadow(0 0 12px var(--tachi-cyan, #4ecdc4)) drop-shadow(0 0 4px var(--tachi-cyan, #4ecdc4));
  }

  .session-meta {
    min-width: 0;
  }

  .session-title {
    margin: 0 0 0.35rem 0;
    font-size: 1.35rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .session-details {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.6);
  }

  .phase-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.2rem 0.6rem;
    border-radius: 999px;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    background: rgba(78, 205, 196, 0.12);
    color: var(--tachi-cyan, #4ecdc4);
    border: 1px solid rgba(78, 205, 196, 0.22);
  }

  .meta-divider {
    opacity: 0.4;
  }

  .header-actions {
    display: flex;
    gap: 0.75rem;
    flex-shrink: 0;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1rem;
    border-radius: 10px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: 1px solid transparent;
  }

  .btn-primary {
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    color: var(--bg-primary, #0d1117);
    border-color: rgba(78, 205, 196, 0.5);
  }

  .btn-primary:hover {
    background: linear-gradient(135deg, var(--tachi-cyan, #4ecdc4), var(--tachi-cyan-bright, #6ee7df));
    box-shadow: 0 0 20px rgba(78, 205, 196, 0.25);
  }

  .btn-secondary {
    background: rgba(13, 17, 23, 0.35);
    color: rgba(230, 237, 243, 0.85);
    border-color: rgba(78, 205, 196, 0.18);
  }

  .btn-secondary:hover {
    background: rgba(78, 205, 196, 0.1);
    border-color: rgba(78, 205, 196, 0.35);
  }

  .btn-large {
    padding: 0.9rem 1.5rem;
    font-size: 0.95rem;
  }

  /* Session Content */
  .session-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.75rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  /* Goal Section */
  .goal-section {
    background: rgba(13, 17, 23, 0.45);
    border: 1px solid rgba(78, 205, 196, 0.22);
    border-radius: 16px;
    overflow: hidden;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    background: rgba(78, 205, 196, 0.06);
    border-bottom: 1px solid rgba(78, 205, 196, 0.14);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .section-icon {
    font-size: 1.1rem;
  }

  .section-title h2 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .section-tag {
    font-size: 0.65rem;
    font-weight: 600;
    letter-spacing: 1px;
    color: rgba(78, 205, 196, 0.7);
    padding: 0.2rem 0.5rem;
    background: rgba(78, 205, 196, 0.1);
    border-radius: 4px;
  }

  .goal-body {
    padding: 1.25rem 1.5rem;
  }

  .goal-text {
    color: rgba(230, 237, 243, 0.9);
    line-height: 1.75;
    font-size: 1rem;
  }

  :global(.goal-text p) {
    margin: 0 0 1rem 0;
  }

  :global(.goal-text p:last-child) {
    margin-bottom: 0;
  }

  :global(.goal-text h1),
  :global(.goal-text h2),
  :global(.goal-text h3) {
    margin: 1.5rem 0 0.75rem 0;
    color: var(--text-primary, #e6edf3);
  }

  :global(.goal-text h1:first-child),
  :global(.goal-text h2:first-child),
  :global(.goal-text h3:first-child) {
    margin-top: 0;
  }

  :global(.goal-text ul),
  :global(.goal-text ol) {
    margin: 0.75rem 0;
    padding-left: 1.5rem;
  }

  :global(.goal-text li) {
    margin-bottom: 0.4rem;
  }

  :global(.goal-text code) {
    background: rgba(78, 205, 196, 0.12);
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    font-size: 0.9rem;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    color: var(--tachi-cyan, #4ecdc4);
  }

  :global(.goal-text pre) {
    background: rgba(13, 17, 23, 0.6);
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    margin: 1rem 0;
    border: 1px solid rgba(78, 205, 196, 0.14);
  }

  :global(.goal-text pre code) {
    background: none;
    padding: 0;
    color: rgba(230, 237, 243, 0.85);
  }

  :global(.goal-text strong) {
    color: var(--text-primary, #e6edf3);
    font-weight: 600;
  }

  .goal-empty {
    color: rgba(230, 237, 243, 0.45);
    font-style: italic;
  }

  /* Info Grid */
  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
  }

  .info-card {
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 14px;
    overflow: hidden;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.9rem 1rem;
    background: rgba(78, 205, 196, 0.04);
    border-bottom: 1px solid rgba(78, 205, 196, 0.1);
    color: var(--tachi-cyan, #4ecdc4);
  }

  .card-header h3 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    flex: 1;
  }

  .card-count {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    background: rgba(78, 205, 196, 0.15);
    border-radius: 999px;
  }

  .card-body {
    padding: 1rem;
  }

  .card-empty {
    color: rgba(230, 237, 243, 0.4);
    font-size: 0.85rem;
    margin: 0;
  }

  /* Participants */
  .participant-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .participant-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.6rem;
    background: rgba(22, 27, 34, 0.4);
    border-radius: 10px;
  }

  .participant-avatar {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: linear-gradient(135deg, rgba(63, 185, 80, 0.3), rgba(63, 185, 80, 0.1));
    border: 1px solid rgba(63, 185, 80, 0.3);
    color: rgba(63, 185, 80, 0.95);
    font-size: 0.8rem;
    font-weight: 600;
  }

  .participant-avatar.ai {
    background: linear-gradient(135deg, rgba(88, 166, 255, 0.3), rgba(88, 166, 255, 0.1));
    border-color: rgba(88, 166, 255, 0.3);
    color: rgba(88, 166, 255, 0.95);
  }

  .participant-info {
    flex: 1;
    min-width: 0;
  }

  .participant-name {
    display: block;
    font-size: 0.85rem;
    font-weight: 500;
    color: rgba(230, 237, 243, 0.9);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .participant-role {
    display: block;
    font-size: 0.75rem;
    color: rgba(230, 237, 243, 0.5);
  }

  .participant-type {
    font-size: 0.6rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    background: rgba(63, 185, 80, 0.12);
    color: rgba(63, 185, 80, 0.9);
  }

  .participant-type.ai {
    background: rgba(88, 166, 255, 0.12);
    color: rgba(88, 166, 255, 0.9);
  }

  .participant-more {
    text-align: center;
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.5);
    padding: 0.5rem;
  }

  /* Oracle */
  .oracle-info {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .oracle-name {
    font-size: 1rem;
    font-weight: 600;
    color: rgba(230, 237, 243, 0.9);
  }

  .oracle-type {
    font-size: 0.8rem;
    color: var(--tachi-cyan, #4ecdc4);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .oracle-cost {
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.5);
    font-family: 'JetBrains Mono', monospace;
  }

  /* Config */
  .config-list {
    margin: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.6rem;
  }

  .config-item {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .config-item dt {
    font-size: 0.7rem;
    color: rgba(230, 237, 243, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .config-item dd {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 500;
    color: rgba(230, 237, 243, 0.85);
  }

  /* Activity/Results Sections */
  .activity-section,
  .results-section {
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 14px;
    overflow: hidden;
  }

  .activity-body,
  .results-body {
    padding: 1.5rem;
    text-align: center;
  }

  .activity-description {
    color: rgba(230, 237, 243, 0.7);
    margin-bottom: 1.5rem;
  }

  .activity-placeholder,
  .results-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
    color: rgba(230, 237, 243, 0.4);
  }

  /* Welcome View */
  .welcome-view {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
  }

  .welcome-content {
    max-width: 700px;
    text-align: center;
  }

  .welcome-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .welcome-content h1 {
    margin: 0 0 0.5rem 0;
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 2px;
    text-transform: uppercase;
  }

  .welcome-tagline {
    font-size: 1rem;
    color: rgba(230, 237, 243, 0.6);
    margin-bottom: 2rem;
  }

  .welcome-actions {
    margin-bottom: 3rem;
  }

  .welcome-features {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1.5rem;
  }

  .feature-card {
    padding: 1.5rem;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 14px;
    text-align: center;
  }

  .feature-card .feature-icon {
    font-size: 2rem;
    margin-bottom: 0.75rem;
  }

  .feature-card h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    color: var(--text-primary, #e6edf3);
  }

  .feature-card p {
    margin: 0;
    font-size: 0.85rem;
    color: rgba(230, 237, 243, 0.6);
    line-height: 1.4;
  }

  @media (max-width: 768px) {
    .session-header {
      flex-direction: column;
      align-items: flex-start;
    }

    .header-actions {
      width: 100%;
    }

    .header-actions .btn {
      flex: 1;
    }

    .info-grid {
      grid-template-columns: 1fr;
    }

    .welcome-features {
      grid-template-columns: 1fr;
    }

    .config-list {
      grid-template-columns: 1fr;
    }
  }
</style>
