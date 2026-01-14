# 298 - Active Status

**Phase:** 14 - Dashboard
**Spec ID:** 298
**Status:** Planned
**Dependencies:** 297-mission-cards, 314-realtime-updates
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create an active status component that displays real-time information about currently running missions, including live progress updates, current operations, and resource utilization.

---

## Acceptance Criteria

- [x] `ActiveStatus.svelte` component created
- [x] Real-time mission progress updates
- [x] Current operation display
- [x] Resource utilization indicators
- [x] WebSocket integration for live updates
- [x] Pulse animation for active state
- [x] Expandable detail panel
- [x] Multi-mission support

---

## Implementation Details

### 1. Active Status Component (web/src/lib/components/status/ActiveStatus.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fly, fade, scale } from 'svelte/transition';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import type { ActiveMission } from '$lib/types/mission';
  import { activeMissions } from '$lib/stores/missions';
  import { wsConnection } from '$lib/stores/websocket';
  import Icon from '$lib/components/common/Icon.svelte';
  import CircularProgress from '$lib/components/common/CircularProgress.svelte';

  export let maxDisplay: number = 3;
  export let showDetails: boolean = true;

  let expanded = false;
  let selectedMissionId: string | null = null;

  $: displayedMissions = $activeMissions.slice(0, maxDisplay);
  $: hasMore = $activeMissions.length > maxDisplay;
  $: totalActive = $activeMissions.length;

  const progressValue = tweened(0, {
    duration: 300,
    easing: cubicOut
  });

  $: if ($activeMissions.length > 0) {
    const avgProgress = $activeMissions.reduce((sum, m) => sum + m.progress, 0) / $activeMissions.length;
    progressValue.set(avgProgress);
  }

  function selectMission(id: string) {
    selectedMissionId = selectedMissionId === id ? null : id;
  }
</script>

<div class="active-status" class:expanded class:has-active={totalActive > 0}>
  <div class="status-header">
    <div class="status-indicator" class:active={totalActive > 0}>
      {#if totalActive > 0}
        <span class="pulse" />
      {/if}
      <Icon name={totalActive > 0 ? 'activity' : 'circle'} size={18} />
    </div>

    <div class="status-info">
      <span class="status-label">
        {#if totalActive > 0}
          {totalActive} Active Mission{totalActive > 1 ? 's' : ''}
        {:else}
          No Active Missions
        {/if}
      </span>

      {#if totalActive > 0}
        <span class="status-progress">
          {Math.round($progressValue)}% overall
        </span>
      {/if}
    </div>

    {#if totalActive > 0}
      <button
        class="expand-btn"
        on:click={() => expanded = !expanded}
        aria-expanded={expanded}
        aria-label={expanded ? 'Collapse' : 'Expand'}
      >
        <Icon name={expanded ? 'chevron-up' : 'chevron-down'} size={16} />
      </button>
    {/if}
  </div>

  {#if totalActive > 0 && expanded}
    <div class="status-content" transition:fly={{ y: -10, duration: 200 }}>
      {#each displayedMissions as mission (mission.id)}
        <div
          class="active-mission"
          class:selected={selectedMissionId === mission.id}
          on:click={() => selectMission(mission.id)}
          on:keypress={(e) => e.key === 'Enter' && selectMission(mission.id)}
          role="button"
          tabindex="0"
          transition:fade={{ duration: 150 }}
        >
          <div class="mission-progress">
            <CircularProgress
              value={mission.progress}
              size={40}
              strokeWidth={3}
              animated
            />
          </div>

          <div class="mission-info">
            <span class="mission-title">{mission.title}</span>
            <span class="mission-operation">
              {mission.currentOperation || 'Initializing...'}
            </span>
          </div>

          <div class="mission-stats">
            <span class="stat">
              <Icon name="clock" size={12} />
              {formatDuration(mission.elapsedTime)}
            </span>
            <span class="stat">
              <Icon name="cpu" size={12} />
              {mission.tokensPerSecond} tok/s
            </span>
          </div>
        </div>

        {#if selectedMissionId === mission.id && showDetails}
          <div class="mission-details" transition:fly={{ y: -5, duration: 150 }}>
            <div class="detail-row">
              <span class="detail-label">Spec</span>
              <span class="detail-value">{mission.specId}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Tokens Used</span>
              <span class="detail-value">
                {mission.tokenUsage.input.toLocaleString()} / {mission.tokenUsage.output.toLocaleString()}
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Current Step</span>
              <span class="detail-value">{mission.currentStep} / {mission.totalSteps}</span>
            </div>

            {#if mission.recentOutput}
              <div class="recent-output">
                <pre>{mission.recentOutput}</pre>
              </div>
            {/if}
          </div>
        {/if}
      {/each}

      {#if hasMore}
        <a href="/missions" class="view-all">
          View all {totalActive} missions
          <Icon name="arrow-right" size={14} />
        </a>
      {/if}
    </div>
  {/if}
</div>

<script context="module" lang="ts">
  function formatDuration(seconds: number): string {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
  }
</script>

<style>
  .active-status {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .active-status.has-active {
    border-color: var(--blue-300);
  }

  .status-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
  }

  .status-indicator {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    background: var(--bg-secondary);
    border-radius: 50%;
    color: var(--text-tertiary);
  }

  .status-indicator.active {
    background: var(--blue-100);
    color: var(--blue-600);
  }

  .pulse {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: var(--blue-400);
    animation: pulse 2s ease-out infinite;
  }

  @keyframes pulse {
    0% {
      opacity: 0.5;
      transform: scale(1);
    }
    100% {
      opacity: 0;
      transform: scale(1.5);
    }
  }

  .status-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .status-label {
    font-weight: 600;
    color: var(--text-primary);
  }

  .status-progress {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .expand-btn {
    padding: 0.5rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    cursor: pointer;
    color: var(--text-secondary);
  }

  .expand-btn:hover {
    background: var(--bg-hover);
  }

  .status-content {
    border-top: 1px solid var(--border-color);
    padding: 0.5rem;
  }

  .active-mission {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .active-mission:hover {
    background: var(--bg-hover);
  }

  .active-mission.selected {
    background: var(--bg-secondary);
  }

  .mission-info {
    flex: 1;
    min-width: 0;
  }

  .mission-title {
    display: block;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mission-operation {
    display: block;
    font-size: 0.75rem;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mission-stats {
    display: flex;
    gap: 0.75rem;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .mission-details {
    margin: 0.5rem 0.75rem 0.75rem;
    padding: 0.75rem;
    background: var(--bg-primary);
    border-radius: 0.375rem;
    border: 1px solid var(--border-color);
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 0.25rem 0;
    font-size: 0.75rem;
  }

  .detail-label {
    color: var(--text-tertiary);
  }

  .detail-value {
    color: var(--text-primary);
    font-family: monospace;
  }

  .recent-output {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 0.25rem;
    max-height: 100px;
    overflow-y: auto;
  }

  .recent-output pre {
    margin: 0;
    font-size: 0.6875rem;
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .view-all {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem;
    color: var(--accent-color);
    text-decoration: none;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .view-all:hover {
    text-decoration: underline;
  }
</style>
```

### 2. Active Mission Type (web/src/lib/types/mission.ts addition)

```typescript
export interface ActiveMission {
  id: string;
  specId: string;
  title: string;
  state: 'running' | 'paused';
  progress: number;
  currentOperation: string | null;
  currentStep: number;
  totalSteps: number;
  elapsedTime: number;
  tokensPerSecond: number;
  tokenUsage: TokenUsage;
  recentOutput: string | null;
  startedAt: string;
}
```

### 3. Active Missions Store (web/src/lib/stores/missions.ts addition)

```typescript
import { writable, derived } from 'svelte/store';
import type { ActiveMission } from '$lib/types/mission';

export const activeMissions = writable<ActiveMission[]>([]);

export const activeMissionCount = derived(
  activeMissions,
  $missions => $missions.length
);

export const averageProgress = derived(
  activeMissions,
  $missions => {
    if ($missions.length === 0) return 0;
    return $missions.reduce((sum, m) => sum + m.progress, 0) / $missions.length;
  }
);

export function updateMissionProgress(id: string, progress: number, operation: string | null) {
  activeMissions.update(missions =>
    missions.map(m =>
      m.id === id ? { ...m, progress, currentOperation: operation } : m
    )
  );
}

export function addActiveMission(mission: ActiveMission) {
  activeMissions.update(missions => [...missions, mission]);
}

export function removeActiveMission(id: string) {
  activeMissions.update(missions => missions.filter(m => m.id !== id));
}
```

---

## Testing Requirements

1. Component shows correct active count
2. Progress updates animate smoothly
3. Real-time updates via WebSocket work
4. Expanded view toggles correctly
5. Mission selection highlights properly
6. Details panel shows correct information
7. Pulse animation runs when active

---

## Related Specs

- Depends on: [297-mission-cards.md](297-mission-cards.md)
- Related: [314-realtime-updates.md](314-realtime-updates.md)
- Next: [299-recent-missions.md](299-recent-missions.md)
