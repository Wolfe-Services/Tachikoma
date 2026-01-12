# 274 - Appearance/Theme Settings

**Phase:** 13 - Settings UI
**Spec ID:** 274
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~10% of model context window

---

## Objective

Create the Appearance Settings panel that allows users to customize the visual appearance of the application including theme selection, accent colors, font settings, and accessibility options.

---

## Acceptance Criteria

- [ ] `AppearanceSettings.svelte` component with theme options
- [ ] Theme selector with light/dark/system modes
- [ ] Accent color picker with presets and custom colors
- [ ] Font family and size configuration
- [ ] Line height adjustment
- [ ] Accessibility options (reduced motion, high contrast)
- [ ] Live preview of appearance changes
- [ ] Sidebar position toggle

---

## Implementation Details

### 1. Theme Types (src/lib/types/theme.ts)

```typescript
/**
 * Theme and appearance type definitions.
 */

export type ThemeMode = 'light' | 'dark' | 'system';

export interface ThemeColors {
  primary: string;
  secondary: string;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  border: string;
  error: string;
  warning: string;
  success: string;
  info: string;
}

export interface AccentColorPreset {
  id: string;
  name: string;
  color: string;
}

export const ACCENT_COLOR_PRESETS: AccentColorPreset[] = [
  { id: 'blue', name: 'Blue', color: '#2196f3' },
  { id: 'purple', name: 'Purple', color: '#9c27b0' },
  { id: 'pink', name: 'Pink', color: '#e91e63' },
  { id: 'red', name: 'Red', color: '#f44336' },
  { id: 'orange', name: 'Orange', color: '#ff9800' },
  { id: 'amber', name: 'Amber', color: '#ffc107' },
  { id: 'green', name: 'Green', color: '#4caf50' },
  { id: 'teal', name: 'Teal', color: '#009688' },
  { id: 'cyan', name: 'Cyan', color: '#00bcd4' },
  { id: 'indigo', name: 'Indigo', color: '#3f51b5' },
];

export interface FontOption {
  name: string;
  family: string;
  category: 'monospace' | 'sans-serif' | 'serif';
}

export const FONT_OPTIONS: FontOption[] = [
  { name: 'JetBrains Mono', family: 'JetBrains Mono, monospace', category: 'monospace' },
  { name: 'Fira Code', family: 'Fira Code, monospace', category: 'monospace' },
  { name: 'Source Code Pro', family: 'Source Code Pro, monospace', category: 'monospace' },
  { name: 'Monaco', family: 'Monaco, monospace', category: 'monospace' },
  { name: 'Menlo', family: 'Menlo, monospace', category: 'monospace' },
  { name: 'Consolas', family: 'Consolas, monospace', category: 'monospace' },
  { name: 'Inter', family: 'Inter, sans-serif', category: 'sans-serif' },
  { name: 'SF Pro', family: '-apple-system, BlinkMacSystemFont, sans-serif', category: 'sans-serif' },
  { name: 'Roboto', family: 'Roboto, sans-serif', category: 'sans-serif' },
];
```

### 2. Theme Store (src/lib/stores/theme-store.ts)

```typescript
import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import type { ThemeMode } from '$lib/types/theme';
import { appearanceSettings } from './settings-store';

function createThemeStore() {
  const systemPreference = writable<'light' | 'dark'>('light');

  if (browser) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    systemPreference.set(mediaQuery.matches ? 'dark' : 'light');

    mediaQuery.addEventListener('change', (e) => {
      systemPreference.set(e.matches ? 'dark' : 'light');
    });
  }

  const effectiveTheme = derived(
    [appearanceSettings, systemPreference],
    ([$appearance, $system]) => {
      if ($appearance.theme === 'system') {
        return $system;
      }
      return $appearance.theme;
    }
  );

  // Apply theme to document
  if (browser) {
    effectiveTheme.subscribe(theme => {
      document.documentElement.setAttribute('data-theme', theme);
    });
  }

  return {
    systemPreference: { subscribe: systemPreference.subscribe },
    effectiveTheme,
  };
}

export const themeStore = createThemeStore();

// Apply CSS variables for accent color
export function applyAccentColor(color: string) {
  if (!browser) return;

  document.documentElement.style.setProperty('--color-primary', color);
  document.documentElement.style.setProperty('--color-primary-hover', adjustColor(color, -10));
  document.documentElement.style.setProperty('--color-primary-active', adjustColor(color, -20));
  document.documentElement.style.setProperty('--color-primary-light', adjustColor(color, 40));
}

function adjustColor(hex: string, percent: number): string {
  const num = parseInt(hex.replace('#', ''), 16);
  const amt = Math.round(2.55 * percent);
  const R = Math.max(0, Math.min(255, (num >> 16) + amt));
  const G = Math.max(0, Math.min(255, ((num >> 8) & 0x00ff) + amt));
  const B = Math.max(0, Math.min(255, (num & 0x0000ff) + amt));
  return `#${(0x1000000 + R * 0x10000 + G * 0x100 + B).toString(16).slice(1)}`;
}
```

### 3. Appearance Settings Component (src/lib/components/settings/AppearanceSettings.svelte)

```svelte
<script lang="ts">
  import { settingsStore, appearanceSettings } from '$lib/stores/settings-store';
  import { themeStore, applyAccentColor } from '$lib/stores/theme-store';
  import { ACCENT_COLOR_PRESETS, FONT_OPTIONS } from '$lib/types/theme';
  import type { ThemeMode } from '$lib/types/theme';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Slider from '$lib/components/ui/Slider.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  let customColor = $appearanceSettings.accentColor;
  let colorInputRef: HTMLInputElement;

  const themeOptions: { value: ThemeMode; label: string; icon: string }[] = [
    { value: 'light', label: 'Light', icon: 'sun' },
    { value: 'dark', label: 'Dark', icon: 'moon' },
    { value: 'system', label: 'System', icon: 'monitor' },
  ];

  function handleThemeChange(theme: ThemeMode) {
    settingsStore.updateSetting('appearance', 'theme', theme);
  }

  function handleAccentColorSelect(color: string) {
    customColor = color;
    settingsStore.updateSetting('appearance', 'accentColor', color);
    applyAccentColor(color);
  }

  function handleCustomColorChange(event: Event) {
    const target = event.target as HTMLInputElement;
    customColor = target.value;
    settingsStore.updateSetting('appearance', 'accentColor', target.value);
    applyAccentColor(target.value);
  }

  function handleFontFamilyChange(event: CustomEvent<string>) {
    settingsStore.updateSetting('appearance', 'fontFamily', event.detail);
  }

  function handleFontSizeChange(event: CustomEvent<number>) {
    settingsStore.updateSetting('appearance', 'fontSize', event.detail);
  }

  function handleLineHeightChange(event: CustomEvent<number>) {
    settingsStore.updateSetting('appearance', 'lineHeight', event.detail);
  }

  function handleSidebarPositionChange(position: 'left' | 'right') {
    settingsStore.updateSetting('appearance', 'sidebarPosition', position);
  }

  function handleToggle(key: 'reducedMotion' | 'highContrast') {
    return (event: CustomEvent<boolean>) => {
      settingsStore.updateSetting('appearance', key, event.detail);
    };
  }

  $: isCustomColor = !ACCENT_COLOR_PRESETS.some(p => p.color === $appearanceSettings.accentColor);
</script>

<div class="appearance-settings">
  <h2 class="settings-title">Appearance</h2>
  <p class="settings-description">
    Customize the visual appearance of the application.
  </p>

  <!-- Theme Section -->
  <SettingsSection title="Theme">
    <div class="theme-selector">
      {#each themeOptions as option}
        <button
          class="theme-option"
          class:theme-option--active={$appearanceSettings.theme === option.value}
          on:click={() => handleThemeChange(option.value)}
          aria-pressed={$appearanceSettings.theme === option.value}
        >
          <div class="theme-option__preview theme-option__preview--{option.value}">
            <Icon name={option.icon} size={24} />
          </div>
          <span class="theme-option__label">{option.label}</span>
        </button>
      {/each}
    </div>

    {#if $appearanceSettings.theme === 'system'}
      <p class="theme-info">
        Current system preference: <strong>{$themeStore.systemPreference}</strong>
      </p>
    {/if}
  </SettingsSection>

  <!-- Accent Color Section -->
  <SettingsSection title="Accent Color">
    <div class="accent-colors">
      {#each ACCENT_COLOR_PRESETS as preset}
        <button
          class="accent-color"
          class:accent-color--active={$appearanceSettings.accentColor === preset.color}
          style="--color: {preset.color}"
          on:click={() => handleAccentColorSelect(preset.color)}
          title={preset.name}
          aria-label={preset.name}
          aria-pressed={$appearanceSettings.accentColor === preset.color}
        >
          {#if $appearanceSettings.accentColor === preset.color}
            <Icon name="check" size={16} />
          {/if}
        </button>
      {/each}

      <button
        class="accent-color accent-color--custom"
        class:accent-color--active={isCustomColor}
        style="--color: {customColor}"
        on:click={() => colorInputRef.click()}
        title="Custom color"
        aria-label="Custom color"
      >
        <Icon name="palette" size={16} />
      </button>

      <input
        bind:this={colorInputRef}
        type="color"
        value={customColor}
        on:input={handleCustomColorChange}
        class="color-input-hidden"
        aria-label="Custom color picker"
      />
    </div>

    <div class="current-color">
      <span class="current-color__label">Current:</span>
      <span class="current-color__value">{$appearanceSettings.accentColor}</span>
      <span
        class="current-color__swatch"
        style="background: {$appearanceSettings.accentColor}"
      />
    </div>
  </SettingsSection>

  <!-- Typography Section -->
  <SettingsSection title="Typography">
    <SettingsRow
      label="Font Family"
      description="Choose the font for code and interface"
    >
      <Select
        value={$appearanceSettings.fontFamily}
        options={FONT_OPTIONS.map(f => ({
          value: f.family,
          label: f.name,
        }))}
        on:change={handleFontFamilyChange}
      />
    </SettingsRow>

    <SettingsRow
      label="Font Size"
      description="Base font size in pixels"
    >
      <div class="slider-with-value">
        <Slider
          min={10}
          max={24}
          step={1}
          value={$appearanceSettings.fontSize}
          on:change={handleFontSizeChange}
        />
        <span class="slider-value">{$appearanceSettings.fontSize}px</span>
      </div>
    </SettingsRow>

    <SettingsRow
      label="Line Height"
      description="Spacing between lines of text"
    >
      <div class="slider-with-value">
        <Slider
          min={1}
          max={2.5}
          step={0.1}
          value={$appearanceSettings.lineHeight}
          on:change={handleLineHeightChange}
        />
        <span class="slider-value">{$appearanceSettings.lineHeight.toFixed(1)}</span>
      </div>
    </SettingsRow>

    <!-- Typography Preview -->
    <div
      class="typography-preview"
      style="font-family: {$appearanceSettings.fontFamily}; font-size: {$appearanceSettings.fontSize}px; line-height: {$appearanceSettings.lineHeight}"
    >
      <p>The quick brown fox jumps over the lazy dog.</p>
      <code>const greeting = "Hello, World!";</code>
    </div>
  </SettingsSection>

  <!-- Layout Section -->
  <SettingsSection title="Layout">
    <SettingsRow
      label="Sidebar Position"
      description="Choose which side the sidebar appears on"
    >
      <div class="position-selector">
        <button
          class="position-option"
          class:position-option--active={$appearanceSettings.sidebarPosition === 'left'}
          on:click={() => handleSidebarPositionChange('left')}
          aria-pressed={$appearanceSettings.sidebarPosition === 'left'}
        >
          <div class="position-option__icon position-option__icon--left">
            <div class="position-option__sidebar" />
            <div class="position-option__content" />
          </div>
          <span>Left</span>
        </button>
        <button
          class="position-option"
          class:position-option--active={$appearanceSettings.sidebarPosition === 'right'}
          on:click={() => handleSidebarPositionChange('right')}
          aria-pressed={$appearanceSettings.sidebarPosition === 'right'}
        >
          <div class="position-option__icon position-option__icon--right">
            <div class="position-option__content" />
            <div class="position-option__sidebar" />
          </div>
          <span>Right</span>
        </button>
      </div>
    </SettingsRow>
  </SettingsSection>

  <!-- Accessibility Section -->
  <SettingsSection title="Accessibility">
    <SettingsRow
      label="Reduced motion"
      description="Minimize animations and transitions"
    >
      <Toggle
        checked={$appearanceSettings.reducedMotion}
        on:change={handleToggle('reducedMotion')}
      />
    </SettingsRow>

    <SettingsRow
      label="High contrast"
      description="Increase contrast for better visibility"
    >
      <Toggle
        checked={$appearanceSettings.highContrast}
        on:change={handleToggle('highContrast')}
      />
    </SettingsRow>
  </SettingsSection>
</div>

<style>
  .appearance-settings {
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

  /* Theme Selector */
  .theme-selector {
    display: flex;
    gap: 16px;
  }

  .theme-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border: 2px solid var(--color-border);
    border-radius: 12px;
    background: transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .theme-option:hover {
    border-color: var(--color-text-muted);
  }

  .theme-option--active {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .theme-option__preview {
    width: 80px;
    height: 60px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .theme-option__preview--light {
    background: #ffffff;
    color: #333333;
    border: 1px solid #e0e0e0;
  }

  .theme-option__preview--dark {
    background: #1a1a2e;
    color: #e0e0e0;
    border: 1px solid #2d3748;
  }

  .theme-option__preview--system {
    background: linear-gradient(135deg, #ffffff 50%, #1a1a2e 50%);
    color: #666666;
    border: 1px solid #e0e0e0;
  }

  .theme-option__label {
    font-size: 13px;
    color: var(--color-text-primary);
    font-weight: 500;
  }

  .theme-info {
    margin-top: 12px;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  /* Accent Colors */
  .accent-colors {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 16px;
  }

  .accent-color {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--color);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    transition: all 0.15s ease;
  }

  .accent-color:hover {
    transform: scale(1.1);
  }

  .accent-color--active {
    border-color: var(--color-text-primary);
    box-shadow: 0 0 0 2px var(--color-bg-primary), 0 0 0 4px var(--color);
  }

  .accent-color--custom {
    background: conic-gradient(red, yellow, lime, aqua, blue, magenta, red);
    color: white;
  }

  .color-input-hidden {
    position: absolute;
    width: 0;
    height: 0;
    opacity: 0;
    pointer-events: none;
  }

  .current-color {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .current-color__label {
    color: var(--color-text-secondary);
  }

  .current-color__value {
    font-family: monospace;
    color: var(--color-text-primary);
  }

  .current-color__swatch {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: 1px solid var(--color-border);
  }

  /* Typography */
  .slider-with-value {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 200px;
  }

  .slider-value {
    min-width: 50px;
    text-align: right;
    font-size: 13px;
    color: var(--color-text-secondary);
    font-family: monospace;
  }

  .typography-preview {
    margin-top: 16px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    border: 1px solid var(--color-border);
  }

  .typography-preview p {
    margin: 0 0 12px 0;
    color: var(--color-text-primary);
  }

  .typography-preview code {
    display: block;
    padding: 8px 12px;
    background: var(--color-bg-primary);
    border-radius: 4px;
    color: var(--color-primary);
  }

  /* Position Selector */
  .position-selector {
    display: flex;
    gap: 12px;
  }

  .position-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border: 2px solid var(--color-border);
    border-radius: 8px;
    background: transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .position-option:hover {
    border-color: var(--color-text-muted);
  }

  .position-option--active {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .position-option__icon {
    display: flex;
    width: 60px;
    height: 40px;
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--color-border);
  }

  .position-option__sidebar {
    width: 16px;
    background: var(--color-primary);
  }

  .position-option__content {
    flex: 1;
    background: var(--color-bg-secondary);
  }

  .position-option span {
    font-size: 12px;
    color: var(--color-text-secondary);
  }
</style>
```

---

## Testing Requirements

1. Theme selector changes theme correctly
2. System theme preference is detected
3. Accent color presets apply correctly
4. Custom color picker works
5. Font settings update in real-time
6. Typography preview reflects changes
7. Accessibility toggles work
8. Sidebar position changes

### Test File (src/lib/components/settings/__tests__/AppearanceSettings.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import AppearanceSettings from '../AppearanceSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

describe('AppearanceSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders all appearance sections', () => {
    render(AppearanceSettings);

    expect(screen.getByText('Theme')).toBeInTheDocument();
    expect(screen.getByText('Accent Color')).toBeInTheDocument();
    expect(screen.getByText('Typography')).toBeInTheDocument();
    expect(screen.getByText('Accessibility')).toBeInTheDocument();
  });

  it('changes theme when clicking theme option', async () => {
    render(AppearanceSettings);

    const darkButton = screen.getByRole('button', { name: /dark/i });
    await fireEvent.click(darkButton);

    const state = get(settingsStore);
    expect(state.settings.appearance.theme).toBe('dark');
  });

  it('selects accent color preset', async () => {
    render(AppearanceSettings);

    const purpleButton = screen.getByRole('button', { name: /purple/i });
    await fireEvent.click(purpleButton);

    const state = get(settingsStore);
    expect(state.settings.appearance.accentColor).toBe('#9c27b0');
  });

  it('updates font size with slider', async () => {
    render(AppearanceSettings);

    const slider = screen.getAllByRole('slider')[0];
    await fireEvent.input(slider, { target: { value: 18 } });

    const state = get(settingsStore);
    expect(state.settings.appearance.fontSize).toBe(18);
  });

  it('toggles reduced motion', async () => {
    render(AppearanceSettings);

    const toggle = screen.getByRole('switch', { name: /reduced motion/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.appearance.reducedMotion).toBe(true);
  });

  it('changes sidebar position', async () => {
    render(AppearanceSettings);

    const rightButton = screen.getByRole('button', { name: /right/i });
    await fireEvent.click(rightButton);

    const state = get(settingsStore);
    expect(state.settings.appearance.sidebarPosition).toBe('right');
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
- Previous: [273-settings-general.md](273-settings-general.md)
- Next: [275-settings-backends.md](275-settings-backends.md)
