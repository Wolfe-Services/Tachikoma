# Spec 205: Tabs Component

## Phase
Phase 9: UI Foundation

## Spec ID
205

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement a Tabs component for Tachikoma with support for horizontal and vertical orientations, icons, badges, closeable tabs, lazy loading, and full keyboard navigation following WAI-ARIA Tabs pattern.

---

## Acceptance Criteria

- [ ] Horizontal and vertical tab orientations
- [ ] Icon and badge support
- [ ] Closeable tabs (for dynamic tabs)
- [ ] Keyboard navigation (arrow keys, Home, End)
- [ ] WAI-ARIA compliant
- [ ] Tab panel lazy loading option
- [ ] Active tab indicator animation
- [ ] Overflow handling with scroll
- [ ] Draggable tab reordering (optional)
- [ ] Controlled and uncontrolled modes

---

## Implementation Details

### src/lib/components/ui/Tabs/Tabs.svelte

```svelte
<script lang="ts" context="module">
  export interface TabItem {
    id: string;
    label: string;
    icon?: string;
    badge?: string | number;
    disabled?: boolean;
    closeable?: boolean;
  }
</script>

<script lang="ts">
  import { createEventDispatcher, setContext, tick } from 'svelte';
  import { writable } from 'svelte/store';
  import { cn } from '@utils/component';
  import { Keys, useRovingFocus } from '@utils/a11y';
  import Icon from '../Icon/Icon.svelte';
  import Badge from '../Badge/Badge.svelte';

  type TabsVariant = 'line' | 'enclosed' | 'pills';
  type TabsOrientation = 'horizontal' | 'vertical';
  type TabsSize = 'sm' | 'md' | 'lg';

  export let tabs: TabItem[] = [];
  export let value: string = tabs[0]?.id ?? '';
  export let variant: TabsVariant = 'line';
  export let orientation: TabsOrientation = 'horizontal';
  export let size: TabsSize = 'md';
  export let fullWidth: boolean = false;
  export let lazyLoad: boolean = false;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    change: string;
    close: string;
  }>();

  const activeTab = writable<string>(value);
  let tabListElement: HTMLElement;
  let tabElements: HTMLElement[] = [];
  let indicatorStyle = '';

  // Provide context for TabPanel components
  setContext('tabs', {
    activeTab,
    lazyLoad
  });

  $: activeTab.set(value);
  $: activeTabs = tabs.filter(tab => !tab.disabled);
  $: activeIndex = tabs.findIndex(tab => tab.id === value);

  $: {
    // Update indicator position
    if (tabElements[activeIndex] && variant === 'line') {
      updateIndicator();
    }
  }

  async function updateIndicator() {
    await tick();
    const activeEl = tabElements[activeIndex];
    if (!activeEl) return;

    if (orientation === 'horizontal') {
      indicatorStyle = `
        left: ${activeEl.offsetLeft}px;
        width: ${activeEl.offsetWidth}px;
      `;
    } else {
      indicatorStyle = `
        top: ${activeEl.offsetTop}px;
        height: ${activeEl.offsetHeight}px;
      `;
    }
  }

  function selectTab(tabId: string) {
    const tab = tabs.find(t => t.id === tabId);
    if (tab?.disabled) return;

    value = tabId;
    dispatch('change', tabId);
  }

  function closeTab(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    dispatch('close', tabId);
  }

  function handleKeyDown(event: KeyboardEvent, index: number) {
    const enabledTabs = tabs.filter(t => !t.disabled);
    const currentEnabledIndex = enabledTabs.findIndex(t => t.id === tabs[index].id);

    let newIndex = currentEnabledIndex;

    const isVertical = orientation === 'vertical';
    const nextKey = isVertical ? Keys.ArrowDown : Keys.ArrowRight;
    const prevKey = isVertical ? Keys.ArrowUp : Keys.ArrowLeft;

    switch (event.key) {
      case nextKey:
        event.preventDefault();
        newIndex = (currentEnabledIndex + 1) % enabledTabs.length;
        break;
      case prevKey:
        event.preventDefault();
        newIndex = (currentEnabledIndex - 1 + enabledTabs.length) % enabledTabs.length;
        break;
      case Keys.Home:
        event.preventDefault();
        newIndex = 0;
        break;
      case Keys.End:
        event.preventDefault();
        newIndex = enabledTabs.length - 1;
        break;
      case Keys.Enter:
      case Keys.Space:
        event.preventDefault();
        selectTab(tabs[index].id);
        return;
      default:
        return;
    }

    const newTab = enabledTabs[newIndex];
    const newTabIndex = tabs.findIndex(t => t.id === newTab.id);
    tabElements[newTabIndex]?.focus();
    selectTab(newTab.id);
  }

  $: containerClasses = cn(
    'tabs',
    `tabs-${variant}`,
    `tabs-${orientation}`,
    `tabs-${size}`,
    fullWidth && 'tabs-full-width',
    className
  );
</script>

<div class={containerClasses}>
  <div
    bind:this={tabListElement}
    class="tabs-list"
    role="tablist"
    aria-orientation={orientation}
  >
    {#each tabs as tab, index (tab.id)}
      <button
        bind:this={tabElements[index]}
        type="button"
        role="tab"
        id="tab-{tab.id}"
        class="tab"
        class:active={value === tab.id}
        class:disabled={tab.disabled}
        tabindex={value === tab.id ? 0 : -1}
        aria-selected={value === tab.id}
        aria-controls="panel-{tab.id}"
        aria-disabled={tab.disabled}
        disabled={tab.disabled}
        on:click={() => selectTab(tab.id)}
        on:keydown={(e) => handleKeyDown(e, index)}
      >
        {#if tab.icon}
          <Icon name={tab.icon} size={size === 'sm' ? 14 : size === 'lg' ? 20 : 16} />
        {/if}

        <span class="tab-label">{tab.label}</span>

        {#if tab.badge !== undefined}
          <Badge size="sm" variant="secondary">{tab.badge}</Badge>
        {/if}

        {#if tab.closeable}
          <button
            type="button"
            class="tab-close"
            on:click={(e) => closeTab(e, tab.id)}
            aria-label="Close {tab.label} tab"
            tabindex="-1"
          >
            <Icon name="x" size={14} />
          </button>
        {/if}
      </button>
    {/each}

    {#if variant === 'line'}
      <span class="tabs-indicator" style={indicatorStyle}></span>
    {/if}
  </div>

  <div class="tabs-panels">
    <slot />
  </div>
</div>

<style>
  .tabs {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .tabs-vertical {
    flex-direction: row;
  }

  /* Tab List */
  .tabs-list {
    display: flex;
    position: relative;
    gap: var(--spacing-1);
  }

  .tabs-horizontal .tabs-list {
    flex-direction: row;
    border-bottom: 1px solid var(--color-border-default);
  }

  .tabs-vertical .tabs-list {
    flex-direction: column;
    border-right: 1px solid var(--color-border-default);
    min-width: 160px;
  }

  /* Tab Button */
  .tab {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    background: transparent;
    border: none;
    border-radius: 0;
    color: var(--color-fg-muted);
    font-family: inherit;
    font-weight: var(--font-medium);
    cursor: pointer;
    white-space: nowrap;
    transition:
      color var(--duration-150) var(--ease-out),
      background-color var(--duration-150) var(--ease-out);
  }

  .tab:hover:not(.disabled) {
    color: var(--color-fg-default);
  }

  .tab:focus-visible {
    outline: none;
    box-shadow: inset var(--focus-ring);
    border-radius: var(--radius-sm);
  }

  .tab.active {
    color: var(--tachikoma-500);
  }

  .tab.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Tab Sizes */
  .tabs-sm .tab {
    padding: var(--spacing-1-5) var(--spacing-2);
    font-size: var(--text-xs);
  }

  .tabs-md .tab {
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--text-sm);
  }

  .tabs-lg .tab {
    padding: var(--spacing-3) var(--spacing-4);
    font-size: var(--text-base);
  }

  /* Full Width */
  .tabs-full-width .tab {
    flex: 1;
    justify-content: center;
  }

  /* Variants */

  /* Line variant - default */
  .tabs-line .tab {
    margin-bottom: -1px;
  }

  .tabs-line.tabs-vertical .tab {
    margin-bottom: 0;
    margin-right: -1px;
  }

  .tabs-indicator {
    position: absolute;
    background-color: var(--tachikoma-500);
    transition: all var(--duration-200) var(--ease-out);
  }

  .tabs-horizontal .tabs-indicator {
    bottom: 0;
    height: 2px;
  }

  .tabs-vertical .tabs-indicator {
    right: 0;
    width: 2px;
  }

  /* Enclosed variant */
  .tabs-enclosed .tabs-list {
    background-color: var(--color-bg-muted);
    border-radius: var(--radius-lg);
    padding: var(--spacing-1);
    gap: var(--spacing-0-5);
    border: none;
  }

  .tabs-enclosed .tab {
    border-radius: var(--radius-md);
  }

  .tabs-enclosed .tab.active {
    background-color: var(--color-bg-surface);
    color: var(--color-fg-default);
    box-shadow: var(--shadow-sm);
  }

  /* Pills variant */
  .tabs-pills .tabs-list {
    gap: var(--spacing-2);
    border: none;
  }

  .tabs-pills .tab {
    border-radius: var(--radius-full);
    padding: var(--spacing-1-5) var(--spacing-4);
  }

  .tabs-pills .tab.active {
    background-color: var(--tachikoma-500);
    color: var(--color-bg-base);
  }

  /* Tab label and close */
  .tab-label {
    flex: 1;
  }

  .tab-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    padding: 0;
    margin-left: var(--spacing-1);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: inherit;
    opacity: 0.6;
    cursor: pointer;
    transition: opacity var(--duration-150) var(--ease-out),
                background-color var(--duration-150) var(--ease-out);
  }

  .tab-close:hover {
    opacity: 1;
    background-color: var(--color-bg-hover);
  }

  /* Panels */
  .tabs-panels {
    flex: 1;
    min-height: 0;
  }

  .tabs-horizontal .tabs-panels {
    padding-top: var(--spacing-4);
  }

  .tabs-vertical .tabs-panels {
    padding-left: var(--spacing-4);
  }
</style>
```

### src/lib/components/ui/Tabs/TabPanel.svelte

```svelte
<script lang="ts">
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';

  export let id: string;
  export let keepMounted: boolean = false;

  const { activeTab, lazyLoad } = getContext<{
    activeTab: Writable<string>;
    lazyLoad: boolean;
  }>('tabs');

  let hasBeenActive = false;

  $: isActive = $activeTab === id;
  $: {
    if (isActive) {
      hasBeenActive = true;
    }
  }
  $: shouldRender = isActive || keepMounted || (lazyLoad && hasBeenActive);
</script>

{#if shouldRender}
  <div
    role="tabpanel"
    id="panel-{id}"
    aria-labelledby="tab-{id}"
    class="tab-panel"
    class:hidden={!isActive}
    tabindex="0"
  >
    <slot />
  </div>
{/if}

<style>
  .tab-panel {
    outline: none;
  }

  .tab-panel:focus-visible {
    box-shadow: var(--focus-ring);
    border-radius: var(--radius-md);
  }

  .tab-panel.hidden {
    display: none;
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { Tabs, TabPanel, Icon } from '@components/ui';

  const tabs = [
    { id: 'overview', label: 'Overview', icon: 'home' },
    { id: 'targets', label: 'Targets', icon: 'target', badge: 5 },
    { id: 'scans', label: 'Scans', icon: 'radar' },
    { id: 'reports', label: 'Reports', icon: 'file-text', disabled: true }
  ];

  let activeTab = 'overview';

  function handleChange(event) {
    console.log('Tab changed:', event.detail);
  }
</script>

<!-- Basic Tabs -->
<Tabs {tabs} bind:value={activeTab} on:change={handleChange}>
  <TabPanel id="overview">
    <h3>Overview Content</h3>
  </TabPanel>
  <TabPanel id="targets">
    <h3>Targets Content</h3>
  </TabPanel>
  <TabPanel id="scans">
    <h3>Scans Content</h3>
  </TabPanel>
  <TabPanel id="reports">
    <h3>Reports Content</h3>
  </TabPanel>
</Tabs>

<!-- Pills Variant -->
<Tabs {tabs} variant="pills" bind:value={activeTab}>
  <!-- Tab panels -->
</Tabs>

<!-- Enclosed Variant -->
<Tabs {tabs} variant="enclosed" bind:value={activeTab}>
  <!-- Tab panels -->
</Tabs>

<!-- Vertical Orientation -->
<Tabs {tabs} orientation="vertical" bind:value={activeTab}>
  <!-- Tab panels -->
</Tabs>

<!-- Closeable Tabs -->
<script>
  let dynamicTabs = [
    { id: '1', label: 'Tab 1', closeable: true },
    { id: '2', label: 'Tab 2', closeable: true },
    { id: '3', label: 'Tab 3', closeable: true }
  ];

  function handleClose(event) {
    const tabId = event.detail;
    dynamicTabs = dynamicTabs.filter(t => t.id !== tabId);
  }
</script>

<Tabs tabs={dynamicTabs} on:close={handleClose}>
  <!-- Tab panels -->
</Tabs>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Tabs.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Tabs from '@components/ui/Tabs/Tabs.svelte';

const tabs = [
  { id: 'tab1', label: 'Tab 1' },
  { id: 'tab2', label: 'Tab 2' },
  { id: 'tab3', label: 'Tab 3', disabled: true }
];

describe('Tabs', () => {
  it('should render all tabs', () => {
    const { getAllByRole } = render(Tabs, { props: { tabs } });
    expect(getAllByRole('tab')).toHaveLength(3);
  });

  it('should have first tab selected by default', () => {
    const { getByRole } = render(Tabs, { props: { tabs } });
    expect(getByRole('tab', { name: 'Tab 1' })).toHaveAttribute('aria-selected', 'true');
  });

  it('should change tab on click', async () => {
    const handleChange = vi.fn();
    const { getByRole, component } = render(Tabs, { props: { tabs } });

    component.$on('change', handleChange);
    await fireEvent.click(getByRole('tab', { name: 'Tab 2' }));

    expect(handleChange).toHaveBeenCalledWith(
      expect.objectContaining({ detail: 'tab2' })
    );
  });

  it('should not change to disabled tab', async () => {
    const handleChange = vi.fn();
    const { getByRole, component } = render(Tabs, { props: { tabs } });

    component.$on('change', handleChange);
    await fireEvent.click(getByRole('tab', { name: 'Tab 3' }));

    expect(handleChange).not.toHaveBeenCalled();
  });

  it('should navigate with arrow keys', async () => {
    const { getByRole } = render(Tabs, { props: { tabs } });

    const tab1 = getByRole('tab', { name: 'Tab 1' });
    await fireEvent.keyDown(tab1, { key: 'ArrowRight' });

    expect(getByRole('tab', { name: 'Tab 2' })).toHaveFocus();
  });

  it('should navigate to first tab on Home key', async () => {
    const { getByRole } = render(Tabs, { props: { tabs, value: 'tab2' } });

    const tab2 = getByRole('tab', { name: 'Tab 2' });
    tab2.focus();
    await fireEvent.keyDown(tab2, { key: 'Home' });

    expect(getByRole('tab', { name: 'Tab 1' })).toHaveFocus();
  });

  it('should have correct ARIA attributes', () => {
    const { getByRole, getAllByRole } = render(Tabs, { props: { tabs } });

    const tablist = getAllByRole('tablist')[0];
    expect(tablist).toHaveAttribute('aria-orientation', 'horizontal');

    const tab1 = getByRole('tab', { name: 'Tab 1' });
    expect(tab1).toHaveAttribute('aria-controls', 'panel-tab1');
  });

  it('should emit close event for closeable tabs', async () => {
    const closeableTabs = [
      { id: 'tab1', label: 'Tab 1', closeable: true }
    ];

    const handleClose = vi.fn();
    const { getByLabelText, component } = render(Tabs, {
      props: { tabs: closeableTabs }
    });

    component.$on('close', handleClose);
    await fireEvent.click(getByLabelText('Close Tab 1 tab'));

    expect(handleClose).toHaveBeenCalledWith(
      expect.objectContaining({ detail: 'tab1' })
    );
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [206-accordion-component.md](./206-accordion-component.md) - Accordion component
- [214-keyboard-shortcuts.md](./214-keyboard-shortcuts.md) - Keyboard navigation
