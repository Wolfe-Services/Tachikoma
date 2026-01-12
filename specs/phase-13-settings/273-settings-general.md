# 273 - General Settings Panel

**Phase:** 13 - Settings UI
**Spec ID:** 273
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~10% of model context window

---

## Objective

Create the General Settings panel component that allows users to configure core application behaviors including language, startup options, update preferences, and telemetry settings.

---

## Acceptance Criteria

- [ ] `GeneralSettings.svelte` component with all general options
- [ ] Language selector with available locales
- [ ] Startup behavior configuration
- [ ] Auto-update settings with manual check option
- [ ] Telemetry opt-in/opt-out toggle
- [ ] Application exit confirmation settings
- [ ] System tray integration options
- [ ] Form validation and error display

---

## Implementation Details

### 1. Types (src/lib/types/settings-general.ts)

```typescript
/**
 * General settings specific types.
 */

export interface LanguageOption {
  code: string;
  name: string;
  nativeName: string;
  flag: string;
}

export const AVAILABLE_LANGUAGES: LanguageOption[] = [
  { code: 'en', name: 'English', nativeName: 'English', flag: 'US' },
  { code: 'es', name: 'Spanish', nativeName: 'Espanol', flag: 'ES' },
  { code: 'fr', name: 'French', nativeName: 'Francais', flag: 'FR' },
  { code: 'de', name: 'German', nativeName: 'Deutsch', flag: 'DE' },
  { code: 'ja', name: 'Japanese', nativeName: 'nihongo', flag: 'JP' },
  { code: 'zh', name: 'Chinese', nativeName: 'zhongwen', flag: 'CN' },
  { code: 'ko', name: 'Korean', nativeName: 'hangugeo', flag: 'KR' },
  { code: 'pt', name: 'Portuguese', nativeName: 'Portugues', flag: 'BR' },
  { code: 'ru', name: 'Russian', nativeName: 'Russkij', flag: 'RU' },
];

export interface UpdateInfo {
  currentVersion: string;
  latestVersion: string | null;
  updateAvailable: boolean;
  lastChecked: number | null;
  isChecking: boolean;
  downloadProgress: number | null;
}
```

### 2. General Settings Component (src/lib/components/settings/GeneralSettings.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { settingsStore, generalSettings } from '$lib/stores/settings-store';
  import { AVAILABLE_LANGUAGES } from '$lib/types/settings-general';
  import type { UpdateInfo } from '$lib/types/settings-general';
  import { invoke } from '$lib/ipc';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  let updateInfo: UpdateInfo = {
    currentVersion: '0.0.0',
    latestVersion: null,
    updateAvailable: false,
    lastChecked: null,
    isChecking: false,
    downloadProgress: null,
  };

  async function checkForUpdates() {
    updateInfo.isChecking = true;
    try {
      const result = await invoke<{ version: string; available: boolean }>('check_updates');
      updateInfo = {
        ...updateInfo,
        latestVersion: result.version,
        updateAvailable: result.available,
        lastChecked: Date.now(),
        isChecking: false,
      };
    } catch (error) {
      console.error('Failed to check for updates:', error);
      updateInfo.isChecking = false;
    }
  }

  async function downloadUpdate() {
    updateInfo.downloadProgress = 0;
    try {
      await invoke('download_update', {
        onProgress: (progress: number) => {
          updateInfo.downloadProgress = progress;
        },
      });
      await invoke('install_update');
    } catch (error) {
      console.error('Failed to download update:', error);
      updateInfo.downloadProgress = null;
    }
  }

  function handleLanguageChange(event: CustomEvent<string>) {
    settingsStore.updateSetting('general', 'language', event.detail);
  }

  function handleToggle(key: keyof typeof $generalSettings) {
    return (event: CustomEvent<boolean>) => {
      settingsStore.updateSetting('general', key, event.detail);
    };
  }

  function formatLastChecked(timestamp: number | null): string {
    if (!timestamp) return 'Never';
    const date = new Date(timestamp);
    return date.toLocaleString();
  }

  onMount(async () => {
    try {
      const version = await invoke<string>('get_app_version');
      updateInfo.currentVersion = version;
    } catch (error) {
      console.error('Failed to get app version:', error);
    }
  });
</script>

<div class="general-settings">
  <h2 class="settings-title">General Settings</h2>
  <p class="settings-description">
    Configure core application behavior and preferences.
  </p>

  <!-- Language Section -->
  <SettingsSection title="Language & Region">
    <SettingsRow
      label="Display Language"
      description="Choose the language for the user interface"
    >
      <Select
        value={$generalSettings.language}
        options={AVAILABLE_LANGUAGES.map(lang => ({
          value: lang.code,
          label: `${lang.nativeName} (${lang.name})`,
        }))}
        on:change={handleLanguageChange}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Startup Section -->
  <SettingsSection title="Startup">
    <SettingsRow
      label="Start minimized"
      description="Launch the application minimized to the system tray"
    >
      <Toggle
        checked={$generalSettings.startMinimized}
        on:change={handleToggle('startMinimized')}
      />
    </SettingsRow>

    <SettingsRow
      label="Check for updates on startup"
      description="Automatically check for new versions when the app starts"
    >
      <Toggle
        checked={$generalSettings.checkUpdatesOnStartup}
        on:change={handleToggle('checkUpdatesOnStartup')}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Updates Section -->
  <SettingsSection title="Updates">
    <SettingsRow
      label="Automatic updates"
      description="Download and install updates automatically"
    >
      <Toggle
        checked={$generalSettings.autoUpdate}
        on:change={handleToggle('autoUpdate')}
      />
    </SettingsRow>

    <div class="update-status">
      <div class="update-status__info">
        <div class="update-status__version">
          <span class="update-status__label">Current version:</span>
          <span class="update-status__value">{updateInfo.currentVersion}</span>
        </div>
        {#if updateInfo.latestVersion}
          <div class="update-status__version">
            <span class="update-status__label">Latest version:</span>
            <span class="update-status__value">{updateInfo.latestVersion}</span>
          </div>
        {/if}
        <div class="update-status__checked">
          <span class="update-status__label">Last checked:</span>
          <span class="update-status__value">{formatLastChecked(updateInfo.lastChecked)}</span>
        </div>
      </div>

      <div class="update-status__actions">
        {#if updateInfo.updateAvailable}
          {#if updateInfo.downloadProgress !== null}
            <div class="update-status__progress">
              <div class="update-status__progress-bar">
                <div
                  class="update-status__progress-fill"
                  style="width: {updateInfo.downloadProgress}%"
                />
              </div>
              <span class="update-status__progress-text">
                Downloading... {updateInfo.downloadProgress}%
              </span>
            </div>
          {:else}
            <Button variant="primary" on:click={downloadUpdate}>
              <Icon name="download" size={16} />
              Download Update
            </Button>
          {/if}
        {:else}
          <Button
            variant="secondary"
            disabled={updateInfo.isChecking}
            on:click={checkForUpdates}
          >
            {#if updateInfo.isChecking}
              <Icon name="loader" size={16} class="spinning" />
              Checking...
            {:else}
              <Icon name="refresh-cw" size={16} />
              Check for Updates
            {/if}
          </Button>
        {/if}
      </div>
    </div>
  </SettingsSection>

  <!-- System Tray Section -->
  <SettingsSection title="System Tray">
    <SettingsRow
      label="Close to tray"
      description="Minimize to system tray instead of closing the application"
    >
      <Toggle
        checked={$generalSettings.closeToTray}
        on:change={handleToggle('closeToTray')}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Exit Behavior Section -->
  <SettingsSection title="Exit Behavior">
    <SettingsRow
      label="Confirm before exit"
      description="Show a confirmation dialog when closing the application"
    >
      <Toggle
        checked={$generalSettings.confirmBeforeExit}
        on:change={handleToggle('confirmBeforeExit')}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Privacy Section -->
  <SettingsSection title="Privacy">
    <SettingsRow
      label="Send usage statistics"
      description="Help improve Tachikoma by sending anonymous usage data"
    >
      <Toggle
        checked={$generalSettings.telemetryEnabled}
        on:change={handleToggle('telemetryEnabled')}
      />
    </SettingsRow>

    <div class="privacy-notice">
      <Icon name="shield" size={16} />
      <p>
        We only collect anonymous usage statistics to improve the application.
        No personal data or code is ever transmitted.
        <a href="/privacy-policy" target="_blank" rel="noopener">Learn more</a>
      </p>
    </div>
  </SettingsSection>
</div>

<style>
  .general-settings {
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

  .update-status {
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    margin-top: 12px;
  }

  .update-status__info {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-bottom: 16px;
  }

  .update-status__version,
  .update-status__checked {
    display: flex;
    gap: 8px;
  }

  .update-status__label {
    color: var(--color-text-secondary);
    font-size: 13px;
  }

  .update-status__value {
    color: var(--color-text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .update-status__actions {
    display: flex;
    gap: 12px;
  }

  .update-status__progress {
    flex: 1;
  }

  .update-status__progress-bar {
    height: 8px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .update-status__progress-fill {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.3s ease;
  }

  .update-status__progress-text {
    color: var(--color-text-secondary);
    font-size: 12px;
  }

  .privacy-notice {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    margin-top: 12px;
    align-items: flex-start;
  }

  .privacy-notice p {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 13px;
    line-height: 1.5;
  }

  .privacy-notice a {
    color: var(--color-primary);
    text-decoration: none;
  }

  .privacy-notice a:hover {
    text-decoration: underline;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
```

### 3. Settings Section Component (src/lib/components/settings/SettingsSection.svelte)

```svelte
<script lang="ts">
  export let title: string;
  export let description: string = '';
  export let collapsible: boolean = false;
  export let collapsed: boolean = false;

  function toggleCollapse() {
    if (collapsible) {
      collapsed = !collapsed;
    }
  }
</script>

<section class="settings-section">
  <header
    class="settings-section__header"
    class:settings-section__header--collapsible={collapsible}
    on:click={toggleCollapse}
    on:keydown={(e) => e.key === 'Enter' && toggleCollapse()}
    role={collapsible ? 'button' : undefined}
    tabindex={collapsible ? 0 : undefined}
    aria-expanded={collapsible ? !collapsed : undefined}
  >
    <div class="settings-section__title-group">
      <h3 class="settings-section__title">{title}</h3>
      {#if description}
        <p class="settings-section__description">{description}</p>
      {/if}
    </div>
    {#if collapsible}
      <svg
        class="settings-section__chevron"
        class:settings-section__chevron--collapsed={collapsed}
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
      >
        <path
          d="M4 6l4 4 4-4"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    {/if}
  </header>

  {#if !collapsed}
    <div class="settings-section__content">
      <slot />
    </div>
  {/if}
</section>

<style>
  .settings-section {
    margin-bottom: 32px;
  }

  .settings-section__header {
    margin-bottom: 16px;
  }

  .settings-section__header--collapsible {
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    border-radius: 6px;
    transition: background 0.15s ease;
  }

  .settings-section__header--collapsible:hover {
    background: var(--color-bg-hover);
    margin: 0 -8px;
    padding: 8px;
  }

  .settings-section__title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .settings-section__description {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 4px 0 0 0;
  }

  .settings-section__chevron {
    color: var(--color-text-muted);
    transition: transform 0.2s ease;
  }

  .settings-section__chevron--collapsed {
    transform: rotate(-90deg);
  }

  .settings-section__content {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
</style>
```

### 4. Settings Row Component (src/lib/components/settings/SettingsRow.svelte)

```svelte
<script lang="ts">
  export let label: string;
  export let description: string = '';
  export let error: string = '';
  export let warning: string = '';
  export let htmlFor: string = '';
</script>

<div class="settings-row" class:settings-row--error={error} class:settings-row--warning={warning}>
  <div class="settings-row__label-group">
    <label class="settings-row__label" for={htmlFor || undefined}>
      {label}
    </label>
    {#if description}
      <p class="settings-row__description">{description}</p>
    {/if}
    {#if error}
      <p class="settings-row__error">{error}</p>
    {:else if warning}
      <p class="settings-row__warning">{warning}</p>
    {/if}
  </div>
  <div class="settings-row__control">
    <slot />
  </div>
</div>

<style>
  .settings-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    padding: 12px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .settings-row:last-child {
    border-bottom: none;
  }

  .settings-row--error {
    background: rgba(244, 67, 54, 0.05);
    margin: 0 -12px;
    padding: 12px;
    border-radius: 6px;
  }

  .settings-row--warning {
    background: rgba(255, 152, 0, 0.05);
    margin: 0 -12px;
    padding: 12px;
    border-radius: 6px;
  }

  .settings-row__label-group {
    flex: 1;
    min-width: 0;
  }

  .settings-row__label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    margin: 0;
  }

  .settings-row__description {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 4px 0 0 0;
    line-height: 1.4;
  }

  .settings-row__error {
    font-size: 12px;
    color: var(--color-error);
    margin: 6px 0 0 0;
  }

  .settings-row__warning {
    font-size: 12px;
    color: var(--color-warning);
    margin: 6px 0 0 0;
  }

  .settings-row__control {
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }
</style>
```

---

## Testing Requirements

1. All general settings render correctly
2. Language selector changes language setting
3. Toggle switches update store state
4. Update check functionality works
5. Update download shows progress
6. Form validation displays errors
7. Privacy notice links are accessible

### Test File (src/lib/components/settings/__tests__/GeneralSettings.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import GeneralSettings from '../GeneralSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockImplementation((command: string) => {
    if (command === 'get_app_version') return Promise.resolve('1.0.0');
    if (command === 'check_updates') return Promise.resolve({ version: '1.1.0', available: true });
    return Promise.resolve(null);
  }),
}));

describe('GeneralSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders all settings sections', () => {
    render(GeneralSettings);

    expect(screen.getByText('Language & Region')).toBeInTheDocument();
    expect(screen.getByText('Startup')).toBeInTheDocument();
    expect(screen.getByText('Updates')).toBeInTheDocument();
    expect(screen.getByText('System Tray')).toBeInTheDocument();
    expect(screen.getByText('Privacy')).toBeInTheDocument();
  });

  it('changes language setting', async () => {
    render(GeneralSettings);

    const select = screen.getByRole('combobox');
    await fireEvent.change(select, { target: { value: 'es' } });

    const state = get(settingsStore);
    expect(state.settings.general.language).toBe('es');
  });

  it('toggles auto-update setting', async () => {
    render(GeneralSettings);

    const toggle = screen.getByRole('switch', { name: /automatic updates/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.general.autoUpdate).toBe(false);
  });

  it('checks for updates', async () => {
    render(GeneralSettings);

    const checkButton = screen.getByRole('button', { name: /check for updates/i });
    await fireEvent.click(checkButton);

    await waitFor(() => {
      expect(screen.getByText('1.1.0')).toBeInTheDocument();
    });
  });

  it('shows download button when update available', async () => {
    render(GeneralSettings);

    const checkButton = screen.getByRole('button', { name: /check for updates/i });
    await fireEvent.click(checkButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /download update/i })).toBeInTheDocument();
    });
  });

  it('toggles telemetry setting', async () => {
    render(GeneralSettings);

    const toggle = screen.getByRole('switch', { name: /send usage statistics/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.general.telemetryEnabled).toBe(true);
  });
});

function get<T>(store: { subscribe: (fn: (value: T) => void) => void }): T {
  let value: T;
  store.subscribe(v => value = v)();
  return value!;
}
```

---

## Related Specs

- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Depends on: [272-settings-store.md](272-settings-store.md)
- Next: [274-settings-appearance.md](274-settings-appearance.md)
- Related: [284-settings-validation.md](284-settings-validation.md)
