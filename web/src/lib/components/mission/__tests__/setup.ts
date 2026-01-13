import { vi } from 'vitest';
import '@testing-library/jest-dom/vitest';

// Mock IPC
vi.mock('$lib/ipc/client', () => ({
  ipc: {
    invoke: vi.fn(),
    on: vi.fn(),
    removeListener: vi.fn(),
  },
}));

// Mock stores
export function createMockMission(overrides = {}) {
  return {
    id: 'test-mission-1',
    title: 'Test Mission',
    prompt: 'Test prompt',
    state: 'idle',
    specIds: [],
    backendId: 'claude-sonnet',
    mode: 'agentic',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    progress: {
      currentStep: 0,
      totalSteps: 0,
      currentAction: '',
      percentage: 0,
      contextUsage: {
        inputTokens: 0,
        outputTokens: 0,
        maxTokens: 200000,
        usagePercent: 0,
        isNearLimit: false,
        isRedlined: false,
      },
    },
    cost: { inputCost: 0, outputCost: 0, totalCost: 0, currency: 'USD' },
    checkpoints: [],
    tags: [],
    ...overrides,
  };
}

export function createMockBackend(overrides = {}) {
  return {
    id: 'claude-sonnet',
    name: 'Claude Sonnet',
    provider: 'anthropic',
    model: 'claude-3-sonnet',
    status: 'available',
    isDefault: true,
    capabilities: {
      maxContextTokens: 200000,
      maxOutputTokens: 4096,
      supportsVision: true,
      supportsTools: true,
      supportsStreaming: true,
      supportsJson: true,
    },
    pricing: {
      inputCostPer1k: 0.003,
      outputCostPer1k: 0.015,
      currency: 'USD',
    },
    lastChecked: new Date().toISOString(),
    ...overrides,
  };
}

export function createMockProgress(overrides = {}) {
  return {
    percentage: 0,
    currentStep: 0,
    totalSteps: 0,
    currentAction: '',
    elapsedMs: 0,
    estimatedRemainingMs: 0,
    stepsCompleted: [],
    isPaused: false,
    isIndeterminate: false,
    ...overrides,
  };
}

export function createMockContextUsage(overrides = {}) {
  return {
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    maxTokens: 200000,
    usagePercent: 0,
    zone: 'safe' as const,
    ...overrides,
  };
}