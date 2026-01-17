import { render, type RenderResult } from '@testing-library/svelte';
import { writable, type Writable } from 'svelte/store';
import type { SvelteComponent } from 'svelte';

interface RenderOptions {
  props?: Record<string, any>;
  stores?: Record<string, Writable<any>>;
}

export function renderWithStores<T extends SvelteComponent>(
  Component: new (...args: any[]) => T,
  options: RenderOptions = {}
): RenderResult<T> & { stores: Record<string, Writable<any>> } {
  const stores = options.stores || {};

  const result = render(Component, {
    props: options.props,
    context: new Map(Object.entries(stores))
  });

  return { ...result, stores };
}

export function createMockMission(overrides = {}) {
  return {
    id: 'msn_test123',
    specId: 'spc_001',
    title: 'Test Mission',
    description: 'A test mission description',
    state: 'running',
    currentStep: 'Executing tests',
    completedSteps: 5,
    totalSteps: 10,
    tokenUsage: {
      input: 1000,
      output: 500,
      total: 1500,
      cost: 0.05
    },
    recentLogs: [],
    error: null,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    startedAt: new Date().toISOString(),
    completedAt: null,
    ...overrides
  };
}

export function createMockCostData(overrides = {}) {
  return {
    totalCost: 125.50,
    changePercent: 5.2,
    breakdown: {
      byModel: [
        { label: 'Claude Opus', value: 80, percent: 63.7, color: '#8b5cf6' },
        { label: 'Claude Sonnet', value: 45.5, percent: 36.3, color: '#3b82f6' }
      ],
      topMissions: []
    },
    budget: {
      limit: 500,
      alertThreshold: 80,
      period: 'month' as const
    },
    sparklineData: [100, 110, 105, 120, 115, 125],
    projection: {
      projectedCost: 180,
      confidence: 0.85,
      basedOnDays: 15
    },
    ...overrides
  };
}

export function waitForAnimation(ms = 300): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createMockSpec(overrides = {}) {
  return {
    id: 'spc_test001',
    title: 'Test Spec',
    description: 'A test specification',
    phase: 'test',
    status: 'draft',
    dependencies: [],
    estimatedContext: '5%',
    acceptanceCriteria: [
      'Implement feature X',
      'Add tests for feature X',
      'Update documentation'
    ],
    ...overrides
  };
}