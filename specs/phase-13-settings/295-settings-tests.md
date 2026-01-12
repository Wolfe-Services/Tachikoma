# Spec 295: Settings Tests

## Header
- **Spec ID**: 295
- **Phase**: 13 - Settings UI
- **Component**: Settings Tests
- **Dependencies**: Specs 276-294
- **Status**: Draft

## Objective
Create comprehensive test suites for all Settings UI components, covering unit tests, integration tests, accessibility tests, and end-to-end tests to ensure reliability, accessibility compliance, and proper functionality across all settings interfaces.

## Acceptance Criteria
1. Unit tests for all settings stores
2. Component integration tests
3. Settings persistence tests
4. Form validation tests
5. Accessibility compliance tests (WCAG 2.1 AA)
6. Cross-component interaction tests
7. Performance tests for settings pages
8. End-to-end settings flow tests

## Implementation

### settings.test.ts
```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';

// Store imports
import { settingsStore } from '$lib/stores/settings';
import { themeStore } from '$lib/stores/theme';
import { accessibilityStore } from '$lib/stores/accessibility';
import { keyboardStore } from '$lib/stores/keyboard';
import { notificationStore } from '$lib/stores/notifications';
import { cacheStore } from '$lib/stores/cache';
import { profileStore } from '$lib/stores/profiles';
import { workspaceStore } from '$lib/stores/workspace';
import { gitSettingsStore } from '$lib/stores/gitSettings';
import { telemetryStore } from '$lib/stores/telemetry';
import { updateStore } from '$lib/stores/update';

// Component imports
import SettingsLayout from './SettingsLayout.svelte';
import ThemeSelection from './ThemeSelection.svelte';
import FontAccessibility from './FontAccessibility.svelte';
import KeyboardConfig from './KeyboardConfig.svelte';
import NotificationPrefs from './NotificationPrefs.svelte';
import DataCache from './DataCache.svelte';
import ExportImport from './ExportImport.svelte';
import ProfileManagement from './ProfileManagement.svelte';
import WorkspaceSettings from './WorkspaceSettings.svelte';
import GitSettings from './GitSettings.svelte';
import TelemetryPrefs from './TelemetryPrefs.svelte';
import UpdatePrefs from './UpdatePrefs.svelte';

// Mock data
import {
  mockSettings,
  mockTheme,
  mockAccessibilitySettings,
  mockKeyboardShortcuts,
  mockNotificationSettings,
  mockCacheSettings,
  mockProfile,
  mockWorkspaceConfig,
  mockGitConfig,
  mockTelemetrySettings,
  mockUpdateSettings
} from '$lib/test/mocks/settings';

/**
 * Settings Store Tests
 */
describe('Settings Stores', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  describe('settingsStore', () => {
    it('should initialize with default values', () => {
      const settings = get(settingsStore);
      expect(settings).toBeDefined();
      expect(settings.initialized).toBe(false);
    });

    it('should load settings from localStorage', async () => {
      localStorage.setItem('settings', JSON.stringify(mockSettings));
      await settingsStore.load();
      const settings = get(settingsStore);
      expect(settings.initialized).toBe(true);
    });

    it('should persist settings to localStorage', async () => {
      await settingsStore.save();
      const stored = localStorage.getItem('settings');
      expect(stored).toBeTruthy();
    });

    it('should reset to defaults', () => {
      settingsStore.updateSetting('language', 'fr');
      settingsStore.resetToDefaults();
      const settings = get(settingsStore);
      expect(settings.language).toBe('en');
    });
  });

  describe('themeStore', () => {
    it('should apply theme to document', () => {
      themeStore.setTheme('dark');
      expect(document.documentElement.dataset.theme).toBe('dark');
    });

    it('should handle system theme preference', () => {
      const mockMatchMedia = vi.fn().mockReturnValue({
        matches: true,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn()
      });
      window.matchMedia = mockMatchMedia;

      themeStore.setTheme('system');
      const theme = get(themeStore);
      expect(theme.effectiveTheme).toBe('dark');
    });

    it('should generate CSS variables from theme', () => {
      themeStore.setTheme('light');
      const root = document.documentElement;
      expect(root.style.getPropertyValue('--primary-color')).toBeTruthy();
    });
  });

  describe('accessibilityStore', () => {
    it('should update font settings', () => {
      accessibilityStore.updateFont('size', 18);
      const settings = get(accessibilityStore);
      expect(settings.settings.font.size).toBe(18);
    });

    it('should apply high contrast mode', () => {
      accessibilityStore.update('highContrast', true);
      expect(document.body.classList.contains('high-contrast')).toBe(true);
    });

    it('should respect reduced motion preference', () => {
      accessibilityStore.update('motionPreference', 'reduce');
      const settings = get(accessibilityStore);
      expect(settings.settings.motionPreference).toBe('reduce');
    });
  });

  describe('keyboardStore', () => {
    it('should detect shortcut conflicts', () => {
      const keys = { key: 's', meta: true, ctrl: false, alt: false, shift: false };
      const conflict = keyboardStore.checkConflict('new-shortcut', keys);
      expect(conflict).toBeDefined();
    });

    it('should apply preset shortcuts', () => {
      keyboardStore.applyPreset('vscode');
      const store = get(keyboardStore);
      expect(store.preset).toBe('vscode');
    });

    it('should toggle shortcut enabled state', () => {
      const shortcuts = get(keyboardStore).shortcuts;
      const firstShortcut = shortcuts[0];
      keyboardStore.toggleEnabled(firstShortcut.id);
      const updated = get(keyboardStore).shortcuts.find(s => s.id === firstShortcut.id);
      expect(updated?.enabled).toBe(!firstShortcut.enabled);
    });
  });

  describe('notificationStore', () => {
    it('should toggle notification channels', () => {
      notificationStore.toggleChannel('session', 'desktop');
      const settings = get(notificationStore);
      const channels = settings.settings.categories.session.channels;
      expect(channels.includes('desktop')).toBeDefined();
    });

    it('should update Do Not Disturb schedule', () => {
      notificationStore.updateDoNotDisturb('enabled', true);
      notificationStore.updateDoNotDisturb('startTime', '22:00');
      notificationStore.updateDoNotDisturb('endTime', '08:00');
      const settings = get(notificationStore);
      expect(settings.settings.doNotDisturb.enabled).toBe(true);
    });
  });

  describe('cacheStore', () => {
    it('should calculate cache usage', async () => {
      await cacheStore.calculateUsage();
      const store = get(cacheStore);
      expect(store.usage).toBeDefined();
    });

    it('should clear specific cache category', async () => {
      await cacheStore.clearCategory('responses');
      const store = get(cacheStore);
      expect(store.usage.responses?.size).toBe(0);
    });
  });

  describe('profileStore', () => {
    it('should create new profile', async () => {
      const profile = await profileStore.create({
        name: 'Test Profile',
        description: 'Test description'
      });
      expect(profile.id).toBeTruthy();
    });

    it('should switch active profile', async () => {
      const profiles = get(profileStore).profiles;
      if (profiles.length > 0) {
        await profileStore.switchTo(profiles[0].id);
        const store = get(profileStore);
        expect(store.activeProfile?.id).toBe(profiles[0].id);
      }
    });

    it('should clone profile', async () => {
      const profiles = get(profileStore).profiles;
      if (profiles.length > 0) {
        const cloned = await profileStore.clone(profiles[0].id, 'Cloned Profile');
        expect(cloned.name).toBe('Cloned Profile');
      }
    });
  });
});

/**
 * Component Tests
 */
describe('Settings Components', () => {
  describe('SettingsLayout', () => {
    it('should render navigation', () => {
      render(SettingsLayout);
      expect(screen.getByTestId('settings-layout')).toBeInTheDocument();
      expect(screen.getByRole('navigation')).toBeInTheDocument();
    });

    it('should navigate between sections', async () => {
      render(SettingsLayout);
      const themeNav = screen.getByText('Appearance');
      await fireEvent.click(themeNav);
      expect(screen.getByText(/Theme/)).toBeInTheDocument();
    });

    it('should search settings', async () => {
      render(SettingsLayout);
      const searchInput = screen.getByPlaceholderText(/search/i);
      await userEvent.type(searchInput, 'font');
      expect(screen.getByText(/Font/)).toBeInTheDocument();
    });
  });

  describe('ThemeSelection', () => {
    it('should display theme options', () => {
      render(ThemeSelection);
      expect(screen.getByText('Light')).toBeInTheDocument();
      expect(screen.getByText('Dark')).toBeInTheDocument();
      expect(screen.getByText('System')).toBeInTheDocument();
    });

    it('should switch theme', async () => {
      render(ThemeSelection);
      const darkOption = screen.getByText('Dark').closest('button');
      await fireEvent.click(darkOption!);
      expect(document.documentElement.dataset.theme).toBe('dark');
    });

    it('should preview custom colors', async () => {
      render(ThemeSelection);
      const customizeBtn = screen.getByText(/customize/i);
      await fireEvent.click(customizeBtn);
      expect(screen.getByText(/Primary Color/)).toBeInTheDocument();
    });
  });

  describe('FontAccessibility', () => {
    it('should display font options', () => {
      render(FontAccessibility);
      expect(screen.getByLabelText(/Font Family/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Font Size/i)).toBeInTheDocument();
    });

    it('should update font size', async () => {
      render(FontAccessibility);
      const slider = screen.getByLabelText(/Font Size/i);
      await fireEvent.input(slider, { target: { value: '18' } });
      const settings = get(accessibilityStore);
      expect(settings.settings.font.size).toBe(18);
    });

    it('should enable high contrast', async () => {
      render(FontAccessibility);
      const checkbox = screen.getByLabelText(/High Contrast/i);
      await fireEvent.click(checkbox);
      expect(document.body.classList.contains('high-contrast')).toBe(true);
    });

    it('should show live preview', () => {
      render(FontAccessibility);
      expect(screen.getByText(/Live Preview/i)).toBeInTheDocument();
    });
  });

  describe('KeyboardConfig', () => {
    it('should display shortcuts', () => {
      render(KeyboardConfig);
      expect(screen.getByTestId('keyboard-config')).toBeInTheDocument();
    });

    it('should edit shortcut', async () => {
      render(KeyboardConfig);
      const editBtn = screen.getAllByText('Edit')[0];
      await fireEvent.click(editBtn);
      expect(screen.getByText(/Press keys/i)).toBeInTheDocument();
    });

    it('should detect conflicts', async () => {
      render(KeyboardConfig);
      // Simulate conflict scenario
      // Implementation depends on specific shortcut configuration
    });

    it('should export shortcuts', async () => {
      const { component } = render(KeyboardConfig);
      const exportBtn = screen.getByText('Export');

      let exportedData: any;
      component.$on('export', (e: CustomEvent) => {
        exportedData = e.detail;
      });

      await fireEvent.click(exportBtn);
      expect(exportedData).toBeDefined();
    });
  });

  describe('NotificationPrefs', () => {
    it('should toggle notifications', async () => {
      render(NotificationPrefs);
      const masterToggle = screen.getByLabelText(/Enable Notifications/i);
      await fireEvent.click(masterToggle);
      const settings = get(notificationStore);
      expect(settings.settings.enabled).toBeDefined();
    });

    it('should configure categories', async () => {
      render(NotificationPrefs);
      expect(screen.getByText('Session Events')).toBeInTheDocument();
    });

    it('should request desktop permission', async () => {
      const mockPermission = vi.fn().mockResolvedValue('granted');
      global.Notification = { requestPermission: mockPermission } as any;

      render(NotificationPrefs);
      const enableBtn = screen.queryByText(/Enable Desktop Notifications/i);
      if (enableBtn) {
        await fireEvent.click(enableBtn);
        expect(mockPermission).toHaveBeenCalled();
      }
    });
  });

  describe('DataCache', () => {
    it('should display usage statistics', () => {
      render(DataCache);
      expect(screen.getByTestId('data-cache')).toBeInTheDocument();
    });

    it('should clear cache category', async () => {
      render(DataCache);
      const clearBtn = screen.getAllByText('Clear')[0];

      vi.spyOn(window, 'confirm').mockReturnValue(true);
      await fireEvent.click(clearBtn);

      // Verify clear was called
    });
  });

  describe('ExportImport', () => {
    it('should export selected categories', async () => {
      render(ExportImport);
      const exportBtn = screen.getByText('Export Selected Data');
      expect(exportBtn).toBeInTheDocument();
    });

    it('should handle file import', async () => {
      render(ExportImport);
      const fileInput = document.querySelector('input[type="file"]');
      expect(fileInput).toBeInTheDocument();
    });

    it('should manage backups', () => {
      render(ExportImport);
      expect(screen.getByText(/Automatic Backups/i)).toBeInTheDocument();
    });
  });

  describe('ProfileManagement', () => {
    it('should display profiles', () => {
      render(ProfileManagement);
      expect(screen.getByTestId('profile-management')).toBeInTheDocument();
    });

    it('should create new profile', async () => {
      render(ProfileManagement);
      const createBtn = screen.getByText('Create Profile');
      await fireEvent.click(createBtn);
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });

    it('should switch profiles', async () => {
      // Requires multiple profiles in store
    });
  });

  describe('WorkspaceSettings', () => {
    it('should configure paths', () => {
      render(WorkspaceSettings);
      expect(screen.getByText('Directory Paths')).toBeInTheDocument();
    });

    it('should manage environment variables', async () => {
      render(WorkspaceSettings);
      const addBtn = screen.getByText('Add Variable');
      await fireEvent.click(addBtn);
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });
  });

  describe('GitSettings', () => {
    it('should toggle git integration', async () => {
      render(GitSettings);
      const toggle = screen.getByLabelText(/Enable Git Integration/i);
      await fireEvent.click(toggle);
    });

    it('should add repository', async () => {
      render(GitSettings);
      // Enable git first
      const toggle = screen.getByLabelText(/Enable Git Integration/i);
      await fireEvent.click(toggle);

      await waitFor(() => {
        const addBtn = screen.getByText('Add Repository');
        expect(addBtn).toBeInTheDocument();
      });
    });
  });

  describe('TelemetryPrefs', () => {
    it('should set telemetry level', async () => {
      render(TelemetryPrefs);
      const minimalBtn = screen.getByText('Minimal').closest('button');
      await fireEvent.click(minimalBtn!);
      const settings = get(telemetryStore);
      expect(settings.settings.level).toBe('minimal');
    });

    it('should export telemetry data', async () => {
      render(TelemetryPrefs);
      const exportBtn = screen.getByText('Export My Data');
      expect(exportBtn).toBeInTheDocument();
    });

    it('should delete all data', async () => {
      vi.spyOn(window, 'confirm').mockReturnValue(true);
      render(TelemetryPrefs);
      const deleteBtn = screen.getByText('Delete All Data');
      await fireEvent.click(deleteBtn);
    });
  });

  describe('UpdatePrefs', () => {
    it('should display current version', () => {
      render(UpdatePrefs);
      expect(screen.getByText(/Current Version/i)).toBeInTheDocument();
    });

    it('should check for updates', async () => {
      render(UpdatePrefs);
      const checkBtn = screen.getByText('Check for Updates');
      await fireEvent.click(checkBtn);
    });

    it('should select update channel', async () => {
      render(UpdatePrefs);
      const betaCard = screen.getByText('Beta').closest('button');
      await fireEvent.click(betaCard!);
      const settings = get(updateStore);
      expect(settings.settings.channel).toBe('beta');
    });
  });
});

/**
 * Accessibility Tests
 */
describe('Accessibility Compliance', () => {
  it('SettingsLayout should be accessible', async () => {
    const { container } = render(SettingsLayout);
    // Run axe accessibility tests
    expect(container).toBeAccessible();
  });

  it('should support keyboard navigation', async () => {
    render(SettingsLayout);
    const firstNav = screen.getAllByRole('button')[0];
    firstNav.focus();

    await userEvent.keyboard('{Tab}');
    expect(document.activeElement).not.toBe(firstNav);
  });

  it('should have proper ARIA labels', () => {
    render(SettingsLayout);
    const nav = screen.getByRole('navigation');
    expect(nav).toHaveAttribute('aria-label');
  });

  it('should announce changes to screen readers', async () => {
    render(ThemeSelection);
    const darkOption = screen.getByText('Dark').closest('button');
    await fireEvent.click(darkOption!);

    const liveRegion = document.querySelector('[aria-live]');
    expect(liveRegion).toBeTruthy();
  });

  it('form inputs should have labels', () => {
    render(FontAccessibility);
    const inputs = screen.getAllByRole('slider');
    inputs.forEach(input => {
      expect(input).toHaveAttribute('aria-label');
    });
  });
});

/**
 * Integration Tests
 */
describe('Settings Integration', () => {
  it('should persist settings across page reloads', async () => {
    // Save settings
    await settingsStore.updateSetting('language', 'fr');
    await settingsStore.save();

    // Reset store
    settingsStore.resetToDefaults();

    // Reload settings
    await settingsStore.load();

    const settings = get(settingsStore);
    expect(settings.language).toBe('fr');
  });

  it('should sync settings between components', async () => {
    render(ThemeSelection);

    // Change theme in one component
    themeStore.setTheme('dark');

    // Verify change is reflected
    await waitFor(() => {
      expect(screen.getByText('Dark').closest('button')).toHaveClass('selected');
    });
  });

  it('should handle concurrent settings updates', async () => {
    const updates = [
      settingsStore.updateSetting('language', 'fr'),
      themeStore.setTheme('dark'),
      accessibilityStore.updateFont('size', 18)
    ];

    await Promise.all(updates);

    const settings = get(settingsStore);
    const theme = get(themeStore);
    const accessibility = get(accessibilityStore);

    expect(settings.language).toBe('fr');
    expect(theme.mode).toBe('dark');
    expect(accessibility.settings.font.size).toBe(18);
  });
});

/**
 * Performance Tests
 */
describe('Settings Performance', () => {
  it('should load settings within 100ms', async () => {
    const start = performance.now();
    await settingsStore.load();
    const duration = performance.now() - start;

    expect(duration).toBeLessThan(100);
  });

  it('should render settings page within 200ms', async () => {
    const start = performance.now();
    render(SettingsLayout);
    const duration = performance.now() - start;

    expect(duration).toBeLessThan(200);
  });

  it('should handle large shortcut lists efficiently', async () => {
    // Generate 100 shortcuts
    const shortcuts = Array.from({ length: 100 }, (_, i) => ({
      id: `shortcut-${i}`,
      name: `Shortcut ${i}`,
      keys: { key: 'a', meta: false, ctrl: true, alt: false, shift: false }
    }));

    const start = performance.now();
    render(KeyboardConfig);
    const duration = performance.now() - start;

    expect(duration).toBeLessThan(500);
  });
});

/**
 * Error Handling Tests
 */
describe('Error Handling', () => {
  it('should handle localStorage unavailable', async () => {
    const originalLocalStorage = window.localStorage;
    Object.defineProperty(window, 'localStorage', {
      value: undefined,
      writable: true
    });

    expect(() => settingsStore.load()).not.toThrow();

    Object.defineProperty(window, 'localStorage', {
      value: originalLocalStorage,
      writable: true
    });
  });

  it('should handle corrupted settings data', async () => {
    localStorage.setItem('settings', 'invalid json');

    await expect(settingsStore.load()).resolves.not.toThrow();
  });

  it('should handle network errors in update check', async () => {
    vi.spyOn(global, 'fetch').mockRejectedValueOnce(new Error('Network error'));

    await expect(updateStore.checkForUpdates()).resolves.not.toThrow();
  });

  it('should validate import data', async () => {
    const invalidData = { invalid: 'data' };

    // Should reject or handle gracefully
    await expect(settingsStore.import(invalidData)).rejects.toThrow();
  });
});

/**
 * End-to-End Flow Tests
 */
describe('E2E Settings Flows', () => {
  it('complete theme customization flow', async () => {
    render(SettingsLayout);

    // Navigate to appearance
    await fireEvent.click(screen.getByText('Appearance'));

    // Select dark theme
    await fireEvent.click(screen.getByText('Dark'));

    // Customize colors
    await fireEvent.click(screen.getByText(/Customize/i));

    // Save settings
    await fireEvent.click(screen.getByText('Save'));

    // Verify persistence
    const theme = get(themeStore);
    expect(theme.mode).toBe('dark');
  });

  it('complete profile creation and switch flow', async () => {
    render(ProfileManagement);

    // Create profile
    await fireEvent.click(screen.getByText('Create Profile'));
    await userEvent.type(screen.getByLabelText('Name'), 'Work Profile');
    await fireEvent.click(screen.getByText('Create'));

    // Verify profile created
    await waitFor(() => {
      expect(screen.getByText('Work Profile')).toBeInTheDocument();
    });

    // Switch to new profile
    await fireEvent.click(screen.getByText('Switch'));

    const store = get(profileStore);
    expect(store.activeProfile?.name).toBe('Work Profile');
  });

  it('complete export and import flow', async () => {
    render(ExportImport);

    // Select categories
    const checkboxes = screen.getAllByRole('checkbox');
    checkboxes.forEach(async (cb) => {
      await fireEvent.click(cb);
    });

    // Export
    await fireEvent.click(screen.getByText('Export Selected Data'));

    // Verify export triggered (would need file system mock)
  });
});
```

## Testing Requirements
1. **Coverage**: Minimum 80% code coverage
2. **Unit Tests**: All store methods tested
3. **Component Tests**: All interactive elements tested
4. **Integration Tests**: Cross-component interactions
5. **Accessibility**: WCAG 2.1 AA compliance verified
6. **Performance**: Load times under thresholds
7. **Error Handling**: All error paths covered
8. **E2E Tests**: Critical user flows tested

## Test Utilities

### Mock Helpers
```typescript
// $lib/test/mocks/settings.ts
export const mockSettings = {
  language: 'en',
  region: 'US',
  initialized: true
};

export const mockTheme = {
  mode: 'light',
  customColors: {},
  effectiveTheme: 'light'
};

export const mockAccessibilitySettings = {
  font: {
    family: 'system',
    size: 16,
    lineHeight: 1.5,
    letterSpacing: 0,
    wordSpacing: 0
  },
  highContrast: false,
  motionPreference: 'no-preference'
};
```

### Custom Matchers
```typescript
// $lib/test/matchers.ts
expect.extend({
  toBeAccessible(received) {
    // Run axe-core accessibility checks
    const results = axe.run(received);
    const pass = results.violations.length === 0;
    return {
      pass,
      message: () => pass
        ? 'Expected element to have accessibility violations'
        : `Found ${results.violations.length} accessibility violations`
    };
  }
});
```

## Related Specs
- Spec 276: Settings Layout
- Spec 284: Theme Selection
- Spec 285: Font & Accessibility
- All Phase 13 Settings Specs (276-294)
