# 216 - Mission Panel Layout

**Phase:** 10 - Mission Panel UI
**Spec ID:** 216
**Status:** Planned
**Dependencies:** 004-svelte-integration, 005-ipc-bridge
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Create the main Mission Panel layout component that serves as the container for all mission-related UI elements, providing a responsive grid-based structure for mission creation, monitoring, and management.

---

## Acceptance Criteria

- [ ] `MissionPanel.svelte` component created with responsive layout
- [ ] Three-column layout: sidebar, main content, details panel
- [ ] Collapsible sidebar and details panel
- [ ] Keyboard shortcuts for panel navigation
- [ ] Dark/light theme support via CSS variables
- [ ] Mobile-responsive design with stacked layout
- [ ] Proper ARIA labels for accessibility

---

## Implementation Details

### 1. Types (src/lib/types/mission-layout.ts)

```typescript
/**
 * Mission panel layout configuration and state types.
 */

export interface PanelConfig {
  id: string;
  minWidth: number;
  maxWidth: number;
  defaultWidth: number;
  collapsible: boolean;
  resizable: boolean;
}

export interface MissionLayoutState {
  sidebarCollapsed: boolean;
  detailsCollapsed: boolean;
  sidebarWidth: number;
  detailsWidth: number;
  activePanel: 'sidebar' | 'main' | 'details';
  focusedElement: string | null;
}

export interface LayoutBreakpoint {
  name: 'mobile' | 'tablet' | 'desktop' | 'wide';
  minWidth: number;
  columns: number;
  sidebarVisible: boolean;
  detailsVisible: boolean;
}

export const LAYOUT_BREAKPOINTS: LayoutBreakpoint[] = [
  { name: 'mobile', minWidth: 0, columns: 1, sidebarVisible: false, detailsVisible: false },
  { name: 'tablet', minWidth: 768, columns: 2, sidebarVisible: true, detailsVisible: false },
  { name: 'desktop', minWidth: 1024, columns: 3, sidebarVisible: true, detailsVisible: true },
  { name: 'wide', minWidth: 1440, columns: 3, sidebarVisible: true, detailsVisible: true },
];

export const DEFAULT_PANEL_CONFIG: Record<string, PanelConfig> = {
  sidebar: {
    id: 'sidebar',
    minWidth: 200,
    maxWidth: 400,
    defaultWidth: 280,
    collapsible: true,
    resizable: true,
  },
  main: {
    id: 'main',
    minWidth: 400,
    maxWidth: Infinity,
    defaultWidth: 600,
    collapsible: false,
    resizable: false,
  },
  details: {
    id: 'details',
    minWidth: 250,
    maxWidth: 500,
    defaultWidth: 320,
    collapsible: true,
    resizable: true,
  },
};
```

### 2. Layout Store (src/lib/stores/mission-layout-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { MissionLayoutState, LayoutBreakpoint } from '$lib/types/mission-layout';
import { LAYOUT_BREAKPOINTS, DEFAULT_PANEL_CONFIG } from '$lib/types/mission-layout';

function createMissionLayoutStore() {
  const initialState: MissionLayoutState = {
    sidebarCollapsed: false,
    detailsCollapsed: false,
    sidebarWidth: DEFAULT_PANEL_CONFIG.sidebar.defaultWidth,
    detailsWidth: DEFAULT_PANEL_CONFIG.details.defaultWidth,
    activePanel: 'main',
    focusedElement: null,
  };

  const { subscribe, set, update } = writable<MissionLayoutState>(initialState);

  return {
    subscribe,
    set,
    update,

    toggleSidebar: () => update(s => ({ ...s, sidebarCollapsed: !s.sidebarCollapsed })),
    toggleDetails: () => update(s => ({ ...s, detailsCollapsed: !s.detailsCollapsed })),

    setSidebarWidth: (width: number) => update(s => ({
      ...s,
      sidebarWidth: Math.max(
        DEFAULT_PANEL_CONFIG.sidebar.minWidth,
        Math.min(DEFAULT_PANEL_CONFIG.sidebar.maxWidth, width)
      ),
    })),

    setDetailsWidth: (width: number) => update(s => ({
      ...s,
      detailsWidth: Math.max(
        DEFAULT_PANEL_CONFIG.details.minWidth,
        Math.min(DEFAULT_PANEL_CONFIG.details.maxWidth, width)
      ),
    })),

    setActivePanel: (panel: 'sidebar' | 'main' | 'details') =>
      update(s => ({ ...s, activePanel: panel })),

    setFocusedElement: (elementId: string | null) =>
      update(s => ({ ...s, focusedElement: elementId })),

    reset: () => set(initialState),
  };
}

export const missionLayoutStore = createMissionLayoutStore();

// Derived store for current breakpoint
export const currentBreakpoint = derived<typeof missionLayoutStore, LayoutBreakpoint>(
  missionLayoutStore,
  ($layout, set) => {
    if (typeof window === 'undefined') {
      set(LAYOUT_BREAKPOINTS[2]); // Default to desktop
      return;
    }

    const updateBreakpoint = () => {
      const width = window.innerWidth;
      const breakpoint = [...LAYOUT_BREAKPOINTS]
        .reverse()
        .find(bp => width >= bp.minWidth) || LAYOUT_BREAKPOINTS[0];
      set(breakpoint);
    };

    updateBreakpoint();
    window.addEventListener('resize', updateBreakpoint);

    return () => window.removeEventListener('resize', updateBreakpoint);
  }
);
```

### 3. Resize Handle Component (src/lib/components/mission/ResizeHandle.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let direction: 'horizontal' | 'vertical' = 'horizontal';
  export let position: 'left' | 'right' | 'top' | 'bottom' = 'right';

  const dispatch = createEventDispatcher<{
    resize: { delta: number };
    resizeStart: void;
    resizeEnd: void;
  }>();

  let isDragging = false;
  let startPos = 0;

  function handleMouseDown(event: MouseEvent) {
    isDragging = true;
    startPos = direction === 'horizontal' ? event.clientX : event.clientY;
    dispatch('resizeStart');

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
    document.body.style.userSelect = 'none';
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging) return;

    const currentPos = direction === 'horizontal' ? event.clientX : event.clientY;
    const delta = position === 'left' || position === 'top'
      ? startPos - currentPos
      : currentPos - startPos;

    dispatch('resize', { delta });
    startPos = currentPos;
  }

  function handleMouseUp() {
    isDragging = false;
    dispatch('resizeEnd');

    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }

  function handleKeyDown(event: KeyboardEvent) {
    const step = event.shiftKey ? 50 : 10;

    if (direction === 'horizontal') {
      if (event.key === 'ArrowLeft') {
        dispatch('resize', { delta: -step });
        event.preventDefault();
      } else if (event.key === 'ArrowRight') {
        dispatch('resize', { delta: step });
        event.preventDefault();
      }
    } else {
      if (event.key === 'ArrowUp') {
        dispatch('resize', { delta: -step });
        event.preventDefault();
      } else if (event.key === 'ArrowDown') {
        dispatch('resize', { delta: step });
        event.preventDefault();
      }
    }
  }
</script>

<div
  class="resize-handle resize-handle--{direction} resize-handle--{position}"
  class:resize-handle--dragging={isDragging}
  role="separator"
  aria-orientation={direction}
  aria-label="Resize panel"
  tabindex="0"
  on:mousedown={handleMouseDown}
  on:keydown={handleKeyDown}
>
  <div class="resize-handle__indicator"></div>
</div>

<style>
  .resize-handle {
    position: absolute;
    background: transparent;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .resize-handle--horizontal {
    width: 8px;
    height: 100%;
    cursor: col-resize;
  }

  .resize-handle--vertical {
    width: 100%;
    height: 8px;
    cursor: row-resize;
  }

  .resize-handle--left {
    left: 0;
    transform: translateX(-50%);
  }

  .resize-handle--right {
    right: 0;
    transform: translateX(50%);
  }

  .resize-handle__indicator {
    background: var(--color-border);
    border-radius: 2px;
    transition: background-color 0.15s ease;
  }

  .resize-handle--horizontal .resize-handle__indicator {
    width: 4px;
    height: 40px;
  }

  .resize-handle--vertical .resize-handle__indicator {
    width: 40px;
    height: 4px;
  }

  .resize-handle:hover .resize-handle__indicator,
  .resize-handle:focus .resize-handle__indicator,
  .resize-handle--dragging .resize-handle__indicator {
    background: var(--color-primary);
  }

  .resize-handle:focus {
    outline: none;
  }

  .resize-handle:focus-visible .resize-handle__indicator {
    box-shadow: 0 0 0 2px var(--color-focus-ring);
  }
</style>
```

### 4. Main Layout Component (src/lib/components/mission/MissionPanel.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { missionLayoutStore, currentBreakpoint } from '$lib/stores/mission-layout-store';
  import ResizeHandle from './ResizeHandle.svelte';

  // Child components (to be implemented in subsequent specs)
  export let showMissionList = true;
  export let showDetails = true;

  let panelRef: HTMLElement;

  // Keyboard shortcuts
  function handleKeyDown(event: KeyboardEvent) {
    // Cmd/Ctrl + B: Toggle sidebar
    if ((event.metaKey || event.ctrlKey) && event.key === 'b') {
      event.preventDefault();
      missionLayoutStore.toggleSidebar();
    }

    // Cmd/Ctrl + D: Toggle details panel
    if ((event.metaKey || event.ctrlKey) && event.key === 'd') {
      event.preventDefault();
      missionLayoutStore.toggleDetails();
    }

    // Cmd/Ctrl + 1/2/3: Focus panels
    if ((event.metaKey || event.ctrlKey) && ['1', '2', '3'].includes(event.key)) {
      event.preventDefault();
      const panels: Array<'sidebar' | 'main' | 'details'> = ['sidebar', 'main', 'details'];
      const panelIndex = parseInt(event.key) - 1;
      missionLayoutStore.setActivePanel(panels[panelIndex]);
    }
  }

  function handleSidebarResize(event: CustomEvent<{ delta: number }>) {
    missionLayoutStore.setSidebarWidth($missionLayoutStore.sidebarWidth + event.detail.delta);
  }

  function handleDetailsResize(event: CustomEvent<{ delta: number }>) {
    missionLayoutStore.setDetailsWidth($missionLayoutStore.detailsWidth + event.detail.delta);
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  $: sidebarWidth = $missionLayoutStore.sidebarCollapsed ? 0 : $missionLayoutStore.sidebarWidth;
  $: detailsWidth = $missionLayoutStore.detailsCollapsed ? 0 : $missionLayoutStore.detailsWidth;
  $: isMobile = $currentBreakpoint.name === 'mobile';
  $: showSidebar = showMissionList && $currentBreakpoint.sidebarVisible && !$missionLayoutStore.sidebarCollapsed;
  $: showDetailsPanel = showDetails && $currentBreakpoint.detailsVisible && !$missionLayoutStore.detailsCollapsed;
</script>

<div
  bind:this={panelRef}
  class="mission-panel"
  class:mission-panel--mobile={isMobile}
  role="region"
  aria-label="Mission Panel"
>
  <!-- Sidebar -->
  {#if showSidebar}
    <aside
      class="mission-panel__sidebar"
      style="width: {sidebarWidth}px"
      role="complementary"
      aria-label="Mission list"
      class:mission-panel__sidebar--active={$missionLayoutStore.activePanel === 'sidebar'}
    >
      <slot name="sidebar">
        <div class="mission-panel__placeholder">Mission List</div>
      </slot>

      <ResizeHandle
        direction="horizontal"
        position="right"
        on:resize={handleSidebarResize}
      />
    </aside>
  {/if}

  <!-- Main Content -->
  <main
    class="mission-panel__main"
    role="main"
    aria-label="Mission workspace"
    class:mission-panel__main--active={$missionLayoutStore.activePanel === 'main'}
  >
    <slot name="main">
      <div class="mission-panel__placeholder">Mission Workspace</div>
    </slot>
  </main>

  <!-- Details Panel -->
  {#if showDetailsPanel}
    <aside
      class="mission-panel__details"
      style="width: {detailsWidth}px"
      role="complementary"
      aria-label="Mission details"
      class:mission-panel__details--active={$missionLayoutStore.activePanel === 'details'}
    >
      <ResizeHandle
        direction="horizontal"
        position="left"
        on:resize={handleDetailsResize}
      />

      <slot name="details">
        <div class="mission-panel__placeholder">Mission Details</div>
      </slot>
    </aside>
  {/if}

  <!-- Mobile Navigation -->
  {#if isMobile}
    <nav class="mission-panel__mobile-nav" aria-label="Panel navigation">
      <button
        class="mission-panel__mobile-nav-btn"
        class:active={$missionLayoutStore.activePanel === 'sidebar'}
        on:click={() => missionLayoutStore.setActivePanel('sidebar')}
        aria-label="Show mission list"
      >
        List
      </button>
      <button
        class="mission-panel__mobile-nav-btn"
        class:active={$missionLayoutStore.activePanel === 'main'}
        on:click={() => missionLayoutStore.setActivePanel('main')}
        aria-label="Show mission workspace"
      >
        Mission
      </button>
      <button
        class="mission-panel__mobile-nav-btn"
        class:active={$missionLayoutStore.activePanel === 'details'}
        on:click={() => missionLayoutStore.setActivePanel('details')}
        aria-label="Show mission details"
      >
        Details
      </button>
    </nav>
  {/if}
</div>

<style>
  .mission-panel {
    display: grid;
    grid-template-columns: auto 1fr auto;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
  }

  .mission-panel--mobile {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr auto;
  }

  .mission-panel__sidebar,
  .mission-panel__details {
    position: relative;
    overflow: hidden;
    background: var(--color-bg-secondary);
    border-color: var(--color-border);
  }

  .mission-panel__sidebar {
    border-right: 1px solid var(--color-border);
  }

  .mission-panel__details {
    border-left: 1px solid var(--color-border);
  }

  .mission-panel__main {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .mission-panel__sidebar--active,
  .mission-panel__main--active,
  .mission-panel__details--active {
    box-shadow: inset 0 0 0 2px var(--color-focus-ring);
  }

  .mission-panel__placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    font-size: 14px;
  }

  .mission-panel__mobile-nav {
    display: flex;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .mission-panel__mobile-nav-btn {
    flex: 1;
    padding: 12px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mission-panel__mobile-nav-btn:hover {
    background: var(--color-bg-hover);
  }

  .mission-panel__mobile-nav-btn.active {
    color: var(--color-primary);
    background: var(--color-bg-active);
  }
</style>
```

### 5. CSS Variables (src/lib/styles/mission-theme.css)

```css
:root {
  /* Light theme */
  --color-bg-primary: #ffffff;
  --color-bg-secondary: #f8f9fa;
  --color-bg-hover: #e9ecef;
  --color-bg-active: #e3f2fd;
  --color-text-primary: #212529;
  --color-text-secondary: #6c757d;
  --color-text-muted: #adb5bd;
  --color-border: #dee2e6;
  --color-primary: #2196f3;
  --color-focus-ring: rgba(33, 150, 243, 0.4);
  --color-success: #4caf50;
  --color-warning: #ff9800;
  --color-error: #f44336;
}

[data-theme='dark'] {
  --color-bg-primary: #1a1a2e;
  --color-bg-secondary: #16213e;
  --color-bg-hover: #1f3460;
  --color-bg-active: #0f3460;
  --color-text-primary: #e8e8e8;
  --color-text-secondary: #a0a0a0;
  --color-text-muted: #6c6c6c;
  --color-border: #2d3748;
  --color-primary: #64b5f6;
  --color-focus-ring: rgba(100, 181, 246, 0.4);
  --color-success: #81c784;
  --color-warning: #ffb74d;
  --color-error: #e57373;
}
```

---

## Testing Requirements

1. Layout renders correctly at all breakpoints
2. Sidebar and details panels collapse/expand properly
3. Resize handles adjust panel widths within bounds
4. Keyboard shortcuts work as expected
5. Mobile navigation switches between panels
6. ARIA labels are correctly set for accessibility
7. Theme CSS variables apply correctly

### Test File (src/lib/components/mission/__tests__/MissionPanel.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import MissionPanel from '../MissionPanel.svelte';
import { missionLayoutStore } from '$lib/stores/mission-layout-store';

describe('MissionPanel', () => {
  it('renders with default layout', () => {
    render(MissionPanel);

    expect(screen.getByRole('region', { name: 'Mission Panel' })).toBeInTheDocument();
    expect(screen.getByRole('complementary', { name: 'Mission list' })).toBeInTheDocument();
    expect(screen.getByRole('main', { name: 'Mission workspace' })).toBeInTheDocument();
  });

  it('toggles sidebar with keyboard shortcut', async () => {
    render(MissionPanel);

    await fireEvent.keyDown(window, { key: 'b', ctrlKey: true });

    const state = get(missionLayoutStore);
    expect(state.sidebarCollapsed).toBe(true);
  });

  it('respects min/max width constraints on resize', async () => {
    render(MissionPanel);

    missionLayoutStore.setSidebarWidth(100); // Below min
    let state = get(missionLayoutStore);
    expect(state.sidebarWidth).toBe(200); // Should be min

    missionLayoutStore.setSidebarWidth(500); // Above max
    state = get(missionLayoutStore);
    expect(state.sidebarWidth).toBe(400); // Should be max
  });

  it('switches panels in mobile view', async () => {
    // Mock mobile breakpoint
    Object.defineProperty(window, 'innerWidth', { value: 500 });

    render(MissionPanel);

    const navButtons = screen.getAllByRole('button');
    await fireEvent.click(navButtons[2]); // Details button

    const state = get(missionLayoutStore);
    expect(state.activePanel).toBe('details');
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

- Depends on: [004-svelte-integration.md](../phase-00-setup/004-svelte-integration.md)
- Depends on: [005-ipc-bridge.md](../phase-00-setup/005-ipc-bridge.md)
- Next: [217-mission-state.md](217-mission-state.md)
- Used by: All Mission Panel UI specs (217-235)
