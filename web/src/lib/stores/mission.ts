import { writable, derived, get } from 'svelte/store';
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
      const state = get({ subscribe });
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