<script lang="ts">
  let platform = 'unknown';

  if (typeof window !== 'undefined' && window.tachikoma) {
    platform = window.tachikoma.platform;
  }
</script>

<main>
  <h1>Tachikoma</h1>
  <p>Your squad of tireless AI coders</p>
  <p class="platform">Running on: {platform}</p>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    font-family: system-ui, -apple-system, sans-serif;
  }

  h1 {
    font-size: 3rem;
    margin-bottom: 0.5rem;
  }

  .platform {
    color: var(--text-muted, #666);
    font-size: 0.875rem;
  }
</style>