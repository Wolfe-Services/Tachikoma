<script lang="ts">
  import SummaryCard from './SummaryCard.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import { missionStats } from '$lib/stores/summary';

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
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .stat-item {
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: var(--text-xl);
    font-weight: var(--font-bold);
    color: var(--color-text-primary);
  }

  .stat-item.running .stat-value {
    color: var(--color-info);
  }

  .stat-item.success .stat-value {
    color: var(--color-success);
  }

  .stat-item.danger .stat-value {
    color: var(--color-error);
  }

  .stat-label {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .progress-bar {
    display: flex;
    height: 0.375rem;
    background: var(--color-bg-surface);
    border-radius: var(--radius-full);
    overflow: hidden;
  }

  .progress-segment {
    transition: width var(--duration-300) var(--ease-out);
  }

  .progress-segment.success {
    background: var(--color-success);
  }

  .progress-segment.running {
    background: var(--color-info);
  }

  .progress-segment.danger {
    background: var(--color-error);
  }

  .footer-content {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  .footer-stat {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }
</style>