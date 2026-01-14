<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from './Icon.svelte';

  export let value = '';
  export let placeholder = 'Search...';

  const dispatch = createEventDispatcher<{
    change: string;
  }>();

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    dispatch('change', value);
  }
</script>

<div class="search-input">
  <Icon name="search" size={16} />
  <input
    type="text"
    {placeholder}
    {value}
    on:input={handleInput}
  />
</div>

<style>
  .search-input {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    color: var(--text-tertiary);
  }

  .search-input input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 0.8125rem;
    color: var(--text-primary);
    outline: none;
  }

  .search-input input::placeholder {
    color: var(--text-tertiary);
  }

  .search-input:focus-within {
    border-color: var(--accent-color);
  }
</style>