<script lang="ts">
  import SummaryCard from './SummaryCard.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import RelativeTime from '$lib/components/common/RelativeTime.svelte';
  import { recentActivity } from '$lib/stores/summary';

  export let loading: boolean = false;
  export let limit: number = 5;

  $: activities = $recentActivity.slice(0, limit);

  function getActivityIcon(type: string) {
    switch (type) {
      case 'mission_started': return 'play';
      case 'mission_completed': return 'check';
      case 'mission_failed': return 'x';
      case 'spec_created': return 'file-plus';
      case 'config_changed': return 'settings';
      default: return 'activity';
    }
  }

  function getActivityColor(type: string) {
    switch (type) {
      case 'mission_started': return 'var(--color-info)';
      case 'mission_completed': return 'var(--color-success)';
      case 'mission_failed': return 'var(--color-error)';
      case 'spec_created': return 'var(--color-accent)';
      case 'config_changed': return 'var(--color-warning)';
      default: return 'var(--color-text-muted)';
    }
  }
</script>

<SummaryCard
  title="Recent Activity"
  icon="activity"
  href="/activity"
  {loading}
>
  <ul class="activity-list">
    {#each activities as activity (activity.id)}
      <li class="activity-item">
        <div class="activity-icon" style="color: {getActivityColor(activity.type)}">
          <Icon name={getActivityIcon(activity.type)} size={14} />
        </div>
        <div class="activity-content">
          <span class="activity-message">{activity.message}</span>
          <RelativeTime date={activity.timestamp} class="activity-time" />
        </div>
      </li>
    {/each}

    {#if activities.length === 0}
      <li class="empty-state">No recent activity</li>
    {/if}
  </ul>

  <svelte:fragment slot="footer">
    <a href="/activity" class="view-all">
      View all activity
      <Icon name="arrow-right" size={12} />
    </a>
  </svelte:fragment>
</SummaryCard>

<style>
  .activity-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .activity-item {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--color-border);
  }

  .activity-item:last-child {
    border-bottom: none;
  }

  .activity-icon {
    flex-shrink: 0;
    width: 1.5rem;
    height: 1.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-surface);
    border-radius: var(--radius-full);
  }

  .activity-content {
    flex: 1;
    min-width: 0;
  }

  .activity-message {
    display: block;
    font-size: var(--text-sm);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.activity-time) {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .empty-state {
    padding: var(--space-4);
    text-align: center;
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }

  .view-all {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    font-size: var(--text-sm);
    color: var(--color-accent);
    text-decoration: none;
    transition: color var(--duration-150) var(--ease-out);
  }

  .view-all:hover {
    color: var(--color-accent-hover);
    text-decoration: underline;
  }
</style>