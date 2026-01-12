# Spec 285: Font & Accessibility

## Header
- **Spec ID**: 285
- **Phase**: 13 - Settings UI
- **Component**: Font & Accessibility
- **Dependencies**: Spec 284 (Theme Selection)
- **Status**: Draft

## Objective
Create accessibility settings for configuring fonts, text sizing, spacing, motion preferences, and other accessibility features to ensure the application is usable by people with various needs.

## Acceptance Criteria
1. Configure font family and size preferences
2. Adjust line height and letter spacing
3. Set high contrast mode options
4. Configure reduced motion preferences
5. Enable focus indicators customization
6. Set up screen reader announcements
7. Configure color blindness adaptations
8. Preview accessibility settings live

## Implementation

### FontAccessibility.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import FontPreview from './FontPreview.svelte';
  import ColorBlindnessFilter from './ColorBlindnessFilter.svelte';
  import FocusIndicatorConfig from './FocusIndicatorConfig.svelte';
  import MotionSettings from './MotionSettings.svelte';
  import { accessibilityStore } from '$lib/stores/accessibility';
  import type {
    AccessibilitySettings,
    FontSettings,
    ColorBlindnessMode,
    MotionPreference
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: AccessibilitySettings;
    reset: void;
  }>();

  const fontFamilies = [
    { id: 'system', name: 'System Default', value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif' },
    { id: 'inter', name: 'Inter', value: '"Inter", sans-serif' },
    { id: 'roboto', name: 'Roboto', value: '"Roboto", sans-serif' },
    { id: 'opensans', name: 'Open Sans', value: '"Open Sans", sans-serif' },
    { id: 'atkinson', name: 'Atkinson Hyperlegible', value: '"Atkinson Hyperlegible", sans-serif' },
    { id: 'opendyslexic', name: 'OpenDyslexic', value: '"OpenDyslexic", sans-serif' },
    { id: 'mono', name: 'Monospace', value: '"JetBrains Mono", "Fira Code", monospace' }
  ];

  const colorBlindnessModes: { id: ColorBlindnessMode; name: string; description: string }[] = [
    { id: 'none', name: 'None', description: 'No color adjustment' },
    { id: 'protanopia', name: 'Protanopia', description: 'Red-blind color vision' },
    { id: 'deuteranopia', name: 'Deuteranopia', description: 'Green-blind color vision' },
    { id: 'tritanopia', name: 'Tritanopia', description: 'Blue-blind color vision' },
    { id: 'achromatopsia', name: 'Achromatopsia', description: 'Complete color blindness' }
  ];

  let showPreview = writable<boolean>(true);

  const settings = derived(accessibilityStore, ($store) => $store.settings);

  function updateFontSettings(field: keyof FontSettings, value: unknown) {
    accessibilityStore.updateFont(field, value);
  }

  function updateSetting(field: keyof AccessibilitySettings, value: unknown) {
    accessibilityStore.update(field, value);
  }

  async function saveSettings() {
    await accessibilityStore.save();
    dispatch('save', $settings);
  }

  function resetToDefaults() {
    if (confirm('Reset all accessibility settings to defaults?')) {
      accessibilityStore.resetToDefaults();
      dispatch('reset');
    }
  }

  onMount(() => {
    accessibilityStore.load();
  });
</script>

<div class="font-accessibility" data-testid="font-accessibility">
  <header class="config-header">
    <div class="header-title">
      <h2>Font & Accessibility</h2>
      <p class="description">Customize fonts and accessibility options</p>
    </div>

    <div class="header-actions">
      <label class="preview-toggle">
        <input type="checkbox" bind:checked={$showPreview} />
        Live Preview
      </label>
      <button class="btn secondary" on:click={resetToDefaults}>
        Reset to Defaults
      </button>
      <button class="btn primary" on:click={saveSettings}>
        Save Settings
      </button>
    </div>
  </header>

  <div class="settings-layout" class:with-preview={$showPreview}>
    <div class="settings-content">
      <section class="config-section">
        <h3>Typography</h3>

        <div class="form-group">
          <label for="font-family">Font Family</label>
          <select
            id="font-family"
            value={$settings.font.family}
            on:change={(e) => updateFontSettings('family', (e.target as HTMLSelectElement).value)}
          >
            {#each fontFamilies as font}
              <option value={font.value}>{font.name}</option>
            {/each}
          </select>
          <span class="help-text">Choose a font for the interface</span>
        </div>

        <div class="form-group">
          <label for="font-size">Base Font Size</label>
          <div class="slider-with-value">
            <input
              id="font-size"
              type="range"
              min="12"
              max="24"
              step="1"
              value={$settings.font.size}
              on:input={(e) => updateFontSettings('size', parseInt((e.target as HTMLInputElement).value))}
            />
            <span class="slider-value">{$settings.font.size}px</span>
          </div>
        </div>

        <div class="form-group">
          <label for="line-height">Line Height</label>
          <div class="slider-with-value">
            <input
              id="line-height"
              type="range"
              min="1.2"
              max="2.0"
              step="0.1"
              value={$settings.font.lineHeight}
              on:input={(e) => updateFontSettings('lineHeight', parseFloat((e.target as HTMLInputElement).value))}
            />
            <span class="slider-value">{$settings.font.lineHeight}</span>
          </div>
        </div>

        <div class="form-group">
          <label for="letter-spacing">Letter Spacing</label>
          <div class="slider-with-value">
            <input
              id="letter-spacing"
              type="range"
              min="-0.05"
              max="0.2"
              step="0.01"
              value={$settings.font.letterSpacing}
              on:input={(e) => updateFontSettings('letterSpacing', parseFloat((e.target as HTMLInputElement).value))}
            />
            <span class="slider-value">{$settings.font.letterSpacing}em</span>
          </div>
        </div>

        <div class="form-group">
          <label for="word-spacing">Word Spacing</label>
          <div class="slider-with-value">
            <input
              id="word-spacing"
              type="range"
              min="0"
              max="0.5"
              step="0.05"
              value={$settings.font.wordSpacing}
              on:input={(e) => updateFontSettings('wordSpacing', parseFloat((e.target as HTMLInputElement).value))}
            />
            <span class="slider-value">{$settings.font.wordSpacing}em</span>
          </div>
        </div>
      </section>

      <section class="config-section">
        <h3>Visual</h3>

        <div class="toggle-options">
          <label class="toggle-option">
            <input
              type="checkbox"
              checked={$settings.highContrast}
              on:change={(e) => updateSetting('highContrast', (e.target as HTMLInputElement).checked)}
            />
            <span class="toggle-content">
              <span class="toggle-label">High Contrast Mode</span>
              <span class="toggle-desc">Increase contrast for better readability</span>
            </span>
          </label>

          <label class="toggle-option">
            <input
              type="checkbox"
              checked={$settings.largerClickTargets}
              on:change={(e) => updateSetting('largerClickTargets', (e.target as HTMLInputElement).checked)}
            />
            <span class="toggle-content">
              <span class="toggle-label">Larger Click Targets</span>
              <span class="toggle-desc">Increase size of buttons and interactive elements</span>
            </span>
          </label>

          <label class="toggle-option">
            <input
              type="checkbox"
              checked={$settings.underlineLinks}
              on:change={(e) => updateSetting('underlineLinks', (e.target as HTMLInputElement).checked)}
            />
            <span class="toggle-content">
              <span class="toggle-label">Underline Links</span>
              <span class="toggle-desc">Always show underlines on links</span>
            </span>
          </label>
        </div>

        <div class="form-group">
          <label>Color Blindness Adaptation</label>
          <div class="color-blindness-options">
            {#each colorBlindnessModes as mode}
              <label class="cb-option" class:selected={$settings.colorBlindnessMode === mode.id}>
                <input
                  type="radio"
                  name="colorBlindness"
                  value={mode.id}
                  checked={$settings.colorBlindnessMode === mode.id}
                  on:change={() => updateSetting('colorBlindnessMode', mode.id)}
                />
                <span class="cb-name">{mode.name}</span>
                <span class="cb-desc">{mode.description}</span>
              </label>
            {/each}
          </div>
        </div>
      </section>

      <section class="config-section">
        <h3>Motion</h3>
        <MotionSettings
          preference={$settings.motionPreference}
          on:change={(e) => updateSetting('motionPreference', e.detail)}
        />
      </section>

      <section class="config-section">
        <h3>Focus Indicators</h3>
        <FocusIndicatorConfig
          settings={$settings.focusIndicator}
          on:change={(e) => updateSetting('focusIndicator', e.detail)}
        />
      </section>

      <section class="config-section">
        <h3>Screen Reader</h3>

        <div class="toggle-options">
          <label class="toggle-option">
            <input
              type="checkbox"
              checked={$settings.announceNavigation}
              on:change={(e) => updateSetting('announceNavigation', (e.target as HTMLInputElement).checked)}
            />
            <span class="toggle-content">
              <span class="toggle-label">Announce Navigation</span>
              <span class="toggle-desc">Announce page changes to screen readers</span>
            </span>
          </label>

          <label class="toggle-option">
            <input
              type="checkbox"
              checked={$settings.verboseDescriptions}
              on:change={(e) => updateSetting('verboseDescriptions', (e.target as HTMLInputElement).checked)}
            />
            <span class="toggle-content">
              <span class="toggle-label">Verbose Descriptions</span>
              <span class="toggle-desc">Provide detailed descriptions for complex elements</span>
            </span>
          </label>

          <label class="toggle-option">
            <input
              type="checkbox"
              checked={$settings.announceUpdates}
              on:change={(e) => updateSetting('announceUpdates', (e.target as HTMLInputElement).checked)}
            />
            <span class="toggle-content">
              <span class="toggle-label">Announce Live Updates</span>
              <span class="toggle-desc">Announce dynamic content changes</span>
            </span>
          </label>
        </div>
      </section>
    </div>

    {#if $showPreview}
      <aside class="preview-panel">
        <FontPreview settings={$settings} />
      </aside>
    {/if}
  </div>
</div>

<style>
  .font-accessibility {
    max-width: 1100px;
  }

  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 2rem;
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
    align-items: center;
  }

  .preview-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .settings-layout {
    display: flex;
    gap: 1.5rem;
  }

  .settings-layout.with-preview .settings-content {
    flex: 1;
  }

  .settings-content {
    flex: 1;
  }

  .preview-panel {
    width: 350px;
    position: sticky;
    top: 1rem;
    height: fit-content;
  }

  .config-section {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .config-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1.25rem;
  }

  .form-group {
    margin-bottom: 1.25rem;
  }

  .form-group:last-child {
    margin-bottom: 0;
  }

  .form-group label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  .form-group select {
    width: 100%;
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

  .slider-with-value {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .slider-with-value input[type="range"] {
    flex: 1;
  }

  .slider-value {
    min-width: 50px;
    text-align: right;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
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

  .toggle-option input {
    margin-top: 0.25rem;
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

  .color-blindness-options {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 0.75rem;
  }

  .cb-option {
    display: flex;
    flex-direction: column;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border: 2px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .cb-option:hover {
    border-color: var(--primary-color);
  }

  .cb-option.selected {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .cb-option input {
    display: none;
  }

  .cb-name {
    font-weight: 500;
    font-size: 0.875rem;
    margin-bottom: 0.25rem;
  }

  .cb-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
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

  @media (max-width: 900px) {
    .settings-layout {
      flex-direction: column;
    }

    .preview-panel {
      width: 100%;
      position: static;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test settings persistence
2. **Accessibility Tests**: Verify WCAG compliance
3. **Visual Tests**: Test all color blindness modes
4. **Font Tests**: Verify font loading and application
5. **Motion Tests**: Test reduced motion implementation

## Related Specs
- Spec 284: Theme Selection
- Spec 286: Keyboard Config
- Spec 295: Settings Tests
