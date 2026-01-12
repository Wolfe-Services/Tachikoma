# 281 - Export Settings

**Phase:** 13 - Settings UI
**Spec ID:** 281
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~8% of model context window

---

## Objective

Create the Settings Export functionality that allows users to export their settings to a file with options for selective export, format selection, and sensitive data handling.

---

## Acceptance Criteria

- [ ] `ExportSettings.svelte` component with export options
- [ ] Export all settings to JSON file
- [ ] Selective export by category
- [ ] Option to exclude sensitive data (API keys)
- [ ] Export format options (JSON, YAML)
- [ ] Export preview before download
- [ ] Copy to clipboard option
- [ ] Export file naming with timestamp

---

## Implementation Details

### 1. Export Settings Component (src/lib/components/settings/ExportSettings.svelte)

```svelte
<script lang="ts">
  import { settingsStore } from '$lib/stores/settings-store';
  import type { AllSettings } from '$lib/types/settings';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  interface ExportSection {
    id: keyof AllSettings;
    label: string;
    description: string;
    selected: boolean;
    hasSensitiveData: boolean;
  }

  let exportSections: ExportSection[] = [
    { id: 'general', label: 'General Settings', description: 'Language, updates, startup', selected: true, hasSensitiveData: false },
    { id: 'appearance', label: 'Appearance', description: 'Theme, colors, fonts', selected: true, hasSensitiveData: false },
    { id: 'editor', label: 'Editor Preferences', description: 'Tab size, formatting', selected: true, hasSensitiveData: false },
    { id: 'keybindings', label: 'Keyboard Shortcuts', description: 'Custom key bindings', selected: true, hasSensitiveData: false },
    { id: 'backends', label: 'LLM Backends', description: 'Backend configurations', selected: true, hasSensitiveData: true },
    { id: 'git', label: 'Git Settings', description: 'Git preferences', selected: true, hasSensitiveData: true },
    { id: 'sync', label: 'Sync Settings', description: 'Cloud sync config', selected: false, hasSensitiveData: true },
  ];

  let exportFormat: 'json' | 'yaml' = 'json';
  let includeSensitiveData = false;
  let prettyPrint = true;
  let showPreview = false;
  let exportPreview = '';
  let copySuccess = false;

  function toggleSection(sectionId: keyof AllSettings) {
    exportSections = exportSections.map(s =>
      s.id === sectionId ? { ...s, selected: !s.selected } : s
    );
    if (showPreview) {
      generatePreview();
    }
  }

  function selectAll() {
    exportSections = exportSections.map(s => ({ ...s, selected: true }));
    if (showPreview) {
      generatePreview();
    }
  }

  function selectNone() {
    exportSections = exportSections.map(s => ({ ...s, selected: false }));
    if (showPreview) {
      generatePreview();
    }
  }

  function sanitizeSettings(settings: Partial<AllSettings>): Partial<AllSettings> {
    const sanitized = structuredClone(settings);

    // Remove sensitive data if not included
    if (!includeSensitiveData) {
      if (sanitized.backends) {
        sanitized.backends = {
          ...sanitized.backends,
          backends: sanitized.backends.backends.map(b => ({
            ...b,
            apiKey: b.apiKey ? '***REDACTED***' : undefined,
          })),
        };
      }

      if (sanitized.git) {
        sanitized.git = {
          ...sanitized.git,
          gpgKey: sanitized.git.gpgKey ? '***REDACTED***' : undefined,
        };
      }

      if (sanitized.sync) {
        sanitized.sync = {
          ...sanitized.sync,
          gistId: sanitized.sync.gistId ? '***REDACTED***' : undefined,
        };
      }
    }

    return sanitized;
  }

  function generateExportData(): object {
    const state = get(settingsStore);
    const selectedSettings: Partial<AllSettings> = {};

    exportSections
      .filter(s => s.selected)
      .forEach(section => {
        selectedSettings[section.id] = state.settings[section.id];
      });

    const sanitized = sanitizeSettings(selectedSettings);

    return {
      version: 1,
      exportedAt: new Date().toISOString(),
      application: 'tachikoma',
      settings: sanitized,
    };
  }

  function generatePreview() {
    const data = generateExportData();

    if (exportFormat === 'json') {
      exportPreview = JSON.stringify(data, null, prettyPrint ? 2 : 0);
    } else {
      // Simple YAML conversion
      exportPreview = convertToYaml(data);
    }
  }

  function convertToYaml(obj: object, indent: number = 0): string {
    const spaces = '  '.repeat(indent);
    let yaml = '';

    for (const [key, value] of Object.entries(obj)) {
      if (value === null || value === undefined) {
        yaml += `${spaces}${key}: null\n`;
      } else if (typeof value === 'object' && !Array.isArray(value)) {
        yaml += `${spaces}${key}:\n${convertToYaml(value, indent + 1)}`;
      } else if (Array.isArray(value)) {
        yaml += `${spaces}${key}:\n`;
        value.forEach(item => {
          if (typeof item === 'object') {
            yaml += `${spaces}  -\n${convertToYaml(item, indent + 2)}`;
          } else {
            yaml += `${spaces}  - ${item}\n`;
          }
        });
      } else if (typeof value === 'string') {
        yaml += `${spaces}${key}: "${value}"\n`;
      } else {
        yaml += `${spaces}${key}: ${value}\n`;
      }
    }

    return yaml;
  }

  function getFilename(): string {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
    return `tachikoma-settings-${timestamp}.${exportFormat}`;
  }

  function downloadExport() {
    const data = generateExportData();
    let content: string;
    let mimeType: string;

    if (exportFormat === 'json') {
      content = JSON.stringify(data, null, prettyPrint ? 2 : 0);
      mimeType = 'application/json';
    } else {
      content = convertToYaml(data);
      mimeType = 'application/x-yaml';
    }

    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = getFilename();
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  async function copyToClipboard() {
    const data = generateExportData();
    const content = exportFormat === 'json'
      ? JSON.stringify(data, null, prettyPrint ? 2 : 0)
      : convertToYaml(data);

    try {
      await navigator.clipboard.writeText(content);
      copySuccess = true;
      setTimeout(() => copySuccess = false, 2000);
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  }

  function get<T>(store: { subscribe: (fn: (value: T) => void) => void }): T {
    let value: T;
    store.subscribe(v => value = v)();
    return value!;
  }

  $: selectedCount = exportSections.filter(s => s.selected).length;
  $: hasSensitiveSelected = exportSections.some(s => s.selected && s.hasSensitiveData);
</script>

<div class="export-settings">
  <h2 class="settings-title">Export Settings</h2>
  <p class="settings-description">
    Export your settings to a file for backup or sharing.
  </p>

  <!-- Export Options -->
  <SettingsSection title="Export Options">
    <SettingsRow
      label="Format"
      description="Choose the export file format"
    >
      <Select
        value={exportFormat}
        options={[
          { value: 'json', label: 'JSON' },
          { value: 'yaml', label: 'YAML' },
        ]}
        on:change={(e) => {
          exportFormat = e.detail as 'json' | 'yaml';
          if (showPreview) generatePreview();
        }}
      />
    </SettingsRow>

    <SettingsRow
      label="Pretty Print"
      description="Format output with indentation for readability"
    >
      <Toggle
        checked={prettyPrint}
        on:change={(e) => {
          prettyPrint = e.detail;
          if (showPreview) generatePreview();
        }}
      />
    </SettingsRow>

    {#if hasSensitiveSelected}
      <SettingsRow
        label="Include Sensitive Data"
        description="Include API keys and tokens in export"
      >
        <Toggle
          checked={includeSensitiveData}
          on:change={(e) => {
            includeSensitiveData = e.detail;
            if (showPreview) generatePreview();
          }}
        />
      </SettingsRow>

      {#if !includeSensitiveData}
        <div class="sensitive-notice">
          <Icon name="shield" size={16} />
          <span>Sensitive data will be redacted in the export</span>
        </div>
      {:else}
        <div class="sensitive-warning">
          <Icon name="alert-triangle" size={16} />
          <span>Warning: Exported file will contain sensitive credentials</span>
        </div>
      {/if}
    {/if}
  </SettingsSection>

  <!-- Select Settings -->
  <SettingsSection title="Select Settings to Export">
    <div class="section-toolbar">
      <span class="selection-count">{selectedCount} of {exportSections.length} selected</span>
      <div class="selection-actions">
        <Button variant="ghost" size="small" on:click={selectAll}>Select All</Button>
        <Button variant="ghost" size="small" on:click={selectNone}>Select None</Button>
      </div>
    </div>

    <div class="export-sections">
      {#each exportSections as section}
        <div
          class="export-section"
          class:export-section--selected={section.selected}
          on:click={() => toggleSection(section.id)}
          on:keydown={(e) => e.key === 'Enter' && toggleSection(section.id)}
          role="checkbox"
          aria-checked={section.selected}
          tabindex="0"
        >
          <div class="export-section__checkbox">
            {#if section.selected}
              <Icon name="check-square" size={20} />
            {:else}
              <Icon name="square" size={20} />
            {/if}
          </div>
          <div class="export-section__info">
            <span class="export-section__label">
              {section.label}
              {#if section.hasSensitiveData}
                <Icon name="lock" size={12} class="sensitive-icon" title="Contains sensitive data" />
              {/if}
            </span>
            <span class="export-section__desc">{section.description}</span>
          </div>
        </div>
      {/each}
    </div>
  </SettingsSection>

  <!-- Preview -->
  <SettingsSection title="Preview">
    <div class="preview-toggle">
      <Button
        variant="secondary"
        on:click={() => {
          showPreview = !showPreview;
          if (showPreview) generatePreview();
        }}
      >
        <Icon name={showPreview ? 'eye-off' : 'eye'} size={16} />
        {showPreview ? 'Hide Preview' : 'Show Preview'}
      </Button>
    </div>

    {#if showPreview}
      <div class="preview-container">
        <div class="preview-header">
          <span class="preview-filename">{getFilename()}</span>
          <span class="preview-size">
            {(new Blob([exportPreview]).size / 1024).toFixed(1)} KB
          </span>
        </div>
        <pre class="preview-content">{exportPreview}</pre>
      </div>
    {/if}
  </SettingsSection>

  <!-- Export Actions -->
  <div class="export-actions">
    <Button
      variant="secondary"
      on:click={copyToClipboard}
      disabled={selectedCount === 0}
    >
      {#if copySuccess}
        <Icon name="check" size={16} />
        Copied!
      {:else}
        <Icon name="clipboard" size={16} />
        Copy to Clipboard
      {/if}
    </Button>

    <Button
      variant="primary"
      on:click={downloadExport}
      disabled={selectedCount === 0}
    >
      <Icon name="download" size={16} />
      Download {getFilename()}
    </Button>
  </div>
</div>

<style>
  .export-settings {
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

  .sensitive-notice,
  .sensitive-warning {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: 6px;
    font-size: 13px;
    margin-top: 12px;
  }

  .sensitive-notice {
    background: rgba(33, 150, 243, 0.1);
    color: var(--color-primary);
  }

  .sensitive-warning {
    background: rgba(255, 152, 0, 0.1);
    color: var(--color-warning);
  }

  /* Section Toolbar */
  .section-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .selection-count {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .selection-actions {
    display: flex;
    gap: 8px;
  }

  /* Export Sections */
  .export-sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .export-section {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary);
    border: 2px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .export-section:hover {
    background: var(--color-bg-hover);
  }

  .export-section--selected {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.05);
  }

  .export-section__checkbox {
    color: var(--color-text-muted);
  }

  .export-section--selected .export-section__checkbox {
    color: var(--color-primary);
  }

  .export-section__info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .export-section__label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  :global(.sensitive-icon) {
    color: var(--color-warning);
  }

  .export-section__desc {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  /* Preview */
  .preview-toggle {
    margin-bottom: 12px;
  }

  .preview-container {
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .preview-filename {
    font-size: 13px;
    font-family: monospace;
    color: var(--color-text-primary);
  }

  .preview-size {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .preview-content {
    margin: 0;
    padding: 12px;
    max-height: 300px;
    overflow: auto;
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    color: var(--color-text-primary);
    background: var(--color-bg-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }

  /* Export Actions */
  .export-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 24px;
    padding-top: 24px;
    border-top: 1px solid var(--color-border);
  }
</style>
```

---

## Testing Requirements

1. Section selection toggles work
2. Select all/none buttons function
3. Format selection changes output
4. Pretty print toggle works
5. Sensitive data exclusion works
6. Preview shows correct content
7. Download creates correct file
8. Copy to clipboard works

### Test File (src/lib/components/settings/__tests__/ExportSettings.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ExportSettings from '../ExportSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

describe('ExportSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders export options', () => {
    render(ExportSettings);

    expect(screen.getByText('Export Settings')).toBeInTheDocument();
    expect(screen.getByText('Format')).toBeInTheDocument();
    expect(screen.getByText('Pretty Print')).toBeInTheDocument();
  });

  it('shows all export sections', () => {
    render(ExportSettings);

    expect(screen.getByText('General Settings')).toBeInTheDocument();
    expect(screen.getByText('Appearance')).toBeInTheDocument();
    expect(screen.getByText('Editor Preferences')).toBeInTheDocument();
  });

  it('toggles section selection', async () => {
    render(ExportSettings);

    const generalSection = screen.getByText('General Settings').closest('.export-section');
    await fireEvent.click(generalSection!);

    expect(screen.getByText(/5 of 7 selected/)).toBeInTheDocument();
  });

  it('selects all sections', async () => {
    render(ExportSettings);

    const selectAll = screen.getByText('Select All');
    await fireEvent.click(selectAll);

    expect(screen.getByText(/7 of 7 selected/)).toBeInTheDocument();
  });

  it('shows preview when button clicked', async () => {
    render(ExportSettings);

    const previewButton = screen.getByText('Show Preview');
    await fireEvent.click(previewButton);

    expect(screen.getByText(/tachikoma-settings/)).toBeInTheDocument();
  });

  it('changes format to YAML', async () => {
    render(ExportSettings);

    const formatSelect = screen.getAllByRole('combobox')[0];
    await fireEvent.change(formatSelect, { target: { value: 'yaml' } });

    const previewButton = screen.getByText('Show Preview');
    await fireEvent.click(previewButton);

    expect(screen.getByText(/\.yaml/)).toBeInTheDocument();
  });

  it('shows sensitive data warning when enabled', async () => {
    render(ExportSettings);

    // Select a section with sensitive data (backends is selected by default)
    const toggle = screen.getByRole('switch', { name: /include sensitive data/i });
    await fireEvent.click(toggle);

    expect(screen.getByText(/sensitive credentials/)).toBeInTheDocument();
  });

  it('copies to clipboard', async () => {
    const mockClipboard = {
      writeText: vi.fn().mockResolvedValue(undefined),
    };
    Object.assign(navigator, { clipboard: mockClipboard });

    render(ExportSettings);

    const copyButton = screen.getByText('Copy to Clipboard');
    await fireEvent.click(copyButton);

    expect(mockClipboard.writeText).toHaveBeenCalled();
  });
});
```

---

## Related Specs

- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Depends on: [272-settings-store.md](272-settings-store.md)
- Previous: [280-settings-reset.md](280-settings-reset.md)
- Next: [282-settings-import.md](282-settings-import.md)
