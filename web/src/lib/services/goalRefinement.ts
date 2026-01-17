/**
 * Goal Refinement Service
 * 
 * Provides AI-assisted goal refinement through intelligent context-gap analysis.
 * 
 * Analyzes the user's ACTUAL prompt content to ask RELEVANT questions,
 * not generic stakeholder/constraint garbage.
 */

import { writable, get } from 'svelte/store';

export interface RefinementMessage {
  id: string;
  role: 'assistant' | 'user' | 'system';
  content: string;
  timestamp: Date;
  status: 'pending' | 'streaming' | 'complete';
}

export interface ContextGap {
  category: string;
  question: string;
  priority: 'high' | 'medium' | 'low';
  filled: boolean;
}

export interface RefinementState {
  isActive: boolean;
  isStreaming: boolean;
  initialGoal: string;
  contextGaps: ContextGap[];
  currentGapIndex: number;
  refinedGoal: string;
  error: string | null;
}

export interface ConversationContext {
  initialGoal: string;
  exchanges: { question: string; answer: string }[];
}

/**
 * Topic patterns to detect what the user is actually talking about.
 */
const topicPatterns: { pattern: RegExp; topic: string; questions: string[] }[] = [
  // Code/Repo related
  {
    pattern: /\b(repo|repository|repositories|git|code\s*base|codebase)\b/i,
    topic: 'repositories',
    questions: [
      'Which repository providers do you need to support? (GitHub, GitLab, Bitbucket, local file system, etc.)',
      'Should this work with private repos that require authentication?',
      'Do you need to track multiple branches, or just main/default?'
    ]
  },
  // Indexing related
  {
    pattern: /\b(index|indexing|search|find|lookup)\b/i,
    topic: 'indexing',
    questions: [
      'What do you want to index? (file contents, symbols/functions, dependencies, commit history?)',
      'How should search work - full text, semantic/meaning-based, or symbol lookup?',
      'Should the index update automatically when code changes, or on-demand?'
    ]
  },
  // Connection related
  {
    pattern: /\b(connect|connection|sync|pull|fetch|clone)\b/i,
    topic: 'connection',
    questions: [
      'For remote connections, what authentication methods should be supported? (SSH keys, tokens, OAuth?)',
      'Should it handle large repos efficiently, or is size not a concern?',
      'Do you need offline support when remote is unavailable?'
    ]
  },
  // Local vs Remote
  {
    pattern: /\b(local|remote|both)\b/i,
    topic: 'locality',
    questions: [
      'For local repos, should it scan for repos automatically or require manual paths?',
      'Should local and remote repos be treated the same, or have different capabilities?'
    ]
  },
  // API related
  {
    pattern: /\b(api|endpoint|service|backend)\b/i,
    topic: 'api',
    questions: [
      'What kind of API - REST, GraphQL, or CLI commands?',
      'Are there rate limits or quotas to consider?',
      'Does it need to integrate with existing Tachikoma services?'
    ]
  },
  // Database/Storage
  {
    pattern: /\b(database|storage|persist|save|store)\b/i,
    topic: 'storage',
    questions: [
      'What storage backend - SQLite, PostgreSQL, file-based?',
      'How much data are we talking about? Rough estimate of repo count/size?'
    ]
  },
  // UI related
  {
    pattern: /\b(ui|interface|display|show|view|screen)\b/i,
    topic: 'interface',
    questions: [
      'Where should this appear in the Tachikoma UI?',
      'Any specific interactions or workflows you envision?'
    ]
  },
  // Performance
  {
    pattern: /\b(fast|performance|speed|efficient|scale)\b/i,
    topic: 'performance',
    questions: [
      'What scale are we dealing with? Number of repos, lines of code?',
      'Are there specific performance targets to hit?'
    ]
  },
  // Configuration
  {
    pattern: /\b(config|configure|settings|options)\b/i,
    topic: 'configuration',
    questions: [
      'What should be configurable by the user vs hardcoded?',
      'Should config be per-project, per-workspace, or global?'
    ]
  }
];

/**
 * Analyzes the user's actual prompt to extract relevant topics and generate
 * questions that are SPECIFIC to what they're asking about.
 */
function analyzePromptForQuestions(initialGoal: string): ContextGap[] {
  const text = initialGoal.toLowerCase();
  const gaps: ContextGap[] = [];
  const usedQuestions = new Set<string>();
  const detectedTopics = new Set<string>();

  // Find all matching topics
  for (const { pattern, topic, questions } of topicPatterns) {
    if (pattern.test(text)) {
      detectedTopics.add(topic);
      
      // Pick the most relevant question for this topic (first one not used)
      for (const q of questions) {
        if (!usedQuestions.has(q) && gaps.length < 4) {
          usedQuestions.add(q);
          gaps.push({
            category: topic,
            question: q,
            priority: gaps.length === 0 ? 'high' : 'medium',
            filled: false
          });
          break; // Only one question per topic initially
        }
      }
    }
  }

  // If prompt is very short, ask for elaboration first
  if (initialGoal.trim().length < 50 && gaps.length === 0) {
    gaps.unshift({
      category: 'clarity',
      question: `You mentioned: "${initialGoal.trim()}"\n\nCan you expand on this? What specifically are you trying to accomplish?`,
      priority: 'high',
      filled: false
    });
  }

  // If we found topics but have room for more questions, add second-tier questions
  if (gaps.length < 3 && detectedTopics.size > 0) {
    for (const { pattern, topic, questions } of topicPatterns) {
      if (pattern.test(text) && detectedTopics.has(topic)) {
        for (const q of questions) {
          if (!usedQuestions.has(q) && gaps.length < 4) {
            usedQuestions.add(q);
            gaps.push({
              category: topic,
              question: q,
              priority: 'low',
              filled: false
            });
          }
        }
      }
    }
  }

  // Fallback if we couldn't detect any topics
  if (gaps.length === 0) {
    gaps.push({
      category: 'scope',
      question: `I want to make sure I understand correctly.\n\nWhat's the main outcome you're looking for from this session? What would success look like?`,
      priority: 'high',
      filled: false
    });
  }

  return gaps.slice(0, 4);
}

/**
 * Synthesizes the conversation into a refined goal.
 */
function synthesizeRefinedGoal(context: ConversationContext): string {
  const { initialGoal, exchanges } = context;
  
  let markdown = `## Objective\n${initialGoal}`;
  
  if (exchanges.length > 0) {
    markdown += `\n\n## Details\n`;
    
    for (const { question, answer } of exchanges) {
      // Extract the essence of the question for a heading
      const shortQ = question.split('\n')[0].replace(/[?:]+$/, '').trim();
      markdown += `\n**${shortQ}**\n${answer}\n`;
    }
  }

  return markdown;
}

function createGoalRefinementStore() {
  const state = writable<RefinementState>({
    isActive: false,
    isStreaming: false,
    initialGoal: '',
    contextGaps: [],
    currentGapIndex: 0,
    refinedGoal: '',
    error: null
  });

  const messages = writable<RefinementMessage[]>([]);
  const conversationContext = writable<ConversationContext>({
    initialGoal: '',
    exchanges: []
  });

  function startRefinement(initialGoal: string) {
    // Analyze what the user ACTUALLY wrote
    const gaps = analyzePromptForQuestions(initialGoal);
    
    state.update(s => ({
      ...s,
      isActive: true,
      initialGoal,
      contextGaps: gaps,
      currentGapIndex: 0,
      error: null
    }));
    
    messages.set([]);
    conversationContext.set({
      initialGoal,
      exchanges: []
    });

    if (gaps.length > 0) {
      askQuestion(gaps[0].question);
    } else {
      addAssistantMessage(`Your goal looks clear. Proceed when ready, or ask me to clarify anything.`);
    }
  }

  async function askQuestion(question: string) {
    const messageId = addPendingMessage();
    await delay(200 + Math.random() * 150);
    state.update(s => ({ ...s, isStreaming: true }));
    await streamMessage(messageId, question);
    state.update(s => ({ ...s, isStreaming: false }));
  }

  async function submitAnswer(answer: string) {
    const currentState = get(state);
    const currentGap = currentState.contextGaps[currentState.currentGapIndex];

    // Add user message
    messages.update(msgs => [...msgs, {
      id: `user-${Date.now()}`,
      role: 'user',
      content: answer,
      timestamp: new Date(),
      status: 'complete'
    }]);

    // Record the exchange
    conversationContext.update(ctx => ({
      ...ctx,
      exchanges: [...ctx.exchanges, { 
        question: currentGap?.question || '', 
        answer 
      }]
    }));

    // Mark current gap as filled
    state.update(s => {
      const gaps = [...s.contextGaps];
      if (gaps[s.currentGapIndex]) {
        gaps[s.currentGapIndex].filled = true;
      }
      return { ...s, contextGaps: gaps };
    });

    await delay(150);

    // Check for more questions
    const updatedState = get(state);
    const nextGapIndex = updatedState.currentGapIndex + 1;

    if (nextGapIndex < updatedState.contextGaps.length) {
      state.update(s => ({ ...s, currentGapIndex: nextGapIndex }));
      
      const nextQuestion = updatedState.contextGaps[nextGapIndex].question;
      await askQuestion(nextQuestion);
    } else {
      await synthesizeAndPresent();
    }
  }

  async function synthesizeAndPresent() {
    const context = get(conversationContext);
    
    const messageId = addPendingMessage();
    await delay(300);
    
    state.update(s => ({ ...s, isStreaming: true }));

    const refinedGoal = synthesizeRefinedGoal(context);
    state.update(s => ({ ...s, refinedGoal }));

    const msg = `Here's your refined goal:\n\n---\n\n${refinedGoal}\n\n---\n\nClick **Apply to Goal** to use this, or keep chatting to refine further.`;

    await streamMessage(messageId, msg);
    state.update(s => ({ ...s, isStreaming: false }));
  }

  function addPendingMessage(): string {
    const messageId = `msg-${Date.now()}`;
    messages.update(msgs => [...msgs, {
      id: messageId,
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      status: 'pending'
    }]);
    return messageId;
  }

  async function addAssistantMessage(content: string) {
    const messageId = addPendingMessage();
    await delay(150);
    state.update(s => ({ ...s, isStreaming: true }));
    await streamMessage(messageId, content);
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

      await delay(10 + Math.random() * 15);
    }

    messages.update(msgs => 
      msgs.map(m => m.id === messageId ? { ...m, status: 'complete' as const } : m)
    );
  }

  function getRefinedGoalMarkdown(): string {
    return get(state).refinedGoal;
  }

  function stopRefinement() {
    state.update(s => ({
      ...s,
      isActive: false,
      isStreaming: false,
      error: null
    }));
  }

  function reset() {
    state.set({
      isActive: false,
      isStreaming: false,
      initialGoal: '',
      contextGaps: [],
      currentGapIndex: 0,
      refinedGoal: '',
      error: null
    });
    messages.set([]);
    conversationContext.set({
      initialGoal: '',
      exchanges: []
    });
  }

  return {
    state: { subscribe: state.subscribe },
    messages: { subscribe: messages.subscribe },
    startRefinement,
    submitAnswer,
    stopRefinement,
    getRefinedGoalMarkdown,
    reset
  };
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export const goalRefinementStore = createGoalRefinementStore();
