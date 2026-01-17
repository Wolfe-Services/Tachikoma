/**
 * Goal Refinement Service
 * 
 * Provides AI-assisted goal refinement through interactive Q&A.
 * Currently uses mock responses - will be connected to real LLM APIs via Tauri IPC.
 */

import { writable, get } from 'svelte/store';

export interface RefinementMessage {
  id: string;
  role: 'assistant' | 'user';
  content: string;
  timestamp: Date;
  status: 'pending' | 'streaming' | 'complete';
}

export interface StructuredGoal {
  objective: string;
  context: string;
  constraints: string;
  successCriteria: string;
}

export interface RefinementState {
  isActive: boolean;
  isStreaming: boolean;
  currentQuestionIndex: number;
  structuredGoal: StructuredGoal;
  error: string | null;
}

// Clarifying questions to guide goal refinement
const clarifyingQuestions = [
  "What specific problem are you trying to solve with this deliberation session?",
  "Who are the stakeholders involved, and what are their primary concerns?",
  "What constraints should the Think Tank consider when deliberating?",
  "What does success look like? How will you measure it?"
];

// Mock synthesis responses based on collected answers
function generateMockSynthesis(answers: string[]): StructuredGoal {
  // Extract key themes from answers
  const hasStakeholders = answers.some(a => 
    a.toLowerCase().includes('team') || 
    a.toLowerCase().includes('user') || 
    a.toLowerCase().includes('customer')
  );
  
  const hasConstraints = answers.some(a => 
    a.toLowerCase().includes('time') || 
    a.toLowerCase().includes('budget') || 
    a.toLowerCase().includes('resource')
  );

  return {
    objective: answers[0] 
      ? `Define a clear approach to: ${answers[0].slice(0, 150)}${answers[0].length > 150 ? '...' : ''}`
      : 'Develop a comprehensive strategy for the stated problem.',
    context: hasStakeholders && answers[1]
      ? `Key stakeholders: ${answers[1].slice(0, 200)}${answers[1].length > 200 ? '...' : ''}`
      : 'Multiple stakeholders with varying priorities will be considered.',
    constraints: hasConstraints && answers[2]
      ? `Operating within: ${answers[2].slice(0, 200)}${answers[2].length > 200 ? '...' : ''}`
      : 'Standard resource and timeline constraints apply.',
    successCriteria: answers[3]
      ? `Success measured by: ${answers[3].slice(0, 200)}${answers[3].length > 200 ? '...' : ''}`
      : 'Clear deliverables and actionable recommendations.'
  };
}

function structuredGoalToMarkdown(goal: StructuredGoal): string {
  return `## Objective
${goal.objective}

## Context
${goal.context}

## Constraints
${goal.constraints}

## Success Criteria
${goal.successCriteria}`;
}

function createGoalRefinementStore() {
  const state = writable<RefinementState>({
    isActive: false,
    isStreaming: false,
    currentQuestionIndex: 0,
    structuredGoal: {
      objective: '',
      context: '',
      constraints: '',
      successCriteria: ''
    },
    error: null
  });

  const messages = writable<RefinementMessage[]>([]);
  const userAnswers = writable<string[]>([]);

  function startRefinement() {
    state.update(s => ({
      ...s,
      isActive: true,
      currentQuestionIndex: 0,
      error: null
    }));
    messages.set([]);
    userAnswers.set([]);
    
    // Ask first question
    askNextQuestion(0);
  }

  async function askNextQuestion(index: number) {
    if (index >= clarifyingQuestions.length) {
      // All questions answered, synthesize
      await synthesizeGoal();
      return;
    }

    const messageId = `msg-${Date.now()}`;
    const question = clarifyingQuestions[index];

    // Add thinking state
    messages.update(msgs => [...msgs, {
      id: messageId,
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      status: 'pending'
    }]);

    // Simulate thinking delay
    await delay(400 + Math.random() * 400);

    // Stream the question
    state.update(s => ({ ...s, isStreaming: true }));
    await streamMessage(messageId, question);
    state.update(s => ({ ...s, isStreaming: false, currentQuestionIndex: index }));
  }

  async function submitAnswer(answer: string) {
    const currentState = get(state);
    const currentIndex = currentState.currentQuestionIndex;

    // Add user message
    messages.update(msgs => [...msgs, {
      id: `user-${Date.now()}`,
      role: 'user',
      content: answer,
      timestamp: new Date(),
      status: 'complete'
    }]);

    // Store answer
    userAnswers.update(answers => {
      const newAnswers = [...answers];
      newAnswers[currentIndex] = answer;
      return newAnswers;
    });

    // Move to next question
    await delay(300);
    await askNextQuestion(currentIndex + 1);
  }

  async function synthesizeGoal() {
    const answers = get(userAnswers);
    const messageId = `synthesis-${Date.now()}`;

    // Add synthesis message
    messages.update(msgs => [...msgs, {
      id: messageId,
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      status: 'pending'
    }]);

    await delay(600);

    // Generate structured goal
    const structured = generateMockSynthesis(answers);
    state.update(s => ({ ...s, structuredGoal: structured, isStreaming: true }));

    // Stream synthesis message
    const synthesisIntro = "Great! Based on your answers, I've structured your goal into a clear format:\n\n" + 
      structuredGoalToMarkdown(structured) + 
      "\n\n*Click \"Apply Suggestions\" to use this as your goal, or continue refining.*";

    await streamMessage(messageId, synthesisIntro);
    state.update(s => ({ ...s, isStreaming: false }));
  }

  async function streamMessage(messageId: string, fullContent: string) {
    const words = fullContent.split(' ');
    let current = '';

    messages.update(msgs => 
      msgs.map(m => m.id === messageId 
        ? { ...m, status: 'streaming' as const } 
        : m
      )
    );

    for (let i = 0; i < words.length; i++) {
      current += (i > 0 ? ' ' : '') + words[i];
      
      messages.update(msgs => 
        msgs.map(m => m.id === messageId ? { ...m, content: current } : m)
      );

      await delay(15 + Math.random() * 25);
    }

    messages.update(msgs => 
      msgs.map(m => m.id === messageId ? { ...m, status: 'complete' as const } : m)
    );
  }

  function getStructuredGoalMarkdown(): string {
    const currentState = get(state);
    return structuredGoalToMarkdown(currentState.structuredGoal);
  }

  function stopRefinement() {
    state.update(s => ({
      ...s,
      isActive: false,
      isStreaming: false,
      currentQuestionIndex: 0,
      error: null
    }));
  }

  function reset() {
    state.set({
      isActive: false,
      isStreaming: false,
      currentQuestionIndex: 0,
      structuredGoal: {
        objective: '',
        context: '',
        constraints: '',
        successCriteria: ''
      },
      error: null
    });
    messages.set([]);
    userAnswers.set([]);
  }

  return {
    state: { subscribe: state.subscribe },
    messages: { subscribe: messages.subscribe },
    startRefinement,
    submitAnswer,
    stopRefinement,
    getStructuredGoalMarkdown,
    reset
  };
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export const goalRefinementStore = createGoalRefinementStore();
