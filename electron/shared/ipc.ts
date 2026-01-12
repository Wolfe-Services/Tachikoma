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

  // Updater operations
  'updater:check': {
    request: { silent?: boolean };
    response: { success: boolean };
  };
  'updater:download': {
    request: {};
    response: { success: boolean };
  };
  'updater:install': {
    request: {};
    response: { success: boolean };
  };
  'updater:getState': {
    request: {};
    response: UpdateState;
  };
  'updater:setChannel': {
    request: { channel: 'stable' | 'beta' | 'alpha' };
    response: { success: boolean };
  };
  'updater:clearSkipped': {
    request: {};
    response: { success: boolean };
  };
  'updater:getHistory': {
    request: {};
    response: UpdateHistoryEntry[];
  };
  'updater:startAutoCheck': {
    request: {};
    response: { success: boolean };
  };
  'updater:stopAutoCheck': {
    request: {};
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