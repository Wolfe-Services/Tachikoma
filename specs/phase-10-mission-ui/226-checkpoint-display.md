# 226 - Checkpoint Display Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 226
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~11% of Sonnet window

---

## Objective

Create a checkpoint display component that shows mission checkpoints/snapshots, allowing users to view, compare, and restore to previous states during or after mission execution.

---

## Acceptance Criteria

- [ ] Timeline view of checkpoints
- [ ] Checkpoint details with file changes
- [ ] Restore to checkpoint functionality
- [ ] Compare checkpoints side-by-side
- [ ] Create manual checkpoints
- [ ] Delete old checkpoints
- [ ] Export checkpoint data

---

## Implementation Details

### 1. Types (src/lib/types/checkpoint.ts)

```typescript
/**
 * Types for checkpoint functionality.
 */

export interface Checkpoint {
  id: string;
  missionId: string;
  name: string;
  description: string;
  createdAt: string;
  type: CheckpointType;
  trigger: CheckpointTrigger;
  snapshotPath: string;
  filesModified: FileChange[];
  metadata: CheckpointMetadata;
}

export type CheckpointType = 'auto' | 'manual' | 'error' | 'milestone';

export type CheckpointTrigger =
  | 'step_complete'
  | 'file_change'
  | 'user_request'
  | 'error_recovery'
  | 'milestone_reached';

export interface FileChange {
  path: string;
  type: 'created' | 'modified' | 'deleted';
  linesAdded: number;
  linesRemoved: number;
  sizeBefore: number;
  sizeAfter: number;
}

export interface CheckpointMetadata {
  stepNumber: number;
  progress: number;
  contextUsage: number;
  cost: number;
  duration: number;
}

export interface CheckpointComparison {
  fromCheckpoint: string;
  toCheckpoint: string;
  files: FileComparison[];
  summary: ComparisonSummary;
}

export interface FileComparison {
  path: string;
  status: 'added' | 'modified' | 'deleted' | 'unchanged';
  hunks: DiffHunk[];
}

export interface DiffHunk {
  startLine: number;
  endLine: number;
  content: string;
  type: 'add' | 'remove' | 'context';
}

export interface ComparisonSummary {
  filesAdded: number;
  filesModified: number;
  filesDeleted: number;
  totalLinesAdded: number;
  totalLinesRemoved: number;
}
```

### 2. Checkpoint Store (src/lib/stores/checkpoint-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { Checkpoint, CheckpointComparison } from '$lib/types/checkpoint';
import { ipcRenderer } from '$lib/ipc';

interface CheckpointStoreState {
  checkpoints: Map<string, Checkpoint>;
  missionId: string | null;
  loading: boolean;
  error: string | null;
  selectedId: string | null;
  comparisonIds: [string, string] | null;
  comparison: CheckpointComparison | null;
}

function createCheckpointStore() {
  const initialState: CheckpointStoreState = {
    checkpoints: new Map(),
    missionId: null,
    loading: false,
    error: null,
    selectedId: null,
    comparisonIds: null,
    comparison: null,
  };

  const { subscribe, set, update } = writable<CheckpointStoreState>(initialState);

  // Listen for new checkpoints
  ipcRenderer.on('mission:checkpoint', (_event, checkpoint: Checkpoint) => {
    update(state => {
      if (state.missionId === checkpoint.missionId) {
        const checkpoints = new Map(state.checkpoints);
        checkpoints.set(checkpoint.id, checkpoint);
        return { ...state, checkpoints };
      }
      return state;
    });
  });

  return {
    subscribe,

    async loadCheckpoints(missionId: string): Promise<void> {
      update(s => ({ ...s, loading: true, error: null, missionId }));

      try {
        const checkpoints: Checkpoint[] = await ipcRenderer.invoke('checkpoint:list', missionId);
        update(s => ({
          ...s,
          checkpoints: new Map(checkpoints.map(c => [c.id, c])),
          loading: false,
        }));
      } catch (error) {
        update(s => ({
          ...s,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to load checkpoints',
        }));
      }
    },

    async createCheckpoint(name: string, description: string): Promise<Checkpoint | null> {
      const state = await new Promise<CheckpointStoreState>(resolve => {
        subscribe(s => resolve(s))();
      });

      if (!state.missionId) return null;

      try {
        const checkpoint = await ipcRenderer.invoke('checkpoint:create', {
          missionId: state.missionId,
          name,
          description,
        });

        update(s => {
          const checkpoints = new Map(s.checkpoints);
          checkpoints.set(checkpoint.id, checkpoint);
          return { ...s, checkpoints };
        });

        return checkpoint;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to create checkpoint',
        }));
        return null;
      }
    },

    async restoreCheckpoint(checkpointId: string): Promise<boolean> {
      try {
        await ipcRenderer.invoke('checkpoint:restore', checkpointId);
        return true;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to restore checkpoint',
        }));
        return false;
      }
    },

    async deleteCheckpoint(checkpointId: string): Promise<boolean> {
      try {
        await ipcRenderer.invoke('checkpoint:delete', checkpointId);
        update(s => {
          const checkpoints = new Map(s.checkpoints);
          checkpoints.delete(checkpointId);
          return {
            ...s,
            checkpoints,
            selectedId: s.selectedId === checkpointId ? null : s.selectedId,
          };
        });
        return true;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to delete checkpoint',
        }));
        return false;
      }
    },

    async compareCheckpoints(fromId: string, toId: string): Promise<void> {
      update(s => ({ ...s, comparisonIds: [fromId, toId], comparison: null }));

      try {
        const comparison = await ipcRenderer.invoke('checkpoint:compare', { fromId, toId });
        update(s => ({ ...s, comparison }));
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to compare checkpoints',
        }));
      }
    },

    selectCheckpoint(checkpointId: string | null) {
      update(s => ({ ...s, selectedId: checkpointId }));
    },

    clearComparison() {
      update(s => ({ ...s, comparisonIds: null, comparison: null }));
    },

    reset() {
      set(initialState);
    },
  };
}

export const checkpointStore = createCheckpointStore();

export const checkpointList = derived(checkpointStore, $state =>
  Array.from($state.checkpoints.values()).sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
  )
);

export const selectedCheckpoint = derived(checkpointStore, $state =>
  $state.selectedId ? $state.checkpoints.get($state.selectedId) || null : null
);
```

### 3. Checkpoint Display Component (src/lib/components/mission/CheckpointDisplay.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { checkpointStore, checkpointList, selectedCheckpoint } from '$lib/stores/checkpoint-store';
  import type { Checkpoint } from '$lib/types/checkpoint';
  import CheckpointCard from './CheckpointCard.svelte';
  import CheckpointDetails from './CheckpointDetails.svelte';
  import CheckpointComparison from './CheckpointComparison.svelte';
  import CreateCheckpointDialog from './CreateCheckpointDialog.svelte';
  import ConfirmDialog from '$lib/components/common/ConfirmDialog.svelte';

  export let missionId: string;

  let showCreateDialog = false;
  let showRestoreConfirm = false;
  let showDeleteConfirm = false;
  let checkpointToRestore: string | null = null;
  let checkpointToDelete: string | null = null;
  let compareMode = false;
  let compareFrom: string | null = null;

  const typeIcons: Record<string, string> = {
    auto: '⏱',
    manual: '📌',
    error: '⚠️',
    milestone: '🏁',
  };

  function handleSelect(checkpoint: Checkpoint) {
    if (compareMode) {
      if (!compareFrom) {
        compareFrom = checkpoint.id;
      } else if (compareFrom !== checkpoint.id) {
        checkpointStore.compareCheckpoints(compareFrom, checkpoint.id);
        compareMode = false;
        compareFrom = null;
      }
    } else {
      checkpointStore.selectCheckpoint(
        $checkpointStore.selectedId === checkpoint.id ? null : checkpoint.id
      );
    }
  }

  function handleRestore(checkpointId: string) {
    checkpointToRestore = checkpointId;
    showRestoreConfirm = true;
  }

  function handleDelete(checkpointId: string) {
    checkpointToDelete = checkpointId;
    showDeleteConfirm = true;
  }

  async function confirmRestore() {
    if (checkpointToRestore) {
      await checkpointStore.restoreCheckpoint(checkpointToRestore);
      checkpointToRestore = null;
    }
    showRestoreConfirm = false;
  }

  async function confirmDelete() {
    if (checkpointToDelete) {
      await checkpointStore.deleteCheckpoint(checkpointToDelete);
      checkpointToDelete = null;
    }
    showDeleteConfirm = false;
  }

  function startCompare() {
    compareMode = true;
    compareFrom = null;
    checkpointStore.clearComparison();
  }

  function cancelCompare() {
    compareMode = false;
    compareFrom = null;
    checkpointStore.clearComparison();
  }

  onMount(() => {
    checkpointStore.loadCheckpoints(missionId);
  });
</script>

<div class="checkpoint-display">
  <!-- Header -->
  <div class="checkpoint-display__header">
    <h3 class="checkpoint-display__title">Checkpoints</h3>

    <div class="checkpoint-display__actions">
      {#if compareMode}
        <span class="compare-hint">
          {compareFrom ? 'Select second checkpoint' : 'Select first checkpoint'}
        </span>
        <button class="action-btn" on:click={cancelCompare}>Cancel</button>
      {:else}
        <button
          class="action-btn"
          on:click={startCompare}
          disabled={$checkpointList.length < 2}
        >
          Compare
        </button>
        <button
          class="action-btn action-btn--primary"
          on:click={() => { showCreateDialog = true; }}
        >
          Create Checkpoint
        </button>
      {/if}
    </div>
  </div>

  <!-- Loading/Error States -->
  {#if $checkpointStore.loading}
    <div class="checkpoint-display__loading">Loading checkpoints...</div>
  {:else if $checkpointStore.error}
    <div class="checkpoint-display__error">{$checkpointStore.error}</div>
  {:else if $checkpointList.length === 0}
    <div class="checkpoint-display__empty">
      <p>No checkpoints yet.</p>
      <p>Checkpoints are created automatically during mission execution, or you can create one manually.</p>
    </div>
  {:else}
    <!-- Timeline -->
    <div class="checkpoint-timeline">
      {#each $checkpointList as checkpoint, index}
        <div class="checkpoint-timeline__item">
          <div class="checkpoint-timeline__connector">
            <span
              class="checkpoint-timeline__dot"
              class:checkpoint-timeline__dot--selected={$checkpointStore.selectedId === checkpoint.id || compareFrom === checkpoint.id}
            >
              {typeIcons[checkpoint.type]}
            </span>
            {#if index < $checkpointList.length - 1}
              <div class="checkpoint-timeline__line"></div>
            {/if}
          </div>

          <CheckpointCard
            {checkpoint}
            selected={$checkpointStore.selectedId === checkpoint.id}
            compareSelected={compareFrom === checkpoint.id}
            {compareMode}
            on:select={() => handleSelect(checkpoint)}
            on:restore={() => handleRestore(checkpoint.id)}
            on:delete={() => handleDelete(checkpoint.id)}
          />
        </div>
      {/each}
    </div>

    <!-- Details Panel -->
    {#if $selectedCheckpoint && !$checkpointStore.comparison}
      <CheckpointDetails
        checkpoint={$selectedCheckpoint}
        on:restore={() => handleRestore($selectedCheckpoint.id)}
        on:close={() => checkpointStore.selectCheckpoint(null)}
      />
    {/if}

    <!-- Comparison View -->
    {#if $checkpointStore.comparison}
      <CheckpointComparison
        comparison={$checkpointStore.comparison}
        fromCheckpoint={$checkpointStore.checkpoints.get($checkpointStore.comparisonIds?.[0] || '')}
        toCheckpoint={$checkpointStore.checkpoints.get($checkpointStore.comparisonIds?.[1] || '')}
        on:close={() => checkpointStore.clearComparison()}
      />
    {/if}
  {/if}
</div>

<!-- Create Checkpoint Dialog -->
{#if showCreateDialog}
  <CreateCheckpointDialog
    on:create={async (e) => {
      await checkpointStore.createCheckpoint(e.detail.name, e.detail.description);
      showCreateDialog = false;
    }}
    on:cancel={() => { showCreateDialog = false; }}
  />
{/if}

<!-- Restore Confirmation -->
{#if showRestoreConfirm}
  <ConfirmDialog
    title="Restore Checkpoint?"
    message="This will revert all files to their state at this checkpoint. Current changes will be preserved as a new checkpoint."
    confirmText="Restore"
    confirmVariant="primary"
    on:confirm={confirmRestore}
    on:cancel={() => { showRestoreConfirm = false; }}
  />
{/if}

<!-- Delete Confirmation -->
{#if showDeleteConfirm}
  <ConfirmDialog
    title="Delete Checkpoint?"
    message="This checkpoint will be permanently deleted. This action cannot be undone."
    confirmText="Delete"
    confirmVariant="danger"
    on:confirm={confirmDelete}
    on:cancel={() => { showDeleteConfirm = false; }}
  />
{/if}

<style>
  .checkpoint-display {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .checkpoint-display__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .checkpoint-display__title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .checkpoint-display__actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .compare-hint {
    font-size: 12px;
    color: var(--color-text-secondary);
    font-style: italic;
  }

  .action-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn--primary {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .checkpoint-display__loading,
  .checkpoint-display__error,
  .checkpoint-display__empty {
    padding: 32px;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 14px;
  }

  .checkpoint-display__error {
    color: var(--color-error);
  }

  .checkpoint-timeline {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .checkpoint-timeline__item {
    display: flex;
    gap: 12px;
  }

  .checkpoint-timeline__connector {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 24px;
    flex-shrink: 0;
  }

  .checkpoint-timeline__dot {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-secondary);
    border: 2px solid var(--color-border);
    border-radius: 50%;
    font-size: 10px;
    z-index: 1;
  }

  .checkpoint-timeline__dot--selected {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .checkpoint-timeline__line {
    flex: 1;
    width: 2px;
    background: var(--color-border);
    margin: 4px 0;
  }
</style>
```

### 4. Checkpoint Card Component (src/lib/components/mission/CheckpointCard.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Checkpoint } from '$lib/types/checkpoint';

  export let checkpoint: Checkpoint;
  export let selected = false;
  export let compareSelected = false;
  export let compareMode = false;

  const dispatch = createEventDispatcher<{
    select: void;
    restore: void;
    delete: void;
  }>();

  function formatTime(timestamp: string): string {
    return new Date(timestamp).toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function formatDate(timestamp: string): string {
    return new Date(timestamp).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    });
  }

  $: filesSummary = (() => {
    const created = checkpoint.filesModified.filter(f => f.type === 'created').length;
    const modified = checkpoint.filesModified.filter(f => f.type === 'modified').length;
    const deleted = checkpoint.filesModified.filter(f => f.type === 'deleted').length;
    return { created, modified, deleted };
  })();
</script>

<div
  class="checkpoint-card"
  class:checkpoint-card--selected={selected}
  class:checkpoint-card--compare={compareSelected}
  class:checkpoint-card--compare-mode={compareMode}
  on:click={() => dispatch('select')}
  role="button"
  tabindex="0"
  on:keydown={(e) => e.key === 'Enter' && dispatch('select')}
>
  <div class="checkpoint-card__header">
    <h4 class="checkpoint-card__name">{checkpoint.name}</h4>
    <span class="checkpoint-card__time">
      {formatDate(checkpoint.createdAt)} {formatTime(checkpoint.createdAt)}
    </span>
  </div>

  {#if checkpoint.description}
    <p class="checkpoint-card__description">{checkpoint.description}</p>
  {/if}

  <div class="checkpoint-card__stats">
    <span class="checkpoint-card__stat" title="Files created">
      <span class="stat-icon stat-icon--add">+</span>
      {filesSummary.created}
    </span>
    <span class="checkpoint-card__stat" title="Files modified">
      <span class="stat-icon stat-icon--modify">~</span>
      {filesSummary.modified}
    </span>
    <span class="checkpoint-card__stat" title="Files deleted">
      <span class="stat-icon stat-icon--delete">-</span>
      {filesSummary.deleted}
    </span>
    <span class="checkpoint-card__stat" title="Progress">
      {checkpoint.metadata.progress}%
    </span>
  </div>

  {#if !compareMode}
    <div class="checkpoint-card__actions" on:click|stopPropagation>
      <button
        class="card-action"
        on:click={() => dispatch('restore')}
        title="Restore to this checkpoint"
      >
        Restore
      </button>
      <button
        class="card-action card-action--danger"
        on:click={() => dispatch('delete')}
        title="Delete checkpoint"
      >
        Delete
      </button>
    </div>
  {/if}
</div>

<style>
  .checkpoint-card {
    flex: 1;
    padding: 12px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    margin-bottom: 8px;
  }

  .checkpoint-card:hover {
    border-color: var(--color-primary);
  }

  .checkpoint-card--selected {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .checkpoint-card--compare {
    border-color: var(--color-success);
    background: rgba(52, 211, 153, 0.1);
  }

  .checkpoint-card--compare-mode {
    cursor: pointer;
  }

  .checkpoint-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .checkpoint-card__name {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .checkpoint-card__time {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .checkpoint-card__description {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 0 0 8px 0;
    line-height: 1.4;
  }

  .checkpoint-card__stats {
    display: flex;
    gap: 12px;
    margin-bottom: 8px;
  }

  .checkpoint-card__stat {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .stat-icon {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    font-weight: 600;
    font-size: 10px;
  }

  .stat-icon--add {
    background: rgba(52, 211, 153, 0.2);
    color: var(--color-success);
  }

  .stat-icon--modify {
    background: rgba(96, 165, 250, 0.2);
    color: var(--color-primary);
  }

  .stat-icon--delete {
    background: rgba(248, 113, 113, 0.2);
    color: var(--color-error);
  }

  .checkpoint-card__actions {
    display: flex;
    gap: 8px;
  }

  .card-action {
    padding: 4px 8px;
    border: none;
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
  }

  .card-action:hover {
    background: var(--color-bg-active);
    color: var(--color-text-primary);
  }

  .card-action--danger:hover {
    background: rgba(248, 113, 113, 0.2);
    color: var(--color-error);
  }
</style>
```

---

## Testing Requirements

1. Checkpoints load and display correctly
2. Create checkpoint works
3. Restore checkpoint shows confirmation
4. Delete checkpoint shows confirmation
5. Compare mode selects two checkpoints
6. Comparison displays correctly
7. Timeline renders properly

### Test File (src/lib/components/mission/__tests__/CheckpointDisplay.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import CheckpointDisplay from '../CheckpointDisplay.svelte';

vi.mock('$lib/ipc', () => ({
  ipcRenderer: {
    invoke: vi.fn().mockResolvedValue([
      {
        id: 'cp-1',
        missionId: 'test-1',
        name: 'Initial checkpoint',
        description: 'Auto-created',
        createdAt: new Date().toISOString(),
        type: 'auto',
        trigger: 'step_complete',
        snapshotPath: '/path/to/snapshot',
        filesModified: [{ path: 'file.ts', type: 'modified', linesAdded: 10, linesRemoved: 5 }],
        metadata: { stepNumber: 1, progress: 25, contextUsage: 10, cost: 0.01, duration: 5000 },
      },
    ]),
    on: vi.fn(),
  },
}));

describe('CheckpointDisplay', () => {
  it('renders checkpoints', async () => {
    render(CheckpointDisplay, { missionId: 'test-1' });

    await screen.findByText('Initial checkpoint');
  });

  it('opens create dialog', async () => {
    render(CheckpointDisplay, { missionId: 'test-1' });

    await fireEvent.click(screen.getByText('Create Checkpoint'));

    expect(screen.getByText('Create Checkpoint')).toBeInTheDocument();
  });

  it('shows confirmation before restore', async () => {
    render(CheckpointDisplay, { missionId: 'test-1' });

    await screen.findByText('Initial checkpoint');
    await fireEvent.click(screen.getByText('Restore'));

    expect(screen.getByText('Restore Checkpoint?')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [227-diff-preview.md](227-diff-preview.md)
- Used by: [216-mission-layout.md](216-mission-layout.md)
