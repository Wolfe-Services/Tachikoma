# 219 - Prompt Editor Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 219
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~13% of Sonnet window

---

## Objective

Create a rich prompt editor component with syntax highlighting, variable interpolation, snippet support, and intelligent autocompletion for mission prompts.

---

## Acceptance Criteria

- [x] Textarea with auto-resize functionality
- [x] Syntax highlighting for code blocks
- [x] Variable interpolation with `{{variable}}` syntax
- [x] Snippet insertion via slash commands
- [x] Character count and token estimation
- [x] Markdown preview toggle
- [x] Undo/redo support
- [x] Keyboard shortcuts for formatting

---

## Implementation Details

### 1. Types (src/lib/types/prompt-editor.ts)

```typescript
/**
 * Types for prompt editor functionality.
 */

export interface PromptVariable {
  name: string;
  value: string;
  description: string;
  source: 'system' | 'user' | 'spec';
}

export interface PromptSnippet {
  id: string;
  trigger: string;
  name: string;
  description: string;
  content: string;
  category: string;
}

export interface PromptEditorState {
  value: string;
  cursorPosition: number;
  selectionStart: number;
  selectionEnd: number;
  history: HistoryEntry[];
  historyIndex: number;
  showPreview: boolean;
  showVariables: boolean;
  showSnippets: boolean;
}

export interface HistoryEntry {
  value: string;
  cursorPosition: number;
  timestamp: number;
}

export interface TokenEstimate {
  inputTokens: number;
  estimatedCost: number;
  modelContext: number;
  usagePercent: number;
}

export const DEFAULT_SNIPPETS: PromptSnippet[] = [
  {
    id: 'implement',
    trigger: '/implement',
    name: 'Implementation Request',
    description: 'Request to implement a feature',
    content: `Implement the following:\n\n**Feature**: \n**Requirements**:\n- \n\n**Constraints**:\n- `,
    category: 'feature',
  },
  {
    id: 'fix',
    trigger: '/fix',
    name: 'Bug Fix Request',
    description: 'Request to fix a bug',
    content: `Fix the following bug:\n\n**Description**: \n**Steps to Reproduce**:\n1. \n\n**Expected Behavior**: \n**Actual Behavior**: `,
    category: 'bugfix',
  },
  {
    id: 'refactor',
    trigger: '/refactor',
    name: 'Refactoring Request',
    description: 'Request to refactor code',
    content: `Refactor the following:\n\n**Target**: \n**Goals**:\n- \n\n**Keep unchanged**:\n- `,
    category: 'refactor',
  },
  {
    id: 'test',
    trigger: '/test',
    name: 'Test Writing Request',
    description: 'Request to write tests',
    content: `Write tests for:\n\n**Target**: \n**Test Types**:\n- [ ] Unit tests\n- [ ] Integration tests\n\n**Coverage Goals**: `,
    category: 'test',
  },
  {
    id: 'review',
    trigger: '/review',
    name: 'Code Review Request',
    description: 'Request for code review',
    content: `Review the following code:\n\n**Focus Areas**:\n- Correctness\n- Performance\n- Security\n- Maintainability\n\n**Code**:\n\`\`\`\n\n\`\`\``,
    category: 'review',
  },
];

export const SYSTEM_VARIABLES: PromptVariable[] = [
  { name: 'project_name', value: '', description: 'Current project name', source: 'system' },
  { name: 'current_date', value: '', description: 'Current date', source: 'system' },
  { name: 'selected_files', value: '', description: 'Currently selected files', source: 'system' },
  { name: 'git_branch', value: '', description: 'Current git branch', source: 'system' },
];
```

### 2. Prompt Editor Store (src/lib/stores/prompt-editor-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { PromptEditorState, HistoryEntry, TokenEstimate } from '$lib/types/prompt-editor';

const MAX_HISTORY = 100;

function createPromptEditorStore() {
  const initialState: PromptEditorState = {
    value: '',
    cursorPosition: 0,
    selectionStart: 0,
    selectionEnd: 0,
    history: [],
    historyIndex: -1,
    showPreview: false,
    showVariables: false,
    showSnippets: false,
  };

  const { subscribe, set, update } = writable<PromptEditorState>(initialState);

  function pushHistory(value: string, cursorPosition: number) {
    update(state => {
      // Don't push if value hasn't changed
      if (state.history.length > 0 && state.history[state.historyIndex]?.value === value) {
        return state;
      }

      const newHistory = state.history.slice(0, state.historyIndex + 1);
      newHistory.push({ value, cursorPosition, timestamp: Date.now() });

      // Limit history size
      if (newHistory.length > MAX_HISTORY) {
        newHistory.shift();
      }

      return {
        ...state,
        history: newHistory,
        historyIndex: newHistory.length - 1,
      };
    });
  }

  return {
    subscribe,

    setValue(value: string, cursorPosition?: number) {
      update(state => {
        const newState = {
          ...state,
          value,
          cursorPosition: cursorPosition ?? value.length,
        };
        return newState;
      });
      pushHistory(value, cursorPosition ?? value.length);
    },

    setCursor(position: number) {
      update(state => ({ ...state, cursorPosition: position }));
    },

    setSelection(start: number, end: number) {
      update(state => ({
        ...state,
        selectionStart: start,
        selectionEnd: end,
        cursorPosition: end,
      }));
    },

    insertAt(position: number, text: string) {
      update(state => {
        const before = state.value.slice(0, position);
        const after = state.value.slice(position);
        const newValue = before + text + after;
        const newPosition = position + text.length;

        pushHistory(newValue, newPosition);

        return {
          ...state,
          value: newValue,
          cursorPosition: newPosition,
        };
      });
    },

    replaceSelection(text: string) {
      update(state => {
        const before = state.value.slice(0, state.selectionStart);
        const after = state.value.slice(state.selectionEnd);
        const newValue = before + text + after;
        const newPosition = state.selectionStart + text.length;

        pushHistory(newValue, newPosition);

        return {
          ...state,
          value: newValue,
          cursorPosition: newPosition,
          selectionStart: newPosition,
          selectionEnd: newPosition,
        };
      });
    },

    undo() {
      update(state => {
        if (state.historyIndex <= 0) return state;

        const newIndex = state.historyIndex - 1;
        const entry = state.history[newIndex];

        return {
          ...state,
          value: entry.value,
          cursorPosition: entry.cursorPosition,
          historyIndex: newIndex,
        };
      });
    },

    redo() {
      update(state => {
        if (state.historyIndex >= state.history.length - 1) return state;

        const newIndex = state.historyIndex + 1;
        const entry = state.history[newIndex];

        return {
          ...state,
          value: entry.value,
          cursorPosition: entry.cursorPosition,
          historyIndex: newIndex,
        };
      });
    },

    togglePreview() {
      update(state => ({ ...state, showPreview: !state.showPreview }));
    },

    toggleVariables() {
      update(state => ({ ...state, showVariables: !state.showVariables }));
    },

    toggleSnippets() {
      update(state => ({ ...state, showSnippets: !state.showSnippets }));
    },

    reset() {
      set(initialState);
    },
  };
}

export const promptEditorStore = createPromptEditorStore();

// Token estimation (rough approximation: ~4 chars per token)
export const tokenEstimate = derived(promptEditorStore, $state => {
  const charCount = $state.value.length;
  const estimatedTokens = Math.ceil(charCount / 4);
  const modelContext = 200000; // Claude 3 Sonnet context
  const inputCostPer1k = 0.003;

  return {
    inputTokens: estimatedTokens,
    estimatedCost: (estimatedTokens / 1000) * inputCostPer1k,
    modelContext,
    usagePercent: (estimatedTokens / modelContext) * 100,
  } as TokenEstimate;
});
```

### 3. Prompt Editor Component (src/lib/components/mission/PromptEditor.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, tick } from 'svelte';
  import { promptEditorStore, tokenEstimate } from '$lib/stores/prompt-editor-store';
  import { DEFAULT_SNIPPETS, SYSTEM_VARIABLES } from '$lib/types/prompt-editor';
  import type { PromptSnippet, PromptVariable } from '$lib/types/prompt-editor';
  import SnippetPicker from './SnippetPicker.svelte';
  import VariablePicker from './VariablePicker.svelte';
  import MarkdownPreview from './MarkdownPreview.svelte';

  export let value = '';
  export let placeholder = 'Describe your mission...';
  export let minHeight = 200;
  export let maxHeight = 500;
  export let disabled = false;

  const dispatch = createEventDispatcher<{
    change: string;
    submit: string;
  }>();

  let textareaRef: HTMLTextAreaElement;
  let showSnippetPicker = false;
  let showVariablePicker = false;
  let snippetFilter = '';

  // Sync external value with store
  $: if (value !== $promptEditorStore.value) {
    promptEditorStore.setValue(value);
  }

  // Emit changes
  $: dispatch('change', $promptEditorStore.value);

  function handleInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    const newValue = target.value;
    const cursorPos = target.selectionStart;

    promptEditorStore.setValue(newValue, cursorPos);

    // Check for slash command
    const beforeCursor = newValue.slice(0, cursorPos);
    const slashMatch = beforeCursor.match(/\/(\w*)$/);

    if (slashMatch) {
      snippetFilter = slashMatch[1];
      showSnippetPicker = true;
    } else {
      showSnippetPicker = false;
    }

    // Check for variable interpolation
    const varMatch = beforeCursor.match(/\{\{(\w*)$/);
    if (varMatch) {
      showVariablePicker = true;
    } else {
      showVariablePicker = false;
    }

    autoResize();
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Undo: Cmd/Ctrl + Z
    if ((event.metaKey || event.ctrlKey) && event.key === 'z' && !event.shiftKey) {
      event.preventDefault();
      promptEditorStore.undo();
      syncTextarea();
    }

    // Redo: Cmd/Ctrl + Shift + Z or Cmd/Ctrl + Y
    if ((event.metaKey || event.ctrlKey) && (event.key === 'y' || (event.key === 'z' && event.shiftKey))) {
      event.preventDefault();
      promptEditorStore.redo();
      syncTextarea();
    }

    // Bold: Cmd/Ctrl + B
    if ((event.metaKey || event.ctrlKey) && event.key === 'b') {
      event.preventDefault();
      wrapSelection('**', '**');
    }

    // Italic: Cmd/Ctrl + I
    if ((event.metaKey || event.ctrlKey) && event.key === 'i') {
      event.preventDefault();
      wrapSelection('_', '_');
    }

    // Code: Cmd/Ctrl + E
    if ((event.metaKey || event.ctrlKey) && event.key === 'e') {
      event.preventDefault();
      wrapSelection('`', '`');
    }

    // Submit: Cmd/Ctrl + Enter
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
      event.preventDefault();
      dispatch('submit', $promptEditorStore.value);
    }

    // Toggle preview: Cmd/Ctrl + P
    if ((event.metaKey || event.ctrlKey) && event.key === 'p') {
      event.preventDefault();
      promptEditorStore.togglePreview();
    }

    // Handle snippet picker navigation
    if (showSnippetPicker) {
      if (event.key === 'Escape') {
        showSnippetPicker = false;
        event.preventDefault();
      }
    }
  }

  function wrapSelection(before: string, after: string) {
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selected = $promptEditorStore.value.slice(start, end);

    promptEditorStore.setSelection(start, end);
    promptEditorStore.replaceSelection(before + selected + after);

    tick().then(() => {
      textareaRef.selectionStart = start + before.length;
      textareaRef.selectionEnd = end + before.length;
      textareaRef.focus();
    });
  }

  function insertSnippet(snippet: PromptSnippet) {
    const cursorPos = textareaRef.selectionStart;
    const value = $promptEditorStore.value;

    // Find the slash command to replace
    const beforeCursor = value.slice(0, cursorPos);
    const slashIndex = beforeCursor.lastIndexOf('/');

    if (slashIndex !== -1) {
      const before = value.slice(0, slashIndex);
      const after = value.slice(cursorPos);
      const newValue = before + snippet.content + after;

      promptEditorStore.setValue(newValue, slashIndex + snippet.content.length);
    }

    showSnippetPicker = false;
    textareaRef.focus();
  }

  function insertVariable(variable: PromptVariable) {
    const cursorPos = textareaRef.selectionStart;
    const value = $promptEditorStore.value;

    // Find the {{ to replace
    const beforeCursor = value.slice(0, cursorPos);
    const openIndex = beforeCursor.lastIndexOf('{{');

    if (openIndex !== -1) {
      const before = value.slice(0, openIndex);
      const after = value.slice(cursorPos);
      const newValue = before + `{{${variable.name}}}` + after;

      promptEditorStore.setValue(newValue, openIndex + variable.name.length + 4);
    }

    showVariablePicker = false;
    textareaRef.focus();
  }

  function syncTextarea() {
    tick().then(() => {
      if (textareaRef) {
        textareaRef.value = $promptEditorStore.value;
        textareaRef.selectionStart = $promptEditorStore.cursorPosition;
        textareaRef.selectionEnd = $promptEditorStore.cursorPosition;
      }
    });
  }

  function autoResize() {
    if (!textareaRef) return;
    textareaRef.style.height = 'auto';
    const newHeight = Math.min(Math.max(textareaRef.scrollHeight, minHeight), maxHeight);
    textareaRef.style.height = `${newHeight}px`;
  }

  onMount(() => {
    if (value) {
      promptEditorStore.setValue(value);
    }
    autoResize();
  });
</script>

<div class="prompt-editor" class:prompt-editor--disabled={disabled}>
  <!-- Toolbar -->
  <div class="prompt-editor__toolbar">
    <div class="prompt-editor__toolbar-group">
      <button
        type="button"
        class="toolbar-btn"
        title="Bold (Cmd+B)"
        on:click={() => wrapSelection('**', '**')}
        {disabled}
      >
        <strong>B</strong>
      </button>
      <button
        type="button"
        class="toolbar-btn"
        title="Italic (Cmd+I)"
        on:click={() => wrapSelection('_', '_')}
        {disabled}
      >
        <em>I</em>
      </button>
      <button
        type="button"
        class="toolbar-btn"
        title="Code (Cmd+E)"
        on:click={() => wrapSelection('`', '`')}
        {disabled}
      >
        <code>&lt;/&gt;</code>
      </button>
    </div>

    <div class="prompt-editor__toolbar-group">
      <button
        type="button"
        class="toolbar-btn"
        title="Insert snippet"
        on:click={() => { showSnippetPicker = !showSnippetPicker; }}
        {disabled}
      >
        /
      </button>
      <button
        type="button"
        class="toolbar-btn"
        title="Insert variable"
        on:click={() => { showVariablePicker = !showVariablePicker; }}
        {disabled}
      >
        {"{ }"}
      </button>
    </div>

    <div class="prompt-editor__toolbar-group prompt-editor__toolbar-group--right">
      <button
        type="button"
        class="toolbar-btn"
        class:toolbar-btn--active={$promptEditorStore.showPreview}
        title="Toggle preview (Cmd+P)"
        on:click={() => promptEditorStore.togglePreview()}
        {disabled}
      >
        Preview
      </button>
    </div>
  </div>

  <!-- Editor Area -->
  <div class="prompt-editor__content">
    {#if $promptEditorStore.showPreview}
      <div class="prompt-editor__preview">
        <MarkdownPreview content={$promptEditorStore.value} />
      </div>
    {:else}
      <textarea
        bind:this={textareaRef}
        class="prompt-editor__textarea"
        {placeholder}
        {disabled}
        value={$promptEditorStore.value}
        style="min-height: {minHeight}px; max-height: {maxHeight}px"
        on:input={handleInput}
        on:keydown={handleKeyDown}
        on:select={() => {
          promptEditorStore.setSelection(
            textareaRef.selectionStart,
            textareaRef.selectionEnd
          );
        }}
        aria-label="Mission prompt"
      />
    {/if}

    <!-- Snippet Picker Dropdown -->
    {#if showSnippetPicker}
      <div class="prompt-editor__dropdown">
        <SnippetPicker
          snippets={DEFAULT_SNIPPETS}
          filter={snippetFilter}
          on:select={(e) => insertSnippet(e.detail)}
          on:close={() => { showSnippetPicker = false; }}
        />
      </div>
    {/if}

    <!-- Variable Picker Dropdown -->
    {#if showVariablePicker}
      <div class="prompt-editor__dropdown">
        <VariablePicker
          variables={SYSTEM_VARIABLES}
          on:select={(e) => insertVariable(e.detail)}
          on:close={() => { showVariablePicker = false; }}
        />
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <div class="prompt-editor__footer">
    <span class="prompt-editor__stat">
      {$promptEditorStore.value.length} characters
    </span>
    <span class="prompt-editor__stat">
      ~{$tokenEstimate.inputTokens.toLocaleString()} tokens
    </span>
    <span class="prompt-editor__stat">
      ~${$tokenEstimate.estimatedCost.toFixed(4)} estimated
    </span>
    <span class="prompt-editor__stat prompt-editor__stat--usage">
      {$tokenEstimate.usagePercent.toFixed(1)}% of context
    </span>
  </div>
</div>

<style>
  .prompt-editor {
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: var(--color-bg-primary);
    overflow: hidden;
  }

  .prompt-editor--disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .prompt-editor__toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .prompt-editor__toolbar-group {
    display: flex;
    gap: 4px;
  }

  .prompt-editor__toolbar-group--right {
    margin-left: auto;
  }

  .toolbar-btn {
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .toolbar-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .toolbar-btn--active {
    background: var(--color-primary);
    color: white;
  }

  .prompt-editor__content {
    position: relative;
  }

  .prompt-editor__textarea {
    width: 100%;
    padding: 16px;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
    font-size: 14px;
    line-height: 1.6;
    resize: none;
    outline: none;
  }

  .prompt-editor__textarea::placeholder {
    color: var(--color-text-muted);
  }

  .prompt-editor__preview {
    min-height: 200px;
    max-height: 500px;
    padding: 16px;
    overflow-y: auto;
  }

  .prompt-editor__dropdown {
    position: absolute;
    left: 16px;
    top: 40px;
    z-index: 100;
    width: 300px;
    max-height: 300px;
    overflow-y: auto;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }

  .prompt-editor__footer {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 12px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .prompt-editor__stat {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .prompt-editor__stat--usage {
    margin-left: auto;
  }
</style>
```

### 4. Snippet Picker (src/lib/components/mission/SnippetPicker.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PromptSnippet } from '$lib/types/prompt-editor';

  export let snippets: PromptSnippet[];
  export let filter = '';

  const dispatch = createEventDispatcher<{
    select: PromptSnippet;
    close: void;
  }>();

  let selectedIndex = 0;

  $: filteredSnippets = snippets.filter(s =>
    s.trigger.toLowerCase().includes(filter.toLowerCase()) ||
    s.name.toLowerCase().includes(filter.toLowerCase())
  );

  $: selectedIndex = Math.min(selectedIndex, filteredSnippets.length - 1);

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filteredSnippets.length - 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (filteredSnippets[selectedIndex]) {
        dispatch('select', filteredSnippets[selectedIndex]);
      }
    } else if (event.key === 'Escape') {
      dispatch('close');
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="snippet-picker" role="listbox" aria-label="Snippets">
  {#if filteredSnippets.length === 0}
    <div class="snippet-picker__empty">No snippets found</div>
  {:else}
    {#each filteredSnippets as snippet, index}
      <button
        class="snippet-item"
        class:snippet-item--selected={index === selectedIndex}
        role="option"
        aria-selected={index === selectedIndex}
        on:click={() => dispatch('select', snippet)}
        on:mouseenter={() => { selectedIndex = index; }}
      >
        <span class="snippet-item__trigger">{snippet.trigger}</span>
        <span class="snippet-item__name">{snippet.name}</span>
        <span class="snippet-item__description">{snippet.description}</span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .snippet-picker {
    padding: 4px;
  }

  .snippet-picker__empty {
    padding: 16px;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .snippet-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    text-align: left;
    transition: background-color 0.1s ease;
  }

  .snippet-item:hover,
  .snippet-item--selected {
    background: var(--color-bg-hover);
  }

  .snippet-item__trigger {
    font-family: monospace;
    font-size: 13px;
    color: var(--color-primary);
    margin-bottom: 2px;
  }

  .snippet-item__name {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    margin-bottom: 2px;
  }

  .snippet-item__description {
    font-size: 12px;
    color: var(--color-text-secondary);
  }
</style>
```

---

## Testing Requirements

1. Auto-resize works correctly
2. Keyboard shortcuts function as expected
3. Undo/redo maintains proper history
4. Snippet picker appears on slash command
5. Variable picker appears on {{ input
6. Token estimation calculates correctly
7. Preview toggle works

### Test File (src/lib/components/mission/__tests__/PromptEditor.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import PromptEditor from '../PromptEditor.svelte';
import { promptEditorStore } from '$lib/stores/prompt-editor-store';

describe('PromptEditor', () => {
  beforeEach(() => {
    promptEditorStore.reset();
  });

  it('renders with placeholder', () => {
    render(PromptEditor, { placeholder: 'Enter prompt' });
    expect(screen.getByPlaceholderText('Enter prompt')).toBeInTheDocument();
  });

  it('emits change events on input', async () => {
    const { component } = render(PromptEditor);
    const handler = vi.fn();
    component.$on('change', handler);

    const textarea = screen.getByRole('textbox');
    await fireEvent.input(textarea, { target: { value: 'Test input' } });

    expect(handler).toHaveBeenCalled();
  });

  it('handles undo/redo', async () => {
    render(PromptEditor);

    const textarea = screen.getByRole('textbox') as HTMLTextAreaElement;
    await fireEvent.input(textarea, { target: { value: 'First' } });
    await fireEvent.input(textarea, { target: { value: 'Second' } });

    // Undo
    await fireEvent.keyDown(textarea, { key: 'z', ctrlKey: true });

    expect(textarea.value).toBe('First');
  });

  it('shows snippet picker on slash', async () => {
    render(PromptEditor);

    const textarea = screen.getByRole('textbox');
    await fireEvent.input(textarea, { target: { value: '/', selectionStart: 1 } });

    expect(screen.getByRole('listbox', { name: 'Snippets' })).toBeInTheDocument();
  });

  it('wraps selection with markdown', async () => {
    render(PromptEditor);

    const textarea = screen.getByRole('textbox') as HTMLTextAreaElement;
    textarea.value = 'Hello world';
    textarea.selectionStart = 0;
    textarea.selectionEnd = 5;

    const boldBtn = screen.getByTitle('Bold (Cmd+B)');
    await fireEvent.click(boldBtn);

    expect(textarea.value).toBe('**Hello** world');
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [220-spec-selector.md](220-spec-selector.md)
- Used by: [218-mission-creation.md](218-mission-creation.md)
