// NAPI-RS native bindings for Tachikoma Forge
// Uses the compiled Rust module for forge operations

import { randomUUID } from 'crypto';
import { BrowserWindow } from 'electron';
import type { 
  MissionStatus, 
  SpecFile, 
  SpecMetadata, 
  TachikomaConfig,
  ForgeSessionRequest,
  ForgeSessionResponse,
  ForgePhase,
  ForgeOutputRequest,
  ForgeOutputResponse,
  ForgeMessageEvent
} from '../shared/ipc';

// Import types from NAPI bindings
import type {
  JsForgeSessionRequest,
  JsForgeSessionResponse,
  DeliberationStream
} from './native-bindings/index';

// Try to load the NAPI module
let napiModule: typeof import('./native-bindings/index') | null = null;
let deliberationStreams: Map<string, DeliberationStream> = new Map();

type ForgeStreamEnvelope = { type: string; data: any };
type RoundType = 'Draft' | 'Critique' | 'Synthesis' | 'Convergence' | string;

const activeMessageByParticipant: Map<string, string> = new Map();
const contentByMessageId: Map<string, string> = new Map();
const participantNameById: Map<string, string> = new Map();
const roundTypeBySession: Map<string, RoundType> = new Map();
const roundNumberBySession: Map<string, number> = new Map();

function broadcast(channel: string, data: unknown) {
  for (const win of BrowserWindow.getAllWindows()) {
    if (!win.isDestroyed()) {
      win.webContents.send(channel, data);
    }
  }
}

function extractVariantData(data: any): any {
  // NAPI currently wraps `ForgeEvent` as a serde enum like { VariantName: { ...fields } }
  if (!data || typeof data !== 'object') return data;
  const keys = Object.keys(data);
  if (keys.length === 1) return data[keys[0]];
  return data;
}

function phaseFromRoundType(roundType: RoundType): ForgePhase {
  switch (roundType) {
    case 'Draft':
      return 'drafting';
    case 'Critique':
      return 'critiquing';
    case 'Synthesis':
    case 'Convergence':
      return 'converging';
    default:
      return 'deliberating';
  }
}

function messageTypeFromRoundType(roundType: RoundType): ForgeMessageEvent['type'] {
  switch (roundType) {
    case 'Draft':
      return 'proposal';
    case 'Critique':
      return 'critique';
    case 'Synthesis':
    case 'Convergence':
      return 'synthesis';
    default:
      return 'proposal';
  }
}

function nextMessageId(sessionId: string, participantId: string) {
  const round = roundNumberBySession.get(sessionId) ?? 0;
  return `${sessionId}:${participantId}:${round}:${Date.now()}`;
}

function upsertStreamMessage(evt: ForgeMessageEvent) {
  broadcast('forge:message', evt);
}

try {
  // Dynamic import for native module
  napiModule = require('./native-bindings/index');
  console.log('[NATIVE] NAPI module loaded successfully');
} catch (error) {
  console.warn('[NATIVE] Failed to load NAPI module, using placeholders:', error);
}

// Convert between IPC types and NAPI types
function normalizeParticipantType(value: unknown): 'human' | 'ai' {
  return value === 'human' ? 'human' : 'ai';
}

function normalizeParticipantStatus(
  value: unknown
): 'active' | 'inactive' | 'thinking' | 'contributing' {
  switch (value) {
    case 'inactive':
      return 'inactive';
    case 'thinking':
      return 'thinking';
    case 'contributing':
      return 'contributing';
    case 'active':
    default:
      return 'active';
  }
}

function toNapiRequest(request: ForgeSessionRequest): JsForgeSessionRequest {
  return {
    name: request.name,
    goal: request.goal,
    participants: request.participants.map(p => ({
      id: p.id,
      name: p.name,
      type: p.type,
      role: p.role,
      modelId: p.modelId,
      status: p.status
    })),
    oracle: request.oracle ? {
      id: request.oracle.id,
      name: request.oracle.name,
      modelId: request.oracle.modelId,
      config: JSON.stringify(request.oracle.config)
    } : undefined,
    config: {
      maxRounds: request.config.maxRounds,
      convergenceThreshold: request.config.convergenceThreshold,
      roundTimeoutMs: request.config.roundTimeoutMs,
      allowHumanIntervention: request.config.allowHumanIntervention
    }
  };
}

function fromNapiResponse(response: JsForgeSessionResponse): ForgeSessionResponse {
  return {
    id: response.id,
    name: response.name,
    goal: response.goal,
    phase: response.phase as ForgePhase,
    participants: response.participants.map(p => ({
      id: p.id,
      name: p.name,
      type: normalizeParticipantType(p.type),
      role: p.role,
      modelId: p.modelId,
      status: normalizeParticipantStatus(p.status)
    })),
    oracle: response.oracle ? {
      id: response.oracle.id,
      name: response.oracle.name,
      modelId: response.oracle.modelId,
      config: JSON.parse(response.oracle.config || '{}')
    } : null,
    config: {
      maxRounds: response.config.maxRounds,
      convergenceThreshold: response.config.convergenceThreshold,
      roundTimeoutMs: response.config.roundTimeoutMs,
      allowHumanIntervention: response.config.allowHumanIntervention
    },
    roundCount: response.roundCount,
    totalCostUsd: response.totalCostUsd,
    totalTokens: {
      input: response.totalTokens.input,
      output: response.totalTokens.output
    },
    createdAt: response.createdAt,
    updatedAt: response.updatedAt
  };
}

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

  // Forge operations
  createForgeSession(request: ForgeSessionRequest): Promise<ForgeSessionResponse>;
  getForgeSession(sessionId: string): Promise<ForgeSessionResponse | null>;
  listForgeSessions(): Promise<ForgeSessionResponse[]>;
  deleteForgeSession(sessionId: string): Promise<boolean>;
  startDeliberation(sessionId: string, phase: ForgePhase): Promise<boolean>;
  stopDeliberation(sessionId: string): Promise<boolean>;
  submitForgeMessage(sessionId: string, content: string, participantId: string): Promise<string>;
  generateForgeOutput(request: ForgeOutputRequest): Promise<ForgeOutputResponse>;
}

// Implementation using NAPI module with fallback to placeholders
export const native: NativeInterface = {
  // === Mission operations (placeholder) ===
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

  // === File system operations (placeholder) ===
  async listSpecs(path?: string): Promise<SpecFile[]> {
    console.log(`[NATIVE PLACEHOLDER] Listing specs in: ${path || 'root'}`);
    return [];
  },

  async readSpec(specPath: string): Promise<{ content: string; metadata: SpecMetadata }> {
    console.log(`[NATIVE PLACEHOLDER] Reading spec: ${specPath}`);
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

  // === Config operations (placeholder) ===
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

  // === Log operations (placeholder) ===
  async getLogHistory(missionId?: string): Promise<Array<{ level: string; message: string; timestamp: string }>> {
    console.log(`[NATIVE PLACEHOLDER] Getting log history for: ${missionId || 'all'}`);
    return [];
  },

  // === Forge operations (NAPI-RS powered) ===
  async createForgeSession(request: ForgeSessionRequest): Promise<ForgeSessionResponse> {
    if (napiModule) {
      try {
        console.log('[NATIVE] Creating forge session via NAPI');
        const napiRequest = toNapiRequest(request);
        const response = napiModule.createForgeSession(napiRequest);
        return fromNapiResponse(response);
      } catch (error) {
        console.error('[NATIVE] NAPI createForgeSession failed:', error);
      }
    }
    
    // Fallback to placeholder
    console.log(`[NATIVE PLACEHOLDER] Creating forge session: ${request.name}`);
    const now = new Date().toISOString();
    return {
      id: randomUUID(),
      name: request.name,
      goal: request.goal,
      phase: 'configuring',
      participants: request.participants,
      oracle: request.oracle,
      config: request.config,
      roundCount: 0,
      totalCostUsd: 0,
      totalTokens: { input: 0, output: 0 },
      createdAt: now,
      updatedAt: now
    };
  },

  async getForgeSession(sessionId: string): Promise<ForgeSessionResponse | null> {
    if (napiModule) {
      try {
        console.log('[NATIVE] Getting forge session via NAPI');
        const response = napiModule.getSession(sessionId);
        return response ? fromNapiResponse(response) : null;
      } catch (error) {
        console.error('[NATIVE] NAPI getSession failed:', error);
      }
    }
    
    console.log(`[NATIVE PLACEHOLDER] Getting forge session: ${sessionId}`);
    return null;
  },

  async listForgeSessions(): Promise<ForgeSessionResponse[]> {
    if (napiModule) {
      try {
        console.log('[NATIVE] Listing forge sessions via NAPI');
        const sessions = napiModule.listSessions();
        return sessions.map(fromNapiResponse);
      } catch (error) {
        console.error('[NATIVE] NAPI listSessions failed:', error);
      }
    }
    
    console.log(`[NATIVE PLACEHOLDER] Listing forge sessions`);
    return [];
  },

  async deleteForgeSession(sessionId: string): Promise<boolean> {
    console.log(`[NATIVE PLACEHOLDER] Deleting forge session: ${sessionId}`);
    return true;
  },

  async startDeliberation(sessionId: string, phase: ForgePhase): Promise<boolean> {
    if (napiModule) {
      try {
        console.log(`[NATIVE] Starting deliberation for session ${sessionId} in phase ${phase}`);

        // If this session wasn't created via the backend (e.g. persisted/mock UI session),
        // return false so the renderer can fall back to mock-mode deliberation.
        try {
          const existing = napiModule.getSession(sessionId);
          if (!existing) {
            console.warn('[NATIVE] startDeliberation: session not found in backend, falling back to mock', { sessionId });
            return false;
          }
        } catch (e) {
          console.warn('[NATIVE] startDeliberation: getSession check failed, falling back to mock', e);
          return false;
        }

        const stream = napiModule.startDeliberation(sessionId);
        deliberationStreams.set(sessionId, stream);

        // Reset per-session ephemeral state
        activeMessageByParticipant.clear();
        contentByMessageId.clear();
        participantNameById.clear();
        roundTypeBySession.set(sessionId, phase === 'critiquing' ? 'Critique' : phase === 'converging' ? 'Synthesis' : 'Draft');
        roundNumberBySession.set(sessionId, 0);
        
        // Start streaming events (async)
        (async () => {
          try {
            let event = await stream.next();
            while (event !== null) {
              try {
                const parsed = JSON.parse(event) as ForgeStreamEnvelope;
                const kind = parsed.type;
                const data = extractVariantData(parsed.data);

                if (kind === 'round_started') {
                  const round = data.round ?? 0;
                  const roundType = (data.round_type ?? data.roundType) as RoundType;
                  roundNumberBySession.set(sessionId, round);
                  roundTypeBySession.set(sessionId, roundType);

                  const nextPhase = phaseFromRoundType(roundType);
                  broadcast('forge:phaseChange', {
                    sessionId,
                    phase: nextPhase,
                    previousPhase: phase,
                  });
                }

                if (kind === 'participant_thinking') {
                  const participantId = String(data.participant_id ?? data.participantId ?? 'unknown');
                  const participantName = String(data.participant_name ?? data.participantName ?? participantId);
                  participantNameById.set(participantId, participantName);

                  const messageId = nextMessageId(sessionId, participantId);
                  activeMessageByParticipant.set(participantId, messageId);
                  contentByMessageId.set(messageId, '');

                  upsertStreamMessage({
                    sessionId,
                    messageId,
                    participantId,
                    participantName,
                    participantType: 'ai',
                    content: '',
                    timestamp: new Date().toISOString(),
                    type: 'thinking',
                    status: 'pending',
                  });
                }

                if (kind === 'content_delta') {
                  const participantId = String(data.participant_id ?? data.participantId ?? 'unknown');
                  const delta = String(data.delta ?? '');
                  const messageId =
                    activeMessageByParticipant.get(participantId) ??
                    (() => {
                      const id = nextMessageId(sessionId, participantId);
                      activeMessageByParticipant.set(participantId, id);
                      contentByMessageId.set(id, '');
                      return id;
                    })();

                  const prev = contentByMessageId.get(messageId) ?? '';
                  const next = prev + delta;
                  contentByMessageId.set(messageId, next);

                  upsertStreamMessage({
                    sessionId,
                    messageId,
                    participantId,
                    participantName: participantNameById.get(participantId) ?? participantId,
                    participantType: 'ai',
                    content: next,
                    contentDelta: delta,
                    timestamp: new Date().toISOString(),
                    type: messageTypeFromRoundType(roundTypeBySession.get(sessionId) ?? 'Draft'),
                    status: 'streaming',
                  });
                }

                if (kind === 'participant_complete') {
                  const participantId = String(data.participant_id ?? data.participantId ?? 'unknown');
                  const content = String(data.content ?? '');
                  const messageId =
                    activeMessageByParticipant.get(participantId) ?? nextMessageId(sessionId, participantId);

                  contentByMessageId.set(messageId, content);

                  upsertStreamMessage({
                    sessionId,
                    messageId,
                    participantId,
                    participantName: participantNameById.get(participantId) ?? participantId,
                    participantType: 'ai',
                    content,
                    timestamp: new Date().toISOString(),
                    type: messageTypeFromRoundType(roundTypeBySession.get(sessionId) ?? 'Draft'),
                    status: 'complete',
                  });
                }

                if (kind === 'round_complete') {
                  const roundNumber = Number(data.round ?? roundNumberBySession.get(sessionId) ?? 0);
                  broadcast('forge:roundComplete', {
                    sessionId,
                    roundNumber,
                    summary: '',
                  });
                }

                if (kind === 'participant_error') {
                  const error = String(data.error ?? 'Participant error');
                  broadcast('forge:error', { sessionId, error });
                }

                if (kind === 'error') {
                  const error = String(data.message ?? data.error ?? 'Deliberation error');
                  broadcast('forge:error', { sessionId, error });
                }
              } catch (e) {
                console.warn('[NATIVE] Failed to parse deliberation event:', e);
              }

              event = await stream.next();
            }
          } catch (error) {
            console.error('[NATIVE] Deliberation stream error:', error);
            broadcast('forge:error', {
              sessionId,
              error: error instanceof Error ? error.message : String(error),
            });
          } finally {
            deliberationStreams.delete(sessionId);
          }
        })();
        
        return true;
      } catch (error) {
        console.error('[NATIVE] NAPI startDeliberation failed:', error);
      }
    }
    
    console.log(`[NATIVE PLACEHOLDER] Starting deliberation for session ${sessionId} in phase ${phase}`);
    return true;
  },

  async stopDeliberation(sessionId: string): Promise<boolean> {
    if (napiModule) {
      try {
        console.log(`[NATIVE] Stopping deliberation for session ${sessionId}`);
        
        // Close the stream if it exists
        const stream = deliberationStreams.get(sessionId);
        if (stream) {
          stream.close();
          deliberationStreams.delete(sessionId);
        }
        
        return napiModule.stopDeliberation(sessionId);
      } catch (error) {
        console.error('[NATIVE] NAPI stopDeliberation failed:', error);
      }
    }
    
    console.log(`[NATIVE PLACEHOLDER] Stopping deliberation for session: ${sessionId}`);
    return true;
  },

  async submitForgeMessage(sessionId: string, _content: string, participantId: string): Promise<string> {
    console.log(`[NATIVE PLACEHOLDER] Submitting message to session ${sessionId} from ${participantId}`);
    return randomUUID();
  },

  async generateForgeOutput(request: ForgeOutputRequest): Promise<ForgeOutputResponse> {
    console.log(`[NATIVE PLACEHOLDER] Generating ${request.format} output for session ${request.sessionId}`);
    
    // Return placeholder content based on format
    const formatExtensions: Record<string, string> = {
      markdown: 'md',
      json: 'json',
      yaml: 'yaml',
      html: 'html',
      plain: 'txt',
      beads: 'yaml'
    };

    const placeholderContent = request.format === 'json' 
      ? JSON.stringify({ sessionId: request.sessionId, placeholder: true }, null, 2)
      : request.format === 'yaml' || request.format === 'beads'
      ? `session_id: ${request.sessionId}\nplaceholder: true\n`
      : `# Forge Session Output\n\nSession: ${request.sessionId}\n\nNo content generated yet.`;

    return {
      sessionId: request.sessionId,
      format: request.format,
      content: placeholderContent,
      filename: `forge-output-${request.sessionId.slice(0, 8)}.${formatExtensions[request.format]}`
    };
  }
};
