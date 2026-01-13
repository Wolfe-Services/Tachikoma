<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { scale } from 'svelte/transition';

  export let checked = false;
  export let partial = false;
  export let disabled = false;
  export let label: string;
  export let id: string;

  const dispatch = createEventDispatcher<{
    change: { id: string; checked: boolean };
  }>();

  let isUpdating = false;
  let pendingState = checked;

  async function toggle() {
    if (disabled || isUpdating) return;

    const newState = !checked;
    pendingState = newState;
    isUpdating = true;

    // Optimistic update
    checked = newState;

    dispatch('change', { id, checked: newState });

    // Wait for confirmation (or rollback on error)
    setTimeout(() => {
      isUpdating = false;
    }, 300);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === ' ' || event.key === 'Enter') {
      event.preventDefault();
      toggle();
    }
  }
</script>

<label
  class="impl-checkbox"
  class:impl-checkbox--checked={checked}
  class:impl-checkbox--partial={partial}
  class:impl-checkbox--disabled={disabled}
  class:impl-checkbox--updating={isUpdating}
>
  <span
    class="impl-checkbox__box"
    role="checkbox"
    aria-checked={partial ? 'mixed' : checked}
    aria-disabled={disabled}
    tabindex={disabled ? -1 : 0}
    on:click={toggle}
    on:keydown={handleKeyDown}
  >
    {#if checked}
      <svg
        class="impl-checkbox__icon"
        viewBox="0 0 12 12"
        fill="none"
        transition:scale={{ duration: 150 }}
      >
        <path
          d="M2 6L5 9L10 3"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    {:else if partial}
      <svg
        class="impl-checkbox__icon impl-checkbox__icon--partial"
        viewBox="0 0 12 12"
        fill="currentColor"
        transition:scale={{ duration: 150 }}
      >
        <rect x="2" y="5" width="8" height="2" rx="1" />
      </svg>
    {/if}
  </span>

  <span class="impl-checkbox__label">
    {label}
  </span>
</label>

<style>
  .impl-checkbox {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    cursor: pointer;
    user-select: none;
  }

  .impl-checkbox--disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .impl-checkbox--updating {
    pointer-events: none;
  }

  .impl-checkbox__box {
    width: 18px;
    height: 18px;
    border: 2px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-surface);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
    transition: all 0.15s ease;
  }

  .impl-checkbox__box:focus {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .impl-checkbox--checked .impl-checkbox__box {
    background: var(--color-success);
    border-color: var(--color-success);
  }

  .impl-checkbox--partial .impl-checkbox__box {
    background: var(--color-warning);
    border-color: var(--color-warning);
  }

  .impl-checkbox__icon {
    width: 12px;
    height: 12px;
    color: white;
  }

  .impl-checkbox__label {
    font-size: 14px;
    color: var(--color-text-primary);
    line-height: 1.4;
  }

  .impl-checkbox--checked .impl-checkbox__label {
    text-decoration: line-through;
    color: var(--color-text-muted);
  }

  .impl-checkbox--updating .impl-checkbox__box {
    animation: pulse 0.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(0.95); }
  }
</style>