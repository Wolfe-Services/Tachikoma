<script lang="ts">
  export let specId: string | null = null;
</script>

<div class="spec-metadata">
  {#if specId}
    <div class="metadata-content">
      <h3>Metadata</h3>
      <p>Spec ID: {specId}</p>
      <!-- Add metadata details here -->
    </div>
  {:else}
    <div class="empty-state">
      <p>No specification selected</p>
    </div>
  {/if}
</div>

<style>
  .spec-metadata {
    height: 100%;
    padding: 1rem;
  }
  
  .metadata-content h3 {
    margin-top: 0;
    margin-bottom: 1rem;
    font-size: 1.1rem;
  }
  
  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-fg-muted);
  }
</style>