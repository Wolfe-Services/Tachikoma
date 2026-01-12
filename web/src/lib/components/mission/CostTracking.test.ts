import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import CostTracking from './CostTracking.svelte';
import type { CostInfo, CostBudget, CostProjection } from '$lib/types/cost';

describe('CostTracking', () => {
  const mockCost: CostInfo = {
    inputTokens: 1000,
    outputTokens: 500,
    inputCost: 0.01,
    outputCost: 0.015,
    totalCost: 0.025,
    currency: 'USD'
  };

  it('displays cost with correct formatting', () => {
    render(CostTracking, { cost: mockCost });
    
    expect(screen.getByText('$0.0250')).toBeInTheDocument();
    expect(screen.getByText('Cost')).toBeInTheDocument();
  });

  it('shows budget bar with correct percentage', () => {
    const budget: CostBudget = {
      daily: 10,
      weekly: 50,
      monthly: 200,
      perMission: 0.1
    };
    
    render(CostTracking, { cost: mockCost, budget });
    
    const budgetText = screen.getByText(/25.0% of/);
    expect(budgetText).toBeInTheDocument();
  });

  it('shows warning state when near budget', () => {
    const budget: CostBudget = {
      daily: 10,
      weekly: 50,
      monthly: 200,
      perMission: 0.03 // 25/30 = 83.3%
    };
    
    const { container } = render(CostTracking, { cost: mockCost, budget });
    const totalElement = container.querySelector('.cost-tracking__total');
    expect(totalElement).toHaveClass('near-budget');
  });

  it('shows error state when over budget', () => {
    const budget: CostBudget = {
      daily: 10,
      weekly: 50,
      monthly: 200,
      perMission: 0.02 // 25/20 = 125%
    };
    
    const { container } = render(CostTracking, { cost: mockCost, budget });
    const totalElement = container.querySelector('.cost-tracking__total');
    expect(totalElement).toHaveClass('over-budget');
  });

  it('displays projection correctly', () => {
    const projection: CostProjection = {
      estimatedTotal: 0.05,
      remainingBudget: 0.025,
      projectedOverage: 0.01,
      confidence: 0.8
    };
    
    render(CostTracking, { cost: mockCost, projection });
    
    expect(screen.getByText('Projected: $0.0500')).toBeInTheDocument();
    expect(screen.getByText('+$0.0100 over budget')).toBeInTheDocument();
  });

  it('shows cost history when provided', () => {
    const history = [
      { timestamp: '2024-01-01T10:00:00Z', cost: 0.01 },
      { timestamp: '2024-01-01T10:05:00Z', cost: 0.025 }
    ];
    
    render(CostTracking, { cost: mockCost, history });
    
    expect(screen.getByText('Cost History')).toBeInTheDocument();
  });

  it('does not show budget bar without budget', () => {
    render(CostTracking, { cost: mockCost });
    
    const budgetElement = screen.queryByText(/% of/);
    expect(budgetElement).not.toBeInTheDocument();
  });

  it('does not show projection without projection data', () => {
    render(CostTracking, { cost: mockCost });
    
    const projectionElement = screen.queryByText(/Projected:/);
    expect(projectionElement).not.toBeInTheDocument();
  });
});