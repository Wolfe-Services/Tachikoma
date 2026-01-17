<script lang="ts">
  import SummaryCard from './SummaryCard.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import { systemHealth, refreshSystemHealth } from '$lib/stores/summary';

  export let loading: boolean = false;

  $: health = $systemHealth;
  $: overallStatus = getOverallStatus(health);

  function getOverallStatus(h: typeof health) {
    if (h.status === 'healthy') return { variant: 'success' as const, label: 'All Systems Operational' };
    if (h.status === 'degraded') return { variant: 'warning' as const, label: 'Degraded Performance' };
    return { variant: 'danger' as const, label: 'System Issues Detected' };
  }

  function getStatusIcon(status: string) {
    switch (status) {
      case 'healthy': return 'check-circle';
      case 'degraded': return 'alert-circle';
      case 'unhealthy': return 'x-circle';
      default: return 'circle';
    }
  }

  function getStatusColor(status: string) {
    switch (status) {
      case 'healthy': return 'var(--color-success)';
      case 'degraded': return 'var(--color-warning)';
      case 'unhealthy': return 'var(--color-error)';
      default: return 'var(--color-text-muted)';
    }
  }

  function handleRefresh(event: Event) {
    event.stopPropagation();
    refreshSystemHealth();
  }
</script>

<SummaryCard
  title="System Health"
  icon="heart"
  variant={overallStatus.variant}
  href="/system"
  {loading}
>
  <div class="status-header">
    <Icon name={getStatusIcon(health.status)} size={24} style="color: {getStatusColor(health.status)}" />
    <span class="status-label">{overallStatus.label}</span>
  </div>

  <ul class="service-list">
    {#each health.services as service}
      <li class="service-item">
        <span class="service-indicator" style="background: {getStatusColor(service.status)}" />
        <span class="service-name">{service.name}</span>
        <span class="service-latency">{service.latency}ms</span>
      </li>
    {/each}
  </ul>

  <svelte:fragment slot="footer">
    <div class="footer-content">
      <span>Last checked: {health.lastCheck}</span>
      <button class="refresh-btn" on:click={handleRefresh}>
        <Icon name="refresh-cw" size={12} />
        Refresh
      </button>
    </div>
  </svelte:fragment>
</SummaryCard>

<style>
  .status-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }

  .status-label {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  .service-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .service-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) 0;
  }

  .service-indicator {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: var(--radius-full);
  }

  .service-name {
    flex: 1;
    font-size: var(--text-sm);
    color: var(--color-text-primary);
  }

  .service-latency {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .footer-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--duration-150) var(--ease-out);
  }

  .refresh-btn:hover {
    background: var(--color-bg-hover);
  }
</style>