<script lang="ts">
  import type { SessionPhase } from '$lib/types/forge';

  export let sessionId: string | null = null;
  export let phase: SessionPhase = 'idle';

  function getPhaseTitle(phase: SessionPhase): string {
    switch (phase) {
      case 'idle': return 'Ready to Begin';
      case 'configuring': return 'Configuring Session';
      case 'drafting': return 'Drafting Phase';
      case 'critiquing': return 'Critique Phase';
      case 'deliberating': return 'Deliberation Phase';
      case 'converging': return 'Convergence Phase';
      case 'completed': return 'Session Complete';
      case 'paused': return 'Session Paused';
      case 'error': return 'Session Error';
      default: return 'Unknown Phase';
    }
  }

  function getPhaseDescription(phase: SessionPhase): string {
    switch (phase) {
      case 'idle': return 'Create a new session or select an existing one to begin.';
      case 'configuring': return 'Setting up participants and session parameters.';
      case 'drafting': return 'Participants are creating initial proposals and ideas.';
      case 'critiquing': return 'Reviewing and providing feedback on contributions.';
      case 'deliberating': return 'Discussing and refining ideas through structured debate.';
      case 'converging': return 'Working towards consensus and final decisions.';
      case 'completed': return 'Session has concluded. Review the results and insights.';
      case 'paused': return 'Session is temporarily paused. Resume when ready.';
      case 'error': return 'An error occurred. Please check session configuration.';
      default: return '';
    }
  }
</script>

<div class="main-content-area" data-testid="main-content-area" data-phase={phase}>
  <div class="content-header">
    <div class="phase-indicator">
      <div class="phase-icon" class:active={phase !== 'idle'}>
        {#if phase === 'idle'}
          ⏸️
        {:else if phase === 'configuring'}
          ⚙️
        {:else if phase === 'drafting'}
          ✏️
        {:else if phase === 'critiquing'}
          🔍
        {:else if phase === 'deliberating'}
          💬
        {:else if phase === 'converging'}
          🎯
        {:else if phase === 'completed'}
          ✅
        {:else if phase === 'paused'}
          ⏸️
        {:else if phase === 'error'}
          ⚠️
        {/if}
      </div>
      
      <div class="phase-info">
        <h1 class="phase-title">{getPhaseTitle(phase)}</h1>
        <p class="phase-description">{getPhaseDescription(phase)}</p>
      </div>
    </div>
  </div>

  <div class="content-body">
    {#if sessionId && phase !== 'idle'}
      <div class="session-workspace" data-testid="session-workspace">
        {#if phase === 'configuring'}
          <div class="configuration-area">
            <h2>Session Configuration</h2>
            <p>Configure participants, goals, and session parameters.</p>
            <!-- TODO: Add configuration components -->
          </div>
        {:else if phase === 'drafting'}
          <div class="drafting-area">
            <h2>Draft Your Contributions</h2>
            <p>Share your ideas and initial proposals.</p>
            <!-- TODO: Add drafting interface -->
          </div>
        {:else if phase === 'critiquing'}
          <div class="critique-area">
            <h2>Review & Critique</h2>
            <p>Provide constructive feedback on submitted contributions.</p>
            <!-- TODO: Add critique interface -->
          </div>
        {:else if phase === 'deliberating'}
          <div class="deliberation-area">
            <h2>Structured Deliberation</h2>
            <p>Engage in facilitated discussion to refine ideas.</p>
            <!-- TODO: Add deliberation interface -->
          </div>
        {:else if phase === 'converging'}
          <div class="convergence-area">
            <h2>Convergence & Decision</h2>
            <p>Work towards consensus and final recommendations.</p>
            <!-- TODO: Add convergence interface -->
          </div>
        {:else if phase === 'completed'}
          <div class="results-area">
            <h2>Session Results</h2>
            <p>Review the outcomes and decisions from this session.</p>
            <!-- TODO: Add results display -->
          </div>
        {:else if phase === 'paused'}
          <div class="paused-area">
            <h2>Session Paused</h2>
            <p>The session is temporarily paused. You can resume at any time.</p>
            <button type="button" class="resume-button">Resume Session</button>
          </div>
        {:else if phase === 'error'}
          <div class="error-area">
            <h2>Session Error</h2>
            <p>There was an error with the session. Please check the configuration or contact support.</p>
            <button type="button" class="retry-button">Retry</button>
          </div>
        {/if}
      </div>
    {:else}
      <div class="welcome-area" data-testid="welcome-area">
        <div class="welcome-content">
          <div class="welcome-icon">🔥</div>
          <h2>Welcome to the Forge</h2>
          <p>
            The Forge is your AI-powered deliberation workspace. Create structured sessions 
            where human participants and AI oracles collaborate to solve complex problems 
            through systematic rounds of drafting, critique, and convergence.
          </p>
          
          <div class="action-buttons">
            <button type="button" class="create-session-button">
              Create New Session
            </button>
            <button type="button" class="browse-sessions-button">
              Browse Sessions
            </button>
          </div>

          <div class="feature-highlights">
            <div class="feature">
              <div class="feature-icon">👥</div>
              <h3>Multi-Participant</h3>
              <p>Bring together humans and AI in structured collaboration</p>
            </div>
            <div class="feature">
              <div class="feature-icon">🎯</div>
              <h3>Goal-Oriented</h3>
              <p>Focus discussions around specific objectives and outcomes</p>
            </div>
            <div class="feature">
              <div class="feature-icon">⚡</div>
              <h3>AI-Facilitated</h3>
              <p>Oracle guides the process and provides expert insights</p>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .main-content-area {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .content-header {
    background: var(--header-bg, rgba(255, 255, 255, 0.02));
    border-bottom: 1px solid var(--border-color, #2a2a4a);
    padding: 1.5rem 2rem;
  }

  .phase-indicator {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .phase-icon {
    font-size: 2rem;
    opacity: 0.6;
    transition: opacity 0.2s ease;
  }

  .phase-icon.active {
    opacity: 1;
  }

  .phase-info {
    flex: 1;
  }

  .phase-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
  }

  .phase-description {
    margin: 0;
    color: var(--text-muted, #999);
    font-size: 0.95rem;
  }

  .content-body {
    flex: 1;
    overflow: auto;
    padding: 2rem;
  }

  .session-workspace {
    max-width: 800px;
    margin: 0 auto;
  }

  .session-workspace h2 {
    margin: 0 0 1rem 0;
    color: var(--forge-text, #eaeaea);
  }

  .session-workspace p {
    color: var(--text-muted, #999);
    margin-bottom: 2rem;
  }

  .welcome-area {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 400px;
  }

  .welcome-content {
    max-width: 600px;
    text-align: center;
  }

  .welcome-icon {
    font-size: 4rem;
    margin-bottom: 1.5rem;
  }

  .welcome-content h2 {
    margin: 0 0 1.5rem 0;
    font-size: 2rem;
    color: var(--forge-text, #eaeaea);
  }

  .welcome-content p {
    color: var(--text-muted, #999);
    line-height: 1.6;
    margin-bottom: 2rem;
  }

  .action-buttons {
    display: flex;
    gap: 1rem;
    justify-content: center;
    margin-bottom: 3rem;
  }

  .create-session-button,
  .browse-sessions-button,
  .resume-button,
  .retry-button {
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .create-session-button,
  .resume-button {
    background: var(--accent-color, #4a9eff);
    color: white;
    border: none;
  }

  .create-session-button:hover,
  .resume-button:hover {
    background: var(--accent-hover, #357abd);
    transform: translateY(-1px);
  }

  .browse-sessions-button,
  .retry-button {
    background: transparent;
    color: var(--forge-text, #eaeaea);
    border: 1px solid var(--border-color, #2a2a4a);
  }

  .browse-sessions-button:hover,
  .retry-button:hover {
    background: var(--hover-bg, rgba(255, 255, 255, 0.05));
    border-color: var(--accent-color, #4a9eff);
  }

  .feature-highlights {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 2rem;
    margin-top: 2rem;
  }

  .feature {
    text-align: center;
  }

  .feature-icon {
    font-size: 2rem;
    margin-bottom: 0.75rem;
  }

  .feature h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    color: var(--forge-text, #eaeaea);
  }

  .feature p {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-muted, #999);
  }

  .error-area,
  .paused-area {
    text-align: center;
    padding: 2rem;
    background: var(--panel-item-bg, rgba(255, 255, 255, 0.02));
    border-radius: 8px;
    margin-top: 2rem;
  }

  .error-area {
    border: 1px solid var(--error-color, #ef4444);
  }

  .paused-area {
    border: 1px solid var(--warning-color, #f59e0b);
  }
</style>