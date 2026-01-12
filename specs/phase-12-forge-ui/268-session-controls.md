# Spec 268: Session Controls

## Header
- **Spec ID**: 268
- **Phase**: 12 - Forge UI
- **Component**: Session Controls
- **Dependencies**: Spec 256 (Forge Layout)
- **Status**: Draft

## Objective
Create a comprehensive control panel for managing active forge sessions, including start/stop controls, round management, timeout handling, and session state transitions.

## Acceptance Criteria
1. Provide clear start, pause, resume, and stop controls
2. Display session state with visual indicators
3. Show round progression controls (next round, skip, restart)
4. Handle timeout warnings and extensions
5. Support emergency stop with confirmation
6. Display session timing and duration
7. Enable round-level rollback capability
8. Provide session save/checkpoint controls

## Implementation

### SessionControls.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import TimeoutWarning from './TimeoutWarning.svelte';
  import RoundControls from './RoundControls.svelte';
  import SessionTimer from './SessionTimer.svelte';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import type { SessionState, SessionAction } from '$lib/types/forge';

  export let sessionId: string;

  const dispatch = createEventDispatcher<{
    action: { type: SessionAction; payload?: unknown };
    checkpoint: void;
    rollback: { roundNumber: number };
  }>();

  let showConfirmDialog = writable<SessionAction | null>(null);
  let showAdvancedControls = writable<boolean>(false);
  let timeoutWarningDismissed = writable<boolean>(false);

  const session = derived(forgeSessionStore, ($store) => $store.activeSession);

  const sessionState = derived(session, ($session) => $session?.state || 'idle');

  const currentRound = derived(session, ($session) => $session?.currentRound || 0);

  const maxRounds = derived(session, ($session) => $session?.config.maxRounds || 5);

  const timeRemaining = derived(session, ($session) => {
    if (!$session?.startedAt || !$session?.config.timeoutMinutes) return null;

    const elapsed = Date.now() - new Date($session.startedAt).getTime();
    const timeout = $session.config.timeoutMinutes * 60 * 1000;
    return Math.max(0, timeout - elapsed);
  });

  const isTimeoutWarning = derived(timeRemaining, ($remaining) => {
    if ($remaining === null) return false;
    return $remaining < 5 * 60 * 1000; // Less than 5 minutes
  });

  const canStart = derived(sessionState, ($state) =>
    $state === 'idle' || $state === 'configured'
  );

  const canPause = derived(sessionState, ($state) =>
    $state === 'running' || $state === 'deliberating'
  );

  const canResume = derived(sessionState, ($state) =>
    $state === 'paused'
  );

  const canStop = derived(sessionState, ($state) =>
    $state !== 'idle' && $state !== 'completed' && $state !== 'stopped'
  );

  const canAdvanceRound = derived(
    [sessionState, currentRound, maxRounds],
    ([$state, $round, $max]) =>
      ($state === 'running' || $state === 'deliberating') && $round < $max
  );

  function requestAction(action: SessionAction) {
    const confirmActions: SessionAction[] = ['stop', 'rollback', 'restart'];

    if (confirmActions.includes(action)) {
      showConfirmDialog.set(action);
    } else {
      executeAction(action);
    }
  }

  async function executeAction(action: SessionAction, payload?: unknown) {
    try {
      switch (action) {
        case 'start':
          await forgeSessionStore.startSession(sessionId);
          break;
        case 'pause':
          await forgeSessionStore.pauseSession(sessionId);
          break;
        case 'resume':
          await forgeSessionStore.resumeSession(sessionId);
          break;
        case 'stop':
          await forgeSessionStore.stopSession(sessionId);
          break;
        case 'nextRound':
          await forgeSessionStore.advanceRound(sessionId);
          break;
        case 'rollback':
          await forgeSessionStore.rollbackToRound(sessionId, payload as number);
          dispatch('rollback', { roundNumber: payload as number });
          break;
        case 'extendTimeout':
          await forgeSessionStore.extendTimeout(sessionId, payload as number);
          break;
        case 'checkpoint':
          await forgeSessionStore.createCheckpoint(sessionId);
          dispatch('checkpoint');
          break;
      }

      dispatch('action', { type: action, payload });
    } catch (error) {
      console.error(`Failed to execute action ${action}:`, error);
    }

    showConfirmDialog.set(null);
  }

  function handleConfirm() {
    if ($showConfirmDialog) {
      executeAction($showConfirmDialog);
    }
  }

  function handleCancel() {
    showConfirmDialog.set(null);
  }

  function getStateColor(state: SessionState): string {
    switch (state) {
      case 'running':
      case 'deliberating':
        return 'var(--success-color)';
      case 'paused':
        return 'var(--warning-color)';
      case 'completed':
        return 'var(--info-color)';
      case 'stopped':
      case 'error':
        return 'var(--error-color)';
      default:
        return 'var(--text-muted)';
    }
  }

  function getStateLabel(state: SessionState): string {
    switch (state) {
      case 'idle': return 'Ready';
      case 'configured': return 'Configured';
      case 'running': return 'Running';
      case 'deliberating': return 'Deliberating';
      case 'paused': return 'Paused';
      case 'completed': return 'Completed';
      case 'stopped': return 'Stopped';
      case 'error': return 'Error';
      default: return state;
    }
  }

  function getConfirmMessage(action: SessionAction): string {
    switch (action) {
      case 'stop':
        return 'Are you sure you want to stop this session? This cannot be undone.';
      case 'rollback':
        return 'Rolling back will discard all progress after the selected round. Continue?';
      case 'restart':
        return 'Restarting will clear all session progress. Are you sure?';
      default:
        return `Confirm ${action}?`;
    }
  }
</script>

<div class="session-controls" data-testid="session-controls">
  <div class="controls-header">
    <div class="state-indicator">
      <span
        class="state-dot"
        style="background-color: {getStateColor($sessionState)}"
        class:pulsing={$sessionState === 'running' || $sessionState === 'deliberating'}
      ></span>
      <span class="state-label">{getStateLabel($sessionState)}</span>
    </div>

    <SessionTimer
      startedAt={$session?.startedAt}
      pausedAt={$session?.pausedAt}
      timeRemaining={$timeRemaining}
    />
  </div>

  {#if $isTimeoutWarning && !$timeoutWarningDismissed}
    <TimeoutWarning
      timeRemaining={$timeRemaining}
      on:extend={(e) => executeAction('extendTimeout', e.detail)}
      on:dismiss={() => timeoutWarningDismissed.set(true)}
    />
  {/if}

  <div class="main-controls">
    {#if $canStart}
      <button
        class="control-btn primary large"
        on:click={() => requestAction('start')}
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z"/>
        </svg>
        Start Session
      </button>
    {:else if $canPause}
      <button
        class="control-btn warning"
        on:click={() => requestAction('pause')}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="4" width="4" height="16"/>
          <rect x="14" y="4" width="4" height="16"/>
        </svg>
        Pause
      </button>
    {:else if $canResume}
      <button
        class="control-btn success"
        on:click={() => requestAction('resume')}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z"/>
        </svg>
        Resume
      </button>
    {/if}

    {#if $canStop}
      <button
        class="control-btn danger"
        on:click={() => requestAction('stop')}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="6" width="12" height="12"/>
        </svg>
        Stop
      </button>
    {/if}
  </div>

  {#if $sessionState !== 'idle' && $sessionState !== 'completed'}
    <div class="round-progress">
      <div class="progress-label">
        <span>Round {$currentRound} of {$maxRounds}</span>
        <span class="progress-percent">
          {(($currentRound / $maxRounds) * 100).toFixed(0)}%
        </span>
      </div>
      <div class="progress-bar">
        <div
          class="progress-fill"
          style="width: {($currentRound / $maxRounds) * 100}%"
        ></div>
        {#each Array($maxRounds) as _, i}
          <div
            class="round-marker"
            class:completed={i < $currentRound}
            class:current={i === $currentRound - 1}
            style="left: {((i + 1) / $maxRounds) * 100}%"
          ></div>
        {/each}
      </div>
    </div>
  {/if}

  <div class="secondary-controls">
    {#if $canAdvanceRound}
      <button
        class="control-btn secondary"
        on:click={() => requestAction('nextRound')}
      >
        Next Round
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M9 5l7 7-7 7" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    {/if}

    <button
      class="control-btn secondary"
      on:click={() => requestAction('checkpoint')}
      disabled={$sessionState === 'idle'}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z" stroke-width="2"/>
        <path d="M17 21v-8H7v8M7 3v5h8" stroke-width="2"/>
      </svg>
      Save Checkpoint
    </button>

    <button
      class="control-btn ghost"
      on:click={() => showAdvancedControls.update(v => !v)}
    >
      {$showAdvancedControls ? 'Hide' : 'Show'} Advanced
    </button>
  </div>

  {#if $showAdvancedControls}
    <div class="advanced-controls" transition:slide>
      <h4>Advanced Controls</h4>

      <RoundControls
        currentRound={$currentRound}
        maxRounds={$maxRounds}
        sessionState={$sessionState}
        on:rollback={(e) => requestAction('rollback')}
        on:skip={(e) => executeAction('nextRound')}
      />

      <div class="timeout-controls">
        <h5>Timeout Management</h5>
        <div class="timeout-buttons">
          <button
            class="control-btn small"
            on:click={() => executeAction('extendTimeout', 10)}
          >
            +10 min
          </button>
          <button
            class="control-btn small"
            on:click={() => executeAction('extendTimeout', 30)}
          >
            +30 min
          </button>
          <button
            class="control-btn small"
            on:click={() => executeAction('extendTimeout', 60)}
          >
            +1 hour
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

{#if $showConfirmDialog}
  <ConfirmDialog
    title="Confirm Action"
    message={getConfirmMessage($showConfirmDialog)}
    confirmText="Confirm"
    cancelText="Cancel"
    variant="warning"
    on:confirm={handleConfirm}
    on:cancel={handleCancel}
  />
{/if}

<style>
  .session-controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.25rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
  }

  .controls-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .state-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .state-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .state-dot.pulsing {
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 currentColor;
    }
    70% {
      box-shadow: 0 0 0 8px transparent;
    }
    100% {
      box-shadow: 0 0 0 0 transparent;
    }
  }

  .state-label {
    font-weight: 500;
    font-size: 0.9375rem;
  }

  .main-controls {
    display: flex;
    gap: 0.75rem;
  }

  .control-btn {
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

  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .control-btn.large {
    padding: 0.875rem 1.5rem;
    font-size: 1rem;
    flex: 1;
  }

  .control-btn.small {
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
  }

  .control-btn.primary {
    background: var(--primary-color);
    color: white;
  }

  .control-btn.primary:hover:not(:disabled) {
    background: var(--primary-hover);
  }

  .control-btn.success {
    background: var(--success-color);
    color: white;
  }

  .control-btn.warning {
    background: var(--warning-color);
    color: white;
  }

  .control-btn.danger {
    background: var(--error-color);
    color: white;
  }

  .control-btn.secondary {
    background: var(--secondary-bg);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .control-btn.secondary:hover:not(:disabled) {
    background: var(--hover-bg);
  }

  .control-btn.ghost {
    background: transparent;
    color: var(--text-secondary);
    border: none;
  }

  .control-btn.ghost:hover {
    color: var(--text-primary);
  }

  .round-progress {
    padding: 0.75rem 0;
  }

  .progress-label {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .progress-bar {
    position: relative;
    height: 8px;
    background: var(--border-color);
    border-radius: 4px;
    overflow: visible;
  }

  .progress-fill {
    height: 100%;
    background: var(--primary-color);
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .round-marker {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--border-color);
    border: 2px solid var(--card-bg);
  }

  .round-marker.completed {
    background: var(--success-color);
  }

  .round-marker.current {
    background: var(--primary-color);
    box-shadow: 0 0 0 3px var(--primary-alpha);
  }

  .secondary-controls {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .advanced-controls {
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .advanced-controls h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .timeout-controls {
    margin-top: 1rem;
  }

  .timeout-controls h5 {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
  }

  .timeout-buttons {
    display: flex;
    gap: 0.5rem;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test state transitions and action validation
2. **Integration Tests**: Verify session control commands execute correctly
3. **Timeout Tests**: Test warning display and extension functionality
4. **Rollback Tests**: Verify rollback preserves data integrity
5. **UI Tests**: Test button states match session state

## Related Specs
- Spec 256: Forge Layout
- Spec 269: Pause/Resume
- Spec 270: Human Intervention
