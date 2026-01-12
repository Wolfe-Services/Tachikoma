# Spec 242: Spec Status Tracking

## Phase
11 - Spec Browser UI

## Spec ID
242

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Spec 236 (Spec Detail View)
- Spec 237 (Spec Editor)

## Estimated Context
~8%

---

## Objective

Implement a comprehensive status tracking system for specs with visual indicators, status transition rules, bulk status updates, and status history tracking. Include Kanban-style status board view.

---

## Acceptance Criteria

- [ ] Status badge component with state colors
- [ ] Editable status with dropdown
- [ ] Status transition validation
- [ ] Bulk status update capability
- [ ] Status change history
- [ ] Kanban board view by status
- [ ] Status notifications
- [ ] Status statistics dashboard

---

## Implementation Details

### StatusBadge.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import type { SpecStatus } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import Popover from '$lib/components/Popover.svelte';

  export let status: SpecStatus;
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let editable = false;
  export let showIcon = true;

  const dispatch = createEventDispatcher<{
    change: SpecStatus;
  }>();

  let isOpen = false;

  const statusConfig: Record<SpecStatus, {
    label: string;
    color: string;
    bgColor: string;
    icon: string;
    next: SpecStatus[];
  }> = {
    'planned': {
      label: 'Planned',
      color: 'var(--color-status-planned)',
      bgColor: 'var(--color-status-planned-bg)',
      icon: 'circle',
      next: ['in-progress', 'deprecated']
    },
    'in-progress': {
      label: 'In Progress',
      color: 'var(--color-status-progress)',
      bgColor: 'var(--color-status-progress-bg)',
      icon: 'loader',
      next: ['implemented', 'planned', 'deprecated']
    },
    'implemented': {
      label: 'Implemented',
      color: 'var(--color-status-implemented)',
      bgColor: 'var(--color-status-implemented-bg)',
      icon: 'code',
      next: ['tested', 'in-progress', 'deprecated']
    },
    'tested': {
      label: 'Tested',
      color: 'var(--color-status-tested)',
      bgColor: 'var(--color-status-tested-bg)',
      icon: 'check-circle',
      next: ['deprecated']
    },
    'deprecated': {
      label: 'Deprecated',
      color: 'var(--color-status-deprecated)',
      bgColor: 'var(--color-status-deprecated-bg)',
      icon: 'archive',
      next: ['planned']
    }
  };

  $: config = statusConfig[status];
  $: allowedTransitions = config.next;

  function handleStatusChange(newStatus: SpecStatus) {
    if (allowedTransitions.includes(newStatus)) {
      dispatch('change', newStatus);
      isOpen = false;
    }
  }

  function getSizeClasses(size: string): string {
    return {
      sm: 'status-badge--sm',
      md: 'status-badge--md',
      lg: 'status-badge--lg'
    }[size] || 'status-badge--md';
  }
</script>

{#if editable}
  <Popover bind:open={isOpen} placement="bottom-start">
    <button
      slot="trigger"
      class="status-badge {getSizeClasses(size)} status-badge--editable"
      style:--status-color={config.color}
      style:--status-bg={config.bgColor}
    >
      {#if showIcon}
        <Icon name={config.icon} size={size === 'sm' ? 10 : size === 'lg' ? 16 : 12} />
      {/if}
      <span>{config.label}</span>
      <Icon name="chevron-down" size={10} />
    </button>

    <div class="status-badge__menu">
      <div class="status-badge__menu-header">
        Change Status
      </div>
      <div class="status-badge__options">
        {#each Object.entries(statusConfig) as [key, value]}
          {@const statusKey = key as SpecStatus}
          {@const isAllowed = allowedTransitions.includes(statusKey)}
          {@const isCurrent = statusKey === status}
          <button
            class="status-badge__option"
            class:status-badge__option--current={isCurrent}
            class:status-badge__option--disabled={!isAllowed && !isCurrent}
            disabled={!isAllowed && !isCurrent}
            on:click={() => handleStatusChange(statusKey)}
          >
            <span
              class="status-badge__option-dot"
              style:background={value.color}
            />
            <span class="status-badge__option-label">{value.label}</span>
            {#if isCurrent}
              <Icon name="check" size={14} />
            {/if}
          </button>
        {/each}
      </div>
      <div class="status-badge__menu-footer">
        <span class="status-badge__hint">
          {allowedTransitions.length} available transition{allowedTransitions.length !== 1 ? 's' : ''}
        </span>
      </div>
    </div>
  </Popover>
{:else}
  <span
    class="status-badge {getSizeClasses(size)}"
    style:--status-color={config.color}
    style:--status-bg={config.bgColor}
  >
    {#if showIcon}
      <Icon name={config.icon} size={size === 'sm' ? 10 : size === 'lg' ? 16 : 12} />
    {/if}
    <span>{config.label}</span>
  </span>
{/if}

<style>
  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    font-weight: 500;
    border-radius: 4px;
    background: var(--status-bg);
    color: var(--status-color);
    white-space: nowrap;
  }

  .status-badge--sm {
    padding: 2px 6px;
    font-size: 0.625rem;
  }

  .status-badge--md {
    padding: 4px 8px;
    font-size: 0.75rem;
  }

  .status-badge--lg {
    padding: 6px 12px;
    font-size: 0.875rem;
  }

  .status-badge--editable {
    cursor: pointer;
    border: none;
    transition: all 0.15s;
  }

  .status-badge--editable:hover {
    filter: brightness(0.95);
  }

  .status-badge__menu {
    width: 200px;
    padding: 8px 0;
  }

  .status-badge__menu-header {
    padding: 8px 12px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
  }

  .status-badge__options {
    padding: 4px 0;
  }

  .status-badge__option {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }

  .status-badge__option:hover:not(:disabled) {
    background: var(--color-hover);
  }

  .status-badge__option--current {
    background: var(--color-primary-subtle);
  }

  .status-badge__option--disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .status-badge__option-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-badge__option-label {
    flex: 1;
  }

  .status-badge__menu-footer {
    padding: 8px 12px;
    border-top: 1px solid var(--color-border);
  }

  .status-badge__hint {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }
</style>
```

### StatusKanban.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { flip } from 'svelte/animate';
  import { dndzone } from 'svelte-dnd-action';
  import type { Spec, SpecStatus } from '$lib/types/spec';
  import SpecCard from './SpecCard.svelte';
  import Icon from '$lib/components/Icon.svelte';

  export let specs: Spec[] = [];

  const dispatch = createEventDispatcher<{
    statusChange: { spec: Spec; newStatus: SpecStatus };
    select: Spec;
  }>();

  const columns: { status: SpecStatus; label: string; color: string }[] = [
    { status: 'planned', label: 'Planned', color: 'var(--color-status-planned)' },
    { status: 'in-progress', label: 'In Progress', color: 'var(--color-status-progress)' },
    { status: 'implemented', label: 'Implemented', color: 'var(--color-status-implemented)' },
    { status: 'tested', label: 'Tested', color: 'var(--color-status-tested)' },
    { status: 'deprecated', label: 'Deprecated', color: 'var(--color-status-deprecated)' }
  ];

  // Group specs by status
  $: columnSpecs = columns.reduce((acc, col) => {
    acc[col.status] = specs
      .filter(s => s.status === col.status)
      .map(s => ({ ...s, id: s.id }));
    return acc;
  }, {} as Record<SpecStatus, Spec[]>);

  function handleDndConsider(status: SpecStatus, e: CustomEvent) {
    columnSpecs[status] = e.detail.items;
  }

  function handleDndFinalize(status: SpecStatus, e: CustomEvent) {
    const items = e.detail.items as Spec[];
    columnSpecs[status] = items;

    // Find the dropped item and dispatch status change
    const info = e.detail.info;
    if (info.trigger === 'droppedIntoZone') {
      const droppedItem = items.find(item =>
        item.status !== status
      );
      if (droppedItem) {
        dispatch('statusChange', { spec: droppedItem, newStatus: status });
      }
    }
  }

  function getColumnCount(status: SpecStatus): number {
    return columnSpecs[status]?.length ?? 0;
  }
</script>

<div class="status-kanban">
  {#each columns as column}
    <div class="status-kanban__column">
      <header
        class="status-kanban__column-header"
        style:--column-color={column.color}
      >
        <div class="status-kanban__column-title">
          <span class="status-kanban__column-dot" />
          <h3>{column.label}</h3>
        </div>
        <span class="status-kanban__column-count">
          {getColumnCount(column.status)}
        </span>
      </header>

      <div
        class="status-kanban__column-body"
        use:dndzone={{
          items: columnSpecs[column.status],
          flipDurationMs: 200,
          dropTargetStyle: { outline: `2px dashed ${column.color}` }
        }}
        on:consider={(e) => handleDndConsider(column.status, e)}
        on:finalize={(e) => handleDndFinalize(column.status, e)}
      >
        {#each columnSpecs[column.status] as spec (spec.id)}
          <div animate:flip={{ duration: 200 }}>
            <div
              class="status-kanban__card"
              on:click={() => dispatch('select', spec)}
              on:keydown={(e) => e.key === 'Enter' && dispatch('select', spec)}
              role="button"
              tabindex="0"
            >
              <div class="status-kanban__card-id">{spec.id}</div>
              <div class="status-kanban__card-title">{spec.title}</div>
              <div class="status-kanban__card-meta">
                <span>Phase {spec.phase}</span>
                {#if spec.dependencies?.length}
                  <span>
                    <Icon name="git-branch" size={10} />
                    {spec.dependencies.length}
                  </span>
                {/if}
              </div>
            </div>
          </div>
        {/each}

        {#if columnSpecs[column.status].length === 0}
          <div class="status-kanban__empty">
            <Icon name="inbox" size={24} />
            <span>No specs</span>
          </div>
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .status-kanban {
    display: flex;
    gap: 16px;
    height: 100%;
    padding: 16px;
    overflow-x: auto;
  }

  .status-kanban__column {
    flex: 0 0 280px;
    display: flex;
    flex-direction: column;
    background: var(--color-surface-subtle);
    border-radius: 8px;
    overflow: hidden;
  }

  .status-kanban__column-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--color-surface);
    border-bottom: 2px solid var(--column-color);
  }

  .status-kanban__column-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-kanban__column-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--column-color);
  }

  .status-kanban__column-title h3 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0;
  }

  .status-kanban__column-count {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    font-size: 0.75rem;
    font-weight: 600;
    background: var(--color-surface-elevated);
    border-radius: 12px;
    color: var(--color-text-secondary);
  }

  .status-kanban__column-body {
    flex: 1;
    padding: 12px;
    overflow-y: auto;
    min-height: 100px;
  }

  .status-kanban__card {
    padding: 12px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    margin-bottom: 8px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .status-kanban__card:hover {
    border-color: var(--color-primary);
    box-shadow: var(--shadow-sm);
  }

  .status-kanban__card:last-child {
    margin-bottom: 0;
  }

  .status-kanban__card-id {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-primary);
    margin-bottom: 4px;
  }

  .status-kanban__card-title {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
    margin-bottom: 8px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .status-kanban__card-meta {
    display: flex;
    gap: 12px;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .status-kanban__card-meta span {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-kanban__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px;
    color: var(--color-text-tertiary);
    font-size: 0.875rem;
  }
</style>
```

### StatusHistory.svelte

```svelte
<script lang="ts">
  import type { StatusChange } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import { formatRelativeTime } from '$lib/utils/date';

  export let history: StatusChange[] = [];

  function getStatusIcon(status: string): string {
    const icons: Record<string, string> = {
      'planned': 'circle',
      'in-progress': 'loader',
      'implemented': 'code',
      'tested': 'check-circle',
      'deprecated': 'archive'
    };
    return icons[status] || 'circle';
  }
</script>

<div class="status-history">
  <h4 class="status-history__title">
    <Icon name="clock" size={14} />
    Status History
  </h4>

  {#if history.length === 0}
    <p class="status-history__empty">No status changes recorded</p>
  {:else}
    <ul class="status-history__list">
      {#each history as change}
        <li class="status-history__item">
          <div class="status-history__icon">
            <Icon name={getStatusIcon(change.to)} size={14} />
          </div>
          <div class="status-history__content">
            <div class="status-history__change">
              <span class="status-history__from">{change.from}</span>
              <Icon name="arrow-right" size={12} />
              <span class="status-history__to">{change.to}</span>
            </div>
            <div class="status-history__meta">
              <span>{formatRelativeTime(change.timestamp)}</span>
              {#if change.user}
                <span>by {change.user}</span>
              {/if}
            </div>
            {#if change.reason}
              <p class="status-history__reason">{change.reason}</p>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .status-history {
    padding: 16px;
  }

  .status-history__title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 16px;
  }

  .status-history__empty {
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
    text-align: center;
    padding: 24px;
  }

  .status-history__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .status-history__item {
    display: flex;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .status-history__item:last-child {
    border-bottom: none;
  }

  .status-history__icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: var(--color-surface-elevated);
    border-radius: 50%;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .status-history__content {
    flex: 1;
    min-width: 0;
  }

  .status-history__change {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
  }

  .status-history__from {
    color: var(--color-text-tertiary);
    text-decoration: line-through;
  }

  .status-history__to {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .status-history__meta {
    display: flex;
    gap: 8px;
    margin-top: 4px;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .status-history__reason {
    margin: 8px 0 0;
    padding: 8px;
    font-size: 0.875rem;
    background: var(--color-surface-subtle);
    border-radius: 4px;
    color: var(--color-text-secondary);
  }
</style>
```

### Status Types

```typescript
// types/spec.ts additions
export interface StatusChange {
  from: SpecStatus;
  to: SpecStatus;
  timestamp: Date;
  user?: string;
  reason?: string;
}

export interface StatusTransitionRule {
  from: SpecStatus;
  to: SpecStatus;
  requiresReason?: boolean;
  requiresApproval?: boolean;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import StatusBadge from './StatusBadge.svelte';
import StatusKanban from './StatusKanban.svelte';
import StatusHistory from './StatusHistory.svelte';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('StatusBadge', () => {
  it('renders status label and color', () => {
    render(StatusBadge, { props: { status: 'planned' } });

    expect(screen.getByText('Planned')).toBeInTheDocument();
  });

  it('shows dropdown when editable', async () => {
    render(StatusBadge, { props: { status: 'planned', editable: true } });

    await fireEvent.click(screen.getByText('Planned'));

    expect(screen.getByText('Change Status')).toBeInTheDocument();
  });

  it('shows only valid transitions', async () => {
    render(StatusBadge, { props: { status: 'planned', editable: true } });

    await fireEvent.click(screen.getByText('Planned'));

    // From 'planned', can go to 'in-progress' or 'deprecated'
    expect(screen.getByText('In Progress')).not.toBeDisabled();
    expect(screen.getByText('Deprecated')).not.toBeDisabled();

    // Cannot go directly to 'tested'
    const testedOption = screen.getByText('Tested').closest('button');
    expect(testedOption).toHaveClass('status-badge__option--disabled');
  });

  it('dispatches change event', async () => {
    const { component } = render(StatusBadge, {
      props: { status: 'planned', editable: true }
    });

    const changeHandler = vi.fn();
    component.$on('change', changeHandler);

    await fireEvent.click(screen.getByText('Planned'));
    await fireEvent.click(screen.getByText('In Progress'));

    expect(changeHandler).toHaveBeenCalledWith(
      expect.objectContaining({ detail: 'in-progress' })
    );
  });
});

describe('StatusKanban', () => {
  const specs = createMockSpecs(10);

  it('renders all status columns', () => {
    render(StatusKanban, { props: { specs } });

    expect(screen.getByText('Planned')).toBeInTheDocument();
    expect(screen.getByText('In Progress')).toBeInTheDocument();
    expect(screen.getByText('Implemented')).toBeInTheDocument();
    expect(screen.getByText('Tested')).toBeInTheDocument();
    expect(screen.getByText('Deprecated')).toBeInTheDocument();
  });

  it('groups specs by status', () => {
    const testSpecs = [
      { ...specs[0], status: 'planned' },
      { ...specs[1], status: 'planned' },
      { ...specs[2], status: 'in-progress' }
    ];

    render(StatusKanban, { props: { specs: testSpecs } });

    // Check column counts
    const plannedColumn = screen.getByText('Planned').closest('.status-kanban__column');
    expect(plannedColumn).toContainElement(screen.getByText('2'));
  });

  it('dispatches select on card click', async () => {
    const { component } = render(StatusKanban, { props: { specs } });

    const selectHandler = vi.fn();
    component.$on('select', selectHandler);

    const firstCard = screen.getAllByRole('button')[0];
    await fireEvent.click(firstCard);

    expect(selectHandler).toHaveBeenCalled();
  });
});

describe('StatusHistory', () => {
  const history = [
    {
      from: 'planned',
      to: 'in-progress',
      timestamp: new Date('2024-01-15'),
      user: 'John'
    },
    {
      from: 'in-progress',
      to: 'implemented',
      timestamp: new Date('2024-01-20'),
      user: 'Jane',
      reason: 'Completed implementation'
    }
  ];

  it('renders history entries', () => {
    render(StatusHistory, { props: { history } });

    expect(screen.getByText('planned')).toBeInTheDocument();
    expect(screen.getByText('in-progress')).toBeInTheDocument();
    expect(screen.getByText('by John')).toBeInTheDocument();
  });

  it('shows reason when provided', () => {
    render(StatusHistory, { props: { history } });

    expect(screen.getByText('Completed implementation')).toBeInTheDocument();
  });

  it('shows empty state when no history', () => {
    render(StatusHistory, { props: { history: [] } });

    expect(screen.getByText('No status changes recorded')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 236: Spec Detail View
- Spec 249: Batch Operations
