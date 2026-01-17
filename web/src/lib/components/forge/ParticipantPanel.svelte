<script lang="ts">
  import type { ForgeSession, Participant } from '$lib/types/forge';

  export let session: ForgeSession | null = null;

  $: participants = session?.participants || [];

  const brokenAvatarIds = new Set<string>();

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
    brokenAvatarIds.add(id);
  }
</script>

<div class="participant-panel" data-testid="participant-panel">
  <h3>Participants</h3>
  
  {#if participants.length > 0}
    <div class="participant-list">
      {#each participants as participant}
        {@const a = parseAvatar(participant.avatar)}
        <div
          class="participant-item"
          class:active={participant.status === 'active'}
          class:thinking={participant.status === 'thinking'}
          class:contributing={participant.status === 'contributing'}
          data-testid="participant-{participant.id}"
        >
          <div class="participant-avatar">
            {#if a.src && !brokenAvatarIds.has(participant.id)}
              <img
                src={a.src}
                alt="{participant.name} avatar"
                on:error={() => markAvatarBroken(participant.id)}
              />
            {:else if a.emoji}
              <div class="avatar-placeholder emoji" aria-hidden="true">{a.emoji}</div>
            {:else}
              <div class="avatar-placeholder">
                {participant.name.charAt(0).toUpperCase()}
              </div>
            {/if}
            <div class="status-indicator" class:ai={participant.type === 'ai'}></div>
          </div>
          
          <div class="participant-info">
            <div class="participant-name">{participant.name}</div>
            <div class="participant-role">{participant.role}</div>
            <div class="participant-status">{participant.status}</div>
          </div>
        </div>
      {/each}
    </div>

    {#if session?.oracle}
      <div class="oracle-section">
        <h4>Oracle</h4>
        <div class="oracle-item">
          <div class="oracle-info">
            <div class="oracle-name">{session.oracle.name}</div>
            <div class="oracle-type">{session.oracle.type}</div>
          </div>
        </div>
      </div>
    {/if}
  {:else}
    <div class="no-participants">
      <p>No participants in this session</p>
    </div>
  {/if}
</div>

<style>
  .participant-panel {
    padding: 1rem;
    height: 100%;
    overflow-y: auto;
    border-top: 1px solid var(--border-color, #2a2a4a);
  }

  .participant-panel h3 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    color: var(--forge-text, #eaeaea);
  }

  .participant-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .participant-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 6px;
    background: var(--panel-item-bg, rgba(255, 255, 255, 0.05));
    transition: all 0.2s ease;
  }

  .participant-item.active {
    background: var(--success-color, #22c55e);
    color: var(--success-text, #ffffff);
  }

  .participant-item.thinking {
    background: var(--warning-color, #f59e0b);
    color: var(--warning-text, #ffffff);
  }

  .participant-item.contributing {
    background: var(--accent-color, #4a9eff);
    color: var(--accent-text, #ffffff);
  }

  .participant-avatar {
    position: relative;
    width: 40px;
    height: 40px;
    flex-shrink: 0;
  }

  .participant-avatar img {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: var(--accent-color, #4a9eff);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    color: white;
    font-size: 1.2rem;
  }

  .avatar-placeholder.emoji {
    background: rgba(13, 17, 23, 0.55);
    border: 1px solid rgba(78, 205, 196, 0.14);
    color: var(--text-primary, #e6edf3);
    font-weight: 600;
  }

  .status-indicator {
    position: absolute;
    bottom: -2px;
    right: -2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--success-color, #22c55e);
    border: 2px solid var(--panel-bg, #16213e);
  }

  .status-indicator.ai {
    background: var(--info-color, #3b82f6);
  }

  .participant-info {
    flex: 1;
    min-width: 0;
  }

  .participant-name {
    font-weight: 600;
    margin-bottom: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .participant-role {
    font-size: 0.875rem;
    opacity: 0.8;
    margin-bottom: 0.25rem;
  }

  .participant-status {
    font-size: 0.75rem;
    text-transform: capitalize;
    opacity: 0.7;
  }

  .oracle-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color, #2a2a4a);
  }

  .oracle-section h4 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    color: var(--forge-text, #eaeaea);
  }

  .oracle-item {
    padding: 0.75rem;
    border-radius: 6px;
    background: var(--info-color, #3b82f6);
    color: white;
  }

  .oracle-name {
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .oracle-type {
    font-size: 0.875rem;
    opacity: 0.8;
    text-transform: uppercase;
  }

  .no-participants {
    text-align: center;
    color: var(--text-muted, #999);
    padding: 2rem 1rem;
  }
</style>