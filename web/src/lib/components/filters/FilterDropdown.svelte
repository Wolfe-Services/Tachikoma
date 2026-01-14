<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly } from 'svelte/transition';
  import type { FilterConfig } from '$lib/types/filters';
  import Icon from '$lib/components/common/Icon.svelte';

  export let filter: FilterConfig;
  export let selectedValues: string[] = [];
  export let inline: boolean = false;

  const dispatch = createEventDispatcher<{
    change: string[];
  }>();

  let open = false;
  let searchQuery = '';

  $: filteredOptions = filter.options.filter(opt =>
    opt.label.toLowerCase().includes(searchQuery.toLowerCase())
  );

  $: selectedCount = selectedValues.length;
  $: displayLabel = selectedCount > 0
    ? `${filter.label} (${selectedCount})`
    : filter.label;

  function toggleOption(value: string) {
    if (selectedValues.includes(value)) {
      selectedValues = selectedValues.filter(v => v !== value);
    } else {
      selectedValues = [...selectedValues, value];
    }
    dispatch('change', selectedValues);
  }

  function selectAll() {
    selectedValues = filter.options.map(o => o.value);
    dispatch('change', selectedValues);
  }

  function clearSelection() {
    selectedValues = [];
    dispatch('change', selectedValues);
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.filter-dropdown')) {
      open = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="filter-dropdown" class:inline class:open>
  <button
    class="dropdown-trigger"
    on:click|stopPropagation={() => open = !open}
    aria-expanded={open}
    aria-haspopup="listbox"
  >
    <span class="trigger-label">{displayLabel}</span>
    <Icon name={open ? 'chevron-up' : 'chevron-down'} size={14} />
  </button>

  {#if open}
    <div
      class="dropdown-menu"
      role="listbox"
      aria-multiselectable="true"
      transition:fly={{ y: -5, duration: 150 }}
    >
      {#if filter.searchable}
        <div class="menu-search">
          <Icon name="search" size={14} />
          <input
            type="text"
            placeholder="Search..."
            bind:value={searchQuery}
            on:click|stopPropagation
          />
        </div>
      {/if}

      <div class="menu-actions">
        <button on:click|stopPropagation={selectAll}>Select All</button>
        <button on:click|stopPropagation={clearSelection}>Clear</button>
      </div>

      <ul class="options-list">
        {#each filteredOptions as option (option.value)}
          <li>
            <label class="option-item">
              <input
                type="checkbox"
                checked={selectedValues.includes(option.value)}
                on:change={() => toggleOption(option.value)}
                on:click|stopPropagation
              />
              {#if option.icon}
                <Icon name={option.icon} size={14} />
              {/if}
              {#if option.color}
                <span class="option-color" style="background: {option.color}" />
              {/if}
              <span class="option-label">{option.label}</span>
              {#if option.count !== undefined}
                <span class="option-count">{option.count}</span>
              {/if}
            </label>
          </li>
        {/each}

        {#if filteredOptions.length === 0}
          <li class="no-results">No options found</li>
        {/if}
      </ul>
    </div>
  {/if}
</div>

<style>
  .filter-dropdown {
    position: relative;
  }

  .dropdown-trigger {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .dropdown-trigger:hover {
    border-color: var(--border-hover);
  }

  .filter-dropdown.open .dropdown-trigger {
    border-color: var(--accent-color);
  }

  .inline .dropdown-trigger {
    width: 100%;
    justify-content: space-between;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 0.25rem;
    min-width: 200px;
    max-height: 300px;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    z-index: 100;
    overflow: hidden;
  }

  .inline .dropdown-menu {
    position: static;
    margin-top: 0.5rem;
    box-shadow: none;
    border: 1px solid var(--border-color);
  }

  .menu-search {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-tertiary);
  }

  .menu-search input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 0.8125rem;
    color: var(--text-primary);
    outline: none;
  }

  .menu-actions {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .menu-actions button {
    padding: 0;
    border: none;
    background: transparent;
    font-size: 0.6875rem;
    color: var(--accent-color);
    cursor: pointer;
  }

  .menu-actions button:hover {
    text-decoration: underline;
  }

  .options-list {
    list-style: none;
    padding: 0.25rem 0;
    margin: 0;
    max-height: 200px;
    overflow-y: auto;
  }

  .option-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .option-item:hover {
    background: var(--bg-hover);
  }

  .option-color {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 0.125rem;
  }

  .option-label {
    flex: 1;
  }

  .option-count {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
  }

  .no-results {
    padding: 1rem;
    text-align: center;
    font-size: 0.8125rem;
    color: var(--text-tertiary);
  }
</style>