# Spec 566: Forge Route Page

## Header
- **Spec ID**: 566
- **Phase**: 26 - Hotfix Critical UI
- **Priority**: P0 - CRITICAL
- **Dependencies**: 562
- **Estimated Time**: 20 minutes

## Objective
Create the /forge route that shows the multi-model brainstorming UI.

## Acceptance Criteria
- [x] File `web/src/routes/forge/+page.svelte` exists
- [x] Page uses ForgeLayout component
- [x] Shows session creation or active session
- [x] Integrates existing forge components

## Implementation

### Create forge folder and page
```bash
mkdir -p web/src/routes/forge
```

### forge/+page.svelte
```svelte
<script lang="ts">
  import ForgeLayout from '$lib/components/forge/ForgeLayout.svelte';
  import SessionSidebar from '$lib/components/forge/SessionSidebar.svelte';
  
  let activeSession: any = null;
  let sessions: any[] = [];
</script>

<div class="forge-page">
  <header class="page-header">
    <h1>Spec Forge</h1>
    <p class="subtitle">Multi-model brainstorming for spec creation</p>
  </header>
  
  {#if activeSession}
    <ForgeLayout session={activeSession} />
  {:else}
    <div class="forge-start">
      <div class="start-card">
        <div class="start-icon">🔥</div>
        <h2>Start a Forge Session</h2>
        <p>Gather multiple AI models to brainstorm and create specs together.</p>
        
        <form class="start-form">
          <label>
            <span>What do you want to build?</span>
            <textarea 
              placeholder="Describe the feature or component you want to create..."
              rows="4"
            ></textarea>
          </label>
          
          <button type="submit" class="btn-primary">
            🔥 Start Forge Session
          </button>
        </form>
      </div>
      
      {#if sessions.length > 0}
        <aside class="past-sessions">
          <h3>Past Sessions</h3>
          <SessionSidebar {sessions} />
        </aside>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-page {
    max-width: 1000px;
    margin: 0 auto;
  }
  
  .page-header {
    margin-bottom: 2rem;
  }
  
  .page-header h1 {
    font-size: 1.75rem;
    margin: 0 0 0.25rem 0;
  }
  
  .subtitle {
    color: var(--text-secondary);
    margin: 0;
  }
  
  .forge-start {
    display: grid;
    gap: 1.5rem;
  }
  
  .start-card {
    background: var(--bg-secondary);
    border-radius: 12px;
    padding: 2rem;
    text-align: center;
  }
  
  .start-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }
  
  .start-card h2 {
    margin: 0 0 0.5rem 0;
  }
  
  .start-card > p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
  }
  
  .start-form {
    max-width: 500px;
    margin: 0 auto;
    text-align: left;
  }
  
  .start-form label {
    display: block;
    margin-bottom: 1rem;
  }
  
  .start-form label span {
    display: block;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }
  
  .start-form textarea {
    width: 100%;
    padding: 0.75rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    color: var(--text-primary);
    font-family: inherit;
    resize: vertical;
  }
  
  .start-form textarea:focus {
    outline: none;
    border-color: var(--accent-primary);
  }
  
  .btn-primary {
    width: 100%;
    background: linear-gradient(135deg, #f59e0b, #ef4444);
    color: white;
    padding: 0.875rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
  }
  
  .btn-primary:hover {
    opacity: 0.9;
  }
  
  .past-sessions h3 {
    font-size: 1rem;
    margin-bottom: 1rem;
  }
</style>
```

## Verification
Navigate to /forge - should show forge session start UI
