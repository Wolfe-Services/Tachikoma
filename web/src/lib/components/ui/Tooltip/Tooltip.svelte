<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let text = '';
  export let position: 'top' | 'bottom' | 'left' | 'right' = 'top';
  export let delay = 500;

  let showTooltip = false;
  let timeoutId: number | null = null;

  const dispatch = createEventDispatcher();

  function handleMouseEnter() {
    timeoutId = setTimeout(() => {
      showTooltip = true;
      dispatch('show');
    }, delay);
  }

  function handleMouseLeave() {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    showTooltip = false;
    dispatch('hide');
  }
</script>

<div
  class="tooltip-container"
  role="tooltip"
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  on:focus={handleMouseEnter}
  on:blur={handleMouseLeave}
>
  <slot />
  
  {#if showTooltip && (text || $$slots.content)}
    <div 
      class="tooltip tooltip--{position}"
      class:tooltip--visible={showTooltip}
    >
      {#if $$slots.content}
        <slot name="content" />
      {:else}
        {text}
      {/if}
    </div>
  {/if}
</div>

<style>
  .tooltip-container {
    position: relative;
    display: inline-block;
  }

  .tooltip {
    position: absolute;
    background: var(--color-bg-overlay);
    color: var(--color-text-primary);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    white-space: nowrap;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border);
    z-index: var(--z-tooltip);
    opacity: 0;
    transform: scale(0.8);
    transition: opacity var(--duration-200) var(--ease-out),
                transform var(--duration-200) var(--ease-out);
  }

  .tooltip--visible {
    opacity: 1;
    transform: scale(1);
  }

  .tooltip--top {
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%) scale(0.8);
    margin-bottom: var(--space-2);
  }

  .tooltip--top.tooltip--visible {
    transform: translateX(-50%) scale(1);
  }

  .tooltip--bottom {
    top: 100%;
    left: 50%;
    transform: translateX(-50%) scale(0.8);
    margin-top: var(--space-2);
  }

  .tooltip--bottom.tooltip--visible {
    transform: translateX(-50%) scale(1);
  }

  .tooltip--left {
    right: 100%;
    top: 50%;
    transform: translateY(-50%) scale(0.8);
    margin-right: var(--space-2);
  }

  .tooltip--left.tooltip--visible {
    transform: translateY(-50%) scale(1);
  }

  .tooltip--right {
    left: 100%;
    top: 50%;
    transform: translateY(-50%) scale(0.8);
    margin-left: var(--space-2);
  }

  .tooltip--right.tooltip--visible {
    transform: translateY(-50%) scale(1);
  }

  /* Arrow indicators */
  .tooltip::after {
    content: '';
    position: absolute;
    width: 0;
    height: 0;
  }

  .tooltip--top::after {
    top: 100%;
    left: 50%;
    margin-left: -4px;
    border-left: 4px solid transparent;
    border-right: 4px solid transparent;
    border-top: 4px solid var(--color-bg-overlay);
  }

  .tooltip--bottom::after {
    bottom: 100%;
    left: 50%;
    margin-left: -4px;
    border-left: 4px solid transparent;
    border-right: 4px solid transparent;
    border-bottom: 4px solid var(--color-bg-overlay);
  }

  .tooltip--left::after {
    right: -4px;
    top: 50%;
    margin-top: -4px;
    border-top: 4px solid transparent;
    border-bottom: 4px solid transparent;
    border-left: 4px solid var(--color-bg-overlay);
  }

  .tooltip--right::after {
    left: -4px;
    top: 50%;
    margin-top: -4px;
    border-top: 4px solid transparent;
    border-bottom: 4px solid transparent;
    border-right: 4px solid var(--color-bg-overlay);
  }
</style>