# 241 - Implementation Plan Viewer

**Phase:** 11 - Spec Browser UI
**Spec ID:** 241
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 239-markdown-renderer
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create an implementation plan viewer that parses and displays the implementation checklist from specs, allowing users to track progress and update completion status.

---

## Acceptance Criteria

- [x] Parse implementation section from markdown
- [x] Display checkboxes with status
- [x] Update checkbox state
- [x] Progress percentage calculation
- [x] Group by section/category
- [x] Filter incomplete items
- [x] Export progress report

---

## Implementation Details

### 1. Types (src/lib/types/impl-plan.ts)

```typescript
export interface ImplementationPlan {
  specId: string;
  title: string;
  sections: PlanSection[];
  progress: PlanProgress;
}

export interface PlanSection {
  id: string;
  title: string;
  items: PlanItem[];
}

export interface PlanItem {
  id: string;
  text: string;
  completed: boolean;
  lineNumber: number;
  subItems?: PlanItem[];
}

export interface PlanProgress {
  completed: number;
  total: number;
  percentage: number;
}
```

### 2. Implementation Plan Viewer Component (src/lib/components/spec-browser/ImplPlanViewer.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ImplementationPlan, PlanSection, PlanItem } from '$lib/types/impl-plan';

  export let plan: ImplementationPlan;
  export let editable = true;

  const dispatch = createEventDispatcher<{
    toggle: { itemId: string; completed: boolean };
  }>();

  let showOnlyIncomplete = false;

  function toggleItem(item: PlanItem) {
    if (!editable) return;
    dispatch('toggle', { itemId: item.id, completed: !item.completed });
  }

  function getFilteredItems(items: PlanItem[]): PlanItem[] {
    if (!showOnlyIncomplete) return items;
    return items.filter(item => !item.completed || item.subItems?.some(s => !s.completed));
  }

  $: visibleSections = plan.sections.map(section => ({
    ...section,
    items: getFilteredItems(section.items),
  })).filter(section => section.items.length > 0);
</script>

<div class="impl-plan-viewer">
  <header class="impl-plan-viewer__header">
    <h3>Implementation Plan</h3>

    <div class="progress-badge">
      <div class="progress-bar">
        <div
          class="progress-bar__fill"
          style="width: {plan.progress.percentage}%"
        ></div>
      </div>
      <span>{plan.progress.completed}/{plan.progress.total}</span>
    </div>

    <label class="filter-toggle">
      <input
        type="checkbox"
        bind:checked={showOnlyIncomplete}
      />
      Show incomplete only
    </label>
  </header>

  <div class="impl-plan-viewer__sections">
    {#each visibleSections as section}
      <div class="plan-section">
        <h4 class="plan-section__title">{section.title}</h4>

        <ul class="plan-section__items">
          {#each section.items as item}
            <li class="plan-item" class:completed={item.completed}>
              <label class="plan-item__checkbox">
                <input
                  type="checkbox"
                  checked={item.completed}
                  disabled={!editable}
                  on:change={() => toggleItem(item)}
                />
                <span class="plan-item__text">{item.text}</span>
              </label>

              {#if item.subItems?.length}
                <ul class="plan-item__subitems">
                  {#each item.subItems as subItem}
                    <li class="plan-item plan-item--sub" class:completed={subItem.completed}>
                      <label class="plan-item__checkbox">
                        <input
                          type="checkbox"
                          checked={subItem.completed}
                          disabled={!editable}
                          on:change={() => toggleItem(subItem)}
                        />
                        <span class="plan-item__text">{subItem.text}</span>
                      </label>
                    </li>
                  {/each}
                </ul>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>
</div>

<style>
  .impl-plan-viewer {
    height: 100%;
    overflow-y: auto;
  }

  .impl-plan-viewer__header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    position: sticky;
    top: 0;
  }

  .impl-plan-viewer__header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .progress-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .progress-bar {
    width: 100px;
    height: 6px;
    background: var(--color-bg-hover);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-bar__fill {
    height: 100%;
    background: var(--color-success);
    transition: width 0.3s ease;
  }

  .filter-toggle {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .impl-plan-viewer__sections {
    padding: 16px;
  }

  .plan-section {
    margin-bottom: 24px;
  }

  .plan-section__title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 12px 0;
    text-transform: uppercase;
  }

  .plan-section__items {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .plan-item {
    margin-bottom: 8px;
  }

  .plan-item.completed .plan-item__text {
    text-decoration: line-through;
    color: var(--color-text-muted);
  }

  .plan-item__checkbox {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    cursor: pointer;
  }

  .plan-item__checkbox input {
    margin-top: 3px;
    cursor: pointer;
  }

  .plan-item__text {
    font-size: 14px;
    color: var(--color-text-primary);
    line-height: 1.4;
  }

  .plan-item__subitems {
    list-style: none;
    padding: 0;
    margin: 8px 0 0 24px;
  }

  .plan-item--sub {
    margin-bottom: 4px;
  }

  .plan-item--sub .plan-item__text {
    font-size: 13px;
  }
</style>
```

---

## Testing Requirements

1. Parses markdown checkboxes
2. Progress calculates correctly
3. Toggle updates state
4. Filter works properly
5. Nested items display

---

## Related Specs

- Depends on: [239-markdown-renderer.md](239-markdown-renderer.md)
- Next: [242-impl-checkbox.md](242-impl-checkbox.md)
