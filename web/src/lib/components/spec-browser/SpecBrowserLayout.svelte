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

    // Toggle nav: Cmd/Ctrl+B
    if (isMod && event.key === 'b') {
      event.preventDefault();
      specBrowserLayoutStore.toggleNavPanel();
    }

    // Toggle metadata: Cmd/Ctrl+M
    if (isMod && event.key === 'm') {
      event.preventDefault();
      specBrowserLayoutStore.toggleMetadataPanel();
    }

    // View mode: Cmd/Ctrl+1/2/3
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
        title="{$specBrowserLayoutStore.navPanelCollapsed ? 'Show' : 'Hide'} navigation panel (Ctrl+B)"
      >
        {$specBrowserLayoutStore.navPanelCollapsed ? '→' : '←'}
      </button>

      <SpecBreadcrumbs specId={$specBrowserLayoutStore.currentSpecId} />

      <div class="view-mode-toggle">
        <button
          class:active={$specBrowserLayoutStore.viewMode === 'view'}
          on:click={() => specBrowserLayoutStore.setViewMode('view')}
          title="View mode (Ctrl+1)"
        >
          View
        </button>
        <button
          class:active={$specBrowserLayoutStore.viewMode === 'edit'}
          on:click={() => specBrowserLayoutStore.setViewMode('edit')}
          title="Edit mode (Ctrl+2)"
        >
          Edit
        </button>
        <button
          class:active={$specBrowserLayoutStore.viewMode === 'split'}
          on:click={() => specBrowserLayoutStore.setViewMode('split')}
          title="Split mode (Ctrl+3)"
        >
          Split
        </button>
      </div>

      <button
        class="panel-toggle"
        on:click={() => specBrowserLayoutStore.toggleMetadataPanel()}
        aria-label="Toggle metadata"
        title="{$specBrowserLayoutStore.metadataPanelCollapsed ? 'Show' : 'Hide'} metadata panel (Ctrl+M)"
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
        on:resize={(e) => specBrowserLayoutStore.setMetadataPanelWidth(metadataWidth - e.detail.delta)}
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
    background: var(--color-bg-base);
  }

  .spec-browser__nav {
    position: relative;
    flex-shrink: 0;
    background: var(--color-bg-surface);
    border-right: 1px solid var(--color-border-default);
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
    border-bottom: 1px solid var(--color-border-default);
    background: var(--color-bg-surface);
  }

  .panel-toggle {
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: var(--color-fg-muted);
    font-size: 14px;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .panel-toggle:hover {
    background: var(--color-bg-hover);
    color: var(--color-fg-default);
  }

  .panel-toggle:focus-visible {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: 2px;
  }

  .view-mode-toggle {
    display: flex;
    gap: 2px;
    background: var(--color-bg-elevated);
    padding: 2px;
    border-radius: 6px;
    margin-left: auto;
    border: 1px solid var(--color-border-subtle);
  }

  .view-mode-toggle button {
    padding: 6px 12px;
    border: none;
    background: transparent;
    color: var(--color-fg-muted);
    font-size: 13px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .view-mode-toggle button:hover {
    background: var(--color-bg-hover);
    color: var(--color-fg-default);
  }

  .view-mode-toggle button.active {
    background: var(--color-bg-base);
    color: var(--color-fg-default);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .view-mode-toggle button:focus-visible {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: 2px;
  }

  .spec-browser__viewer {
    flex: 1;
    overflow: hidden;
  }

  .spec-browser__metadata {
    position: relative;
    flex-shrink: 0;
    background: var(--color-bg-surface);
    border-left: 1px solid var(--color-border-default);
    overflow: hidden;
  }

  /* Mobile responsive design */
  @media (max-width: 768px) {
    .spec-browser {
      flex-direction: column;
    }

    .spec-browser__nav,
    .spec-browser__metadata {
      width: 100% !important;
      height: 200px;
      border: none;
      border-bottom: 1px solid var(--color-border-default);
    }

    .spec-browser__nav {
      order: 1;
    }

    .spec-browser__content {
      order: 2;
      flex: 1;
    }

    .spec-browser__metadata {
      order: 3;
      border-top: 1px solid var(--color-border-default);
    }

    .spec-browser__header {
      flex-wrap: wrap;
      gap: 8px;
    }

    .view-mode-toggle {
      margin-left: 0;
      order: 3;
      width: 100%;
    }

    .panel-toggle {
      padding: 8px 12px;
    }
  }

  @media (max-width: 480px) {
    .spec-browser__header {
      padding: 6px 8px;
    }

    .view-mode-toggle button {
      padding: 8px 10px;
      font-size: 12px;
    }

    .panel-toggle {
      font-size: 16px;
    }
  }

  /* Dark theme enhancements */
  @media (prefers-color-scheme: dark) {
    .view-mode-toggle button.active {
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .panel-toggle,
    .view-mode-toggle button {
      border: 1px solid var(--color-border-emphasis);
    }

    .view-mode-toggle button.active {
      background: var(--color-accent-subtle);
      color: var(--color-accent-fg);
      border-color: var(--color-accent-fg);
    }
  }

  /* Reduce motion for accessibility */
  @media (prefers-reduced-motion: reduce) {
    .panel-toggle,
    .view-mode-toggle button {
      transition: none;
    }
  }
</style>