// Placeholder for NAPI-RS native calls
// This will be implemented when the Rust native module is ready

import { randomUUID } from 'crypto';
import type { MissionStatus, SpecFile, SpecMetadata, TachikomaConfig } from '../shared/ipc';

// Native interface that will be implemented via NAPI-RS
export interface NativeInterface {
  // Mission operations
  startMission(specPath: string, backend: string, mode: string): Promise<string>;
  stopMission(missionId: string): Promise<boolean>;
  getMissionStatus(missionId: string): Promise<MissionStatus>;

  // File system operations
  listSpecs(path?: string): Promise<SpecFile[]>;
  readSpec(path: string): Promise<{ content: string; metadata: SpecMetadata }>;

  // Config operations
  getConfig(key?: string): Promise<TachikomaConfig>;
  setConfig(key: string, value: unknown): Promise<boolean>;

  // Log operations
  getLogHistory(missionId?: string): Promise<Array<{ level: string; message: string; timestamp: string }>>;
}

// Placeholder implementation that will be replaced with actual Rust bindings
export const native: NativeInterface = {
  async startMission(specPath: string, backend: string, mode: string): Promise<string> {
    console.log(`[NATIVE PLACEHOLDER] Starting mission: ${specPath} with ${backend} in ${mode} mode`);
    return randomUUID();
  },

  async stopMission(missionId: string): Promise<boolean> {
    console.log(`[NATIVE PLACEHOLDER] Stopping mission: ${missionId}`);
    return true;
  },

  async getMissionStatus(missionId: string): Promise<MissionStatus> {
    console.log(`[NATIVE PLACEHOLDER] Getting status for mission: ${missionId}`);
    return {
      id: missionId,
      state: 'idle',
      progress: 0,
      currentStep: 'Waiting to start',
      startedAt: new Date().toISOString(),
      contextUsage: 0
    };
  },

  async listSpecs(path?: string): Promise<SpecFile[]> {
    console.log(`[NATIVE PLACEHOLDER] Listing specs in: ${path || 'root'}`);
    return [];
  },

  async readSpec(path: string): Promise<{ content: string; metadata: SpecMetadata }> {
    console.log(`[NATIVE PLACEHOLDER] Reading spec: ${path}`);
    return {
      content: '',
      metadata: {
        id: '',
        phase: 0,
        status: 'planned',
        dependencies: []
      }
    };
  },

  async getConfig(key?: string): Promise<TachikomaConfig> {
    console.log(`[NATIVE PLACEHOLDER] Getting config: ${key || 'all'}`);
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
  },

  async setConfig(key: string, value: unknown): Promise<boolean> {
    console.log(`[NATIVE PLACEHOLDER] Setting config: ${key} = ${value}`);
    return true;
  },

  async getLogHistory(missionId?: string): Promise<Array<{ level: string; message: string; timestamp: string }>> {
    console.log(`[NATIVE PLACEHOLDER] Getting log history for: ${missionId || 'all'}`);
    return [];
  }
};