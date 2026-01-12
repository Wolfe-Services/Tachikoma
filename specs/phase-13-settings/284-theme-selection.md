# Spec 284: Theme Selection

## Header
- **Spec ID**: 284
- **Phase**: 13 - Settings UI
- **Component**: Theme Selection
- **Dependencies**: Spec 276 (Settings Layout)
- **Status**: Draft

## Objective
Create a theme selection interface that allows users to customize the visual appearance of the application, including color schemes, light/dark modes, and custom theme creation.

## Acceptance Criteria
1. Select from built-in light and dark themes
2. Support system preference auto-detection
3. Create and save custom themes
4. Preview themes before applying
5. Configure accent colors
6. Export and import custom themes
7. Schedule theme changes by time
8. Accessibility contrast validation

## Implementation

### ThemeSelection.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade } from 'svelte/transition';
  import ThemeCard from './ThemeCard.svelte';
  import ThemePreview from './ThemePreview.svelte';
  import ThemeEditor from './ThemeEditor.svelte';
  import AccentColorPicker from './AccentColorPicker.svelte';
  import ThemeScheduler from './ThemeScheduler.svelte';
  import { themeStore } from '$lib/stores/theme';
  import { validateContrast } from '$lib/utils/accessibility';
  import type { Theme, ThemeMode, ThemeSchedule, AccentColor } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    change: Theme;
    create: Theme;
  }>();

  const builtInThemes: Theme[] = [
    {
      id: 'light',
      name: 'Light',
      mode: 'light',
      colors: {
        background: '#ffffff',
        foreground: '#1a1a1a',
        primary: '#4a9eff',
        secondary: '#f5f5f5',
        accent: '#4a9eff',
        border: '#e0e0e0',
        muted: '#666666'
      },
      builtIn: true
    },
    {
      id: 'dark',
      name: 'Dark',
      mode: 'dark',
      colors: {
        background: '#1a1a2e',
        foreground: '#eaeaea',
        primary: '#4a9eff',
        secondary: '#16213e',
        accent: '#4a9eff',
        border: '#2a2a4a',
        muted: '#888888'
      },
      builtIn: true
    },
    {
      id: 'midnight',
      name: 'Midnight',
      mode: 'dark',
      colors: {
        background: '#0d1117',
        foreground: '#c9d1d9',
        primary: '#58a6ff',
        secondary: '#161b22',
        accent: '#58a6ff',
        border: '#30363d',
        muted: '#8b949e'
      },
      builtIn: true
    },
    {
      id: 'sepia',
      name: 'Sepia',
      mode: 'light',
      colors: {
        background: '#f4ecd8',
        foreground: '#3c3836',
        primary: '#b57614',
        secondary: '#ebdbb2',
        accent: '#b57614',
        border: '#d5c4a1',
        muted: '#7c6f64'
      },
      builtIn: true
    }
  ];

  const accentColors: AccentColor[] = [
    { id: 'blue', name: 'Blue', value: '#4a9eff' },
    { id: 'purple', name: 'Purple', value: '#8b5cf6' },
    { id: 'green', name: 'Green', value: '#10b981' },
    { id: 'orange', name: 'Orange', value: '#f59e0b' },
    { id: 'pink', name: 'Pink', value: '#ec4899' },
    { id: 'red', name: 'Red', value: '#ef4444' },
    { id: 'teal', name: 'Teal', value: '#14b8a6' }
  ];

  let showEditor = writable<boolean>(false);
  let showScheduler = writable<boolean>(false);
  let editingThemeId = writable<string | null>(null);
  let previewTheme = writable<Theme | null>(null);

  const currentTheme = derived(themeStore, ($store) => $store.current);
  const currentMode = derived(themeStore, ($store) => $store.mode);
  const customThemes = derived(themeStore, ($store) => $store.customThemes);
  const schedule = derived(themeStore, ($store) => $store.schedule);
  const useSystemTheme = derived(themeStore, ($store) => $store.useSystemTheme);

  const allThemes = derived(customThemes, ($custom) => [
    ...builtInThemes,
    ...$custom
  ]);

  function selectTheme(themeId: string) {
    const theme = $allThemes.find(t => t.id === themeId);
    if (theme) {
      themeStore.setTheme(theme);
      dispatch('change', theme);
    }
  }

  function setAccentColor(color: AccentColor) {
    themeStore.setAccentColor(color.value);
  }

  function toggleSystemTheme() {
    themeStore.setUseSystemTheme(!$useSystemTheme);
  }

  function setMode(mode: ThemeMode) {
    themeStore.setMode(mode);
  }

  function previewThemeHover(theme: Theme | null) {
    previewTheme.set(theme);
    if (theme) {
      applyThemePreview(theme);
    } else {
      applyThemePreview($currentTheme);
    }
  }

  function applyThemePreview(theme: Theme) {
    const root = document.documentElement;
    for (const [key, value] of Object.entries(theme.colors)) {
      root.style.setProperty(`--preview-${key}`, value);
    }
  }

  function createTheme() {
    editingThemeId.set(null);
    showEditor.set(true);
  }

  function editTheme(themeId: string) {
    editingThemeId.set(themeId);
    showEditor.set(true);
  }

  function saveTheme(theme: Theme) {
    if ($editingThemeId) {
      themeStore.updateCustomTheme(theme);
    } else {
      themeStore.addCustomTheme(theme);
      dispatch('create', theme);
    }
    showEditor.set(false);
    editingThemeId.set(null);
  }

  function deleteTheme(themeId: string) {
    if (confirm('Delete this custom theme?')) {
      themeStore.removeCustomTheme(themeId);
    }
  }

  function exportTheme(theme: Theme) {
    const data = JSON.stringify(theme, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `theme-${theme.name.toLowerCase().replace(/\s+/g, '-')}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function importTheme() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const text = await file.text();
        const theme = JSON.parse(text);
        theme.id = crypto.randomUUID();
        theme.builtIn = false;
        themeStore.addCustomTheme(theme);
      }
    };
    input.click();
  }

  function saveSchedule(newSchedule: ThemeSchedule) {
    themeStore.setSchedule(newSchedule);
    showScheduler.set(false);
  }

  onMount(() => {
    themeStore.load();
  });
</script>

<div class="theme-selection" data-testid="theme-selection">
  <header class="config-header">
    <div class="header-title">
      <h2>Theme & Appearance</h2>
      <p class="description">Customize how Tachikoma looks</p>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={importTheme}>
        Import
      </button>
      <button class="btn secondary" on:click={() => showScheduler.set(true)}>
        Schedule
      </button>
      <button class="btn primary" on:click={createTheme}>
        Create Theme
      </button>
    </div>
  </header>

  <section class="mode-section">
    <h3>Appearance Mode</h3>

    <div class="mode-options">
      <label class="mode-option" class:active={$useSystemTheme}>
        <input
          type="radio"
          name="mode"
          checked={$useSystemTheme}
          on:change={toggleSystemTheme}
        />
        <div class="mode-icon system">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <rect x="2" y="3" width="20" height="14" rx="2" stroke-width="2"/>
            <path d="M8 21h8M12 17v4" stroke-width="2"/>
          </svg>
        </div>
        <span class="mode-label">System</span>
        <span class="mode-desc">Follow system preference</span>
      </label>

      <label class="mode-option" class:active={!$useSystemTheme && $currentMode === 'light'}>
        <input
          type="radio"
          name="mode"
          checked={!$useSystemTheme && $currentMode === 'light'}
          on:change={() => { themeStore.setUseSystemTheme(false); setMode('light'); }}
        />
        <div class="mode-icon light">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="5" stroke-width="2"/>
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" stroke-width="2"/>
          </svg>
        </div>
        <span class="mode-label">Light</span>
        <span class="mode-desc">Light appearance</span>
      </label>

      <label class="mode-option" class:active={!$useSystemTheme && $currentMode === 'dark'}>
        <input
          type="radio"
          name="mode"
          checked={!$useSystemTheme && $currentMode === 'dark'}
          on:change={() => { themeStore.setUseSystemTheme(false); setMode('dark'); }}
        />
        <div class="mode-icon dark">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" stroke-width="2"/>
          </svg>
        </div>
        <span class="mode-label">Dark</span>
        <span class="mode-desc">Dark appearance</span>
      </label>
    </div>
  </section>

  <section class="accent-section">
    <h3>Accent Color</h3>
    <AccentColorPicker
      colors={accentColors}
      selected={$currentTheme?.colors.accent}
      on:select={(e) => setAccentColor(e.detail)}
    />
  </section>

  <section class="themes-section">
    <h3>Themes</h3>

    <div class="theme-group">
      <h4>Built-in Themes</h4>
      <div class="themes-grid">
        {#each builtInThemes.filter(t => t.mode === $currentMode || $useSystemTheme) as theme (theme.id)}
          <ThemeCard
            {theme}
            selected={$currentTheme?.id === theme.id}
            on:select={() => selectTheme(theme.id)}
            on:mouseenter={() => previewThemeHover(theme)}
            on:mouseleave={() => previewThemeHover(null)}
          />
        {/each}
      </div>
    </div>

    {#if $customThemes.length > 0}
      <div class="theme-group">
        <h4>Custom Themes</h4>
        <div class="themes-grid">
          {#each $customThemes.filter(t => t.mode === $currentMode || $useSystemTheme) as theme (theme.id)}
            <ThemeCard
              {theme}
              selected={$currentTheme?.id === theme.id}
              editable
              on:select={() => selectTheme(theme.id)}
              on:edit={() => editTheme(theme.id)}
              on:delete={() => deleteTheme(theme.id)}
              on:export={() => exportTheme(theme)}
              on:mouseenter={() => previewThemeHover(theme)}
              on:mouseleave={() => previewThemeHover(null)}
            />
          {/each}
        </div>
      </div>
    {/if}
  </section>

  {#if $schedule?.enabled}
    <section class="schedule-section">
      <h3>Theme Schedule</h3>
      <div class="schedule-info">
        <p>
          <strong>Light theme:</strong> {$schedule.lightTime} -
          <strong>Dark theme:</strong> {$schedule.darkTime}
        </p>
        <button class="btn secondary" on:click={() => showScheduler.set(true)}>
          Edit Schedule
        </button>
      </div>
    </section>
  {/if}

  {#if $showEditor}
    <div class="modal-overlay" transition:fade on:click={() => showEditor.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <ThemeEditor
          theme={$editingThemeId ? $customThemes.find(t => t.id === $editingThemeId) : null}
          mode={$currentMode}
          on:save={(e) => saveTheme(e.detail)}
          on:close={() => {
            showEditor.set(false);
            editingThemeId.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showScheduler}
    <div class="modal-overlay" transition:fade on:click={() => showScheduler.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ThemeScheduler
          schedule={$schedule}
          on:save={(e) => saveSchedule(e.detail)}
          on:close={() => showScheduler.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .theme-selection {
    max-width: 900px;
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
  }

  .mode-section,
  .accent-section,
  .themes-section,
  .schedule-section {
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

  .mode-options {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .mode-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1.25rem;
    background: var(--secondary-bg);
    border: 2px solid var(--border-color);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mode-option:hover {
    border-color: var(--primary-color);
  }

  .mode-option.active {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .mode-option input {
    display: none;
  }

  .mode-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--card-bg);
    border-radius: 12px;
    margin-bottom: 0.75rem;
  }

  .mode-label {
    font-weight: 600;
    font-size: 0.9375rem;
    margin-bottom: 0.25rem;
  }

  .mode-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .theme-group {
    margin-bottom: 1.5rem;
  }

  .theme-group:last-child {
    margin-bottom: 0;
  }

  .theme-group h4 {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
  }

  .themes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 1rem;
  }

  .schedule-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .schedule-info p {
    font-size: 0.875rem;
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

  .modal-content.large {
    max-width: 700px;
  }

  @media (max-width: 768px) {
    .mode-options {
      grid-template-columns: 1fr;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test theme switching and persistence
2. **Visual Tests**: Verify theme application accuracy
3. **Accessibility Tests**: Test contrast validation
4. **Schedule Tests**: Test time-based switching
5. **Import/Export Tests**: Test theme portability

## Related Specs
- Spec 276: Settings Layout
- Spec 285: Font & Accessibility
- Spec 295: Settings Tests
