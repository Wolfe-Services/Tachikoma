# 236 - Spec Browser Layout

**Phase:** 11 - Spec Browser UI
**Spec ID:** 236
**Status:** Planned
**Dependencies:** 004-svelte-integration, 005-ipc-bridge
**Estimated Context:** ~14% of Sonnet window

---

## Objective

Create the main Spec Browser layout component that provides a three-panel interface for navigating, viewing, and editing specification files with responsive design.

---

## Acceptance Criteria

- [x] Three-panel layout: navigation tree, content viewer, metadata panel
- [x] Collapsible panels with resizable widths
- [x] Breadcrumb navigation
- [x] Keyboard shortcuts for navigation
- [x] Dark/light theme support
- [x] Mobile-responsive design

---

## Implementation Details

### 1. Types (src/lib/types/spec-browser-layout.ts)

```typescript
export interface SpecBrowserLayoutState {
  navPanelCollapsed: boolean;
  metadataPanelCollapsed: boolean;
  navPanelWidth: number;
  metadataPanelWidth: number;
  activePanel: 'nav' | 'content' | 'metadata';
  currentSpecId: string | null;
  viewMode: 'view' | 'edit' | 'split';
}

export interface SpecBrowserConfig {
  navPanel: PanelConfig;
  metadataPanel: PanelConfig;
  defaultViewMode: 'view' | 'edit' | 'split';
  showLineNumbers: boolean;
  syntaxHighlight: boolean;
}

export interface PanelConfig {
  minWidth: number;
  maxWidth: number;
  defaultWidth: number;
  collapsible: boolean;
}

export const DEFAULT_BROWSER_CONFIG: SpecBrowserConfig = {
  navPanel: { minWidth: 200, maxWidth: 400, defaultWidth: 260, collapsible: true },
  metadataPanel: { minWidth: 220, maxWidth: 400, defaultWidth: 280, collapsible: true },
  defaultViewMode: 'view',
  showLineNumbers: true,
  syntaxHighlight: true,
};
```

### 2. Layout Store (src/lib/stores/spec-browser-layout-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { SpecBrowserLayoutState } from '$lib/types/spec-browser-layout';
import { DEFAULT_BROWSER_CONFIG } from '$lib/types/spec-browser-layout';

function createSpecBrowserLayoutStore() {
  const initialState: SpecBrowserLayoutState = {
    navPanelCollapsed: false,
    metadataPanelCollapsed: false,
    navPanelWidth: DEFAULT_BROWSER_CONFIG.navPanel.defaultWidth,
    metadataPanelWidth: DEFAULT_BROWSER_CONFIG.metadataPanel.defaultWidth,
    activePanel: 'content',
    currentSpecId: null,
    viewMode: 'view',
  };

  const { subscribe, set, update } = writable<SpecBrowserLayoutState>(initialState);

  return {
    subscribe,

    toggleNavPanel: () => update(s => ({ ...s, navPanelCollapsed: !s.navPanelCollapsed })),
    toggleMetadataPanel: () => update(s => ({ ...s, metadataPanelCollapsed: !s.metadataPanelCollapsed })),

    setNavPanelWidth: (width: number) => update(s => ({
      ...s,
      navPanelWidth: Math.max(
        DEFAULT_BROWSER_CONFIG.navPanel.minWidth,
        Math.min(DEFAULT_BROWSER_CONFIG.navPanel.maxWidth, width)
      ),
    })),

    setMetadataPanelWidth: (width: number) => update(s => ({
      ...s,
      metadataPanelWidth: Math.max(
        DEFAULT_BROWSER_CONFIG.metadataPanel.minWidth,
        Math.min(DEFAULT_BROWSER_CONFIG.metadataPanel.maxWidth, width)
      ),
    })),

    setActivePanel: (panel: 'nav' | 'content' | 'metadata') =>
      update(s => ({ ...s, activePanel: panel })),

    setCurrentSpec: (specId: string | null) =>
      update(s => ({ ...s, currentSpecId: specId })),

    setViewMode: (mode: 'view' | 'edit' | 'split') =>
      update(s => ({ ...s, viewMode: mode })),

    reset: () => set(initialState),
  };
}

export const specBrowserLayoutStore = createSpecBrowserLayoutStore();
```

### 3. Spec Browser Layout Component (src/lib/components/spec-browser/SpecBrowserLayout.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { specBrowserLayoutStore } from '$lib/stores/spec-browser-layout-store';
  import ResizeHandle from '$lib/components/mission/ResizeHandle.svelte';
  import SpecTreeNav from './SpecTreeNav.svelte';
  import SpecFileViewer from './SpecFileViewer.svelte';
  import SpecMetadata from './SpecMetadata.svelte';
  import SpecBreadcrumbs from './SpecBreadcrumbs.svelte';

  export let initialSpecId: string | null = null;

  $: if (initialSpecId) {
    specBrowserLayoutStore.setCurrentSpec(initialSpecId);
  }

  function handleKeyDown(event: KeyboardEvent) {
    const isMod = event.metaKey || event.ctrlKey;

    // Toggle nav: Cmd+B
    if (isMod && event.key === 'b') {
      event.preventDefault();
      specBrowserLayoutStore.toggleNavPanel();
    }

    // Toggle metadata: Cmd+M
    if (isMod && event.key === 'm') {
      event.preventDefault();
      specBrowserLayoutStore.toggleMetadataPanel();
    }

    // View mode: Cmd+1/2/3
    if (isMod && ['1', '2', '3'].includes(event.key)) {
      event.preventDefault();
      const modes: Array<'view' | 'edit' | 'split'> = ['view', 'edit', 'split'];
      specBrowserLayoutStore.setViewMode(modes[parseInt(event.key) - 1]);
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  $: navWidth = $specBrowserLayoutStore.navPanelCollapsed ? 0 : $specBrowserLayoutStore.navPanelWidth;
  $: metadataWidth = $specBrowserLayoutStore.metadataPanelCollapsed ? 0 : $specBrowserLayoutStore.metadataPanelWidth;
</script>

<div class="spec-browser" role="region" aria-label="Specification Browser">
  <!-- Navigation Panel -->
  {#if !$specBrowserLayoutStore.navPanelCollapsed}
    <aside
      class="spec-browser__nav"
      style="width: {navWidth}px"
      role="navigation"
      aria-label="Specification tree"
    >
      <slot name="nav">
        <SpecTreeNav
          selectedId={$specBrowserLayoutStore.currentSpecId}
          on:select={(e) => specBrowserLayoutStore.setCurrentSpec(e.detail)}
        />
      </slot>

      <ResizeHandle
        direction="horizontal"
        position="right"
        on:resize={(e) => specBrowserLayoutStore.setNavPanelWidth(navWidth + e.detail.delta)}
      />
    </aside>
  {/if}

  <!-- Main Content -->
  <main class="spec-browser__content">
    <header class="spec-browser__header">
      <button
        class="panel-toggle"
        on:click={() => specBrowserLayoutStore.toggleNavPanel()}
        aria-label="Toggle navigation"
      >
        {$specBrowserLayoutStore.navPanelCollapsed ? '→' : '←'}
      </button>

      <SpecBreadcrumbs specId={$specBrowserLayoutStore.currentSpecId} />

      <div class="view-mode-toggle">
        <button
          class:active={$specBrowserLayoutStore.viewMode === 'view'}
          on:click={() => specBrowserLayoutStore.setViewMode('view')}
        >
          View
        </button>
        <button
          class:active={$specBrowserLayoutStore.viewMode === 'edit'}
          on:click={() => specBrowserLayoutStore.setViewMode('edit')}
        >
          Edit
        </button>
        <button
          class:active={$specBrowserLayoutStore.viewMode === 'split'}
          on:click={() => specBrowserLayoutStore.setViewMode('split')}
        >
          Split
        </button>
      </div>

      <button
        class="panel-toggle"
        on:click={() => specBrowserLayoutStore.toggleMetadataPanel()}
        aria-label="Toggle metadata"
      >
        {$specBrowserLayoutStore.metadataPanelCollapsed ? '←' : '→'}
      </button>
    </header>

    <div class="spec-browser__viewer">
      <slot name="viewer">
        <SpecFileViewer
          specId={$specBrowserLayoutStore.currentSpecId}
          viewMode={$specBrowserLayoutStore.viewMode}
        />
      </slot>
    </div>
  </main>

  <!-- Metadata Panel -->
  {#if !$specBrowserLayoutStore.metadataPanelCollapsed}
    <aside
      class="spec-browser__metadata"
      style="width: {metadataWidth}px"
      role="complementary"
      aria-label="Specification metadata"
    >
      <ResizeHandle
        direction="horizontal"
        position="left"
        on:resize={(e) => specBrowserLayoutStore.setMetadataPanelWidth(metadataWidth + e.detail.delta)}
      />

      <slot name="metadata">
        <SpecMetadata specId={$specBrowserLayoutStore.currentSpecId} />
      </slot>
    </aside>
  {/if}
</div>

<style>
  .spec-browser {
    display: flex;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--color-bg-primary);
  }

  .spec-browser__nav {
    position: relative;
    flex-shrink: 0;
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    overflow: hidden;
  }

  .spec-browser__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .spec-browser__header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .panel-toggle {
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 14px;
    cursor: pointer;
    border-radius: 4px;
  }

  .panel-toggle:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .view-mode-toggle {
    display: flex;
    gap: 2px;
    background: var(--color-bg-hover);
    padding: 2px;
    border-radius: 6px;
    margin-left: auto;
  }

  .view-mode-toggle button {
    padding: 6px 12px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    border-radius: 4px;
    cursor: pointer;
  }

  .view-mode-toggle button.active {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .spec-browser__viewer {
    flex: 1;
    overflow: hidden;
  }

  .spec-browser__metadata {
    position: relative;
    flex-shrink: 0;
    background: var(--color-bg-secondary);
    border-left: 1px solid var(--color-border);
    overflow: hidden;
  }
</style>
```

---

## Testing Requirements

1. Layout renders with three panels
2. Panel toggle works correctly
3. Resize handles adjust widths
4. Keyboard shortcuts function
5. View mode toggle works
6. Responsive design adapts

---

## Related Specs

- Depends on: [004-svelte-integration.md](../phase-00-setup/004-svelte-integration.md)
- Next: [237-spec-tree-nav.md](237-spec-tree-nav.md)
- Used by: All Spec Browser UI specs (237-255)
