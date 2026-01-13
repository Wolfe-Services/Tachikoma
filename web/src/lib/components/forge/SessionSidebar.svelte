<script lang="ts">
  export let sessionId: string | null = null;
</script>

<div class="session-sidebar" data-testid="session-sidebar">
  <h2>Sessions</h2>
  {#if sessionId}
    <div class="active-session">
      <h3>Active Session</h3>
      <p>Session ID: {sessionId}</p>
    </div>
  {:else}
    <div class="no-session">
      <p>No active session</p>
      <button type="button">Create New Session</button>
    </div>
  {/if}
</div>

<style>
  .session-sidebar {
    padding: 1rem;
    height: 100%;
    overflow-y: auto;
  }

  .session-sidebar h2 {
    margin: 0 0 1rem 0;
    font-size: 1.2rem;
    color: var(--forge-text, #eaeaea);
  }

  .active-session {
    background: var(--accent-color, #4a9eff);
    padding: 0.75rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .no-session {
    text-align: center;
    color: var(--text-muted, #999);
  }

  .no-session button {
    background: var(--accent-color, #4a9eff);
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 0.5rem;
  }

  .no-session button:hover {
    background: var(--accent-hover, #357abd);
  }
</style>