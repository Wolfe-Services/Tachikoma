/**
 * Forge Service - Unified API for forge operations
 * 
 * This service provides a clean interface for forge functionality,
 * using IPC when available (in Electron) or mock mode for development.
 */

import { writable, get } from 'svelte/store';
import { ipc } from '$lib/ipc/client';
import { isIpcAvailable } from '$lib/ipc/errors';
import type { 
  ForgeSessionRequest, 
  ForgeSessionResponse, 
  ForgePhase,
  ForgeOutputRequest,
  ForgeOutputResponse,
  ForgeMessageEvent
} from '$lib/ipc/types';
import type { ForgeSession, Participant, SessionPhase } from '$lib/types/forge';

// Re-export types for convenience
export type { ForgeOutputFormat, ForgeOutputRequest, ForgeOutputResponse } from '$lib/ipc/types';

// Service state
export interface ForgeServiceState {
  isConnected: boolean;
  useMockMode: boolean;
  activeSessionId: string | null;
  isDeliberating: boolean;
  currentPhase: ForgePhase;
  error: string | null;
}

const initialState: ForgeServiceState = {
  isConnected: false,
  useMockMode: true,
  activeSessionId: null,
  isDeliberating: false,
  currentPhase: 'idle',
  error: null
};

function createForgeService() {
  const state = writable<ForgeServiceState>(initialState);
  const messages = writable<ForgeMessageEvent[]>([]);
  
  // Check IPC availability on init
  const ipcAvailable = isIpcAvailable();
  if (ipcAvailable) {
    state.update(s => ({ ...s, isConnected: true, useMockMode: false }));
    
    // Set up event listeners for streaming messages
    setupEventListeners();
  }

  function setupEventListeners() {
    // Listen for forge events from main process
    ipc.on('forge:message', (event) => {
      messages.update(msgs => {
        const existingIndex = msgs.findIndex(m => m.messageId === event.messageId);
        if (existingIndex >= 0) {
          // Update existing message (for streaming)
          const updated = [...msgs];
          updated[existingIndex] = event;
          return updated;
        } else {
          // Add new message
          return [...msgs, event];
        }
      });
    });

    ipc.on('forge:phaseChange', (event) => {
      state.update(s => ({ ...s, currentPhase: event.phase }));
    });

    ipc.on('forge:error', (event) => {
      state.update(s => ({ ...s, error: event.error, isDeliberating: false }));
    });
  }

  // Convert frontend types to IPC request format
  function toSessionRequest(session: Partial<ForgeSession>): ForgeSessionRequest {
    return {
      name: session.name || 'Untitled Session',
      goal: session.goal || '',
      participants: (session.participants || []).map(p => ({
        id: p.id,
        name: p.name,
        type: p.type,
        role: p.role,
        modelId: p.type === 'ai' ? getModelId(p) : undefined,
        status: p.status
      })),
      oracle: session.oracle ? {
        id: session.oracle.id,
        name: session.oracle.name,
        modelId: session.oracle.type,
        config: session.oracle.config
      } : null,
      config: {
        maxRounds: session.config?.maxRounds || 5,
        convergenceThreshold: session.config?.convergenceThreshold || 0.8,
        roundTimeoutMs: (session.config?.timeoutMinutes || 60) * 60000,
        allowHumanIntervention: session.config?.allowHumanIntervention ?? true
      }
    };
  }

  function getModelId(participant: Participant): string {
    // Map participant names/types to model IDs
    const modelMap: Record<string, string> = {
      'claude': 'claude-3-5-sonnet-20241022',
      'gpt4': 'gpt-4-turbo',
      'gpt-4': 'gpt-4-turbo',
      'gemini': 'gemini-pro',
      'ollama': 'ollama/llama2'
    };
    const key = participant.name.toLowerCase();
    return modelMap[key] || 'claude-3-5-sonnet-20241022';
  }

  // Convert IPC response to frontend session format
  function fromSessionResponse(response: ForgeSessionResponse): ForgeSession {
    return {
      id: response.id,
      name: response.name,
      goal: response.goal,
      phase: response.phase as SessionPhase,
      participants: response.participants.map(p => ({
        id: p.id,
        name: p.name,
        type: p.type,
        role: p.role,
        status: p.status
      })),
      oracle: response.oracle ? {
        id: response.oracle.id,
        name: response.oracle.name,
        type: response.oracle.modelId,
        config: response.oracle.config as Record<string, any>
      } : null,
      config: {
        maxRounds: response.config.maxRounds,
        convergenceThreshold: response.config.convergenceThreshold,
        allowHumanIntervention: response.config.allowHumanIntervention,
        autoSaveInterval: 30000,
        timeoutMinutes: Math.floor(response.config.roundTimeoutMs / 60000)
      },
      rounds: [],
      hasResults: response.roundCount > 0,
      createdAt: new Date(response.createdAt),
      updatedAt: new Date(response.updatedAt)
    };
  }

  return {
    state: { subscribe: state.subscribe },
    messages: { subscribe: messages.subscribe },

    /**
     * Create a new forge session
     */
    async createSession(sessionData: Partial<ForgeSession>): Promise<ForgeSession> {
      const currentState = get(state);
      
      if (!currentState.useMockMode) {
        try {
          const request = toSessionRequest(sessionData);
          const response = await ipc.invoke('forge:createSession', request);
          const session = fromSessionResponse(response);
          state.update(s => ({ ...s, activeSessionId: session.id, error: null }));
          return session;
        } catch (error) {
          console.warn('IPC createSession failed, falling back to mock mode:', error);
          state.update(s => ({ ...s, useMockMode: true }));
        }
      }
      
      // Mock mode
      const mockSession: ForgeSession = {
        id: `session-${Date.now()}`,
        name: sessionData.name || 'Untitled Session',
        goal: sessionData.goal || '',
        phase: 'configuring',
        participants: sessionData.participants || [],
        oracle: sessionData.oracle || null,
        config: sessionData.config,
        rounds: [],
        hasResults: false,
        createdAt: new Date(),
        updatedAt: new Date()
      };
      state.update(s => ({ ...s, activeSessionId: mockSession.id }));
      return mockSession;
    },

    /**
     * Start deliberation for a session
     * Returns true only if real IPC succeeded; returns false for mock mode
     * so that callers can run their own mock logic.
     */
    async startDeliberation(sessionId: string, phase: ForgePhase = 'drafting'): Promise<boolean> {
      const currentState = get(state);
      
      if (!currentState.useMockMode) {
        try {
          const result = await ipc.invoke('forge:startDeliberation', { sessionId, phase });
          if (result.success) {
            state.update(s => ({ 
              ...s, 
              isDeliberating: true, 
              currentPhase: phase,
              error: null 
            }));
            return true; // Real IPC succeeded
          }
        } catch (error) {
          console.warn('IPC startDeliberation failed:', error);
        }
      }
      
      // Mock mode - update state but return false so caller runs mock logic
      state.update(s => ({ 
        ...s, 
        isDeliberating: true, 
        currentPhase: phase 
      }));
      return false; // Signal to caller to use mock mode
    },

    /**
     * Stop deliberation for a session
     */
    async stopDeliberation(sessionId: string): Promise<boolean> {
      const currentState = get(state);
      
      if (!currentState.useMockMode) {
        try {
          const result = await ipc.invoke('forge:stopDeliberation', { sessionId });
          if (result.success) {
            state.update(s => ({ ...s, isDeliberating: false }));
          }
          return result.success;
        } catch (error) {
          console.warn('IPC stopDeliberation failed:', error);
        }
      }
      
      state.update(s => ({ ...s, isDeliberating: false }));
      return true;
    },

    /**
     * Submit a human message to the deliberation
     */
    async submitMessage(sessionId: string, content: string, participantId: string): Promise<string | null> {
      const currentState = get(state);
      
      if (!currentState.useMockMode) {
        try {
          const result = await ipc.invoke('forge:submitMessage', { 
            sessionId, 
            content, 
            participantId 
          });
          return result.messageId;
        } catch (error) {
          console.warn('IPC submitMessage failed:', error);
        }
      }
      
      // Mock mode
      const messageId = `msg-${Date.now()}`;
      const mockMessage: ForgeMessageEvent = {
        sessionId,
        messageId,
        participantId,
        participantName: 'You',
        participantType: 'human',
        content,
        timestamp: new Date().toISOString(),
        type: 'proposal',
        status: 'complete'
      };
      messages.update(msgs => [...msgs, mockMessage]);
      return messageId;
    },

    /**
     * Generate output in the specified format
     */
    async generateOutput(request: ForgeOutputRequest): Promise<ForgeOutputResponse> {
      const currentState = get(state);
      
      if (!currentState.useMockMode) {
        try {
          return await ipc.invoke('forge:generateOutput', request);
        } catch (error) {
          console.warn('IPC generateOutput failed, using mock:', error);
        }
      }
      
      // Mock output generation
      const mockContent = generateMockOutput(request);
      return {
        sessionId: request.sessionId,
        format: request.format,
        content: mockContent,
        filename: `forge-output.${getExtension(request.format)}`
      };
    },

    /**
     * List all forge sessions
     */
    async listSessions(): Promise<ForgeSession[]> {
      const currentState = get(state);
      
      if (!currentState.useMockMode) {
        try {
          const sessions = await ipc.invoke('forge:listSessions', {});
          return sessions.map(fromSessionResponse);
        } catch (error) {
          console.warn('IPC listSessions failed:', error);
        }
      }
      
      return [];
    },

    /**
     * Clear messages for a fresh start
     */
    clearMessages() {
      messages.set([]);
    },

    /**
     * Clear any error state
     */
    clearError() {
      state.update(s => ({ ...s, error: null }));
    },

    /**
     * Reset the service state
     */
    reset() {
      state.set(initialState);
      messages.set([]);
    }
  };
}

function getExtension(format: string): string {
  const extensions: Record<string, string> = {
    markdown: 'md',
    json: 'json',
    yaml: 'yaml',
    html: 'html',
    plain: 'txt',
    beads: 'yaml'
  };
  return extensions[format] || 'txt';
}

function generateMockOutput(request: ForgeOutputRequest): string {
  const { format, sessionId } = request;
  
  switch (format) {
    case 'json':
      return JSON.stringify({
        sessionId,
        title: 'Mock Forge Output',
        summary: 'This is a mock output generated for development.',
        decisions: [],
        metadata: {
          generated: new Date().toISOString()
        }
      }, null, 2);
    
    case 'yaml':
    case 'beads':
      return `# Forge Session Output
session_id: ${sessionId}
title: Mock Forge Output
summary: This is a mock output generated for development.
decisions: []
metadata:
  generated: ${new Date().toISOString()}
`;
    
    case 'html':
      return `<!DOCTYPE html>
<html>
<head>
  <title>Forge Output</title>
  <style>
    body { font-family: system-ui; max-width: 800px; margin: 2rem auto; padding: 1rem; }
    h1 { color: #4ecdc4; }
  </style>
</head>
<body>
  <h1>Forge Session Output</h1>
  <p>Session: ${sessionId}</p>
  <p>This is a mock output generated for development.</p>
</body>
</html>`;
    
    case 'markdown':
    default:
      return `# Forge Session Output

**Session ID:** ${sessionId}

## Summary

This is a mock output generated for development. 
When connected to the backend, this will contain the actual deliberation results.

## Decisions

_No decisions recorded yet._

---

*Generated: ${new Date().toISOString()}*
`;
  }
}

// Export singleton instance
export const forgeService = createForgeService();
