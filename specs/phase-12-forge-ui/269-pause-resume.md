# Spec 269: Pause/Resume

## Header
- **Spec ID**: 269
- **Phase**: 12 - Forge UI
- **Component**: Pause/Resume
- **Dependencies**: Spec 268 (Session Controls)
- **Status**: Draft

## Objective
Implement robust pause and resume functionality for forge sessions, ensuring graceful handling of in-progress operations, state preservation, and seamless continuation of deliberation.

## Acceptance Criteria
- [x] Pause gracefully completes current atomic operations before stopping
- [x] Resume restores exact session state including partial progress
- [x] Display clear status during pause transition
- [x] Track pause duration and history
- [x] Handle auto-pause on timeout warnings
- [x] Support pause scheduling for planned interruptions
- [x] Preserve all participant context during pause
- [x] Notify participants of pause/resume events

## Implementation

### PauseResumeController.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, scale } from 'svelte/transition';
  import PauseOverlay from './PauseOverlay.svelte';
  import PauseScheduler from './PauseScheduler.svelte';
  import PauseHistory from './PauseHistory.svelte';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import { notificationService } from '$lib/services/notifications';
  import type { PauseState, PauseReason, ScheduledPause } from '$lib/types/forge';

  export let sessionId: string;

  const dispatch = createEventDispatcher<{
    paused: { reason: PauseReason; timestamp: Date };
    resumed: { pauseDuration: number };
    scheduled: ScheduledPause;
  }>();

  let pauseState = writable<PauseState>({
    isPaused: false,
    isPausing: false,
    isResuming: false,
    pausedAt: null,
    pauseReason: null,
    scheduledPauses: []
  });

  let showScheduler = writable<boolean>(false);
  let showHistory = writable<boolean>(false);
  let pauseHistory = writable<Array<{
    pausedAt: Date;
    resumedAt: Date | null;
    reason: PauseReason;
    duration: number | null;
  }>>([]);

  const session = derived(forgeSessionStore, ($store) => $store.activeSession);

  const pauseDuration = derived(pauseState, ($state) => {
    if (!$state.isPaused || !$state.pausedAt) return 0;
    return Date.now() - new Date($state.pausedAt).getTime();
  });

  const totalPauseDuration = derived(pauseHistory, ($history) => {
    return $history.reduce((total, pause) => total + (pause.duration || 0), 0);
  });

  const nextScheduledPause = derived(pauseState, ($state) => {
    const now = Date.now();
    return $state.scheduledPauses
      .filter(p => new Date(p.scheduledAt).getTime() > now)
      .sort((a, b) => new Date(a.scheduledAt).getTime() - new Date(b.scheduledAt).getTime())[0];
  });

  async function pause(reason: PauseReason = 'manual') {
    if ($pauseState.isPaused || $pauseState.isPausing) return;

    pauseState.update(s => ({ ...s, isPausing: true }));

    try {
      // Wait for current operations to complete gracefully
      await waitForSafePoint();

      // Execute pause
      await forgeSessionStore.pauseSession(sessionId);

      const pausedAt = new Date();

      pauseState.update(s => ({
        ...s,
        isPaused: true,
        isPausing: false,
        pausedAt,
        pauseReason: reason
      }));

      // Add to history
      pauseHistory.update(h => [
        ...h,
        { pausedAt, resumedAt: null, reason, duration: null }
      ]);

      // Notify participants
      notificationService.broadcast({
        type: 'session_paused',
        sessionId,
        reason,
        message: getPauseMessage(reason)
      });

      dispatch('paused', { reason, timestamp: pausedAt });
    } catch (error) {
      console.error('Failed to pause session:', error);
      pauseState.update(s => ({ ...s, isPausing: false }));
    }
  }

  async function resume() {
    if (!$pauseState.isPaused || $pauseState.isResuming) return;

    pauseState.update(s => ({ ...s, isResuming: true }));

    try {
      // Restore session state
      await forgeSessionStore.resumeSession(sessionId);

      const resumedAt = new Date();
      const duration = $pauseState.pausedAt
        ? resumedAt.getTime() - new Date($pauseState.pausedAt).getTime()
        : 0;

      // Update history
      pauseHistory.update(h => {
        const updated = [...h];
        const lastPause = updated[updated.length - 1];
        if (lastPause && !lastPause.resumedAt) {
          lastPause.resumedAt = resumedAt;
          lastPause.duration = duration;
        }
        return updated;
      });

      pauseState.update(s => ({
        ...s,
        isPaused: false,
        isResuming: false,
        pausedAt: null,
        pauseReason: null
      }));

      // Notify participants
      notificationService.broadcast({
        type: 'session_resumed',
        sessionId,
        message: 'Session has resumed'
      });

      dispatch('resumed', { pauseDuration: duration });
    } catch (error) {
      console.error('Failed to resume session:', error);
      pauseState.update(s => ({ ...s, isResuming: false }));
    }
  }

  async function waitForSafePoint(): Promise<void> {
    // Poll until session reaches a safe pause point
    const maxWait = 30000; // 30 seconds
    const pollInterval = 500;
    const startTime = Date.now();

    return new Promise((resolve, reject) => {
      const checkSafePoint = async () => {
        const status = await forgeSessionStore.getOperationStatus(sessionId);

        if (status.canPause) {
          resolve();
          return;
        }

        if (Date.now() - startTime > maxWait) {
          reject(new Error('Timeout waiting for safe pause point'));
          return;
        }

        setTimeout(checkSafePoint, pollInterval);
      };

      checkSafePoint();
    });
  }

  function schedulePause(schedule: ScheduledPause) {
    pauseState.update(s => ({
      ...s,
      scheduledPauses: [...s.scheduledPauses, schedule]
    }));

    // Set up timer
    const delay = new Date(schedule.scheduledAt).getTime() - Date.now();
    if (delay > 0) {
      setTimeout(() => {
        if (!$pauseState.isPaused) {
          pause(schedule.reason || 'scheduled');
        }
      }, delay);
    }

    dispatch('scheduled', schedule);
    showScheduler.set(false);
  }

  function cancelScheduledPause(scheduleId: string) {
    pauseState.update(s => ({
      ...s,
      scheduledPauses: s.scheduledPauses.filter(p => p.id !== scheduleId)
    }));
  }

  function getPauseMessage(reason: PauseReason): string {
    switch (reason) {
      case 'manual': return 'Session paused by operator';
      case 'scheduled': return 'Scheduled pause activated';
      case 'timeout_warning': return 'Session paused due to timeout warning';
      case 'human_intervention': return 'Session paused for human review';
      case 'error_recovery': return 'Session paused for error recovery';
      default: return 'Session paused';
    }
  }

  function formatDuration(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (hours > 0) {
      return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    }
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    }
    return `${seconds}s`;
  }

  // Check for scheduled pauses periodically
  let scheduleCheckInterval: number;

  onMount(() => {
    // Load pause history
    loadPauseHistory();

    // Check scheduled pauses every minute
    scheduleCheckInterval = setInterval(checkScheduledPauses, 60000);
  });

  onDestroy(() => {
    if (scheduleCheckInterval) {
      clearInterval(scheduleCheckInterval);
    }
  });

  async function loadPauseHistory() {
    const history = await forgeSessionStore.getPauseHistory(sessionId);
    pauseHistory.set(history);
  }

  function checkScheduledPauses() {
    const now = Date.now();

    for (const schedule of $pauseState.scheduledPauses) {
      const scheduleTime = new Date(schedule.scheduledAt).getTime();

      if (scheduleTime <= now && !$pauseState.isPaused) {
        pause(schedule.reason || 'scheduled');
        break;
      }
    }
  }
</script>

<div class="pause-resume-controller" data-testid="pause-resume-controller">
  {#if $pauseState.isPaused}
    <PauseOverlay
      pausedAt={$pauseState.pausedAt}
      reason={$pauseState.pauseReason}
      duration={$pauseDuration}
      isResuming={$pauseState.isResuming}
      on:resume={resume}
    />
  {/if}

  <div class="control-panel">
    <div class="status-section">
      {#if $pauseState.isPausing}
        <div class="status pausing" transition:fade>
          <span class="status-icon">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="spinning">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
            </svg>
          </span>
          <span>Pausing... waiting for safe point</span>
        </div>
      {:else if $pauseState.isPaused}
        <div class="status paused" transition:fade>
          <span class="status-icon">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <rect x="6" y="4" width="4" height="16"/>
              <rect x="14" y="4" width="4" height="16"/>
            </svg>
          </span>
          <span>Paused for {formatDuration($pauseDuration)}</span>
        </div>
      {:else if $nextScheduledPause}
        <div class="status scheduled" transition:fade>
          <span class="status-icon">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <circle cx="12" cy="12" r="10" stroke-width="2"/>
              <path d="M12 6v6l4 2" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </span>
          <span>
            Pause scheduled: {new Date($nextScheduledPause.scheduledAt).toLocaleTimeString()}
          </span>
          <button
            class="cancel-schedule"
            on:click={() => cancelScheduledPause($nextScheduledPause.id)}
          >
            Cancel
          </button>
        </div>
      {/if}
    </div>

    <div class="button-section">
      {#if $pauseState.isPaused}
        <button
          class="btn resume"
          on:click={resume}
          disabled={$pauseState.isResuming}
        >
          {#if $pauseState.isResuming}
            Resuming...
          {:else}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z"/>
            </svg>
            Resume Session
          {/if}
        </button>
      {:else}
        <button
          class="btn pause"
          on:click={() => pause('manual')}
          disabled={$pauseState.isPausing}
        >
          {#if $pauseState.isPausing}
            Pausing...
          {:else}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <rect x="6" y="4" width="4" height="16"/>
              <rect x="14" y="4" width="4" height="16"/>
            </svg>
            Pause Session
          {/if}
        </button>
      {/if}

      <button
        class="btn secondary"
        on:click={() => showScheduler.set(true)}
        disabled={$pauseState.isPaused}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <circle cx="12" cy="12" r="10" stroke-width="2"/>
          <path d="M12 6v6l4 2" stroke-width="2" stroke-linecap="round"/>
        </svg>
        Schedule Pause
      </button>

      <button
        class="btn ghost"
        on:click={() => showHistory.set(true)}
      >
        History ({$pauseHistory.length})
      </button>
    </div>

    {#if $totalPauseDuration > 0}
      <div class="pause-summary">
        <span class="summary-label">Total pause time:</span>
        <span class="summary-value">{formatDuration($totalPauseDuration)}</span>
      </div>
    {/if}
  </div>

  {#if $showScheduler}
    <PauseScheduler
      on:schedule={(e) => schedulePause(e.detail)}
      on:close={() => showScheduler.set(false)}
    />
  {/if}

  {#if $showHistory}
    <PauseHistory
      history={$pauseHistory}
      on:close={() => showHistory.set(false)}
    />
  {/if}
</div>

<style>
  .pause-resume-controller {
    position: relative;
  }

  .control-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
  }

  .status-section {
    min-height: 32px;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .status.pausing {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .status.paused {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .status.scheduled {
    background: var(--info-alpha);
    color: var(--info-color);
  }

  .status-icon {
    display: flex;
    align-items: center;
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .cancel-schedule {
    margin-left: auto;
    padding: 0.25rem 0.5rem;
    background: transparent;
    border: 1px solid currentColor;
    border-radius: 3px;
    color: inherit;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .button-section {
    display: flex;
    gap: 0.75rem;
  }

  .btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.pause {
    background: var(--warning-color);
    color: white;
  }

  .btn.resume {
    background: var(--success-color);
    color: white;
  }

  .btn.secondary {
    background: var(--secondary-bg);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .btn.ghost {
    background: transparent;
    color: var(--text-secondary);
  }

  .pause-summary {
    display: flex;
    gap: 0.5rem;
    font-size: 0.8125rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border-color);
  }

  .summary-label {
    color: var(--text-muted);
  }

  .summary-value {
    color: var(--text-secondary);
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test pause/resume state transitions
2. **Integration Tests**: Verify graceful operation completion before pause
3. **Scheduling Tests**: Test scheduled pause execution
4. **Duration Tests**: Verify pause duration tracking accuracy
5. **Recovery Tests**: Test resume with various session states

## Related Specs
- Spec 268: Session Controls
- Spec 270: Human Intervention
- Spec 275: Forge UI Tests
