import { derived, get } from 'svelte/store';
import { createPersistedStore } from '$lib/stores/persistedStore';
import type { ForgeSession, ForgeSessionState, SessionPhase, SessionDraft } from '$lib/types/forge';
import { forgeService } from '$lib/services/forgeService';

function createForgeSessionStore() {
  const initialState: ForgeSessionState = {
    activeSession: null,
    sessions: [],
    loading: false,
    error: null
  };

  type StoredForgeSession = Omit<ForgeSession, 'createdAt' | 'updatedAt'> & {
    createdAt: string;
    updatedAt: string;
  };

  type StoredForgeSessionState = Omit<ForgeSessionState, 'sessions' | 'activeSession' | 'loading' | 'error'> & {
    sessions: StoredForgeSession[];
    activeSession: StoredForgeSession | null;
    loading?: boolean;
    error?: string | null;
  };

  function toStoredSession(session: ForgeSession): StoredForgeSession {
    return {
      ...session,
      createdAt: session.createdAt.toISOString(),
      updatedAt: session.updatedAt.toISOString()
    };
  }

  function fromStoredSession(session: StoredForgeSession): ForgeSession {
    return {
      ...session,
      createdAt: new Date(session.createdAt),
      updatedAt: new Date(session.updatedAt)
    };
  }

  function defaultAvatarForParticipant(p: { id: string; name: string; type: 'human' | 'ai' }): string | undefined {
    const id = (p.id || '').toLowerCase();
    const name = (p.name || '').toLowerCase();
    const icon = (filename: string, emoji?: string) => `asset:/icons/${filename}${emoji ? `|${emoji}` : ''}`;

    if (id.includes('fuchikoma-blue') || name.includes('fuchikoma blue')) return icon('Iconfactory-Ghost-In-The-Shell-Fuchikoma-Blue.32.png', '🕷️');
    if (id.includes('fuchikoma-red') || name.includes('fuchikoma red')) return icon('Iconfactory-Ghost-In-The-Shell-Fuchikoma-Red.32.png', '🔴');
    if (id.includes('fuchikoma-purple') || name.includes('fuchikoma purple')) return icon('Iconfactory-Ghost-In-The-Shell-Fuchikoma-Purple.32.png', '🟣');
    if (id.includes('arakone') || name.includes('arakone')) return icon('Iconfactory-Ghost-In-The-Shell-Arakone-Unit.32.png', '🦂');
    if (name.includes('laughing man')) return '🌀';
    if (name.includes('puppet master')) return '🧬';

    if (name.includes('kusanagi') || name.includes('motoko')) return icon('Iconfactory-Ghost-In-The-Shell-Motoko-Kusanagi.32.png', '🕶️');
    if (name.includes('batou') || name.includes('bateau')) return icon('Iconfactory-Ghost-In-The-Shell-Bateau.32.png', '🦾');
    if (name.includes('togusa')) return icon('Iconfactory-Ghost-In-The-Shell-Togusa.32.png', '🕵️');
    if (name.includes('ishikawa')) return icon('Iconfactory-Ghost-In-The-Shell-Ishikawa.32.png', '📡');

    if (p.type === 'ai') return icon('Iconfactory-Ghost-In-The-Shell-Fuchikoma-Gray.32.png', '🤖');
    return undefined;
  }

  function normalizeSession(session: ForgeSession): ForgeSession {
    return {
      ...session,
      participants: (session.participants || []).map(p => ({
        ...p,
        avatar: p.avatar || defaultAvatarForParticipant({ id: p.id, name: p.name, type: p.type }),
      })),
    };
  }

  const store = createPersistedStore<ForgeSessionState>(initialState, {
    key: 'forgeSessionState',
    version: 1,
    serialize: (value) => {
      const stored: StoredForgeSessionState = {
        activeSession: value.activeSession ? toStoredSession(value.activeSession) : null,
        sessions: value.sessions.map(toStoredSession),
        // Don't persist transient UI state
        loading: false,
        error: null
      };
      return JSON.stringify(stored);
    },
    deserialize: (raw) => {
      const parsed = JSON.parse(raw) as StoredForgeSessionState;
      return {
        activeSession: parsed.activeSession ? normalizeSession(fromStoredSession(parsed.activeSession)) : null,
        sessions: Array.isArray(parsed.sessions) ? parsed.sessions.map(fromStoredSession).map(normalizeSession) : [],
        loading: false,
        error: null
      };
    }
  });

  const { subscribe, set, update } = store;

  return {
    subscribe,

    async loadSessions(): Promise<void> {
      // If we already have persisted sessions, don't clobber them with mock data.
      const existing = get(store).sessions;
      if (existing.length > 0) {
        update(state => ({ ...state, loading: false, error: null }));
        return;
      }

      update(state => ({ ...state, loading: true, error: null }));
      
      try {
        // Try to load from backend first
        const backendSessions = await forgeService.listSessions();
        
        if (backendSessions.length > 0) {
          update(state => ({
            ...state,
            sessions: backendSessions,
            loading: false
          }));
          return;
        }
        
        // Fallback to mock data for demo purposes
        const mockSessions: ForgeSession[] = [
          {
            id: 'session-1',
            name: 'Product Strategy Session',
            goal: 'Define Q4 product roadmap priorities',
            phase: 'configuring',
            participants: [
              {
                id: 'p1',
                name: 'Alice Product',
                type: 'human',
                role: 'Product Manager',
                status: 'active',
                avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Motoko-Kusanagi.32.png|🕶️',
              },
              {
                id: 'p2',
                name: 'Bob Engineer',
                type: 'human',
                role: 'Tech Lead',
                status: 'active',
                avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Bateau.32.png|🦾',
              },
              {
                id: 'p3',
                name: 'Claude Analyst',
                type: 'ai',
                role: 'Business Analyst',
                status: 'active',
                modelId: 'claude-sonnet-4-20250514',
                avatar: 'asset:/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Blue.32.png|🕷️',
              }
            ],
            oracle: {
              id: 'oracle-1',
              name: 'Strategic Oracle',
              type: 'gpt-4',
              config: { temperature: 0.7 }
            },
            rounds: [],
            hasResults: false,
            createdAt: new Date('2024-01-15T10:00:00Z'),
            updatedAt: new Date('2024-01-15T10:30:00Z')
          }
        ];

        update(state => ({
          ...state,
          sessions: mockSessions,
          loading: false
        }));
      } catch (error) {
        update(state => ({
          ...state,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to load sessions'
        }));
      }
    },

    setActiveSession(sessionId: string): void {
      update(state => {
        const session = state.sessions.find(s => s.id === sessionId);
        return {
          ...state,
          activeSession: session || null
        };
      });
    },

    clearActiveSession(): void {
      update(state => ({ ...state, activeSession: null }));
    },

    updateSessionPhase(sessionId: string, phase: SessionPhase): void {
      update(state => {
        const sessions = state.sessions.map(session =>
          session.id === sessionId
            ? { ...session, phase, updatedAt: new Date() }
            : session
        );
        
        const activeSession = state.activeSession?.id === sessionId
          ? sessions.find(s => s.id === sessionId) || null
          : state.activeSession;

        return {
          ...state,
          sessions,
          activeSession
        };
      });
    },

    updateParticipantModel(sessionId: string, participantId: string, modelId: string | undefined): void {
      update(state => {
        const patchSession = (session: ForgeSession): ForgeSession => {
          if (session.id !== sessionId) return session;
          return {
            ...session,
            participants: session.participants.map(p =>
              p.id === participantId ? { ...p, modelId } : p
            ),
            updatedAt: new Date(),
          };
        };

        const sessions = state.sessions.map(patchSession);
        const activeSession = state.activeSession ? patchSession(state.activeSession) : null;

        return {
          ...state,
          sessions,
          activeSession,
        };
      });
    },

    clearError(): void {
      update(state => ({ ...state, error: null }));
    },

    async saveDraft(draft: SessionDraft): Promise<string> {
      try {
        // TODO: Replace with actual API call
        const draftId = `draft-${Date.now()}`;
        
        // For now, just simulate saving
        console.log('Saving draft:', draft);
        
        return draftId;
      } catch (error) {
        throw new Error(error instanceof Error ? error.message : 'Failed to save draft');
      }
    },

    async createSession(draft: SessionDraft): Promise<string> {
      update(state => ({ ...state, loading: true, error: null }));
      
      try {
        // Use forgeService to create session (handles IPC vs mock)
        const newSession = await forgeService.createSession({
          name: draft.name,
          goal: draft.goal,
          participants: draft.participants,
          oracle: draft.oracle,
          config: draft.config
        });

        update(state => ({
          ...state,
          sessions: [...state.sessions, newSession],
          loading: false
        }));

        return newSession.id;
      } catch (error) {
        update(state => ({
          ...state,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to create session'
        }));
        throw error;
      }
    },

    async updateSession(sessionId: string, draft: SessionDraft): Promise<string> {
      update(state => ({ ...state, loading: true, error: null }));
      
      try {
        update(state => {
          const sessions = state.sessions.map(session =>
            session.id === sessionId
              ? {
                  ...session,
                  name: draft.name,
                  goal: draft.goal,
                  participants: draft.participants,
                  oracle: draft.oracle,
                  config: draft.config,
                  updatedAt: new Date()
                }
              : session
          );
          
          const activeSession = state.activeSession?.id === sessionId
            ? sessions.find(s => s.id === sessionId) || null
            : state.activeSession;

          return {
            ...state,
            sessions,
            activeSession,
            loading: false
          };
        });

        return sessionId;
      } catch (error) {
        update(state => ({
          ...state,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to update session'
        }));
        throw error;
      }
    },

    reset(): void {
      set(initialState);
    }
  };
}

export const forgeSessionStore = createForgeSessionStore();

// Derived stores
export const activeSession = derived(
  forgeSessionStore,
  $store => $store.activeSession
);

export const sessions = derived(
  forgeSessionStore,
  $store => $store.sessions
);

export const sessionLoading = derived(
  forgeSessionStore,
  $store => $store.loading
);

export const sessionError = derived(
  forgeSessionStore,
  $store => $store.error
);