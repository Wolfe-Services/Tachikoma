# Spec 232: Spec Card Component

## Phase
11 - Spec Browser UI

## Spec ID
232

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Phase 10 (Core UI Components)

## Estimated Context
~8%

---

## Objective

Create a reusable card component for displaying spec information in grid view. The card should show key metadata, status indicators, dependency count, and support hover interactions with quick actions.

---

## Acceptance Criteria

- [ ] Card displays spec ID, title, status, and phase
- [ ] Dependency count badge shows incoming/outgoing deps
- [ ] Hover state reveals quick action buttons
- [ ] Card supports selection visual state
- [ ] Progress indicator shows completion status
- [ ] Tags display with overflow handling
- [ ] Accessible with proper ARIA attributes
- [ ] Responsive sizing based on container

---

## Implementation Details

### SpecCard.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import type { Spec } from '$lib/types/spec';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import ProgressRing from '$lib/components/ProgressRing.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Tooltip from '$lib/components/Tooltip.svelte';
  import { formatRelativeTime } from '$lib/utils/date';
  import { calculateSpecProgress } from '$lib/utils/spec';

  export let spec: Spec;
  export let selected = false;
  export let width: number | 'auto' = 'auto';
  export let showProgress = true;
  export let showTags = true;

  const dispatch = createEventDispatcher<{
    click: MouseEvent;
    dblclick: void;
    contextmenu: MouseEvent;
    action: { action: string; spec: Spec };
  }>();

  let isHovered = false;
  let cardRef: HTMLElement;

  $: progress = calculateSpecProgress(spec);
  $: incomingDeps = spec.dependencies?.length ?? 0;
  $: outgoingDeps = spec.dependents?.length ?? 0;
  $: visibleTags = spec.tags?.slice(0, 3) ?? [];
  $: remainingTagCount = (spec.tags?.length ?? 0) - visibleTags.length;

  function handleQuickAction(action: string, event: MouseEvent) {
    event.stopPropagation();
    dispatch('action', { action, spec });
  }

  function getPhaseColor(phase: number): string {
    const colors = [
      'var(--color-phase-1)',
      'var(--color-phase-2)',
      'var(--color-phase-3)',
      'var(--color-phase-4)',
      'var(--color-phase-5)',
    ];
    return colors[(phase - 1) % colors.length] || 'var(--color-neutral)';
  }

  function getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      'planned': 'var(--color-status-planned)',
      'in-progress': 'var(--color-status-progress)',
      'implemented': 'var(--color-status-implemented)',
      'tested': 'var(--color-status-tested)',
      'deprecated': 'var(--color-status-deprecated)',
    };
    return colors[status] || 'var(--color-neutral)';
  }
</script>

<article
  bind:this={cardRef}
  class="spec-card"
  class:spec-card--selected={selected}
  class:spec-card--hovered={isHovered}
  style:width={width === 'auto' ? 'auto' : `${width}px`}
  role="option"
  aria-selected={selected}
  tabindex="0"
  on:click={(e) => dispatch('click', e)}
  on:dblclick={() => dispatch('dblclick')}
  on:contextmenu|preventDefault={(e) => dispatch('contextmenu', e)}
  on:mouseenter={() => isHovered = true}
  on:mouseleave={() => isHovered = false}
  on:focus={() => isHovered = true}
  on:blur={() => isHovered = false}
>
  <!-- Phase indicator stripe -->
  <div
    class="spec-card__phase-stripe"
    style:background-color={getPhaseColor(spec.phase)}
    aria-hidden="true"
  />

  <header class="spec-card__header">
    <span class="spec-card__id">{spec.id}</span>
    <div class="spec-card__header-right">
      {#if showProgress}
        <ProgressRing
          value={progress}
          size={24}
          strokeWidth={2}
          color={getStatusColor(spec.status)}
        />
      {/if}
      <StatusBadge status={spec.status} size="sm" />
    </div>
  </header>

  <div class="spec-card__body">
    <h3 class="spec-card__title">{spec.title}</h3>
    {#if spec.description}
      <p class="spec-card__description">{spec.description}</p>
    {/if}
  </div>

  <div class="spec-card__meta">
    <div class="spec-card__dependencies">
      <Tooltip content="Dependencies: {incomingDeps} in / {outgoingDeps} out">
        <div class="spec-card__dep-badge">
          <Icon name="arrow-left" size={12} />
          <span>{incomingDeps}</span>
          <span class="spec-card__dep-divider">/</span>
          <span>{outgoingDeps}</span>
          <Icon name="arrow-right" size={12} />
        </div>
      </Tooltip>
    </div>

    <div class="spec-card__phase-badge">
      Phase {spec.phase}
    </div>
  </div>

  {#if showTags && visibleTags.length > 0}
    <div class="spec-card__tags">
      {#each visibleTags as tag}
        <span class="spec-card__tag">{tag}</span>
      {/each}
      {#if remainingTagCount > 0}
        <span class="spec-card__tag spec-card__tag--more">
          +{remainingTagCount}
        </span>
      {/if}
    </div>
  {/if}

  <footer class="spec-card__footer">
    <span class="spec-card__date">
      Updated {formatRelativeTime(spec.updatedAt)}
    </span>
    {#if spec.estimatedContext}
      <span class="spec-card__context">
        {spec.estimatedContext}
      </span>
    {/if}
  </footer>

  <!-- Quick actions overlay -->
  {#if isHovered}
    <div class="spec-card__actions" transition:fade={{ duration: 150 }}>
      <button
        class="spec-card__action-btn"
        on:click={(e) => handleQuickAction('edit', e)}
        aria-label="Edit spec"
      >
        <Icon name="edit" size={16} />
      </button>
      <button
        class="spec-card__action-btn"
        on:click={(e) => handleQuickAction('duplicate', e)}
        aria-label="Duplicate spec"
      >
        <Icon name="copy" size={16} />
      </button>
      <button
        class="spec-card__action-btn"
        on:click={(e) => handleQuickAction('view-deps', e)}
        aria-label="View dependencies"
      >
        <Icon name="git-branch" size={16} />
      </button>
      <button
        class="spec-card__action-btn spec-card__action-btn--danger"
        on:click={(e) => handleQuickAction('delete', e)}
        aria-label="Delete spec"
      >
        <Icon name="trash" size={16} />
      </button>
    </div>
  {/if}
</article>

<style>
  .spec-card {
    position: relative;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition:
      box-shadow 0.2s ease,
      border-color 0.2s ease,
      transform 0.15s ease;
  }

  .spec-card:hover,
  .spec-card:focus {
    border-color: var(--color-border-hover);
    box-shadow: var(--shadow-md);
    outline: none;
  }

  .spec-card:focus {
    border-color: var(--color-primary);
  }

  .spec-card--selected {
    border-color: var(--color-primary);
    background: var(--color-selected);
  }

  .spec-card--hovered {
    transform: translateY(-2px);
  }

  .spec-card__phase-stripe {
    height: 4px;
    width: 100%;
  }

  .spec-card__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px 8px;
  }

  .spec-card__id {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .spec-card__header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .spec-card__body {
    padding: 0 16px 12px;
    flex: 1;
  }

  .spec-card__title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 4px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .spec-card__description {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .spec-card__meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px 8px;
  }

  .spec-card__dep-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    padding: 2px 8px;
    background: var(--color-surface-elevated);
    border-radius: 4px;
  }

  .spec-card__dep-divider {
    color: var(--color-border);
  }

  .spec-card__phase-badge {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-tertiary);
  }

  .spec-card__tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 0 16px 8px;
  }

  .spec-card__tag {
    font-size: 0.625rem;
    padding: 2px 6px;
    background: var(--color-tag-bg);
    color: var(--color-tag-text);
    border-radius: 3px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .spec-card__tag--more {
    background: var(--color-surface-elevated);
    color: var(--color-text-tertiary);
  }

  .spec-card__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-card__date {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-card__context {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
  }

  .spec-card__actions {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    gap: 4px;
    padding: 4px;
    background: var(--color-surface);
    border-radius: 6px;
    box-shadow: var(--shadow-lg);
  }

  .spec-card__action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-secondary);
    transition:
      background-color 0.15s ease,
      color 0.15s ease;
  }

  .spec-card__action-btn:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .spec-card__action-btn--danger:hover {
    background: var(--color-danger-subtle);
    color: var(--color-danger);
  }
</style>
```

### ProgressRing.svelte

```svelte
<script lang="ts">
  export let value = 0;
  export let size = 24;
  export let strokeWidth = 2;
  export let color = 'var(--color-primary)';
  export let trackColor = 'var(--color-border)';
  export let showLabel = false;

  $: radius = (size - strokeWidth) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: offset = circumference - (value / 100) * circumference;
  $: center = size / 2;
</script>

<svg
  class="progress-ring"
  width={size}
  height={size}
  viewBox="0 0 {size} {size}"
  role="progressbar"
  aria-valuenow={value}
  aria-valuemin="0"
  aria-valuemax="100"
>
  <circle
    class="progress-ring__track"
    cx={center}
    cy={center}
    r={radius}
    fill="none"
    stroke={trackColor}
    stroke-width={strokeWidth}
  />
  <circle
    class="progress-ring__progress"
    cx={center}
    cy={center}
    r={radius}
    fill="none"
    stroke={color}
    stroke-width={strokeWidth}
    stroke-dasharray={circumference}
    stroke-dashoffset={offset}
    stroke-linecap="round"
    transform="rotate(-90 {center} {center})"
  />
  {#if showLabel}
    <text
      class="progress-ring__label"
      x={center}
      y={center}
      text-anchor="middle"
      dominant-baseline="central"
      font-size={size * 0.3}
    >
      {Math.round(value)}%
    </text>
  {/if}
</svg>

<style>
  .progress-ring {
    display: block;
  }

  .progress-ring__progress {
    transition: stroke-dashoffset 0.3s ease;
  }

  .progress-ring__label {
    fill: var(--color-text-primary);
    font-weight: 600;
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import SpecCard from './SpecCard.svelte';
import { createMockSpec } from '$lib/test-utils/mock-data';

describe('SpecCard', () => {
  const mockSpec = createMockSpec({
    id: '232',
    title: 'Spec Card Component',
    status: 'in-progress',
    phase: 11,
    dependencies: ['231'],
    tags: ['ui', 'component', 'svelte', 'extra'],
  });

  it('renders spec information correctly', () => {
    render(SpecCard, { props: { spec: mockSpec } });

    expect(screen.getByText('232')).toBeInTheDocument();
    expect(screen.getByText('Spec Card Component')).toBeInTheDocument();
    expect(screen.getByText('Phase 11')).toBeInTheDocument();
  });

  it('displays status badge', () => {
    render(SpecCard, { props: { spec: mockSpec } });

    expect(screen.getByText('in-progress')).toBeInTheDocument();
  });

  it('shows dependency count', () => {
    render(SpecCard, { props: { spec: mockSpec } });

    expect(screen.getByText('1')).toBeInTheDocument(); // incoming deps
  });

  it('limits visible tags to 3', () => {
    render(SpecCard, { props: { spec: mockSpec } });

    expect(screen.getByText('ui')).toBeInTheDocument();
    expect(screen.getByText('component')).toBeInTheDocument();
    expect(screen.getByText('svelte')).toBeInTheDocument();
    expect(screen.getByText('+1')).toBeInTheDocument();
  });

  it('shows quick actions on hover', async () => {
    render(SpecCard, { props: { spec: mockSpec } });

    const card = screen.getByRole('option');
    await fireEvent.mouseEnter(card);

    expect(screen.getByLabelText('Edit spec')).toBeInTheDocument();
    expect(screen.getByLabelText('Duplicate spec')).toBeInTheDocument();
  });

  it('dispatches action events', async () => {
    const { component } = render(SpecCard, { props: { spec: mockSpec } });

    const actionHandler = vi.fn();
    component.$on('action', actionHandler);

    const card = screen.getByRole('option');
    await fireEvent.mouseEnter(card);
    await fireEvent.click(screen.getByLabelText('Edit spec'));

    expect(actionHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { action: 'edit', spec: mockSpec }
      })
    );
  });

  it('applies selected state styling', () => {
    render(SpecCard, { props: { spec: mockSpec, selected: true } });

    const card = screen.getByRole('option');
    expect(card).toHaveClass('spec-card--selected');
    expect(card).toHaveAttribute('aria-selected', 'true');
  });

  it('supports custom width', () => {
    render(SpecCard, { props: { spec: mockSpec, width: 320 } });

    const card = screen.getByRole('option');
    expect(card).toHaveStyle({ width: '320px' });
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 236: Spec Detail View
- Spec 242: Spec Status Tracking
- Spec 243: Dependency Visualization
