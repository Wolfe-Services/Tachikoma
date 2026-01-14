<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import Icon from '$lib/components/common/Icon.svelte';

  export let title: string;
  export let icon: string = 'info';
  export let variant: 'default' | 'success' | 'warning' | 'danger' | 'info' = 'default';
  export let loading: boolean = false;
  export let href: string | null = null;

  const variantColors = {
    default: { bg: 'var(--color-bg-elevated)', border: 'var(--color-border)', icon: 'var(--color-text-secondary)' },
    success: { bg: 'var(--color-success-subtle)', border: 'var(--color-success)', icon: 'var(--color-success)' },
    warning: { bg: 'var(--color-warning-subtle)', border: 'var(--color-warning)', icon: 'var(--color-warning)' },
    danger: { bg: 'var(--color-error-subtle)', border: 'var(--color-error)', icon: 'var(--color-error)' },
    info: { bg: 'var(--color-info-subtle)', border: 'var(--color-info)', icon: 'var(--color-info)' }
  };

  $: colors = variantColors[variant];
</script>

<svelte:element
  this={href ? 'a' : 'div'}
  class="summary-card"
  class:clickable={href}
  class:loading
  {href}
  style="--card-bg: {colors.bg}; --card-border: {colors.border}; --icon-color: {colors.icon}"
  transition:fade={{ duration: 150 }}
>
  <div class="card-header">
    <div class="header-icon">
      <Icon name={icon} size={18} />
    </div>
    <h3 class="header-title">{title}</h3>
    {#if href}
      <Icon name="arrow-right" size={16} class="arrow-icon" />
    {/if}
  </div>

  <div class="card-content">
    {#if loading}
      <div class="loading-skeleton">
        <div class="skeleton-line" />
        <div class="skeleton-line short" />
      </div>
    {:else}
      <slot />
    {/if}
  </div>

  {#if $$slots.footer}
    <div class="card-footer">
      <slot name="footer" />
    </div>
  {/if}
</svelte:element>

<style>
  .summary-card {
    display: flex;
    flex-direction: column;
    background: var(--card-bg);
    border: 1px solid var(--card-border);
    border-radius: var(--card-radius);
    overflow: hidden;
    text-decoration: none;
    transition: all var(--duration-200) var(--ease-out);
  }

  .summary-card.clickable:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }

  .summary-card.loading {
    pointer-events: none;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--card-border);
  }

  .header-icon {
    color: var(--icon-color);
  }

  .header-title {
    flex: 1;
    margin: 0;
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  :global(.arrow-icon) {
    color: var(--color-text-muted);
    transition: transform var(--duration-200) var(--ease-out);
  }

  .summary-card:hover :global(.arrow-icon) {
    transform: translateX(2px);
  }

  .card-content {
    flex: 1;
    padding: var(--space-4) var(--space-5);
  }

  .card-footer {
    padding: var(--space-3) var(--space-5);
    border-top: 1px solid var(--card-border);
    background: var(--color-bg-surface);
  }

  .loading-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .skeleton-line {
    height: 1rem;
    background: var(--color-bg-surface);
    border-radius: var(--radius-sm);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .skeleton-line.short {
    width: 60%;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>