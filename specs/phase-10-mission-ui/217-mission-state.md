# 217 - Mission State Management

**Phase:** 10 - Mission Panel UI
**Spec ID:** 217
**Status:** Planned
**Dependencies:** 216-mission-layout, 011-common-core-types
**Estimated Context:** ~14% of Sonnet window

---

## Objective

Implement comprehensive state management for missions in the UI layer, including reactive stores for mission data, execution status, and real-time updates from the Rust backend via IPC.

---

## Acceptance Criteria

- [x] `MissionStore` with full CRUD operations
- [x] Real-time state synchronization via IPC
- [x] Optimistic updates with rollback on failure
- [x] Derived stores for filtered/sorted missions
- [x] Mission state history tracking
- [x] Persistence of UI state across sessions

---

## Implementation Details

### 1. Mission Types (src/lib/types/mission.ts)

```typescript
/**
 * Mission types for the UI layer.
 * These mirror the Rust types but are optimized for UI consumption.
 */

export type MissionState =
  | 'idle'
  | 'running'
  | 'paused'
  | 'complete'
  | 'error'
  | 'redlined';

export interface Mission {
  id: string;
  title: string;
  prompt: string;
  state: MissionState;
  specIds: string[];
  backendId: string;
  mode: 'agentic' | 'interactive';
  createdAt: string;
  updatedAt: string;
  startedAt?: string;
  completedAt?: string;
  error?: MissionError;
  progress: MissionProgress;
  cost: MissionCost;
  checkpoints: Checkpoint[];
  tags: string[];
}

export interface MissionProgress {
  currentStep: number;
  totalSteps: number;
  currentAction: string;
  percentage: number;
  contextUsage: ContextUsage;
}

export interface ContextUsage {
  inputTokens: number;
  outputTokens: number;
  maxTokens: number;
  usagePercent: number;
  isNearLimit: boolean;
  isRedlined: boolean;
}

export interface MissionCost {
  inputCost: number;
  outputCost: number;
  totalCost: number;
  currency: string;
}

export interface MissionError {
  code: string;
  message: string;
  details?: string;
  recoverable: boolean;
  timestamp: string;
}

export interface Checkpoint {
  id: string;
  missionId: string;
  name: string;
  description: string;
  createdAt: string;
  snapshotPath: string;
  filesModified: string[];
}

export interface MissionFilter {
  states?: MissionState[];
  tags?: string[];
  search?: string;
  dateFrom?: string;
  dateTo?: string;
  backendId?: string;
}

export interface MissionSort {
  field: 'createdAt' | 'updatedAt' | 'title' | 'state';
  direction: 'asc' | 'desc';
}

export interface MissionListOptions {
  filter?: MissionFilter;
  sort?: MissionSort;
  limit?: number;
  offset?: number;
}

export interface CreateMissionInput {
  title: string;
  prompt: string;
  specIds: string[];
  backendId: string;
  mode: 'agentic' | 'interactive';
  tags?: string[];
}

export interface UpdateMissionInput {
  title?: string;
  prompt?: string;
  specIds?: string[];
  backendId?: string;
  mode?: 'agentic' | 'interactive';
  tags?: string[];
}
```

### 2. IPC Types (src/lib/types/mission-ipc.ts)

```typescript
/**
 * IPC message types for mission communication.
 */

export interface MissionIPCMessage {
  type: MissionIPCMessageType;
  payload: unknown;
  correlationId?: string;
}

export type MissionIPCMessageType =
  | 'mission:created'
  | 'mission:updated'
  | 'mission:deleted'
  | 'mission:state-changed'
  | 'mission:progress'
  | 'mission:log'
  | 'mission:checkpoint'
  | 'mission:error'
  | 'mission:cost-updated'
  | 'mission:context-warning';

export interface MissionStateChangedPayload {
  missionId: string;
  previousState: MissionState;
  newState: MissionState;
  timestamp: string;
}

export interface MissionProgressPayload {
  missionId: string;
  progress: MissionProgress;
}

export interface MissionLogPayload {
  missionId: string;
  level: 'trace' | 'debug' | 'info' | 'warn' | 'error';
  message: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

export interface MissionCheckpointPayload {
  missionId: string;
  checkpoint: Checkpoint;
}

export interface MissionCostPayload {
  missionId: string;
  cost: MissionCost;
}

export interface MissionContextWarningPayload {
  missionId: string;
  contextUsage: ContextUsage;
  warningLevel: 'yellow' | 'orange' | 'red';
}
```

### 3. Mission Store (src/lib/stores/mission-store.ts)

```typescript
import { writable, derived, get } from 'svelte/store';
import type {
  Mission,
  MissionState,
  MissionFilter,
  MissionSort,
  MissionListOptions,
  CreateMissionInput,
  UpdateMissionInput,
  MissionProgress,
  MissionCost,
  Checkpoint,
} from '$lib/types/mission';
import type {
  MissionIPCMessage,
  MissionStateChangedPayload,
  MissionProgressPayload,
  MissionLogPayload,
  MissionCheckpointPayload,
  MissionCostPayload,
} from '$lib/types/mission-ipc';
import { ipcRenderer } from '$lib/ipc';

interface MissionStoreState {
  missions: Map<string, Mission>;
  loading: boolean;
  error: string | null;
  selectedMissionId: string | null;
  filter: MissionFilter;
  sort: MissionSort;
  pendingOperations: Map<string, { type: string; timestamp: number }>;
}

function createMissionStore() {
  const initialState: MissionStoreState = {
    missions: new Map(),
    loading: false,
    error: null,
    selectedMissionId: null,
    filter: {},
    sort: { field: 'updatedAt', direction: 'desc' },
    pendingOperations: new Map(),
  };

  const { subscribe, set, update } = writable<MissionStoreState>(initialState);

  // IPC event handlers
  function handleIPCMessage(message: MissionIPCMessage) {
    switch (message.type) {
      case 'mission:created':
        handleMissionCreated(message.payload as Mission);
        break;
      case 'mission:updated':
        handleMissionUpdated(message.payload as Mission);
        break;
      case 'mission:deleted':
        handleMissionDeleted(message.payload as { missionId: string });
        break;
      case 'mission:state-changed':
        handleStateChanged(message.payload as MissionStateChangedPayload);
        break;
      case 'mission:progress':
        handleProgress(message.payload as MissionProgressPayload);
        break;
      case 'mission:checkpoint':
        handleCheckpoint(message.payload as MissionCheckpointPayload);
        break;
      case 'mission:cost-updated':
        handleCostUpdated(message.payload as MissionCostPayload);
        break;
    }
  }

  function handleMissionCreated(mission: Mission) {
    update(state => {
      const missions = new Map(state.missions);
      missions.set(mission.id, mission);
      return { ...state, missions };
    });
  }

  function handleMissionUpdated(mission: Mission) {
    update(state => {
      const missions = new Map(state.missions);
      missions.set(mission.id, mission);
      return { ...state, missions };
    });
  }

  function handleMissionDeleted({ missionId }: { missionId: string }) {
    update(state => {
      const missions = new Map(state.missions);
      missions.delete(missionId);
      return {
        ...state,
        missions,
        selectedMissionId: state.selectedMissionId === missionId ? null : state.selectedMissionId,
      };
    });
  }

  function handleStateChanged(payload: MissionStateChangedPayload) {
    update(state => {
      const mission = state.missions.get(payload.missionId);
      if (!mission) return state;

      const missions = new Map(state.missions);
      missions.set(payload.missionId, {
        ...mission,
        state: payload.newState as MissionState,
        updatedAt: payload.timestamp,
      });
      return { ...state, missions };
    });
  }

  function handleProgress(payload: MissionProgressPayload) {
    update(state => {
      const mission = state.missions.get(payload.missionId);
      if (!mission) return state;

      const missions = new Map(state.missions);
      missions.set(payload.missionId, {
        ...mission,
        progress: payload.progress,
      });
      return { ...state, missions };
    });
  }

  function handleCheckpoint(payload: MissionCheckpointPayload) {
    update(state => {
      const mission = state.missions.get(payload.missionId);
      if (!mission) return state;

      const missions = new Map(state.missions);
      missions.set(payload.missionId, {
        ...mission,
        checkpoints: [...mission.checkpoints, payload.checkpoint],
      });
      return { ...state, missions };
    });
  }

  function handleCostUpdated(payload: MissionCostPayload) {
    update(state => {
      const mission = state.missions.get(payload.missionId);
      if (!mission) return state;

      const missions = new Map(state.missions);
      missions.set(payload.missionId, {
        ...mission,
        cost: payload.cost,
      });
      return { ...state, missions };
    });
  }

  // Initialize IPC listener
  ipcRenderer.on('mission:event', (_event, message: MissionIPCMessage) => {
    handleIPCMessage(message);
  });

  return {
    subscribe,

    // Load missions from backend
    async loadMissions(options?: MissionListOptions): Promise<void> {
      update(s => ({ ...s, loading: true, error: null }));

      try {
        const missions = await ipcRenderer.invoke('mission:list', options);
        update(s => ({
          ...s,
          missions: new Map(missions.map((m: Mission) => [m.id, m])),
          loading: false,
        }));
      } catch (error) {
        update(s => ({
          ...s,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to load missions',
        }));
      }
    },

    // Create a new mission with optimistic update
    async createMission(input: CreateMissionInput): Promise<Mission | null> {
      const optimisticId = `temp-${Date.now()}`;
      const optimisticMission: Mission = {
        id: optimisticId,
        ...input,
        state: 'idle',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        progress: {
          currentStep: 0,
          totalSteps: 0,
          currentAction: '',
          percentage: 0,
          contextUsage: {
            inputTokens: 0,
            outputTokens: 0,
            maxTokens: 200000,
            usagePercent: 0,
            isNearLimit: false,
            isRedlined: false,
          },
        },
        cost: { inputCost: 0, outputCost: 0, totalCost: 0, currency: 'USD' },
        checkpoints: [],
        tags: input.tags || [],
      };

      // Optimistic update
      update(s => {
        const missions = new Map(s.missions);
        missions.set(optimisticId, optimisticMission);
        const pendingOperations = new Map(s.pendingOperations);
        pendingOperations.set(optimisticId, { type: 'create', timestamp: Date.now() });
        return { ...s, missions, pendingOperations };
      });

      try {
        const mission = await ipcRenderer.invoke('mission:create', input);

        // Replace optimistic mission with real one
        update(s => {
          const missions = new Map(s.missions);
          missions.delete(optimisticId);
          missions.set(mission.id, mission);
          const pendingOperations = new Map(s.pendingOperations);
          pendingOperations.delete(optimisticId);
          return { ...s, missions, pendingOperations, selectedMissionId: mission.id };
        });

        return mission;
      } catch (error) {
        // Rollback optimistic update
        update(s => {
          const missions = new Map(s.missions);
          missions.delete(optimisticId);
          const pendingOperations = new Map(s.pendingOperations);
          pendingOperations.delete(optimisticId);
          return {
            ...s,
            missions,
            pendingOperations,
            error: error instanceof Error ? error.message : 'Failed to create mission',
          };
        });
        return null;
      }
    },

    // Update mission
    async updateMission(missionId: string, input: UpdateMissionInput): Promise<boolean> {
      const state = get({ subscribe });
      const originalMission = state.missions.get(missionId);
      if (!originalMission) return false;

      // Optimistic update
      update(s => {
        const missions = new Map(s.missions);
        missions.set(missionId, {
          ...originalMission,
          ...input,
          updatedAt: new Date().toISOString(),
        });
        return { ...s, missions };
      });

      try {
        await ipcRenderer.invoke('mission:update', { missionId, input });
        return true;
      } catch (error) {
        // Rollback
        update(s => {
          const missions = new Map(s.missions);
          missions.set(missionId, originalMission);
          return {
            ...s,
            missions,
            error: error instanceof Error ? error.message : 'Failed to update mission',
          };
        });
        return false;
      }
    },

    // Delete mission
    async deleteMission(missionId: string): Promise<boolean> {
      const state = get({ subscribe });
      const originalMission = state.missions.get(missionId);

      // Optimistic delete
      update(s => {
        const missions = new Map(s.missions);
        missions.delete(missionId);
        return {
          ...s,
          missions,
          selectedMissionId: s.selectedMissionId === missionId ? null : s.selectedMissionId,
        };
      });

      try {
        await ipcRenderer.invoke('mission:delete', missionId);
        return true;
      } catch (error) {
        // Rollback
        if (originalMission) {
          update(s => {
            const missions = new Map(s.missions);
            missions.set(missionId, originalMission);
            return {
              ...s,
              missions,
              error: error instanceof Error ? error.message : 'Failed to delete mission',
            };
          });
        }
        return false;
      }
    },

    // Mission control actions
    async startMission(missionId: string): Promise<boolean> {
      try {
        await ipcRenderer.invoke('mission:start', missionId);
        return true;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to start mission',
        }));
        return false;
      }
    },

    async pauseMission(missionId: string): Promise<boolean> {
      try {
        await ipcRenderer.invoke('mission:pause', missionId);
        return true;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to pause mission',
        }));
        return false;
      }
    },

    async resumeMission(missionId: string): Promise<boolean> {
      try {
        await ipcRenderer.invoke('mission:resume', missionId);
        return true;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to resume mission',
        }));
        return false;
      }
    },

    async abortMission(missionId: string): Promise<boolean> {
      try {
        await ipcRenderer.invoke('mission:abort', missionId);
        return true;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to abort mission',
        }));
        return false;
      }
    },

    // Selection
    selectMission(missionId: string | null) {
      update(s => ({ ...s, selectedMissionId: missionId }));
    },

    // Filtering and sorting
    setFilter(filter: MissionFilter) {
      update(s => ({ ...s, filter }));
    },

    setSort(sort: MissionSort) {
      update(s => ({ ...s, sort }));
    },

    clearError() {
      update(s => ({ ...s, error: null }));
    },
  };
}

export const missionStore = createMissionStore();

// Derived stores
export const selectedMission = derived(missionStore, $store =>
  $store.selectedMissionId ? $store.missions.get($store.selectedMissionId) || null : null
);

export const missionList = derived(missionStore, $store => {
  let missions = Array.from($store.missions.values());

  // Apply filters
  const { filter, sort } = $store;

  if (filter.states?.length) {
    missions = missions.filter(m => filter.states!.includes(m.state));
  }

  if (filter.tags?.length) {
    missions = missions.filter(m => filter.tags!.some(tag => m.tags.includes(tag)));
  }

  if (filter.search) {
    const search = filter.search.toLowerCase();
    missions = missions.filter(
      m =>
        m.title.toLowerCase().includes(search) ||
        m.prompt.toLowerCase().includes(search)
    );
  }

  if (filter.backendId) {
    missions = missions.filter(m => m.backendId === filter.backendId);
  }

  // Apply sorting
  missions.sort((a, b) => {
    const aVal = a[sort.field];
    const bVal = b[sort.field];

    if (aVal < bVal) return sort.direction === 'asc' ? -1 : 1;
    if (aVal > bVal) return sort.direction === 'asc' ? 1 : -1;
    return 0;
  });

  return missions;
});

export const runningMissions = derived(missionStore, $store =>
  Array.from($store.missions.values()).filter(m => m.state === 'running')
);

export const missionStats = derived(missionStore, $store => {
  const missions = Array.from($store.missions.values());
  return {
    total: missions.length,
    running: missions.filter(m => m.state === 'running').length,
    paused: missions.filter(m => m.state === 'paused').length,
    completed: missions.filter(m => m.state === 'complete').length,
    errors: missions.filter(m => m.state === 'error').length,
    totalCost: missions.reduce((sum, m) => sum + m.cost.totalCost, 0),
  };
});
```

---

## Testing Requirements

1. Store initializes with correct default state
2. CRUD operations work with optimistic updates
3. Rollback occurs on failed operations
4. IPC messages update store correctly
5. Derived stores compute correctly
6. Filtering and sorting work as expected
7. Mission control actions (start/pause/resume/abort) work

### Test File (src/lib/stores/__tests__/mission-store.test.ts)

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { missionStore, missionList, selectedMission, missionStats } from '../mission-store';

// Mock IPC
vi.mock('$lib/ipc', () => ({
  ipcRenderer: {
    invoke: vi.fn(),
    on: vi.fn(),
  },
}));

import { ipcRenderer } from '$lib/ipc';

describe('MissionStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with empty state', () => {
    const state = get(missionStore);
    expect(state.missions.size).toBe(0);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('loads missions from backend', async () => {
    const mockMissions = [
      { id: '1', title: 'Test Mission', state: 'idle' },
    ];
    vi.mocked(ipcRenderer.invoke).mockResolvedValue(mockMissions);

    await missionStore.loadMissions();

    const state = get(missionStore);
    expect(state.missions.size).toBe(1);
    expect(state.missions.get('1')?.title).toBe('Test Mission');
  });

  it('performs optimistic create with rollback on failure', async () => {
    vi.mocked(ipcRenderer.invoke).mockRejectedValue(new Error('Network error'));

    const result = await missionStore.createMission({
      title: 'New Mission',
      prompt: 'Do something',
      specIds: [],
      backendId: 'claude',
      mode: 'agentic',
    });

    expect(result).toBeNull();
    const state = get(missionStore);
    expect(state.missions.size).toBe(0);
    expect(state.error).toBe('Network error');
  });

  it('filters missions correctly', async () => {
    // Setup missions
    const mockMissions = [
      { id: '1', title: 'Running', state: 'running', tags: ['test'] },
      { id: '2', title: 'Completed', state: 'complete', tags: ['prod'] },
    ];
    vi.mocked(ipcRenderer.invoke).mockResolvedValue(mockMissions);
    await missionStore.loadMissions();

    // Apply filter
    missionStore.setFilter({ states: ['running'] });

    const filtered = get(missionList);
    expect(filtered.length).toBe(1);
    expect(filtered[0].id).toBe('1');
  });

  it('computes stats correctly', async () => {
    const mockMissions = [
      { id: '1', state: 'running', cost: { totalCost: 0.10 } },
      { id: '2', state: 'complete', cost: { totalCost: 0.25 } },
      { id: '3', state: 'error', cost: { totalCost: 0.05 } },
    ];
    vi.mocked(ipcRenderer.invoke).mockResolvedValue(mockMissions);
    await missionStore.loadMissions();

    const stats = get(missionStats);
    expect(stats.total).toBe(3);
    expect(stats.running).toBe(1);
    expect(stats.completed).toBe(1);
    expect(stats.errors).toBe(1);
    expect(stats.totalCost).toBeCloseTo(0.40);
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [218-mission-creation.md](218-mission-creation.md)
- Used by: All Mission Panel UI specs
