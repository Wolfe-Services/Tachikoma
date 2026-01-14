# Spec 287: Notification Prefs

## Header
- **Spec ID**: 287
- **Phase**: 13 - Settings UI
- **Component**: Notification Prefs
- **Dependencies**: Spec 286 (Keyboard Config)
- **Status**: Draft

## Objective
Create a notification preferences interface that allows users to configure how and when they receive notifications, including desktop notifications, sound alerts, and in-app notifications for various application events.

## Acceptance Criteria
- [x] Configure notification channels (desktop, sound, in-app)
- [x] Set notification preferences per event type
- [x] Configure Do Not Disturb schedules
- [x] Set notification grouping preferences
- [x] Configure sound and volume settings
- [x] Enable/disable specific notification categories
- [x] Preview notifications
- [x] Set notification retention period

## Implementation

### NotificationPrefs.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import NotificationPreview from './NotificationPreview.svelte';
  import SoundPicker from './SoundPicker.svelte';
  import ScheduleEditor from './ScheduleEditor.svelte';
  import { notificationStore } from '$lib/stores/notifications';
  import type {
    NotificationSettings,
    NotificationCategory,
    NotificationChannel,
    DoNotDisturbSchedule
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: NotificationSettings;
    test: { category: string; channel: string };
  }>();

  const categories: NotificationCategory[] = [
    {
      id: 'session',
      name: 'Session Events',
      description: 'Session start, completion, and errors',
      events: ['session_started', 'session_completed', 'session_error', 'session_paused']
    },
    {
      id: 'convergence',
      name: 'Convergence',
      description: 'Convergence milestones and achievements',
      events: ['convergence_reached', 'convergence_stalled', 'milestone_reached']
    },
    {
      id: 'intervention',
      name: 'Human Intervention',
      description: 'Requests for human input or review',
      events: ['intervention_required', 'approval_needed', 'review_requested']
    },
    {
      id: 'system',
      name: 'System',
      description: 'Updates, warnings, and system status',
      events: ['update_available', 'error_occurred', 'warning', 'maintenance']
    },
    {
      id: 'cost',
      name: 'Cost Alerts',
      description: 'API usage and cost notifications',
      events: ['cost_threshold', 'quota_warning', 'rate_limit']
    }
  ];

  const channels: { id: NotificationChannel; name: string; icon: string }[] = [
    { id: 'desktop', name: 'Desktop', icon: '🖥️' },
    { id: 'sound', name: 'Sound', icon: '🔔' },
    { id: 'inapp', name: 'In-App', icon: '📱' },
    { id: 'badge', name: 'Badge', icon: '🔴' }
  ];

  const sounds = [
    { id: 'default', name: 'Default', file: 'default.mp3' },
    { id: 'chime', name: 'Chime', file: 'chime.mp3' },
    { id: 'ping', name: 'Ping', file: 'ping.mp3' },
    { id: 'bell', name: 'Bell', file: 'bell.mp3' },
    { id: 'pop', name: 'Pop', file: 'pop.mp3' },
    { id: 'none', name: 'None', file: null }
  ];

  let showScheduleEditor = writable<boolean>(false);
  let previewCategory = writable<string | null>(null);
  let showSoundPicker = writable<boolean>(false);

  const settings = derived(notificationStore, ($store) => $store.settings);

  const masterEnabled = derived(settings, ($settings) => $settings.enabled);

  const dndActive = derived(settings, ($settings) => {
    if (!$settings.doNotDisturb.enabled) return false;
    const now = new Date();
    const currentTime = now.getHours() * 60 + now.getMinutes();
    const start = parseTime($settings.doNotDisturb.startTime);
    const end = parseTime($settings.doNotDisturb.endTime);

    if (start <= end) {
      return currentTime >= start && currentTime <= end;
    } else {
      return currentTime >= start || currentTime <= end;
    }
  });

  function parseTime(time: string): number {
    const [hours, minutes] = time.split(':').map(Number);
    return hours * 60 + minutes;
  }

  function toggleMaster() {
    notificationStore.updateSetting('enabled', !$settings.enabled);
  }

  function toggleChannel(categoryId: string, channel: NotificationChannel) {
    notificationStore.toggleChannel(categoryId, channel);
  }

  function updateCategorySetting(categoryId: string, field: string, value: unknown) {
    notificationStore.updateCategory(categoryId, field, value);
  }

  function updateDoNotDisturb(field: keyof DoNotDisturbSchedule, value: unknown) {
    notificationStore.updateDoNotDisturb(field, value);
  }

  function updateSound(sound: string) {
    notificationStore.updateSetting('sound', sound);
    showSoundPicker.set(false);
  }

  function updateVolume(volume: number) {
    notificationStore.updateSetting('volume', volume);
  }

  function testNotification(categoryId: string) {
    previewCategory.set(categoryId);
    dispatch('test', { category: categoryId, channel: 'all' });
  }

  function playTestSound() {
    const sound = $settings.sound;
    if (sound !== 'none') {
      const audio = new Audio(`/sounds/${sounds.find(s => s.id === sound)?.file}`);
      audio.volume = $settings.volume / 100;
      audio.play();
    }
  }

  async function saveSettings() {
    await notificationStore.save();
    dispatch('save', $settings);
  }

  function resetToDefaults() {
    if (confirm('Reset notification preferences to defaults?')) {
      notificationStore.resetToDefaults();
    }
  }

  async function requestPermission() {
    if ('Notification' in window) {
      const permission = await Notification.requestPermission();
      notificationStore.updateSetting('desktopPermission', permission);
    }
  }

  onMount(() => {
    notificationStore.load();
    if ('Notification' in window) {
      notificationStore.updateSetting('desktopPermission', Notification.permission);
    }
  });
</script>

<div class="notification-prefs" data-testid="notification-prefs">
  <header class="config-header">
    <div class="header-title">
      <h2>Notification Preferences</h2>
      <p class="description">Configure how you receive notifications</p>
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

  <section class="master-toggle">
    <label class="toggle-row">
      <input
        type="checkbox"
        checked={$masterEnabled}
        on:change={toggleMaster}
      />
      <div class="toggle-info">
        <span class="toggle-label">Enable Notifications</span>
        <span class="toggle-desc">Receive notifications from the application</span>
      </div>
    </label>

    {#if $dndActive}
      <div class="dnd-badge">
        Do Not Disturb Active
      </div>
    {/if}
  </section>

  {#if $masterEnabled}
    <section class="permission-check" transition:slide>
      {#if $settings.desktopPermission !== 'granted'}
        <div class="permission-banner">
          <span>Desktop notifications are not enabled</span>
          <button class="btn secondary small" on:click={requestPermission}>
            Enable Desktop Notifications
          </button>
        </div>
      {/if}
    </section>

    <section class="sound-config" transition:slide>
      <h3>Sound Settings</h3>
      <div class="sound-controls">
        <div class="sound-select">
          <label>Notification Sound</label>
          <button class="sound-btn" on:click={() => showSoundPicker.set(true)}>
            {sounds.find(s => s.id === $settings.sound)?.name || 'Default'}
          </button>
        </div>

        <div class="volume-control">
          <label>Volume</label>
          <div class="volume-slider">
            <input
              type="range"
              min="0"
              max="100"
              value={$settings.volume}
              on:input={(e) => updateVolume(parseInt((e.target as HTMLInputElement).value))}
            />
            <span class="volume-value">{$settings.volume}%</span>
          </div>
          <button class="btn secondary small" on:click={playTestSound}>
            Test
          </button>
        </div>
      </div>
    </section>

    <section class="dnd-config" transition:slide>
      <h3>Do Not Disturb</h3>
      <div class="dnd-controls">
        <label class="toggle-row compact">
          <input
            type="checkbox"
            checked={$settings.doNotDisturb.enabled}
            on:change={(e) => updateDoNotDisturb('enabled', (e.target as HTMLInputElement).checked)}
          />
          <span>Enable scheduled Do Not Disturb</span>
        </label>

        {#if $settings.doNotDisturb.enabled}
          <div class="dnd-schedule" transition:slide>
            <div class="time-inputs">
              <div class="time-group">
                <label>From</label>
                <input
                  type="time"
                  value={$settings.doNotDisturb.startTime}
                  on:change={(e) => updateDoNotDisturb('startTime', (e.target as HTMLInputElement).value)}
                />
              </div>
              <div class="time-group">
                <label>To</label>
                <input
                  type="time"
                  value={$settings.doNotDisturb.endTime}
                  on:change={(e) => updateDoNotDisturb('endTime', (e.target as HTMLInputElement).value)}
                />
              </div>
            </div>

            <button
              class="btn secondary small"
              on:click={() => showScheduleEditor.set(true)}
            >
              Advanced Schedule
            </button>
          </div>
        {/if}
      </div>
    </section>

    <section class="category-config" transition:slide>
      <h3>Notification Categories</h3>

      <div class="categories-list">
        {#each categories as category (category.id)}
          {@const catSettings = $settings.categories[category.id]}
          <div class="category-item" class:disabled={!catSettings?.enabled}>
            <div class="category-header">
              <label class="category-toggle">
                <input
                  type="checkbox"
                  checked={catSettings?.enabled ?? true}
                  on:change={(e) => updateCategorySetting(category.id, 'enabled', (e.target as HTMLInputElement).checked)}
                />
                <div class="category-info">
                  <span class="category-name">{category.name}</span>
                  <span class="category-desc">{category.description}</span>
                </div>
              </label>

              <button
                class="test-btn"
                on:click={() => testNotification(category.id)}
              >
                Test
              </button>
            </div>

            {#if catSettings?.enabled}
              <div class="channel-config" transition:slide>
                <span class="channel-label">Notify via:</span>
                <div class="channel-toggles">
                  {#each channels as channel}
                    <label
                      class="channel-toggle"
                      class:active={catSettings?.channels?.includes(channel.id)}
                    >
                      <input
                        type="checkbox"
                        checked={catSettings?.channels?.includes(channel.id)}
                        on:change={() => toggleChannel(category.id, channel.id)}
                      />
                      <span class="channel-icon">{channel.icon}</span>
                      <span class="channel-name">{channel.name}</span>
                    </label>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </section>

    <section class="grouping-config" transition:slide>
      <h3>Grouping & Display</h3>

      <div class="display-options">
        <div class="option-group">
          <label>Group notifications</label>
          <select
            value={$settings.grouping}
            on:change={(e) => notificationStore.updateSetting('grouping', (e.target as HTMLSelectElement).value)}
          >
            <option value="none">No grouping</option>
            <option value="category">By category</option>
            <option value="session">By session</option>
            <option value="time">By time (5 min)</option>
          </select>
        </div>

        <div class="option-group">
          <label>Auto-dismiss after</label>
          <select
            value={$settings.autoDismiss}
            on:change={(e) => notificationStore.updateSetting('autoDismiss', parseInt((e.target as HTMLSelectElement).value))}
          >
            <option value="0">Never</option>
            <option value="3">3 seconds</option>
            <option value="5">5 seconds</option>
            <option value="10">10 seconds</option>
            <option value="30">30 seconds</option>
          </select>
        </div>

        <div class="option-group">
          <label>Keep notifications for</label>
          <select
            value={$settings.retention}
            on:change={(e) => notificationStore.updateSetting('retention', (e.target as HTMLSelectElement).value)}
          >
            <option value="1h">1 hour</option>
            <option value="24h">24 hours</option>
            <option value="7d">7 days</option>
            <option value="30d">30 days</option>
            <option value="forever">Forever</option>
          </select>
        </div>
      </div>
    </section>
  {/if}

  {#if $showSoundPicker}
    <div class="modal-overlay" transition:fade on:click={() => showSoundPicker.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <SoundPicker
          {sounds}
          selected={$settings.sound}
          volume={$settings.volume}
          on:select={(e) => updateSound(e.detail)}
          on:close={() => showSoundPicker.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showScheduleEditor}
    <div class="modal-overlay" transition:fade on:click={() => showScheduleEditor.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ScheduleEditor
          schedule={$settings.doNotDisturb}
          on:save={(e) => {
            notificationStore.updateSetting('doNotDisturb', e.detail);
            showScheduleEditor.set(false);
          }}
          on:close={() => showScheduleEditor.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $previewCategory}
    <div class="preview-toast" transition:fade>
      <NotificationPreview
        category={categories.find(c => c.id === $previewCategory)}
        on:close={() => previewCategory.set(null)}
      />
    </div>
  {/if}
</div>

<style>
  .notification-prefs {
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
    padding: 1.25rem;
    margin-bottom: 1rem;
  }

  section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .master-toggle {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
  }

  .toggle-row.compact {
    font-size: 0.875rem;
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

  .dnd-badge {
    padding: 0.375rem 0.75rem;
    background: var(--warning-alpha);
    color: var(--warning-color);
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .permission-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--info-alpha);
    border: 1px solid var(--info-color);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .sound-controls {
    display: flex;
    gap: 2rem;
    align-items: flex-end;
  }

  .sound-select label,
  .volume-control label {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.375rem;
  }

  .sound-btn {
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
  }

  .volume-slider {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .volume-slider input[type="range"] {
    width: 150px;
  }

  .volume-value {
    min-width: 40px;
    text-align: right;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .dnd-schedule {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-color);
  }

  .time-inputs {
    display: flex;
    gap: 1.5rem;
    margin-bottom: 0.75rem;
  }

  .time-group label {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.25rem;
  }

  .time-group input {
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
  }

  .categories-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .category-item {
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .category-item.disabled {
    opacity: 0.6;
  }

  .category-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .category-toggle {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    cursor: pointer;
  }

  .category-info {
    display: flex;
    flex-direction: column;
  }

  .category-name {
    font-weight: 500;
    font-size: 0.9375rem;
  }

  .category-desc {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .test-btn {
    padding: 0.375rem 0.75rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .channel-config {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-color);
  }

  .channel-label {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-right: 0.75rem;
  }

  .channel-toggles {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .channel-toggle {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.625rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8125rem;
    transition: all 0.15s ease;
  }

  .channel-toggle input {
    display: none;
  }

  .channel-toggle.active {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .display-options {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1.5rem;
  }

  .option-group label {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.375rem;
  }

  .option-group select {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
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
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .preview-toast {
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    z-index: 1001;
  }

  @media (max-width: 768px) {
    .display-options {
      grid-template-columns: 1fr;
    }

    .sound-controls {
      flex-direction: column;
      gap: 1rem;
      align-items: flex-start;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test notification preference updates
2. **Permission Tests**: Test browser notification permission flow
3. **DND Tests**: Test Do Not Disturb schedule logic
4. **Sound Tests**: Test sound playback and volume
5. **Category Tests**: Test per-category settings

## Related Specs
- Spec 286: Keyboard Config
- Spec 288: Data Cache
- Spec 295: Settings Tests
