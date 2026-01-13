import { writable, derived } from 'svelte/store';
import type { ForgeSession, ForgeSessionState, SessionPhase } from '$lib/types/forge';

function createForgeSessionStore() {
  const initialState: ForgeSessionState = {
    activeSession: null,
    sessions: [],
    loading: false,
    error: null
  };

  const { subscribe, set, update } = writable<ForgeSessionState>(initialState);

  return {
    subscribe,

    async loadSessions(): Promise<void> {
      update(state => ({ ...state, loading: true, error: null }));
      
      try {
        // TODO: Replace with actual API call
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