# 240 - Spec Editor

**Phase:** 11 - Spec Browser UI
**Spec ID:** 240
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 238-spec-file-viewer
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create a rich spec editor component with markdown syntax highlighting, auto-completion for spec references, template snippets, and formatting toolbar.

---

## Acceptance Criteria

- [ ] Markdown syntax highlighting
- [ ] Line numbers with gutter
- [ ] Auto-completion for spec references
- [ ] Template snippet insertion
- [ ] Formatting toolbar
- [ ] Undo/redo with history
- [ ] Find and replace

---

## Implementation Details

### 1. Types (src/lib/types/spec-editor.ts)

```typescript
export interface EditorConfig {
  lineNumbers: boolean;
  wordWrap: boolean;
  tabSize: number;
  autoSave: boolean;
  autoSaveDelay: number;
  minimap: boolean;
}

export interface EditorState {
  content: string;
  cursorPosition: { line: number; column: number };
  selection: { start: Position; end: Position } | null;
  history: HistoryEntry[];
  historyIndex: number;
}

export interface Position {
  line: number;
  column: number;
}

export interface HistoryEntry {
  content: string;
  cursor: Position;
  timestamp: number;
}

export interface CompletionItem {
  label: string;
  kind: 'spec' | 'section' | 'template';
  detail: string;
  insertText: string;
}

export const DEFAULT_EDITOR_CONFIG: EditorConfig = {
  lineNumbers: true,
  wordWrap: true,
  tabSize: 2,
  autoSave: true,
  autoSaveDelay: 2000,
  minimap: false,
};
```

### 2. Spec Editor Component (src/lib/components/spec-browser/SpecEditor.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy, tick } from 'svelte';
  import type { EditorConfig, EditorState, CompletionItem, Position } from '$lib/types/spec-editor';
  import { DEFAULT_EDITOR_CONFIG } from '$lib/types/spec-editor';
  import EditorToolbar from './EditorToolbar.svelte';
  import CompletionMenu from './CompletionMenu.svelte';
  import FindReplace from './FindReplace.svelte';

  export let value: string;
  export let config: EditorConfig = DEFAULT_EDITOR_CONFIG;

  const dispatch = createEventDispatcher<{
    change: string;
    save: void;
  }>();

  let textareaRef: HTMLTextAreaElement;
  let lineNumbers: number[] = [];
  let showCompletion = false;
  let completionItems: CompletionItem[] = [];
  let completionPosition = { top: 0, left: 0 };
  let showFindReplace = false;
  let history: { content: string; cursor: number }[] = [];
  let historyIndex = -1;

  function updateLineNumbers() {
    const lines = value.split('\n').length;
    lineNumbers = Array.from({ length: lines }, (_, i) => i + 1);
  }

  function handleInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    value = target.value;
    updateLineNumbers();
    dispatch('change', value);
    checkForCompletion();
    pushHistory();
  }

  function handleKeyDown(event: KeyboardEvent) {
    const isMod = event.metaKey || event.ctrlKey;

    // Save: Cmd+S
    if (isMod && event.key === 's') {
      event.preventDefault();
      dispatch('save');
    }

    // Find: Cmd+F
    if (isMod && event.key === 'f') {
      event.preventDefault();
      showFindReplace = true;
    }

    // Undo: Cmd+Z
    if (isMod && event.key === 'z' && !event.shiftKey) {
      event.preventDefault();
      undo();
    }

    // Redo: Cmd+Shift+Z
    if (isMod && event.key === 'z' && event.shiftKey) {
      event.preventDefault();
      redo();
    }

    // Tab handling
    if (event.key === 'Tab') {
      event.preventDefault();
      insertText('  ');
    }

    // Completion navigation
    if (showCompletion) {
      if (event.key === 'Escape') {
        showCompletion = false;
      } else if (event.key === 'Enter') {
        event.preventDefault();
        // Insert selected completion
      }
    }
  }

  function insertText(text: string) {
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const before = value.slice(0, start);
    const after = value.slice(end);
    value = before + text + after;

    tick().then(() => {
      textareaRef.selectionStart = textareaRef.selectionEnd = start + text.length;
    });

    dispatch('change', value);
  }

  function wrapSelection(before: string, after: string) {
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selected = value.slice(start, end);
    const beforeText = value.slice(0, start);
    const afterText = value.slice(end);
    value = beforeText + before + selected + after + afterText;

    tick().then(() => {
      textareaRef.selectionStart = start + before.length;
      textareaRef.selectionEnd = end + before.length;
      textareaRef.focus();
    });

    dispatch('change', value);
  }

  function checkForCompletion() {
    const cursor = textareaRef.selectionStart;
    const textBeforeCursor = value.slice(0, cursor);

    // Check for spec reference pattern [[
    if (textBeforeCursor.endsWith('[[')) {
      showCompletionMenu('spec');
    }
    // Check for section reference #
    else if (textBeforeCursor.match(/\[.*\]\(#$/)) {
      showCompletionMenu('section');
    }
    // Check for template trigger /
    else if (textBeforeCursor.endsWith('/')) {
      showCompletionMenu('template');
    } else {
      showCompletion = false;
    }
  }

  function showCompletionMenu(kind: string) {
    // Get cursor position for menu placement
    const rect = textareaRef.getBoundingClientRect();
    completionPosition = {
      top: rect.top + 20,
      left: rect.left + 100,
    };

    // Load completion items based on kind
    completionItems = getCompletionItems(kind);
    showCompletion = true;
  }

  function getCompletionItems(kind: string): CompletionItem[] {
    // This would load from store/IPC
    return [
      { label: '216-mission-layout', kind: 'spec', detail: 'Mission Panel Layout', insertText: '216-mission-layout' },
      { label: '217-mission-state', kind: 'spec', detail: 'Mission State Management', insertText: '217-mission-state' },
    ];
  }

  function pushHistory() {
    history = history.slice(0, historyIndex + 1);
    history.push({ content: value, cursor: textareaRef.selectionStart });
    historyIndex = history.length - 1;

    if (history.length > 100) {
      history = history.slice(-100);
      historyIndex = history.length - 1;
    }
  }

  function undo() {
    if (historyIndex > 0) {
      historyIndex--;
      const entry = history[historyIndex];
      value = entry.content;
      tick().then(() => {
        textareaRef.selectionStart = textareaRef.selectionEnd = entry.cursor;
      });
    }
  }

  function redo() {
    if (historyIndex < history.length - 1) {
      historyIndex++;
      const entry = history[historyIndex];
      value = entry.content;
      tick().then(() => {
        textareaRef.selectionStart = textareaRef.selectionEnd = entry.cursor;
      });
    }
  }

  $: updateLineNumbers();

  onMount(() => {
    pushHistory();
  });
</script>

<div class="spec-editor">
  <EditorToolbar
    on:bold={() => wrapSelection('**', '**')}
    on:italic={() => wrapSelection('_', '_')}
    on:code={() => wrapSelection('`', '`')}
    on:codeBlock={() => wrapSelection('\n```\n', '\n```\n')}
    on:link={() => wrapSelection('[', ']()')}
    on:heading={() => insertText('## ')}
    on:bullet={() => insertText('- ')}
    on:checkbox={() => insertText('- [ ] ')}
    on:undo={undo}
    on:redo={redo}
    canUndo={historyIndex > 0}
    canRedo={historyIndex < history.length - 1}
  />

  <div class="spec-editor__content">
    {#if config.lineNumbers}
      <div class="spec-editor__gutter">
        {#each lineNumbers as num}
          <div class="line-number">{num}</div>
        {/each}
      </div>
    {/if}

    <textarea
      bind:this={textareaRef}
      bind:value
      class="spec-editor__textarea"
      class:word-wrap={config.wordWrap}
      spellcheck="false"
      on:input={handleInput}
      on:keydown={handleKeyDown}
    ></textarea>
  </div>

  {#if showCompletion}
    <CompletionMenu
      items={completionItems}
      position={completionPosition}
      on:select={(e) => {
        insertText(e.detail.insertText);
        showCompletion = false;
      }}
      on:close={() => { showCompletion = false; }}
    />
  {/if}

  {#if showFindReplace}
    <FindReplace
      {value}
      on:replace={(e) => { value = e.detail; dispatch('change', value); }}
      on:close={() => { showFindReplace = false; }}
    />
  {/if}
</div>

<style>
  .spec-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-primary);
  }

  .spec-editor__content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .spec-editor__gutter {
    width: 48px;
    padding: 16px 8px;
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    text-align: right;
    font-family: 'SF Mono', monospace;
    font-size: 13px;
    color: var(--color-text-muted);
    user-select: none;
    overflow: hidden;
  }

  .line-number {
    height: 20px;
    line-height: 20px;
  }

  .spec-editor__textarea {
    flex: 1;
    padding: 16px;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    font-family: 'SF Mono', monospace;
    font-size: 14px;
    line-height: 20px;
    resize: none;
    outline: none;
    white-space: pre;
  }

  .spec-editor__textarea.word-wrap {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>
```

---

## Testing Requirements

1. Syntax highlighting works
2. Line numbers display
3. Auto-completion triggers
4. Undo/redo works
5. Formatting toolbar functions
6. Find/replace works

---

## Related Specs

- Depends on: [238-spec-file-viewer.md](238-spec-file-viewer.md)
- Next: [241-impl-plan-viewer.md](241-impl-plan-viewer.md)
