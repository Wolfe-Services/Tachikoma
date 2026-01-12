# Spec 210: Terminal Component

## Phase
Phase 9: UI Foundation

## Spec ID
210

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System
- Spec 190: IPC Store Bindings

## Estimated Context
~12%

---

## Objective

Implement a Terminal emulator component for Tachikoma that provides an interactive command-line interface within the application, supporting ANSI colors, scrollback, command history, and integration with the Rust backend PTY.

---

## Acceptance Criteria

- [ ] ANSI escape code rendering (colors, bold, etc.)
- [ ] Command input with history
- [ ] Scrollback buffer with configurable size
- [ ] Copy/paste support
- [ ] Selection and text highlighting
- [ ] Multiple terminal tabs
- [ ] Resize handling
- [ ] Integration with backend PTY
- [ ] Search within terminal output
- [ ] Clear command support
- [ ] Keyboard shortcuts (Ctrl+C, Ctrl+V, etc.)

---

## Implementation Details

### src/lib/components/ui/Terminal/Terminal.svelte

```svelte
<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher, tick } from 'svelte';
  import { cn } from '@utils/component';
  import { invoke } from '@ipc/invoke';
  import { subscribe } from '@ipc/events';
  import AnsiRenderer from './AnsiRenderer.svelte';

  export let id: string = crypto.randomUUID();
  export let initialCommand: string = '';
  export let workingDirectory: string = '';
  export let scrollbackLines: number = 10000;
  export let fontSize: number = 14;
  export let fontFamily: string = 'var(--font-mono)';
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    output: string;
    exit: { code: number };
    ready: void;
  }>();

  interface TerminalLine {
    id: string;
    content: string;
    timestamp: number;
  }

  let containerElement: HTMLElement;
  let inputElement: HTMLInputElement;
  let lines: TerminalLine[] = [];
  let currentInput = '';
  let commandHistory: string[] = [];
  let historyIndex = -1;
  let isReady = false;
  let unsubscribe: (() => void) | null = null;

  // ANSI color map
  const ansiColors: Record<string, string> = {
    '30': 'var(--color-fg-muted)',
    '31': '#ef4444',
    '32': '#22c55e',
    '33': '#eab308',
    '34': '#3b82f6',
    '35': '#a855f7',
    '36': 'var(--tachikoma-500)',
    '37': 'var(--color-fg-default)',
    '90': 'var(--color-fg-subtle)',
    '91': '#f87171',
    '92': '#4ade80',
    '93': '#fbbf24',
    '94': '#60a5fa',
    '95': '#c084fc',
    '96': 'var(--tachikoma-400)',
    '97': '#ffffff'
  };

  onMount(async () => {
    // Initialize terminal session
    try {
      await invoke('terminal_create', {
        id,
        workingDirectory: workingDirectory || undefined
      });

      // Subscribe to terminal output
      unsubscribe = await subscribe<{ id: string; data: string }>(
        'terminal:output',
        (event) => {
          if (event.id === id) {
            appendOutput(event.data);
          }
        }
      );

      isReady = true;
      dispatch('ready');

      // Execute initial command if provided
      if (initialCommand) {
        executeCommand(initialCommand);
      }

      // Focus input
      inputElement?.focus();
    } catch (error) {
      appendOutput(`Error initializing terminal: ${error}\n`);
    }
  });

  onDestroy(async () => {
    if (unsubscribe) {
      unsubscribe();
    }
    // Clean up terminal session
    try {
      await invoke('terminal_destroy', { id });
    } catch {
      // Ignore cleanup errors
    }
  });

  function appendOutput(data: string) {
    const newLines = data.split('\n').map((content, i) => ({
      id: `${Date.now()}-${i}`,
      content,
      timestamp: Date.now()
    }));

    lines = [...lines, ...newLines].slice(-scrollbackLines);

    // Auto-scroll to bottom
    tick().then(() => {
      if (containerElement) {
        containerElement.scrollTop = containerElement.scrollHeight;
      }
    });

    dispatch('output', data);
  }

  async function executeCommand(command: string) {
    if (!command.trim()) return;

    // Add to history
    if (commandHistory[commandHistory.length - 1] !== command) {
      commandHistory = [...commandHistory, command];
    }
    historyIndex = commandHistory.length;

    // Echo command
    appendOutput(`$ ${command}\n`);

    // Handle built-in commands
    if (command === 'clear') {
      lines = [];
      currentInput = '';
      return;
    }

    // Send to backend
    try {
      await invoke('terminal_execute', { id, command });
    } catch (error) {
      appendOutput(`Error: ${error}\n`);
    }

    currentInput = '';
  }

  function handleKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case 'Enter':
        event.preventDefault();
        executeCommand(currentInput);
        break;

      case 'ArrowUp':
        event.preventDefault();
        if (historyIndex > 0) {
          historyIndex--;
          currentInput = commandHistory[historyIndex];
        }
        break;

      case 'ArrowDown':
        event.preventDefault();
        if (historyIndex < commandHistory.length - 1) {
          historyIndex++;
          currentInput = commandHistory[historyIndex];
        } else {
          historyIndex = commandHistory.length;
          currentInput = '';
        }
        break;

      case 'c':
        if (event.ctrlKey) {
          // Send interrupt signal
          invoke('terminal_signal', { id, signal: 'SIGINT' });
        }
        break;

      case 'l':
        if (event.ctrlKey) {
          event.preventDefault();
          lines = [];
        }
        break;
    }
  }

  function handleContainerClick() {
    inputElement?.focus();
  }

  function handlePaste(event: ClipboardEvent) {
    event.preventDefault();
    const text = event.clipboardData?.getData('text');
    if (text) {
      currentInput += text;
    }
  }

  $: classes = cn(
    'terminal',
    className
  );
</script>

<div
  bind:this={containerElement}
  class={classes}
  style="--terminal-font-size: {fontSize}px; --terminal-font-family: {fontFamily};"
  on:click={handleContainerClick}
  role="textbox"
  tabindex="0"
>
  <div class="terminal-output">
    {#each lines as line (line.id)}
      <div class="terminal-line">
        <AnsiRenderer content={line.content} {ansiColors} />
      </div>
    {/each}
  </div>

  <div class="terminal-input-line">
    <span class="terminal-prompt">$</span>
    <input
      bind:this={inputElement}
      bind:value={currentInput}
      class="terminal-input"
      type="text"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      on:keydown={handleKeyDown}
      on:paste={handlePaste}
    />
  </div>
</div>

<style>
  .terminal {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--color-bg-base);
    color: var(--color-fg-default);
    font-family: var(--terminal-font-family);
    font-size: var(--terminal-font-size);
    line-height: 1.5;
    padding: var(--spacing-3);
    overflow-y: auto;
    cursor: text;
  }

  .terminal-output {
    flex: 1;
  }

  .terminal-line {
    white-space: pre-wrap;
    word-break: break-all;
    min-height: 1.5em;
  }

  .terminal-input-line {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .terminal-prompt {
    color: var(--tachikoma-500);
    font-weight: var(--font-bold);
    user-select: none;
  }

  .terminal-input {
    flex: 1;
    background: transparent;
    border: none;
    color: inherit;
    font: inherit;
    outline: none;
    padding: 0;
    caret-color: var(--tachikoma-500);
  }

  /* Selection styling */
  .terminal ::selection {
    background-color: var(--color-primary-muted);
  }
</style>
```

### src/lib/components/ui/Terminal/AnsiRenderer.svelte

```svelte
<script lang="ts">
  export let content: string;
  export let ansiColors: Record<string, string> = {};

  interface Span {
    text: string;
    style: string;
  }

  $: spans = parseAnsi(content);

  function parseAnsi(text: string): Span[] {
    const result: Span[] = [];
    const ansiRegex = /\x1b\[([0-9;]+)m/g;

    let lastIndex = 0;
    let currentStyles: string[] = [];
    let match;

    while ((match = ansiRegex.exec(text)) !== null) {
      // Add text before this match
      if (match.index > lastIndex) {
        result.push({
          text: text.slice(lastIndex, match.index),
          style: currentStyles.join('; ')
        });
      }

      // Parse ANSI codes
      const codes = match[1].split(';');
      for (const code of codes) {
        if (code === '0') {
          currentStyles = [];
        } else if (code === '1') {
          currentStyles.push('font-weight: bold');
        } else if (code === '3') {
          currentStyles.push('font-style: italic');
        } else if (code === '4') {
          currentStyles.push('text-decoration: underline');
        } else if (ansiColors[code]) {
          currentStyles.push(`color: ${ansiColors[code]}`);
        } else if (code.startsWith('4') && ansiColors[code.slice(1)]) {
          // Background colors (40-47, 100-107)
          const bgCode = code.startsWith('10') ? code.slice(2) : code.slice(1);
          if (ansiColors['3' + bgCode]) {
            currentStyles.push(`background-color: ${ansiColors['3' + bgCode]}`);
          }
        }
      }

      lastIndex = match.index + match[0].length;
    }

    // Add remaining text
    if (lastIndex < text.length) {
      result.push({
        text: text.slice(lastIndex),
        style: currentStyles.join('; ')
      });
    }

    return result;
  }
</script>

{#each spans as span}
  {#if span.style}
    <span style={span.style}>{span.text}</span>
  {:else}
    {span.text}
  {/if}
{/each}
```

### src/lib/components/ui/Terminal/TerminalTabs.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { cn } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';
  import Button from '../Button/Button.svelte';
  import Terminal from './Terminal.svelte';

  interface TerminalTab {
    id: string;
    title: string;
    workingDirectory?: string;
  }

  export let tabs: TerminalTab[] = [{ id: crypto.randomUUID(), title: 'Terminal 1' }];
  export let activeTabId: string = tabs[0]?.id;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    newTab: void;
    closeTab: string;
    tabChange: string;
  }>();

  function addTab() {
    const newTab: TerminalTab = {
      id: crypto.randomUUID(),
      title: `Terminal ${tabs.length + 1}`
    };
    tabs = [...tabs, newTab];
    activeTabId = newTab.id;
    dispatch('newTab');
  }

  function closeTab(tabId: string) {
    if (tabs.length === 1) return;

    const index = tabs.findIndex(t => t.id === tabId);
    tabs = tabs.filter(t => t.id !== tabId);

    if (activeTabId === tabId) {
      activeTabId = tabs[Math.max(0, index - 1)].id;
    }

    dispatch('closeTab', tabId);
  }

  function selectTab(tabId: string) {
    activeTabId = tabId;
    dispatch('tabChange', tabId);
  }
</script>

<div class={cn('terminal-tabs', className)}>
  <div class="terminal-tabs-header">
    <div class="terminal-tabs-list" role="tablist">
      {#each tabs as tab (tab.id)}
        <button
          class="terminal-tab"
          class:active={activeTabId === tab.id}
          role="tab"
          aria-selected={activeTabId === tab.id}
          on:click={() => selectTab(tab.id)}
        >
          <Icon name="terminal" size={14} />
          <span class="terminal-tab-title">{tab.title}</span>
          {#if tabs.length > 1}
            <button
              class="terminal-tab-close"
              on:click|stopPropagation={() => closeTab(tab.id)}
              aria-label="Close terminal"
            >
              <Icon name="x" size={12} />
            </button>
          {/if}
        </button>
      {/each}
    </div>

    <Button
      variant="ghost"
      size="sm"
      iconOnly
      on:click={addTab}
      aria-label="New terminal"
    >
      <Icon name="plus" size={16} />
    </Button>
  </div>

  <div class="terminal-tabs-content">
    {#each tabs as tab (tab.id)}
      <div
        class="terminal-tab-panel"
        class:active={activeTabId === tab.id}
        role="tabpanel"
        aria-hidden={activeTabId !== tab.id}
      >
        <Terminal
          id={tab.id}
          workingDirectory={tab.workingDirectory}
        />
      </div>
    {/each}
  </div>
</div>

<style>
  .terminal-tabs {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--color-bg-base);
  }

  .terminal-tabs-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-1) var(--spacing-2);
    background-color: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border-default);
  }

  .terminal-tabs-list {
    display: flex;
    flex: 1;
    gap: var(--spacing-1);
    overflow-x: auto;
  }

  .terminal-tab {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-1-5) var(--spacing-3);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-fg-muted);
    font-size: var(--text-sm);
    cursor: pointer;
    white-space: nowrap;
    transition: background-color var(--duration-150) var(--ease-out),
                color var(--duration-150) var(--ease-out);
  }

  .terminal-tab:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-fg-default);
  }

  .terminal-tab.active {
    background-color: var(--color-bg-elevated);
    color: var(--color-fg-default);
  }

  .terminal-tab-title {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .terminal-tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-0-5);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: inherit;
    cursor: pointer;
    opacity: 0.6;
    transition: opacity var(--duration-150) var(--ease-out),
                background-color var(--duration-150) var(--ease-out);
  }

  .terminal-tab-close:hover {
    opacity: 1;
    background-color: var(--color-bg-active);
  }

  .terminal-tabs-content {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .terminal-tab-panel {
    position: absolute;
    inset: 0;
    display: none;
  }

  .terminal-tab-panel.active {
    display: block;
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { Terminal, TerminalTabs } from '@components/ui';

  function handleOutput(event) {
    console.log('Terminal output:', event.detail);
  }
</script>

<!-- Single terminal -->
<Terminal
  workingDirectory="/home/user"
  on:output={handleOutput}
/>

<!-- Multiple terminals with tabs -->
<TerminalTabs />

<!-- Terminal with initial command -->
<Terminal
  initialCommand="ls -la"
  workingDirectory="/var/log"
/>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Terminal.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Terminal from '@components/ui/Terminal/Terminal.svelte';
import AnsiRenderer from '@components/ui/Terminal/AnsiRenderer.svelte';

// Mock IPC
vi.mock('@ipc/invoke', () => ({
  invoke: vi.fn().mockResolvedValue(undefined)
}));

vi.mock('@ipc/events', () => ({
  subscribe: vi.fn().mockResolvedValue(() => {})
}));

describe('Terminal', () => {
  it('should render terminal', () => {
    const { container } = render(Terminal);
    expect(container.querySelector('.terminal')).toBeInTheDocument();
  });

  it('should have prompt', () => {
    const { getByText } = render(Terminal);
    expect(getByText('$')).toBeInTheDocument();
  });

  it('should handle input', async () => {
    const { container } = render(Terminal);
    const input = container.querySelector('.terminal-input') as HTMLInputElement;

    await fireEvent.input(input, { target: { value: 'ls' } });
    expect(input.value).toBe('ls');
  });
});

describe('AnsiRenderer', () => {
  it('should render plain text', () => {
    const { container } = render(AnsiRenderer, {
      props: { content: 'Hello World', ansiColors: {} }
    });

    expect(container).toHaveTextContent('Hello World');
  });

  it('should render colored text', () => {
    const { container } = render(AnsiRenderer, {
      props: {
        content: '\x1b[31mRed Text\x1b[0m',
        ansiColors: { '31': '#ff0000' }
      }
    });

    const span = container.querySelector('span');
    expect(span).toHaveStyle({ color: '#ff0000' });
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [208-code-block.md](./208-code-block.md) - Code block component
- [190-ipc-store-bindings.md](./190-ipc-store-bindings.md) - IPC bindings
