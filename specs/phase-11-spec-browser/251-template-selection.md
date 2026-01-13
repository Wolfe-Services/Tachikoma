# 251 - Template Selection

**Phase:** 11 - Spec Browser UI
**Spec ID:** 251
**Status:** Planned
**Dependencies:** 236-spec-browser-layout
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Create a template selection component that displays available spec templates with previews, categories, and customization options for creating new specifications.

---

## Acceptance Criteria

- [x] Display template grid/list
- [x] Template preview on hover/select
- [x] Category filtering
- [x] Search templates
- [x] Template metadata display
- [x] Custom template creation
- [x] Recently used templates

---

## Implementation Details

### 1. Types (src/lib/types/spec-templates.ts)

```typescript
export interface SpecTemplate {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  icon: string;
  tags: string[];
  sections: TemplateSection[];
  frontmatterDefaults: Partial<SpecFrontmatter>;
  contentTemplate: string;
  isBuiltin: boolean;
  isCustom: boolean;
  createdAt?: string;
  usageCount: number;
}

export type TemplateCategory =
  | 'component'
  | 'feature'
  | 'api'
  | 'integration'
  | 'test'
  | 'documentation'
  | 'infrastructure'
  | 'custom';

export interface TemplateSection {
  id: string;
  title: string;
  content: string;
  isOptional: boolean;
}

export interface TemplatePreview {
  template: SpecTemplate;
  renderedContent: string;
}
```

### 2. Template Selector Component (src/lib/components/spec-browser/TemplateSelector.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { SpecTemplate, TemplateCategory } from '$lib/types/spec-templates';
  import { fade, scale } from 'svelte/transition';
  import { ipcRenderer } from '$lib/ipc';

  export let templates: SpecTemplate[] = [];
  export let selectedId: string = '';

  const dispatch = createEventDispatcher<{
    select: string;
    preview: SpecTemplate;
    createCustom: void;
  }>();

  let searchQuery = '';
  let selectedCategory: TemplateCategory | 'all' = 'all';
  let viewMode: 'grid' | 'list' = 'grid';
  let recentTemplates: SpecTemplate[] = [];
  let previewTemplate: SpecTemplate | null = null;

  const categories: { id: TemplateCategory | 'all'; label: string; icon: string }[] = [
    { id: 'all', label: 'All Templates', icon: 'grid' },
    { id: 'component', label: 'Components', icon: 'cube' },
    { id: 'feature', label: 'Features', icon: 'star' },
    { id: 'api', label: 'API', icon: 'code' },
    { id: 'integration', label: 'Integration', icon: 'link' },
    { id: 'test', label: 'Testing', icon: 'check' },
    { id: 'documentation', label: 'Documentation', icon: 'document' },
    { id: 'infrastructure', label: 'Infrastructure', icon: 'server' },
    { id: 'custom', label: 'Custom', icon: 'pencil' },
  ];

  const categoryIcons: Record<TemplateCategory, string> = {
    component: '🧩',
    feature: '⭐',
    api: '🔌',
    integration: '🔗',
    test: '✅',
    documentation: '📄',
    infrastructure: '🏗️',
    custom: '✏️',
  };

  function filterTemplates(templates: SpecTemplate[]): SpecTemplate[] {
    let filtered = templates;

    // Filter by category
    if (selectedCategory !== 'all') {
      filtered = filtered.filter(t => t.category === selectedCategory);
    }

    // Filter by search
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(t =>
        t.name.toLowerCase().includes(query) ||
        t.description.toLowerCase().includes(query) ||
        t.tags.some(tag => tag.toLowerCase().includes(query))
      );
    }

    return filtered;
  }

  function selectTemplate(template: SpecTemplate) {
    dispatch('select', template.id);
    saveToRecent(template);
  }

  function showPreview(template: SpecTemplate) {
    previewTemplate = template;
    dispatch('preview', template);
  }

  function hidePreview() {
    previewTemplate = null;
  }

  function saveToRecent(template: SpecTemplate) {
    const recent = recentTemplates.filter(t => t.id !== template.id);
    recentTemplates = [template, ...recent].slice(0, 5);
    localStorage.setItem('recent-templates', JSON.stringify(recentTemplates.map(t => t.id)));
  }

  function loadRecentTemplates() {
    const savedIds = localStorage.getItem('recent-templates');
    if (savedIds) {
      const ids: string[] = JSON.parse(savedIds);
      recentTemplates = ids
        .map(id => templates.find(t => t.id === id))
        .filter((t): t is SpecTemplate => t !== undefined);
    }
  }

  onMount(() => {
    loadRecentTemplates();
  });

  $: filteredTemplates = filterTemplates(templates);
  $: if (templates.length > 0) loadRecentTemplates();
</script>

<div class="template-selector">
  <header class="template-selector__header">
    <div class="search-container">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M11.742 10.344a6.5 6.5 0 10-1.397 1.398h-.001l3.85 3.85a1 1 0 001.415-1.414l-3.867-3.834zm-5.242.656a5 5 0 110-10 5 5 0 010 10z"/>
      </svg>
      <input
        type="search"
        placeholder="Search templates..."
        bind:value={searchQuery}
      />
    </div>

    <div class="view-toggle">
      <button
        class:active={viewMode === 'grid'}
        on:click={() => { viewMode = 'grid'; }}
        title="Grid view"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 1h6v6H1V1zm8 0h6v6H9V1zM1 9h6v6H1V9zm8 0h6v6H9V9z"/>
        </svg>
      </button>
      <button
        class:active={viewMode === 'list'}
        on:click={() => { viewMode = 'list'; }}
        title="List view"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M2 4h12v2H2V4zm0 4h12v2H2V8zm0 4h12v2H2v-2z"/>
        </svg>
      </button>
    </div>
  </header>

  <nav class="category-nav">
    {#each categories as category}
      <button
        class="category-btn"
        class:active={selectedCategory === category.id}
        on:click={() => { selectedCategory = category.id; }}
      >
        {category.label}
        {#if category.id !== 'all'}
          <span class="category-count">
            {templates.filter(t => t.category === category.id).length}
          </span>
        {/if}
      </button>
    {/each}
  </nav>

  {#if recentTemplates.length > 0 && selectedCategory === 'all' && !searchQuery}
    <section class="recent-section">
      <h4>Recently Used</h4>
      <div class="recent-templates">
        {#each recentTemplates as template}
          <button
            class="recent-template"
            class:selected={selectedId === template.id}
            on:click={() => selectTemplate(template)}
          >
            <span class="template-icon">{categoryIcons[template.category]}</span>
            <span class="template-name">{template.name}</span>
          </button>
        {/each}
      </div>
    </section>
  {/if}

  <div class="template-list" class:grid-view={viewMode === 'grid'}>
    {#if filteredTemplates.length === 0}
      <div class="empty-state">
        <p>No templates found</p>
        {#if searchQuery}
          <button class="clear-search" on:click={() => { searchQuery = ''; }}>
            Clear search
          </button>
        {/if}
      </div>
    {:else}
      {#each filteredTemplates as template (template.id)}
        <button
          class="template-card"
          class:selected={selectedId === template.id}
          on:click={() => selectTemplate(template)}
          on:mouseenter={() => showPreview(template)}
          on:mouseleave={hidePreview}
          transition:fade={{ duration: 150 }}
        >
          <div class="template-card__icon">
            {categoryIcons[template.category]}
          </div>

          <div class="template-card__content">
            <h3 class="template-name">{template.name}</h3>
            <p class="template-description">{template.description}</p>

            <div class="template-meta">
              <span class="template-category">{template.category}</span>
              {#if template.isCustom}
                <span class="custom-badge">Custom</span>
              {/if}
            </div>

            {#if template.tags.length > 0 && viewMode === 'list'}
              <div class="template-tags">
                {#each template.tags.slice(0, 3) as tag}
                  <span class="tag">{tag}</span>
                {/each}
                {#if template.tags.length > 3}
                  <span class="tag more">+{template.tags.length - 3}</span>
                {/if}
              </div>
            {/if}
          </div>

          {#if selectedId === template.id}
            <div class="selected-indicator">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 0a8 8 0 100 16A8 8 0 008 0zm3.78 5.28l-4.5 6a.75.75 0 01-1.14.06l-2.25-2.25a.75.75 0 111.06-1.06l1.64 1.64 3.97-5.3a.75.75 0 011.22.91z"/>
              </svg>
            </div>
          {/if}
        </button>
      {/each}

      <button
        class="template-card template-card--create"
        on:click={() => dispatch('createCustom')}
      >
        <div class="template-card__icon">➕</div>
        <div class="template-card__content">
          <h3 class="template-name">Create Custom Template</h3>
          <p class="template-description">Design your own template structure</p>
        </div>
      </button>
    {/if}
  </div>

  {#if previewTemplate}
    <aside
      class="template-preview"
      transition:scale={{ duration: 150, start: 0.95 }}
    >
      <header class="preview-header">
        <span class="preview-icon">{categoryIcons[previewTemplate.category]}</span>
        <h4>{previewTemplate.name}</h4>
      </header>

      <div class="preview-content">
        <p class="preview-description">{previewTemplate.description}</p>

        <div class="preview-sections">
          <h5>Sections</h5>
          <ul>
            {#each previewTemplate.sections as section}
              <li class:optional={section.isOptional}>
                {section.title}
                {#if section.isOptional}
                  <span class="optional-tag">Optional</span>
                {/if}
              </li>
            {/each}
          </ul>
        </div>

        {#if previewTemplate.tags.length > 0}
          <div class="preview-tags">
            {#each previewTemplate.tags as tag}
              <span class="tag">{tag}</span>
            {/each}
          </div>
        {/if}

        <div class="preview-stats">
          <span>Used {previewTemplate.usageCount} times</span>
        </div>
      </div>
    </aside>
  {/if}
</div>

<style>
  .template-selector {
    display: flex;
    flex-direction: column;
    gap: 16px;
    position: relative;
  }

  .template-selector__header {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .search-container {
    flex: 1;
    position: relative;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--color-text-muted);
  }

  .search-container input {
    width: 100%;
    padding: 10px 14px 10px 38px;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    font-size: 14px;
  }

  .search-container input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .view-toggle {
    display: flex;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    overflow: hidden;
  }

  .view-toggle button {
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .view-toggle button:hover {
    background: var(--color-bg-hover);
  }

  .view-toggle button.active {
    background: var(--color-primary);
    color: white;
  }

  .category-nav {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 4px;
  }

  .category-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 20px;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
  }

  .category-btn:hover {
    background: var(--color-bg-hover);
  }

  .category-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .category-count {
    font-size: 11px;
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 10px;
  }

  .recent-section {
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }

  .recent-section h4 {
    margin: 0 0 12px 0;
    font-size: 12px;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .recent-templates {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .recent-template {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }

  .recent-template:hover {
    border-color: var(--color-primary);
  }

  .recent-template.selected {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.1);
  }

  .template-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .template-list.grid-view {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  }

  .empty-state {
    text-align: center;
    padding: 48px 24px;
    color: var(--color-text-muted);
  }

  .clear-search {
    margin-top: 12px;
    padding: 8px 16px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    color: var(--color-text-secondary);
  }

  .template-card {
    display: flex;
    gap: 12px;
    padding: 16px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    position: relative;
    transition: all 0.15s ease;
  }

  .template-card:hover {
    border-color: var(--color-primary);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .template-card.selected {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.05);
  }

  .template-card--create {
    border-style: dashed;
    background: transparent;
  }

  .template-card--create:hover {
    background: var(--color-bg-secondary);
  }

  .grid-view .template-card {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .template-card__icon {
    font-size: 24px;
    flex-shrink: 0;
  }

  .grid-view .template-card__icon {
    font-size: 32px;
  }

  .template-card__content {
    flex: 1;
    min-width: 0;
  }

  .template-card__content h3 {
    margin: 0 0 4px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .template-description {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .template-meta {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .template-category {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .custom-badge {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
  }

  .template-tags {
    display: flex;
    gap: 6px;
    margin-top: 8px;
    flex-wrap: wrap;
  }

  .tag {
    font-size: 11px;
    padding: 2px 8px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    color: var(--color-text-secondary);
  }

  .tag.more {
    background: var(--color-bg-hover);
    color: var(--color-text-muted);
  }

  .selected-indicator {
    position: absolute;
    top: 12px;
    right: 12px;
    color: var(--color-primary);
  }

  /* Preview Panel */
  .template-preview {
    position: absolute;
    top: 0;
    right: -320px;
    width: 300px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
    z-index: 10;
  }

  .preview-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .preview-icon {
    font-size: 24px;
  }

  .preview-header h4 {
    margin: 0;
    font-size: 16px;
  }

  .preview-content {
    padding: 16px;
  }

  .preview-description {
    margin: 0 0 16px 0;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .preview-sections h5 {
    margin: 0 0 8px 0;
    font-size: 12px;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .preview-sections ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .preview-sections li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    font-size: 13px;
    border-bottom: 1px solid var(--color-border);
  }

  .preview-sections li.optional {
    color: var(--color-text-muted);
  }

  .optional-tag {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--color-bg-hover);
    border-radius: 4px;
  }

  .preview-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 16px 0;
  }

  .preview-stats {
    font-size: 12px;
    color: var(--color-text-muted);
  }
</style>
```

---

## Testing Requirements

1. Templates display correctly
2. Category filtering works
3. Search filters templates
4. Selection highlights card
5. Preview shows on hover
6. Recent templates persist

---

## Related Specs

- Depends on: [236-spec-browser-layout.md](236-spec-browser-layout.md)
- Used by: [250-spec-creation.md](250-spec-creation.md)
- Next: [252-spec-validation-ui.md](252-spec-validation-ui.md)
