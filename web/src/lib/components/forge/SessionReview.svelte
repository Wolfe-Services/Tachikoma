<script lang="ts">
  import { marked } from 'marked';
  import type { SessionDraft, CostEstimate } from '$lib/types/forge';

  export let draft: SessionDraft;
  export let costEstimate: CostEstimate;
  export let errors: Map<string, string[]>;

  $: goalHtml = marked(draft.goal || '');
  $: hasErrors = errors.size > 0;
  $: allErrors = Array.from(errors.values()).flat();
</script>

<div class="session-review" data-testid="session-review">
  <div class="review-header">
    <h2>Review & Start Session</h2>
    <p class="review-description">
      Review your session configuration before starting the deliberation. Make sure all details are correct.
    </p>
  </div>

  {#if hasErrors}
    <div class="error-summary" role="alert">
      <h3>⚠️ Issues Found</h3>
      <p>Please resolve the following issues before starting your session:</p>
      <ul class="error-list">
        {#each allErrors as error}
          <li>{error}</li>
        {/each}
      </ul>
    </div>
  {/if}

  <div class="review-sections">
    <!-- Session Details -->
    <section class="review-section">
      <h3 class="section-title">
        <span class="section-icon">📋</span>
        Session Details
      </h3>
      <div class="section-content">
        <div class="detail-item">
          <label>Session Name:</label>
          <span class="detail-value">{draft.name || 'Untitled Session'}</span>
        </div>
        <div class="detail-item goal-detail">
          <label>Goal:</label>
          <div class="goal-preview">
            {@html goalHtml}
          </div>
        </div>
      </div>
    </section>

    <!-- Participants -->
    <section class="review-section">
      <h3 class="section-title">
        <span class="section-icon">👥</span>
        Participants ({draft.participants.length})
      </h3>
      <div class="section-content">
        <div class="participant-grid">
          {#each draft.participants as participant}
            <div class="participant-card" data-testid="participant-{participant.id}">
              <div class="participant-avatar">
                {#if participant.avatar}
                  <img src={participant.avatar} alt="{participant.name} avatar" />
                {:else}
                  <div class="avatar-placeholder">
                    {participant.name.charAt(0).toUpperCase()}
                  </div>
                {/if}
              </div>
              <div class="participant-info">
                <div class="participant-name">{participant.name}</div>
                <div class="participant-role">{participant.role}</div>
                <div class="participant-type">{participant.type}</div>
                {#if participant.estimatedCostPerRound}
                  <div class="participant-cost">
                    ${participant.estimatedCostPerRound.toFixed(4)}/round
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    </section>

    <!-- Oracle -->
    <section class="review-section">
      <h3 class="section-title">
        <span class="section-icon">🧠</span>
        Oracle
      </h3>
      <div class="section-content">
        {#if draft.oracle}
          <div class="oracle-card">
            <div class="oracle-info">
              <div class="oracle-name">{draft.oracle.name}</div>
              <div class="oracle-type">{draft.oracle.type}</div>
              {#if draft.oracle.estimatedCostPerRound}
                <div class="oracle-cost">
                  ${draft.oracle.estimatedCostPerRound.toFixed(4)}/round
                </div>
              {/if}
            </div>
            <div class="oracle-config">
              <summary class="config-summary">Configuration</summary>
              <pre class="config-details">{JSON.stringify(draft.oracle.config, null, 2)}</pre>
            </div>
          </div>
        {:else}
          <div class="no-oracle">No oracle selected</div>
        {/if}
      </div>
    </section>

    <!-- Session Configuration -->
    <section class="review-section">
      <h3 class="section-title">
        <span class="section-icon">⚙️</span>
        Session Configuration
      </h3>
      <div class="section-content">
        <div class="config-grid">
          <div class="config-item">
            <label>Maximum Rounds:</label>
            <span class="config-value">{draft.config.maxRounds}</span>
          </div>
          <div class="config-item">
            <label>Convergence Threshold:</label>
            <span class="config-value">{(draft.config.convergenceThreshold * 100).toFixed(0)}%</span>
          </div>
          <div class="config-item">
            <label>Human Intervention:</label>
            <span class="config-value">{draft.config.allowHumanIntervention ? 'Allowed' : 'Disabled'}</span>
          </div>
          <div class="config-item">
            <label>Auto-save Interval:</label>
            <span class="config-value">{draft.config.autoSaveInterval / 1000}s</span>
          </div>
          <div class="config-item">
            <label>Timeout:</label>
            <span class="config-value">{draft.config.timeoutMinutes} minutes</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Cost Estimate -->
    <section class="review-section cost-section">
      <h3 class="section-title">
        <span class="section-icon">💰</span>
        Cost Estimate
      </h3>
      <div class="section-content">
        <div class="cost-breakdown">
          <div class="cost-item primary">
            <label>Estimated Total Cost:</label>
            <span class="cost-value primary">${costEstimate.estimated.toFixed(4)}</span>
          </div>
          <div class="cost-item">
            <label>Cost per Round:</label>
            <span class="cost-value">${costEstimate.perRound.toFixed(4)}</span>
          </div>
          <div class="cost-item">
            <label>Maximum Possible Cost:</label>
            <span class="cost-value">${costEstimate.maximum.toFixed(4)}</span>
          </div>
        </div>
        <div class="cost-note">
          <span class="note-icon">💡</span>
          Estimated cost assumes ~70% of maximum rounds. Actual cost may vary based on convergence.
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  .session-review {
    max-width: 800px;
    margin: 0 auto;
  }

  .review-header {
    margin-bottom: 2rem;
  }

  .review-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--text-primary);
  }

  .review-description {
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .error-summary {
    background: var(--error-bg, #fef2f2);
    border: 1px solid var(--error-color, #dc2626);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .error-summary h3 {
    color: var(--error-color, #dc2626);
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .error-summary p {
    color: var(--error-color, #dc2626);
    margin: 0 0 1rem 0;
  }

  .error-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .error-list li {
    background: rgba(220, 38, 38, 0.1);
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    margin-bottom: 0.5rem;
    color: var(--error-color, #dc2626);
    font-size: 0.875rem;
  }

  .review-sections {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .review-section {
    background: var(--card-bg, white);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.5rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .section-icon {
    font-size: 1.25rem;
  }

  .section-content {
    padding: 1.5rem;
  }

  .detail-item {
    display: flex;
    margin-bottom: 1rem;
  }

  .detail-item:last-child {
    margin-bottom: 0;
  }

  .detail-item label {
    font-weight: 500;
    color: var(--text-secondary);
    min-width: 120px;
    flex-shrink: 0;
  }

  .detail-value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .goal-detail {
    flex-direction: column;
  }

  .goal-detail label {
    margin-bottom: 0.75rem;
  }

  .goal-preview {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 1rem;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .participant-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
  }

  .participant-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
  }

  .participant-avatar {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
  }

  .participant-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 100%;
    height: 100%;
    background: var(--primary-color);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 1.25rem;
  }

  .participant-info {
    flex: 1;
    min-width: 0;
  }

  .participant-name {
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.25rem;
  }

  .participant-role {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin-bottom: 0.25rem;
  }

  .participant-type {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    background: var(--info-bg, #eff6ff);
    color: var(--info-color, #2563eb);
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .participant-cost {
    margin-top: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: monospace;
  }

  .oracle-card {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    overflow: hidden;
  }

  .oracle-info {
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .oracle-name {
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.25rem;
  }

  .oracle-type {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
  }

  .oracle-cost {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: monospace;
  }

  .oracle-config {
    background: var(--code-bg, #f8fafc);
  }

  .config-summary {
    display: block;
    padding: 0.75rem 1rem;
    background: var(--tertiary-bg, #f1f5f9);
    border-bottom: 1px solid var(--border-color);
    cursor: pointer;
    font-weight: 500;
    color: var(--text-primary);
  }

  .config-details {
    padding: 1rem;
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-secondary);
    background: transparent;
    overflow-x: auto;
  }

  .no-oracle {
    color: var(--text-muted);
    font-style: italic;
    text-align: center;
    padding: 2rem;
  }

  .config-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .config-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
  }

  .config-item label {
    font-weight: 500;
    color: var(--text-secondary);
  }

  .config-value {
    font-weight: 600;
    color: var(--text-primary);
  }

  .cost-section {
    background: var(--success-bg, #f0fdf4);
    border-color: var(--success-color, #22c55e);
  }

  .cost-breakdown {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .cost-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: var(--card-bg, white);
    border: 1px solid var(--border-color);
    border-radius: 6px;
  }

  .cost-item.primary {
    background: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  .cost-item.primary label,
  .cost-item.primary .cost-value {
    color: white;
  }

  .cost-item label {
    font-weight: 500;
  }

  .cost-value {
    font-family: monospace;
    font-weight: 600;
    font-size: 1.125rem;
  }

  .cost-value.primary {
    font-size: 1.25rem;
  }

  .cost-note {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: var(--info-bg, #eff6ff);
    color: var(--info-color, #2563eb);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .note-icon {
    flex-shrink: 0;
  }

  /* Global styles for goal preview */
  :global(.goal-preview h1),
  :global(.goal-preview h2),
  :global(.goal-preview h3) {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
  }

  :global(.goal-preview h1:first-child),
  :global(.goal-preview h2:first-child),
  :global(.goal-preview h3:first-child) {
    margin-top: 0;
  }

  :global(.goal-preview ul),
  :global(.goal-preview ol) {
    margin: 1rem 0;
    padding-left: 2rem;
  }

  :global(.goal-preview p) {
    margin: 1rem 0;
  }

  :global(.goal-preview p:first-child) {
    margin-top: 0;
  }

  :global(.goal-preview p:last-child) {
    margin-bottom: 0;
  }

  :global(.goal-preview code) {
    background: var(--code-bg, #f3f4f6);
    padding: 0.125rem 0.25rem;
    border-radius: 3px;
    font-size: 0.875rem;
  }

  :global(.goal-preview pre) {
    background: var(--code-bg, #f3f4f6);
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
    margin: 1rem 0;
  }

  /* Responsive design */
  @media (max-width: 768px) {
    .participant-grid {
      grid-template-columns: 1fr;
    }

    .config-grid {
      grid-template-columns: 1fr;
    }

    .cost-item,
    .config-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.5rem;
    }
  }
</style>