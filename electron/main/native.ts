// NAPI-RS native bindings for Tachikoma Forge
// Uses the compiled Rust module for forge operations

import { randomUUID } from 'crypto';
import path from 'path';
import type { 
  MissionStatus, 
  SpecFile, 
  SpecMetadata, 
  TachikomaConfig,
  ForgeSessionRequest,
  ForgeSessionResponse,
  ForgePhase,
  ForgeOutputRequest,
  ForgeOutputResponse
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

try {
  // Dynamic import for native module
  napiModule = require('./native-bindings/index');
  console.log('[NATIVE] NAPI module loaded successfully');
} catch (error) {
  console.warn('[NATIVE] Failed to load NAPI module, using placeholders:', error);
}

// Convert between IPC types and NAPI types
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
      type: p.type,
      role: p.role,
      modelId: p.modelId,
      status: p.status
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
        const stream = napiModule.startDeliberation(sessionId);
        deliberationStreams.set(sessionId, stream);
        
        // Start streaming events (async)
        (async () => {
          try {
            let event = await stream.next();
            while (event !== null) {
              console.log('[NATIVE] Deliberation event:', event);
              // TODO: Emit event to renderer via IPC
              event = await stream.next();
            }
          } catch (error) {
            console.error('[NATIVE] Deliberation stream error:', error);
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
