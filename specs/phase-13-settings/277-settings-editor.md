# 277 - Code Editor Preferences Settings

**Phase:** 13 - Settings UI
**Spec ID:** 277
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~10% of model context window

---

## Objective

Create the Code Editor Settings panel that allows users to configure editor behavior including indentation, word wrap, line numbers, minimap, auto-save, formatting, and syntax highlighting options.

---

## Acceptance Criteria

- [ ] `EditorSettings.svelte` component with all editor options
- [ ] Tab size and spaces/tabs configuration
- [ ] Word wrap settings with column option
- [ ] Line numbers display mode
- [ ] Minimap toggle and scale
- [ ] Auto-save configuration
- [ ] Format on save/paste options
- [ ] Bracket pair colorization
- [ ] Whitespace rendering options
- [ ] Live preview panel

---

## Implementation Details

### 1. Editor Preview Component (src/lib/components/settings/EditorPreview.svelte)

```svelte
<script lang="ts">
  import { editorSettings } from '$lib/stores/settings-store';

  const sampleCode = `function fibonacci(n: number): number {
  if (n <= 1) return n;

  let prev = 0, curr = 1;
  for (let i = 2; i <= n; i++) {
    const next = prev + curr;
    prev = curr;
    curr = next;
  }

  return curr;
}

// Calculate the 10th Fibonacci number
const result = fibonacci(10);
console.log(\`Fibonacci(10) = \${result}\`);`;

  function getLineNumbers(): string[] {
    const lines = sampleCode.split('\n');
    if ($editorSettings.lineNumbers === 'off') return [];
    if ($editorSettings.lineNumbers === 'relative') {
      return lines.map((_, i) => {
        const center = Math.floor(lines.length / 2);
        return Math.abs(i - center).toString();
      });
    }
    return lines.map((_, i) => (i + 1).toString());
  }

  function renderWhitespace(text: string): string {
    if ($editorSettings.renderWhitespace === 'none') return text;

    const showAll = $editorSettings.renderWhitespace === 'all';
    const showBoundary = $editorSettings.renderWhitespace === 'boundary';
    const showTrailing = $editorSettings.renderWhitespace === 'trailing';

    let result = text;

    if (showAll || showBoundary) {
      // Show leading spaces
      result = result.replace(/^( +)/g, (match) =>
        '<span class="whitespace">' + '\u00B7'.repeat(match.length) + '</span>'
      );
    }

    if (showAll || showTrailing) {
      // Show trailing spaces
      result = result.replace(/( +)$/g, (match) =>
        '<span class="whitespace">' + '\u00B7'.repeat(match.length) + '</span>'
      );
    }

    if (showAll) {
      // Show tabs
      result = result.replace(/\t/g, '<span class="whitespace">\u2192</span>');
    }

    return result;
  }

  $: lineNumbers = getLineNumbers();
  $: lines = sampleCode.split('\n');
</script>

<div
  class="editor-preview"
  style="
    font-size: {$editorSettings.tabSize * 2 + 10}px;
    line-height: 1.5;
  "
>
  {#if $editorSettings.minimap}
    <div
      class="editor-preview__minimap"
      style="transform: scale({$editorSettings.minimapScale})"
    >
      {#each lines as line}
        <div class="minimap-line" style="width: {Math.min(line.length * 2, 100)}px" />
      {/each}
    </div>
  {/if}

  <div class="editor-preview__content">
    {#if $editorSettings.lineNumbers !== 'off'}
      <div class="editor-preview__gutter">
        {#each lineNumbers as num}
          <div class="line-number">{num}</div>
        {/each}
      </div>
    {/if}

    <div
      class="editor-preview__code"
      style="white-space: {$editorSettings.wordWrap === 'off' ? 'pre' : 'pre-wrap'}"
    >
      {#each lines as line, i}
        <div class="code-line">
          {@html renderWhitespace(line) || '&nbsp;'}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .editor-preview {
    position: relative;
    display: flex;
    height: 300px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
    font-family: 'JetBrains Mono', monospace;
  }

  .editor-preview__minimap {
    position: absolute;
    right: 8px;
    top: 8px;
    width: 60px;
    padding: 4px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
    transform-origin: top right;
  }

  .minimap-line {
    height: 2px;
    margin-bottom: 1px;
    background: var(--color-text-muted);
    opacity: 0.3;
    border-radius: 1px;
  }

  .editor-preview__content {
    display: flex;
    flex: 1;
    overflow: auto;
  }

  .editor-preview__gutter {
    padding: 12px 8px;
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    text-align: right;
    user-select: none;
  }

  .line-number {
    color: var(--color-text-muted);
    font-size: 0.85em;
    line-height: 1.5;
  }

  .editor-preview__code {
    flex: 1;
    padding: 12px;
    overflow-x: auto;
  }

  .code-line {
    line-height: 1.5;
    color: var(--color-text-primary);
  }

  :global(.whitespace) {
    color: var(--color-text-muted);
    opacity: 0.5;
  }
</style>
```

### 2. Editor Settings Component (src/lib/components/settings/EditorSettings.svelte)

```svelte
<script lang="ts">
  import { settingsStore, editorSettings } from '$lib/stores/settings-store';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Slider from '$lib/components/ui/Slider.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import EditorPreview from './EditorPreview.svelte';

  function handleToggle<K extends keyof typeof $editorSettings>(key: K) {
    return (event: CustomEvent<boolean>) => {
      settingsStore.updateSetting('editor', key, event.detail as any);
    };
  }

  function handleSelect<K extends keyof typeof $editorSettings>(key: K) {
    return (event: CustomEvent<string>) => {
      settingsStore.updateSetting('editor', key, event.detail as any);
    };
  }

  function handleNumber<K extends keyof typeof $editorSettings>(key: K) {
    return (event: CustomEvent<number>) => {
      settingsStore.updateSetting('editor', key, event.detail as any);
    };
  }
</script>

<div class="editor-settings">
  <h2 class="settings-title">Code Editor</h2>
  <p class="settings-description">
    Configure code editor behavior and appearance.
  </p>

  <!-- Preview -->
  <SettingsSection title="Preview">
    <EditorPreview />
  </SettingsSection>

  <!-- Indentation Section -->
  <SettingsSection title="Indentation">
    <SettingsRow
      label="Tab Size"
      description="Number of spaces per tab"
    >
      <Select
        value={$editorSettings.tabSize.toString()}
        options={[
          { value: '2', label: '2 spaces' },
          { value: '4', label: '4 spaces' },
          { value: '8', label: '8 spaces' },
        ]}
        on:change={(e) => settingsStore.updateSetting('editor', 'tabSize', parseInt(e.detail))}
      />
    </SettingsRow>

    <SettingsRow
      label="Insert Spaces"
      description="Use spaces instead of tabs for indentation"
    >
      <Toggle
        checked={$editorSettings.insertSpaces}
        on:change={handleToggle('insertSpaces')}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Text Display Section -->
  <SettingsSection title="Text Display">
    <SettingsRow
      label="Word Wrap"
      description="How to wrap long lines"
    >
      <Select
        value={$editorSettings.wordWrap}
        options={[
          { value: 'off', label: 'Off' },
          { value: 'on', label: 'On' },
          { value: 'wordWrapColumn', label: 'At Column' },
          { value: 'bounded', label: 'Bounded' },
        ]}
        on:change={handleSelect('wordWrap')}
      />
    </SettingsRow>

    {#if $editorSettings.wordWrap === 'wordWrapColumn' || $editorSettings.wordWrap === 'bounded'}
      <SettingsRow
        label="Word Wrap Column"
        description="Column at which to wrap text"
      >
        <div class="input-with-unit">
          <Input
            type="number"
            value={$editorSettings.wordWrapColumn}
            min={40}
            max={200}
            on:change={(e) => settingsStore.updateSetting('editor', 'wordWrapColumn', parseInt(e.target.value))}
          />
          <span class="unit">characters</span>
        </div>
      </SettingsRow>
    {/if}

    <SettingsRow
      label="Line Numbers"
      description="How to display line numbers"
    >
      <Select
        value={$editorSettings.lineNumbers}
        options={[
          { value: 'on', label: 'Absolute' },
          { value: 'relative', label: 'Relative' },
          { value: 'off', label: 'Off' },
        ]}
        on:change={handleSelect('lineNumbers')}
      />
    </SettingsRow>

    <SettingsRow
      label="Render Whitespace"
      description="When to show whitespace characters"
    >
      <Select
        value={$editorSettings.renderWhitespace}
        options={[
          { value: 'none', label: 'None' },
          { value: 'boundary', label: 'Boundary' },
          { value: 'selection', label: 'Selection' },
          { value: 'trailing', label: 'Trailing' },
          { value: 'all', label: 'All' },
        ]}
        on:change={handleSelect('renderWhitespace')}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Minimap Section -->
  <SettingsSection title="Minimap">
    <SettingsRow
      label="Show Minimap"
      description="Display a miniature overview of the code"
    >
      <Toggle
        checked={$editorSettings.minimap}
        on:change={handleToggle('minimap')}
      />
    </SettingsRow>

    {#if $editorSettings.minimap}
      <SettingsRow
        label="Minimap Scale"
        description="Size of the minimap relative to editor"
      >
        <div class="slider-with-value">
          <Slider
            min={0.5}
            max={2}
            step={0.25}
            value={$editorSettings.minimapScale}
            on:change={handleNumber('minimapScale')}
          />
          <span class="slider-value">{$editorSettings.minimapScale}x</span>
        </div>
      </SettingsRow>
    {/if}
  </SettingsSection>

  <!-- Brackets Section -->
  <SettingsSection title="Brackets">
    <SettingsRow
      label="Bracket Pair Colorization"
      description="Color matching brackets with different colors"
    >
      <Toggle
        checked={$editorSettings.bracketPairColorization}
        on:change={handleToggle('bracketPairColorization')}
      />
    </SettingsRow>

    <SettingsRow
      label="Auto Closing Brackets"
      description="Automatically insert closing brackets"
    >
      <Select
        value={$editorSettings.autoClosingBrackets}
        options={[
          { value: 'always', label: 'Always' },
          { value: 'languageDefined', label: 'Language Defined' },
          { value: 'never', label: 'Never' },
        ]}
        on:change={handleSelect('autoClosingBrackets')}
      />
    </SettingsRow>

    <SettingsRow
      label="Auto Closing Quotes"
      description="Automatically insert closing quotes"
    >
      <Select
        value={$editorSettings.autoClosingQuotes}
        options={[
          { value: 'always', label: 'Always' },
          { value: 'languageDefined', label: 'Language Defined' },
          { value: 'never', label: 'Never' },
        ]}
        on:change={handleSelect('autoClosingQuotes')}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Saving Section -->
  <SettingsSection title="Saving">
    <SettingsRow
      label="Auto Save"
      description="When to automatically save files"
    >
      <Select
        value={$editorSettings.autoSave}
        options={[
          { value: 'off', label: 'Off' },
          { value: 'afterDelay', label: 'After Delay' },
          { value: 'onFocusChange', label: 'On Focus Change' },
          { value: 'onWindowChange', label: 'On Window Change' },
        ]}
        on:change={handleSelect('autoSave')}
      />
    </SettingsRow>

    {#if $editorSettings.autoSave === 'afterDelay'}
      <SettingsRow
        label="Auto Save Delay"
        description="Delay before auto-saving"
      >
        <div class="slider-with-value">
          <Slider
            min={500}
            max={10000}
            step={500}
            value={$editorSettings.autoSaveDelay}
            on:change={handleNumber('autoSaveDelay')}
          />
          <span class="slider-value">{($editorSettings.autoSaveDelay / 1000).toFixed(1)}s</span>
        </div>
      </SettingsRow>
    {/if}
  </SettingsSection>

  <!-- Formatting Section -->
  <SettingsSection title="Formatting">
    <SettingsRow
      label="Format On Save"
      description="Automatically format code when saving"
    >
      <Toggle
        checked={$editorSettings.formatOnSave}
        on:change={handleToggle('formatOnSave')}
      />
    </SettingsRow>

    <SettingsRow
      label="Format On Paste"
      description="Automatically format pasted code"
    >
      <Toggle
        checked={$editorSettings.formatOnPaste}
        on:change={handleToggle('formatOnPaste')}
      />
    </SettingsRow>
  </SettingsSection>
</div>

<style>
  .editor-settings {
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

  .input-with-unit {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .input-with-unit :global(input) {
    width: 100px;
  }

  .unit {
    font-size: 13px;
    color: var(--color-text-secondary);
  }
</style>
```

---

## Testing Requirements

1. All editor settings render correctly
2. Tab size selection updates store
3. Word wrap options work correctly
4. Minimap toggle and scale work
5. Auto-save settings persist
6. Format options toggle correctly
7. Preview reflects setting changes
8. Whitespace rendering options work

### Test File (src/lib/components/settings/__tests__/EditorSettings.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import EditorSettings from '../EditorSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

describe('EditorSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders all editor sections', () => {
    render(EditorSettings);

    expect(screen.getByText('Indentation')).toBeInTheDocument();
    expect(screen.getByText('Text Display')).toBeInTheDocument();
    expect(screen.getByText('Minimap')).toBeInTheDocument();
    expect(screen.getByText('Brackets')).toBeInTheDocument();
    expect(screen.getByText('Saving')).toBeInTheDocument();
    expect(screen.getByText('Formatting')).toBeInTheDocument();
  });

  it('changes tab size', async () => {
    render(EditorSettings);

    const select = screen.getAllByRole('combobox')[0];
    await fireEvent.change(select, { target: { value: '4' } });

    const state = get(settingsStore);
    expect(state.settings.editor.tabSize).toBe(4);
  });

  it('toggles insert spaces', async () => {
    render(EditorSettings);

    const toggle = screen.getByRole('switch', { name: /insert spaces/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.editor.insertSpaces).toBe(false);
  });

  it('changes word wrap mode', async () => {
    render(EditorSettings);

    const select = screen.getAllByRole('combobox')[1]; // Word wrap select
    await fireEvent.change(select, { target: { value: 'off' } });

    const state = get(settingsStore);
    expect(state.settings.editor.wordWrap).toBe('off');
  });

  it('shows word wrap column when needed', async () => {
    render(EditorSettings);

    const select = screen.getAllByRole('combobox')[1];
    await fireEvent.change(select, { target: { value: 'wordWrapColumn' } });

    expect(screen.getByText('Word Wrap Column')).toBeInTheDocument();
  });

  it('toggles minimap', async () => {
    render(EditorSettings);

    const toggle = screen.getByRole('switch', { name: /show minimap/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.editor.minimap).toBe(false);
  });

  it('changes auto-save mode', async () => {
    render(EditorSettings);

    const select = screen.getAllByRole('combobox').find(s =>
      s.querySelector('option[value="afterDelay"]')
    );
    await fireEvent.change(select!, { target: { value: 'onFocusChange' } });

    const state = get(settingsStore);
    expect(state.settings.editor.autoSave).toBe('onFocusChange');
  });

  it('toggles format on save', async () => {
    render(EditorSettings);

    const toggle = screen.getByRole('switch', { name: /format on save/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.editor.formatOnSave).toBe(false);
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
- Previous: [276-settings-keybindings.md](276-settings-keybindings.md)
- Next: [278-settings-git.md](278-settings-git.md)
