<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import SessionSidebar from './SessionSidebar.svelte';
  import ParticipantPanel from './ParticipantPanel.svelte';
  import MainContentArea from './MainContentArea.svelte';
  import ResultPanel from './ResultPanel.svelte';
  import ForgeToolbar from './ForgeToolbar.svelte';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import { layoutPreferencesStore } from '$lib/stores/layoutPreferences';
  import type { ForgeLayoutConfig, PanelState } from '$lib/types/forge';

  export let sessionId: string | null = null;

  const dispatch = createEventDispatcher<{
    newSession: void;
    editSession: { sessionId: string };
  }>();

  const defaultConfig: ForgeLayoutConfig = {
    leftSidebarWidth: 280,
    rightPanelWidth: 320,
    leftSidebarVisible: true,
    rightPanelVisible: true,
    bottomPanelVisible: false,
    bottomPanelHeight: 200
  };

  let layoutConfig = writable<ForgeLayoutConfig>(defaultConfig);
  let isResizing = writable<string | null>(null);
  let containerRef: HTMLElement;

  const panelStates = derived(
    [layoutConfig, forgeSessionStore],
    ([$config, $session]) => ({
      hasActiveSession: !!$session?.activeSession,
      showParticipants: $config.leftSidebarVisible && !!$session?.activeSession,
      showResults: $config.rightPanelVisible && !!$session?.activeSession?.hasResults,
      sessionPhase: $session?.activeSession?.phase ?? 'idle'
    })
  );

  function handleResizeStart(panel: string) {
    return (event: MouseEvent) => {
      event.preventDefault();
      isResizing.set(panel);
      document.addEventListener('mousemove', handleResizeMove);
      document.addEventListener('mouseup', handleResizeEnd);
    };
  }

  function handleResizeMove(event: MouseEvent) {
    const resizingPanel = $isResizing;
    if (!resizingPanel || !containerRef) return;

    const containerRect = containerRef.getBoundingClientRect();

    layoutConfig.update(config => {
      if (resizingPanel === 'left') {
        const newWidth = Math.max(200, Math.min(400, event.clientX - containerRect.left));
        return { ...config, leftSidebarWidth: newWidth };
      } else if (resizingPanel === 'right') {
        const newWidth = Math.max(250, Math.min(500, containerRect.right - event.clientX));
        return { ...config, rightPanelWidth: newWidth };
      } else if (resizingPanel === 'bottom') {
        const newHeight = Math.max(150, Math.min(400, containerRect.bottom - event.clientY));
        return { ...config, bottomPanelHeight: newHeight };
      }
      return config;
    });
  }

  function handleResizeEnd() {
    isResizing.set(null);
    document.removeEventListener('mousemove', handleResizeMove);
    document.removeEventListener('mouseup', handleResizeEnd);
    saveLayoutPreferences();
  }

  function togglePanel(panel: 'left' | 'right' | 'bottom') {
    layoutConfig.update(config => {
      switch (panel) {
        case 'left':
          return { ...config, leftSidebarVisible: !config.leftSidebarVisible };
        case 'right':
          return { ...config, rightPanelVisible: !config.rightPanelVisible };
        case 'bottom':
          return { ...config, bottomPanelVisible: !config.bottomPanelVisible };
      }
    });
    saveLayoutPreferences();
  }

  function saveLayoutPreferences() {
    layoutPreferencesStore.save('forge', $layoutConfig);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case 'b':
          event.preventDefault();
          togglePanel('left');
          break;
        case '\\':
          event.preventDefault();
          togglePanel('right');
          break;
        case 'j':
          event.preventDefault();
          togglePanel('bottom');
          break;
      }
    }
  }

  onMount(async () => {
    const savedPrefs = await layoutPreferencesStore.load('forge');
    if (savedPrefs) {
      layoutConfig.set({ ...defaultConfig, ...savedPrefs });
    }
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div
  class="forge-layout"
  bind:this={containerRef}
  class:resizing={$isResizing !== null}
  data-testid="forge-layout"
>
  <ForgeToolbar
    {sessionId}
    on:togglePanel={(e) => togglePanel(e.detail)}
    on:newSession={() => dispatch('newSession')}
    layoutConfig={$layoutConfig}
  />

  <div class="forge-content">
    {#if $layoutConfig.leftSidebarVisible}
      <aside
        class="left-sidebar"
        style="width: {$layoutConfig.leftSidebarWidth}px"
        aria-label="Session sidebar"
        data-visible={$layoutConfig.leftSidebarVisible}
      >
        <SessionSidebar {sessionId} />

        {#if $panelStates.showParticipants}
          <ParticipantPanel session={$forgeSessionStore.activeSession} />
        {/if}

        <div
          class="resize-handle vertical"
          on:mousedown={handleResizeStart('left')}
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize left sidebar"
          tabindex="0"
        />
      </aside>
    {/if}

    <main class="main-content" role="main" aria-label="Forge session content">
      <MainContentArea
        {sessionId}
        phase={$panelStates.sessionPhase}
        on:editSession={(e) => dispatch('editSession', e.detail)}
      />

      {#if $layoutConfig.bottomPanelVisible}
        <div
          class="resize-handle horizontal"
          on:mousedown={handleResizeStart('bottom')}
          role="separator"
          aria-orientation="horizontal"
          aria-label="Resize bottom panel"
          tabindex="0"
        />
        <div
          class="bottom-panel"
          style="height: {$layoutConfig.bottomPanelHeight}px"
          aria-label="Session logs"
        >
          <slot name="bottom-panel" />
        </div>
      {/if}
    </main>

    {#if $layoutConfig.rightPanelVisible}
      <aside
        class="right-panel"
        style="width: {$layoutConfig.rightPanelWidth}px"
        aria-label="Results panel"
        data-visible={$layoutConfig.rightPanelVisible}
      >
        <div
          class="resize-handle vertical"
          on:mousedown={handleResizeStart('right')}
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize right panel"
          tabindex="0"
        />

        <ResultPanel
          session={$forgeSessionStore.activeSession}
          visible={$panelStates.showResults}
        />
      </aside>
    {/if}
  </div>
</div>

<style>
  .forge-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary, #0d1117);
    color: var(--text-primary, #e6edf3);
  }

  .forge-layout.resizing {
    cursor: col-resize;
    user-select: none;
  }

  .forge-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .left-sidebar {
    display: flex;
    flex-direction: column;
    background: rgba(13, 17, 23, 0.35);
    border-right: 1px solid rgba(78, 205, 196, 0.12);
    position: relative;
    overflow: hidden;
    transition: transform 0.3s ease;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 400px;
  }

  .right-panel {
    display: flex;
    flex-direction: column;
    background: rgba(13, 17, 23, 0.35);
    border-left: 1px solid rgba(78, 205, 196, 0.12);
    position: relative;
    overflow: hidden;
    transition: transform 0.3s ease;
  }

  .bottom-panel {
    background: var(--panel-bg, #16213e);
    border-top: 1px solid var(--border-color, #2a2a4a);
    overflow: auto;
  }

  .resize-handle {
    position: absolute;
    background: transparent;
    z-index: 10;
  }

  .resize-handle.vertical {
    width: 4px;
    height: 100%;
    top: 0;
    cursor: col-resize;
  }

  .left-sidebar .resize-handle.vertical {
    right: 0;
  }

  .right-panel .resize-handle.vertical {
    left: 0;
  }

  .resize-handle.horizontal {
    height: 4px;
    width: 100%;
    cursor: row-resize;
  }

  .resize-handle:hover,
  .resize-handle:focus {
    background: var(--accent-color, #4a9eff);
  }

  @media (max-width: 768px) {
    .left-sidebar,
    .right-panel {
      position: absolute;
      z-index: 100;
      height: 100%;
    }

    .left-sidebar {
      left: 0;
      transform: translateX(-100%);
    }

    .left-sidebar[data-visible="true"] {
      transform: translateX(0);
    }

    .right-panel {
      right: 0;
      transform: translateX(100%);
    }

    .right-panel[data-visible="true"] {
      transform: translateX(0);
    }
  }
</style>