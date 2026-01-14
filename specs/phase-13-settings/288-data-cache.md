# Spec 288: Data Cache

## Header
- **Spec ID**: 288
- **Phase**: 13 - Settings UI
- **Component**: Data Cache
- **Dependencies**: Spec 287 (Notification Prefs)
- **Status**: Draft

## Objective
Create a data and cache management interface that allows users to view, manage, and configure caching behavior, local storage usage, and data retention policies for optimal application performance.

## Acceptance Criteria
- [x] Display current cache usage and statistics
- [x] Configure cache size limits
- [x] Set data retention policies per data type
- [x] Clear specific cache categories
- [x] Enable/disable caching per feature
- [x] Configure offline data sync
- [x] View and manage stored sessions
- [x] Set up automatic cache cleanup

## Implementation

### DataCache.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import StorageChart from './StorageChart.svelte';
  import CacheDetails from './CacheDetails.svelte';
  import RetentionConfig from './RetentionConfig.svelte';
  import { cacheStore } from '$lib/stores/cache';
  import type {
    CacheSettings,
    CacheCategory,
    StorageUsage,
    RetentionPolicy
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: CacheSettings;
    clear: { category: string };
    clearAll: void;
  }>();

  const cacheCategories: CacheCategory[] = [
    {
      id: 'sessions',
      name: 'Session Data',
      description: 'Cached session results and history',
      clearable: true,
      icon: '📁'
    },
    {
      id: 'responses',
      name: 'API Responses',
      description: 'Cached API responses for faster loading',
      clearable: true,
      icon: '🔄'
    },
    {
      id: 'models',
      name: 'Model Cache',
      description: 'Downloaded model configurations',
      clearable: true,
      icon: '🧠'
    },
    {
      id: 'media',
      name: 'Media & Assets',
      description: 'Images, icons, and other assets',
      clearable: true,
      icon: '🖼️'
    },
    {
      id: 'preferences',
      name: 'User Preferences',
      description: 'Settings and customizations',
      clearable: false,
      icon: '⚙️'
    },
    {
      id: 'offline',
      name: 'Offline Data',
      description: 'Data for offline access',
      clearable: true,
      icon: '📴'
    }
  ];

  let showRetentionConfig = writable<boolean>(false);
  let showCacheDetails = writable<string | null>(null);
  let isClearing = writable<boolean>(false);

  const settings = derived(cacheStore, ($store) => $store.settings);
  const usage = derived(cacheStore, ($store) => $store.usage);
  const lastCleared = derived(cacheStore, ($store) => $store.lastCleared);

  const totalUsage = derived(usage, ($usage) => {
    return Object.values($usage).reduce((sum, cat) => sum + (cat?.size || 0), 0);
  });

  const usagePercent = derived([totalUsage, settings], ([$total, $settings]) => {
    return ($total / ($settings.maxCacheSize * 1024 * 1024)) * 100;
  });

  const categoryUsage = derived(usage, ($usage) => {
    return cacheCategories.map(cat => ({
      ...cat,
      size: $usage[cat.id]?.size || 0,
      items: $usage[cat.id]?.items || 0,
      lastAccessed: $usage[cat.id]?.lastAccessed
    }));
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatDate(date: Date | null): string {
    if (!date) return 'Never';
    return new Date(date).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function updateSetting(field: keyof CacheSettings, value: unknown) {
    cacheStore.updateSetting(field, value);
  }

  function updateRetention(categoryId: string, policy: RetentionPolicy) {
    cacheStore.updateRetention(categoryId, policy);
  }

  async function clearCategory(categoryId: string) {
    const category = cacheCategories.find(c => c.id === categoryId);
    if (!category?.clearable) return;

    if (confirm(`Clear all ${category.name.toLowerCase()}? This cannot be undone.`)) {
      isClearing.set(true);
      await cacheStore.clearCategory(categoryId);
      isClearing.set(false);
      dispatch('clear', { category: categoryId });
    }
  }

  async function clearAllCache() {
    if (confirm('Clear all cached data? This cannot be undone and may affect performance temporarily.')) {
      isClearing.set(true);
      await cacheStore.clearAll();
      isClearing.set(false);
      dispatch('clearAll');
    }
  }

  async function saveSettings() {
    await cacheStore.save();
    dispatch('save', $settings);
  }

  function toggleOfflineMode() {
    updateSetting('offlineMode', !$settings.offlineMode);
  }

  function toggleAutoCleanup() {
    updateSetting('autoCleanup', !$settings.autoCleanup);
  }

  onMount(() => {
    cacheStore.load();
    cacheStore.calculateUsage();
  });
</script>

<div class="data-cache" data-testid="data-cache">
  <header class="config-header">
    <div class="header-title">
      <h2>Data & Cache</h2>
      <p class="description">Manage cached data and storage settings</p>
    </div>

    <div class="header-actions">
      <button
        class="btn secondary"
        on:click={clearAllCache}
        disabled={$isClearing}
      >
        {$isClearing ? 'Clearing...' : 'Clear All Cache'}
      </button>
      <button class="btn primary" on:click={saveSettings}>
        Save Settings
      </button>
    </div>
  </header>

  <section class="storage-overview">
    <div class="overview-content">
      <div class="usage-summary">
        <div class="usage-main">
          <span class="usage-value">{formatBytes($totalUsage)}</span>
          <span class="usage-limit">of {$settings.maxCacheSize} MB</span>
        </div>
        <div class="usage-bar">
          <div
            class="usage-fill"
            class:warning={$usagePercent > 75}
            class:danger={$usagePercent > 90}
            style="width: {Math.min($usagePercent, 100)}%"
          ></div>
        </div>
        <div class="usage-meta">
          <span>{$usagePercent.toFixed(1)}% used</span>
          {#if $lastCleared}
            <span>Last cleared: {formatDate($lastCleared)}</span>
          {/if}
        </div>
      </div>

      <div class="chart-container">
        <StorageChart
          categories={$categoryUsage}
          total={$totalUsage}
        />
      </div>
    </div>
  </section>

  <section class="cache-limits">
    <h3>Cache Limits</h3>

    <div class="limits-grid">
      <div class="form-group">
        <label for="max-cache">Maximum Cache Size</label>
        <div class="input-with-unit">
          <input
            id="max-cache"
            type="number"
            value={$settings.maxCacheSize}
            on:input={(e) => updateSetting('maxCacheSize', parseInt((e.target as HTMLInputElement).value))}
            min="50"
            max="5000"
            step="50"
          />
          <span class="unit">MB</span>
        </div>
        <span class="help-text">Maximum disk space for cached data</span>
      </div>

      <div class="form-group">
        <label for="max-age">Default Cache Duration</label>
        <select
          id="max-age"
          value={$settings.defaultMaxAge}
          on:change={(e) => updateSetting('defaultMaxAge', (e.target as HTMLSelectElement).value)}
        >
          <option value="1h">1 hour</option>
          <option value="6h">6 hours</option>
          <option value="24h">24 hours</option>
          <option value="7d">7 days</option>
          <option value="30d">30 days</option>
          <option value="never">Never expire</option>
        </select>
        <span class="help-text">How long to keep cached data by default</span>
      </div>

      <div class="form-group">
        <label for="cleanup-threshold">Auto-cleanup Threshold</label>
        <div class="input-with-unit">
          <input
            id="cleanup-threshold"
            type="number"
            value={$settings.cleanupThreshold}
            on:input={(e) => updateSetting('cleanupThreshold', parseInt((e.target as HTMLInputElement).value))}
            min="50"
            max="95"
          />
          <span class="unit">%</span>
        </div>
        <span class="help-text">Trigger cleanup when usage exceeds this</span>
      </div>
    </div>

    <div class="toggle-options">
      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.autoCleanup}
          on:change={toggleAutoCleanup}
        />
        <span class="toggle-content">
          <span class="toggle-label">Automatic Cleanup</span>
          <span class="toggle-desc">Automatically remove old cache when threshold is reached</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.offlineMode}
          on:change={toggleOfflineMode}
        />
        <span class="toggle-content">
          <span class="toggle-label">Offline Mode</span>
          <span class="toggle-desc">Cache data for offline access</span>
        </span>
      </label>

      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.compressCache}
          on:change={(e) => updateSetting('compressCache', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Compress Cache</span>
          <span class="toggle-desc">Compress cached data to save space (slower access)</span>
        </span>
      </label>
    </div>
  </section>

  <section class="cache-categories">
    <div class="section-header">
      <h3>Cache Categories</h3>
      <button
        class="btn secondary small"
        on:click={() => showRetentionConfig.set(true)}
      >
        Configure Retention
      </button>
    </div>

    <div class="categories-list">
      {#each $categoryUsage as category (category.id)}
        <div class="category-card">
          <div class="category-icon">{category.icon}</div>

          <div class="category-info">
            <div class="category-header">
              <span class="category-name">{category.name}</span>
              <span class="category-size">{formatBytes(category.size)}</span>
            </div>
            <p class="category-desc">{category.description}</p>
            <div class="category-meta">
              <span>{category.items} items</span>
              {#if category.lastAccessed}
                <span>Last accessed: {formatDate(category.lastAccessed)}</span>
              {/if}
            </div>
          </div>

          <div class="category-actions">
            <button
              class="action-btn"
              on:click={() => showCacheDetails.set(category.id)}
            >
              Details
            </button>
            {#if category.clearable}
              <button
                class="action-btn danger"
                on:click={() => clearCategory(category.id)}
                disabled={$isClearing || category.size === 0}
              >
                Clear
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section class="sync-settings">
    <h3>Sync & Backup</h3>

    <div class="sync-options">
      <label class="toggle-option">
        <input
          type="checkbox"
          checked={$settings.syncEnabled}
          on:change={(e) => updateSetting('syncEnabled', (e.target as HTMLInputElement).checked)}
        />
        <span class="toggle-content">
          <span class="toggle-label">Enable Cloud Sync</span>
          <span class="toggle-desc">Sync settings and session data across devices</span>
        </span>
      </label>

      {#if $settings.syncEnabled}
        <div class="sync-config" transition:slide>
          <div class="sync-status">
            <span class="status-label">Last synced:</span>
            <span class="status-value">{formatDate($settings.lastSync)}</span>
          </div>

          <div class="sync-actions">
            <button class="btn secondary small" on:click={() => cacheStore.syncNow()}>
              Sync Now
            </button>
          </div>

          <div class="sync-items">
            <label>
              <input
                type="checkbox"
                checked={$settings.syncItems.includes('settings')}
                on:change={() => cacheStore.toggleSyncItem('settings')}
              />
              Settings
            </label>
            <label>
              <input
                type="checkbox"
                checked={$settings.syncItems.includes('sessions')}
                on:change={() => cacheStore.toggleSyncItem('sessions')}
              />
              Sessions
            </label>
            <label>
              <input
                type="checkbox"
                checked={$settings.syncItems.includes('templates')}
                on:change={() => cacheStore.toggleSyncItem('templates')}
              />
              Templates
            </label>
          </div>
        </div>
      {/if}
    </div>
  </section>

  {#if $showRetentionConfig}
    <div class="modal-overlay" transition:fade on:click={() => showRetentionConfig.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <RetentionConfig
          categories={cacheCategories}
          retention={$settings.retention}
          on:save={(e) => {
            e.detail.forEach(({ categoryId, policy }) => updateRetention(categoryId, policy));
            showRetentionConfig.set(false);
          }}
          on:close={() => showRetentionConfig.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showCacheDetails}
    <div class="modal-overlay" transition:fade on:click={() => showCacheDetails.set(null)}>
      <div class="modal-content large" on:click|stopPropagation>
        <CacheDetails
          categoryId={$showCacheDetails}
          category={cacheCategories.find(c => c.id === $showCacheDetails)}
          usage={$usage[$showCacheDetails]}
          on:clear={() => clearCategory($showCacheDetails)}
          on:close={() => showCacheDetails.set(null)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .data-cache {
    max-width: 1000px;
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

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.25rem;
  }

  .section-header h3 {
    margin-bottom: 0;
  }

  .storage-overview {
    background: linear-gradient(135deg, var(--primary-alpha) 0%, var(--card-bg) 100%);
  }

  .overview-content {
    display: grid;
    grid-template-columns: 1fr 200px;
    gap: 2rem;
    align-items: center;
  }

  .usage-main {
    margin-bottom: 0.75rem;
  }

  .usage-value {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--primary-color);
  }

  .usage-limit {
    font-size: 1rem;
    color: var(--text-muted);
    margin-left: 0.5rem;
  }

  .usage-bar {
    height: 8px;
    background: var(--secondary-bg);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .usage-fill {
    height: 100%;
    background: var(--primary-color);
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .usage-fill.warning {
    background: var(--warning-color);
  }

  .usage-fill.danger {
    background: var(--error-color);
  }

  .usage-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .limits-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1.5rem;
    margin-bottom: 1.5rem;
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

  .input-with-unit {
    display: flex;
    align-items: center;
  }

  .input-with-unit input {
    flex: 1;
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--border-color);
    border-radius: 6px 0 0 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .input-with-unit .unit {
    padding: 0.625rem 0.75rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-left: none;
    border-radius: 0 6px 6px 0;
    font-size: 0.875rem;
    color: var(--text-secondary);
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

  .categories-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .category-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .category-icon {
    font-size: 1.5rem;
    width: 40px;
    text-align: center;
  }

  .category-info {
    flex: 1;
  }

  .category-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .category-name {
    font-weight: 500;
    font-size: 0.9375rem;
  }

  .category-size {
    font-weight: 600;
    color: var(--primary-color);
  }

  .category-desc {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin-bottom: 0.25rem;
  }

  .category-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .category-actions {
    display: flex;
    gap: 0.5rem;
  }

  .action-btn {
    padding: 0.375rem 0.75rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    border-color: var(--primary-color);
  }

  .action-btn.danger:hover {
    border-color: var(--error-color);
    color: var(--error-color);
  }

  .sync-config {
    margin-top: 1rem;
    padding: 1rem;
    background: var(--card-bg);
    border-radius: 6px;
  }

  .sync-status {
    margin-bottom: 0.75rem;
    font-size: 0.875rem;
  }

  .status-label {
    color: var(--text-muted);
    margin-right: 0.5rem;
  }

  .sync-actions {
    margin-bottom: 0.75rem;
  }

  .sync-items {
    display: flex;
    gap: 1rem;
  }

  .sync-items label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
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
    .overview-content {
      grid-template-columns: 1fr;
    }

    .limits-grid {
      grid-template-columns: 1fr;
    }

    .chart-container {
      display: none;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test cache calculation logic
2. **Storage Tests**: Test storage quota handling
3. **Cleanup Tests**: Test automatic cleanup
4. **Sync Tests**: Test cloud sync functionality
5. **Compression Tests**: Test cache compression

## Related Specs
- Spec 287: Notification Prefs
- Spec 289: Export/Import
- Spec 295: Settings Tests
