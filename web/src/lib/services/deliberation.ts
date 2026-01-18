/**
 * Deliberation Engine Service
 * 
 * Orchestrates multi-model deliberation sessions.
 * Integrates with forgeService for IPC communication, with fallback to mock mode.
 */

import { writable, get } from 'svelte/store';
import type { ForgeSession, Participant } from '$lib/types/forge';
import { forgeService } from './forgeService';

export interface DeliberationState {
  isRunning: boolean;
  currentRound: number;
  activeParticipantId: string | null;
  pendingContributions: Map<string, string>; // participantId -> partial content
  error: string | null;
  useMockMode: boolean;
}

export interface DeliberationMessage {
  id: string;
  participantId: string;
  participantName: string;
  participantType: 'human' | 'ai';
  content: string;
  timestamp: Date;
  type: 'proposal' | 'critique' | 'synthesis' | 'thinking';
  status: 'pending' | 'streaming' | 'complete';
}

function createDeliberationStore() {
  const state = writable<DeliberationState>({
    isRunning: false,
    currentRound: 0,
    activeParticipantId: null,
    pendingContributions: new Map(),
    error: null,
    useMockMode: true
  });

  const messages = writable<DeliberationMessage[]>([]);
  // Used to safely cancel in-flight mock runs when the user hits Stop or starts a new run.
  let runToken = 0;

  // Subscribe to forge service state to detect connection mode
  forgeService.state.subscribe(forgeState => {
    state.update(s => ({
      ...s,
      useMockMode: forgeState.useMockMode,
      // When connected to IPC, reflect backend state for a predictable UX
      ...(forgeState.useMockMode
        ? {}
        : {
            isRunning: forgeState.isDeliberating,
            error: forgeState.error,
          }),
    }));
  });

  // Subscribe to forge service messages for real-time updates
  forgeService.messages.subscribe(forgeMessages => {
    // Convert forge messages to deliberation messages
    const converted: DeliberationMessage[] = forgeMessages.map(fm => ({
      id: fm.messageId,
      participantId: fm.participantId,
      participantName: fm.participantName,
      participantType: fm.participantType,
      content: fm.content,
      timestamp: new Date(fm.timestamp),
      type: fm.type,
      status: fm.status
    }));
    
    // Only update if we have real messages from IPC
    if (converted.length > 0 && !get(state).useMockMode) {
      messages.set(converted);

      // Best-effort "who is active" indicator from streaming/pending messages
      const active =
        [...converted]
          .reverse()
          .find(m => m.status === 'streaming' || m.status === 'pending')?.participantId ?? null;
      state.update(s => ({ ...s, activeParticipantId: active }));
    }
  });

  // Mock responses for different participant types
  const mockResponses: Record<string, string[]> = {
    'drafting': [
      "Based on the session goal, I propose we structure this into three main components:\n\n1. **Core Architecture** - Define the fundamental building blocks\n2. **Interface Layer** - How users interact with the system\n3. **Data Flow** - Information movement between components\n\nThis approach allows for modular development and clear separation of concerns.",
      
      "I'll approach this from a user-centric perspective:\n\n**User Stories:**\n- As a developer, I need clear APIs to integrate quickly\n- As an operator, I want visibility into system health\n- As an admin, I require granular access controls\n\n**Key Considerations:**\n- Performance at scale\n- Security by default\n- Developer experience",
      
      "Looking at this systematically, the goal suggests we need:\n\n```\nInput → Processing → Output\n         ↓\n    Validation\n         ↓\n    Persistence\n```\n\nI recommend starting with the happy path, then adding error handling and edge cases iteratively."
    ],
    'critiquing': [
      "I see some strong elements in the proposals. However:\n\n⚠️ **Concern**: The modular approach may introduce unnecessary complexity for the MVP\n\n💡 **Suggestion**: Start with a monolithic core, then extract services as needed\n\n✓ **Agreement**: The user-centric framing is valuable",
      
      "Building on the previous contributions:\n\n**Strengths:**\n- Clear structure\n- Good separation of concerns\n\n**Gaps identified:**\n- No mention of testing strategy\n- Security considerations need elaboration\n- What about backwards compatibility?",
      
      "The systematic approach is sound, but I'd challenge:\n\n1. The linear flow might not capture real-world async requirements\n2. We should consider event-driven patterns\n3. The persistence layer needs more detail - what's our consistency model?"
    ],
    'synthesis': [
      "**Synthesizing the discussion:**\n\nWe've converged on a layered architecture with:\n- User-centric design principles\n- Modular but pragmatic structure\n- Event-driven core with sync interfaces\n\n**Action Items:**\n1. Define core data models\n2. Draft API specifications\n3. Create testing framework\n\n**Open Questions:**\n- Consistency vs availability trade-offs\n- MVP scope boundaries"
    ]
  };

  function isCancelled(token: number) {
    return token !== runToken || !get(state).isRunning;
  }

  function planPhases(startPhase: string): Array<'drafting' | 'critiquing' | 'synthesis'> {
    // Keep the UI simple: when you hit Start, models run through a coherent mini-pipeline.
    // (Backend mode emits real rounds; mock mode mirrors the intent.)
    if (startPhase === 'critiquing') return ['critiquing', 'synthesis'];
    if (startPhase === 'converging') return ['synthesis'];
    // default: drafting
    return ['drafting', 'critiquing', 'synthesis'];
  }

  function messageTypeForPhase(phase: string): DeliberationMessage['type'] {
    if (phase === 'critiquing') return 'critique';
    if (phase === 'synthesis') return 'synthesis';
    return 'proposal';
  }

  function latestCompleteOfType(type: DeliberationMessage['type']): DeliberationMessage | null {
    const current = get(messages);
    return (
      [...current]
        .reverse()
        .find(m => m.status === 'complete' && m.type === type) ?? null
    );
  }

  async function startDeliberation(session: ForgeSession) {
    // Cancel any in-flight mock run, then start a fresh run.
    runToken += 1;
    const token = runToken;

    state.update(s => ({ ...s, isRunning: true, error: null, currentRound: 1, activeParticipantId: null }));
    
    const currentState = get(state);
    
    // Try to use real IPC first (only if we think IPC is available)
    if (!currentState.useMockMode) {
      try {
        const phase = session.phase as import('$lib/ipc/types').ForgePhase;
        const success = await forgeService.startDeliberation(session.id, phase);
        if (success) {
          // Real deliberation started - messages will come via events
          return;
        }
        // If success is false, fall through to mock mode
      } catch (error) {
        console.warn('Failed to start real deliberation, falling back to mock:', error);
      }
      // Update state to use mock mode going forward
      state.update(s => ({ ...s, useMockMode: true }));
    }
    
    // Run mock mode - generate simulated AI responses
    messages.set([]);
    
    const aiParticipants = session.participants.filter(p => p.type === 'ai');
    const startPhase = String(session.phase || 'drafting');
    const phases = planPhases(startPhase);

    // Simulate a structured multi-phase run so models "talk" without extra user clicks.
    for (let phaseIdx = 0; phaseIdx < phases.length; phaseIdx++) {
      const phase = phases[phaseIdx];
      state.update(s => ({ ...s, currentRound: phaseIdx + 1 }));

      const phaseResponses = mockResponses[phase] || mockResponses['drafting'];

      for (let i = 0; i < aiParticipants.length; i++) {
        if (isCancelled(token)) return;
        const participant = aiParticipants[i];

        // Set active participant
        state.update(s => ({ ...s, activeParticipantId: participant.id }));

        // Add "thinking" message
        const thinkingMsg: DeliberationMessage = {
          id: `msg-${Date.now()}-${phaseIdx}-${i}`,
          participantId: participant.id,
          participantName: participant.name,
          participantType: participant.type,
          content: '',
          timestamp: new Date(),
          type: 'thinking',
          status: 'pending'
        };
        messages.update(msgs => [...msgs, thinkingMsg]);

        // Simulate thinking delay
        await delay(650 + Math.random() * 900);
        if (isCancelled(token)) return;

        // Stream the response (best-effort: reference prior content to feel "interactive")
        let response = phaseResponses[i % phaseResponses.length];
        if (phase === 'critiquing') {
          const lastProposal = latestCompleteOfType('proposal');
          if (lastProposal?.content) {
            response =
              `**Critiquing (context: ${lastProposal.participantName})**\n\n` +
              response +
              `\n\n---\n\n**Snippet I’m responding to:**\n> ${lastProposal.content.slice(0, 220).replace(/\n/g, '\n> ')}`;
          }
        } else if (phase === 'synthesis') {
          const lastCritique = latestCompleteOfType('critique');
          if (lastCritique?.content) {
            response =
              response +
              `\n\n---\n\n**Incorporating critique from ${lastCritique.participantName}:**\n> ${lastCritique.content.slice(0, 220).replace(/\n/g, '\n> ')}`;
          }
        }

        await streamResponse(thinkingMsg.id, response, participant, messageTypeForPhase(phase));
      }
    }

    if (!isCancelled(token)) {
      state.update(s => ({ ...s, isRunning: false, activeParticipantId: null }));
    }
  }

  async function streamResponse(
    messageId: string,
    fullResponse: string,
    _participant: Participant,
    finalType: DeliberationMessage['type']
  ) {
    const words = fullResponse.split(' ');
    let current = '';

    messages.update(msgs => 
      msgs.map(m => m.id === messageId 
        ? { ...m, type: finalType, status: 'streaming' as const } 
        : m
      )
    );

    for (let i = 0; i < words.length; i++) {
      current += (i > 0 ? ' ' : '') + words[i];
      
      messages.update(msgs => 
        msgs.map(m => m.id === messageId ? { ...m, content: current } : m)
      );

      // Variable delay for natural feel
      await delay(20 + Math.random() * 40);
    }

    // Mark as complete
    messages.update(msgs => 
      msgs.map(m => m.id === messageId ? { ...m, status: 'complete' as const } : m)
    );
  }

  async function stopDeliberation(sessionId?: string) {
    const currentState = get(state);
    
    if (!currentState.useMockMode && sessionId) {
      try {
        await forgeService.stopDeliberation(sessionId);
      } catch (error) {
        console.warn('Failed to stop real deliberation:', error);
      }
    }
    
    // Cancel any in-flight mock work.
    runToken += 1;
    state.update(s => ({ ...s, isRunning: false, activeParticipantId: null }));
  }

  function clearMessages() {
    messages.set([]);
    forgeService.clearMessages();
  }

  return {
    state: { subscribe: state.subscribe },
    messages: { subscribe: messages.subscribe },
    startDeliberation,
    stopDeliberation,
    clearMessages
  };
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export const deliberationStore = createDeliberationStore();
