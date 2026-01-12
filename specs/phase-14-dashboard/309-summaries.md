# 309 - Summaries

**Phase:** 14 - Dashboard
**Spec ID:** 309
**Status:** Planned
**Dependencies:** 296-dashboard-layout
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create summary card components that provide at-a-glance overviews of key metrics, system health indicators, and quick access to important information.

---

## Acceptance Criteria

- [ ] `SummaryCard.svelte` base component created
- [ ] `MissionSummary.svelte` for mission stats
- [ ] `HealthSummary.svelte` for system health
- [ ] `ActivitySummary.svelte` for recent activity
- [ ] Configurable display options
- [ ] Responsive card layouts
- [ ] Interactive quick actions
- [ ] Real-time update support

---

## Implementation Details

### 1. Summary Card Base (web/src/lib/components/summary/SummaryCard.svelte)

```svelte
<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import Icon from '$lib/components/common/Icon.svelte';

  export let title: string;
  export let icon: string = 'info';
  export let variant: 'default' | 'success' | 'warning' | 'danger' | 'info' = 'default';
  export let loading: boolean = false;
  export let href: string | null = null;

  const variantColors = {
    default: { bg: 'var(--bg-secondary)', border: 'var(--border-color)', icon: 'var(--text-tertiary)' },
    success: { bg: 'var(--green-50)', border: 'var(--green-200)', icon: 'var(--green-500)' },
    warning: { bg: 'var(--yellow-50)', border: 'var(--yellow-200)', icon: 'var(--yellow-500)' },
    danger: { bg: 'var(--red-50)', border: 'var(--red-200)', icon: 'var(--red-500)' },
    info: { bg: 'var(--blue-50)', border: 'var(--blue-200)', icon: 'var(--blue-500)' }
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
    border-radius: 0.75rem;
    overflow: hidden;
    text-decoration: none;
    transition: all 0.2s ease;
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
    gap: 0.5rem;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--card-border);
  }

  .header-icon {
    color: var(--icon-color);
  }

  .header-title {
    flex: 1;
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  :global(.arrow-icon) {
    color: var(--text-tertiary);
    transition: transform 0.2s ease;
  }

  .summary-card:hover :global(.arrow-icon) {
    transform: translateX(2px);
  }

  .card-content {
    flex: 1;
    padding: 1rem 1.25rem;
  }

  .card-footer {
    padding: 0.75rem 1.25rem;
    border-top: 1px solid var(--card-border);
    background: var(--bg-primary);
  }

  .loading-skeleton {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .skeleton-line {
    height: 1rem;
    background: var(--bg-secondary);
    border-radius: 0.25rem;
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
```

### 2. Mission Summary (web/src/lib/components/summary/MissionSummary.svelte)

```svelte
<script lang="ts">
  import SummaryCard from './SummaryCard.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import { missionStats } from '$lib/stores/missions';

  export let loading: boolean = false;

  $: stats = $missionStats;
</script>

<SummaryCard
  title="Mission Overview"
  icon="target"
  variant={stats.failed > 0 ? 'warning' : 'success'}
  href="/missions"
  {loading}
>
  <div class="stat-grid">
    <div class="stat-item">
      <span class="stat-value">{stats.total}</span>
      <span class="stat-label">Total</span>
    </div>
    <div class="stat-item running">
      <span class="stat-value">{stats.running}</span>
      <span class="stat-label">Running</span>
    </div>
    <div class="stat-item success">
      <span class="stat-value">{stats.completed}</span>
      <span class="stat-label">Completed</span>
    </div>
    <div class="stat-item danger">
      <span class="stat-value">{stats.failed}</span>
      <span class="stat-label">Failed</span>
    </div>
  </div>

  <div class="progress-bar">
    <div
      class="progress-segment success"
      style="width: {(stats.completed / stats.total) * 100}%"
    />
    <div
      class="progress-segment running"
      style="width: {(stats.running / stats.total) * 100}%"
    />
    <div
      class="progress-segment danger"
      style="width: {(stats.failed / stats.total) * 100}%"
    />
  </div>

  <svelte:fragment slot="footer">
    <div class="footer-content">
      <span class="footer-stat">
        <Icon name="clock" size={12} />
        Avg: {stats.avgDuration}
      </span>
      <span class="footer-stat">
        <Icon name="check" size={12} />
        {stats.successRate}% success
      </span>
    </div>
  </svelte:fragment>
</SummaryCard>

<style>
  .stat-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .stat-item {
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .stat-item.running .stat-value {
    color: var(--blue-500);
  }

  .stat-item.success .stat-value {
    color: var(--green-500);
  }

  .stat-item.danger .stat-value {
    color: var(--red-500);
  }

  .stat-label {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
  }

  .progress-bar {
    display: flex;
    height: 0.375rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
    overflow: hidden;
  }

  .progress-segment {
    transition: width 0.3s ease;
  }

  .progress-segment.success {
    background: var(--green-500);
  }

  .progress-segment.running {
    background: var(--blue-500);
  }

  .progress-segment.danger {
    background: var(--red-500);
  }

  .footer-content {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .footer-stat {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
</style>
```

### 3. Health Summary (web/src/lib/components/summary/HealthSummary.svelte)

```svelte
<script lang="ts">
  import SummaryCard from './SummaryCard.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import { systemHealth } from '$lib/stores/system';

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
      case 'healthy': return 'var(--green-500)';
      case 'degraded': return 'var(--yellow-500)';
      case 'unhealthy': return 'var(--red-500)';
      default: return 'var(--gray-500)';
    }
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
      <button class="refresh-btn" on:click|stopPropagation>
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
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .status-label {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .service-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .service-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0;
  }

  .service-indicator {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
  }

  .service-name {
    flex: 1;
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .service-latency {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    font-family: monospace;
  }

  .footer-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--border-color);
    background: transparent;
    border-radius: 0.25rem;
    font-size: 0.6875rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .refresh-btn:hover {
    background: var(--bg-hover);
  }
</style>
```

### 4. Activity Summary (web/src/lib/components/summary/ActivitySummary.svelte)

```svelte
<script lang="ts">
  import SummaryCard from './SummaryCard.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import RelativeTime from '$lib/components/common/RelativeTime.svelte';
  import { recentActivity } from '$lib/stores/activity';

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
      case 'mission_started': return 'var(--blue-500)';
      case 'mission_completed': return 'var(--green-500)';
      case 'mission_failed': return 'var(--red-500)';
      case 'spec_created': return 'var(--purple-500)';
      case 'config_changed': return 'var(--orange-500)';
      default: return 'var(--gray-500)';
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
    gap: 0.75rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-color);
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
    background: var(--bg-secondary);
    border-radius: 50%;
  }

  .activity-content {
    flex: 1;
    min-width: 0;
  }

  .activity-message {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.activity-time) {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
  }

  .empty-state {
    padding: 1rem;
    text-align: center;
    font-size: 0.8125rem;
    color: var(--text-tertiary);
  }

  .view-all {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    font-size: 0.8125rem;
    color: var(--accent-color);
    text-decoration: none;
  }

  .view-all:hover {
    text-decoration: underline;
  }
</style>
```

---

## Testing Requirements

1. Summary cards render with correct variants
2. Loading states show skeleton
3. Mission stats calculate correctly
4. Health indicators reflect system status
5. Activity list truncates at limit
6. Links navigate correctly
7. Real-time updates work

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [310-export-reports.md](310-export-reports.md)
- Used by: Dashboard overview, summary widgets
