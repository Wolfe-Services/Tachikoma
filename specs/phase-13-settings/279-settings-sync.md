# 279 - Settings Sync

**Phase:** 13 - Settings UI
**Spec ID:** 279
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~10% of model context window

---

## Objective

Create the Settings Sync panel that allows users to configure cloud synchronization of their settings across devices, supporting multiple sync providers (GitHub Gist, custom endpoints) with selective sync options and conflict resolution.

---

## Acceptance Criteria

- [ ] `SyncSettings.svelte` component with sync configuration
- [ ] Enable/disable sync toggle
- [ ] Sync provider selection (GitHub Gist, custom)
- [ ] GitHub Gist authentication flow
- [ ] Manual sync trigger buttons
- [ ] Sync status display with last sync time
- [ ] Selective sync options (choose what to sync)
- [ ] Conflict resolution settings
- [ ] Sync history/log viewer

---

## Implementation Details

### 1. Sync Types (src/lib/types/sync.ts)

```typescript
/**
 * Settings sync type definitions.
 */

export type SyncProvider = 'none' | 'github-gist' | 'custom';

export interface SyncStatus {
  connected: boolean;
  lastSync: number | null;
  lastError: string | null;
  syncInProgress: boolean;
  direction: 'upload' | 'download' | null;
  progress: number;
}

export interface SyncConflict {
  id: string;
  setting: string;
  localValue: unknown;
  remoteValue: unknown;
  localModified: number;
  remoteModified: number;
}

export interface SyncHistoryEntry {
  id: string;
  timestamp: number;
  direction: 'upload' | 'download';
  provider: SyncProvider;
  success: boolean;
  error?: string;
  changesCount: number;
}

export interface SyncableSection {
  id: string;
  label: string;
  description: string;
  settingsKey: string;
  enabled: boolean;
}

export const SYNCABLE_SECTIONS: SyncableSection[] = [
  { id: 'general', label: 'General Settings', description: 'Language, updates, startup options', settingsKey: 'general', enabled: true },
  { id: 'appearance', label: 'Appearance', description: 'Theme, colors, fonts', settingsKey: 'appearance', enabled: true },
  { id: 'editor', label: 'Editor Preferences', description: 'Tab size, word wrap, formatting', settingsKey: 'editor', enabled: true },
  { id: 'keybindings', label: 'Keyboard Shortcuts', description: 'Custom key bindings', settingsKey: 'keybindings', enabled: true },
  { id: 'backends', label: 'LLM Backends', description: 'Backend configurations (excluding API keys)', settingsKey: 'backends', enabled: false },
  { id: 'git', label: 'Git Settings', description: 'Git integration preferences', settingsKey: 'git', enabled: true },
];

export type ConflictResolution = 'local' | 'remote' | 'newest' | 'ask';
```

### 2. Sync Store (src/lib/stores/sync-store.ts)

```typescript
import { writable, derived, get } from 'svelte/store';
import type { SyncStatus, SyncHistoryEntry, SyncConflict, SyncProvider } from '$lib/types/sync';
import { syncSettings, settingsStore } from './settings-store';
import { invoke } from '$lib/ipc';

function createSyncStore() {
  const status = writable<SyncStatus>({
    connected: false,
    lastSync: null,
    lastError: null,
    syncInProgress: false,
    direction: null,
    progress: 0,
  });

  const history = writable<SyncHistoryEntry[]>([]);
  const conflicts = writable<SyncConflict[]>([]);

  return {
    status: { subscribe: status.subscribe },
    history: { subscribe: history.subscribe },
    conflicts: { subscribe: conflicts.subscribe },

    async connect(provider: SyncProvider, credentials?: Record<string, string>): Promise<boolean> {
      try {
        status.update(s => ({ ...s, syncInProgress: true }));

        const result = await invoke<{ success: boolean; error?: string }>('sync_connect', {
          provider,
          credentials,
        });

        if (result.success) {
          status.update(s => ({
            ...s,
            connected: true,
            lastError: null,
            syncInProgress: false,
          }));
          settingsStore.updateCategory('sync', { enabled: true, provider });
          return true;
        } else {
          status.update(s => ({
            ...s,
            connected: false,
            lastError: result.error || 'Connection failed',
            syncInProgress: false,
          }));
          return false;
        }
      } catch (error) {
        status.update(s => ({
          ...s,
          connected: false,
          lastError: (error as Error).message,
          syncInProgress: false,
        }));
        return false;
      }
    },

    async disconnect(): Promise<void> {
      await invoke('sync_disconnect');
      status.update(s => ({
        ...s,
        connected: false,
        lastError: null,
      }));
      settingsStore.updateCategory('sync', { enabled: false, provider: 'none' });
    },

    async syncNow(direction: 'upload' | 'download' = 'upload'): Promise<boolean> {
      status.update(s => ({
        ...s,
        syncInProgress: true,
        direction,
        progress: 0,
      }));

      try {
        const result = await invoke<{ success: boolean; conflicts?: SyncConflict[]; error?: string }>('sync_now', {
          direction,
          sections: get(syncSettings),
        });

        if (result.conflicts && result.conflicts.length > 0) {
          conflicts.set(result.conflicts);
        }

        const entry: SyncHistoryEntry = {
          id: crypto.randomUUID(),
          timestamp: Date.now(),
          direction,
          provider: get(syncSettings).provider,
          success: result.success,
          error: result.error,
          changesCount: 0,
        };

        history.update(h => [entry, ...h].slice(0, 50));

        status.update(s => ({
          ...s,
          syncInProgress: false,
          direction: null,
          progress: 100,
          lastSync: result.success ? Date.now() : s.lastSync,
          lastError: result.error || null,
        }));

        if (result.success) {
          settingsStore.updateCategory('sync', { lastSyncTime: Date.now() });
        }

        return result.success;
      } catch (error) {
        status.update(s => ({
          ...s,
          syncInProgress: false,
          direction: null,
          lastError: (error as Error).message,
        }));
        return false;
      }
    },

    async resolveConflict(conflictId: string, resolution: 'local' | 'remote'): Promise<void> {
      const currentConflicts = get(conflicts);
      const conflict = currentConflicts.find(c => c.id === conflictId);

      if (conflict) {
        await invoke('sync_resolve_conflict', {
          conflictId,
          resolution,
          value: resolution === 'local' ? conflict.localValue : conflict.remoteValue,
        });

        conflicts.update(c => c.filter(x => x.id !== conflictId));
      }
    },

    async loadHistory(): Promise<void> {
      const entries = await invoke<SyncHistoryEntry[]>('sync_get_history');
      history.set(entries);
    },

    clearHistory(): void {
      history.set([]);
      invoke('sync_clear_history');
    },
  };
}

export const syncStore = createSyncStore();
```

### 3. Sync Settings Component (src/lib/components/settings/SyncSettings.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { settingsStore, syncSettings } from '$lib/stores/settings-store';
  import { syncStore } from '$lib/stores/sync-store';
  import { SYNCABLE_SECTIONS } from '$lib/types/sync';
  import type { SyncProvider, ConflictResolution } from '$lib/types/sync';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let showGistModal = false;
  let gistToken = '';
  let gistId = '';
  let isConnecting = false;
  let connectionError = '';

  let conflictResolution: ConflictResolution = 'newest';
  let syncableSections = [...SYNCABLE_SECTIONS];

  function formatLastSync(timestamp: number | null): string {
    if (!timestamp) return 'Never';
    const date = new Date(timestamp);
    const now = Date.now();
    const diff = now - timestamp;

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)} minutes ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)} hours ago`;
    return date.toLocaleDateString();
  }

  function formatHistoryTime(timestamp: number): string {
    return new Date(timestamp).toLocaleString();
  }

  async function connectGitHubGist() {
    if (!gistToken) {
      connectionError = 'Please enter your GitHub token';
      return;
    }

    isConnecting = true;
    connectionError = '';

    const success = await syncStore.connect('github-gist', {
      token: gistToken,
      gistId: gistId || undefined,
    });

    if (success) {
      showGistModal = false;
      gistToken = '';
    } else {
      connectionError = $syncStore.status.lastError || 'Connection failed';
    }

    isConnecting = false;
  }

  async function handleProviderChange(provider: SyncProvider) {
    if (provider === 'github-gist') {
      showGistModal = true;
    } else if (provider === 'custom') {
      // Handle custom endpoint
    } else {
      await syncStore.disconnect();
    }
  }

  function toggleSyncSection(sectionId: string) {
    syncableSections = syncableSections.map(s =>
      s.id === sectionId ? { ...s, enabled: !s.enabled } : s
    );
  }

  onMount(() => {
    syncStore.loadHistory();
  });
</script>

<div class="sync-settings">
  <h2 class="settings-title">Settings Sync</h2>
  <p class="settings-description">
    Synchronize your settings across devices using cloud storage.
  </p>

  <!-- Sync Status -->
  <SettingsSection title="Sync Status">
    <div class="sync-status">
      <div class="sync-status__indicator" class:sync-status__indicator--connected={$syncStore.status.connected}>
        <Icon name={$syncStore.status.connected ? 'cloud' : 'cloud-off'} size={32} />
      </div>
      <div class="sync-status__info">
        <div class="sync-status__state">
          {#if $syncStore.status.connected}
            <span class="sync-status__connected">Connected</span>
            <span class="sync-status__provider">via {$syncSettings.provider}</span>
          {:else}
            <span class="sync-status__disconnected">Not connected</span>
          {/if}
        </div>
        <div class="sync-status__details">
          <span>Last sync: {formatLastSync($syncSettings.lastSyncTime || null)}</span>
          {#if $syncStore.status.lastError}
            <span class="sync-status__error">Error: {$syncStore.status.lastError}</span>
          {/if}
        </div>
      </div>
      <div class="sync-status__actions">
        {#if $syncStore.status.connected}
          <Button
            variant="secondary"
            size="small"
            disabled={$syncStore.status.syncInProgress}
            on:click={() => syncStore.syncNow('upload')}
          >
            {#if $syncStore.status.syncInProgress && $syncStore.status.direction === 'upload'}
              <Icon name="loader" size={14} class="spinning" />
            {:else}
              <Icon name="upload-cloud" size={14} />
            {/if}
            Upload
          </Button>
          <Button
            variant="secondary"
            size="small"
            disabled={$syncStore.status.syncInProgress}
            on:click={() => syncStore.syncNow('download')}
          >
            {#if $syncStore.status.syncInProgress && $syncStore.status.direction === 'download'}
              <Icon name="loader" size={14} class="spinning" />
            {:else}
              <Icon name="download-cloud" size={14} />
            {/if}
            Download
          </Button>
        {/if}
      </div>
    </div>

    {#if $syncStore.status.syncInProgress}
      <div class="sync-progress">
        <div class="sync-progress__bar">
          <div class="sync-progress__fill" style="width: {$syncStore.status.progress}%" />
        </div>
        <span class="sync-progress__text">
          {$syncStore.status.direction === 'upload' ? 'Uploading' : 'Downloading'}...
        </span>
      </div>
    {/if}
  </SettingsSection>

  <!-- Sync Provider -->
  <SettingsSection title="Sync Provider">
    <SettingsRow
      label="Provider"
      description="Choose where to store your synced settings"
    >
      <Select
        value={$syncSettings.provider}
        options={[
          { value: 'none', label: 'None (disabled)' },
          { value: 'github-gist', label: 'GitHub Gist' },
          { value: 'custom', label: 'Custom Endpoint' },
        ]}
        on:change={(e) => handleProviderChange(e.detail as SyncProvider)}
      />
    </SettingsRow>

    {#if $syncSettings.provider === 'custom'}
      <SettingsRow
        label="Custom Endpoint"
        description="URL of your sync server"
      >
        <Input
          type="url"
          value={$syncSettings.customEndpoint || ''}
          placeholder="https://sync.example.com/api"
          on:input={(e) => settingsStore.updateSetting('sync', 'customEndpoint', e.target.value)}
        />
      </SettingsRow>
    {/if}

    {#if $syncStore.status.connected}
      <div class="provider-actions">
        <Button variant="ghost" on:click={() => syncStore.disconnect()}>
          <Icon name="log-out" size={14} />
          Disconnect
        </Button>
      </div>
    {/if}
  </SettingsSection>

  <!-- Auto Sync -->
  {#if $syncStore.status.connected}
    <SettingsSection title="Auto Sync">
      <SettingsRow
        label="Auto Sync"
        description="Automatically sync settings periodically"
      >
        <Toggle
          checked={$syncSettings.autoSync}
          on:change={(e) => settingsStore.updateSetting('sync', 'autoSync', e.detail)}
        />
      </SettingsRow>

      {#if $syncSettings.autoSync}
        <SettingsRow
          label="Sync Interval"
          description="How often to sync automatically"
        >
          <Select
            value={$syncSettings.syncInterval.toString()}
            options={[
              { value: '900000', label: 'Every 15 minutes' },
              { value: '1800000', label: 'Every 30 minutes' },
              { value: '3600000', label: 'Every hour' },
              { value: '86400000', label: 'Once a day' },
            ]}
            on:change={(e) => settingsStore.updateSetting('sync', 'syncInterval', parseInt(e.detail))}
          />
        </SettingsRow>
      {/if}

      <SettingsRow
        label="Sync on Startup"
        description="Download settings when the app starts"
      >
        <Toggle
          checked={$syncSettings.syncOnStartup}
          on:change={(e) => settingsStore.updateSetting('sync', 'syncOnStartup', e.detail)}
        />
      </SettingsRow>
    </SettingsSection>

    <!-- What to Sync -->
    <SettingsSection title="What to Sync">
      <p class="section-note">
        Choose which settings to include in sync. API keys are never synced.
      </p>
      <div class="sync-sections">
        {#each syncableSections as section}
          <div class="sync-section">
            <div class="sync-section__info">
              <span class="sync-section__label">{section.label}</span>
              <span class="sync-section__desc">{section.description}</span>
            </div>
            <Toggle
              checked={section.enabled}
              on:change={() => toggleSyncSection(section.id)}
            />
          </div>
        {/each}
      </div>
    </SettingsSection>

    <!-- Conflict Resolution -->
    <SettingsSection title="Conflict Resolution">
      <SettingsRow
        label="When conflicts occur"
        description="How to handle settings that differ between local and cloud"
      >
        <Select
          value={conflictResolution}
          options={[
            { value: 'local', label: 'Keep local changes' },
            { value: 'remote', label: 'Use cloud version' },
            { value: 'newest', label: 'Use newest changes' },
            { value: 'ask', label: 'Ask me each time' },
          ]}
          on:change={(e) => conflictResolution = e.detail as ConflictResolution}
        />
      </SettingsRow>
    </SettingsSection>

    <!-- Sync History -->
    <SettingsSection title="Sync History" collapsible collapsed>
      {#if $syncStore.history.length > 0}
        <div class="sync-history">
          {#each $syncStore.history.slice(0, 10) as entry}
            <div
              class="sync-history__entry"
              class:sync-history__entry--success={entry.success}
              class:sync-history__entry--error={!entry.success}
            >
              <Icon
                name={entry.direction === 'upload' ? 'upload-cloud' : 'download-cloud'}
                size={16}
              />
              <span class="sync-history__time">{formatHistoryTime(entry.timestamp)}</span>
              <span class="sync-history__status">
                {entry.success ? 'Success' : entry.error || 'Failed'}
              </span>
            </div>
          {/each}
        </div>
        <Button variant="ghost" size="small" on:click={() => syncStore.clearHistory()}>
          Clear History
        </Button>
      {:else}
        <p class="sync-history__empty">No sync history yet</p>
      {/if}
    </SettingsSection>
  {/if}

  <!-- Conflicts -->
  {#if $syncStore.conflicts.length > 0}
    <SettingsSection title="Conflicts">
      <div class="conflicts">
        {#each $syncStore.conflicts as conflict}
          <div class="conflict">
            <div class="conflict__setting">{conflict.setting}</div>
            <div class="conflict__values">
              <div class="conflict__value conflict__value--local">
                <span class="conflict__label">Local:</span>
                <code>{JSON.stringify(conflict.localValue)}</code>
              </div>
              <div class="conflict__value conflict__value--remote">
                <span class="conflict__label">Cloud:</span>
                <code>{JSON.stringify(conflict.remoteValue)}</code>
              </div>
            </div>
            <div class="conflict__actions">
              <Button
                variant="secondary"
                size="small"
                on:click={() => syncStore.resolveConflict(conflict.id, 'local')}
              >
                Keep Local
              </Button>
              <Button
                variant="secondary"
                size="small"
                on:click={() => syncStore.resolveConflict(conflict.id, 'remote')}
              >
                Use Cloud
              </Button>
            </div>
          </div>
        {/each}
      </div>
    </SettingsSection>
  {/if}
</div>

<!-- GitHub Gist Modal -->
{#if showGistModal}
  <Modal title="Connect to GitHub Gist" on:close={() => showGistModal = false}>
    <div class="gist-modal">
      <p class="gist-modal__intro">
        Create a <a href="https://github.com/settings/tokens/new?scopes=gist" target="_blank">personal access token</a>
        with the <code>gist</code> scope to sync your settings.
      </p>

      <div class="form-group">
        <label for="gist-token">GitHub Token</label>
        <Input
          id="gist-token"
          type="password"
          bind:value={gistToken}
          placeholder="ghp_..."
        />
      </div>

      <div class="form-group">
        <label for="gist-id">Gist ID (optional)</label>
        <Input
          id="gist-id"
          type="text"
          bind:value={gistId}
          placeholder="Leave empty to create new"
        />
        <p class="form-hint">Enter an existing Gist ID to sync with, or leave empty to create a new one.</p>
      </div>

      {#if connectionError}
        <div class="gist-modal__error">
          <Icon name="alert-circle" size={14} />
          {connectionError}
        </div>
      {/if}

      <div class="gist-modal__actions">
        <Button variant="secondary" on:click={() => showGistModal = false}>
          Cancel
        </Button>
        <Button
          variant="primary"
          disabled={isConnecting || !gistToken}
          on:click={connectGitHubGist}
        >
          {#if isConnecting}
            <Icon name="loader" size={14} class="spinning" />
          {/if}
          Connect
        </Button>
      </div>
    </div>
  </Modal>
{/if}

<style>
  .sync-settings {
    max-width: 720px;
  }

  .settings-title {
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .settings-description {
    color: var(--color-text-secondary);
    font-size: 14px;
    margin: 0 0 24px 0;
  }

  /* Sync Status */
  .sync-status {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }

  .sync-status__indicator {
    width: 56px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-hover);
    border-radius: 12px;
    color: var(--color-text-muted);
  }

  .sync-status__indicator--connected {
    color: var(--color-success);
    background: rgba(76, 175, 80, 0.1);
  }

  .sync-status__info {
    flex: 1;
  }

  .sync-status__state {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .sync-status__connected {
    font-weight: 600;
    color: var(--color-success);
  }

  .sync-status__disconnected {
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .sync-status__provider {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .sync-status__details {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .sync-status__error {
    color: var(--color-error);
  }

  .sync-status__actions {
    display: flex;
    gap: 8px;
  }

  .sync-progress {
    margin-top: 12px;
  }

  .sync-progress__bar {
    height: 4px;
    background: var(--color-bg-hover);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .sync-progress__fill {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.3s ease;
  }

  .sync-progress__text {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .provider-actions {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
  }

  /* Sync Sections */
  .section-note {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 0 0 16px 0;
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 6px;
  }

  .sync-sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sync-section {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 6px;
  }

  .sync-section__info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .sync-section__label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .sync-section__desc {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  /* Sync History */
  .sync-history {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 12px;
  }

  .sync-history__entry {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--color-bg-secondary);
    border-radius: 6px;
    font-size: 13px;
  }

  .sync-history__entry--success {
    color: var(--color-success);
  }

  .sync-history__entry--error {
    color: var(--color-error);
  }

  .sync-history__time {
    color: var(--color-text-secondary);
    min-width: 150px;
  }

  .sync-history__status {
    color: var(--color-text-primary);
  }

  .sync-history__empty {
    color: var(--color-text-muted);
    font-size: 13px;
    text-align: center;
    padding: 24px;
  }

  /* Conflicts */
  .conflicts {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .conflict {
    padding: 16px;
    background: rgba(255, 152, 0, 0.05);
    border: 1px solid var(--color-warning);
    border-radius: 8px;
  }

  .conflict__setting {
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 12px;
  }

  .conflict__values {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 12px;
  }

  .conflict__value {
    padding: 8px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
  }

  .conflict__label {
    display: block;
    font-size: 11px;
    color: var(--color-text-muted);
    margin-bottom: 4px;
  }

  .conflict__value code {
    font-size: 12px;
    color: var(--color-text-primary);
    word-break: break-all;
  }

  .conflict__actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  /* Gist Modal */
  .gist-modal {
    padding: 8px;
  }

  .gist-modal__intro {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0 0 20px 0;
  }

  .gist-modal__intro a {
    color: var(--color-primary);
  }

  .gist-modal__intro code {
    padding: 2px 6px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
    font-size: 13px;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    margin-bottom: 8px;
  }

  .form-hint {
    font-size: 12px;
    color: var(--color-text-muted);
    margin-top: 6px;
  }

  .gist-modal__error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    background: rgba(244, 67, 54, 0.1);
    color: var(--color-error);
    border-radius: 6px;
    font-size: 13px;
    margin-bottom: 16px;
  }

  .gist-modal__actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 20px;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
```

---

## Testing Requirements

1. Sync status display shows correctly
2. Provider selection changes work
3. GitHub Gist authentication flow works
4. Manual sync triggers work (upload/download)
5. Auto-sync settings update correctly
6. Sync section toggles work
7. Conflict resolution displays and works
8. Sync history shows entries

### Test File (src/lib/components/settings/__tests__/SyncSettings.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SyncSettings from '../SyncSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';
import { syncStore } from '$lib/stores/sync-store';

vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockResolvedValue({ success: true }),
}));

describe('SyncSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders sync status', () => {
    render(SyncSettings);

    expect(screen.getByText('Sync Status')).toBeInTheDocument();
    expect(screen.getByText('Not connected')).toBeInTheDocument();
  });

  it('opens GitHub Gist modal when selecting provider', async () => {
    render(SyncSettings);

    const select = screen.getByRole('combobox');
    await fireEvent.change(select, { target: { value: 'github-gist' } });

    expect(screen.getByText('Connect to GitHub Gist')).toBeInTheDocument();
  });

  it('shows sync sections when connected', async () => {
    // Mock connected state
    syncStore.status.update(s => ({ ...s, connected: true }));
    settingsStore.updateCategory('sync', { enabled: true, provider: 'github-gist' });

    render(SyncSettings);

    expect(screen.getByText('What to Sync')).toBeInTheDocument();
    expect(screen.getByText('General Settings')).toBeInTheDocument();
  });

  it('triggers manual sync', async () => {
    syncStore.status.update(s => ({ ...s, connected: true }));
    settingsStore.updateCategory('sync', { enabled: true, provider: 'github-gist' });

    render(SyncSettings);

    const uploadButton = screen.getByText('Upload');
    await fireEvent.click(uploadButton);

    // Verify sync was triggered
    expect(screen.getByText(/Uploading/)).toBeInTheDocument();
  });
});
```

---

## Related Specs

- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Depends on: [272-settings-store.md](272-settings-store.md)
- Previous: [278-settings-git.md](278-settings-git.md)
- Next: [280-settings-reset.md](280-settings-reset.md)
- Related: [281-settings-export.md](281-settings-export.md)
