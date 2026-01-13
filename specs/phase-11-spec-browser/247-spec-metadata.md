# 247 - Spec Metadata Panel

**Phase:** 11 - Spec Browser UI
**Spec ID:** 247
**Status:** Planned
**Dependencies:** 236-spec-browser-layout
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a metadata panel component that displays spec properties, dependencies, related specs, and editing controls for spec metadata.

---

## Acceptance Criteria

- [x] Display all spec frontmatter fields
- [x] Show dependency graph/links
- [x] Related specs list
- [x] Edit metadata inline
- [x] Tag management
- [x] Last modified info

---

## Implementation Details

### 1. Spec Metadata Panel Component (src/lib/components/spec-browser/SpecMetadata.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SpecFile, SpecFrontmatter } from '$lib/types/spec-viewer';
  import { ipcRenderer } from '$lib/ipc';

  export let specId: string | null;

  const dispatch = createEventDispatcher<{
    navigate: string;
    update: Partial<SpecFrontmatter>;
  }>();

  let spec: SpecFile | null = null;
  let dependencies: { id: string; title: string }[] = [];
  let dependents: { id: string; title: string }[] = [];
  let isEditing = false;

  async function loadSpec() {
    if (!specId) {
      spec = null;
      return;
    }

    spec = await ipcRenderer.invoke('spec:get', specId);
    if (spec) {
      await loadRelated();
    }
  }

  async function loadRelated() {
    if (!spec) return;

    dependencies = await ipcRenderer.invoke('spec:get-dependencies', specId);
    dependents = await ipcRenderer.invoke('spec:get-dependents', specId);
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  $: if (specId) loadSpec();
</script>

<div class="spec-metadata">
  {#if !specId}
    <div class="spec-metadata__empty">
      Select a spec to view metadata
    </div>
  {:else if !spec}
    <div class="spec-metadata__loading">Loading...</div>
  {:else}
    <header class="spec-metadata__header">
      <h3>Metadata</h3>
      <button
        class="edit-btn"
        on:click={() => { isEditing = !isEditing; }}
      >
        {isEditing ? 'Done' : 'Edit'}
      </button>
    </header>

    <section class="metadata-section">
      <h4>Properties</h4>

      <div class="property">
        <span class="property-label">Spec ID</span>
        <span class="property-value">{spec.frontmatter.specId}</span>
      </div>

      <div class="property">
        <span class="property-label">Phase</span>
        <span class="property-value">{spec.frontmatter.phase}</span>
      </div>

      <div class="property">
        <span class="property-label">Status</span>
        <span class="property-value status-badge status-badge--{spec.frontmatter.status}">
          {spec.frontmatter.status}
        </span>
      </div>

      <div class="property">
        <span class="property-label">Est. Context</span>
        <span class="property-value">{spec.frontmatter.estimatedContext}</span>
      </div>
    </section>

    <section class="metadata-section">
      <h4>Dependencies ({dependencies.length})</h4>

      {#if dependencies.length === 0}
        <p class="empty-list">No dependencies</p>
      {:else}
        <ul class="related-list">
          {#each dependencies as dep}
            <li>
              <button
                class="related-link"
                on:click={() => dispatch('navigate', dep.id)}
              >
                <span class="related-id">{dep.id}</span>
                <span class="related-title">{dep.title}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="metadata-section">
      <h4>Used By ({dependents.length})</h4>

      {#if dependents.length === 0}
        <p class="empty-list">No dependents</p>
      {:else}
        <ul class="related-list">
          {#each dependents as dep}
            <li>
              <button
                class="related-link"
                on:click={() => dispatch('navigate', dep.id)}
              >
                <span class="related-id">{dep.id}</span>
                <span class="related-title">{dep.title}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="metadata-section">
      <h4>File Info</h4>

      <div class="property">
        <span class="property-label">Path</span>
        <span class="property-value path">{spec.path}</span>
      </div>

      <div class="property">
        <span class="property-label">Modified</span>
        <span class="property-value">{formatDate(spec.lastModified)}</span>
      </div>
    </section>
  {/if}
</div>

<style>
  .spec-metadata {
    height: 100%;
    overflow-y: auto;
    padding: 16px;
  }

  .spec-metadata__empty,
  .spec-metadata__loading {
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .spec-metadata__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .spec-metadata__header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .edit-btn {
    padding: 4px 10px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .metadata-section {
    margin-bottom: 24px;
  }

  .metadata-section h4 {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--color-text-muted);
    margin: 0 0 12px 0;
  }

  .property {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .property-label {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .property-value {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .property-value.path {
    font-family: monospace;
    font-size: 11px;
    word-break: break-all;
  }

  .status-badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    text-transform: uppercase;
  }

  .status-badge--planned {
    background: var(--color-bg-hover);
    color: var(--color-text-muted);
  }

  .status-badge--in_progress {
    background: rgba(33, 150, 243, 0.1);
    color: var(--color-primary);
  }

  .status-badge--complete {
    background: rgba(76, 175, 80, 0.1);
    color: var(--color-success);
  }

  .empty-list {
    font-size: 13px;
    color: var(--color-text-muted);
    font-style: italic;
    margin: 0;
  }

  .related-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .related-link {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
  }

  .related-link:hover {
    background: var(--color-bg-hover);
  }

  .related-id {
    font-family: monospace;
    font-size: 11px;
    color: var(--color-primary);
  }

  .related-title {
    font-size: 13px;
    color: var(--color-text-primary);
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
```

---

## Testing Requirements

1. Metadata loads correctly
2. Dependencies display
3. Dependents display
4. Navigation works
5. Edit mode toggles

---

## Related Specs

- Depends on: [236-spec-browser-layout.md](236-spec-browser-layout.md)
- Next: [248-spec-version-history.md](248-spec-version-history.md)
