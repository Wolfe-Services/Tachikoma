# 223 - Mission Controls Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 223
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create mission control components that provide start, pause, resume, abort, and restart functionality for missions, with appropriate state-based button visibility and confirmation dialogs for destructive actions.

---

## Acceptance Criteria

- [x] Start button to begin mission execution
- [x] Pause/Resume toggle for running missions
- [x] Abort button with confirmation dialog
- [x] Restart option for completed/failed missions
- [x] State-appropriate button visibility
- [x] Keyboard shortcuts for controls
- [x] Loading states during transitions

---

## Implementation Details

### 1. Types (src/lib/types/mission-controls.ts)

```typescript
/**
 * Types for mission control functionality.
 */

export type ControlAction = 'start' | 'pause' | 'resume' | 'abort' | 'restart';

export interface ControlState {
  canStart: boolean;
  canPause: boolean;
  canResume: boolean;
  canAbort: boolean;
  canRestart: boolean;
}

export interface ControlConfig {
  confirmAbort: boolean;
  confirmRestart: boolean;
  showKeyboardHints: boolean;
}

export interface PendingAction {
  action: ControlAction;
  startedAt: number;
  timeoutMs: number;
}

export const CONTROL_SHORTCUTS: Record<ControlAction, string> = {
  start: 'Cmd+Enter',
  pause: 'Cmd+P',
  resume: 'Cmd+R',
  abort: 'Cmd+.',
  restart: 'Cmd+Shift+R',
};

export function getControlState(missionState: string): ControlState {
  return {
    canStart: missionState === 'idle',
    canPause: missionState === 'running',
    canResume: missionState === 'paused',
    canAbort: ['running', 'paused'].includes(missionState),
    canRestart: ['complete', 'error', 'redlined'].includes(missionState),
  };
}
```

### 2. Mission Controls Component (src/lib/components/mission/MissionControls.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { missionStore, selectedMission } from '$lib/stores/mission-store';
  import type { MissionState } from '$lib/types/mission';
  import type { ControlAction, ControlConfig } from '$lib/types/mission-controls';
  import { getControlState, CONTROL_SHORTCUTS } from '$lib/types/mission-controls';
  import ConfirmDialog from '$lib/components/common/ConfirmDialog.svelte';

  export let missionId: string | null = null;
  export let config: ControlConfig = {
    confirmAbort: true,
    confirmRestart: true,
    showKeyboardHints: true,
  };
  export let compact = false;

  const dispatch = createEventDispatcher<{
    action: { action: ControlAction; missionId: string };
  }>();

  let pendingAction: ControlAction | null = null;
  let showAbortConfirm = false;
  let showRestartConfirm = false;

  $: mission = missionId
    ? $missionStore.missions.get(missionId)
    : $selectedMission;

  $: controlState = mission
    ? getControlState(mission.state)
    : { canStart: false, canPause: false, canResume: false, canAbort: false, canRestart: false };

  async function handleAction(action: ControlAction) {
    if (!mission) return;

    // Check for confirmation dialogs
    if (action === 'abort' && config.confirmAbort) {
      showAbortConfirm = true;
      return;
    }

    if (action === 'restart' && config.confirmRestart) {
      showRestartConfirm = true;
      return;
    }

    await executeAction(action);
  }

  async function executeAction(action: ControlAction) {
    if (!mission) return;

    pendingAction = action;
    dispatch('action', { action, missionId: mission.id });

    try {
      switch (action) {
        case 'start':
          await missionStore.startMission(mission.id);
          break;
        case 'pause':
          await missionStore.pauseMission(mission.id);
          break;
        case 'resume':
          await missionStore.resumeMission(mission.id);
          break;
        case 'abort':
          await missionStore.abortMission(mission.id);
          break;
        case 'restart':
          // Restart creates a new mission with the same config
          await missionStore.createMission({
            title: mission.title,
            prompt: mission.prompt,
            specIds: mission.specIds,
            backendId: mission.backendId,
            mode: mission.mode,
            tags: mission.tags,
          });
          break;
      }
    } finally {
      pendingAction = null;
    }
  }

  function handleConfirmAbort() {
    showAbortConfirm = false;
    executeAction('abort');
  }

  function handleConfirmRestart() {
    showRestartConfirm = false;
    executeAction('restart');
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!mission) return;

    const isMod = event.metaKey || event.ctrlKey;

    // Start: Cmd+Enter
    if (isMod && event.key === 'Enter' && controlState.canStart) {
      event.preventDefault();
      handleAction('start');
    }

    // Pause: Cmd+P
    if (isMod && event.key === 'p' && controlState.canPause) {
      event.preventDefault();
      handleAction('pause');
    }

    // Resume: Cmd+R
    if (isMod && event.key === 'r' && !event.shiftKey && controlState.canResume) {
      event.preventDefault();
      handleAction('resume');
    }

    // Abort: Cmd+.
    if (isMod && event.key === '.' && controlState.canAbort) {
      event.preventDefault();
      handleAction('abort');
    }

    // Restart: Cmd+Shift+R
    if (isMod && event.shiftKey && event.key === 'r' && controlState.canRestart) {
      event.preventDefault();
      handleAction('restart');
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<div class="mission-controls" class:mission-controls--compact={compact}>
  {#if !mission}
    <p class="mission-controls__no-mission">No mission selected</p>
  {:else}
    <!-- Start Button -->
    {#if controlState.canStart}
      <button
        class="control-btn control-btn--start"
        disabled={pendingAction !== null}
        on:click={() => handleAction('start')}
      >
        {#if pendingAction === 'start'}
          <span class="control-btn__spinner"></span>
          Starting...
        {:else}
          <svg class="control-btn__icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4 2.5v11l9-5.5-9-5.5z"/>
          </svg>
          {#if !compact}Start Mission{/if}
          {#if config.showKeyboardHints && !compact}
            <kbd class="control-btn__shortcut">{CONTROL_SHORTCUTS.start}</kbd>
          {/if}
        {/if}
      </button>
    {/if}

    <!-- Pause Button -->
    {#if controlState.canPause}
      <button
        class="control-btn control-btn--pause"
        disabled={pendingAction !== null}
        on:click={() => handleAction('pause')}
      >
        {#if pendingAction === 'pause'}
          <span class="control-btn__spinner"></span>
          Pausing...
        {:else}
          <svg class="control-btn__icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M3 2h4v12H3V2zm6 0h4v12H9V2z"/>
          </svg>
          {#if !compact}Pause{/if}
          {#if config.showKeyboardHints && !compact}
            <kbd class="control-btn__shortcut">{CONTROL_SHORTCUTS.pause}</kbd>
          {/if}
        {/if}
      </button>
    {/if}

    <!-- Resume Button -->
    {#if controlState.canResume}
      <button
        class="control-btn control-btn--resume"
        disabled={pendingAction !== null}
        on:click={() => handleAction('resume')}
      >
        {#if pendingAction === 'resume'}
          <span class="control-btn__spinner"></span>
          Resuming...
        {:else}
          <svg class="control-btn__icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4 2.5v11l9-5.5-9-5.5z"/>
          </svg>
          {#if !compact}Resume{/if}
          {#if config.showKeyboardHints && !compact}
            <kbd class="control-btn__shortcut">{CONTROL_SHORTCUTS.resume}</kbd>
          {/if}
        {/if}
      </button>
    {/if}

    <!-- Abort Button -->
    {#if controlState.canAbort}
      <button
        class="control-btn control-btn--abort"
        disabled={pendingAction !== null}
        on:click={() => handleAction('abort')}
      >
        {#if pendingAction === 'abort'}
          <span class="control-btn__spinner"></span>
          Aborting...
        {:else}
          <svg class="control-btn__icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4.646 4.646a.5.5 0 01.708 0L8 7.293l2.646-2.647a.5.5 0 01.708.708L8.707 8l2.647 2.646a.5.5 0 01-.708.708L8 8.707l-2.646 2.647a.5.5 0 01-.708-.708L7.293 8 4.646 5.354a.5.5 0 010-.708z"/>
          </svg>
          {#if !compact}Abort{/if}
          {#if config.showKeyboardHints && !compact}
            <kbd class="control-btn__shortcut">{CONTROL_SHORTCUTS.abort}</kbd>
          {/if}
        {/if}
      </button>
    {/if}

    <!-- Restart Button -->
    {#if controlState.canRestart}
      <button
        class="control-btn control-btn--restart"
        disabled={pendingAction !== null}
        on:click={() => handleAction('restart')}
      >
        {#if pendingAction === 'restart'}
          <span class="control-btn__spinner"></span>
          Restarting...
        {:else}
          <svg class="control-btn__icon" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 3a5 5 0 104.546 2.914.5.5 0 01.908-.418A6 6 0 118 2v1z"/>
            <path d="M8 4.466V.534a.25.25 0 01.41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 018 4.466z"/>
          </svg>
          {#if !compact}Restart{/if}
          {#if config.showKeyboardHints && !compact}
            <kbd class="control-btn__shortcut">{CONTROL_SHORTCUTS.restart}</kbd>
          {/if}
        {/if}
      </button>
    {/if}
  {/if}
</div>

<!-- Abort Confirmation Dialog -->
{#if showAbortConfirm}
  <ConfirmDialog
    title="Abort Mission?"
    message="This will stop the current mission. Any unsaved progress may be lost. You can resume from the last checkpoint."
    confirmText="Abort Mission"
    confirmVariant="danger"
    on:confirm={handleConfirmAbort}
    on:cancel={() => { showAbortConfirm = false; }}
  />
{/if}

<!-- Restart Confirmation Dialog -->
{#if showRestartConfirm}
  <ConfirmDialog
    title="Restart Mission?"
    message="This will create a new mission with the same configuration. The current mission results will be preserved in history."
    confirmText="Restart Mission"
    confirmVariant="primary"
    on:confirm={handleConfirmRestart}
    on:cancel={() => { showRestartConfirm = false; }}
  />
{/if}

<style>
  .mission-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .mission-controls--compact {
    gap: 4px;
  }

  .mission-controls__no-mission {
    color: var(--color-text-muted);
    font-size: 14px;
    margin: 0;
  }

  .control-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mission-controls--compact .control-btn {
    padding: 8px 12px;
  }

  .control-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .control-btn--start,
  .control-btn--resume {
    background: var(--color-success);
    color: white;
  }

  .control-btn--start:hover:not(:disabled),
  .control-btn--resume:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .control-btn--pause {
    background: var(--color-warning);
    color: white;
  }

  .control-btn--pause:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .control-btn--abort {
    background: var(--color-error);
    color: white;
  }

  .control-btn--abort:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .control-btn--restart {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }

  .control-btn--restart:hover:not(:disabled) {
    background: var(--color-bg-active);
    border-color: var(--color-primary);
  }

  .control-btn__icon {
    flex-shrink: 0;
  }

  .control-btn__shortcut {
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    font-size: 11px;
    font-family: inherit;
    opacity: 0.8;
  }

  .control-btn--restart .control-btn__shortcut {
    background: var(--color-bg-hover);
  }

  .control-btn__spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

### 3. Confirm Dialog Component (src/lib/components/common/ConfirmDialog.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, fly } from 'svelte/transition';

  export let title: string;
  export let message: string;
  export let confirmText = 'Confirm';
  export let cancelText = 'Cancel';
  export let confirmVariant: 'primary' | 'danger' = 'primary';

  const dispatch = createEventDispatcher<{
    confirm: void;
    cancel: void;
  }>();

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      dispatch('cancel');
    } else if (event.key === 'Enter') {
      dispatch('confirm');
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="confirm-overlay" transition:fade={{ duration: 150 }} on:click={() => dispatch('cancel')}>
  <div
    class="confirm-dialog"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="confirm-title"
    aria-describedby="confirm-message"
    transition:fly={{ y: 10, duration: 200 }}
    on:click|stopPropagation
  >
    <h3 id="confirm-title" class="confirm-dialog__title">{title}</h3>
    <p id="confirm-message" class="confirm-dialog__message">{message}</p>

    <div class="confirm-dialog__actions">
      <button
        class="confirm-dialog__btn confirm-dialog__btn--cancel"
        on:click={() => dispatch('cancel')}
      >
        {cancelText}
      </button>
      <button
        class="confirm-dialog__btn confirm-dialog__btn--{confirmVariant}"
        on:click={() => dispatch('confirm')}
      >
        {confirmText}
      </button>
    </div>
  </div>
</div>

<style>
  .confirm-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
  }

  .confirm-dialog {
    background: var(--color-bg-primary);
    border-radius: 12px;
    padding: 24px;
    width: 90%;
    max-width: 400px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  }

  .confirm-dialog__title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
  }

  .confirm-dialog__message {
    font-size: 14px;
    color: var(--color-text-secondary);
    line-height: 1.5;
    margin: 0 0 24px 0;
  }

  .confirm-dialog__actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
  }

  .confirm-dialog__btn {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .confirm-dialog__btn--cancel {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .confirm-dialog__btn--cancel:hover {
    background: var(--color-bg-active);
  }

  .confirm-dialog__btn--primary {
    background: var(--color-primary);
    color: white;
  }

  .confirm-dialog__btn--primary:hover {
    filter: brightness(1.1);
  }

  .confirm-dialog__btn--danger {
    background: var(--color-error);
    color: white;
  }

  .confirm-dialog__btn--danger:hover {
    filter: brightness(1.1);
  }
</style>
```

---

## Testing Requirements

1. Correct buttons display for each mission state
2. Actions trigger correct store methods
3. Confirmation dialogs appear for abort/restart
4. Keyboard shortcuts work correctly
5. Loading states display during transitions
6. Disabled state prevents multiple clicks
7. Compact mode renders properly

### Test File (src/lib/components/mission/__tests__/MissionControls.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import MissionControls from '../MissionControls.svelte';
import { missionStore } from '$lib/stores/mission-store';

vi.mock('$lib/stores/mission-store', () => ({
  missionStore: {
    subscribe: vi.fn(),
    startMission: vi.fn().mockResolvedValue(true),
    pauseMission: vi.fn().mockResolvedValue(true),
    resumeMission: vi.fn().mockResolvedValue(true),
    abortMission: vi.fn().mockResolvedValue(true),
    createMission: vi.fn().mockResolvedValue({ id: 'new-1' }),
    missions: new Map([
      ['test-1', { id: 'test-1', title: 'Test', state: 'idle', prompt: '', specIds: [], backendId: 'claude', mode: 'agentic', tags: [] }],
    ]),
  },
  selectedMission: {
    subscribe: vi.fn(cb => {
      cb({ id: 'test-1', state: 'idle' });
      return () => {};
    }),
  },
}));

describe('MissionControls', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows start button for idle mission', () => {
    render(MissionControls, { missionId: 'test-1' });
    expect(screen.getByText('Start Mission')).toBeInTheDocument();
  });

  it('calls startMission on start click', async () => {
    render(MissionControls, { missionId: 'test-1' });

    await fireEvent.click(screen.getByText('Start Mission'));

    expect(missionStore.startMission).toHaveBeenCalledWith('test-1');
  });

  it('shows confirmation dialog before abort', async () => {
    // Update mock to show running mission
    vi.mocked(missionStore).missions = new Map([
      ['test-1', { id: 'test-1', state: 'running' }],
    ]);

    render(MissionControls, { missionId: 'test-1', config: { confirmAbort: true, confirmRestart: true, showKeyboardHints: true } });

    await fireEvent.click(screen.getByText('Abort'));

    expect(screen.getByText('Abort Mission?')).toBeInTheDocument();
  });

  it('aborts on confirmation', async () => {
    vi.mocked(missionStore).missions = new Map([
      ['test-1', { id: 'test-1', state: 'running' }],
    ]);

    render(MissionControls, { missionId: 'test-1' });

    await fireEvent.click(screen.getByText('Abort'));
    await fireEvent.click(screen.getByText('Abort Mission'));

    expect(missionStore.abortMission).toHaveBeenCalledWith('test-1');
  });

  it('responds to keyboard shortcuts', async () => {
    render(MissionControls, { missionId: 'test-1' });

    await fireEvent.keyDown(window, { key: 'Enter', metaKey: true });

    expect(missionStore.startMission).toHaveBeenCalled();
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [224-progress-display.md](224-progress-display.md)
- Used by: [216-mission-layout.md](216-mission-layout.md)
