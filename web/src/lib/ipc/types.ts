// Shared IPC types for web app
// These are duplicated from electron/shared/ipc.ts to avoid path issues

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