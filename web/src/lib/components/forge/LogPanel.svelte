<script lang="ts">
  import { derived } from 'svelte/store';
  import Icon from '$lib/components/common/Icon.svelte';
  import { forgeService } from '$lib/services/forgeService';
  import type { ForgeSession } from '$lib/types/forge';

  export let session: ForgeSession | null = null;

  const forgeState = forgeService.state;
  const forgeMessages = forgeService.messages;

  const filtered = derived([forgeMessages], ([$msgs]) => {
    if (!session) return $msgs;
    return $msgs.filter(m => m.sessionId === session.id);
  });
</script>

<div class="log-panel">
  <header class="log-header">
    <div class="title">
      <Icon name="terminal" size={16} />
      <span>Forge Logs</span>
    </div>
    <div class="meta">
      {#if $forgeState.useMockMode}
        <span class="chip chip-warn">Mock</span>
      {:else}
        <span class="chip chip-ok">IPC</span>
      {/if}
      {#if $forgeState.isDeliberating}
        <span class="chip chip-run">Running</span>
      {/if}
      {#if $forgeState.error}
        <span class="chip chip-err">Error</span>
      {/if}
    </div>
  </header>

  <div class="log-body">
    {#if $forgeState.error}
      <div class="error-row">
        <Icon name="alert-triangle" size={14} />
        <span>{$forgeState.error}</span>
      </div>
    {/if}

    {#if $filtered.length === 0}
      <div class="empty">
        <div class="empty-title">No log events yet</div>
        <div class="empty-subtitle">Start a deliberation to see the raw event stream.</div>
      </div>
    {:else}
      <ul class="log-list">
        {#each $filtered as m (m.messageId)}
          <li class="log-item">
            <span class="ts">{new Date(m.timestamp).toLocaleTimeString()}</span>
            <span class="who">{m.participantName}</span>
            <span class="type">{m.type}/{m.status}</span>
            <span class="msg">{(m.contentDelta ?? m.content ?? '').replaceAll('\n', ' ').slice(0, 160)}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .log-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: rgba(13, 17, 23, 0.45);
    border-top: 1px solid rgba(78, 205, 196, 0.14);
  }

  .log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.65rem 0.9rem;
    border-bottom: 1px solid rgba(78, 205, 196, 0.12);
    background: rgba(78, 205, 196, 0.04);
  }

  .title {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--tachi-cyan, #4ecdc4);
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    font-size: 0.75rem;
  }

  .meta {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .chip {
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.15rem 0.45rem;
    border-radius: 999px;
    border: 1px solid transparent;
    letter-spacing: 0.4px;
    text-transform: uppercase;
  }

  .chip-ok {
    color: rgba(63, 185, 80, 0.95);
    background: rgba(63, 185, 80, 0.12);
    border-color: rgba(63, 185, 80, 0.22);
  }

  .chip-warn {
    color: rgba(243, 156, 18, 0.95);
    background: rgba(243, 156, 18, 0.12);
    border-color: rgba(243, 156, 18, 0.22);
  }

  .chip-run {
    color: rgba(78, 205, 196, 0.95);
    background: rgba(78, 205, 196, 0.12);
    border-color: rgba(78, 205, 196, 0.22);
  }

  .chip-err {
    color: rgba(255, 107, 107, 0.95);
    background: rgba(255, 107, 107, 0.12);
    border-color: rgba(255, 107, 107, 0.22);
  }

  .log-body {
    flex: 1;
    overflow: auto;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    font-size: 0.78rem;
    color: rgba(230, 237, 243, 0.78);
  }

  .error-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.55rem 0.75rem;
    color: rgba(255, 107, 107, 0.95);
    border-bottom: 1px solid rgba(255, 107, 107, 0.2);
    background: rgba(255, 107, 107, 0.06);
  }

  .empty {
    padding: 0.9rem;
    color: rgba(230, 237, 243, 0.55);
  }

  .empty-title {
    font-weight: 600;
    margin-bottom: 0.2rem;
  }

  .empty-subtitle {
    font-size: 0.75rem;
    color: rgba(230, 237, 243, 0.45);
  }

  .log-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .log-item {
    display: grid;
    grid-template-columns: 90px 160px 130px 1fr;
    gap: 0.75rem;
    padding: 0.4rem 0.75rem;
    border-bottom: 1px solid rgba(78, 205, 196, 0.08);
  }

  .log-item:hover {
    background: rgba(78, 205, 196, 0.05);
  }

  .ts {
    color: rgba(230, 237, 243, 0.45);
  }

  .who {
    color: rgba(230, 237, 243, 0.85);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .type {
    color: rgba(78, 205, 196, 0.9);
  }

  .msg {
    color: rgba(230, 237, 243, 0.72);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
