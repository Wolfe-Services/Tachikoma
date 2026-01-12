<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import type { RedlineWarning, Recommendation, RedlineAction } from '$lib/types/redline';

  export let warning: RedlineWarning;
  export let show = true;

  const dispatch = createEventDispatcher<{
    action: RedlineAction;
    dismiss: void;
    snooze: { minutes: number };
  }>();

  const levelConfig = {
    yellow: {
      color: 'var(--color-warning-fg)',
      bg: 'var(--color-warning-subtle)',
      icon: '⚠️',
      title: 'Context Warning',
    },
    orange: {
      color: '#ff6b35',
      bg: 'rgba(255, 107, 53, 0.15)',
      icon: '🔶',
      title: 'High Context Usage',
    },
    red: {
      color: 'var(--color-error-fg)',
      bg: 'var(--color-error-subtle)',
      icon: '🔴',
      title: 'Context Redline',
    },
  };

  $: config = levelConfig[warning.level];
</script>

{#if show}
  <div
    class="redline-warning"
    class:redline-warning--red={warning.level === 'red'}
    style="background: {config.bg}; border-color: {config.color}"
    role="alert"
    aria-live={warning.level === 'red' ? 'assertive' : 'polite'}
    transition:fly={{ y: -20, duration: 300 }}
  >
    <div class="redline-warning__header">
      <span class="redline-warning__icon">{config.icon}</span>
      <span class="redline-warning__title" style="color: {config.color}">
        {config.title}
      </span>
      <span class="redline-warning__percent">{warning.contextPercent}%</span>

      {#if warning.canDismiss}
        <button
          class="redline-warning__dismiss"
          on:click={() => dispatch('dismiss')}
          aria-label="Dismiss warning"
        >
          ×
        </button>
      {/if}
    </div>

    <p class="redline-warning__message">{warning.message}</p>

    <div class="redline-warning__recommendations">
      {#each warning.recommendations as rec}
        <button
          class="recommendation"
          on:click={() => dispatch('action', rec.action)}
          title={rec.description}
        >
          <span class="recommendation__title">{rec.title}</span>
          <span class="recommendation__impact">{rec.impact}</span>
        </button>
      {/each}
    </div>

    {#if warning.canDismiss}
      <div class="redline-warning__snooze">
        <span>Snooze for:</span>
        <button on:click={() => dispatch('snooze', { minutes: 5 })}>5m</button>
        <button on:click={() => dispatch('snooze', { minutes: 15 })}>15m</button>
        <button on:click={() => dispatch('snooze', { minutes: 30 })}>30m</button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .redline-warning {
    padding: 16px;
    border: 2px solid;
    border-radius: 8px;
    margin-bottom: 16px;
  }

  .redline-warning--red {
    animation: pulse-border 1s ease-in-out infinite;
  }

  @keyframes pulse-border {
    0%, 100% { border-color: var(--color-error-fg); }
    50% { border-color: transparent; }
  }

  .redline-warning__header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .redline-warning__icon {
    font-size: 18px;
  }

  .redline-warning__title {
    font-size: 14px;
    font-weight: 600;
  }

  .redline-warning__percent {
    font-size: 14px;
    font-weight: 700;
    margin-left: auto;
  }

  .redline-warning__dismiss {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--color-fg-muted);
    font-size: 18px;
    cursor: pointer;
  }

  .redline-warning__dismiss:hover {
    color: var(--color-fg-default);
  }

  .redline-warning__message {
    margin: 0 0 16px 0;
    font-size: 13px;
    color: var(--color-fg-default);
    line-height: 1.5;
  }

  .redline-warning__recommendations {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
  }

  .recommendation {
    display: flex;
    flex-direction: column;
    padding: 8px 12px;
    border: 1px solid var(--color-border-default);
    background: var(--color-bg-surface);
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .recommendation:hover {
    border-color: var(--color-accent-fg);
    background: var(--color-bg-hover);
  }

  .recommendation__title {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-fg-default);
  }

  .recommendation__impact {
    font-size: 11px;
    color: var(--color-success-fg);
  }

  .redline-warning__snooze {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--color-fg-muted);
  }

  .redline-warning__snooze button {
    padding: 4px 8px;
    border: 1px solid var(--color-border-default);
    background: transparent;
    color: var(--color-fg-muted);
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .redline-warning__snooze button:hover {
    border-color: var(--color-accent-fg);
    color: var(--color-fg-default);
  }
</style>