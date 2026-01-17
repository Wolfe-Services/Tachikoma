<script lang="ts">
  export let sessionId: string | null = null;
  import Icon from '$lib/components/common/Icon.svelte';
  import RelativeTime from '$lib/components/common/RelativeTime.svelte';
  import Spinner from '$lib/components/ui/Spinner/Spinner.svelte';
  import { forgeSessionStore, sessions, activeSession, sessionLoading, sessionError } from '$lib/stores/forgeSession';

  function handleSelect(id: string) {
    forgeSessionStore.setActiveSession(id);
  }

  $: sortedSessions = [...$sessions].sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());
</script>

<div class="session-sidebar" data-testid="session-sidebar">
  <div class="sidebar-header">
    <div class="title">
      <img 
        src="/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Blue.32.png" 
        alt="" 
        class="title-tachi-icon"
      />
      <h2>Sessions</h2>
    </div>
    {#if $sessionLoading}
      <div class="loading" aria-label="Loading sessions">
        <Spinner size={16} color="var(--tachi-cyan, #4ecdc4)" />
      </div>
    {/if}
  </div>

  {#if $sessionError}
    <div class="banner" role="alert">
      <Icon name="alert-triangle" size={16} />
      <span class="banner-text">{$sessionError}</span>
      <button type="button" class="banner-btn" on:click={() => forgeSessionStore.clearError()}>
        Dismiss
      </button>
    </div>
  {/if}

  {#if sortedSessions.length === 0}
    <div class="empty">
      <p class="empty-title">No sessions yet</p>
      <p class="empty-subtitle">Create a new Think Tank session to begin deliberation.</p>
    </div>
  {:else}
    <ul class="session-list" aria-label="Forge sessions">
      {#each sortedSessions as s (s.id)}
        <li>
          <button
            type="button"
            class="session-row"
            class:active={$activeSession?.id === s.id}
            on:click={() => handleSelect(s.id)}
            aria-current={$activeSession?.id === s.id ? 'true' : 'false'}
          >
            <div class="row-left">
              <div class="row-title">
                <span class="name">{s.name}</span>
                <span class="phase">{s.phase}</span>
              </div>
              <div class="row-subtitle">{s.goal}</div>
            </div>
            <div class="row-right">
              <RelativeTime date={s.updatedAt} />
              <span class="pill">{s.participants.length}p</span>
            </div>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .session-sidebar {
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem;
    border-radius: 12px;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.12);
    -webkit-backdrop-filter: blur(12px);
    backdrop-filter: blur(12px);
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .title-tachi-icon {
    width: 20px;
    height: 20px;
    object-fit: contain;
    filter: drop-shadow(0 0 6px var(--tachi-cyan, #4ecdc4));
  }

  .title h2 {
    margin: 0;
    font-size: 0.9rem;
    letter-spacing: 1.5px;
    text-transform: uppercase;
  }

  .banner {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.65rem 0.75rem;
    border-radius: 10px;
    background: rgba(255, 107, 107, 0.08);
    border: 1px solid rgba(255, 107, 107, 0.25);
    color: rgba(230, 237, 243, 0.85);
  }

  .banner-text {
    flex: 1;
    font-size: 0.85rem;
    color: rgba(230, 237, 243, 0.75);
  }

  .banner-btn {
    padding: 0.35rem 0.55rem;
    border-radius: 8px;
    background: transparent;
    border: 1px solid rgba(255, 107, 107, 0.35);
    color: rgba(230, 237, 243, 0.85);
    cursor: pointer;
  }

  .banner-btn:hover {
    background: rgba(255, 107, 107, 0.1);
  }

  .empty {
    padding: 1.25rem 0.75rem;
    text-align: center;
  }

  .empty-title {
    margin: 0 0 0.35rem;
    font-size: 0.95rem;
    color: rgba(230, 237, 243, 0.85);
  }

  .empty-subtitle {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.45));
  }

  .session-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .session-row {
    width: 100%;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.75rem 0.75rem;
    border-radius: 12px;
    background: rgba(22, 27, 34, 0.45);
    border: 1px solid rgba(78, 205, 196, 0.1);
    cursor: pointer;
    text-align: left;
    transition: all 0.2s ease;
  }

  .session-row:hover {
    border-color: rgba(78, 205, 196, 0.35);
    background: rgba(22, 27, 34, 0.55);
  }

  .session-row.active {
    border-color: rgba(78, 205, 196, 0.6);
    box-shadow: 0 0 0 3px rgba(78, 205, 196, 0.12);
  }

  .row-left {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .row-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .name {
    font-weight: 700;
    color: rgba(230, 237, 243, 0.9);
    font-size: 0.9rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .phase {
    font-size: 0.65rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 1px;
    text-transform: uppercase;
    color: rgba(78, 205, 196, 0.9);
    border: 1px solid rgba(78, 205, 196, 0.22);
    background: rgba(78, 205, 196, 0.08);
    padding: 0.15rem 0.4rem;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .row-subtitle {
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.55);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-right {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.35rem;
    flex-shrink: 0;
    color: rgba(230, 237, 243, 0.55);
    font-size: 0.75rem;
  }

  .pill {
    font-size: 0.65rem;
    color: rgba(230, 237, 243, 0.7);
    padding: 0.15rem 0.4rem;
    border-radius: 999px;
    background: rgba(78, 205, 196, 0.08);
    border: 1px solid rgba(78, 205, 196, 0.14);
  }
</style>