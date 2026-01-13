# 246 - Spec Breadcrumbs

**Phase:** 11 - Spec Browser UI
**Spec ID:** 246
**Status:** Planned
**Dependencies:** 236-spec-browser-layout
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create a breadcrumb navigation component that shows the current location in the spec hierarchy and allows quick navigation to parent sections.

---

## Acceptance Criteria

- [x] Show phase > spec path
- [x] Clickable navigation links
- [x] Current item highlighted
- [x] Overflow handling with ellipsis
- [x] Dropdown for long paths
- [x] Keyboard accessible

---

## Implementation Details

### 1. Spec Breadcrumbs Component (src/lib/components/spec-browser/SpecBreadcrumbs.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { ipcRenderer } from '$lib/ipc';

  export let specId: string | null;

  const dispatch = createEventDispatcher<{
    navigate: string | null;
  }>();

  interface BreadcrumbItem {
    id: string;
    label: string;
    type: 'root' | 'phase' | 'spec';
  }

  let breadcrumbs: BreadcrumbItem[] = [];
  let showOverflow = false;

  async function loadBreadcrumbs() {
    if (!specId) {
      breadcrumbs = [{ id: 'root', label: 'Specifications', type: 'root' }];
      return;
    }

    const spec = await ipcRenderer.invoke('spec:get', specId);
    if (spec) {
      breadcrumbs = [
        { id: 'root', label: 'Specifications', type: 'root' },
        { id: `phase-${spec.phase}`, label: `Phase ${spec.phase}`, type: 'phase' },
        { id: spec.id, label: `${spec.number} - ${spec.title}`, type: 'spec' },
      ];
    }
  }

  function navigate(item: BreadcrumbItem) {
    if (item.type === 'root') {
      dispatch('navigate', null);
    } else if (item.type === 'phase') {
      // Navigate to phase overview
      dispatch('navigate', item.id);
    } else {
      dispatch('navigate', item.id);
    }
  }

  $: if (specId !== undefined) loadBreadcrumbs();
</script>

<nav class="spec-breadcrumbs" aria-label="Breadcrumb">
  <ol class="breadcrumb-list">
    {#each breadcrumbs as item, index}
      <li class="breadcrumb-item">
        {#if index < breadcrumbs.length - 1}
          <button
            class="breadcrumb-link"
            on:click={() => navigate(item)}
          >
            {item.label}
          </button>
          <span class="breadcrumb-separator">/</span>
        {:else}
          <span class="breadcrumb-current" aria-current="page">
            {item.label}
          </span>
        {/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
  .spec-breadcrumbs {
    flex: 1;
    min-width: 0;
  }

  .breadcrumb-list {
    display: flex;
    align-items: center;
    list-style: none;
    padding: 0;
    margin: 0;
    overflow: hidden;
  }

  .breadcrumb-item {
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .breadcrumb-link {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    border-radius: 4px;
    white-space: nowrap;
  }

  .breadcrumb-link:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .breadcrumb-separator {
    color: var(--color-text-muted);
    margin: 0 4px;
  }

  .breadcrumb-current {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
```

---

## Testing Requirements

1. Breadcrumbs display path
2. Navigation works
3. Current item highlighted
4. Overflow handled

---

## Related Specs

- Depends on: [236-spec-browser-layout.md](236-spec-browser-layout.md)
- Next: [247-spec-metadata.md](247-spec-metadata.md)
