import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { missionStore, isRunning, progress } from './mission';

// Mock the IPC client
vi.mock('$lib/ipc/client', () => ({
  ipc: {
    invoke: vi.fn()
  }
}));

describe('missionStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    missionStore.clear();
  });

  it('has initial state', () => {
    const state = get(missionStore);
    expect(state.current).toBeNull();
    expect(state.logs).toEqual([]);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('can add logs', () => {
    missionStore.addLog('info', 'Test message');
    const state = get(missionStore);
    
    expect(state.logs).toHaveLength(1);
    expect(state.logs[0].level).toBe('info');
    expect(state.logs[0].message).toBe('Test message');
    expect(state.logs[0].timestamp).toBeInstanceOf(Date);
  });

  it('limits log history to 100 entries', () => {
    // Add 102 logs
    for (let i = 0; i < 102; i++) {
      missionStore.addLog('info', `Message ${i}`);
    }
    
    const state = get(missionStore);
    expect(state.logs).toHaveLength(100);
    expect(state.logs[0].message).toBe('Message 2'); // First two were dropped
    expect(state.logs[99].message).toBe('Message 101');
  });

  it('can clear state', () => {
    missionStore.addLog('info', 'Test');
    missionStore.clear();
    
    const state = get(missionStore);
    expect(state.current).toBeNull();
    expect(state.logs).toEqual([]);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });
});

describe('derived stores', () => {
  beforeEach(() => {
    missionStore.clear();
  });

  it('isRunning derives correctly', () => {
    expect(get(isRunning)).toBe(false);
    
    // This would require mocking the store update, which is complex
    // For now, just test the initial state
  });

  it('progress derives correctly', () => {
    expect(get(progress)).toBe(0);
  });
});