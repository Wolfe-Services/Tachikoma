# Spec 236: Spec Detail View

## Phase
11 - Spec Browser UI

## Spec ID
236

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Spec 232 (Spec Card Component)
- Spec 238 (Markdown Preview)

## Estimated Context
~12%

---

## Objective

Create a comprehensive detail view for displaying individual specs with full content, metadata, dependencies visualization, history, and quick actions. Support split-pane layout, collapsible sections, and seamless navigation between specs.

---

## Acceptance Criteria

- [ ] Display full spec content with markdown rendering
- [ ] Show all metadata in organized sections
- [ ] Visualize dependencies with clickable links
- [ ] Display acceptance criteria as checklist
- [ ] Show implementation details with code blocks
- [ ] Support keyboard navigation between specs
- [ ] Collapsible sections for long content
- [ ] Quick action toolbar
- [ ] Breadcrumb navigation
- [ ] Print-friendly view option

---

## Implementation Details

### SpecDetailView.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { slide, fade } from 'svelte/transition';
  import type { Spec } from '$lib/types/spec';
  import MarkdownPreview from './MarkdownPreview.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Tooltip from '$lib/components/Tooltip.svelte';
  import DependencyGraph from './DependencyGraph.svelte';
  import AcceptanceCriteria from './AcceptanceCriteria.svelte';
  import SpecHistory from './SpecHistory.svelte';
  import { formatDate, formatRelativeTime } from '$lib/utils/date';
  import { specStore } from '$lib/stores/spec-store';

  export let spec: Spec;
  export let showHistory = false;
  export let showDependencies = true;
  export let readOnly = false;

  const dispatch = createEventDispatcher<{
    close: void;
    edit: Spec;
    duplicate: Spec;
    delete: Spec;
    navigate: { specId: string };
    statusChange: { spec: Spec; status: string };
  }>();

  let containerRef: HTMLElement;
  let collapsedSections = new Set<string>();

  // Derive related specs
  $: dependencySpecs = spec.dependencies
    ?.map(id => $specStore.find(s => s.id === id))
    .filter(Boolean) as Spec[] ?? [];

  $: dependentSpecs = $specStore.filter(s =>
    s.dependencies?.includes(spec.id)
  );

  // Parse sections from content
  $: sections = parseSpecSections(spec.content);

  function parseSpecSections(content: string) {
    const sections: { id: string; title: string; content: string; level: number }[] = [];
    const lines = content.split('\n');
    let currentSection: typeof sections[0] | null = null;
    let sectionContent: string[] = [];

    for (const line of lines) {
      const headerMatch = line.match(/^(#{1,3})\s+(.+)$/);
      if (headerMatch) {
        if (currentSection) {
          currentSection.content = sectionContent.join('\n').trim();
          sections.push(currentSection);
        }
        currentSection = {
          id: headerMatch[2].toLowerCase().replace(/\s+/g, '-'),
          title: headerMatch[2],
          content: '',
          level: headerMatch[1].length
        };
        sectionContent = [];
      } else if (currentSection) {
        sectionContent.push(line);
      }
    }

    if (currentSection) {
      currentSection.content = sectionContent.join('\n').trim();
      sections.push(currentSection);
    }

    return sections;
  }

  function toggleSection(sectionId: string) {
    if (collapsedSections.has(sectionId)) {
      collapsedSections.delete(sectionId);
    } else {
      collapsedSections.add(sectionId);
    }
    collapsedSections = collapsedSections;
  }

  function navigateToSpec(specId: string) {
    dispatch('navigate', { specId });
  }

  function handleKeydown(event: KeyboardEvent) {
    const specs = $specStore;
    const currentIndex = specs.findIndex(s => s.id === spec.id);

    if (event.key === 'ArrowLeft' && event.altKey) {
      // Navigate to previous spec
      if (currentIndex > 0) {
        navigateToSpec(specs[currentIndex - 1].id);
      }
    } else if (event.key === 'ArrowRight' && event.altKey) {
      // Navigate to next spec
      if (currentIndex < specs.length - 1) {
        navigateToSpec(specs[currentIndex + 1].id);
      }
    } else if (event.key === 'Escape') {
      dispatch('close');
    } else if (event.key === 'e' && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      dispatch('edit', spec);
    }
  }

  function handlePrint() {
    window.print();
  }

  onMount(() => {
    containerRef?.focus();
  });
</script>

<article
  bind:this={containerRef}
  class="spec-detail"
  tabindex="-1"
  on:keydown={handleKeydown}
  aria-label="Spec {spec.id} details"
>
  <!-- Header -->
  <header class="spec-detail__header">
    <div class="spec-detail__breadcrumb">
      <button on:click={() => dispatch('close')} class="spec-detail__back">
        <Icon name="arrow-left" size={16} />
        Back to list
      </button>
      <span class="spec-detail__breadcrumb-sep">/</span>
      <span>Phase {spec.phase}</span>
      <span class="spec-detail__breadcrumb-sep">/</span>
      <span>Spec {spec.id}</span>
    </div>

    <div class="spec-detail__actions">
      {#if !readOnly}
        <Tooltip content="Edit (Cmd+E)">
          <Button variant="ghost" size="sm" on:click={() => dispatch('edit', spec)}>
            <Icon name="edit" size={16} />
          </Button>
        </Tooltip>
        <Tooltip content="Duplicate">
          <Button variant="ghost" size="sm" on:click={() => dispatch('duplicate', spec)}>
            <Icon name="copy" size={16} />
          </Button>
        </Tooltip>
      {/if}
      <Tooltip content="Print">
        <Button variant="ghost" size="sm" on:click={handlePrint}>
          <Icon name="printer" size={16} />
        </Button>
      </Tooltip>
      <Tooltip content="Close (Esc)">
        <Button variant="ghost" size="sm" on:click={() => dispatch('close')}>
          <Icon name="x" size={16} />
        </Button>
      </Tooltip>
    </div>
  </header>

  <!-- Title Section -->
  <div class="spec-detail__title-section">
    <div class="spec-detail__id-badge">Spec {spec.id}</div>
    <h1 class="spec-detail__title">{spec.title}</h1>
    <div class="spec-detail__meta-row">
      <StatusBadge status={spec.status} size="md" editable={!readOnly}
        on:change={(e) => dispatch('statusChange', { spec, status: e.detail })}
      />
      <span class="spec-detail__meta-item">
        <Icon name="layers" size={14} />
        Phase {spec.phase}
      </span>
      {#if spec.estimatedContext}
        <span class="spec-detail__meta-item">
          <Icon name="cpu" size={14} />
          {spec.estimatedContext}
        </span>
      {/if}
    </div>
    {#if spec.tags && spec.tags.length > 0}
      <div class="spec-detail__tags">
        {#each spec.tags as tag}
          <span class="spec-detail__tag">{tag}</span>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Main Content -->
  <div class="spec-detail__body">
    <!-- Description -->
    {#if spec.description}
      <section class="spec-detail__section">
        <p class="spec-detail__description">{spec.description}</p>
      </section>
    {/if}

    <!-- Dependencies -->
    {#if showDependencies && (dependencySpecs.length > 0 || dependentSpecs.length > 0)}
      <section class="spec-detail__section">
        <button
          class="spec-detail__section-header"
          on:click={() => toggleSection('dependencies')}
          aria-expanded={!collapsedSections.has('dependencies')}
        >
          <Icon name="git-branch" size={16} />
          <h2>Dependencies</h2>
          <Icon
            name={collapsedSections.has('dependencies') ? 'chevron-down' : 'chevron-up'}
            size={16}
          />
        </button>

        {#if !collapsedSections.has('dependencies')}
          <div class="spec-detail__section-content" transition:slide={{ duration: 200 }}>
            {#if dependencySpecs.length > 0}
              <div class="spec-detail__dep-group">
                <h3>Depends on ({dependencySpecs.length})</h3>
                <div class="spec-detail__dep-list">
                  {#each dependencySpecs as dep}
                    <button
                      class="spec-detail__dep-item"
                      on:click={() => navigateToSpec(dep.id)}
                    >
                      <span class="spec-detail__dep-id">{dep.id}</span>
                      <span class="spec-detail__dep-title">{dep.title}</span>
                      <StatusBadge status={dep.status} size="sm" />
                    </button>
                  {/each}
                </div>
              </div>
            {/if}

            {#if dependentSpecs.length > 0}
              <div class="spec-detail__dep-group">
                <h3>Required by ({dependentSpecs.length})</h3>
                <div class="spec-detail__dep-list">
                  {#each dependentSpecs as dep}
                    <button
                      class="spec-detail__dep-item"
                      on:click={() => navigateToSpec(dep.id)}
                    >
                      <span class="spec-detail__dep-id">{dep.id}</span>
                      <span class="spec-detail__dep-title">{dep.title}</span>
                      <StatusBadge status={dep.status} size="sm" />
                    </button>
                  {/each}
                </div>
              </div>
            {/if}

            <DependencyGraph {spec} specs={$specStore} />
          </div>
        {/if}
      </section>
    {/if}

    <!-- Objective -->
    {#each sections.filter(s => s.title === 'Objective') as section}
      <section class="spec-detail__section">
        <h2 class="spec-detail__section-title">
          <Icon name="target" size={16} />
          Objective
        </h2>
        <div class="spec-detail__section-content">
          <MarkdownPreview content={section.content} />
        </div>
      </section>
    {/each}

    <!-- Acceptance Criteria -->
    {#each sections.filter(s => s.title === 'Acceptance Criteria') as section}
      <section class="spec-detail__section">
        <h2 class="spec-detail__section-title">
          <Icon name="check-circle" size={16} />
          Acceptance Criteria
        </h2>
        <div class="spec-detail__section-content">
          <AcceptanceCriteria
            content={section.content}
            specId={spec.id}
            {readOnly}
          />
        </div>
      </section>
    {/each}

    <!-- Implementation Details -->
    {#each sections.filter(s => s.title === 'Implementation Details') as section}
      <section class="spec-detail__section">
        <button
          class="spec-detail__section-header"
          on:click={() => toggleSection('implementation')}
          aria-expanded={!collapsedSections.has('implementation')}
        >
          <Icon name="code" size={16} />
          <h2>Implementation Details</h2>
          <Icon
            name={collapsedSections.has('implementation') ? 'chevron-down' : 'chevron-up'}
            size={16}
          />
        </button>

        {#if !collapsedSections.has('implementation')}
          <div class="spec-detail__section-content" transition:slide={{ duration: 200 }}>
            <MarkdownPreview content={section.content} />
          </div>
        {/if}
      </section>
    {/each}

    <!-- Testing Requirements -->
    {#each sections.filter(s => s.title === 'Testing Requirements') as section}
      <section class="spec-detail__section">
        <button
          class="spec-detail__section-header"
          on:click={() => toggleSection('testing')}
          aria-expanded={!collapsedSections.has('testing')}
        >
          <Icon name="flask" size={16} />
          <h2>Testing Requirements</h2>
          <Icon
            name={collapsedSections.has('testing') ? 'chevron-down' : 'chevron-up'}
            size={16}
          />
        </button>

        {#if !collapsedSections.has('testing')}
          <div class="spec-detail__section-content" transition:slide={{ duration: 200 }}>
            <MarkdownPreview content={section.content} />
          </div>
        {/if}
      </section>
    {/each}

    <!-- Related Specs -->
    {#each sections.filter(s => s.title === 'Related Specs') as section}
      <section class="spec-detail__section">
        <h2 class="spec-detail__section-title">
          <Icon name="link" size={16} />
          Related Specs
        </h2>
        <div class="spec-detail__section-content">
          <MarkdownPreview content={section.content} />
        </div>
      </section>
    {/each}
  </div>

  <!-- Footer -->
  <footer class="spec-detail__footer">
    <div class="spec-detail__timestamps">
      <span>
        Created: {formatDate(spec.createdAt)}
        {#if spec.author}
          by {spec.author}
        {/if}
      </span>
      <span>
        Updated: {formatRelativeTime(spec.updatedAt)}
      </span>
    </div>

    {#if showHistory}
      <Button variant="ghost" size="sm" on:click={() => showHistory = !showHistory}>
        <Icon name="history" size={14} />
        View History
      </Button>
    {/if}
  </footer>

  {#if showHistory}
    <aside class="spec-detail__history-panel" transition:slide={{ axis: 'x', duration: 200 }}>
      <SpecHistory specId={spec.id} />
    </aside>
  {/if}
</article>

<style>
  .spec-detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
    outline: none;
    overflow: hidden;
  }

  .spec-detail__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 24px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-detail__breadcrumb {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }

  .spec-detail__back {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
  }

  .spec-detail__back:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .spec-detail__breadcrumb-sep {
    color: var(--color-border);
  }

  .spec-detail__actions {
    display: flex;
    gap: 4px;
  }

  .spec-detail__title-section {
    padding: 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-detail__id-badge {
    display: inline-block;
    padding: 4px 12px;
    font-size: 0.75rem;
    font-weight: 600;
    font-family: var(--font-mono);
    background: var(--color-primary-subtle);
    color: var(--color-primary);
    border-radius: 4px;
    margin-bottom: 12px;
  }

  .spec-detail__title {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 16px;
    line-height: 1.3;
  }

  .spec-detail__meta-row {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 12px;
  }

  .spec-detail__meta-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }

  .spec-detail__tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .spec-detail__tag {
    padding: 4px 10px;
    font-size: 0.75rem;
    background: var(--color-tag-bg);
    color: var(--color-tag-text);
    border-radius: 4px;
  }

  .spec-detail__body {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .spec-detail__section {
    margin-bottom: 32px;
  }

  .spec-detail__section:last-child {
    margin-bottom: 0;
  }

  .spec-detail__section-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 12px 16px;
    background: var(--color-surface-subtle);
    border: 1px solid var(--color-border);
    border-radius: 8px 8px 0 0;
    cursor: pointer;
    text-align: left;
  }

  .spec-detail__section-header h2 {
    flex: 1;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .spec-detail__section-header + .spec-detail__section-content {
    border: 1px solid var(--color-border);
    border-top: none;
    border-radius: 0 0 8px 8px;
    padding: 16px;
  }

  .spec-detail__section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 16px;
    padding-bottom: 8px;
    border-bottom: 2px solid var(--color-border);
  }

  .spec-detail__section-content {
    color: var(--color-text-primary);
    line-height: 1.6;
  }

  .spec-detail__description {
    font-size: 1.125rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.6;
  }

  .spec-detail__dep-group {
    margin-bottom: 16px;
  }

  .spec-detail__dep-group h3 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 8px;
  }

  .spec-detail__dep-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .spec-detail__dep-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .spec-detail__dep-item:hover {
    background: var(--color-hover);
    border-color: var(--color-primary-alpha);
  }

  .spec-detail__dep-id {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .spec-detail__dep-title {
    flex: 1;
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }

  .spec-detail__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 24px;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-detail__timestamps {
    display: flex;
    gap: 24px;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-detail__history-panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 400px;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
    box-shadow: var(--shadow-xl);
    z-index: 50;
  }

  /* Print styles */
  @media print {
    .spec-detail__header,
    .spec-detail__actions,
    .spec-detail__footer {
      display: none;
    }

    .spec-detail__body {
      overflow: visible;
    }

    .spec-detail__section-header {
      border: none;
      background: none;
      padding-left: 0;
    }
  }
</style>
```

### AcceptanceCriteria.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';

  export let content: string;
  export let specId: string;
  export let readOnly = false;

  const dispatch = createEventDispatcher<{
    update: { index: number; checked: boolean };
  }>();

  interface CriteriaItem {
    text: string;
    checked: boolean;
    index: number;
  }

  $: items = parseCriteria(content);

  function parseCriteria(content: string): CriteriaItem[] {
    const lines = content.split('\n');
    return lines
      .map((line, index) => {
        const match = line.match(/^-\s*\[([ xX])\]\s*(.+)$/);
        if (match) {
          return {
            text: match[2].trim(),
            checked: match[1].toLowerCase() === 'x',
            index
          };
        }
        return null;
      })
      .filter((item): item is CriteriaItem => item !== null);
  }

  function toggleItem(item: CriteriaItem) {
    if (readOnly) return;
    dispatch('update', { index: item.index, checked: !item.checked });
  }

  $: completedCount = items.filter(i => i.checked).length;
  $: progressPercent = items.length > 0 ? (completedCount / items.length) * 100 : 0;
</script>

<div class="acceptance-criteria">
  <div class="acceptance-criteria__progress">
    <div class="acceptance-criteria__progress-bar">
      <div
        class="acceptance-criteria__progress-fill"
        style:width="{progressPercent}%"
      />
    </div>
    <span class="acceptance-criteria__progress-text">
      {completedCount} / {items.length} complete
    </span>
  </div>

  <ul class="acceptance-criteria__list">
    {#each items as item}
      <li class="acceptance-criteria__item" class:acceptance-criteria__item--checked={item.checked}>
        <button
          class="acceptance-criteria__checkbox"
          on:click={() => toggleItem(item)}
          disabled={readOnly}
          aria-checked={item.checked}
          role="checkbox"
        >
          {#if item.checked}
            <Icon name="check-square" size={18} />
          {:else}
            <Icon name="square" size={18} />
          {/if}
        </button>
        <span class="acceptance-criteria__text">{item.text}</span>
      </li>
    {/each}
  </ul>
</div>

<style>
  .acceptance-criteria__progress {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .acceptance-criteria__progress-bar {
    flex: 1;
    height: 6px;
    background: var(--color-border);
    border-radius: 3px;
    overflow: hidden;
  }

  .acceptance-criteria__progress-fill {
    height: 100%;
    background: var(--color-success);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .acceptance-criteria__progress-text {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }

  .acceptance-criteria__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .acceptance-criteria__item {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 8px 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .acceptance-criteria__item:last-child {
    border-bottom: none;
  }

  .acceptance-criteria__item--checked .acceptance-criteria__text {
    color: var(--color-text-tertiary);
    text-decoration: line-through;
  }

  .acceptance-criteria__checkbox {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .acceptance-criteria__checkbox:not(:disabled):hover {
    color: var(--color-primary);
  }

  .acceptance-criteria__checkbox:disabled {
    cursor: default;
  }

  .acceptance-criteria__item--checked .acceptance-criteria__checkbox {
    color: var(--color-success);
  }

  .acceptance-criteria__text {
    font-size: 0.875rem;
    color: var(--color-text-primary);
    line-height: 1.5;
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import SpecDetailView from './SpecDetailView.svelte';
import { createMockSpec, createMockSpecs } from '$lib/test-utils/mock-data';
import { specStore } from '$lib/stores/spec-store';

describe('SpecDetailView', () => {
  const mockSpec = createMockSpec({
    id: '236',
    title: 'Spec Detail View',
    status: 'in-progress',
    phase: 11,
    dependencies: ['231', '232'],
    content: `
## Objective
Create detail view component.

## Acceptance Criteria
- [ ] Display full content
- [x] Show metadata
- [ ] Visualize dependencies
`
  });

  beforeEach(() => {
    specStore.set(createMockSpecs(10));
  });

  it('renders spec title and ID', () => {
    render(SpecDetailView, { props: { spec: mockSpec } });

    expect(screen.getByText('Spec Detail View')).toBeInTheDocument();
    expect(screen.getByText('Spec 236')).toBeInTheDocument();
  });

  it('displays status badge', () => {
    render(SpecDetailView, { props: { spec: mockSpec } });

    expect(screen.getByText('in-progress')).toBeInTheDocument();
  });

  it('shows dependencies section', () => {
    render(SpecDetailView, { props: { spec: mockSpec } });

    expect(screen.getByText('Dependencies')).toBeInTheDocument();
    expect(screen.getByText('Depends on (2)')).toBeInTheDocument();
  });

  it('handles close action', async () => {
    const { component } = render(SpecDetailView, { props: { spec: mockSpec } });

    const closeHandler = vi.fn();
    component.$on('close', closeHandler);

    await fireEvent.click(screen.getByLabelText('Close (Esc)'));

    expect(closeHandler).toHaveBeenCalled();
  });

  it('handles edit action', async () => {
    const { component } = render(SpecDetailView, { props: { spec: mockSpec } });

    const editHandler = vi.fn();
    component.$on('edit', editHandler);

    await fireEvent.click(screen.getByLabelText('Edit (Cmd+E)'));

    expect(editHandler).toHaveBeenCalledWith(
      expect.objectContaining({ detail: mockSpec })
    );
  });

  it('supports keyboard navigation', async () => {
    const { component } = render(SpecDetailView, { props: { spec: mockSpec } });

    const closeHandler = vi.fn();
    component.$on('close', closeHandler);

    const container = screen.getByLabelText(/Spec 236 details/);
    await fireEvent.keyDown(container, { key: 'Escape' });

    expect(closeHandler).toHaveBeenCalled();
  });

  it('toggles collapsible sections', async () => {
    render(SpecDetailView, { props: { spec: mockSpec } });

    const depsHeader = screen.getByText('Dependencies').closest('button');
    await fireEvent.click(depsHeader!);

    // Content should be hidden after toggle
    expect(screen.queryByText('Depends on (2)')).not.toBeInTheDocument();
  });

  it('navigates to dependent spec on click', async () => {
    const { component } = render(SpecDetailView, { props: { spec: mockSpec } });

    const navigateHandler = vi.fn();
    component.$on('navigate', navigateHandler);

    const depItems = screen.getAllByRole('button').filter(
      btn => btn.textContent?.includes('231')
    );

    if (depItems[0]) {
      await fireEvent.click(depItems[0]);
      expect(navigateHandler).toHaveBeenCalledWith(
        expect.objectContaining({ detail: { specId: '231' } })
      );
    }
  });
});

describe('AcceptanceCriteria', () => {
  it('parses and displays criteria', () => {
    const content = `
- [ ] First item
- [x] Second item
- [ ] Third item
`;
    render(AcceptanceCriteria, { props: { content, specId: '236' } });

    expect(screen.getByText('First item')).toBeInTheDocument();
    expect(screen.getByText('Second item')).toBeInTheDocument();
    expect(screen.getByText('1 / 3 complete')).toBeInTheDocument();
  });

  it('toggles criteria when not read-only', async () => {
    const content = '- [ ] Test item';
    const { component } = render(AcceptanceCriteria, {
      props: { content, specId: '236', readOnly: false }
    });

    const updateHandler = vi.fn();
    component.$on('update', updateHandler);

    const checkbox = screen.getByRole('checkbox');
    await fireEvent.click(checkbox);

    expect(updateHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { index: 0, checked: true }
      })
    );
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 237: Spec Editor
- Spec 238: Markdown Preview
- Spec 243: Dependency Visualization
- Spec 244: Version History
