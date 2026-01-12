# 005 - IPC Bridge

**Phase:** 0 - Setup
**Spec ID:** 005
**Status:** Planned
**Dependencies:** 003-electron-shell, 004-svelte-integration
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Create a type-safe IPC bridge between Electron main process and Svelte renderer, with preparation for Rust native module integration via NAPI-RS.

---

## Acceptance Criteria

- [ ] Type-safe IPC channel definitions
- [ ] Main process IPC handlers registered
- [ ] Renderer-side typed client
- [ ] Svelte store bindings for IPC state
- [ ] Error handling for IPC failures
- [ ] Placeholder for NAPI-RS native calls

---

## Implementation Details

### 1. Shared Types (electron/shared/ipc.ts)

```typescript
// IPC Channel definitions - shared between main and renderer

export interface IpcChannels {
  // Mission operations
  'mission:start': {
    request: { specPath: string; backend: string; mode: 'attended' | 'unattended' };
    response: { missionId: string };
  };
  'mission:stop': {
    request: { missionId: string };
    response: { success: boolean };
  };
  'mission:status': {
    request: { missionId: string };
    response: MissionStatus;
  };

  // Spec operations
  'spec:list': {
    request: { path?: string };
    response: SpecFile[];
  };
  'spec:read': {
    request: { path: string };
    response: { content: string; metadata: SpecMetadata };
  };

  // Config operations
  'config:get': {
    request: { key?: string };
    response: TachikomaConfig;
  };
  'config:set': {
    request: { key: string; value: unknown };
    response: { success: boolean };
  };
}

// Event channels (main -> renderer)
export interface IpcEvents {
  'mission:progress': { missionId: string; progress: number; message: string };
  'mission:log': { missionId: string; level: 'info' | 'warn' | 'error'; message: string };
  'mission:complete': { missionId: string; success: boolean; summary: string };
  'mission:error': { missionId: string; error: string };
}

// Types
export interface MissionStatus {
  id: string;
  state: 'idle' | 'running' | 'paused' | 'complete' | 'error';
  progress: number;
  currentStep: string;
  startedAt: string;
  contextUsage: number;
}

export interface SpecFile {
  path: string;
  name: string;
  type: 'spec' | 'plan' | 'readme';
  status: 'planned' | 'in_progress' | 'complete';
}

export interface SpecMetadata {
  id: string;
  phase: number;
  status: string;
  dependencies: string[];
}

export interface TachikomaConfig {
  backend: {
    brain: string;
    thinkTank: string;
  };
  loop: {
    maxIterations: number;
    stopOn: string[];
  };
}
```

### 2. Main Process Handlers (electron/main/ipc-handlers.ts)

```typescript
import { ipcMain, IpcMainInvokeEvent } from 'electron';
import type { IpcChannels } from '../shared/ipc';

// Type-safe handler registration
function handle<K extends keyof IpcChannels>(
  channel: K,
  handler: (
    event: IpcMainInvokeEvent,
    request: IpcChannels[K]['request']
  ) => Promise<IpcChannels[K]['response']>
): void {
  ipcMain.handle(channel, handler);
}

export function registerIpcHandlers(): void {
  // Mission handlers
  handle('mission:start', async (_event, request) => {
    // TODO: Implement via Rust native module
    console.log('Starting mission:', request);
    return { missionId: crypto.randomUUID() };
  });

  handle('mission:stop', async (_event, request) => {
    console.log('Stopping mission:', request.missionId);
    return { success: true };
  });

  handle('mission:status', async (_event, request) => {
    return {
      id: request.missionId,
      state: 'idle',
      progress: 0,
      currentStep: '',
      startedAt: new Date().toISOString(),
      contextUsage: 0
    };
  });

  // Spec handlers
  handle('spec:list', async (_event, request) => {
    // TODO: Read from file system via Rust
    console.log('Listing specs:', request.path);
    return [];
  });

  handle('spec:read', async (_event, request) => {
    // TODO: Read file via Rust
    return {
      content: '',
      metadata: {
        id: '',
        phase: 0,
        status: 'planned',
        dependencies: []
      }
    };
  });

  // Config handlers
  handle('config:get', async (_event, _request) => {
    return {
      backend: {
        brain: 'claude',
        thinkTank: 'o3'
      },
      loop: {
        maxIterations: 100,
        stopOn: ['redline', 'test_fail_streak:3']
      }
    };
  });

  handle('config:set', async (_event, request) => {
    console.log('Setting config:', request.key, request.value);
    return { success: true };
  });
}
```

### 3. Renderer Client (web/src/lib/ipc/client.ts)

```typescript
import type { IpcChannels, IpcEvents } from './types';

class TachikomaIpc {
  private listeners = new Map<string, Set<Function>>();

  async invoke<K extends keyof IpcChannels>(
    channel: K,
    request: IpcChannels[K]['request']
  ): Promise<IpcChannels[K]['response']> {
    if (typeof window === 'undefined' || !window.tachikoma) {
      throw new Error('Tachikoma IPC not available');
    }
    return window.tachikoma.invoke(channel, request) as Promise<IpcChannels[K]['response']>;
  }

  on<K extends keyof IpcEvents>(
    channel: K,
    callback: (data: IpcEvents[K]) => void
  ): () => void {
    if (typeof window === 'undefined' || !window.tachikoma) {
      return () => {};
    }

    if (!this.listeners.has(channel)) {
      this.listeners.set(channel, new Set());
    }
    this.listeners.get(channel)!.add(callback);

    window.tachikoma.on(channel, callback as any);

    // Return unsubscribe function
    return () => {
      this.listeners.get(channel)?.delete(callback);
      window.tachikoma.off(channel, callback as any);
    };
  }
}

export const ipc = new TachikomaIpc();
```

### 4. Svelte Store Bindings (web/src/lib/stores/mission.ts)

```typescript
import { writable, derived } from 'svelte/store';
import { ipc } from '$lib/ipc/client';
import type { MissionStatus } from '$lib/ipc/types';

interface MissionState {
  current: MissionStatus | null;
  logs: Array<{ level: string; message: string; timestamp: Date }>;
  loading: boolean;
  error: string | null;
}

function createMissionStore() {
  const { subscribe, set, update } = writable<MissionState>({
    current: null,
    logs: [],
    loading: false,
    error: null
  });

  return {
    subscribe,

    async start(specPath: string, backend: string, mode: 'attended' | 'unattended') {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const { missionId } = await ipc.invoke('mission:start', { specPath, backend, mode });
        const status = await ipc.invoke('mission:status', { missionId });
        update(s => ({ ...s, current: status, loading: false }));
        return missionId;
      } catch (e) {
        update(s => ({ ...s, loading: false, error: String(e) }));
        throw e;
      }
    },

    async stop() {
      const state = await new Promise<MissionState>(resolve => {
        subscribe(s => resolve(s))();
      });
      if (state.current) {
        await ipc.invoke('mission:stop', { missionId: state.current.id });
        update(s => ({ ...s, current: null }));
      }
    },

    addLog(level: string, message: string) {
      update(s => ({
        ...s,
        logs: [...s.logs.slice(-99), { level, message, timestamp: new Date() }]
      }));
    },

    clear() {
      set({ current: null, logs: [], loading: false, error: null });
    }
  };
}

export const missionStore = createMissionStore();

// Derived stores
export const isRunning = derived(missionStore, $m => $m.current?.state === 'running');
export const progress = derived(missionStore, $m => $m.current?.progress ?? 0);
```

---

## Testing Requirements

1. IPC invoke calls from renderer reach main process
2. Events from main process reach renderer callbacks
3. Type errors caught at compile time
4. Store updates correctly on IPC responses

---

## Related Specs

- Depends on: [003-electron-shell.md](003-electron-shell.md), [004-svelte-integration.md](004-svelte-integration.md)
- Next: [006-dev-tooling.md](006-dev-tooling.md)
- Related: [173-rust-native.md](../phase-08-electron/173-rust-native.md)
