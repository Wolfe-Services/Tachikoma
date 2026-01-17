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
        activeSession: parsed.activeSession ? fromStoredSession(parsed.activeSession) : null,
        sessions: Array.isArray(parsed.sessions) ? parsed.sessions.map(fromStoredSession) : [],
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
                status: 'active'
              },
              {
                id: 'p2',
                name: 'Bob Engineer',
                type: 'human',
                role: 'Tech Lead',
                status: 'active'
              },
              {
                id: 'p3',
                name: 'Claude Analyst',
                type: 'ai',
                role: 'Business Analyst',
                status: 'active'
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