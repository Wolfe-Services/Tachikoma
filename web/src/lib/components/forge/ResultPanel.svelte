<script lang="ts">
  import type { ForgeSession } from '$lib/types/forge';

  export let session: ForgeSession | null = null;
  export let visible: boolean = false;

  $: hasResults = session?.hasResults ?? false;
  $: completedRounds = session?.rounds.filter(r => r.status === 'completed') ?? [];
  $: totalContributions = completedRounds.reduce((sum, round) => sum + round.contributions.length, 0);
  $: totalCritiques = completedRounds.reduce((sum, round) => sum + round.critiques.length, 0);

  function formatTimestamp(date: Date): string {
    return new Intl.DateTimeFormat('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    }).format(date);
  }

  function getContributionTypeColor(type: string): string {
    switch (type) {
      case 'proposal': return '#4a9eff';
      case 'refinement': return '#f39c12';
      case 'alternative': return '#e74c3c';
      default: return '#95a5a6';
    }
  }

  function getCritiqueColor(severity: string): string {
    switch (severity) {
      case 'critical': return '#e74c3c';
      case 'concern': return '#f39c12';
      case 'suggestion': return '#3498db';
      case 'info': return '#95a5a6';
      default: return '#95a5a6';
    }
  }
</script>

<div class="result-panel" class:visible>
  <header class="panel-header">
    <h2>Session Results</h2>
    {#if session}
      <div class="session-status">
        <span class="phase-badge phase-{session.phase}">{session.phase}</span>
      </div>
    {/if}
  </header>

  <div class="panel-content">
    {#if !session}
      <div class="empty-state">
        <p>No active session</p>
      </div>
    {:else if !hasResults}
      <div class="empty-state">
        <p>No results yet</p>
        <small>Results will appear as the session progresses</small>
      </div>
    {:else}
      <div class="results-summary">
        <div class="stat-card">
          <div class="stat-value">{completedRounds.length}</div>
          <div class="stat-label">Rounds Completed</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">{totalContributions}</div>
          <div class="stat-label">Contributions</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">{totalCritiques}</div>
          <div class="stat-label">Critiques</div>
        </div>
      </div>

      <div class="results-sections">
        <section class="rounds-section">
          <h3>Round History</h3>
          <div class="rounds-list">
            {#each completedRounds as round (round.id)}
              <div class="round-item">
                <div class="round-header">
                  <span class="round-number">Round {round.number}</span>
                  <span class="round-duration">
                    {#if round.endTime}
                      {Math.round((round.endTime.getTime() - round.startTime.getTime()) / 60000)}m
                    {/if}
                  </span>
                </div>
                
                <div class="round-contributions">
                  {#each round.contributions as contribution (contribution.id)}
                    <div class="contribution-item">
                      <div class="contribution-header">
                        <div 
                          class="contribution-type"
                          style="background-color: {getContributionTypeColor(contribution.type)}"
                        >
                          {contribution.type}
                        </div>
                        <span class="timestamp">
                          {formatTimestamp(contribution.timestamp)}
                        </span>
                      </div>
                      <div class="contribution-content">
                        {contribution.content.substring(0, 100)}...
                      </div>
                    </div>
                  {/each}
                </div>

                <div class="round-critiques">
                  {#each round.critiques as critique (critique.id)}
                    <div class="critique-item">
                      <div class="critique-header">
                        <div 
                          class="critique-severity"
                          style="background-color: {getCritiqueColor(critique.severity)}"
                        >
                          {critique.severity}
                        </div>
                        <span class="timestamp">
                          {formatTimestamp(critique.timestamp)}
                        </span>
                      </div>
                      <div class="critique-content">
                        {critique.content.substring(0, 80)}...
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </section>

        <section class="insights-section">
          <h3>Key Insights</h3>
          <div class="insights-placeholder">
            <p>Insights will be generated automatically as the session progresses.</p>
          </div>
        </section>
      </div>
    {/if}
  </div>
</div>

<style>
  .result-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color, #2a2a4a);
    background: var(--panel-header-bg, #1a1a2e);
  }

  .panel-header h2 {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
    margin: 0;
  }

  .session-status {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .phase-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: capitalize;
  }

  .phase-idle { background: #374151; color: #9CA3AF; }
  .phase-configuring { background: #1E40AF; color: #DBEAFE; }
  .phase-drafting { background: #059669; color: #D1FAE5; }
  .phase-critiquing { background: #DC2626; color: #FEE2E2; }
  .phase-deliberating { background: #7C2D12; color: #FED7AA; }
  .phase-converging { background: #7C3AED; color: #EDE9FE; }
  .phase-completed { background: #166534; color: #DCFCE7; }
  .phase-paused { background: #CA8A04; color: #FEF3C7; }
  .phase-error { background: #DC2626; color: #FEE2E2; }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 2rem;
    color: var(--text-muted, #6b7280);
  }

  .empty-state small {
    margin-top: 0.5rem;
    font-size: 0.875rem;
    opacity: 0.7;
  }

  .results-summary {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    flex: 1;
    background: var(--card-bg, #1e293b);
    border: 1px solid var(--border-color, #2a2a4a);
    border-radius: 8px;
    padding: 1rem;
    text-align: center;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--accent-color, #4a9eff);
    margin-bottom: 0.25rem;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .results-sections {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .rounds-section h3,
  .insights-section h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
    margin: 0 0 1rem 0;
  }

  .rounds-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .round-item {
    background: var(--card-bg, #1e293b);
    border: 1px solid var(--border-color, #2a2a4a);
    border-radius: 8px;
    padding: 1rem;
  }

  .round-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .round-number {
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
  }

  .round-duration {
    font-size: 0.875rem;
    color: var(--text-muted, #6b7280);
  }

  .round-contributions,
  .round-critiques {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .contribution-item,
  .critique-item {
    background: var(--sub-card-bg, #0f172a);
    border-radius: 6px;
    padding: 0.75rem;
  }

  .contribution-header,
  .critique-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .contribution-type,
  .critique-severity {
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: capitalize;
    color: white;
  }

  .timestamp {
    font-size: 0.75rem;
    color: var(--text-muted, #6b7280);
  }

  .contribution-content,
  .critique-content {
    font-size: 0.875rem;
    color: var(--forge-text, #eaeaea);
    line-height: 1.4;
  }

  .insights-placeholder {
    background: var(--card-bg, #1e293b);
    border: 1px dashed var(--border-color, #2a2a4a);
    border-radius: 8px;
    padding: 2rem;
    text-align: center;
    color: var(--text-muted, #6b7280);
  }

  @media (max-width: 768px) {
    .results-summary {
      flex-direction: column;
      gap: 0.5rem;
    }

    .stat-card {
      padding: 0.75rem;
    }

    .stat-value {
      font-size: 1.25rem;
    }
  }
</style>