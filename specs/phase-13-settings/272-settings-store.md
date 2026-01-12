# 272 - Settings State Management

**Phase:** 13 - Settings UI
**Spec ID:** 272
**Status:** Planned
**Dependencies:** 189-store-architecture, 271-settings-layout
**Estimated Context:** ~12% of model context window

---

## Objective

Implement a comprehensive settings store that manages all application settings with support for persistence, validation, change tracking, default values, and real-time synchronization across components.

---

## Acceptance Criteria

- [ ] Central `SettingsStore` with typed settings schema
- [ ] Settings persistence to local storage and filesystem
- [ ] Change tracking with dirty state detection
- [ ] Default values and reset functionality
- [ ] Settings validation with error reporting
- [ ] Derived stores for individual setting categories
- [ ] Settings migration support for version upgrades
- [ ] Real-time sync between UI and backend

---

## Implementation Details

### 1. Settings Types (src/lib/types/settings.ts)

```typescript
/**
 * Complete settings type definitions for Tachikoma.
 */

export interface GeneralSettings {
  language: string;
  autoUpdate: boolean;
  checkUpdatesOnStartup: boolean;
  telemetryEnabled: boolean;
  startMinimized: boolean;
  closeToTray: boolean;
  confirmBeforeExit: boolean;
}

export interface AppearanceSettings {
  theme: 'light' | 'dark' | 'system';
  accentColor: string;
  fontSize: number;
  fontFamily: string;
  lineHeight: number;
  reducedMotion: boolean;
  highContrast: boolean;
  sidebarPosition: 'left' | 'right';
}

export interface EditorSettings {
  tabSize: number;
  insertSpaces: boolean;
  wordWrap: 'off' | 'on' | 'wordWrapColumn' | 'bounded';
  wordWrapColumn: number;
  lineNumbers: 'on' | 'off' | 'relative';
  minimap: boolean;
  minimapScale: number;
  bracketPairColorization: boolean;
  autoClosingBrackets: 'always' | 'languageDefined' | 'never';
  autoClosingQuotes: 'always' | 'languageDefined' | 'never';
  formatOnSave: boolean;
  formatOnPaste: boolean;
  autoSave: 'off' | 'afterDelay' | 'onFocusChange' | 'onWindowChange';
  autoSaveDelay: number;
  renderWhitespace: 'none' | 'boundary' | 'selection' | 'trailing' | 'all';
}

export interface BackendSettings {
  defaultBackend: string;
  backends: BackendConfig[];
  timeout: number;
  maxRetries: number;
  streamResponses: boolean;
}

export interface BackendConfig {
  id: string;
  name: string;
  type: 'anthropic' | 'openai' | 'ollama' | 'azure' | 'custom';
  apiKey?: string;
  baseUrl?: string;
  model: string;
  maxTokens: number;
  temperature: number;
  enabled: boolean;
}

export interface KeybindingSettings {
  preset: 'default' | 'vim' | 'emacs' | 'custom';
  customBindings: KeyBinding[];
}

export interface KeyBinding {
  id: string;
  command: string;
  key: string;
  when?: string;
}

export interface GitSettings {
  enabled: boolean;
  autoFetch: boolean;
  fetchInterval: number;
  autoPush: boolean;
  defaultBranch: string;
  signCommits: boolean;
  gpgKey?: string;
  userName?: string;
  userEmail?: string;
}

export interface SyncSettings {
  enabled: boolean;
  provider: 'none' | 'github-gist' | 'custom';
  autoSync: boolean;
  syncInterval: number;
  syncOnStartup: boolean;
  lastSyncTime?: number;
  gistId?: string;
  customEndpoint?: string;
}

export interface AllSettings {
  general: GeneralSettings;
  appearance: AppearanceSettings;
  editor: EditorSettings;
  backends: BackendSettings;
  keybindings: KeybindingSettings;
  git: GitSettings;
  sync: SyncSettings;
}

export interface SettingsMetadata {
  version: number;
  lastModified: number;
  source: 'local' | 'synced' | 'default';
}

export interface SettingsState {
  settings: AllSettings;
  metadata: SettingsMetadata;
  isDirty: boolean;
  errors: SettingsValidationError[];
  isLoading: boolean;
  isSaving: boolean;
}

export interface SettingsValidationError {
  path: string;
  message: string;
  severity: 'error' | 'warning';
}

export const SETTINGS_VERSION = 1;
```

### 2. Default Settings (src/lib/stores/settings-defaults.ts)

```typescript
import type { AllSettings } from '$lib/types/settings';

export const DEFAULT_SETTINGS: AllSettings = {
  general: {
    language: 'en',
    autoUpdate: true,
    checkUpdatesOnStartup: true,
    telemetryEnabled: false,
    startMinimized: false,
    closeToTray: false,
    confirmBeforeExit: true,
  },
  appearance: {
    theme: 'system',
    accentColor: '#2196f3',
    fontSize: 14,
    fontFamily: 'JetBrains Mono, Menlo, Monaco, monospace',
    lineHeight: 1.5,
    reducedMotion: false,
    highContrast: false,
    sidebarPosition: 'left',
  },
  editor: {
    tabSize: 2,
    insertSpaces: true,
    wordWrap: 'on',
    wordWrapColumn: 80,
    lineNumbers: 'on',
    minimap: true,
    minimapScale: 1,
    bracketPairColorization: true,
    autoClosingBrackets: 'languageDefined',
    autoClosingQuotes: 'languageDefined',
    formatOnSave: true,
    formatOnPaste: false,
    autoSave: 'afterDelay',
    autoSaveDelay: 1000,
    renderWhitespace: 'selection',
  },
  backends: {
    defaultBackend: 'anthropic',
    backends: [
      {
        id: 'anthropic-default',
        name: 'Anthropic Claude',
        type: 'anthropic',
        model: 'claude-sonnet-4-20250514',
        maxTokens: 4096,
        temperature: 0.7,
        enabled: true,
      }
    ],
    timeout: 60000,
    maxRetries: 3,
    streamResponses: true,
  },
  keybindings: {
    preset: 'default',
    customBindings: [],
  },
  git: {
    enabled: true,
    autoFetch: true,
    fetchInterval: 300000, // 5 minutes
    autoPush: false,
    defaultBranch: 'main',
    signCommits: false,
  },
  sync: {
    enabled: false,
    provider: 'none',
    autoSync: false,
    syncInterval: 3600000, // 1 hour
    syncOnStartup: false,
  },
};

export function getDefaultSetting<K extends keyof AllSettings>(
  category: K
): AllSettings[K] {
  return structuredClone(DEFAULT_SETTINGS[category]);
}

export function getDefaultValue<K extends keyof AllSettings>(
  category: K,
  key: keyof AllSettings[K]
): AllSettings[K][keyof AllSettings[K]] {
  return DEFAULT_SETTINGS[category][key];
}
```

### 3. Settings Store (src/lib/stores/settings-store.ts)

```typescript
import { writable, derived, get } from 'svelte/store';
import type {
  AllSettings,
  SettingsState,
  SettingsMetadata,
  SettingsValidationError,
  GeneralSettings,
  AppearanceSettings,
  EditorSettings,
  BackendSettings,
  KeybindingSettings,
  GitSettings,
  SyncSettings
} from '$lib/types/settings';
import { DEFAULT_SETTINGS, getDefaultSetting } from './settings-defaults';
import { validateSettings } from '$lib/utils/settings-validation';
import { invoke } from '$lib/ipc';
import { browser } from '$app/environment';

const STORAGE_KEY = 'tachikoma:settings';
const SETTINGS_VERSION = 1;

function createSettingsStore() {
  const initialState: SettingsState = {
    settings: structuredClone(DEFAULT_SETTINGS),
    metadata: {
      version: SETTINGS_VERSION,
      lastModified: Date.now(),
      source: 'default',
    },
    isDirty: false,
    errors: [],
    isLoading: true,
    isSaving: false,
  };

  const { subscribe, set, update } = writable<SettingsState>(initialState);

  let originalSettings: AllSettings = structuredClone(DEFAULT_SETTINGS);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function deepEqual(a: unknown, b: unknown): boolean {
    return JSON.stringify(a) === JSON.stringify(b);
  }

  function checkDirty(current: AllSettings): boolean {
    return !deepEqual(current, originalSettings);
  }

  async function loadFromStorage(): Promise<AllSettings | null> {
    if (!browser) return null;

    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        return parsed.settings;
      }
    } catch (error) {
      console.error('Failed to load settings from storage:', error);
    }
    return null;
  }

  async function loadFromBackend(): Promise<AllSettings | null> {
    try {
      const settings = await invoke<AllSettings>('settings_load');
      return settings;
    } catch (error) {
      console.error('Failed to load settings from backend:', error);
      return null;
    }
  }

  async function saveToStorage(settings: AllSettings): Promise<void> {
    if (!browser) return;

    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        settings,
        version: SETTINGS_VERSION,
        lastModified: Date.now(),
      }));
    } catch (error) {
      console.error('Failed to save settings to storage:', error);
    }
  }

  async function saveToBackend(settings: AllSettings): Promise<void> {
    try {
      await invoke('settings_save', { settings });
    } catch (error) {
      console.error('Failed to save settings to backend:', error);
      throw error;
    }
  }

  return {
    subscribe,

    // Initialize settings from storage/backend
    async init(): Promise<void> {
      update(s => ({ ...s, isLoading: true }));

      // Try loading from backend first, then storage
      let loaded = await loadFromBackend();
      if (!loaded) {
        loaded = await loadFromStorage();
      }

      if (loaded) {
        // Migrate if needed
        const migrated = migrateSettings(loaded);
        const errors = validateSettings(migrated);

        originalSettings = structuredClone(migrated);

        update(s => ({
          ...s,
          settings: migrated,
          errors,
          isLoading: false,
          isDirty: false,
          metadata: {
            ...s.metadata,
            source: loaded ? 'local' : 'default',
            lastModified: Date.now(),
          },
        }));
      } else {
        update(s => ({
          ...s,
          isLoading: false,
        }));
      }
    },

    // Update a single category
    updateCategory<K extends keyof AllSettings>(
      category: K,
      updates: Partial<AllSettings[K]>
    ): void {
      update(s => {
        const newSettings = {
          ...s.settings,
          [category]: {
            ...s.settings[category],
            ...updates,
          },
        };
        const errors = validateSettings(newSettings);

        return {
          ...s,
          settings: newSettings,
          errors,
          isDirty: checkDirty(newSettings),
          metadata: {
            ...s.metadata,
            lastModified: Date.now(),
          },
        };
      });

      // Debounced auto-save
      this.debouncedSave();
    },

    // Update a single setting
    updateSetting<K extends keyof AllSettings, P extends keyof AllSettings[K]>(
      category: K,
      key: P,
      value: AllSettings[K][P]
    ): void {
      update(s => {
        const newSettings = {
          ...s.settings,
          [category]: {
            ...s.settings[category],
            [key]: value,
          },
        };
        const errors = validateSettings(newSettings);

        return {
          ...s,
          settings: newSettings,
          errors,
          isDirty: checkDirty(newSettings),
          metadata: {
            ...s.metadata,
            lastModified: Date.now(),
          },
        };
      });

      this.debouncedSave();
    },

    // Replace all settings
    setSettings(settings: AllSettings): void {
      const errors = validateSettings(settings);
      update(s => ({
        ...s,
        settings,
        errors,
        isDirty: checkDirty(settings),
        metadata: {
          ...s.metadata,
          lastModified: Date.now(),
        },
      }));
    },

    // Save settings
    async save(): Promise<void> {
      const state = get({ subscribe });
      if (!state.isDirty || state.errors.some(e => e.severity === 'error')) {
        return;
      }

      update(s => ({ ...s, isSaving: true }));

      try {
        await Promise.all([
          saveToStorage(state.settings),
          saveToBackend(state.settings),
        ]);

        originalSettings = structuredClone(state.settings);

        update(s => ({
          ...s,
          isDirty: false,
          isSaving: false,
          metadata: {
            ...s.metadata,
            source: 'local',
            lastModified: Date.now(),
          },
        }));
      } catch (error) {
        update(s => ({ ...s, isSaving: false }));
        throw error;
      }
    },

    // Debounced save (for auto-save)
    debouncedSave(delay: number = 2000): void {
      if (saveTimeout) {
        clearTimeout(saveTimeout);
      }
      saveTimeout = setTimeout(() => {
        this.save().catch(console.error);
      }, delay);
    },

    // Reset a category to defaults
    resetCategory<K extends keyof AllSettings>(category: K): void {
      const defaultValue = getDefaultSetting(category);
      this.updateCategory(category, defaultValue as Partial<AllSettings[K]>);
    },

    // Reset all settings to defaults
    resetAll(): void {
      const defaults = structuredClone(DEFAULT_SETTINGS);
      this.setSettings(defaults);
    },

    // Discard unsaved changes
    discardChanges(): void {
      update(s => ({
        ...s,
        settings: structuredClone(originalSettings),
        isDirty: false,
        errors: validateSettings(originalSettings),
      }));
    },

    // Export settings
    exportSettings(): string {
      const state = get({ subscribe });
      return JSON.stringify({
        settings: state.settings,
        version: SETTINGS_VERSION,
        exportedAt: new Date().toISOString(),
      }, null, 2);
    },

    // Import settings
    async importSettings(json: string): Promise<void> {
      const parsed = JSON.parse(json);
      const migrated = migrateSettings(parsed.settings);
      const errors = validateSettings(migrated);

      if (errors.some(e => e.severity === 'error')) {
        throw new Error('Invalid settings file');
      }

      this.setSettings(migrated);
      await this.save();
    },
  };
}

function migrateSettings(settings: unknown): AllSettings {
  // Handle version migrations here
  const s = settings as AllSettings;

  // Merge with defaults to ensure all fields exist
  return {
    general: { ...DEFAULT_SETTINGS.general, ...s.general },
    appearance: { ...DEFAULT_SETTINGS.appearance, ...s.appearance },
    editor: { ...DEFAULT_SETTINGS.editor, ...s.editor },
    backends: { ...DEFAULT_SETTINGS.backends, ...s.backends },
    keybindings: { ...DEFAULT_SETTINGS.keybindings, ...s.keybindings },
    git: { ...DEFAULT_SETTINGS.git, ...s.git },
    sync: { ...DEFAULT_SETTINGS.sync, ...s.sync },
  };
}

export const settingsStore = createSettingsStore();

// Derived stores for individual categories
export const generalSettings = derived(
  settingsStore,
  ($state) => $state.settings.general
);

export const appearanceSettings = derived(
  settingsStore,
  ($state) => $state.settings.appearance
);

export const editorSettings = derived(
  settingsStore,
  ($state) => $state.settings.editor
);

export const backendSettings = derived(
  settingsStore,
  ($state) => $state.settings.backends
);

export const keybindingSettings = derived(
  settingsStore,
  ($state) => $state.settings.keybindings
);

export const gitSettings = derived(
  settingsStore,
  ($state) => $state.settings.git
);

export const syncSettings = derived(
  settingsStore,
  ($state) => $state.settings.sync
);

export const settingsErrors = derived(
  settingsStore,
  ($state) => $state.errors
);

export const settingsIsDirty = derived(
  settingsStore,
  ($state) => $state.isDirty
);

export const settingsIsLoading = derived(
  settingsStore,
  ($state) => $state.isLoading
);
```

### 4. Settings Validation (src/lib/utils/settings-validation.ts)

```typescript
import type { AllSettings, SettingsValidationError } from '$lib/types/settings';

type Validator = (settings: AllSettings) => SettingsValidationError[];

const validators: Validator[] = [
  validateGeneral,
  validateAppearance,
  validateEditor,
  validateBackends,
  validateGit,
  validateSync,
];

export function validateSettings(settings: AllSettings): SettingsValidationError[] {
  const errors: SettingsValidationError[] = [];

  for (const validator of validators) {
    errors.push(...validator(settings));
  }

  return errors;
}

function validateGeneral(settings: AllSettings): SettingsValidationError[] {
  const errors: SettingsValidationError[] = [];
  const { general } = settings;

  if (!general.language || general.language.length < 2) {
    errors.push({
      path: 'general.language',
      message: 'Language code must be at least 2 characters',
      severity: 'error',
    });
  }

  return errors;
}

function validateAppearance(settings: AllSettings): SettingsValidationError[] {
  const errors: SettingsValidationError[] = [];
  const { appearance } = settings;

  if (appearance.fontSize < 8 || appearance.fontSize > 32) {
    errors.push({
      path: 'appearance.fontSize',
      message: 'Font size must be between 8 and 32',
      severity: 'error',
    });
  }

  if (!/^#[0-9A-Fa-f]{6}$/.test(appearance.accentColor)) {
    errors.push({
      path: 'appearance.accentColor',
      message: 'Invalid color format (use #RRGGBB)',
      severity: 'error',
    });
  }

  if (appearance.lineHeight < 1 || appearance.lineHeight > 3) {
    errors.push({
      path: 'appearance.lineHeight',
      message: 'Line height must be between 1 and 3',
      severity: 'warning',
    });
  }

  return errors;
}

function validateEditor(settings: AllSettings): SettingsValidationError[] {
  const errors: SettingsValidationError[] = [];
  const { editor } = settings;

  if (editor.tabSize < 1 || editor.tabSize > 8) {
    errors.push({
      path: 'editor.tabSize',
      message: 'Tab size must be between 1 and 8',
      severity: 'error',
    });
  }

  if (editor.wordWrapColumn < 40 || editor.wordWrapColumn > 200) {
    errors.push({
      path: 'editor.wordWrapColumn',
      message: 'Word wrap column must be between 40 and 200',
      severity: 'warning',
    });
  }

  if (editor.autoSaveDelay < 100 || editor.autoSaveDelay > 60000) {
    errors.push({
      path: 'editor.autoSaveDelay',
      message: 'Auto-save delay must be between 100ms and 60s',
      severity: 'error',
    });
  }

  return errors;
}

function validateBackends(settings: AllSettings): SettingsValidationError[] {
  const errors: SettingsValidationError[] = [];
  const { backends } = settings;

  if (backends.backends.length === 0) {
    errors.push({
      path: 'backends.backends',
      message: 'At least one backend must be configured',
      severity: 'warning',
    });
  }

  const enabledBackends = backends.backends.filter(b => b.enabled);
  if (enabledBackends.length === 0 && backends.backends.length > 0) {
    errors.push({
      path: 'backends.backends',
      message: 'At least one backend must be enabled',
      severity: 'warning',
    });
  }

  const defaultExists = backends.backends.some(b => b.id === backends.defaultBackend);
  if (backends.defaultBackend && !defaultExists) {
    errors.push({
      path: 'backends.defaultBackend',
      message: 'Default backend does not exist',
      severity: 'error',
    });
  }

  backends.backends.forEach((backend, index) => {
    if (!backend.model) {
      errors.push({
        path: `backends.backends[${index}].model`,
        message: 'Backend model is required',
        severity: 'error',
      });
    }

    if (backend.maxTokens < 1 || backend.maxTokens > 100000) {
      errors.push({
        path: `backends.backends[${index}].maxTokens`,
        message: 'Max tokens must be between 1 and 100000',
        severity: 'error',
      });
    }

    if (backend.temperature < 0 || backend.temperature > 2) {
      errors.push({
        path: `backends.backends[${index}].temperature`,
        message: 'Temperature must be between 0 and 2',
        severity: 'error',
      });
    }
  });

  return errors;
}

function validateGit(settings: AllSettings): SettingsValidationError[] {
  const errors: SettingsValidationError[] = [];
  const { git } = settings;

  if (git.enabled && git.signCommits && !git.gpgKey) {
    errors.push({
      path: 'git.gpgKey',
      message: 'GPG key is required when commit signing is enabled',
      severity: 'warning',
    });
  }

  if (git.fetchInterval < 60000) {
    errors.push({
      path: 'git.fetchInterval',
      message: 'Fetch interval should be at least 1 minute',
      severity: 'warning',
    });
  }

  return errors;
}

function validateSync(settings: AllSettings): SettingsValidationError[] {
  const errors: SettingsValidationError[] = [];
  const { sync } = settings;

  if (sync.enabled && sync.provider === 'github-gist' && !sync.gistId) {
    errors.push({
      path: 'sync.gistId',
      message: 'Gist ID is required for GitHub Gist sync',
      severity: 'warning',
    });
  }

  if (sync.enabled && sync.provider === 'custom' && !sync.customEndpoint) {
    errors.push({
      path: 'sync.customEndpoint',
      message: 'Custom endpoint is required',
      severity: 'error',
    });
  }

  return errors;
}
```

---

## Testing Requirements

1. Settings load correctly from storage and backend
2. Individual settings update correctly
3. Category updates work as expected
4. Validation catches errors and warnings
5. Dirty state tracked correctly
6. Settings save to storage and backend
7. Reset functionality works for categories and all
8. Import/export functions correctly
9. Migration handles version changes

### Test File (src/lib/stores/__tests__/settings-store.test.ts)

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { settingsStore, generalSettings, appearanceSettings } from '../settings-store';
import { DEFAULT_SETTINGS } from '../settings-defaults';

vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

describe('settingsStore', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('initializes with default settings', () => {
    const state = get(settingsStore);
    expect(state.settings).toEqual(DEFAULT_SETTINGS);
    expect(state.isDirty).toBe(false);
  });

  it('updates individual setting', () => {
    settingsStore.updateSetting('general', 'language', 'es');

    const general = get(generalSettings);
    expect(general.language).toBe('es');
  });

  it('updates category settings', () => {
    settingsStore.updateCategory('appearance', {
      theme: 'dark',
      fontSize: 16,
    });

    const appearance = get(appearanceSettings);
    expect(appearance.theme).toBe('dark');
    expect(appearance.fontSize).toBe(16);
  });

  it('tracks dirty state', () => {
    expect(get(settingsStore).isDirty).toBe(false);

    settingsStore.updateSetting('general', 'language', 'fr');
    expect(get(settingsStore).isDirty).toBe(true);
  });

  it('validates settings', () => {
    settingsStore.updateSetting('appearance', 'fontSize', 50);

    const state = get(settingsStore);
    expect(state.errors.some(e => e.path === 'appearance.fontSize')).toBe(true);
  });

  it('resets category to defaults', () => {
    settingsStore.updateSetting('editor', 'tabSize', 8);
    settingsStore.resetCategory('editor');

    const state = get(settingsStore);
    expect(state.settings.editor.tabSize).toBe(DEFAULT_SETTINGS.editor.tabSize);
  });

  it('resets all settings to defaults', () => {
    settingsStore.updateSetting('general', 'language', 'de');
    settingsStore.updateSetting('appearance', 'theme', 'dark');
    settingsStore.resetAll();

    const state = get(settingsStore);
    expect(state.settings).toEqual(DEFAULT_SETTINGS);
  });

  it('exports settings as JSON', () => {
    const exported = settingsStore.exportSettings();
    const parsed = JSON.parse(exported);

    expect(parsed.settings).toEqual(DEFAULT_SETTINGS);
    expect(parsed.version).toBeDefined();
    expect(parsed.exportedAt).toBeDefined();
  });

  it('imports settings from JSON', async () => {
    const importData = JSON.stringify({
      settings: {
        ...DEFAULT_SETTINGS,
        general: { ...DEFAULT_SETTINGS.general, language: 'jp' },
      },
      version: 1,
    });

    await settingsStore.importSettings(importData);

    const general = get(generalSettings);
    expect(general.language).toBe('jp');
  });

  it('discards unsaved changes', () => {
    settingsStore.updateSetting('general', 'language', 'ru');
    expect(get(settingsStore).isDirty).toBe(true);

    settingsStore.discardChanges();

    const state = get(settingsStore);
    expect(state.isDirty).toBe(false);
    expect(state.settings.general.language).toBe('en');
  });
});
```

---

## Related Specs

- Depends on: [189-store-architecture.md](../phase-09-ui-foundation/189-store-architecture.md)
- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Next: [273-settings-general.md](273-settings-general.md)
- Used by: All Settings panel specs (273-285)
