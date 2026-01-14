<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';

  export let label: string;
  export let values: string[];

  const dispatch = createEventDispatcher<{ remove: void }>();

  $: displayValue = values.length > 2
    ? `${values.slice(0, 2).join(', ')} +${values.length - 2}`
    : values.join(', ');
</script>

<div class="filter-chip">
  <span class="chip-label">{label}:</span>
  <span class="chip-value">{displayValue}</span>
  <button
    class="chip-remove"
    on:click={() => dispatch('remove')}
    aria-label="Remove filter"
  >
    <Icon name="x" size={12} />
  </button>
</div>

<style>
  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.375rem 0.25rem 0.625rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 9999px;
    font-size: 0.75rem;
  }

  .chip-label {
    color: var(--text-tertiary);
  }

  .chip-value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .chip-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.125rem;
    height: 1.125rem;
    padding: 0;
    border: none;
    background: var(--bg-hover);
    border-radius: 50%;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .chip-remove:hover {
    background: var(--text-tertiary);
    color: white;
  }
</style>