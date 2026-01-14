# 306 - Deploy Metrics

**Phase:** 14 - Dashboard
**Spec ID:** 306
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 303-time-series
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create deployment metrics visualization components that display deployment frequency, success rates, rollback statistics, and deployment pipeline performance.

---

## Acceptance Criteria

- [x] `DeployMetrics.svelte` component created
- [x] Deployment frequency timeline
- [x] Success/failure rate visualization
- [x] Rollback tracking
- [x] Mean time to deploy (MTTD)
- [x] Pipeline stage breakdown
- [x] Environment comparison
- [x] Deployment history table

---

## Implementation Details

### 1. Deploy Metrics Component (web/src/lib/components/deploy/DeployMetrics.svelte)

```svelte
<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import type { DeploymentData, DeploymentSummary } from '$lib/types/deploy';
  import Icon from '$lib/components/common/Icon.svelte';
  import TimeSeriesChart from '$lib/components/charts/TimeSeriesChart.svelte';
  import SparkLine from '$lib/components/charts/SparkLine.svelte';

  export let summary: DeploymentSummary;
  export let deployments: DeploymentData[] = [];
  export let period: 'day' | 'week' | 'month' = 'week';

  let selectedEnvironment: string = 'all';

  $: filteredDeployments = selectedEnvironment === 'all'
    ? deployments
    : deployments.filter(d => d.environment === selectedEnvironment);

  $: environments = [...new Set(deployments.map(d => d.environment))];

  $: deployFrequencyData = calculateFrequency(filteredDeployments);
  $: successRate = summary.successful / summary.total * 100;

  function calculateFrequency(deps: DeploymentData[]) {
    // Group by day
    const groups = new Map<string, number>();
    deps.forEach(d => {
      const day = new Date(d.timestamp).toISOString().split('T')[0];
      groups.set(day, (groups.get(day) || 0) + 1);
    });
    return Array.from(groups.entries()).map(([date, count]) => ({
      timestamp: date,
      value: count
    }));
  }

  function formatDuration(ms: number): string {
    if (ms < 60000) return `${(ms / 1000).toFixed(0)}s`;
    if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
    return `${(ms / 3600000).toFixed(1)}h`;
  }

  function getStatusColor(status: DeploymentData['status']): string {
    switch (status) {
      case 'success': return 'var(--green-500)';
      case 'failed': return 'var(--red-500)';
      case 'rolled_back': return 'var(--yellow-500)';
      case 'in_progress': return 'var(--blue-500)';
      default: return 'var(--gray-500)';
    }
  }

  function getStatusIcon(status: DeploymentData['status']): string {
    switch (status) {
      case 'success': return 'check-circle';
      case 'failed': return 'x-circle';
      case 'rolled_back': return 'rotate-ccw';
      case 'in_progress': return 'loader';
      default: return 'circle';
    }
  }
</script>

<div class="deploy-metrics">
  <div class="metrics-header">
    <div class="header-left">
      <Icon name="upload-cloud" size={20} />
      <h3>Deployment Metrics</h3>
    </div>

    <div class="header-controls">
      <select bind:value={selectedEnvironment} class="env-select">
        <option value="all">All Environments</option>
        {#each environments as env}
          <option value={env}>{env}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="metrics-grid">
    <div class="metric-card">
      <div class="metric-header">
        <span class="metric-label">Total Deployments</span>
        <Icon name="layers" size={16} />
      </div>
      <div class="metric-value">{summary.total}</div>
      <div class="metric-change" class:positive={summary.change > 0}>
        {summary.change > 0 ? '+' : ''}{summary.change}% from last {period}
      </div>
    </div>

    <div class="metric-card">
      <div class="metric-header">
        <span class="metric-label">Success Rate</span>
        <Icon name="target" size={16} />
      </div>
      <div class="metric-value" style="color: {successRate >= 95 ? 'var(--green-500)' : successRate >= 80 ? 'var(--yellow-500)' : 'var(--red-500)'}">
        {successRate.toFixed(1)}%
      </div>
      <div class="metric-bar">
        <div class="bar-fill success" style="width: {successRate}%" />
      </div>
    </div>

    <div class="metric-card">
      <div class="metric-header">
        <span class="metric-label">Mean Deploy Time</span>
        <Icon name="clock" size={16} />
      </div>
      <div class="metric-value">{formatDuration(summary.meanDeployTime)}</div>
      <SparkLine
        data={summary.deployTimeTrend}
        height={30}
        color="var(--accent-color)"
      />
    </div>

    <div class="metric-card">
      <div class="metric-header">
        <span class="metric-label">Rollbacks</span>
        <Icon name="rotate-ccw" size={16} />
      </div>
      <div class="metric-value" style="color: {summary.rollbacks > 0 ? 'var(--yellow-500)' : 'var(--green-500)'}">
        {summary.rollbacks}
      </div>
      <div class="metric-subtext">
        {(summary.rollbacks / summary.total * 100).toFixed(1)}% rollback rate
      </div>
    </div>
  </div>

  <div class="frequency-section">
    <h4>Deployment Frequency</h4>
    <TimeSeriesChart
      data={[{
        id: 'deploys',
        label: 'Deployments',
        color: 'var(--accent-color)',
        points: deployFrequencyData
      }]}
      height={200}
      showLegend={false}
      showBrush={false}
    />
  </div>

  <div class="pipeline-section">
    <h4>Pipeline Performance</h4>
    <div class="pipeline-stages">
      {#each summary.pipelineStages as stage}
        <div class="stage">
          <div class="stage-header">
            <span class="stage-name">{stage.name}</span>
            <span class="stage-duration">{formatDuration(stage.avgDuration)}</span>
          </div>
          <div class="stage-bar">
            <div
              class="stage-fill"
              style="width: {(stage.avgDuration / summary.meanDeployTime) * 100}%"
            />
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="history-section">
    <h4>Recent Deployments</h4>
    <table class="deploy-table">
      <thead>
        <tr>
          <th>Status</th>
          <th>Version</th>
          <th>Environment</th>
          <th>Duration</th>
          <th>Deployed</th>
          <th>By</th>
        </tr>
      </thead>
      <tbody>
        {#each filteredDeployments.slice(0, 10) as deploy (deploy.id)}
          <tr class={deploy.status} transition:fade={{ duration: 100 }}>
            <td>
              <span class="status-badge" style="color: {getStatusColor(deploy.status)}">
                <Icon name={getStatusIcon(deploy.status)} size={14} />
                {deploy.status.replace('_', ' ')}
              </span>
            </td>
            <td class="version">{deploy.version}</td>
            <td>
              <span class="env-badge">{deploy.environment}</span>
            </td>
            <td class="duration">{formatDuration(deploy.duration)}</td>
            <td class="timestamp">{new Date(deploy.timestamp).toLocaleString()}</td>
            <td class="deployer">{deploy.deployedBy}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .deploy-metrics {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .metrics-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .header-left h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .env-select {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    background: var(--bg-primary);
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
    padding: 1.25rem;
  }

  @media (max-width: 1024px) {
    .metrics-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .metric-card {
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 0.5rem;
  }

  .metric-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .metric-label {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .metric-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .metric-change {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    margin-top: 0.25rem;
  }

  .metric-change.positive {
    color: var(--green-500);
  }

  .metric-bar {
    height: 0.375rem;
    background: var(--bg-primary);
    border-radius: 9999px;
    margin-top: 0.5rem;
    overflow: hidden;
  }

  .bar-fill.success {
    height: 100%;
    background: var(--green-500);
    border-radius: 9999px;
    transition: width 0.3s ease;
  }

  .metric-subtext {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    margin-top: 0.25rem;
  }

  .frequency-section,
  .pipeline-section,
  .history-section {
    padding: 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .frequency-section h4,
  .pipeline-section h4,
  .history-section h4 {
    margin: 0 0 1rem;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .pipeline-stages {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .stage-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.25rem;
  }

  .stage-name {
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .stage-duration {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    font-family: monospace;
  }

  .stage-bar {
    height: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
    overflow: hidden;
  }

  .stage-fill {
    height: 100%;
    background: var(--accent-color);
    border-radius: 9999px;
  }

  .deploy-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  .deploy-table th {
    text-align: left;
    padding: 0.75rem 0.5rem;
    font-weight: 500;
    color: var(--text-tertiary);
    border-bottom: 1px solid var(--border-color);
  }

  .deploy-table td {
    padding: 0.75rem 0.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    text-transform: capitalize;
    font-weight: 500;
  }

  .version {
    font-family: monospace;
    font-size: 0.75rem;
  }

  .env-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
    font-size: 0.6875rem;
    text-transform: uppercase;
  }

  .duration {
    font-family: monospace;
    color: var(--text-secondary);
  }

  .timestamp {
    color: var(--text-tertiary);
    font-size: 0.75rem;
  }

  .deployer {
    color: var(--text-secondary);
  }
</style>
```

### 2. Deploy Types (web/src/lib/types/deploy.ts)

```typescript
export interface DeploymentData {
  id: string;
  version: string;
  environment: string;
  status: 'success' | 'failed' | 'rolled_back' | 'in_progress';
  duration: number;
  timestamp: string;
  deployedBy: string;
  commitSha?: string;
  rollbackOf?: string;
}

export interface PipelineStage {
  name: string;
  avgDuration: number;
  successRate: number;
}

export interface DeploymentSummary {
  total: number;
  successful: number;
  failed: number;
  rollbacks: number;
  change: number;
  meanDeployTime: number;
  deployTimeTrend: number[];
  pipelineStages: PipelineStage[];
}
```

---

## Testing Requirements

1. Metrics cards show correct values
2. Environment filter works correctly
3. Frequency chart displays accurate data
4. Pipeline stages render proportionally
5. Deployment table sorts correctly
6. Status colors and icons are correct
7. Duration formatting is accurate

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [307-error-rate.md](307-error-rate.md)
- Used by: CI/CD dashboards, deployment views
