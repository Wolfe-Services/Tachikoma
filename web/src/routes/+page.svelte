<script lang="ts">
  import { onMount } from 'svelte';
  import { missionStore, isRunning, progress } from '$lib/stores/mission';
  import { ipc } from '$lib/ipc';
  
  let platform = 'unknown';
  let testResults: string[] = [];

  if (typeof window !== 'undefined' && window.tachikoma) {
    platform = window.tachikoma.platform;
  }

  async function testIpc() {
    testResults = ['Testing IPC Bridge...'];
    
    try {
      // Test config
      const config = await ipc.invoke('config:get', {});
      testResults = [...testResults, `✓ Config loaded: ${config.backend.brain}`];
      
      // Test spec list
      const specs = await ipc.invoke('spec:list', {});
      testResults = [...testResults, `✓ Specs listed: ${specs.length} found`];
      
      // Test mission start
      const mission = await missionStore.start('/test/path', 'claude', 'attended');
      testResults = [...testResults, `✓ Mission started: ${mission}`];
      
      testResults = [...testResults, '✅ All IPC tests passed!'];
    } catch (error) {
      testResults = [...testResults, `❌ Test failed: ${error}`];
    }
  }
</script>

<main>
  <h1>Tachikoma</h1>
  <p>Your squad of tireless AI coders</p>
  <p class="platform">Running on: {platform}</p>
  
  <div class="test-section">
    <button 
      on:click={testIpc}
      class="test-button"
    >
      Test IPC Bridge
    </button>
    
    {#if testResults.length > 0}
      <div class="test-results">
        <h2>Test Results:</h2>
        {#each testResults as result}
          <div class="test-result">{result}</div>
        {/each}
      </div>
    {/if}
    
    <div class="mission-status">
      <h2>Mission Status</h2>
      <p>Running: {$isRunning}</p>
      <p>Progress: {$progress}%</p>
    </div>
  </div>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    font-family: system-ui, -apple-system, sans-serif;
    padding: 2rem;
  }

  h1 {
    font-size: 3rem;
    margin-bottom: 0.5rem;
  }

  .platform {
    color: var(--text-muted, #666);
    font-size: 0.875rem;
    margin-bottom: 2rem;
  }

  .test-section {
    width: 100%;
    max-width: 600px;
  }

  .test-button {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    cursor: pointer;
    font-size: 1rem;
    margin-bottom: 1rem;
  }

  .test-button:hover {
    background: #2563eb;
  }

  .test-results {
    background: #f3f4f6;
    padding: 1rem;
    border-radius: 0.5rem;
    margin-bottom: 1rem;
  }

  .test-results h2 {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }

  .test-result {
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.875rem;
    margin-bottom: 0.25rem;
  }

  .mission-status {
    background: #f9fafb;
    padding: 1rem;
    border-radius: 0.5rem;
  }

  .mission-status h2 {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }
</style>