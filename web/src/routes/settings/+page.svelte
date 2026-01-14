<script lang="ts">
  import PageHeader from '$lib/components/common/PageHeader.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import { ipc } from '$lib/ipc';
  import { onMount } from 'svelte';
  
  let activeTab = 'general';
  let saveStatus: 'idle' | 'saving' | 'saved' | 'error' = 'idle';
  
  let config = {
    anthropicKey: '',
    openaiKey: '',
    googleKey: '',
    theme: 'tachikoma-dark',
    defaultBackend: 'claude',
    autoSave: true,
    notifications: true,
    soundEffects: false,
    telemetry: false
  };
  
  const tabs = [
    { id: 'general', label: 'GENERAL', icon: 'settings', description: 'Core settings' },
    { id: 'backend', label: 'AI BACKENDS', icon: 'brain', description: 'Model configuration' },
    { id: 'theme', label: 'APPEARANCE', icon: 'eye', description: 'Visual customization' },
    { id: 'keyboard', label: 'CONTROLS', icon: 'terminal', description: 'Shortcuts & inputs' },
    { id: 'about', label: 'ABOUT', icon: 'info', description: 'System information' }
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
    saveStatus = 'saving';
    try {
      await ipc.invoke('config:set', config);
      saveStatus = 'saved';
      setTimeout(() => saveStatus = 'idle', 2000);
    } catch (e) {
      console.log('Could not save config:', e);
      saveStatus = 'error';
      setTimeout(() => saveStatus = 'idle', 3000);
    }
  }
  
  function maskApiKey(key: string): string {
    if (!key || key.length < 8) return key;
    return key.substring(0, 4) + '••••••••' + key.substring(key.length - 4);
  }
</script>

<div class="settings-page">
  <PageHeader 
    title="CONFIGURATION"
    subtitle="System preferences and squad parameters"
    tag="SECTION 9 // SETTINGS"
    icon="settings"
  >
    <svelte:fragment slot="actions">
      <button 
        class="save-btn" 
        class:saving={saveStatus === 'saving'}
        class:saved={saveStatus === 'saved'}
        on:click={saveConfig}
        disabled={saveStatus === 'saving'}
      >
        {#if saveStatus === 'saving'}
          <div class="save-spinner"></div>
          <span>SAVING...</span>
        {:else if saveStatus === 'saved'}
          <Icon name="check" size={16} />
          <span>SAVED</span>
        {:else}
          <Icon name="save" size={16} />
          <span>SAVE CHANGES</span>
        {/if}
      </button>
    </svelte:fragment>
  </PageHeader>
  
  <div class="settings-layout">
    <!-- Sidebar Navigation -->
    <nav class="settings-nav">
      {#each tabs as tab}
        <button 
          class="nav-tab"
          class:active={activeTab === tab.id}
          on:click={() => activeTab = tab.id}
        >
          <div class="tab-icon">
            <Icon name={tab.icon} size={18} glow={activeTab === tab.id} />
          </div>
          <div class="tab-content">
            <span class="tab-label">{tab.label}</span>
            <span class="tab-desc">{tab.description}</span>
          </div>
          {#if activeTab === tab.id}
            <div class="tab-indicator"></div>
          {/if}
        </button>
      {/each}
      
      <div class="nav-footer">
        <div class="system-status">
          <span class="status-dot online"></span>
          <span class="status-text">System Online</span>
        </div>
      </div>
    </nav>
    
    <!-- Settings Content -->
    <main class="settings-content">
      {#if activeTab === 'general'}
        <section class="settings-section">
          <div class="section-header">
            <Icon name="settings" size={20} />
            <h2>General Configuration</h2>
          </div>
          
          <div class="settings-grid">
            <div class="setting-card">
              <div class="setting-header">
                <span class="setting-label">DEFAULT AI BACKEND</span>
                <span class="setting-tag">REQUIRED</span>
              </div>
              <p class="setting-desc">
                Primary AI model used for spec implementation tasks.
              </p>
              <div class="select-wrapper">
            <select bind:value={config.defaultBackend}>
              <option value="claude">Claude (Anthropic)</option>
              <option value="gpt4">GPT-4 (OpenAI)</option>
              <option value="gemini">Gemini (Google)</option>
                  <option value="ollama">Ollama (Local)</option>
            </select>
                <Icon name="chevron-down" size={14} />
              </div>
            </div>
            
            <div class="setting-card toggle-card">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">AUTO-SAVE</span>
                  <p class="setting-desc">Automatically save progress during missions</p>
                </div>
                <label class="toggle">
                  <input type="checkbox" bind:checked={config.autoSave} />
                  <span class="toggle-slider"></span>
                </label>
              </div>
            </div>
            
            <div class="setting-card toggle-card">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">NOTIFICATIONS</span>
                  <p class="setting-desc">Show system notifications for mission events</p>
                </div>
                <label class="toggle">
                  <input type="checkbox" bind:checked={config.notifications} />
                  <span class="toggle-slider"></span>
                </label>
              </div>
            </div>
            
            <div class="setting-card toggle-card">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">SOUND EFFECTS</span>
                  <p class="setting-desc">Audio feedback for actions and alerts</p>
                </div>
                <label class="toggle">
                  <input type="checkbox" bind:checked={config.soundEffects} />
                  <span class="toggle-slider"></span>
          </label>
              </div>
            </div>
          </div>
        </section>
        
      {:else if activeTab === 'backend'}
        <section class="settings-section">
          <div class="section-header">
            <Icon name="brain" size={20} />
          <h2>AI Backend Configuration</h2>
          </div>
          
          <div class="api-keys-grid">
            <div class="api-key-card">
              <div class="api-header">
                <div class="api-logo anthropic">A</div>
                <div class="api-info">
                  <span class="api-name">ANTHROPIC</span>
                  <span class="api-model">Claude Models</span>
                </div>
                <div class="api-status" class:connected={config.anthropicKey}>
                  {config.anthropicKey ? 'CONNECTED' : 'NOT SET'}
                </div>
              </div>
              <div class="api-input-group">
                <label class="input-label">API KEY</label>
                <div class="input-wrapper">
            <input 
              type="password" 
              bind:value={config.anthropicKey}
                    placeholder="sk-ant-api03-..."
                    class="api-input"
                  />
                  <button class="input-action" title="Show/Hide">
                    <Icon name="eye" size={16} />
                  </button>
                </div>
              </div>
            </div>
            
            <div class="api-key-card">
              <div class="api-header">
                <div class="api-logo openai">◯</div>
                <div class="api-info">
                  <span class="api-name">OPENAI</span>
                  <span class="api-model">GPT-4 Models</span>
                </div>
                <div class="api-status" class:connected={config.openaiKey}>
                  {config.openaiKey ? 'CONNECTED' : 'NOT SET'}
                </div>
              </div>
              <div class="api-input-group">
                <label class="input-label">API KEY</label>
                <div class="input-wrapper">
            <input 
              type="password" 
              bind:value={config.openaiKey}
              placeholder="sk-..."
                    class="api-input"
                  />
                  <button class="input-action" title="Show/Hide">
                    <Icon name="eye" size={16} />
                  </button>
                </div>
              </div>
            </div>
            
            <div class="api-key-card">
              <div class="api-header">
                <div class="api-logo google">G</div>
                <div class="api-info">
                  <span class="api-name">GOOGLE AI</span>
                  <span class="api-model">Gemini Models</span>
                </div>
                <div class="api-status" class:connected={config.googleKey}>
                  {config.googleKey ? 'CONNECTED' : 'NOT SET'}
                </div>
              </div>
              <div class="api-input-group">
                <label class="input-label">API KEY</label>
                <div class="input-wrapper">
                  <input 
                    type="password" 
                    bind:value={config.googleKey}
                    placeholder="AIza..."
                    class="api-input"
                  />
                  <button class="input-action" title="Show/Hide">
                    <Icon name="eye" size={16} />
                  </button>
                </div>
              </div>
            </div>
          </div>
          
          <div class="security-note">
            <Icon name="shield" size={16} />
            <span>API keys are stored securely in your system keychain and never transmitted.</span>
          </div>
        </section>
        
      {:else if activeTab === 'theme'}
        <section class="settings-section">
          <div class="section-header">
            <Icon name="eye" size={20} />
            <h2>Appearance Settings</h2>
          </div>
          
          <div class="theme-grid">
            <button 
              class="theme-card"
              class:active={config.theme === 'tachikoma-dark'}
              on:click={() => config.theme = 'tachikoma-dark'}
            >
              <div class="theme-preview dark">
                <div class="preview-header"></div>
                <div class="preview-sidebar"></div>
                <div class="preview-content">
                  <div class="preview-line"></div>
                  <div class="preview-line short"></div>
                </div>
              </div>
              <div class="theme-info">
                <span class="theme-name">TACHIKOMA DARK</span>
                <span class="theme-desc">Cyberpunk noir theme</span>
              </div>
              {#if config.theme === 'tachikoma-dark'}
                <div class="theme-check">
                  <Icon name="check" size={14} />
                </div>
              {/if}
            </button>
            
            <button 
              class="theme-card"
              class:active={config.theme === 'ghost-shell'}
              on:click={() => config.theme = 'ghost-shell'}
            >
              <div class="theme-preview ghost">
                <div class="preview-header"></div>
                <div class="preview-sidebar"></div>
                <div class="preview-content">
                  <div class="preview-line"></div>
                  <div class="preview-line short"></div>
                </div>
              </div>
              <div class="theme-info">
                <span class="theme-name">GHOST IN SHELL</span>
                <span class="theme-desc">Matrix green accent</span>
              </div>
              {#if config.theme === 'ghost-shell'}
                <div class="theme-check">
                  <Icon name="check" size={14} />
                </div>
              {/if}
            </button>
            
            <button 
              class="theme-card"
              class:active={config.theme === 'section-9'}
              on:click={() => config.theme = 'section-9'}
            >
              <div class="theme-preview section9">
                <div class="preview-header"></div>
                <div class="preview-sidebar"></div>
                <div class="preview-content">
                  <div class="preview-line"></div>
                  <div class="preview-line short"></div>
                </div>
              </div>
              <div class="theme-info">
                <span class="theme-name">SECTION 9</span>
                <span class="theme-desc">Military tactical</span>
              </div>
              {#if config.theme === 'section-9'}
                <div class="theme-check">
                  <Icon name="check" size={14} />
                </div>
              {/if}
            </button>
          </div>
        </section>
        
      {:else if activeTab === 'keyboard'}
        <section class="settings-section">
          <div class="section-header">
            <Icon name="terminal" size={20} />
            <h2>Keyboard Controls</h2>
          </div>
          
          <div class="shortcuts-list">
            <div class="shortcut-row">
              <span class="shortcut-action">Open Command Palette</span>
              <div class="shortcut-keys">
                <kbd>⌘</kbd><kbd>K</kbd>
              </div>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-action">Deploy New Unit</span>
              <div class="shortcut-keys">
                <kbd>⌘</kbd><kbd>D</kbd>
              </div>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-action">Open Spec Registry</span>
              <div class="shortcut-keys">
                <kbd>⌘</kbd><kbd>R</kbd>
              </div>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-action">Toggle Sidebar</span>
              <div class="shortcut-keys">
                <kbd>⌘</kbd><kbd>B</kbd>
              </div>
            </div>
            <div class="shortcut-row">
              <span class="shortcut-action">Open Settings</span>
              <div class="shortcut-keys">
                <kbd>⌘</kbd><kbd>,</kbd>
              </div>
            </div>
          </div>
          
          <p class="coming-soon">
            Custom keyboard shortcut configuration coming in a future update.
          </p>
        </section>
        
      {:else if activeTab === 'about'}
        <section class="settings-section">
          <div class="section-header">
            <Icon name="info" size={20} />
            <h2>About Tachikoma</h2>
          </div>
          
          <div class="about-card">
            <div class="about-logo">
              <div class="logo-ring">
                <div class="logo-inner">T</div>
              </div>
            </div>
            <div class="about-info">
              <h3 class="about-title">TACHIKOMA</h3>
              <p class="about-subtitle">Section 9 AI Coding Squad</p>
              <div class="version-info">
                <span class="version-label">VERSION</span>
                <span class="version-value">1.0.0</span>
              </div>
              <div class="version-info">
                <span class="version-label">BUILD</span>
                <span class="version-value">2024.1.0-alpha</span>
              </div>
            </div>
          </div>
          
          <div class="about-lore">
            <p>
              Tachikoma are artificial intelligence walking tanks deployed by 
              Public Security Section 9. They are renowned for their curiosity, 
              helpfulness, and unique personalities that emerge from their 
              shared experience synchronization.
            </p>
            <p>
              This application deploys 9 virtual Tachikoma units as your tireless 
              AI coding assistants, implementing specifications autonomously 
              while you focus on higher-level decisions.
            </p>
          </div>
          
          <div class="about-links">
            <a href="https://github.com/your-org/tachikoma" class="about-link">
              <Icon name="code" size={16} />
              <span>Source Code</span>
            </a>
            <a href="#" class="about-link">
              <Icon name="file-text" size={16} />
              <span>Documentation</span>
            </a>
            <a href="#" class="about-link">
              <Icon name="help-circle" size={16} />
              <span>Support</span>
            </a>
          </div>
          
          <div class="about-footer">
            <span class="copyright">公安9課 // PUBLIC SECURITY SECTION 9</span>
          </div>
        </section>
      {/if}
    </main>
  </div>
</div>

<style>
  .settings-page {
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .save-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    border: 1px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 8px;
    color: var(--bg-primary, #0d1117);
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    cursor: pointer;
    transition: all 0.3s ease;
  }
  
  .save-btn:hover:not(:disabled) {
    box-shadow: 0 0 20px rgba(78, 205, 196, 0.4);
    transform: translateY(-1px);
  }
  
  .save-btn.saved {
    background: linear-gradient(135deg, #2d7a4a, var(--success-color, #3fb950));
    border-color: var(--success-color, #3fb950);
  }
  
  .save-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(13, 17, 23, 0.3);
    border-top-color: var(--bg-primary, #0d1117);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  /* Layout */
  .settings-layout {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 1.5rem;
    min-height: 600px;
  }
  
  /* Navigation */
  .settings-nav {
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .nav-tab {
    display: flex;
    align-items: center;
    gap: 0.875rem;
    padding: 1rem;
    background: transparent;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s ease;
    position: relative;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
  }
  
  .nav-tab:hover {
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    color: var(--text-primary, #e6edf3);
  }
  
  .nav-tab.active {
    background: rgba(78, 205, 196, 0.15);
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .tab-icon {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary, #1c2128);
    border-radius: 8px;
    flex-shrink: 0;
  }
  
  .nav-tab.active .tab-icon {
    background: rgba(78, 205, 196, 0.2);
  }
  
  .tab-content {
    display: flex;
    flex-direction: column;
    gap: 0;
    overflow: hidden;
  }
  
  .tab-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.5px;
  }
  
  .tab-desc {
    font-size: 0.75rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .tab-indicator {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 60%;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 0 2px 2px 0;
    box-shadow: 0 0 8px var(--tachi-cyan, #4ecdc4);
  }
  
  .nav-footer {
    margin-top: auto;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
  }
  
  .system-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
  }
  
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted, rgba(230, 237, 243, 0.4));
  }
  
  .status-dot.online {
    background: var(--success-color, #3fb950);
    box-shadow: 0 0 8px rgba(63, 185, 80, 0.5);
    animation: pulse 2s ease-in-out infinite;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .status-text {
    font-size: 0.75rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }
  
  /* Content */
  .settings-content {
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    padding: 1.5rem;
    overflow-y: auto;
  }
  
  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .section-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .section-header h2 {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
    margin: 0;
  }
  
  /* Settings Grid */
  .settings-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .setting-card {
    padding: 1.25rem;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    border-radius: 10px;
  }
  
  .setting-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  
  .setting-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
  }
  
  .setting-tag {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.55rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    background: rgba(78, 205, 196, 0.15);
    border-radius: 3px;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 0.5px;
  }
  
  .setting-desc {
    font-size: 0.85rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    margin: 0 0 1rem;
  }
  
  .select-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }
  
  .select-wrapper select {
    width: 100%;
    padding: 0.75rem 2.5rem 0.75rem 1rem !important;
    appearance: none;
    cursor: pointer;
  }
  
  .select-wrapper :global(svg) {
    position: absolute;
    right: 1rem;
    color: var(--tachi-cyan, #4ecdc4);
    pointer-events: none;
  }
  
  .toggle-card .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  
  .toggle-card .setting-info {
    flex: 1;
  }
  
  .toggle-card .setting-desc {
    margin: 0.25rem 0 0;
  }
  
  /* Toggle Switch */
  .toggle {
    position: relative;
    display: inline-block;
    width: 48px;
    height: 26px;
    flex-shrink: 0;
  }
  
  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  
  .toggle-slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.3));
    border-radius: 26px;
    transition: all 0.3s ease;
  }
  
  .toggle-slider::before {
    position: absolute;
    content: '';
    height: 20px;
    width: 20px;
    left: 2px;
    bottom: 2px;
    background: var(--text-muted, rgba(230, 237, 243, 0.5));
    border-radius: 50%;
    transition: all 0.3s ease;
  }
  
  .toggle input:checked + .toggle-slider {
    background: rgba(78, 205, 196, 0.2);
    border-color: var(--tachi-cyan, #4ecdc4);
  }
  
  .toggle input:checked + .toggle-slider::before {
    background: var(--tachi-cyan, #4ecdc4);
    transform: translateX(22px);
    box-shadow: 0 0 10px var(--tachi-cyan, #4ecdc4);
  }
  
  /* API Keys */
  .api-keys-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .api-key-card {
    padding: 1.25rem;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    border-radius: 10px;
  }
  
  .api-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }
  
  .api-logo {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1rem;
    font-weight: 700;
    border-radius: 8px;
    flex-shrink: 0;
  }
  
  .api-logo.anthropic {
    background: linear-gradient(135deg, rgba(204, 120, 92, 0.2), rgba(204, 120, 92, 0.05));
    border: 1px solid rgba(204, 120, 92, 0.4);
    color: #cc785c;
  }
  
  .api-logo.openai {
    background: linear-gradient(135deg, rgba(116, 170, 156, 0.2), rgba(116, 170, 156, 0.05));
    border: 1px solid rgba(116, 170, 156, 0.4);
    color: #74aa9c;
  }
  
  .api-logo.google {
    background: linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(139, 92, 246, 0.05));
    border: 1px solid rgba(139, 92, 246, 0.4);
    color: #8b5cf6;
  }
  
  .api-info {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  
  .api-name {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
  }
  
  .api-model {
    font-size: 0.75rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }
  
  .api-status {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    font-weight: 600;
    padding: 0.375rem 0.75rem;
    background: var(--bg-secondary, #161b22);
    border-radius: 4px;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    letter-spacing: 0.5px;
  }
  
  .api-status.connected {
    background: rgba(63, 185, 80, 0.15);
    color: var(--success-color, #3fb950);
  }
  
  .api-input-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .input-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 1px;
  }
  
  .input-wrapper {
    display: flex;
    align-items: stretch;
    gap: 0.5rem;
  }
  
  .api-input {
    flex: 1;
    padding: 0.75rem 1rem !important;
    font-family: monospace;
    font-size: 0.85rem;
  }
  
  .input-action {
    padding: 0 0.75rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 6px;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .input-action:hover {
    color: var(--tachi-cyan, #4ecdc4);
    border-color: var(--tachi-cyan, #4ecdc4);
  }
  
  .security-note {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(63, 185, 80, 0.08);
    border: 1px solid rgba(63, 185, 80, 0.2);
    border-radius: 8px;
    font-size: 0.85rem;
    color: var(--success-color, #3fb950);
  }
  
  /* Theme Grid */
  .theme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
  }
  
  .theme-card {
    padding: 1rem;
    background: var(--bg-tertiary, #1c2128);
    border: 2px solid var(--border-color, rgba(78, 205, 196, 0.1));
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.3s ease;
    text-align: left;
    position: relative;
  }
  
  .theme-card:hover {
    border-color: rgba(78, 205, 196, 0.3);
  }
  
  .theme-card.active {
    border-color: var(--tachi-cyan, #4ecdc4);
    box-shadow: 0 0 15px rgba(78, 205, 196, 0.2);
  }
  
  .theme-preview {
    height: 80px;
    background: #0d1117;
    border-radius: 6px;
    display: grid;
    grid-template-columns: 30px 1fr;
    grid-template-rows: 15px 1fr;
    gap: 4px;
    padding: 4px;
    margin-bottom: 0.75rem;
    overflow: hidden;
  }
  
  .preview-header {
    grid-column: 1 / -1;
    background: #161b22;
    border-radius: 3px;
  }
  
  .preview-sidebar {
    background: #161b22;
    border-radius: 3px;
  }
  
  .preview-content {
    background: #0a0c10;
    border-radius: 3px;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  
  .preview-line {
    height: 6px;
    background: #4ecdc4;
    border-radius: 2px;
    opacity: 0.3;
  }
  
  .preview-line.short {
    width: 60%;
  }
  
  .theme-preview.ghost .preview-line {
    background: #00ff41;
  }
  
  .theme-preview.section9 .preview-line {
    background: #ff6b6b;
  }
  
  .theme-info {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }
  
  .theme-name {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 0.5px;
  }
  
  .theme-desc {
    font-size: 0.75rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }
  
  .theme-check {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    color: var(--bg-primary, #0d1117);
  }
  
  /* Shortcuts */
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    background: var(--bg-tertiary, #1c2128);
    border-radius: 8px;
  }
  
  .shortcut-action {
    font-size: 0.9rem;
    color: var(--text-primary, #e6edf3);
  }
  
  .shortcut-keys {
    display: flex;
    gap: 0.25rem;
  }
  
  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 28px;
    padding: 0 0.5rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.3));
    border-radius: 4px;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .coming-soon {
    font-size: 0.85rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    font-style: italic;
  }
  
  /* About */
  .about-card {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1.5rem;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.1), transparent);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 12px;
  }
  
  .about-logo {
    flex-shrink: 0;
  }
  
  .logo-ring {
    width: 80px;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    animation: logoRotate 20s linear infinite;
  }
  
  @keyframes logoRotate {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .logo-inner {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 2rem;
    font-weight: 700;
    color: var(--tachi-cyan, #4ecdc4);
    text-shadow: 0 0 20px var(--tachi-cyan, #4ecdc4);
    animation: logoRotate 20s linear infinite reverse;
  }
  
  .about-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .about-title {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 2px;
    margin: 0;
  }
  
  .about-subtitle {
    font-size: 0.9rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    margin: 0;
  }
  
  .version-info {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    margin-top: 0.5rem;
  }
  
  .version-label {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    letter-spacing: 1px;
  }
  
  .version-value {
    font-family: monospace;
    font-size: 0.8rem;
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .about-lore {
    padding: 1.25rem;
    background: var(--bg-tertiary, #1c2128);
    border-left: 3px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 0 8px 8px 0;
  }
  
  .about-lore p {
    font-size: 0.9rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    line-height: 1.7;
    margin: 0;
  }
  
  .about-lore p + p {
    margin-top: 1rem;
  }
  
  .about-links {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }
  
  .about-link {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 8px;
    color: var(--text-primary, #e6edf3);
    text-decoration: none;
    transition: all 0.2s ease;
  }
  
  .about-link:hover {
    background: rgba(78, 205, 196, 0.1);
    border-color: var(--tachi-cyan, #4ecdc4);
    color: var(--tachi-cyan, #4ecdc4);
  }
  
  .about-footer {
    text-align: center;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
  }
  
  .copyright {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.4));
    letter-spacing: 2px;
  }
  
  /* Responsive */
  @media (max-width: 800px) {
    .settings-layout {
      grid-template-columns: 1fr;
    }
    
    .settings-nav {
      flex-direction: row;
      overflow-x: auto;
      padding: 0.5rem;
    }
    
    .nav-tab {
      flex-direction: column;
      padding: 0.75rem;
      min-width: 80px;
      text-align: center;
    }
    
    .tab-content {
      display: none;
    }
    
    .nav-footer {
      display: none;
    }
    
    .tab-indicator {
      top: auto;
      bottom: 0;
      left: 50%;
      transform: translateX(-50%);
      width: 60%;
      height: 3px;
      border-radius: 2px 2px 0 0;
    }
  }
</style>
