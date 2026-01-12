# Spec 291: Workspace Settings

## Header
- **Spec ID**: 291
- **Phase**: 13 - Settings UI
- **Component**: Workspace Settings
- **Dependencies**: Spec 290 (Profile Management)
- **Status**: Draft

## Objective
Create a workspace settings interface that allows users to configure workspace-specific preferences, project organization, file handling, and collaboration settings for team environments.

## Acceptance Criteria
1. Configure workspace directories and paths
2. Set up project organization preferences
3. Configure file watching and sync behavior
4. Set collaboration and sharing preferences
5. Configure workspace-specific templates
6. Set up environment variables
7. Configure workspace backup settings
8. Manage workspace access permissions

## Implementation

### WorkspaceSettings.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import PathPicker from './PathPicker.svelte';
  import EnvironmentEditor from './EnvironmentEditor.svelte';
  import CollaboratorList from './CollaboratorList.svelte';
  import { workspaceStore } from '$lib/stores/workspace';
  import type {
    WorkspaceConfig,
    WorkspacePath,
    EnvironmentVariable,
    Collaborator
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: WorkspaceConfig;
    reset: void;
  }>();

  let showPathPicker = writable<boolean>(false);
  let showEnvEditor = writable<boolean>(false);
  let editingPath = writable<WorkspacePath | null>(null);

  const config = derived(workspaceStore, ($store) => $store.config);
  const paths = derived(config, ($config) => $config.paths);
  const envVars = derived(config, ($config) => $config.environment);
  const collaborators = derived(workspaceStore, ($store) => $store.collaborators);

  const defaultPaths: { id: string; name: string; description: string; required: boolean }[] = [
    { id: 'root', name: 'Workspace Root', description: 'Main workspace directory', required: true },
    { id: 'sessions', name: 'Sessions Directory', description: 'Where session data is stored', required: true },
    { id: 'templates', name: 'Templates Directory', description: 'Custom template storage', required: false },
    { id: 'output', name: 'Output Directory', description: 'Default output location', required: false },
    { id: 'cache', name: 'Cache Directory', description: 'Local cache storage', required: false }
  ];

  function updateConfig(field: keyof WorkspaceConfig, value: unknown) {
    workspaceStore.updateConfig(field, value);
  }

  function updatePath(pathId: string, value: string) {
    workspaceStore.updatePath(pathId, value);
  }

  function addEnvironmentVariable(variable: EnvironmentVariable) {
    workspaceStore.addEnvVar(variable);
  }

  function updateEnvironmentVariable(name: string, value: string) {
    workspaceStore.updateEnvVar(name, value);
  }

  function removeEnvironmentVariable(name: string) {
    if (confirm(`Remove environment variable "${name}"?`)) {
      workspaceStore.removeEnvVar(name);
    }
  }

  async function addCollaborator(email: string, role: string) {
    await workspaceStore.addCollaborator(email, role);
  }

  async function updateCollaboratorRole(userId: string, role: string) {
    await workspaceStore.updateCollaboratorRole(userId, role);
  }

  async function removeCollaborator(userId: string) {
    if (confirm('Remove this collaborator from the workspace?')) {
      await workspaceStore.removeCollaborator(userId);
    }
  }

  async function saveSettings() {
    await workspaceStore.save();
    dispatch('save', $config);
  }

  function resetToDefaults() {
    if (confirm('Reset workspace settings to defaults?')) {
      workspaceStore.resetToDefaults();
      dispatch('reset');
    }
  }

  async function browseDirectory(pathId: string) {
    editingPath.set({ id: pathId, path: $paths[pathId] || '' });
    showPathPicker.set(true);
  }

  function handlePathSelect(path: string) {
    if ($editingPath) {
      updatePath($editingPath.id, path);
    }
    showPathPicker.set(false);
    editingPath.set(null);
  }

  onMount(() => {
    workspaceStore.load();
  });
</script>

<div class="workspace-settings" data-testid="workspace-settings">
  <header class="config-header">
    <div class="header-title">
      <h2>Workspace Settings</h2>
      <p class="description">Configure workspace preferences and organization</p>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={resetToDefaults}>
        Reset to Defaults
      </button>
      <button class="btn primary" on:click={saveSettings}>
        Save Settings
      </button>
    </div>
  </header>

  <section class="workspace-info">
    <div class="info-grid">
      <div class="form-group">
        <label for="workspace-name">Workspace Name</label>
        <input
          id="workspace-name"
          type="text"
          value={$config.name}
          on:input={(e) => updateConfig('name', (e.target as HTMLInputElement).value)}
        />
      </div>

      <div class="form-group">
        <label for="workspace-description">Description</label>
        <input
          id="workspace-description"
          type="text"
          value={$config.description || ''}
          on:input={(e) => updateConfig('description', (e.target as HTMLInputElement).value)}
          placeholder="Optional description"
        />
      </div>
    </div>
  </section>

  <section class="paths-config">
    <h3>Directory Paths</h3>

    <div class="paths-list">
      {#each defaultPaths as pathDef (pathDef.id)}
        <div class="path-item">
          <div class="path-info">
            <span class="path-name">
              {pathDef.name}
              {#if pathDef.required}
                <span class="required">*</span>
              {/if}
            </span>
            <span class="path-desc">{pathDef.description}</span>
          </div>

          <div class="path-input">
            <input
              type="text"
              value={$paths[pathDef.id] || ''}
              on:input={(e) => updatePath(pathDef.id, (e.target as HTMLInputElement).value)}
              placeholder="Enter path..."
            />
            <button
              class="btn secondary small"
              on:click={() => browseDirectory(pathDef.id)}
            >
              Browse
            </button>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section class="file-handling">
    <h3>File Handling</h3>

    <div class="toggle-options">
      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.watchForChanges}
          on:change={(e) => updateConfig('watchForChanges', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Watch for file changes</span>
          <span class="toggle-desc">Automatically detect and sync file changes</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.autoSave}
          on:change={(e) => updateConfig('autoSave', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Auto-save sessions</span>
          <span class="toggle-desc">Automatically save session progress</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.createBackups}
          on:change={(e) => updateConfig('createBackups', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Create backups before overwriting</span>
          <span class="toggle-desc">Keep backup copies of modified files</span>
        </span>
      </label>
    </div>

    <div class="file-config-grid">
      <div class="form-group">
        <label>Auto-save interval</label>
        <select
          value={$config.autoSaveInterval}
          on:change={(e) => updateConfig('autoSaveInterval', parseInt((e.target as HTMLSelectElement).value))}
          disabled={!$config.autoSave}
        >
          <option value="30">30 seconds</option>
          <option value="60">1 minute</option>
          <option value="120">2 minutes</option>
          <option value="300">5 minutes</option>
        </select>
      </div>

      <div class="form-group">
        <label>Backup retention</label>
        <select
          value={$config.backupRetention}
          on:change={(e) => updateConfig('backupRetention', (e.target as HTMLSelectElement).value)}
          disabled={!$config.createBackups}
        >
          <option value="1d">1 day</option>
          <option value="7d">7 days</option>
          <option value="30d">30 days</option>
          <option value="forever">Forever</option>
        </select>
      </div>

      <div class="form-group">
        <label>File encoding</label>
        <select
          value={$config.defaultEncoding}
          on:change={(e) => updateConfig('defaultEncoding', (e.target as HTMLSelectElement).value)}
        >
          <option value="utf-8">UTF-8</option>
          <option value="utf-16">UTF-16</option>
          <option value="ascii">ASCII</option>
        </select>
      </div>
    </div>
  </section>

  <section class="environment-section">
    <div class="section-header">
      <h3>Environment Variables</h3>
      <button class="btn secondary small" on:click={() => showEnvEditor.set(true)}>
        Add Variable
      </button>
    </div>

    {#if Object.keys($envVars).length > 0}
      <div class="env-vars-list">
        {#each Object.entries($envVars) as [name, value] (name)}
          <div class="env-var-item">
            <span class="var-name">{name}</span>
            <input
              type="text"
              class="var-value"
              value={value}
              on:input={(e) => updateEnvironmentVariable(name, (e.target as HTMLInputElement).value)}
            />
            <button
              class="action-btn danger"
              on:click={() => removeEnvironmentVariable(name)}
            >
              Remove
            </button>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-vars">
        <p>No environment variables configured</p>
      </div>
    {/if}
  </section>

  <section class="collaboration-section">
    <div class="section-header">
      <h3>Collaboration</h3>
    </div>

    <div class="sharing-config">
      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.sharingEnabled}
          on:change={(e) => updateConfig('sharingEnabled', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Enable workspace sharing</span>
          <span class="toggle-desc">Allow others to collaborate on this workspace</span>
        </span>
      </label>
    </div>

    {#if $config.sharingEnabled}
      <div class="collaborators-section" transition:slide>
        <h4>Collaborators</h4>

        <CollaboratorList
          collaborators={$collaborators}
          on:add={(e) => addCollaborator(e.detail.email, e.detail.role)}
          on:updateRole={(e) => updateCollaboratorRole(e.detail.userId, e.detail.role)}
          on:remove={(e) => removeCollaborator(e.detail)}
        />

        <div class="sharing-options">
          <div class="form-group">
            <label>Default permission for new collaborators</label>
            <select
              value={$config.defaultCollaboratorRole}
              on:change={(e) => updateConfig('defaultCollaboratorRole', (e.target as HTMLSelectElement).value)}
            >
              <option value="viewer">Viewer</option>
              <option value="editor">Editor</option>
              <option value="admin">Admin</option>
            </select>
          </div>

          <label class="toggle-option compact">
            <input
              type="checkbox"
              checked={$config.allowAnonymousView}
              on:change={(e) => updateConfig('allowAnonymousView', (e.target as HTMLInputElement).checked)}
            />
            <span>Allow anonymous viewing (with link)</span>
          </label>
        </div>
      </div>
    {/if}
  </section>

  <section class="advanced-section">
    <h3>Advanced Settings</h3>

    <div class="advanced-options">
      <div class="form-group">
        <label>Workspace isolation</label>
        <select
          value={$config.isolation}
          on:change={(e) => updateConfig('isolation', (e.target as HTMLSelectElement).value)}
        >
          <option value="none">None - Share settings with other workspaces</option>
          <option value="partial">Partial - Workspace-specific sessions only</option>
          <option value="full">Full - Completely isolated workspace</option>
        </select>
        <span class="help-text">Controls how settings and data are shared between workspaces</span>
      </div>

      <div class="form-group">
        <label>Session naming pattern</label>
        <input
          type="text"
          value={$config.sessionNamingPattern}
          on:input={(e) => updateConfig('sessionNamingPattern', (e.target as HTMLInputElement).value)}
          placeholder="{workspace}-{date}-{index}"
        />
        <span class="help-text">Template for auto-generated session names</span>
      </div>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.gitIntegration}
          on:change={(e) => updateConfig('gitIntegration', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Git integration</span>
          <span class="toggle-desc">Track session results in version control</span>
        </span>
      </label>
    </div>
  </section>

  {#if $showPathPicker}
    <div class="modal-overlay" transition:fade on:click={() => showPathPicker.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <PathPicker
          currentPath={$editingPath?.path || ''}
          on:select={(e) => handlePathSelect(e.detail)}
          on:close={() => {
            showPathPicker.set(false);
            editingPath.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showEnvEditor}
    <div class="modal-overlay" transition:fade on:click={() => showEnvEditor.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <EnvironmentEditor
          existingVars={Object.keys($envVars)}
          on:save={(e) => {
            addEnvironmentVariable(e.detail);
            showEnvEditor.set(false);
          }}
          on:close={() => showEnvEditor.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .workspace-settings {
    max-width: 900px;
  }

  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1.5rem;
  }

  .header-title h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .header-actions {
    display: flex;
    gap: 0.75rem;
  }

  section {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1.25rem;
  }

  section h4 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.25rem;
  }

  .section-header h3 {
    margin-bottom: 0;
  }

  .info-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  .form-group label {
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  .form-group input,
  .form-group select {
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .help-text {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.375rem;
  }

  .paths-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .path-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .path-info {
    display: flex;
    flex-direction: column;
    min-width: 150px;
  }

  .path-name {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .path-name .required {
    color: var(--error-color);
  }

  .path-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .path-input {
    display: flex;
    gap: 0.5rem;
    flex: 1;
  }

  .path-input input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
    font-family: monospace;
  }

  .toggle-options {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1.25rem;
  }

  .toggle-option {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    cursor: pointer;
  }

  .toggle-option.compact {
    padding: 0.5rem 0;
    background: transparent;
    font-size: 0.875rem;
  }

  .toggle-content {
    display: flex;
    flex-direction: column;
  }

  .toggle-label {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .toggle-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.125rem;
  }

  .file-config-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1.25rem;
  }

  .env-vars-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .env-var-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .var-name {
    font-weight: 500;
    font-family: monospace;
    font-size: 0.875rem;
    min-width: 150px;
  }

  .var-value {
    flex: 1;
    padding: 0.375rem 0.625rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-family: monospace;
    font-size: 0.875rem;
  }

  .empty-vars {
    padding: 1.5rem;
    text-align: center;
    color: var(--text-muted);
  }

  .collaborators-section {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .sharing-options {
    margin-top: 1rem;
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .advanced-options {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .action-btn {
    padding: 0.375rem 0.625rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .action-btn.danger:hover {
    border-color: var(--error-color);
    color: var(--error-color);
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn.small {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
  }

  .btn.primary {
    background: var(--primary-color);
    color: white;
  }

  .btn.secondary {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--card-bg);
    border-radius: 8px;
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  @media (max-width: 768px) {
    .info-grid,
    .file-config-grid {
      grid-template-columns: 1fr;
    }

    .path-item {
      flex-direction: column;
      align-items: flex-start;
    }

    .path-input {
      width: 100%;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test workspace configuration updates
2. **Path Tests**: Test directory path validation
3. **Environment Tests**: Test environment variable handling
4. **Collaboration Tests**: Test collaborator management
5. **File Handling Tests**: Test file watching and backup

## Related Specs
- Spec 290: Profile Management
- Spec 292: Git Settings
- Spec 295: Settings Tests
