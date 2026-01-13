<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { MissionHistoryEntry } from '$lib/types/history';
  import Tooltip from '$lib/components/ui/Tooltip/Tooltip.svelte';

  export let entry: MissionHistoryEntry;
  export let selected = false;

  const dispatch = createEventDispatcher();

  let isHovered = false;
  let showPreview = false;
  let previewTimeout: number | null = null;

  function handleMouseEnter() {
    isHovered = true;
    previewTimeout = window.setTimeout(() => {
      showPreview = true;
    }, 500); // 500ms delay for preview
  }

  function handleMouseLeave() {
    isHovered = false;
    showPreview = false;
    if (previewTimeout) {
      clearTimeout(previewTimeout);
      previewTimeout = null;
    }
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatDuration(seconds: number): string {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }

  function formatCost(cost: number): string {
    return `$${cost.toFixed(3)}`;
  }

  function getStatusColor(state: string): string {
    switch (state) {
      case 'complete': return 'var(--color-success)';
      case 'error': return 'var(--color-error)';
      case 'running': return 'var(--color-info)';
      default: return 'var(--color-text-muted)';
    }
  }

  function handleSelect(event: Event) {
    event.stopPropagation();
    dispatch('select');
  }

  function handleOpen() {
    dispatch('open', { entry });
  }
</script>

<div 
  class="history-card"
  class:history-card--selected={selected}
  class:history-card--hovered={isHovered}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  on:click={handleOpen}
  role="button"
  tabindex="0"
  on:keydown={(e) => e.key === 'Enter' && handleOpen()}
>
  <div class="history-card__header">
    <label class="history-card__checkbox" on:click={handleSelect}>
      <input 
        type="checkbox" 
        bind:checked={selected}
        on:click={handleSelect}
      />
      <span class="checkmark"></span>
    </label>
    
    <div class="history-card__title">
      <h3>{entry.title || 'Untitled Mission'}</h3>
      <span 
        class="history-card__status"
        style="color: {getStatusColor(entry.state)}"
      >
        {entry.state}
      </span>
    </div>
    
    <div class="history-card__date">
      {formatDate(entry.createdAt)}
    </div>
  </div>

  <div class="history-card__content">
    <p class="history-card__prompt">
      {entry.prompt.length > 100 ? entry.prompt.slice(0, 100) + '...' : entry.prompt}
    </p>
    
    {#if entry.tags.length > 0}
      <div class="history-card__tags">
        {#each entry.tags.slice(0, 3) as tag}
          <span class="tag">{tag}</span>
        {/each}
        {#if entry.tags.length > 3}
          <span class="tag tag--more">+{entry.tags.length - 3}</span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="history-card__footer">
    <div class="history-card__stats">
      <span class="stat">
        <span class="stat__label">Duration:</span>
        <span class="stat__value">{formatDuration(entry.duration)}</span>
      </span>
      <span class="stat">
        <span class="stat__label">Cost:</span>
        <span class="stat__value">{formatCost(entry.cost)}</span>
      </span>
      <span class="stat">
        <span class="stat__label">Files:</span>
        <span class="stat__value">{entry.filesChanged}</span>
      </span>
    </div>
  </div>

  {#if showPreview}
    <div class="history-card__preview">
      <div class="preview-content">
        <h4>Mission Preview</h4>
        <div class="preview-details">
          <p><strong>Prompt:</strong> {entry.prompt}</p>
          <p><strong>Duration:</strong> {formatDuration(entry.duration)}</p>
          <p><strong>Cost:</strong> {formatCost(entry.cost)}</p>
          <p><strong>Tokens Used:</strong> {entry.tokensUsed.toLocaleString()}</p>
          <p><strong>Files Changed:</strong> {entry.filesChanged}</p>
          {#if entry.tags.length > 0}
            <p><strong>Tags:</strong> {entry.tags.join(', ')}</p>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .history-card {
    position: relative;
    display: flex;
    flex-direction: column;
    padding: var(--space-4);
    margin-bottom: var(--space-3);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--card-radius);
    cursor: pointer;
    transition: all var(--duration-150) var(--ease-out);
  }

  .history-card:hover {
    border-color: var(--color-border-strong);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .history-card--selected {
    border-color: var(--color-primary);
    background: var(--color-primary-subtle);
  }

  .history-card--hovered {
    z-index: 10;
  }

  .history-card__header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .history-card__checkbox {
    position: relative;
    cursor: pointer;
  }

  .history-card__checkbox input {
    position: absolute;
    opacity: 0;
    cursor: pointer;
  }

  .checkmark {
    position: relative;
    display: inline-block;
    width: 16px;
    height: 16px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    transition: all var(--duration-150) var(--ease-out);
  }

  .history-card__checkbox input:checked ~ .checkmark {
    background: var(--color-primary);
    border-color: var(--color-primary);
  }

  .history-card__checkbox input:checked ~ .checkmark::after {
    content: '';
    position: absolute;
    left: 5px;
    top: 2px;
    width: 3px;
    height: 7px;
    border: solid white;
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .history-card__title {
    flex: 1;
    min-width: 0;
  }

  .history-card__title h3 {
    margin: 0;
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .history-card__status {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-left: var(--space-2);
  }

  .history-card__date {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .history-card__content {
    margin-bottom: var(--space-3);
  }

  .history-card__prompt {
    margin: 0 0 var(--space-2) 0;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    line-height: var(--leading-relaxed);
  }

  .history-card__tags {
    display: flex;
    gap: var(--space-1);
    flex-wrap: wrap;
  }

  .tag {
    padding: var(--space-1) var(--space-2);
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .tag--more {
    background: var(--color-primary-subtle);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .history-card__footer {
    margin-top: auto;
  }

  .history-card__stats {
    display: flex;
    gap: var(--space-4);
    font-size: var(--text-xs);
  }

  .stat__label {
    color: var(--color-text-muted);
    margin-right: var(--space-1);
  }

  .stat__value {
    color: var(--color-text-primary);
    font-weight: var(--font-medium);
  }

  .history-card__preview {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: var(--z-tooltip);
    margin-top: var(--space-2);
    padding: var(--space-4);
    background: var(--color-bg-overlay);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--card-radius);
    box-shadow: var(--shadow-lg);
    max-width: 400px;
  }

  .preview-content h4 {
    margin: 0 0 var(--space-3) 0;
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  .preview-details {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .preview-details p {
    margin: 0;
    font-size: var(--text-sm);
    line-height: var(--leading-relaxed);
    color: var(--color-text-secondary);
  }

  .preview-details strong {
    color: var(--color-text-primary);
    font-weight: var(--font-medium);
  }

  @media (max-width: 768px) {
    .history-card {
      padding: var(--space-3);
    }

    .history-card__header {
      flex-wrap: wrap;
      gap: var(--space-2);
    }

    .history-card__stats {
      flex-direction: column;
      gap: var(--space-2);
    }

    .history-card__preview {
      left: var(--space-2);
      right: var(--space-2);
      max-width: none;
    }
  }
</style>