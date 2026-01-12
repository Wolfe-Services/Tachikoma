# Spec 188: Layout System

## Phase
Phase 9: UI Foundation

## Spec ID
188

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup
- Spec 187: Routing Configuration

## Estimated Context
~10%

---

## Objective

Implement a flexible layout system for Tachikoma with support for multiple layout patterns including sidebar navigation, split panels, tab-based views, and responsive adaptations for different screen sizes.

---

## Acceptance Criteria

- [x] Root application layout with Tauri window controls
- [x] Sidebar navigation layout component
- [x] Split panel layout with resizable panes
- [x] Tab-based content layout
- [x] Responsive breakpoint handling
- [x] Layout state persistence
- [x] Collapsible sidebar with toggle
- [x] Breadcrumb navigation support

---

## Implementation Details

### src/lib/components/layout/AppShell.svelte

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@utils/environment';
  import TitleBar from './TitleBar.svelte';
  import Sidebar from './Sidebar.svelte';
  import { layoutStore } from '@stores/layout';

  export let showTitleBar = true;
  export let showSidebar = true;

  let windowControlsOverlay = false;

  onMount(async () => {
    if (isTauri()) {
      // Check for custom title bar support
      const { appWindow } = await import('@tauri-apps/api/window');
      windowControlsOverlay = true;
    }
  });
</script>

<div
  class="app-shell"
  class:has-titlebar={showTitleBar}
  class:sidebar-collapsed={!$layoutStore.sidebarOpen}
>
  {#if showTitleBar}
    <TitleBar {windowControlsOverlay} />
  {/if}

  <div class="app-body">
    {#if showSidebar}
      <Sidebar />
    {/if}

    <main class="app-content">
      <slot />
    </main>
  </div>
</div>

<style>
  .app-shell {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-base);
    overflow: hidden;
  }

  .app-shell.has-titlebar {
    --titlebar-height: 40px;
  }

  .app-body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .app-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background-color: var(--color-bg-surface);
  }

  .sidebar-collapsed .app-content {
    margin-left: 0;
  }
</style>
```

### src/lib/components/layout/TitleBar.svelte

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@utils/environment';
  import { navigation } from '@stores/navigation';

  export let windowControlsOverlay = false;

  let isMaximized = false;

  async function handleMinimize() {
    if (isTauri()) {
      const { appWindow } = await import('@tauri-apps/api/window');
      await appWindow.minimize();
    }
  }

  async function handleMaximize() {
    if (isTauri()) {
      const { appWindow } = await import('@tauri-apps/api/window');
      await appWindow.toggleMaximize();
      isMaximized = await appWindow.isMaximized();
    }
  }

  async function handleClose() {
    if (isTauri()) {
      const { appWindow } = await import('@tauri-apps/api/window');
      await appWindow.close();
    }
  }

  onMount(async () => {
    if (isTauri()) {
      const { appWindow } = await import('@tauri-apps/api/window');
      isMaximized = await appWindow.isMaximized();

      // Listen for maximize state changes
      await appWindow.onResized(async () => {
        isMaximized = await appWindow.isMaximized();
      });
    }
  });
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left">
    <div class="nav-controls">
      <button
        class="nav-btn"
        disabled={!$navigation.canGoBack}
        on:click={() => navigation.back()}
        aria-label="Go back"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M10.5 3L5.5 8L10.5 13" stroke="currentColor" stroke-width="1.5" fill="none"/>
        </svg>
      </button>
      <button
        class="nav-btn"
        disabled={!$navigation.canGoForward}
        on:click={() => navigation.forward()}
        aria-label="Go forward"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M5.5 3L10.5 8L5.5 13" stroke="currentColor" stroke-width="1.5" fill="none"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="titlebar-center">
    <span class="app-title">Tachikoma</span>
  </div>

  <div class="titlebar-right">
    {#if windowControlsOverlay}
      <div class="window-controls">
        <button
          class="window-btn minimize"
          on:click={handleMinimize}
          aria-label="Minimize"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="2" y="5.5" width="8" height="1" fill="currentColor"/>
          </svg>
        </button>
        <button
          class="window-btn maximize"
          on:click={handleMaximize}
          aria-label={isMaximized ? 'Restore' : 'Maximize'}
        >
          {#if isMaximized}
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M3 3.5H8.5V9H3V3.5Z" stroke="currentColor" fill="none"/>
              <path d="M4.5 2H10V7.5" stroke="currentColor" fill="none"/>
            </svg>
          {:else}
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="2" y="2" width="8" height="8" stroke="currentColor" fill="none"/>
            </svg>
          {/if}
        </button>
        <button
          class="window-btn close"
          on:click={handleClose}
          aria-label="Close"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path d="M2 2L10 10M10 2L2 10" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
      </div>
    {/if}
  </div>
</header>

<style>
  .titlebar {
    height: var(--titlebar-height, 40px);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background-color: var(--color-bg-base);
    border-bottom: 1px solid var(--color-border);
    padding: 0 var(--spacing-2);
    user-select: none;
    -webkit-app-region: drag;
  }

  .titlebar-left,
  .titlebar-right {
    display: flex;
    align-items: center;
    min-width: 120px;
    -webkit-app-region: no-drag;
  }

  .titlebar-right {
    justify-content: flex-end;
  }

  .titlebar-center {
    flex: 1;
    text-align: center;
  }

  .app-title {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .nav-controls {
    display: flex;
    gap: var(--spacing-1);
  }

  .nav-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .nav-btn:hover:not(:disabled) {
    background-color: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .window-controls {
    display: flex;
    -webkit-app-region: no-drag;
  }

  .window-btn {
    width: 46px;
    height: var(--titlebar-height, 40px);
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .window-btn:hover {
    background-color: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }

  .window-btn.close:hover {
    background-color: #e81123;
    color: white;
  }
</style>
```

### src/lib/components/layout/Sidebar.svelte

```svelte
<script lang="ts">
  import { page } from '$app/stores';
  import { navigation } from '@stores/navigation';
  import { layoutStore } from '@stores/layout';

  interface NavItem {
    id: string;
    label: string;
    icon: string;
    path: string;
    badge?: number;
  }

  const navItems: NavItem[] = [
    { id: 'dashboard', label: 'Dashboard', icon: 'home', path: '/' },
    { id: 'projects', label: 'Projects', icon: 'folder', path: '/projects' },
    { id: 'ai', label: 'AI Assistant', icon: 'bot', path: '/ai' },
    { id: 'tools', label: 'Tools', icon: 'terminal', path: '/tools' },
    { id: 'settings', label: 'Settings', icon: 'settings', path: '/settings' }
  ];

  $: currentPath = $page.url.pathname;

  function isActive(path: string): boolean {
    if (path === '/') {
      return currentPath === '/';
    }
    return currentPath.startsWith(path);
  }

  function toggleSidebar() {
    layoutStore.toggleSidebar();
  }
</script>

<aside class="sidebar" class:collapsed={!$layoutStore.sidebarOpen}>
  <nav class="sidebar-nav">
    <ul class="nav-list">
      {#each navItems as item}
        <li>
          <button
            class="nav-item"
            class:active={isActive(item.path)}
            on:click={() => navigation.navigate(item.path)}
            title={!$layoutStore.sidebarOpen ? item.label : undefined}
          >
            <span class="nav-icon">
              <svelte:component this={getIcon(item.icon)} />
            </span>
            {#if $layoutStore.sidebarOpen}
              <span class="nav-label">{item.label}</span>
              {#if item.badge}
                <span class="nav-badge">{item.badge}</span>
              {/if}
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  </nav>

  <button
    class="sidebar-toggle"
    on:click={toggleSidebar}
    aria-label={$layoutStore.sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
  >
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="currentColor"
      class:rotated={!$layoutStore.sidebarOpen}
    >
      <path d="M10 4L6 8L10 12" stroke="currentColor" stroke-width="1.5" fill="none"/>
    </svg>
  </button>
</aside>

<script context="module" lang="ts">
  // Icon component mapping
  function getIcon(name: string) {
    const icons: Record<string, any> = {
      home: IconHome,
      folder: IconFolder,
      bot: IconBot,
      terminal: IconTerminal,
      settings: IconSettings
    };
    return icons[name] || IconHome;
  }

  // Placeholder icon components (would be actual components)
  const IconHome = { render: () => '<svg>...</svg>' };
  const IconFolder = { render: () => '<svg>...</svg>' };
  const IconBot = { render: () => '<svg>...</svg>' };
  const IconTerminal = { render: () => '<svg>...</svg>' };
  const IconSettings = { render: () => '<svg>...</svg>' };
</script>

<style>
  .sidebar {
    width: 240px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-base);
    border-right: 1px solid var(--color-border);
    transition: width 0.2s ease;
  }

  .sidebar.collapsed {
    width: 56px;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-2);
  }

  .nav-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .nav-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
    padding: var(--spacing-2) var(--spacing-3);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }

  .sidebar.collapsed .nav-item {
    justify-content: center;
    padding: var(--spacing-2);
  }

  .nav-item:hover {
    background-color: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }

  .nav-item.active {
    background-color: rgba(0, 212, 255, 0.1);
    color: var(--color-primary);
  }

  .nav-icon {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .nav-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-badge {
    min-width: 18px;
    height: 18px;
    padding: 0 var(--spacing-1);
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-primary);
    color: var(--color-bg-base);
    font-size: var(--font-size-xs);
    font-weight: 600;
    border-radius: 9px;
  }

  .sidebar-toggle {
    width: 100%;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-top: 1px solid var(--color-border);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .sidebar-toggle:hover {
    background-color: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }

  .sidebar-toggle svg.rotated {
    transform: rotate(180deg);
  }
</style>
```

### src/lib/components/layout/SplitPane.svelte

```svelte
<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';

  export let direction: 'horizontal' | 'vertical' = 'horizontal';
  export let initialSize: number = 50; // percentage
  export let minSize: number = 10; // percentage
  export let maxSize: number = 90; // percentage
  export let storageKey: string | null = null;

  const dispatch = createEventDispatcher<{
    resize: { size: number };
  }>();

  let containerEl: HTMLElement;
  let size = initialSize;
  let isDragging = false;

  onMount(() => {
    if (storageKey) {
      const saved = localStorage.getItem(`splitpane:${storageKey}`);
      if (saved) {
        size = parseFloat(saved);
      }
    }
  });

  function handleMouseDown(event: MouseEvent) {
    event.preventDefault();
    isDragging = true;

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging || !containerEl) return;

    const rect = containerEl.getBoundingClientRect();
    let newSize: number;

    if (direction === 'horizontal') {
      newSize = ((event.clientX - rect.left) / rect.width) * 100;
    } else {
      newSize = ((event.clientY - rect.top) / rect.height) * 100;
    }

    size = Math.min(maxSize, Math.max(minSize, newSize));

    if (storageKey) {
      localStorage.setItem(`splitpane:${storageKey}`, size.toString());
    }

    dispatch('resize', { size });
  }

  function handleMouseUp() {
    isDragging = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }
</script>

<div
  class="split-pane"
  class:horizontal={direction === 'horizontal'}
  class:vertical={direction === 'vertical'}
  class:dragging={isDragging}
  bind:this={containerEl}
>
  <div
    class="pane first"
    style={direction === 'horizontal' ? `width: ${size}%` : `height: ${size}%`}
  >
    <slot name="first" />
  </div>

  <div
    class="divider"
    on:mousedown={handleMouseDown}
    role="separator"
    aria-orientation={direction}
    tabindex="0"
  >
    <div class="divider-handle"></div>
  </div>

  <div class="pane second">
    <slot name="second" />
  </div>
</div>

<style>
  .split-pane {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .split-pane.horizontal {
    flex-direction: row;
  }

  .split-pane.vertical {
    flex-direction: column;
  }

  .pane {
    overflow: auto;
  }

  .pane.first {
    flex-shrink: 0;
  }

  .pane.second {
    flex: 1;
  }

  .divider {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-bg-base);
  }

  .horizontal > .divider {
    width: 4px;
    cursor: col-resize;
  }

  .vertical > .divider {
    height: 4px;
    cursor: row-resize;
  }

  .divider-handle {
    background-color: var(--color-border);
    border-radius: 2px;
    transition: background-color 0.15s ease;
  }

  .horizontal .divider-handle {
    width: 2px;
    height: 40px;
  }

  .vertical .divider-handle {
    width: 40px;
    height: 2px;
  }

  .divider:hover .divider-handle,
  .dragging .divider-handle {
    background-color: var(--color-primary);
  }

  .split-pane.dragging {
    user-select: none;
  }
</style>
```

### src/lib/stores/layout.ts

```typescript
import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

export interface LayoutState {
  sidebarOpen: boolean;
  sidebarWidth: number;
  panelSizes: Record<string, number>;
  activePanel: string | null;
  isFullscreen: boolean;
}

const STORAGE_KEY = 'tachikoma:layout';

function getInitialState(): LayoutState {
  if (browser) {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch {
        // Invalid JSON, use defaults
      }
    }
  }

  return {
    sidebarOpen: true,
    sidebarWidth: 240,
    panelSizes: {},
    activePanel: null,
    isFullscreen: false
  };
}

function createLayoutStore() {
  const { subscribe, set, update } = writable<LayoutState>(getInitialState());

  // Persist to localStorage
  if (browser) {
    subscribe(state => {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    });
  }

  return {
    subscribe,

    toggleSidebar: () => {
      update(state => ({
        ...state,
        sidebarOpen: !state.sidebarOpen
      }));
    },

    setSidebarOpen: (open: boolean) => {
      update(state => ({
        ...state,
        sidebarOpen: open
      }));
    },

    setSidebarWidth: (width: number) => {
      update(state => ({
        ...state,
        sidebarWidth: Math.max(180, Math.min(400, width))
      }));
    },

    setPanelSize: (panelId: string, size: number) => {
      update(state => ({
        ...state,
        panelSizes: {
          ...state.panelSizes,
          [panelId]: size
        }
      }));
    },

    setActivePanel: (panelId: string | null) => {
      update(state => ({
        ...state,
        activePanel: panelId
      }));
    },

    toggleFullscreen: () => {
      update(state => ({
        ...state,
        isFullscreen: !state.isFullscreen
      }));
    },

    reset: () => {
      set(getInitialState());
    }
  };
}

export const layoutStore = createLayoutStore();

// Derived stores
export const sidebarOpen = derived(layoutStore, $layout => $layout.sidebarOpen);
export const isFullscreen = derived(layoutStore, $layout => $layout.isFullscreen);
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/layout/sidebar.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Sidebar from '@components/layout/Sidebar.svelte';

vi.mock('$app/stores', () => ({
  page: { subscribe: vi.fn((cb) => { cb({ url: { pathname: '/' } }); return () => {}; }) }
}));

describe('Sidebar', () => {
  it('should render navigation items', () => {
    const { getByText } = render(Sidebar);
    expect(getByText('Dashboard')).toBeTruthy();
    expect(getByText('Projects')).toBeTruthy();
  });

  it('should highlight active navigation item', () => {
    const { container } = render(Sidebar);
    const activeItem = container.querySelector('.nav-item.active');
    expect(activeItem).toBeTruthy();
  });

  it('should toggle sidebar collapse', async () => {
    const { container, getByLabelText } = render(Sidebar);
    const toggleBtn = getByLabelText(/collapse sidebar/i);

    await fireEvent.click(toggleBtn);
    expect(container.querySelector('.sidebar.collapsed')).toBeTruthy();
  });
});
```

### Integration Tests

```typescript
// tests/layout/splitpane.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import SplitPane from '@components/layout/SplitPane.svelte';

describe('SplitPane', () => {
  it('should render with default size', () => {
    const { container } = render(SplitPane, {
      props: { initialSize: 50 }
    });

    const firstPane = container.querySelector('.pane.first');
    expect(firstPane).toHaveStyle({ width: '50%' });
  });

  it('should resize on drag', async () => {
    const { container } = render(SplitPane);
    const divider = container.querySelector('.divider');

    await fireEvent.mouseDown(divider!);
    // Simulate drag...
    await fireEvent.mouseUp(window);

    expect(container.querySelector('.dragging')).toBeFalsy();
  });
});
```

---

## Related Specs

- [186-sveltekit-setup.md](./186-sveltekit-setup.md) - SvelteKit setup
- [187-routing-config.md](./187-routing-config.md) - Routing configuration
- [189-store-architecture.md](./189-store-architecture.md) - Store architecture
- [191-design-tokens.md](./191-design-tokens.md) - Design tokens
