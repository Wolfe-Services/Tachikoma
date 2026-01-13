<script lang="ts">
  export let specId: string | null = null;

  // Mock breadcrumb generation - this would be replaced with actual spec hierarchy logic
  $: breadcrumbs = specId ? generateBreadcrumbs(specId) : [];

  function generateBreadcrumbs(id: string) {
    // Parse spec ID to create breadcrumb trail
    // Format: "phase-XX-name/###-spec-name"
    const parts = id.split('/');
    const crumbs: { label: string; path?: string }[] = [
      { label: 'Specs', path: '/' }
    ];

    if (parts.length >= 1) {
      const phase = parts[0];
      // Extract phase number and name from format like "phase-11-spec-browser"
      const phaseMatch = phase.match(/phase-(\d+)-(.+)/);
      if (phaseMatch) {
        const [, phaseNum, phaseName] = phaseMatch;
        crumbs.push({
          label: `Phase ${phaseNum}`,
          path: `/${phase}`
        });
        crumbs.push({
          label: phaseName.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' '),
          path: `/${phase}`
        });
      }
    }

    if (parts.length >= 2) {
      const specFile = parts[1];
      // Extract spec number and name from format like "236-spec-browser-layout"
      const specMatch = specFile.match(/(\d+)-(.+)\.md$/);
      if (specMatch) {
        const [, specNum, specName] = specMatch;
        crumbs.push({
          label: `${specNum} - ${specName.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}`
        });
      } else {
        // Fallback for specs without .md extension
        crumbs.push({ label: specFile });
      }
    }

    return crumbs;
  }
</script>

{#if breadcrumbs.length > 0}
  <nav class="breadcrumbs" aria-label="Breadcrumb navigation">
    <ol class="breadcrumbs__list">
      {#each breadcrumbs as crumb, index}
        <li class="breadcrumbs__item">
          {#if crumb.path && index < breadcrumbs.length - 1}
            <a 
              href={crumb.path}
              class="breadcrumbs__link"
              aria-current={index === breadcrumbs.length - 1 ? 'page' : undefined}
            >
              {crumb.label}
            </a>
          {:else}
            <span class="breadcrumbs__current" aria-current="page">
              {crumb.label}
            </span>
          {/if}
          {#if index < breadcrumbs.length - 1}
            <span class="breadcrumbs__separator" aria-hidden="true">/</span>
          {/if}
        </li>
      {/each}
    </ol>
  </nav>
{:else}
  <div class="breadcrumbs__empty">
    <span class="breadcrumbs__placeholder">No specification selected</span>
  </div>
{/if}

<style>
  .breadcrumbs {
    flex: 1;
    min-width: 0;
  }

  .breadcrumbs__list {
    display: flex;
    align-items: center;
    list-style: none;
    margin: 0;
    padding: 0;
    gap: 4px;
    overflow: hidden;
  }

  .breadcrumbs__item {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .breadcrumbs__link {
    color: var(--color-fg-muted);
    text-decoration: none;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-radius: 4px;
    padding: 2px 4px;
    transition: all 0.15s ease;
  }

  .breadcrumbs__link:hover {
    color: var(--color-fg-default);
    background: var(--color-bg-hover);
  }

  .breadcrumbs__link:focus-visible {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: 2px;
  }

  .breadcrumbs__current {
    color: var(--color-fg-default);
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .breadcrumbs__separator {
    color: var(--color-fg-subtle);
    font-size: 12px;
    user-select: none;
  }

  .breadcrumbs__empty {
    flex: 1;
    display: flex;
    align-items: center;
  }

  .breadcrumbs__placeholder {
    color: var(--color-fg-subtle);
    font-size: 14px;
    font-style: italic;
  }

  /* Mobile responsive */
  @media (max-width: 768px) {
    .breadcrumbs__list {
      gap: 2px;
    }

    .breadcrumbs__link,
    .breadcrumbs__current {
      font-size: 12px;
    }

    .breadcrumbs__separator {
      font-size: 10px;
    }

    /* Hide intermediate breadcrumbs on very small screens */
    .breadcrumbs__item:not(:first-child):not(:last-child) {
      display: none;
    }

    /* Show ellipsis when items are hidden */
    .breadcrumbs__list::after {
      content: "...";
      color: var(--color-fg-subtle);
      font-size: 12px;
    }
  }

  @media (max-width: 480px) {
    .breadcrumbs__link,
    .breadcrumbs__current,
    .breadcrumbs__placeholder {
      font-size: 11px;
    }
  }
</style>