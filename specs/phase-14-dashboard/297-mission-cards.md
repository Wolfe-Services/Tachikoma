# 297 - Mission Cards

**Phase:** 14 - Dashboard
**Spec ID:** 297
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 011-common-core-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create reusable mission card components that display mission status, progress, and key metrics in a visually appealing and informative format.

---

## Acceptance Criteria

- [ ] `MissionCard.svelte` component created
- [ ] `MissionCardGrid.svelte` for card layouts
- [ ] Status indicators with color coding
- [ ] Progress bars for active missions
- [ ] Token usage display
- [ ] Expandable card details
- [ ] Action buttons (pause, resume, cancel)
- [ ] Animation on state changes

---

## Implementation Details

### 1. Mission Card Component (web/src/lib/components/missions/MissionCard.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import type { Mission, MissionState } from '$lib/types/mission';
  import Icon from '$lib/components/common/Icon.svelte';
  import ProgressBar from '$lib/components/common/ProgressBar.svelte';
  import TokenUsage from '$lib/components/common/TokenUsage.svelte';
  import RelativeTime from '$lib/components/common/RelativeTime.svelte';

  export let mission: Mission;
  export let expanded: boolean = false;
  export let compact: boolean = false;

  const dispatch = createEventDispatcher<{
    select: { missionId: string };
    pause: { missionId: string };
    resume: { missionId: string };
    cancel: { missionId: string };
  }>();

  $: stateConfig = getStateConfig(mission.state);
  $: progressPercent = calculateProgress(mission);

  function getStateConfig(state: MissionState): {
    icon: string;
    color: string;
    label: string;
    bgColor: string;
  } {
    const configs: Record<MissionState, typeof stateConfig> = {
      idle: { icon: 'circle', color: 'var(--gray-500)', label: 'Idle', bgColor: 'var(--gray-100)' },
      running: { icon: 'play', color: 'var(--blue-500)', label: 'Running', bgColor: 'var(--blue-100)' },
      paused: { icon: 'pause', color: 'var(--yellow-500)', label: 'Paused', bgColor: 'var(--yellow-100)' },
      complete: { icon: 'check', color: 'var(--green-500)', label: 'Complete', bgColor: 'var(--green-100)' },
      error: { icon: 'x-circle', color: 'var(--red-500)', label: 'Error', bgColor: 'var(--red-100)' },
      redlined: { icon: 'alert-triangle', color: 'var(--orange-500)', label: 'Redlined', bgColor: 'var(--orange-100)' }
    };
    return configs[state];
  }

  function calculateProgress(mission: Mission): number {
    if (mission.state === 'complete') return 100;
    if (!mission.totalSteps || mission.totalSteps === 0) return 0;
    return Math.round((mission.completedSteps / mission.totalSteps) * 100);
  }

  function handleAction(action: 'pause' | 'resume' | 'cancel') {
    dispatch(action, { missionId: mission.id });
  }
</script>

<article
  class="mission-card"
  class:compact
  class:expanded
  class:running={mission.state === 'running'}
  role="button"
  tabindex="0"
  on:click={() => dispatch('select', { missionId: mission.id })}
  on:keypress={(e) => e.key === 'Enter' && dispatch('select', { missionId: mission.id })}
  transition:fly={{ y: 20, duration: 200 }}
>
  <div class="card-header">
    <div class="status-badge" style="background: {stateConfig.bgColor}; color: {stateConfig.color}">
      <Icon name={stateConfig.icon} size={14} />
      <span>{stateConfig.label}</span>
    </div>

    <div class="card-actions">
      {#if mission.state === 'running'}
        <button
          class="action-btn"
          on:click|stopPropagation={() => handleAction('pause')}
          title="Pause mission"
        >
          <Icon name="pause" size={16} />
        </button>
      {:else if mission.state === 'paused'}
        <button
          class="action-btn"
          on:click|stopPropagation={() => handleAction('resume')}
          title="Resume mission"
        >
          <Icon name="play" size={16} />
        </button>
      {/if}

      {#if mission.state === 'running' || mission.state === 'paused'}
        <button
          class="action-btn danger"
          on:click|stopPropagation={() => handleAction('cancel')}
          title="Cancel mission"
        >
          <Icon name="x" size={16} />
        </button>
      {/if}
    </div>
  </div>

  <div class="card-body">
    <h3 class="mission-title">{mission.title}</h3>

    {#if !compact}
      <p class="mission-description">{mission.description}</p>
    {/if}

    {#if mission.state === 'running' || mission.state === 'paused'}
      <div class="progress-section">
        <ProgressBar
          value={progressPercent}
          animated={mission.state === 'running'}
          color={stateConfig.color}
        />
        <span class="progress-label">{progressPercent}%</span>
      </div>
    {/if}

    {#if mission.currentStep && !compact}
      <div class="current-step" in:fade>
        <Icon name="arrow-right" size={14} />
        <span>{mission.currentStep}</span>
      </div>
    {/if}
  </div>

  <div class="card-footer">
    <div class="meta-info">
      <span class="spec-ref">
        <Icon name="file-text" size={14} />
        {mission.specId}
      </span>
      <RelativeTime date={mission.updatedAt} />
    </div>

    <TokenUsage
      input={mission.tokenUsage.input}
      output={mission.tokenUsage.output}
      compact={!expanded}
    />
  </div>

  {#if expanded}
    <div class="card-expanded" transition:fly={{ y: -10, duration: 150 }}>
      <div class="expanded-section">
        <h4>Mission Log</h4>
        <ul class="log-list">
          {#each mission.recentLogs as log}
            <li class="log-item {log.level}">
              <span class="log-time">{log.timestamp}</span>
              <span class="log-message">{log.message}</span>
            </li>
          {/each}
        </ul>
      </div>

      {#if mission.error}
        <div class="error-section">
          <h4>Error Details</h4>
          <pre class="error-message">{mission.error}</pre>
        </div>
      {/if}
    </div>
  {/if}
</article>

<style>
  .mission-card {
    display: flex;
    flex-direction: column;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .mission-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
  }

  .mission-card.running {
    border-color: var(--blue-300);
    box-shadow: 0 0 0 1px var(--blue-100);
  }

  .mission-card:focus {
    outline: 2px solid var(--accent-color);
    outline-offset: 2px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.625rem;
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 9999px;
  }

  .card-actions {
    display: flex;
    gap: 0.25rem;
  }

  .action-btn {
    padding: 0.375rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .action-btn.danger:hover {
    background: var(--red-100);
    color: var(--red-600);
  }

  .card-body {
    padding: 1rem;
    flex: 1;
  }

  .mission-title {
    margin: 0 0 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .mission-description {
    margin: 0 0 0.75rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .progress-section {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .progress-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
    min-width: 3rem;
  }

  .current-step {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-secondary);
    border-radius: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .meta-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .spec-ref {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .card-expanded {
    padding: 1rem;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .expanded-section h4 {
    margin: 0 0 0.5rem;
    font-size: 0.8125rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .log-list {
    list-style: none;
    padding: 0;
    margin: 0;
    max-height: 150px;
    overflow-y: auto;
  }

  .log-item {
    display: flex;
    gap: 0.5rem;
    padding: 0.25rem 0;
    font-size: 0.75rem;
    font-family: monospace;
  }

  .log-time {
    color: var(--text-tertiary);
    white-space: nowrap;
  }

  .log-item.error .log-message {
    color: var(--red-600);
  }

  .log-item.warn .log-message {
    color: var(--yellow-600);
  }

  .error-section {
    margin-top: 1rem;
  }

  .error-message {
    margin: 0;
    padding: 0.75rem;
    background: var(--red-50);
    border: 1px solid var(--red-200);
    border-radius: 0.375rem;
    font-size: 0.75rem;
    color: var(--red-700);
    overflow-x: auto;
  }

  /* Compact variant */
  .compact .card-body {
    padding: 0.75rem;
  }

  .compact .mission-title {
    font-size: 0.875rem;
  }
</style>
```

### 2. Mission Card Grid (web/src/lib/components/missions/MissionCardGrid.svelte)

```svelte
<script lang="ts">
  import type { Mission } from '$lib/types/mission';
  import MissionCard from './MissionCard.svelte';

  export let missions: Mission[] = [];
  export let columns: 1 | 2 | 3 | 4 = 3;
  export let compact: boolean = false;

  let expandedId: string | null = null;

  function handleSelect(event: CustomEvent<{ missionId: string }>) {
    expandedId = expandedId === event.detail.missionId ? null : event.detail.missionId;
  }

  function handlePause(event: CustomEvent<{ missionId: string }>) {
    // Dispatch to parent or call store action
  }

  function handleResume(event: CustomEvent<{ missionId: string }>) {
    // Dispatch to parent or call store action
  }

  function handleCancel(event: CustomEvent<{ missionId: string }>) {
    // Dispatch to parent or call store action
  }
</script>

<div class="mission-grid" style="--columns: {columns}">
  {#each missions as mission (mission.id)}
    <MissionCard
      {mission}
      {compact}
      expanded={expandedId === mission.id}
      on:select={handleSelect}
      on:pause={handlePause}
      on:resume={handleResume}
      on:cancel={handleCancel}
    />
  {/each}

  {#if missions.length === 0}
    <div class="empty-state">
      <p>No missions to display</p>
    </div>
  {/if}
</div>

<style>
  .mission-grid {
    display: grid;
    grid-template-columns: repeat(var(--columns), 1fr);
    gap: 1.5rem;
  }

  @media (max-width: 1200px) {
    .mission-grid {
      grid-template-columns: repeat(min(var(--columns), 2), 1fr);
    }
  }

  @media (max-width: 768px) {
    .mission-grid {
      grid-template-columns: 1fr;
    }
  }

  .empty-state {
    grid-column: 1 / -1;
    padding: 3rem;
    text-align: center;
    color: var(--text-tertiary);
  }
</style>
```

### 3. Mission Type Definitions (web/src/lib/types/mission.ts)

```typescript
export type MissionState =
  | 'idle'
  | 'running'
  | 'paused'
  | 'complete'
  | 'error'
  | 'redlined';

export interface TokenUsage {
  input: number;
  output: number;
  total: number;
  cost: number;
}

export interface MissionLog {
  timestamp: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

export interface Mission {
  id: string;
  specId: string;
  title: string;
  description: string;
  state: MissionState;
  currentStep: string | null;
  completedSteps: number;
  totalSteps: number;
  tokenUsage: TokenUsage;
  recentLogs: MissionLog[];
  error: string | null;
  createdAt: string;
  updatedAt: string;
  startedAt: string | null;
  completedAt: string | null;
}
```

---

## Testing Requirements

1. Cards render correctly for all states
2. State transitions animate smoothly
3. Actions dispatch correct events
4. Progress bar updates in real-time
5. Expanded view shows/hides correctly
6. Grid responds to viewport changes
7. Keyboard navigation works

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [298-active-status.md](298-active-status.md)
- Used by: Dashboard overview, mission list views
