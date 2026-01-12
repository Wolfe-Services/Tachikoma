# 238 - Spec File Viewer

**Phase:** 11 - Spec Browser UI
**Spec ID:** 238
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 239-markdown-renderer
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create a spec file viewer component that displays specification markdown files with syntax highlighting, rendered output, and support for view/edit/split modes.

---

## Acceptance Criteria

- [ ] View mode with rendered markdown
- [ ] Edit mode with syntax highlighting
- [ ] Split mode with side-by-side view
- [ ] Line numbers in edit mode
- [ ] Find and replace functionality
- [ ] Auto-save with debouncing
- [ ] Unsaved changes indicator

---

## Implementation Details

### 1. Types (src/lib/types/spec-viewer.ts)

```typescript
export interface SpecFile {
  id: string;
  path: string;
  content: string;
  frontmatter: SpecFrontmatter;
  lastModified: string;
  checksum: string;
}

export interface SpecFrontmatter {
  phase: number;
  specId: number;
  title: string;
  status: string;
  dependencies: string[];
  estimatedContext: string;
}

export interface ViewerState {
  specId: string | null;
  content: string;
  originalContent: string;
  isDirty: boolean;
  isSaving: boolean;
  lastSaved: string | null;
  cursorPosition: { line: number; column: number };
  scrollPosition: { top: number; left: number };
}
```

### 2. Spec File Viewer Component (src/lib/components/spec-browser/SpecFileViewer.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  import type { SpecFile, ViewerState } from '$lib/types/spec-viewer';
  import { ipcRenderer } from '$lib/ipc';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import CodeEditor from '$lib/components/common/CodeEditor.svelte';

  export let specId: string | null;
  export let viewMode: 'view' | 'edit' | 'split' = 'view';

  let specFile: SpecFile | null = null;
  let content = '';
  let originalContent = '';
  let isDirty = false;
  let isSaving = false;
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let editorRef: any;

  async function loadSpec() {
    if (!specId) {
      specFile = null;
      content = '';
      return;
    }

    try {
      specFile = await ipcRenderer.invoke('spec:read', specId);
      content = specFile.content;
      originalContent = specFile.content;
      isDirty = false;
    } catch (error) {
      console.error('Failed to load spec:', error);
    }
  }

  function handleContentChange(newContent: string) {
    content = newContent;
    isDirty = content !== originalContent;

    // Auto-save with debounce
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => saveContent(), 2000);
  }

  async function saveContent() {
    if (!specId || !isDirty || isSaving) return;

    isSaving = true;
    try {
      await ipcRenderer.invoke('spec:write', { specId, content });
      originalContent = content;
      isDirty = false;
    } catch (error) {
      console.error('Failed to save spec:', error);
    } finally {
      isSaving = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Save: Cmd+S
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault();
      saveContent();
    }
  }

  $: if (specId) loadSpec();

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
    if (saveTimeout) clearTimeout(saveTimeout);
  });
</script>

<div class="spec-file-viewer">
  {#if !specId}
    <div class="spec-file-viewer__empty">
      <p>Select a specification to view</p>
    </div>
  {:else if !specFile}
    <div class="spec-file-viewer__loading">
      Loading...
    </div>
  {:else}
    <!-- Status Bar -->
    <div class="spec-file-viewer__status">
      <span class="status-path">{specFile.path}</span>
      {#if isDirty}
        <span class="status-dirty">Modified</span>
      {/if}
      {#if isSaving}
        <span class="status-saving">Saving...</span>
      {/if}
    </div>

    <!-- Content Area -->
    <div class="spec-file-viewer__content" class:split={viewMode === 'split'}>
      {#if viewMode === 'view'}
        <div class="viewer-pane">
          <MarkdownRenderer content={content} />
        </div>
      {:else if viewMode === 'edit'}
        <div class="editor-pane">
          <CodeEditor
            bind:this={editorRef}
            value={content}
            language="markdown"
            showLineNumbers={true}
            on:change={(e) => handleContentChange(e.detail)}
          />
        </div>
      {:else}
        <div class="editor-pane">
          <CodeEditor
            bind:this={editorRef}
            value={content}
            language="markdown"
            showLineNumbers={true}
            on:change={(e) => handleContentChange(e.detail)}
          />
        </div>
        <div class="viewer-pane">
          <MarkdownRenderer content={content} />
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .spec-file-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .spec-file-viewer__empty,
  .spec-file-viewer__loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    font-size: 14px;
  }

  .spec-file-viewer__status {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    font-size: 12px;
  }

  .status-path {
    color: var(--color-text-secondary);
    font-family: monospace;
  }

  .status-dirty {
    color: var(--color-warning);
  }

  .status-saving {
    color: var(--color-primary);
  }

  .spec-file-viewer__content {
    flex: 1;
    overflow: hidden;
    display: flex;
  }

  .spec-file-viewer__content.split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px;
    background: var(--color-border);
  }

  .viewer-pane,
  .editor-pane {
    overflow: auto;
    background: var(--color-bg-primary);
  }

  .viewer-pane {
    padding: 24px;
  }
</style>
```

---

## Testing Requirements

1. File loads correctly
2. View/edit/split modes work
3. Auto-save triggers
4. Dirty indicator shows
5. Keyboard save works

---

## Related Specs

- Depends on: [236-spec-browser-layout.md](236-spec-browser-layout.md)
- Depends on: [239-markdown-renderer.md](239-markdown-renderer.md)
- Next: [239-markdown-renderer.md](239-markdown-renderer.md)
