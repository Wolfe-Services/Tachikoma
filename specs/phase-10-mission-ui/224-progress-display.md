# 224 - Progress Display Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 224
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~11% of Sonnet window

---

## Objective

Create a progress display component that shows real-time mission execution progress, including step indicators, percentage completion, current action status, and estimated time remaining.

---

## Acceptance Criteria

- [ ] Visual progress bar with percentage
- [ ] Step counter (current/total)
- [ ] Current action description
- [ ] Elapsed and estimated time display
- [ ] Animated transitions for progress updates
- [ ] Compact and expanded view modes
- [ ] Accessible progress announcements

---

## Implementation Details

### 1. Types (src/lib/types/progress.ts)

```typescript
/**
 * Types for progress display functionality.
 */

export interface ProgressInfo {
  percentage: number;
  currentStep: number;
  totalSteps: number;
  currentAction: string;
  elapsedMs: number;
  estimatedRemainingMs: number;
  stepsCompleted: StepInfo[];
  isPaused: boolean;
  isIndeterminate: boolean;
}

export interface StepInfo {
  number: number;
  name: string;
  status: StepStatus;
  duration: number;
  startedAt: string;
  completedAt?: string;
}

export type StepStatus = 'pending' | 'running' | 'complete' | 'error' | 'skipped';

export interface ProgressDisplayConfig {
  showSteps: boolean;
  showTime: boolean;
  showCurrentAction: boolean;
  animate: boolean;
  announceChanges: boolean;
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return 'less than a second';

  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  }
  return `${seconds}s`;
}

export function estimateRemaining(
  elapsed: number,
  percentage: number
): number {
  if (percentage <= 0) return 0;
  const total = elapsed / (percentage / 100);
  return Math.max(0, total - elapsed);
}
```

### 2. Progress Display Component (src/lib/components/mission/ProgressDisplay.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import type { ProgressInfo, ProgressDisplayConfig } from '$lib/types/progress';
  import { formatDuration, estimateRemaining } from '$lib/types/progress';
  import StepTimeline from './StepTimeline.svelte';

  export let progress: ProgressInfo;
  export let config: ProgressDisplayConfig = {
    showSteps: true,
    showTime: true,
    showCurrentAction: true,
    animate: true,
    announceChanges: true,
  };
  export let compact = false;

  // Animated percentage
  const animatedPercentage = tweened(0, {
    duration: config.animate ? 400 : 0,
    easing: cubicOut,
  });

  // Timer for elapsed time
  let elapsedInterval: ReturnType<typeof setInterval> | null = null;
  let displayedElapsed = 0;
  let startTime = Date.now();

  $: if (progress.percentage !== $animatedPercentage) {
    animatedPercentage.set(progress.percentage);
  }

  $: estimatedRemaining = estimateRemaining(displayedElapsed, progress.percentage);

  $: progressColor = getProgressColor(progress.percentage, progress.isPaused);

  function getProgressColor(percentage: number, isPaused: boolean): string {
    if (isPaused) return 'var(--color-warning)';
    if (percentage >= 100) return 'var(--color-success)';
    return 'var(--color-primary)';
  }

  function updateElapsed() {
    if (!progress.isPaused) {
      displayedElapsed = Date.now() - startTime + progress.elapsedMs;
    }
  }

  onMount(() => {
    startTime = Date.now();
    displayedElapsed = progress.elapsedMs;
    elapsedInterval = setInterval(updateElapsed, 1000);
  });

  onDestroy(() => {
    if (elapsedInterval) {
      clearInterval(elapsedInterval);
    }
  });

  // Accessibility announcement
  let lastAnnouncement = '';
  $: if (config.announceChanges) {
    const announcement = `${Math.round(progress.percentage)}% complete, step ${progress.currentStep} of ${progress.totalSteps}`;
    if (announcement !== lastAnnouncement && Math.abs(progress.percentage - parseInt(lastAnnouncement)) >= 10) {
      lastAnnouncement = announcement;
    }
  }
</script>

<div
  class="progress-display"
  class:progress-display--compact={compact}
  role="progressbar"
  aria-valuenow={progress.percentage}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuetext="{Math.round(progress.percentage)}% complete"
  aria-label="Mission progress"
>
  <!-- Progress Bar -->
  <div class="progress-bar">
    <div
      class="progress-bar__fill"
      class:progress-bar__fill--indeterminate={progress.isIndeterminate}
      class:progress-bar__fill--paused={progress.isPaused}
      style="width: {$animatedPercentage}%; background-color: {progressColor}"
    ></div>
  </div>

  <!-- Stats Row -->
  <div class="progress-stats">
    <!-- Percentage -->
    <div class="progress-stat progress-stat--percentage">
      <span class="progress-stat__value">{Math.round($animatedPercentage)}%</span>
      {#if !compact}
        <span class="progress-stat__label">Complete</span>
      {/if}
    </div>

    <!-- Steps -->
    {#if config.showSteps}
      <div class="progress-stat">
        <span class="progress-stat__value">
          {progress.currentStep}/{progress.totalSteps}
        </span>
        {#if !compact}
          <span class="progress-stat__label">Steps</span>
        {/if}
      </div>
    {/if}

    <!-- Time -->
    {#if config.showTime && !compact}
      <div class="progress-stat">
        <span class="progress-stat__value">{formatDuration(displayedElapsed)}</span>
        <span class="progress-stat__label">Elapsed</span>
      </div>

      {#if estimatedRemaining > 0 && progress.percentage < 100}
        <div class="progress-stat">
          <span class="progress-stat__value">~{formatDuration(estimatedRemaining)}</span>
          <span class="progress-stat__label">Remaining</span>
        </div>
      {/if}
    {/if}

    <!-- Status Badge -->
    {#if progress.isPaused}
      <div class="progress-status progress-status--paused">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
          <path d="M3 2h2v8H3V2zm4 0h2v8H7V2z"/>
        </svg>
        Paused
      </div>
    {:else if progress.percentage >= 100}
      <div class="progress-status progress-status--complete">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
          <path d="M10.28 3.28a.75.75 0 010 1.06l-5.25 5.25a.75.75 0 01-1.06 0L1.72 7.34a.75.75 0 011.06-1.06l1.72 1.72 4.72-4.72a.75.75 0 011.06 0z"/>
        </svg>
        Complete
      </div>
    {/if}
  </div>

  <!-- Current Action -->
  {#if config.showCurrentAction && progress.currentAction && !compact}
    <div class="progress-action">
      <span class="progress-action__icon">
        {#if !progress.isPaused}
          <span class="spinner"></span>
        {/if}
      </span>
      <span class="progress-action__text">{progress.currentAction}</span>
    </div>
  {/if}

  <!-- Step Timeline -->
  {#if config.showSteps && !compact && progress.stepsCompleted.length > 0}
    <StepTimeline steps={progress.stepsCompleted} />
  {/if}

  <!-- Screen reader announcements -->
  <div class="sr-only" aria-live="polite" aria-atomic="true">
    {lastAnnouncement}
  </div>
</div>

<style>
  .progress-display {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .progress-display--compact {
    gap: 8px;
  }

  .progress-bar {
    height: 8px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-display--compact .progress-bar {
    height: 6px;
  }

  .progress-bar__fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .progress-bar__fill--indeterminate {
    width: 30% !important;
    animation: indeterminate 1.5s ease-in-out infinite;
  }

  .progress-bar__fill--paused {
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(400%); }
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  .progress-stats {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .progress-display--compact .progress-stats {
    gap: 12px;
  }

  .progress-stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .progress-stat--percentage {
    min-width: 50px;
  }

  .progress-stat__value {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .progress-display--compact .progress-stat__value {
    font-size: 14px;
  }

  .progress-stat__label {
    font-size: 11px;
    color: var(--color-text-muted);
    text-transform: uppercase;
  }

  .progress-status {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
    margin-left: auto;
  }

  .progress-status--paused {
    background: rgba(255, 152, 0, 0.1);
    color: var(--color-warning);
  }

  .progress-status--complete {
    background: rgba(76, 175, 80, 0.1);
    color: var(--color-success);
  }

  .progress-action {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--color-bg-secondary);
    border-radius: 6px;
  }

  .progress-action__icon {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .progress-action__text {
    font-size: 13px;
    color: var(--color-text-secondary);
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
```

### 3. Step Timeline Component (src/lib/components/mission/StepTimeline.svelte)

```svelte
<script lang="ts">
  import type { StepInfo } from '$lib/types/progress';
  import { formatDuration } from '$lib/types/progress';

  export let steps: StepInfo[];
  export let maxVisible = 5;

  $: visibleSteps = steps.slice(-maxVisible);
  $: hiddenCount = Math.max(0, steps.length - maxVisible);

  const statusIcons: Record<string, string> = {
    pending: '○',
    running: '◉',
    complete: '●',
    error: '✕',
    skipped: '⊘',
  };

  const statusColors: Record<string, string> = {
    pending: 'var(--color-text-muted)',
    running: 'var(--color-primary)',
    complete: 'var(--color-success)',
    error: 'var(--color-error)',
    skipped: 'var(--color-text-muted)',
  };
</script>

<div class="step-timeline">
  {#if hiddenCount > 0}
    <div class="step-timeline__hidden">
      +{hiddenCount} earlier steps
    </div>
  {/if}

  {#each visibleSteps as step, index}
    <div
      class="step-item"
      class:step-item--current={step.status === 'running'}
    >
      <span
        class="step-item__icon"
        style="color: {statusColors[step.status]}"
      >
        {statusIcons[step.status]}
      </span>

      <div class="step-item__content">
        <span class="step-item__name">{step.name}</span>
        {#if step.duration > 0}
          <span class="step-item__duration">{formatDuration(step.duration)}</span>
        {/if}
      </div>

      {#if index < visibleSteps.length - 1}
        <div class="step-item__connector"></div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .step-timeline {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }

  .step-timeline__hidden {
    font-size: 12px;
    color: var(--color-text-muted);
    padding-left: 24px;
    margin-bottom: 8px;
  }

  .step-item {
    display: flex;
    align-items: center;
    gap: 8px;
    position: relative;
    padding-left: 4px;
  }

  .step-item--current {
    background: var(--color-bg-active);
    margin: 0 -12px;
    padding: 8px 12px 8px 16px;
    border-radius: 4px;
  }

  .step-item__icon {
    width: 16px;
    text-align: center;
    font-size: 10px;
  }

  .step-item--current .step-item__icon {
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .step-item__content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .step-item__name {
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .step-item__duration {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .step-item__connector {
    position: absolute;
    left: 11px;
    top: 18px;
    width: 2px;
    height: 12px;
    background: var(--color-border);
  }
</style>
```

---

## Testing Requirements

1. Progress bar updates with correct percentage
2. Animation works smoothly
3. Time calculations are accurate
4. Indeterminate state displays correctly
5. Paused state shows visual indicator
6. Step timeline renders correctly
7. Screen reader announcements work

### Test File (src/lib/components/mission/__tests__/ProgressDisplay.test.ts)

```typescript
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import ProgressDisplay from '../ProgressDisplay.svelte';

describe('ProgressDisplay', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('renders progress percentage', () => {
    render(ProgressDisplay, {
      progress: {
        percentage: 45,
        currentStep: 3,
        totalSteps: 10,
        currentAction: 'Processing files',
        elapsedMs: 30000,
        estimatedRemainingMs: 40000,
        stepsCompleted: [],
        isPaused: false,
        isIndeterminate: false,
      },
    });

    expect(screen.getByText('45%')).toBeInTheDocument();
    expect(screen.getByText('3/10')).toBeInTheDocument();
  });

  it('shows current action', () => {
    render(ProgressDisplay, {
      progress: {
        percentage: 50,
        currentStep: 1,
        totalSteps: 2,
        currentAction: 'Analyzing code',
        elapsedMs: 0,
        estimatedRemainingMs: 0,
        stepsCompleted: [],
        isPaused: false,
        isIndeterminate: false,
      },
    });

    expect(screen.getByText('Analyzing code')).toBeInTheDocument();
  });

  it('shows paused status', () => {
    render(ProgressDisplay, {
      progress: {
        percentage: 30,
        currentStep: 1,
        totalSteps: 3,
        currentAction: '',
        elapsedMs: 15000,
        estimatedRemainingMs: 0,
        stepsCompleted: [],
        isPaused: true,
        isIndeterminate: false,
      },
    });

    expect(screen.getByText('Paused')).toBeInTheDocument();
  });

  it('shows complete status at 100%', () => {
    render(ProgressDisplay, {
      progress: {
        percentage: 100,
        currentStep: 5,
        totalSteps: 5,
        currentAction: '',
        elapsedMs: 60000,
        estimatedRemainingMs: 0,
        stepsCompleted: [],
        isPaused: false,
        isIndeterminate: false,
      },
    });

    expect(screen.getByText('Complete')).toBeInTheDocument();
  });

  it('has correct ARIA attributes', () => {
    render(ProgressDisplay, {
      progress: {
        percentage: 75,
        currentStep: 3,
        totalSteps: 4,
        currentAction: '',
        elapsedMs: 0,
        estimatedRemainingMs: 0,
        stepsCompleted: [],
        isPaused: false,
        isIndeterminate: false,
      },
    });

    const progressbar = screen.getByRole('progressbar');
    expect(progressbar).toHaveAttribute('aria-valuenow', '75');
    expect(progressbar).toHaveAttribute('aria-valuemin', '0');
    expect(progressbar).toHaveAttribute('aria-valuemax', '100');
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [225-log-viewer.md](225-log-viewer.md)
- Used by: [216-mission-layout.md](216-mission-layout.md)
