# Spec 292: Git Settings

## Header
- **Spec ID**: 292
- **Phase**: 13 - Settings UI
- **Component**: Git Settings
- **Dependencies**: Spec 291 (Workspace Settings)
- **Status**: Draft

## Objective
Create a Git integration settings interface that allows users to configure version control behavior, repository connections, commit settings, and branch management for tracking session results and configurations.

## Acceptance Criteria
- [x] Configure Git repository connections
- [x] Set up authentication methods (SSH, token)
- [x] Configure commit behavior and templates
- [x] Set branch management preferences
- [x] Configure auto-commit triggers
- [x] Set up remote sync behavior
- [x] Configure diff and merge preferences
- [x] View Git operation history

## Implementation

### GitSettings.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import RepositoryConfig from './RepositoryConfig.svelte';
  import AuthConfig from './AuthConfig.svelte';
  import CommitTemplateEditor from './CommitTemplateEditor.svelte';
  import { gitSettingsStore } from '$lib/stores/gitSettings';
  import type {
    GitConfig,
    Repository,
    GitAuthMethod,
    CommitTemplate
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: GitConfig;
    test: { repoId: string };
  }>();

  let showRepoConfig = writable<boolean>(false);
  let showAuthConfig = writable<boolean>(false);
  let showTemplateEditor = writable<boolean>(false);
  let editingRepo = writable<Repository | null>(null);
  let testingConnection = writable<string | null>(null);

  const config = derived(gitSettingsStore, ($store) => $store.config);
  const repositories = derived(gitSettingsStore, ($store) => $store.repositories);
  const activeRepo = derived(gitSettingsStore, ($store) => $store.activeRepository);
  const recentOperations = derived(gitSettingsStore, ($store) => $store.recentOperations);

  const authMethods: { id: GitAuthMethod; name: string; description: string }[] = [
    { id: 'ssh', name: 'SSH Key', description: 'Authenticate using SSH key pair' },
    { id: 'token', name: 'Personal Access Token', description: 'Authenticate using API token' },
    { id: 'https', name: 'HTTPS', description: 'Authenticate using username/password' },
    { id: 'none', name: 'None', description: 'No authentication (public repos only)' }
  ];

  const autoCommitTriggers = [
    { id: 'session_complete', label: 'Session completed' },
    { id: 'session_save', label: 'Session saved' },
    { id: 'config_change', label: 'Configuration changed' },
    { id: 'template_update', label: 'Template updated' },
    { id: 'manual', label: 'Manual only' }
  ];

  function updateConfig(field: keyof GitConfig, value: unknown) {
    gitSettingsStore.updateConfig(field, value);
  }

  function updateBranchConfig(field: string, value: unknown) {
    gitSettingsStore.updateBranchConfig(field, value);
  }

  async function addRepository(repo: Omit<Repository, 'id'>) {
    await gitSettingsStore.addRepository(repo);
    showRepoConfig.set(false);
  }

  async function updateRepository(repo: Repository) {
    await gitSettingsStore.updateRepository(repo);
    showRepoConfig.set(false);
    editingRepo.set(null);
  }

  async function removeRepository(repoId: string) {
    if (confirm('Remove this repository? This will not delete the actual repository.')) {
      await gitSettingsStore.removeRepository(repoId);
    }
  }

  async function setActiveRepository(repoId: string) {
    await gitSettingsStore.setActiveRepository(repoId);
  }

  async function testConnection(repoId: string) {
    testingConnection.set(repoId);
    try {
      const result = await gitSettingsStore.testConnection(repoId);
      if (result.success) {
        alert('Connection successful!');
      } else {
        alert('Connection failed: ' + result.error);
      }
    } finally {
      testingConnection.set(null);
    }
    dispatch('test', { repoId });
  }

  function openRepoConfig(repo?: Repository) {
    editingRepo.set(repo || null);
    showRepoConfig.set(true);
  }

  async function saveSettings() {
    await gitSettingsStore.save();
    dispatch('save', $config);
  }

  function formatDate(date: Date): string {
    return new Date(date).toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  onMount(() => {
    gitSettingsStore.load();
  });
</script>

<div class="git-settings" data-testid="git-settings">
  <header class="config-header">
    <div class="header-title">
      <h2>Git Settings</h2>
      <p class="description">Configure version control and repository settings</p>
    </div>

    <div class="header-actions">
      <button class="btn primary" on:click={saveSettings}>
        Save Settings
      </button>
    </div>
  </header>

  <section class="git-toggle">
    <label class="master-toggle">
      <input
        type="checkbox"
        checked={$config.enabled}
        on:change={(e) => updateConfig('enabled', (e.target as HTMLInputElement).checked)}
      />
      <div class="toggle-info">
        <span class="toggle-label">Enable Git Integration</span>
        <span class="toggle-desc">Track changes and sync with remote repositories</span>
      </div>
    </label>
  </section>

  {#if $config.enabled}
    <section class="repositories-section" transition:slide>
      <div class="section-header">
        <h3>Repositories</h3>
        <button class="btn secondary small" on:click={() => openRepoConfig()}>
          Add Repository
        </button>
      </div>

      {#if $repositories.length > 0}
        <div class="repo-list">
          {#each $repositories as repo (repo.id)}
            <div
              class="repo-card"
              class:active={repo.id === $activeRepo?.id}
            >
              <div class="repo-header">
                <div class="repo-info">
                  <span class="repo-name">{repo.name}</span>
                  {#if repo.id === $activeRepo?.id}
                    <span class="badge">Active</span>
                  {/if}
                </div>
                <div class="repo-actions">
                  <button
                    class="action-btn"
                    on:click={() => testConnection(repo.id)}
                    disabled={$testingConnection === repo.id}
                  >
                    {$testingConnection === repo.id ? 'Testing...' : 'Test'}
                  </button>
                  <button
                    class="action-btn"
                    on:click={() => openRepoConfig(repo)}
                  >
                    Edit
                  </button>
                </div>
              </div>

              <div class="repo-url">{repo.url}</div>

              <div class="repo-meta">
                <span>Branch: {repo.defaultBranch}</span>
                <span>Auth: {authMethods.find(a => a.id === repo.authMethod)?.name}</span>
              </div>

              <div class="repo-footer">
                {#if repo.id !== $activeRepo?.id}
                  <button
                    class="btn secondary small"
                    on:click={() => setActiveRepository(repo.id)}
                  >
                    Set Active
                  </button>
                {/if}
                <button
                  class="btn secondary small danger"
                  on:click={() => removeRepository(repo.id)}
                >
                  Remove
                </button>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="empty-state">
          <p>No repositories configured</p>
          <p class="hint">Add a repository to start tracking changes</p>
        </div>
      {/if}
    </section>

    <section class="commit-section" transition:slide>
      <h3>Commit Settings</h3>

      <div class="commit-config">
        <div class="form-group">
          <label for="author-name">Author Name</label>
          <input
            id="author-name"
            type="text"
            value={$config.authorName}
            on:input={(e) => updateConfig('authorName', (e.target as HTMLInputElement).value)}
            placeholder="Your Name"
          />
        </div>

        <div class="form-group">
          <label for="author-email">Author Email</label>
          <input
            id="author-email"
            type="email"
            value={$config.authorEmail}
            on:input={(e) => updateConfig('authorEmail', (e.target as HTMLInputElement).value)}
            placeholder="you@example.com"
          />
        </div>
      </div>

      <div class="commit-options">
        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$config.signCommits}
            on:change={(e) => updateConfig('signCommits', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Sign commits with GPG</span>
            <span class="toggle-desc">Cryptographically sign all commits</span>
          </span>
        </label>

        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$config.includeTimestamp}
            on:change={(e) => updateConfig('includeTimestamp', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Include timestamp in commit message</span>
            <span class="toggle-desc">Add timestamp to auto-generated commit messages</span>
          </span>
        </label>
      </div>

      <div class="template-config">
        <div class="template-header">
          <label>Commit Message Template</label>
          <button
            class="btn secondary small"
            on:click={() => showTemplateEditor.set(true)}
          >
            Edit Template
          </button>
        </div>
        <pre class="template-preview">{$config.commitTemplate || '[type]: [description]\n\nSession: [session_id]'}</pre>
      </div>
    </section>

    <section class="auto-commit-section" transition:slide>
      <h3>Auto-Commit</h3>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$config.autoCommit}
          on:change={(e) => updateConfig('autoCommit', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Enable auto-commit</span>
          <span class="toggle-desc">Automatically commit changes on triggers</span>
        </span>
      </label>

      {#if $config.autoCommit}
        <div class="trigger-options" transition:slide>
          <label>Auto-commit triggers:</label>
          <div class="trigger-list">
            {#each autoCommitTriggers as trigger}
              <label class="trigger-option">
                <input
                  type="checkbox"
                  checked={$config.autoCommitTriggers?.includes(trigger.id)}
                  on:change={() => gitSettingsStore.toggleAutoCommitTrigger(trigger.id)}
                />
                {trigger.label}
              </label>
            {/each}
          </div>
        </div>
      {/if}
    </section>

    <section class="branch-section" transition:slide>
      <h3>Branch Management</h3>

      <div class="branch-config">
        <div class="form-group">
          <label for="default-branch">Default Branch</label>
          <input
            id="default-branch"
            type="text"
            value={$config.branchConfig.defaultBranch}
            on:input={(e) => updateBranchConfig('defaultBranch', (e.target as HTMLInputElement).value)}
            placeholder="main"
          />
        </div>

        <div class="form-group">
          <label for="branch-prefix">Feature Branch Prefix</label>
          <input
            id="branch-prefix"
            type="text"
            value={$config.branchConfig.featureBranchPrefix}
            on:input={(e) => updateBranchConfig('featureBranchPrefix', (e.target as HTMLInputElement).value)}
            placeholder="session/"
          />
          <span class="help-text">Prefix for auto-created session branches</span>
        </div>
      </div>

      <div class="branch-options">
        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$config.branchConfig.createFeatureBranches}
            on:change={(e) => updateBranchConfig('createFeatureBranches', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Create feature branches for sessions</span>
            <span class="toggle-desc">Each session creates its own branch</span>
          </span>
        </label>

        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$config.branchConfig.autoMerge}
            on:change={(e) => updateBranchConfig('autoMerge', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Auto-merge completed sessions</span>
            <span class="toggle-desc">Automatically merge session branches when complete</span>
          </span>
        </label>

        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$config.branchConfig.deleteAfterMerge}
            on:change={(e) => updateBranchConfig('deleteAfterMerge', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Delete branch after merge</span>
            <span class="toggle-desc">Clean up feature branches after merging</span>
          </span>
        </label>
      </div>
    </section>

    <section class="sync-section" transition:slide>
      <h3>Remote Sync</h3>

      <div class="sync-options">
        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$config.autoPush}
            on:change={(e) => updateConfig('autoPush', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Auto-push commits</span>
            <span class="toggle-desc">Automatically push commits to remote</span>
          </span>
        </label>

        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$config.autoPull}
            on:change={(e) => updateConfig('autoPull', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Auto-pull on startup</span>
            <span class="toggle-desc">Pull latest changes when opening workspace</span>
          </span>
        </label>
      </div>

      <div class="form-group">
        <label>Sync frequency</label>
        <select
          value={$config.syncFrequency}
          on:change={(e) => updateConfig('syncFrequency', (e.target as HTMLSelectElement).value)}
        >
          <option value="immediate">Immediate</option>
          <option value="5min">Every 5 minutes</option>
          <option value="15min">Every 15 minutes</option>
          <option value="hourly">Hourly</option>
          <option value="manual">Manual only</option>
        </select>
      </div>
    </section>

    {#if $recentOperations.length > 0}
      <section class="history-section" transition:slide>
        <h3>Recent Operations</h3>

        <div class="operations-list">
          {#each $recentOperations.slice(0, 10) as op (op.id)}
            <div class="operation-item" class:error={op.status === 'error'}>
              <div class="op-info">
                <span class="op-type">{op.type}</span>
                <span class="op-message">{op.message}</span>
              </div>
              <span class="op-time">{formatDate(op.timestamp)}</span>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}

  {#if $showRepoConfig}
    <div class="modal-overlay" transition:fade on:click={() => showRepoConfig.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <RepositoryConfig
          repository={$editingRepo}
          authMethods={authMethods}
          on:save={(e) => $editingRepo ? updateRepository(e.detail) : addRepository(e.detail)}
          on:close={() => {
            showRepoConfig.set(false);
            editingRepo.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showAuthConfig}
    <div class="modal-overlay" transition:fade on:click={() => showAuthConfig.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <AuthConfig
          currentMethod={$activeRepo?.authMethod || 'none'}
          on:save={(e) => {
            if ($activeRepo) {
              gitSettingsStore.updateRepository({ ...$activeRepo, authMethod: e.detail.method });
            }
            showAuthConfig.set(false);
          }}
          on:close={() => showAuthConfig.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showTemplateEditor}
    <div class="modal-overlay" transition:fade on:click={() => showTemplateEditor.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <CommitTemplateEditor
          template={$config.commitTemplate}
          on:save={(e) => {
            updateConfig('commitTemplate', e.detail);
            showTemplateEditor.set(false);
          }}
          on:close={() => showTemplateEditor.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .git-settings {
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

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.25rem;
  }

  .section-header h3 {
    margin-bottom: 0;
  }

  .master-toggle {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
  }

  .toggle-label {
    font-weight: 500;
    font-size: 0.9375rem;
  }

  .toggle-desc {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .repo-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .repo-card {
    padding: 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
  }

  .repo-card.active {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .repo-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .repo-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .repo-name {
    font-weight: 600;
    font-size: 0.9375rem;
  }

  .badge {
    padding: 0.125rem 0.5rem;
    background: var(--primary-color);
    color: white;
    border-radius: 4px;
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .repo-actions {
    display: flex;
    gap: 0.5rem;
  }

  .repo-url {
    font-family: monospace;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
  }

  .repo-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.75rem;
  }

  .repo-footer {
    display: flex;
    gap: 0.5rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-color);
  }

  .commit-config {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
    margin-bottom: 1.25rem;
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

  .commit-options,
  .branch-options,
  .sync-options {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
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

  .toggle-content {
    display: flex;
    flex-direction: column;
  }

  .template-config {
    margin-top: 1rem;
  }

  .template-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .template-preview {
    padding: 0.75rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    font-family: monospace;
    font-size: 0.8125rem;
    white-space: pre-wrap;
    margin: 0;
  }

  .trigger-options {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .trigger-options > label {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
  }

  .trigger-list {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
  }

  .trigger-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .branch-config {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
    margin-bottom: 1.25rem;
  }

  .operations-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .operation-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.8125rem;
  }

  .operation-item.error {
    background: var(--error-alpha);
  }

  .op-info {
    display: flex;
    gap: 0.75rem;
  }

  .op-type {
    font-weight: 500;
    text-transform: capitalize;
  }

  .op-message {
    color: var(--text-secondary);
  }

  .op-time {
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .action-btn {
    padding: 0.375rem 0.625rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .action-btn:hover {
    border-color: var(--primary-color);
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

  .btn.danger:hover {
    border-color: var(--error-color);
    color: var(--error-color);
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
    .commit-config,
    .branch-config {
      grid-template-columns: 1fr;
    }

    .trigger-list {
      grid-template-columns: 1fr;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test Git configuration updates
2. **Connection Tests**: Test repository connection validation
3. **Auth Tests**: Test authentication methods
4. **Commit Tests**: Test auto-commit functionality
5. **Branch Tests**: Test branch management

## Related Specs
- Spec 291: Workspace Settings
- Spec 293: Telemetry Prefs
- Spec 295: Settings Tests
