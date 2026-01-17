<script lang="ts">
  import { marked } from 'marked';
  import type { SessionDraft, CostEstimate } from '$lib/types/forge';
  import Icon from '$lib/components/common/Icon.svelte';

  export let draft: SessionDraft;
  export let costEstimate: CostEstimate;
  export let errors: Map<string, string[]>;

  $: goalHtml = marked(draft.goal || '');
  $: hasErrors = errors.size > 0;
  $: allErrors = Array.from(errors.values()).flat();

  let brokenAvatarIds = new Set<string>();

  function parseAvatar(avatar?: string): { src?: string; emoji?: string } {
    if (!avatar) return {};
    if (avatar.startsWith('asset:')) {
      const raw = avatar.slice('asset:'.length);
      const [src, emoji] = raw.split('|');
      return { src, emoji };
    }
    if (avatar.startsWith('/') || avatar.startsWith('http')) {
      return { src: avatar };
    }
    return { emoji: avatar };
  }

  function markAvatarBroken(id: string) {
    brokenAvatarIds = new Set([...brokenAvatarIds, id]);
  }
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
      <div class="error-header">
        <Icon name="alert-triangle" size={20} />
        <h3>Issues Found</h3>
      </div>
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
      <div class="section-header">
        <div class="section-title">
          <span class="section-icon">📋</span>
          <h3>Session Details</h3>
        </div>
      </div>
      <div class="section-content">
        <div class="detail-row">
          <span class="detail-label">Session Name</span>
          <span class="detail-value">{draft.name || 'Untitled Session'}</span>
        </div>
        <div class="detail-row goal-row">
          <span class="detail-label">Goal</span>
          <div class="goal-preview">
            {@html goalHtml}
          </div>
        </div>
      </div>
    </section>

    <!-- Participants -->
    <section class="review-section">
      <div class="section-header">
        <div class="section-title">
          <span class="section-icon">👥</span>
          <h3>Participants ({draft.participants.length})</h3>
        </div>
      </div>
      <div class="section-content">
        <div class="participant-grid">
          {#each draft.participants as participant}
            {@const avatar = parseAvatar(participant.avatar)}
            <div class="participant-card" data-testid="participant-{participant.id}">
              <div class="participant-avatar" class:ai={participant.type === 'ai'}>
                {#if avatar.src && !brokenAvatarIds.has(participant.id)}
                  <img
                    src={avatar.src}
                    alt=""
                    on:error={() => markAvatarBroken(participant.id)}
                  />
                {:else if avatar.emoji}
                  <span class="avatar-emoji">{avatar.emoji}</span>
                {:else}
                  <span class="avatar-letter">{participant.name.charAt(0).toUpperCase()}</span>
                {/if}
              </div>
              <div class="participant-info">
                <div class="participant-name">{participant.name}</div>
                <div class="participant-role">{participant.role}</div>
                <div class="participant-meta">
                  <span class="type-badge" class:ai={participant.type === 'ai'}>
                    {participant.type.toUpperCase()}
                  </span>
                  {#if participant.estimatedCostPerRound}
                    <span class="cost-badge">${participant.estimatedCostPerRound.toFixed(4)}/round</span>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </section>

    <!-- Oracle -->
    <section class="review-section">
      <div class="section-header">
        <div class="section-title">
          <span class="section-icon">🧠</span>
          <h3>Oracle</h3>
        </div>
      </div>
      <div class="section-content">
        {#if draft.oracle}
          <div class="oracle-card">
            <div class="oracle-main">
              <div class="oracle-name">{draft.oracle.name}</div>
              <div class="oracle-type">{draft.oracle.type}</div>
              {#if draft.oracle.estimatedCostPerRound}
                <div class="oracle-cost">${draft.oracle.estimatedCostPerRound.toFixed(4)}/round</div>
              {/if}
            </div>
            {#if draft.oracle.config && Object.keys(draft.oracle.config).length > 0}
              <details class="oracle-config">
                <summary class="config-toggle">Configuration</summary>
                <pre class="config-code">{JSON.stringify(draft.oracle.config, null, 2)}</pre>
              </details>
            {/if}
          </div>
        {:else}
          <div class="empty-state">No oracle selected</div>
        {/if}
      </div>
    </section>

    <!-- Session Configuration -->
    <section class="review-section">
      <div class="section-header">
        <div class="section-title">
          <span class="section-icon">⚙️</span>
          <h3>Session Configuration</h3>
        </div>
      </div>
      <div class="section-content">
        <div class="config-grid">
          <div class="config-item">
            <span class="config-label">Maximum Rounds:</span>
            <span class="config-value">{draft.config.maxRounds}</span>
          </div>
          <div class="config-item">
            <span class="config-label">Convergence Threshold:</span>
            <span class="config-value">{(draft.config.convergenceThreshold * 100).toFixed(0)}%</span>
          </div>
          <div class="config-item">
            <span class="config-label">Human Intervention:</span>
            <span class="config-value">{draft.config.allowHumanIntervention ? 'Allowed' : 'Disabled'}</span>
          </div>
          <div class="config-item">
            <span class="config-label">Auto-save Interval:</span>
            <span class="config-value">{draft.config.autoSaveInterval / 1000}s</span>
          </div>
          <div class="config-item">
            <span class="config-label">Timeout:</span>
            <span class="config-value">{draft.config.timeoutMinutes} minutes</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Cost Estimate -->
    <section class="review-section cost-section">
      <div class="section-header">
        <div class="section-title">
          <span class="section-icon">💰</span>
          <h3>Cost Estimate</h3>
        </div>
      </div>
      <div class="section-content">
        <div class="cost-breakdown">
          <div class="cost-row primary">
            <span class="cost-label">Estimated Total Cost:</span>
            <span class="cost-amount">${costEstimate.estimated.toFixed(4)}</span>
          </div>
          <div class="cost-row">
            <span class="cost-label">Cost per Round:</span>
            <span class="cost-amount">${costEstimate.perRound.toFixed(4)}</span>
          </div>
          <div class="cost-row">
            <span class="cost-label">Maximum Possible Cost:</span>
            <span class="cost-amount">${costEstimate.maximum.toFixed(4)}</span>
          </div>
        </div>
        <div class="cost-note">
          <span class="note-icon">💡</span>
          <span>Estimated cost assumes ~70% of maximum rounds. Actual cost may vary based on convergence.</span>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  .session-review {
    max-width: 900px;
    margin: 0 auto;
  }

  .review-header {
    margin-bottom: 2rem;
  }

  .review-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
  }

  .review-description {
    color: rgba(230, 237, 243, 0.7);
    line-height: 1.5;
    margin: 0;
  }

  /* Error Summary */
  .error-summary {
    background: rgba(255, 107, 107, 0.08);
    border: 1px solid rgba(255, 107, 107, 0.35);
    border-radius: 14px;
    padding: 1.25rem;
    margin-bottom: 2rem;
  }

  .error-header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    color: rgba(255, 107, 107, 0.95);
    margin-bottom: 0.75rem;
  }

  .error-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .error-summary p {
    color: rgba(230, 237, 243, 0.7);
    margin: 0 0 1rem 0;
    font-size: 0.9rem;
  }

  .error-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .error-list li {
    padding: 0.6rem 0.9rem;
    background: rgba(255, 107, 107, 0.1);
    border-radius: 8px;
    color: rgba(255, 107, 107, 0.95);
    font-size: 0.85rem;
  }

  /* Review Sections */
  .review-sections {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .review-section {
    background: rgba(13, 17, 23, 0.45);
    border: 1px solid rgba(78, 205, 196, 0.14);
    border-radius: 16px;
    overflow: hidden;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    background: rgba(78, 205, 196, 0.06);
    border-bottom: 1px solid rgba(78, 205, 196, 0.12);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .section-icon {
    font-size: 1.15rem;
  }

  .section-title h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
  }

  .section-content {
    padding: 1.25rem;
  }

  /* Session Details */
  .detail-row {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .detail-row:last-child {
    margin-bottom: 0;
  }

  .detail-label {
    font-size: 0.8rem;
    font-weight: 500;
    color: rgba(78, 205, 196, 0.9);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    min-width: 100px;
    flex-shrink: 0;
  }

  .detail-value {
    color: var(--text-primary, #e6edf3);
    font-weight: 500;
  }

  .goal-row {
    flex-direction: column;
  }

  .goal-preview {
    background: rgba(13, 17, 23, 0.55);
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 10px;
    padding: 1rem 1.25rem;
    color: rgba(230, 237, 243, 0.9);
    font-size: 0.95rem;
    line-height: 1.65;
  }

  :global(.goal-preview p) {
    margin: 0 0 0.75rem 0;
  }

  :global(.goal-preview p:last-child) {
    margin-bottom: 0;
  }

  :global(.goal-preview ul),
  :global(.goal-preview ol) {
    margin: 0.75rem 0;
    padding-left: 1.5rem;
  }

  :global(.goal-preview code) {
    background: rgba(78, 205, 196, 0.12);
    padding: 0.15rem 0.35rem;
    border-radius: 4px;
    font-size: 0.85rem;
    color: var(--tachi-cyan, #4ecdc4);
  }

  /* Participants */
  .participant-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
  }

  .participant-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: rgba(22, 27, 34, 0.5);
    border: 1px solid rgba(78, 205, 196, 0.1);
    border-radius: 12px;
  }

  .participant-avatar {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, rgba(63, 185, 80, 0.25), rgba(63, 185, 80, 0.08));
    border: 2px solid rgba(63, 185, 80, 0.35);
    color: rgba(63, 185, 80, 0.95);
    font-size: 1.1rem;
  }

  .participant-avatar.ai {
    background: linear-gradient(135deg, rgba(88, 166, 255, 0.25), rgba(88, 166, 255, 0.08));
    border-color: rgba(88, 166, 255, 0.35);
    color: rgba(88, 166, 255, 0.95);
  }

  .participant-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-emoji {
    font-size: 1.4rem;
    line-height: 1;
  }

  .avatar-letter {
    font-weight: 700;
    font-size: 1.1rem;
  }

  .participant-info {
    flex: 1;
    min-width: 0;
  }

  .participant-name {
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    margin-bottom: 0.2rem;
  }

  .participant-role {
    font-size: 0.85rem;
    color: rgba(230, 237, 243, 0.6);
    margin-bottom: 0.5rem;
  }

  .participant-meta {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .type-badge {
    font-size: 0.65rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    padding: 0.2rem 0.5rem;
    border-radius: 6px;
    background: rgba(63, 185, 80, 0.12);
    color: rgba(63, 185, 80, 0.95);
  }

  .type-badge.ai {
    background: rgba(88, 166, 255, 0.12);
    color: rgba(88, 166, 255, 0.95);
  }

  .cost-badge {
    font-size: 0.75rem;
    color: rgba(230, 237, 243, 0.5);
    font-family: 'JetBrains Mono', monospace;
  }

  /* Oracle */
  .oracle-card {
    background: rgba(22, 27, 34, 0.5);
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 12px;
    overflow: hidden;
  }

  .oracle-main {
    padding: 1rem 1.25rem;
  }

  .oracle-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    margin-bottom: 0.25rem;
  }

  .oracle-type {
    font-size: 0.85rem;
    color: var(--tachi-cyan, #4ecdc4);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 0.5rem;
  }

  .oracle-cost {
    font-size: 0.85rem;
    color: rgba(230, 237, 243, 0.5);
    font-family: 'JetBrains Mono', monospace;
  }

  .oracle-config {
    border-top: 1px solid rgba(78, 205, 196, 0.1);
  }

  .config-toggle {
    display: block;
    padding: 0.75rem 1.25rem;
    background: rgba(78, 205, 196, 0.04);
    cursor: pointer;
    font-weight: 500;
    color: rgba(230, 237, 243, 0.8);
    font-size: 0.85rem;
  }

  .config-toggle:hover {
    background: rgba(78, 205, 196, 0.08);
  }

  .config-code {
    margin: 0;
    padding: 1rem 1.25rem;
    background: rgba(13, 17, 23, 0.5);
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.7);
    font-family: 'JetBrains Mono', monospace;
    overflow-x: auto;
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    color: rgba(230, 237, 243, 0.4);
    font-style: italic;
  }

  /* Session Configuration */
  .config-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .config-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.65rem 1rem;
    background: rgba(22, 27, 34, 0.5);
    border: 1px solid rgba(78, 205, 196, 0.1);
    border-radius: 10px;
    flex: 1 1 auto;
    min-width: fit-content;
  }

  .config-label {
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.6);
  }

  .config-value {
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
  }

  /* Cost Section */
  .cost-section {
    border-color: rgba(78, 205, 196, 0.25);
  }

  .cost-section .section-header {
    background: rgba(78, 205, 196, 0.1);
  }

  .cost-breakdown {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    margin-bottom: 1rem;
  }

  .cost-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: rgba(22, 27, 34, 0.5);
    border: 1px solid rgba(78, 205, 196, 0.1);
    border-radius: 10px;
  }

  .cost-row.primary {
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.25), rgba(78, 205, 196, 0.12));
    border-color: rgba(78, 205, 196, 0.35);
  }

  .cost-label {
    font-size: 0.85rem;
    color: rgba(230, 237, 243, 0.7);
  }

  .cost-row.primary .cost-label {
    color: rgba(230, 237, 243, 0.9);
    font-weight: 500;
  }

  .cost-amount {
    font-family: 'JetBrains Mono', monospace;
    font-weight: 600;
    font-size: 1rem;
    color: rgba(230, 237, 243, 0.8);
  }

  .cost-row.primary .cost-amount {
    font-size: 1.25rem;
    color: var(--tachi-cyan, #4ecdc4);
  }

  .cost-note {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
    padding: 0.85rem 1rem;
    background: rgba(78, 205, 196, 0.06);
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 10px;
    font-size: 0.85rem;
    color: rgba(230, 237, 243, 0.65);
    line-height: 1.4;
  }

  .note-icon {
    flex-shrink: 0;
    font-size: 1rem;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .participant-grid {
      grid-template-columns: 1fr;
    }

    .config-grid {
      flex-direction: column;
    }

    .config-item {
      width: 100%;
      justify-content: space-between;
    }
  }
</style>
