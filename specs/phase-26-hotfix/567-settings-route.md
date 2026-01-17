# Spec 567: Settings Route Page

## Header
- **Spec ID**: 567
- **Phase**: 26 - Hotfix Critical UI
- **Priority**: P1
- **Dependencies**: 562
- **Estimated Time**: 20 minutes

## Objective
Create the /settings route with configuration options.

## Acceptance Criteria
- [x] File `web/src/routes/settings/+page.svelte` exists
- [x] Settings page has tabs: General, Backend, Theme, Keyboard
- [x] API key input for Anthropic/OpenAI
- [x] Theme toggle (dark/light)
- [x] Settings persist via IPC

## Implementation

### settings/+page.svelte
```svelte
<script lang="ts">
  import { ipc } from '$lib/ipc';
  import { onMount } from 'svelte';
  
  let activeTab = 'general';
  let config = {
    anthropicKey: '',
    openaiKey: '',
    theme: 'dark',
    defaultBackend: 'claude'
  };
  
  const tabs = [
    { id: 'general', label: 'General' },
    { id: 'backend', label: 'AI Backends' },
    { id: 'theme', label: 'Appearance' },
    { id: 'keyboard', label: 'Keyboard' }
  ];
  
  onMount(async () => {
    try {
      const loaded = await ipc.invoke('config:get', {});
      config = { ...config, ...loaded };
    } catch (e) {
      console.log('Config not available:', e);
    }
  });
  
  async function saveConfig() {
    try {
      await ipc.invoke('config:set', config);
    } catch (e) {
      console.log('Could not save config:', e);
    }
  }
</script>

<div class="settings-page">
  <header class="page-header">
    <h1>Settings</h1>
  </header>
  
  <div class="settings-layout">
    <nav class="settings-tabs">
      {#each tabs as tab}
        <button 
          class="tab" 
          class:active={activeTab === tab.id}
          on:click={() => activeTab = tab.id}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
    
    <main class="settings-content">
      {#if activeTab === 'general'}
        <section class="settings-section">
          <h2>General Settings</h2>
          <label class="setting-row">
            <span>Default AI Backend</span>
            <select bind:value={config.defaultBackend}>
              <option value="claude">Claude (Anthropic)</option>
              <option value="gpt4">GPT-4 (OpenAI)</option>
              <option value="gemini">Gemini (Google)</option>
            </select>
          </label>
        </section>
        
      {:else if activeTab === 'backend'}
        <section class="settings-section">
          <h2>AI Backend Configuration</h2>
          
          <label class="setting-row">
            <span>Anthropic API Key</span>
            <input 
              type="password" 
              bind:value={config.anthropicKey}
              placeholder="sk-ant-..."
            />
          </label>
          
          <label class="setting-row">
            <span>OpenAI API Key</span>
            <input 
              type="password" 
              bind:value={config.openaiKey}
              placeholder="sk-..."
            />
          </label>
          
          <button class="btn-primary" on:click={saveConfig}>
            Save API Keys
          </button>
        </section>
        
      {:else if activeTab === 'theme'}
        <section class="settings-section">
          <h2>Appearance</h2>
          <label class="setting-row">
            <span>Theme</span>
            <select bind:value={config.theme}>
              <option value="dark">Dark</option>
              <option value="light">Light</option>
              <option value="system">System</option>
            </select>
          </label>
        </section>
        
      {:else if activeTab === 'keyboard'}
        <section class="settings-section">
          <h2>Keyboard Shortcuts</h2>
          <p class="muted">Keyboard shortcut customization coming soon.</p>
        </section>
      {/if}
    </main>
  </div>
</div>

<style>
  .settings-page {
    max-width: 900px;
    margin: 0 auto;
  }
  
  .page-header h1 {
    font-size: 1.75rem;
    margin: 0 0 1.5rem 0;
  }
  
  .settings-layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 1.5rem;
    background: var(--bg-secondary);
    border-radius: 12px;
    overflow: hidden;
  }
  
  .settings-tabs {
    background: var(--bg-tertiary);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .tab {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 0.75rem 1rem;
    text-align: left;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s;
  }
  
  .tab:hover {
    background: rgba(255,255,255,0.05);
    color: var(--text-primary);
  }
  
  .tab.active {
    background: var(--accent-primary);
    color: white;
  }
  
  .settings-content {
    padding: 1.5rem;
  }
  
  .settings-section h2 {
    font-size: 1.25rem;
    margin: 0 0 1.5rem 0;
  }
  
  .setting-row {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1.25rem;
  }
  
  .setting-row span {
    font-weight: 500;
  }
  
  .setting-row input,
  .setting-row select {
    padding: 0.625rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 0.9375rem;
  }
  
  .setting-row input:focus,
  .setting-row select:focus {
    outline: none;
    border-color: var(--accent-primary);
  }
  
  .btn-primary {
    background: var(--accent-primary);
    color: white;
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
  }
  
  .muted {
    color: var(--text-muted);
  }
</style>
```

## Verification
Navigate to /settings - should show tabbed settings UI
