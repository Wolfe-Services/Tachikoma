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

  // Forge operations
  'forge:createSession': {
    request: ForgeSessionRequest;
    response: ForgeSessionResponse;
  };
  'forge:getSession': {
    request: { sessionId: string };
    response: ForgeSessionResponse | null;
  };
  'forge:listSessions': {
    request: {};
    response: ForgeSessionResponse[];
  };
  'forge:deleteSession': {
    request: { sessionId: string };
    response: { success: boolean };
  };
  'forge:startDeliberation': {
    request: { sessionId: string; phase: ForgePhase };
    response: { success: boolean };
  };
  'forge:stopDeliberation': {
    request: { sessionId: string };
    response: { success: boolean };
  };
  'forge:submitMessage': {
    request: { sessionId: string; content: string; participantId: string };
    response: { messageId: string };
  };
  'forge:generateOutput': {
    request: ForgeOutputRequest;
    response: ForgeOutputResponse;
  };
}

// Event channels (main -> renderer)
export interface IpcEvents {
  'mission:progress': { missionId: string; progress: number; message: string };
  'mission:log': { missionId: string; level: 'info' | 'warn' | 'error'; message: string };
  'mission:complete': { missionId: string; success: boolean; summary: string };
  'mission:error': { missionId: string; error: string };
  
  // Forge events (streaming deliberation)
  'forge:message': ForgeMessageEvent;
  'forge:phaseChange': { sessionId: string; phase: ForgePhase; previousPhase: ForgePhase };
  'forge:roundComplete': { sessionId: string; roundNumber: number; summary: string };
  'forge:convergence': { sessionId: string; score: number; converged: boolean };
  'forge:error': { sessionId: string; error: string };
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

// Updater Types
export interface UpdateState {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  downloaded: boolean;
  error: string | null;
  updateInfo: UpdateInfo | null;
  progress: ProgressInfo | null;
}

export interface UpdateInfo {
  version: string;
  releaseDate: string;
  releaseNotes?: string;
}

export interface ProgressInfo {
  percent: number;
  bytesPerSecond: number;
  total: number;
  transferred: number;
}

export interface UpdateHistoryEntry {
  version: string;
  date: string;
  status: 'installed' | 'skipped' | 'failed';
  notes?: string;
}

// Forge Types
export type ForgePhase =
  | 'idle'
  | 'configuring'
  | 'drafting'
  | 'critiquing'
  | 'deliberating'
  | 'converging'
  | 'completed'
  | 'paused'
  | 'error';

export interface ForgeParticipant {
  id: string;
  name: string;
  type: 'human' | 'ai';
  role: string;
  modelId?: string; // e.g., "claude-3-5-sonnet-20241022"
  status: 'active' | 'inactive' | 'thinking' | 'contributing';
}

export interface ForgeOracle {
  id: string;
  name: string;
  modelId: string;
  config: Record<string, unknown>;
}

export interface ForgeSessionConfig {
  maxRounds: number;
  convergenceThreshold: number;
  roundTimeoutMs: number;
  allowHumanIntervention: boolean;
}

export interface ForgeSessionRequest {
  name: string;
  goal: string;
  participants: ForgeParticipant[];
  oracle: ForgeOracle | null;
  config: ForgeSessionConfig;
}

export interface ForgeSessionResponse {
  id: string;
  name: string;
  goal: string;
  phase: ForgePhase;
  participants: ForgeParticipant[];
  oracle: ForgeOracle | null;
  config: ForgeSessionConfig;
  roundCount: number;
  totalCostUsd: number;
  totalTokens: { input: number; output: number };
  createdAt: string;
  updatedAt: string;
}

export interface ForgeMessageEvent {
  sessionId: string;
  messageId: string;
  participantId: string;
  participantName: string;
  participantType: 'human' | 'ai';
  content: string;
  contentDelta?: string; // For streaming
  timestamp: string;
  type: 'proposal' | 'critique' | 'synthesis' | 'thinking';
  status: 'pending' | 'streaming' | 'complete';
}

export type ForgeOutputFormat = 'markdown' | 'json' | 'yaml' | 'html' | 'plain' | 'beads';

export interface ForgeOutputRequest {
  sessionId: string;
  format: ForgeOutputFormat;
  includeMetadata?: boolean;
  includeHistory?: boolean;
  includeDecisions?: boolean;
  includeDissents?: boolean;
  includeMetrics?: boolean;
}

export interface ForgeOutputResponse {
  sessionId: string;
  format: ForgeOutputFormat;
  content: string;
  filename: string;
}