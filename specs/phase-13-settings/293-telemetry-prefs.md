# Spec 293: Telemetry Prefs

## Header
- **Spec ID**: 293
- **Phase**: 13 - Settings UI
- **Component**: Telemetry Prefs
- **Dependencies**: Spec 292 (Git Settings)
- **Status**: Draft

## Objective
Create a telemetry and analytics preferences interface that allows users to control what data is collected, view collected data, manage privacy settings, and configure usage reporting for product improvement.

## Acceptance Criteria
- [x] Configure telemetry collection levels
- [x] View what data is being collected
- [x] Export collected telemetry data
- [x] Delete telemetry data
- [x] Configure crash reporting
- [x] Set usage analytics preferences
- [x] Manage data sharing consent
- [x] View privacy policy and data handling

## Implementation

### TelemetryPrefs.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import DataViewer from './DataViewer.svelte';
  import ConsentManager from './ConsentManager.svelte';
  import { telemetryStore } from '$lib/stores/telemetry';
  import type {
    TelemetrySettings,
    TelemetryLevel,
    DataCategory,
    CollectedData
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: TelemetrySettings;
    delete: void;
    export: CollectedData;
  }>();

  const telemetryLevels: { id: TelemetryLevel; name: string; description: string; items: string[] }[] = [
    {
      id: 'none',
      name: 'Off',
      description: 'No data collection',
      items: []
    },
    {
      id: 'minimal',
      name: 'Minimal',
      description: 'Essential crash reports only',
      items: ['Crash reports', 'Fatal errors']
    },
    {
      id: 'basic',
      name: 'Basic',
      description: 'Anonymous usage statistics',
      items: ['Crash reports', 'Feature usage counts', 'Session duration', 'Error rates']
    },
    {
      id: 'full',
      name: 'Full',
      description: 'Detailed analytics for improvement',
      items: ['All basic data', 'UI interactions', 'Performance metrics', 'Feature preferences', 'Session details']
    }
  ];

  const dataCategories: DataCategory[] = [
    {
      id: 'crashes',
      name: 'Crash Reports',
      description: 'Error stack traces and crash information',
      sensitive: false
    },
    {
      id: 'usage',
      name: 'Usage Statistics',
      description: 'Feature usage and interaction patterns',
      sensitive: false
    },
    {
      id: 'performance',
      name: 'Performance Metrics',
      description: 'Load times, response times, resource usage',
      sensitive: false
    },
    {
      id: 'sessions',
      name: 'Session Data',
      description: 'Session duration and workflow patterns',
      sensitive: true
    },
    {
      id: 'preferences',
      name: 'Preference Analytics',
      description: 'Settings and configuration choices',
      sensitive: true
    }
  ];

  let showDataViewer = writable<boolean>(false);
  let showConsentManager = writable<boolean>(false);
  let viewingCategory = writable<string | null>(null);
  let isDeleting = writable<boolean>(false);
  let isExporting = writable<boolean>(false);

  const settings = derived(telemetryStore, ($store) => $store.settings);
  const collectedData = derived(telemetryStore, ($store) => $store.collectedData);
  const dataStats = derived(telemetryStore, ($store) => $store.stats);

  const currentLevel = derived(settings, ($settings) =>
    telemetryLevels.find(l => l.id === $settings.level) || telemetryLevels[0]
  );

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatDate(date: Date | null): string {
    if (!date) return 'Never';
    return new Date(date).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }

  function updateSetting(field: keyof TelemetrySettings, value: unknown) {
    telemetryStore.updateSetting(field, value);
  }

  function setTelemetryLevel(level: TelemetryLevel) {
    telemetryStore.setLevel(level);
  }

  function toggleCategory(categoryId: string) {
    telemetryStore.toggleCategory(categoryId);
  }

  async function exportData() {
    isExporting.set(true);
    try {
      const data = await telemetryStore.exportData();
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `telemetry-data-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
      dispatch('export', data);
    } finally {
      isExporting.set(false);
    }
  }

  async function deleteAllData() {
    if (!confirm('Delete all collected telemetry data? This cannot be undone.')) {
      return;
    }

    isDeleting.set(true);
    try {
      await telemetryStore.deleteAllData();
      dispatch('delete');
      alert('All telemetry data has been deleted.');
    } finally {
      isDeleting.set(false);
    }
  }

  async function saveSettings() {
    await telemetryStore.save();
    dispatch('save', $settings);
  }

  function viewCategoryData(categoryId: string) {
    viewingCategory.set(categoryId);
    showDataViewer.set(true);
  }

  function openPrivacyPolicy() {
    window.open('/privacy', '_blank');
  }

  onMount(() => {
    telemetryStore.load();
  });
</script>

<div class="telemetry-prefs" data-testid="telemetry-prefs">
  <header class="config-header">
    <div class="header-title">
      <h2>Telemetry & Privacy</h2>
      <p class="description">Control data collection and privacy settings</p>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={openPrivacyPolicy}>
        Privacy Policy
      </button>
      <button class="btn primary" on:click={saveSettings}>
        Save Settings
      </button>
    </div>
  </header>

  <section class="privacy-notice">
    <div class="notice-icon">🔒</div>
    <div class="notice-content">
      <h3>Your Privacy Matters</h3>
      <p>We collect data only to improve the product and never sell your information. You have full control over what data is collected.</p>
    </div>
  </section>

  <section class="telemetry-level">
    <h3>Data Collection Level</h3>

    <div class="level-options">
      {#each telemetryLevels as level (level.id)}
        <button
          class="level-card"
          class:selected={$settings.level === level.id}
          on:click={() => setTelemetryLevel(level.id)}
        >
          <div class="level-header">
            <span class="level-name">{level.name}</span>
            {#if $settings.level === level.id}
              <span class="check-icon">✓</span>
            {/if}
          </div>
          <p class="level-desc">{level.description}</p>
          {#if level.items.length > 0}
            <ul class="level-items">
              {#each level.items as item}
                <li>{item}</li>
              {/each}
            </ul>
          {/if}
        </button>
      {/each}
    </div>
  </section>

  {#if $settings.level !== 'none'}
    <section class="category-config" transition:slide>
      <h3>Data Categories</h3>
      <p class="section-desc">Fine-tune which types of data are collected</p>

      <div class="categories-list">
        {#each dataCategories as category (category.id)}
          {@const isEnabled = $settings.enabledCategories?.includes(category.id)}
          {@const categoryStats = $dataStats[category.id]}
          <div class="category-item" class:disabled={!isEnabled}>
            <div class="category-toggle">
              <label class="toggle-switch">
                <input
                  type="checkbox"
                  checked={isEnabled}
                  on:change={() => toggleCategory(category.id)}
                  disabled={$settings.level === 'minimal' && category.id !== 'crashes'}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>

            <div class="category-info">
              <div class="category-header">
                <span class="category-name">
                  {category.name}
                  {#if category.sensitive}
                    <span class="sensitive-badge">Sensitive</span>
                  {/if}
                </span>
              </div>
              <p class="category-desc">{category.description}</p>
              {#if categoryStats}
                <div class="category-stats">
                  <span>{categoryStats.count} records</span>
                  <span>{formatBytes(categoryStats.size)}</span>
                </div>
              {/if}
            </div>

            <button
              class="view-btn"
              on:click={() => viewCategoryData(category.id)}
              disabled={!isEnabled || !categoryStats?.count}
            >
              View Data
            </button>
          </div>
        {/each}
      </div>
    </section>

    <section class="crash-reporting" transition:slide>
      <h3>Crash Reporting</h3>

      <div class="toggle-options">
        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$settings.sendCrashReports}
            on:change={(e) => updateSetting('sendCrashReports', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Send crash reports automatically</span>
            <span class="toggle-desc">Help us fix issues by sending crash data</span>
          </span>
        </label>

        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$settings.includeSystemInfo}
            on:change={(e) => updateSetting('includeSystemInfo', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Include system information</span>
            <span class="toggle-desc">OS version, memory, and hardware info</span>
          </span>
        </label>

        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$settings.includeSessionContext}
            on:change={(e) => updateSetting('includeSessionContext', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Include session context</span>
            <span class="toggle-desc">Recent actions before the crash</span>
          </span>
        </label>
      </div>
    </section>

    <section class="data-sharing" transition:slide>
      <h3>Data Sharing</h3>

      <div class="toggle-options">
        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$settings.shareAnonymizedData}
            on:change={(e) => updateSetting('shareAnonymizedData', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Share anonymized usage data</span>
            <span class="toggle-desc">Help improve the product for all users</span>
          </span>
        </label>

        <label class="toggle-option">
          <input
            type="checkbox"
            checked={$settings.participateInResearch}
            on:change={(e) => updateSetting('participateInResearch', (e.target as HTMLInputElement).checked)}
          />
          <span class="toggle-content">
            <span class="toggle-label">Participate in product research</span>
            <span class="toggle-desc">May be contacted for surveys or feedback</span>
          </span>
        </label>
      </div>
    </section>
  {/if}

  <section class="data-management">
    <h3>Your Data</h3>

    <div class="data-summary">
      <div class="summary-item">
        <span class="summary-label">Data collected since</span>
        <span class="summary-value">{formatDate($settings.collectionStartDate)}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Total data size</span>
        <span class="summary-value">{formatBytes($dataStats.total?.size || 0)}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Last upload</span>
        <span class="summary-value">{formatDate($settings.lastUpload)}</span>
      </div>
    </div>

    <div class="data-actions">
      <button
        class="btn secondary"
        on:click={exportData}
        disabled={$isExporting}
      >
        {$isExporting ? 'Exporting...' : 'Export My Data'}
      </button>
      <button
        class="btn secondary"
        on:click={() => showDataViewer.set(true)}
      >
        View Collected Data
      </button>
      <button
        class="btn danger"
        on:click={deleteAllData}
        disabled={$isDeleting}
      >
        {$isDeleting ? 'Deleting...' : 'Delete All Data'}
      </button>
    </div>
  </section>

  <section class="consent-section">
    <div class="consent-header">
      <h3>Consent Management</h3>
      <button
        class="btn secondary small"
        on:click={() => showConsentManager.set(true)}
      >
        Manage Consents
      </button>
    </div>

    <p class="consent-desc">
      Review and update your data processing consents. You can withdraw consent at any time.
    </p>

    <div class="consent-summary">
      {#if $settings.consents}
        <div class="consent-item">
          <span class="consent-type">Essential Data</span>
          <span class="consent-status granted">Required</span>
        </div>
        <div class="consent-item">
          <span class="consent-type">Analytics</span>
          <span class="consent-status" class:granted={$settings.consents.analytics}>
            {$settings.consents.analytics ? 'Granted' : 'Denied'}
          </span>
        </div>
        <div class="consent-item">
          <span class="consent-type">Marketing</span>
          <span class="consent-status" class:granted={$settings.consents.marketing}>
            {$settings.consents.marketing ? 'Granted' : 'Denied'}
          </span>
        </div>
      {/if}
    </div>
  </section>

  {#if $showDataViewer}
    <div class="modal-overlay" transition:fade on:click={() => showDataViewer.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <DataViewer
          data={$collectedData}
          categoryFilter={$viewingCategory}
          on:close={() => {
            showDataViewer.set(false);
            viewingCategory.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showConsentManager}
    <div class="modal-overlay" transition:fade on:click={() => showConsentManager.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ConsentManager
          consents={$settings.consents}
          on:update={(e) => {
            telemetryStore.updateConsents(e.detail);
            showConsentManager.set(false);
          }}
          on:close={() => showConsentManager.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .telemetry-prefs {
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
    margin-bottom: 1rem;
  }

  .section-desc {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .privacy-notice {
    display: flex;
    gap: 1rem;
    background: linear-gradient(135deg, var(--primary-alpha) 0%, var(--card-bg) 100%);
    border-color: var(--primary-color);
  }

  .notice-icon {
    font-size: 2rem;
  }

  .notice-content h3 {
    margin-bottom: 0.375rem;
  }

  .notice-content p {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin: 0;
  }

  .level-options {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
  }

  .level-card {
    padding: 1rem;
    background: var(--secondary-bg);
    border: 2px solid var(--border-color);
    border-radius: 8px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .level-card:hover {
    border-color: var(--primary-color);
  }

  .level-card.selected {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .level-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .level-name {
    font-weight: 600;
    font-size: 0.9375rem;
  }

  .check-icon {
    color: var(--primary-color);
    font-weight: bold;
  }

  .level-desc {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
  }

  .level-items {
    margin: 0;
    padding-left: 1rem;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .level-items li {
    margin-bottom: 0.25rem;
  }

  .categories-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .category-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .category-item.disabled {
    opacity: 0.5;
  }

  .toggle-switch {
    position: relative;
    width: 40px;
    height: 22px;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border-color);
    transition: 0.3s;
    border-radius: 22px;
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 16px;
    width: 16px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  .toggle-switch input:checked + .toggle-slider {
    background-color: var(--primary-color);
  }

  .toggle-switch input:checked + .toggle-slider:before {
    transform: translateX(18px);
  }

  .category-info {
    flex: 1;
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .category-name {
    font-weight: 500;
    font-size: 0.9375rem;
  }

  .sensitive-badge {
    padding: 0.125rem 0.375rem;
    background: var(--warning-alpha);
    color: var(--warning-color);
    border-radius: 4px;
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .category-desc {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin: 0.25rem 0 0;
  }

  .category-stats {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.375rem;
  }

  .view-btn {
    padding: 0.375rem 0.75rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .view-btn:hover:not(:disabled) {
    border-color: var(--primary-color);
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

  .data-summary {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    margin-bottom: 1.25rem;
  }

  .summary-item {
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    text-align: center;
  }

  .summary-label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.25rem;
  }

  .summary-value {
    font-weight: 600;
    font-size: 1rem;
  }

  .data-actions {
    display: flex;
    gap: 0.75rem;
  }

  .consent-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .consent-header h3 {
    margin-bottom: 0;
  }

  .consent-desc {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .consent-summary {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .consent-item {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .consent-status {
    color: var(--text-muted);
  }

  .consent-status.granted {
    color: var(--success-color);
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

  .btn.danger {
    background: var(--error-alpha);
    border: 1px solid var(--error-color);
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

  .modal-content.large {
    max-width: 900px;
  }

  @media (max-width: 768px) {
    .level-options {
      grid-template-columns: 1fr 1fr;
    }

    .data-summary {
      grid-template-columns: 1fr;
    }

    .data-actions {
      flex-direction: column;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test telemetry setting updates
2. **Level Tests**: Test data collection levels
3. **Export Tests**: Test data export functionality
4. **Delete Tests**: Test data deletion
5. **Consent Tests**: Test consent management

## Related Specs
- Spec 292: Git Settings
- Spec 294: Update Prefs
- Spec 295: Settings Tests
