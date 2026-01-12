# Spec 294: Update Prefs

## Header
- **Spec ID**: 294
- **Phase**: 13 - Settings UI
- **Component**: Update Prefs
- **Dependencies**: Spec 293 (Telemetry Prefs)
- **Status**: Draft

## Objective
Create an update preferences interface that allows users to configure how the application checks for, downloads, and installs updates, including channel selection, scheduling, and rollback options.

## Acceptance Criteria
1. Configure automatic update behavior
2. Select update channel (stable, beta, canary)
3. Schedule update installation times
4. View update history
5. Manage rollback options
6. Configure bandwidth limits for downloads
7. View release notes before updating
8. Enable/disable pre-release notifications

## Implementation

### UpdatePrefs.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import ReleaseNotes from './ReleaseNotes.svelte';
  import UpdateHistory from './UpdateHistory.svelte';
  import SchedulePicker from './SchedulePicker.svelte';
  import { updateStore } from '$lib/stores/update';
  import type {
    UpdateSettings,
    UpdateChannel,
    UpdateSchedule,
    UpdateHistoryEntry,
    ReleaseInfo
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: UpdateSettings;
    checkUpdate: void;
    installUpdate: ReleaseInfo;
    rollback: UpdateHistoryEntry;
  }>();

  const updateChannels: { id: UpdateChannel; name: string; description: string; badge?: string }[] = [
    {
      id: 'stable',
      name: 'Stable',
      description: 'Recommended for most users. Thoroughly tested releases.'
    },
    {
      id: 'beta',
      name: 'Beta',
      description: 'Preview upcoming features. May have some bugs.',
      badge: 'Pre-release'
    },
    {
      id: 'canary',
      name: 'Canary',
      description: 'Cutting edge features. Daily builds, may be unstable.',
      badge: 'Experimental'
    }
  ];

  let showReleaseNotes = writable<boolean>(false);
  let showHistory = writable<boolean>(false);
  let showSchedulePicker = writable<boolean>(false);
  let isCheckingUpdate = writable<boolean>(false);
  let isDownloading = writable<boolean>(false);
  let downloadProgress = writable<number>(0);

  const settings = derived(updateStore, ($store) => $store.settings);
  const currentVersion = derived(updateStore, ($store) => $store.currentVersion);
  const latestRelease = derived(updateStore, ($store) => $store.latestRelease);
  const updateAvailable = derived(updateStore, ($store) => $store.updateAvailable);
  const history = derived(updateStore, ($store) => $store.history);
  const downloadState = derived(updateStore, ($store) => $store.downloadState);

  const hasUpdate = derived([currentVersion, latestRelease], ([$current, $latest]) => {
    if (!$latest) return false;
    return $latest.version !== $current;
  });

  function formatDate(date: Date | null): string {
    if (!date) return 'Never';
    return new Date(date).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function updateSetting(field: keyof UpdateSettings, value: unknown) {
    updateStore.updateSetting(field, value);
  }

  function setChannel(channel: UpdateChannel) {
    updateStore.setChannel(channel);
  }

  async function checkForUpdates() {
    isCheckingUpdate.set(true);
    try {
      await updateStore.checkForUpdates();
      dispatch('checkUpdate');
    } finally {
      isCheckingUpdate.set(false);
    }
  }

  async function downloadUpdate() {
    if (!$latestRelease) return;

    isDownloading.set(true);
    downloadProgress.set(0);

    try {
      await updateStore.downloadUpdate((progress) => {
        downloadProgress.set(progress);
      });
    } finally {
      isDownloading.set(false);
    }
  }

  async function installUpdate() {
    if (!$latestRelease) return;

    if (confirm(`Install version ${$latestRelease.version}? The application will restart.`)) {
      await updateStore.installUpdate();
      dispatch('installUpdate', $latestRelease);
    }
  }

  async function rollbackToVersion(entry: UpdateHistoryEntry) {
    if (confirm(`Roll back to version ${entry.version}? The application will restart.`)) {
      await updateStore.rollbackTo(entry.version);
      dispatch('rollback', entry);
    }
  }

  async function saveSettings() {
    await updateStore.save();
    dispatch('save', $settings);
  }

  function getChannelBadgeClass(channelId: UpdateChannel): string {
    switch (channelId) {
      case 'beta': return 'beta';
      case 'canary': return 'canary';
      default: return '';
    }
  }

  onMount(() => {
    updateStore.load();
  });
</script>

<div class="update-prefs" data-testid="update-prefs">
  <header class="config-header">
    <div class="header-title">
      <h2>Updates</h2>
      <p class="description">Configure application update preferences</p>
    </div>

    <div class="header-actions">
      <button class="btn primary" on:click={saveSettings}>
        Save Settings
      </button>
    </div>
  </header>

  <section class="current-version">
    <div class="version-info">
      <div class="version-main">
        <span class="version-label">Current Version</span>
        <span class="version-number">{$currentVersion}</span>
        <span class="channel-badge {$settings.channel}">{$settings.channel}</span>
      </div>
      <div class="version-meta">
        <span>Last checked: {formatDate($settings.lastCheck)}</span>
      </div>
    </div>

    <div class="version-actions">
      <button
        class="btn secondary"
        on:click={() => showHistory.set(true)}
      >
        View History
      </button>
      <button
        class="btn primary"
        on:click={checkForUpdates}
        disabled={$isCheckingUpdate}
      >
        {$isCheckingUpdate ? 'Checking...' : 'Check for Updates'}
      </button>
    </div>
  </section>

  {#if $hasUpdate && $latestRelease}
    <section class="update-available" transition:slide>
      <div class="update-header">
        <span class="update-badge">Update Available</span>
        <span class="new-version">{$latestRelease.version}</span>
      </div>

      <p class="update-summary">{$latestRelease.summary}</p>

      <div class="update-meta">
        <span>Released: {formatDate($latestRelease.releaseDate)}</span>
        <span>Size: {formatBytes($latestRelease.size)}</span>
      </div>

      <div class="update-actions">
        <button
          class="btn secondary"
          on:click={() => showReleaseNotes.set(true)}
        >
          View Release Notes
        </button>

        {#if $downloadState === 'idle'}
          <button
            class="btn primary"
            on:click={downloadUpdate}
          >
            Download Update
          </button>
        {:else if $downloadState === 'downloading'}
          <div class="download-progress">
            <div class="progress-bar">
              <div class="progress-fill" style="width: {$downloadProgress}%"></div>
            </div>
            <span class="progress-text">{$downloadProgress}%</span>
          </div>
        {:else if $downloadState === 'ready'}
          <button
            class="btn primary"
            on:click={installUpdate}
          >
            Install & Restart
          </button>
        {/if}
      </div>
    </section>
  {/if}

  <section class="channel-selection">
    <h3>Update Channel</h3>

    <div class="channel-options">
      {#each updateChannels as channel (channel.id)}
        <button
          class="channel-card"
          class:selected={$settings.channel === channel.id}
          on:click={() => setChannel(channel.id)}
        >
          <div class="channel-header">
            <span class="channel-name">{channel.name}</span>
            {#if channel.badge}
              <span class="channel-badge-small {getChannelBadgeClass(channel.id)}">
                {channel.badge}
              </span>
            {/if}
            {#if $settings.channel === channel.id}
              <span class="check-icon">✓</span>
            {/if}
          </div>
          <p class="channel-desc">{channel.description}</p>
        </button>
      {/each}
    </div>
  </section>

  <section class="auto-update">
    <h3>Automatic Updates</h3>

    <div class="toggle-options">
      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.autoCheck}
          on:change={(e) => updateSetting('autoCheck', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Check for updates automatically</span>
          <span class="toggle-desc">Periodically check for new versions</span>
        </span>
      </label>

      {#if $settings.autoCheck}
        <div class="check-frequency" transition:slide>
          <label>Check frequency:</label>
          <select
            value={$settings.checkFrequency}
            on:change={(e) => updateSetting('checkFrequency', (e.target as HTMLSelectElement).value)}
          >
            <option value="startup">On startup</option>
            <option value="daily">Daily</option>
            <option value="weekly">Weekly</option>
          </select>
        </div>
      {/if}

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.autoDownload}
          on:change={(e) => updateSetting('autoDownload', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Download updates automatically</span>
          <span class="toggle-desc">Download updates in the background</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.autoInstall}
          on:change={(e) => updateSetting('autoInstall', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Install updates automatically</span>
          <span class="toggle-desc">Install and restart when idle</span>
        </span>
      </label>
    </div>
  </section>

  <section class="schedule-section">
    <div class="section-header">
      <h3>Update Schedule</h3>
      <button
        class="btn secondary small"
        on:click={() => showSchedulePicker.set(true)}
      >
        Configure Schedule
      </button>
    </div>

    <div class="schedule-summary">
      {#if $settings.schedule.enabled}
        <p>
          Updates will be installed during:
          <strong>{$settings.schedule.startTime}</strong> -
          <strong>{$settings.schedule.endTime}</strong>
          on <strong>{$settings.schedule.days.join(', ')}</strong>
        </p>
      {:else}
        <p>No schedule configured. Updates can install at any time.</p>
      {/if}
    </div>
  </section>

  <section class="download-settings">
    <h3>Download Settings</h3>

    <div class="download-config">
      <div class="form-group">
        <label>Bandwidth limit</label>
        <div class="bandwidth-input">
          <select
            value={$settings.bandwidthLimit}
            on:change={(e) => updateSetting('bandwidthLimit', (e.target as HTMLSelectElement).value)}
          >
            <option value="unlimited">Unlimited</option>
            <option value="1">1 MB/s</option>
            <option value="5">5 MB/s</option>
            <option value="10">10 MB/s</option>
            <option value="50">50 MB/s</option>
          </select>
        </div>
        <span class="help-text">Limit download speed to prevent network congestion</span>
      </div>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.downloadOnMetered}
          on:change={(e) => updateSetting('downloadOnMetered', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Download on metered connections</span>
          <span class="toggle-desc">Allow downloads on mobile data or limited connections</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.resumeDownloads}
          on:change={(e) => updateSetting('resumeDownloads', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Resume interrupted downloads</span>
          <span class="toggle-desc">Continue downloads after network interruption</span>
        </span>
      </label>
    </div>
  </section>

  <section class="notifications-section">
    <h3>Notifications</h3>

    <div class="toggle-options">
      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.notifyOnUpdate}
          on:change={(e) => updateSetting('notifyOnUpdate', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Notify when updates are available</span>
          <span class="toggle-desc">Show a notification when a new version is ready</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.notifyPrerelease}
          on:change={(e) => updateSetting('notifyPrerelease', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Notify about pre-release versions</span>
          <span class="toggle-desc">Get notified about beta and canary releases</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.showReleaseNotes}
          on:change={(e) => updateSetting('showReleaseNotes', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Show release notes after update</span>
          <span class="toggle-desc">Display what's new after installing an update</span>
        </span>
      </label>
    </div>
  </section>

  <section class="rollback-section">
    <h3>Recovery Options</h3>

    <div class="rollback-info">
      <p>If an update causes issues, you can roll back to a previous version.</p>

      {#if $history.length > 1}
        <div class="rollback-options">
          <label>Roll back to:</label>
          <select on:change={(e) => {
            const version = (e.target as HTMLSelectElement).value;
            const entry = $history.find(h => h.version === version);
            if (entry) rollbackToVersion(entry);
          }}>
            <option value="">Select version...</option>
            {#each $history.slice(1, 5) as entry (entry.version)}
              <option value={entry.version}>
                {entry.version} ({formatDate(entry.installedAt)})
              </option>
            {/each}
          </select>
        </div>
      {:else}
        <p class="no-history">No previous versions available for rollback.</p>
      {/if}
    </div>

    <label class="toggle-option">
      <input
        type="checkbox"
        checked={$settings.keepPreviousVersions}
        on:change={(e) => updateSetting('keepPreviousVersions', (e.target as HTMLInputElement).checked)}
      />
      <span class="toggle-content">
        <span class="toggle-label">Keep previous versions for rollback</span>
        <span class="toggle-desc">Store last 3 versions for recovery (uses more disk space)</span>
      </span>
    </label>
  </section>

  {#if $showReleaseNotes && $latestRelease}
    <div class="modal-overlay" transition:fade on:click={() => showReleaseNotes.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <ReleaseNotes
          release={$latestRelease}
          on:close={() => showReleaseNotes.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showHistory}
    <div class="modal-overlay" transition:fade on:click={() => showHistory.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <UpdateHistory
          history={$history}
          on:rollback={(e) => rollbackToVersion(e.detail)}
          on:close={() => showHistory.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showSchedulePicker}
    <div class="modal-overlay" transition:fade on:click={() => showSchedulePicker.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <SchedulePicker
          schedule={$settings.schedule}
          on:save={(e) => {
            updateSetting('schedule', e.detail);
            showSchedulePicker.set(false);
          }}
          on:close={() => showSchedulePicker.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .update-prefs {
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
    margin-bottom: 1rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .section-header h3 {
    margin-bottom: 0;
  }

  .current-version {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: linear-gradient(135deg, var(--primary-alpha) 0%, var(--card-bg) 100%);
  }

  .version-main {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .version-label {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .version-number {
    font-size: 1.5rem;
    font-weight: 700;
  }

  .channel-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .channel-badge.stable {
    background: var(--success-alpha);
    color: var(--success-color);
  }

  .channel-badge.beta {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .channel-badge.canary {
    background: var(--error-alpha);
    color: var(--error-color);
  }

  .version-meta {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin-top: 0.375rem;
  }

  .version-actions {
    display: flex;
    gap: 0.75rem;
  }

  .update-available {
    border-color: var(--success-color);
    background: var(--success-alpha);
  }

  .update-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

  .update-badge {
    padding: 0.25rem 0.625rem;
    background: var(--success-color);
    color: white;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .new-version {
    font-size: 1.25rem;
    font-weight: 700;
  }

  .update-summary {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
  }

  .update-meta {
    display: flex;
    gap: 1.5rem;
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin-bottom: 1rem;
  }

  .update-actions {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .download-progress {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    max-width: 200px;
  }

  .progress-bar {
    flex: 1;
    height: 8px;
    background: var(--border-color);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--primary-color);
    transition: width 0.2s ease;
  }

  .progress-text {
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .channel-options {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .channel-card {
    padding: 1rem;
    background: var(--secondary-bg);
    border: 2px solid var(--border-color);
    border-radius: 8px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .channel-card:hover {
    border-color: var(--primary-color);
  }

  .channel-card.selected {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .channel-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .channel-name {
    font-weight: 600;
  }

  .channel-badge-small {
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-size: 0.625rem;
    font-weight: 600;
  }

  .channel-badge-small.beta {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .channel-badge-small.canary {
    background: var(--error-alpha);
    color: var(--error-color);
  }

  .check-icon {
    margin-left: auto;
    color: var(--primary-color);
    font-weight: bold;
  }

  .channel-desc {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin: 0;
  }

  .toggle-options {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
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

  .toggle-label {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .toggle-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.125rem;
  }

  .check-frequency {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--card-bg);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .check-frequency select {
    padding: 0.375rem 0.625rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .schedule-summary {
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .schedule-summary p {
    margin: 0;
  }

  .download-config {
    display: flex;
    flex-direction: column;
    gap: 1rem;
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

  .rollback-info {
    margin-bottom: 1rem;
  }

  .rollback-info p {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
  }

  .rollback-options {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .rollback-options select {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .no-history {
    color: var(--text-muted);
    font-style: italic;
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

  .modal-content.large {
    max-width: 800px;
  }

  @media (max-width: 768px) {
    .channel-options {
      grid-template-columns: 1fr;
    }

    .current-version {
      flex-direction: column;
      gap: 1rem;
      align-items: flex-start;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test update setting updates
2. **Channel Tests**: Test channel switching
3. **Download Tests**: Test download progress and resume
4. **Install Tests**: Test update installation
5. **Rollback Tests**: Test version rollback

## Related Specs
- Spec 293: Telemetry Prefs
- Spec 295: Settings Tests
