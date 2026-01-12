# 280 - Reset/Defaults Settings

**Phase:** 13 - Settings UI
**Spec ID:** 280
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~8% of model context window

---

## Objective

Create the Reset Settings panel that allows users to reset individual setting categories or all settings to their default values with confirmation dialogs and backup options before reset.

---

## Acceptance Criteria

- [ ] `ResetSettings.svelte` component with reset options
- [ ] Reset individual setting categories
- [ ] Reset all settings to defaults
- [ ] Confirmation dialogs for destructive actions
- [ ] Optional backup before reset
- [ ] Clear cache and data options
- [ ] Application restart option after reset
- [ ] Undo last reset within session

---

## Implementation Details

### 1. Reset Settings Component (src/lib/components/settings/ResetSettings.svelte)

```svelte
<script lang="ts">
  import { settingsStore } from '$lib/stores/settings-store';
  import { invoke } from '$lib/ipc';
  import SettingsSection from './SettingsSection.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';

  interface ResetCategory {
    id: string;
    label: string;
    description: string;
    icon: string;
    settingsKey: keyof typeof import('$lib/types/settings').AllSettings;
  }

  const resetCategories: ResetCategory[] = [
    { id: 'general', label: 'General Settings', description: 'Language, updates, startup options', icon: 'settings', settingsKey: 'general' },
    { id: 'appearance', label: 'Appearance', description: 'Theme, colors, fonts, layout', icon: 'palette', settingsKey: 'appearance' },
    { id: 'editor', label: 'Editor Preferences', description: 'Tab size, word wrap, formatting', icon: 'edit-3', settingsKey: 'editor' },
    { id: 'keybindings', label: 'Keyboard Shortcuts', description: 'All custom key bindings', icon: 'keyboard', settingsKey: 'keybindings' },
    { id: 'backends', label: 'LLM Backends', description: 'Backend configurations', icon: 'server', settingsKey: 'backends' },
    { id: 'git', label: 'Git Settings', description: 'Git integration preferences', icon: 'git-branch', settingsKey: 'git' },
    { id: 'sync', label: 'Sync Settings', description: 'Cloud sync configuration', icon: 'cloud', settingsKey: 'sync' },
  ];

  interface ClearOption {
    id: string;
    label: string;
    description: string;
    icon: string;
    dangerous: boolean;
  }

  const clearOptions: ClearOption[] = [
    { id: 'cache', label: 'Clear Cache', description: 'Remove cached data and temporary files', icon: 'trash-2', dangerous: false },
    { id: 'history', label: 'Clear History', description: 'Remove mission history and logs', icon: 'clock', dangerous: true },
    { id: 'credentials', label: 'Clear Credentials', description: 'Remove all stored API keys and tokens', icon: 'key', dangerous: true },
    { id: 'all', label: 'Clear All Data', description: 'Remove all user data and start fresh', icon: 'alert-triangle', dangerous: true },
  ];

  let showConfirmModal = false;
  let confirmAction: (() => Promise<void>) | null = null;
  let confirmTitle = '';
  let confirmMessage = '';
  let confirmDangerous = false;
  let createBackup = true;
  let isProcessing = false;
  let lastResetBackup: string | null = null;
  let showRestartModal = false;

  async function confirmAndExecute(
    title: string,
    message: string,
    action: () => Promise<void>,
    dangerous: boolean = false
  ) {
    confirmTitle = title;
    confirmMessage = message;
    confirmAction = action;
    confirmDangerous = dangerous;
    showConfirmModal = true;
  }

  async function executeConfirmedAction() {
    if (!confirmAction) return;

    isProcessing = true;
    try {
      if (createBackup) {
        lastResetBackup = settingsStore.exportSettings();
      }
      await confirmAction();
    } catch (error) {
      console.error('Reset action failed:', error);
    }
    isProcessing = false;
    showConfirmModal = false;
    confirmAction = null;
  }

  async function resetCategory(category: ResetCategory) {
    await confirmAndExecute(
      `Reset ${category.label}`,
      `This will reset all ${category.label.toLowerCase()} to their default values. This action cannot be undone.`,
      async () => {
        settingsStore.resetCategory(category.settingsKey);
        await settingsStore.save();
      }
    );
  }

  async function resetAllSettings() {
    await confirmAndExecute(
      'Reset All Settings',
      'This will reset ALL settings to their default values. All your customizations will be lost. This action cannot be undone.',
      async () => {
        settingsStore.resetAll();
        await settingsStore.save();
      },
      true
    );
  }

  async function clearData(option: ClearOption) {
    await confirmAndExecute(
      option.label,
      option.description + '. This action cannot be undone.',
      async () => {
        await invoke('clear_data', { type: option.id });
        if (option.id === 'all') {
          showRestartModal = true;
        }
      },
      option.dangerous
    );
  }

  async function undoLastReset() {
    if (!lastResetBackup) return;

    try {
      await settingsStore.importSettings(lastResetBackup);
      lastResetBackup = null;
    } catch (error) {
      console.error('Failed to restore backup:', error);
    }
  }

  async function restartApp() {
    await invoke('app_restart');
  }

  function cancelConfirm() {
    showConfirmModal = false;
    confirmAction = null;
  }
</script>

<div class="reset-settings">
  <h2 class="settings-title">Reset & Clear Data</h2>
  <p class="settings-description">
    Reset settings to defaults or clear application data.
  </p>

  <!-- Undo Banner -->
  {#if lastResetBackup}
    <div class="undo-banner">
      <Icon name="rotate-ccw" size={20} />
      <div class="undo-banner__content">
        <strong>Settings were reset</strong>
        <p>You can restore your previous settings.</p>
      </div>
      <Button variant="secondary" size="small" on:click={undoLastReset}>
        Undo Reset
      </Button>
      <button class="undo-banner__dismiss" on:click={() => lastResetBackup = null}>
        <Icon name="x" size={16} />
      </button>
    </div>
  {/if}

  <!-- Reset Categories -->
  <SettingsSection title="Reset Individual Settings">
    <p class="section-note">
      Reset specific setting categories to their default values.
    </p>

    <div class="reset-categories">
      {#each resetCategories as category}
        <div class="reset-category">
          <div class="reset-category__icon">
            <Icon name={category.icon} size={20} />
          </div>
          <div class="reset-category__info">
            <span class="reset-category__label">{category.label}</span>
            <span class="reset-category__desc">{category.description}</span>
          </div>
          <Button
            variant="secondary"
            size="small"
            on:click={() => resetCategory(category)}
          >
            Reset
          </Button>
        </div>
      {/each}
    </div>
  </SettingsSection>

  <!-- Reset All -->
  <SettingsSection title="Reset All Settings">
    <div class="reset-all">
      <div class="reset-all__warning">
        <Icon name="alert-triangle" size={24} />
        <div>
          <strong>Reset all settings to defaults</strong>
          <p>This will reset every setting in the application to its default value. All your customizations including themes, keybindings, and preferences will be lost.</p>
        </div>
      </div>
      <Button variant="danger" on:click={resetAllSettings}>
        <Icon name="refresh-cw" size={16} />
        Reset All Settings
      </Button>
    </div>
  </SettingsSection>

  <!-- Clear Data -->
  <SettingsSection title="Clear Data">
    <p class="section-note">
      Clear cached data and application storage. Some options may require a restart.
    </p>

    <div class="clear-options">
      {#each clearOptions as option}
        <div class="clear-option" class:clear-option--dangerous={option.dangerous}>
          <div class="clear-option__icon">
            <Icon name={option.icon} size={20} />
          </div>
          <div class="clear-option__info">
            <span class="clear-option__label">{option.label}</span>
            <span class="clear-option__desc">{option.description}</span>
          </div>
          <Button
            variant={option.dangerous ? 'danger' : 'secondary'}
            size="small"
            on:click={() => clearData(option)}
          >
            Clear
          </Button>
        </div>
      {/each}
    </div>
  </SettingsSection>

  <!-- Storage Info -->
  <SettingsSection title="Storage Information">
    <div class="storage-info">
      <div class="storage-info__row">
        <span class="storage-info__label">Settings file:</span>
        <code class="storage-info__value">~/.config/tachikoma/settings.json</code>
      </div>
      <div class="storage-info__row">
        <span class="storage-info__label">Cache directory:</span>
        <code class="storage-info__value">~/.cache/tachikoma/</code>
      </div>
      <div class="storage-info__row">
        <span class="storage-info__label">Data directory:</span>
        <code class="storage-info__value">~/.local/share/tachikoma/</code>
      </div>
    </div>
  </SettingsSection>
</div>

<!-- Confirmation Modal -->
{#if showConfirmModal}
  <Modal
    title={confirmTitle}
    on:close={cancelConfirm}
  >
    <div class="confirm-modal">
      <div
        class="confirm-modal__icon"
        class:confirm-modal__icon--dangerous={confirmDangerous}
      >
        <Icon name={confirmDangerous ? 'alert-triangle' : 'help-circle'} size={48} />
      </div>

      <p class="confirm-modal__message">{confirmMessage}</p>

      <div class="confirm-modal__backup">
        <Toggle bind:checked={createBackup} />
        <span>Create backup before reset</span>
      </div>

      <div class="confirm-modal__actions">
        <Button variant="secondary" on:click={cancelConfirm} disabled={isProcessing}>
          Cancel
        </Button>
        <Button
          variant={confirmDangerous ? 'danger' : 'primary'}
          on:click={executeConfirmedAction}
          disabled={isProcessing}
        >
          {#if isProcessing}
            <Icon name="loader" size={16} class="spinning" />
          {/if}
          {confirmDangerous ? 'Yes, I understand' : 'Confirm'}
        </Button>
      </div>
    </div>
  </Modal>
{/if}

<!-- Restart Modal -->
{#if showRestartModal}
  <Modal title="Restart Required" on:close={() => showRestartModal = false}>
    <div class="restart-modal">
      <div class="restart-modal__icon">
        <Icon name="refresh-cw" size={48} />
      </div>
      <p>The application needs to restart to complete the reset.</p>
      <div class="restart-modal__actions">
        <Button variant="secondary" on:click={() => showRestartModal = false}>
          Later
        </Button>
        <Button variant="primary" on:click={restartApp}>
          Restart Now
        </Button>
      </div>
    </div>
  </Modal>
{/if}

<style>
  .reset-settings {
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

  /* Undo Banner */
  .undo-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: rgba(33, 150, 243, 0.1);
    border: 1px solid var(--color-primary);
    border-radius: 8px;
    margin-bottom: 24px;
  }

  .undo-banner__content {
    flex: 1;
  }

  .undo-banner__content strong {
    display: block;
    color: var(--color-text-primary);
    font-size: 14px;
  }

  .undo-banner__content p {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .undo-banner__dismiss {
    padding: 4px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 4px;
  }

  .undo-banner__dismiss:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .section-note {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 0 0 16px 0;
  }

  /* Reset Categories */
  .reset-categories {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .reset-category {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    transition: background 0.15s ease;
  }

  .reset-category:hover {
    background: var(--color-bg-hover);
  }

  .reset-category__icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-hover);
    border-radius: 8px;
    color: var(--color-primary);
  }

  .reset-category__info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .reset-category__label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .reset-category__desc {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  /* Reset All */
  .reset-all {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .reset-all__warning {
    display: flex;
    gap: 16px;
    padding: 16px;
    background: rgba(255, 152, 0, 0.1);
    border: 1px solid var(--color-warning);
    border-radius: 8px;
    color: var(--color-warning);
  }

  .reset-all__warning strong {
    display: block;
    color: var(--color-text-primary);
    margin-bottom: 4px;
  }

  .reset-all__warning p {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  /* Clear Options */
  .clear-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .clear-option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    transition: background 0.15s ease;
  }

  .clear-option:hover {
    background: var(--color-bg-hover);
  }

  .clear-option--dangerous {
    border: 1px solid rgba(244, 67, 54, 0.2);
  }

  .clear-option__icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-hover);
    border-radius: 8px;
    color: var(--color-text-muted);
  }

  .clear-option--dangerous .clear-option__icon {
    color: var(--color-error);
    background: rgba(244, 67, 54, 0.1);
  }

  .clear-option__info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .clear-option__label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .clear-option__desc {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  /* Storage Info */
  .storage-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }

  .storage-info__row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
  }

  .storage-info__label {
    color: var(--color-text-secondary);
  }

  .storage-info__value {
    padding: 4px 8px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    font-size: 12px;
    color: var(--color-text-primary);
  }

  /* Confirm Modal */
  .confirm-modal {
    text-align: center;
    padding: 16px;
  }

  .confirm-modal__icon {
    margin-bottom: 16px;
    color: var(--color-primary);
  }

  .confirm-modal__icon--dangerous {
    color: var(--color-warning);
  }

  .confirm-modal__message {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0 0 20px 0;
    line-height: 1.5;
  }

  .confirm-modal__backup {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 6px;
    margin-bottom: 20px;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .confirm-modal__actions {
    display: flex;
    justify-content: center;
    gap: 12px;
  }

  /* Restart Modal */
  .restart-modal {
    text-align: center;
    padding: 16px;
  }

  .restart-modal__icon {
    margin-bottom: 16px;
    color: var(--color-primary);
  }

  .restart-modal p {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0 0 20px 0;
  }

  .restart-modal__actions {
    display: flex;
    justify-content: center;
    gap: 12px;
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

1. Individual category reset works
2. Reset all settings functions correctly
3. Confirmation modal displays
4. Backup creation toggle works
5. Undo reset restores settings
6. Clear data options work
7. Restart modal appears when needed
8. Storage info displays correctly

### Test File (src/lib/components/settings/__tests__/ResetSettings.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ResetSettings from '../ResetSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

describe('ResetSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders all reset categories', () => {
    render(ResetSettings);

    expect(screen.getByText('General Settings')).toBeInTheDocument();
    expect(screen.getByText('Appearance')).toBeInTheDocument();
    expect(screen.getByText('Editor Preferences')).toBeInTheDocument();
    expect(screen.getByText('Keyboard Shortcuts')).toBeInTheDocument();
  });

  it('opens confirmation modal for category reset', async () => {
    render(ResetSettings);

    const resetButtons = screen.getAllByText('Reset');
    await fireEvent.click(resetButtons[0]);

    expect(screen.getByText(/Reset General Settings/)).toBeInTheDocument();
  });

  it('resets category when confirmed', async () => {
    // Modify a setting first
    settingsStore.updateSetting('general', 'language', 'fr');

    render(ResetSettings);

    const resetButtons = screen.getAllByText('Reset');
    await fireEvent.click(resetButtons[0]);

    const confirmButton = screen.getByText('Confirm');
    await fireEvent.click(confirmButton);

    const state = get(settingsStore);
    expect(state.settings.general.language).toBe('en');
  });

  it('shows undo banner after reset', async () => {
    render(ResetSettings);

    const resetButtons = screen.getAllByText('Reset');
    await fireEvent.click(resetButtons[0]);

    const confirmButton = screen.getByText('Confirm');
    await fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(screen.getByText('Settings were reset')).toBeInTheDocument();
    });
  });

  it('opens dangerous confirmation for Reset All', async () => {
    render(ResetSettings);

    const resetAllButton = screen.getByText('Reset All Settings');
    await fireEvent.click(resetAllButton);

    expect(screen.getByText("Yes, I understand")).toBeInTheDocument();
  });

  it('shows clear data options', () => {
    render(ResetSettings);

    expect(screen.getByText('Clear Cache')).toBeInTheDocument();
    expect(screen.getByText('Clear History')).toBeInTheDocument();
    expect(screen.getByText('Clear Credentials')).toBeInTheDocument();
    expect(screen.getByText('Clear All Data')).toBeInTheDocument();
  });

  it('restores settings with undo', async () => {
    settingsStore.updateSetting('general', 'language', 'de');

    render(ResetSettings);

    // Reset general settings
    const resetButtons = screen.getAllByText('Reset');
    await fireEvent.click(resetButtons[0]);
    await fireEvent.click(screen.getByText('Confirm'));

    // Undo the reset
    await waitFor(() => {
      expect(screen.getByText('Undo Reset')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Undo Reset'));

    const state = get(settingsStore);
    expect(state.settings.general.language).toBe('de');
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
- Previous: [279-settings-sync.md](279-settings-sync.md)
- Next: [281-settings-export.md](281-settings-export.md)
