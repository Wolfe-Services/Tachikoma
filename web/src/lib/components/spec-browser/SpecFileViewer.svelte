<script lang="ts">
  export let specId: string | null = null;
  export let viewMode: 'view' | 'edit' | 'split' = 'view';
</script>

<div class="spec-file-viewer">
  {#if specId}
    <div class="content">
      <p>Spec content for ID: {specId}</p>
      <p>View mode: {viewMode}</p>
    </div>
  {:else}
    <div class="empty-state">
      <p>No specification selected</p>
    </div>
  {/if}
</div>

<style>
  .spec-file-viewer {
    height: 100%;
    padding: 1rem;
  }
  
  .content {
    height: 100%;
  }
  
  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-fg-muted);
  }
</style>