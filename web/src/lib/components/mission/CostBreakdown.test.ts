import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import CostBreakdown from './CostBreakdown.svelte';

describe('CostBreakdown', () => {
  const defaultProps = {
    inputTokens: 1000,
    outputTokens: 500,
    inputCost: 0.01,
    outputCost: 0.015
  };

  it('displays input token count and cost', () => {
    render(CostBreakdown, defaultProps);
    
    expect(screen.getByText('1.0k tokens')).toBeInTheDocument();
    expect(screen.getByText('$0.0100')).toBeInTheDocument();
  });

  it('displays output token count and cost', () => {
    render(CostBreakdown, defaultProps);
    
    expect(screen.getByText('500 tokens')).toBeInTheDocument();
    expect(screen.getByText('$0.0150')).toBeInTheDocument();
  });

  it('displays total calculations correctly', () => {
    render(CostBreakdown, defaultProps);
    
    expect(screen.getByText('1.5k tokens')).toBeInTheDocument();
    expect(screen.getByText('$0.0250')).toBeInTheDocument();
  });

  it('formats large token counts with M suffix', () => {
    render(CostBreakdown, {
      inputTokens: 2500000,
      outputTokens: 1500000,
      inputCost: 2.5,
      outputCost: 1.5
    });
    
    expect(screen.getByText('2.50M tokens')).toBeInTheDocument();
    expect(screen.getByText('1.50M tokens')).toBeInTheDocument();
    expect(screen.getByText('4.00M tokens')).toBeInTheDocument();
  });

  it('handles zero values gracefully', () => {
    render(CostBreakdown, {
      inputTokens: 0,
      outputTokens: 0,
      inputCost: 0,
      outputCost: 0
    });
    
    expect(screen.getByText('0 tokens')).toBeInTheDocument();
    expect(screen.getByText('$0.0000')).toBeInTheDocument();
  });
});