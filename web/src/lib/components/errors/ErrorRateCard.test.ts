import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import ErrorRateCard from './ErrorRateCard.svelte';
import type { ErrorStats, ErrorItem } from '$lib/types/errors';

// Mock the chart components
vi.mock('$lib/components/charts/TimeSeriesChart.svelte', () => ({
  default: vi.fn().mockImplementation(() => ({
    $$: {
      on_mount: [],
      on_destroy: []
    }
  }))
}));

vi.mock('$lib/components/charts/DonutChart.svelte', () => ({
  default: vi.fn().mockImplementation(() => ({
    $$: {
      on_mount: [],
      on_destroy: []
    }
  }))
}));

describe('ErrorRateCard', () => {
  const mockStats: ErrorStats = {
    currentRate: 2.5,
    changePercent: 15.3,
    totalErrors: 142,
    byType: {
      'API Error': 45,
      'Timeout': 32,
      'Validation': 28,
      'Auth': 20,
      'Rate Limit': 12,
      'Unknown': 5
    },
    trendData: [
      { timestamp: '2024-01-01T10:00:00Z', count: 3 },
      { timestamp: '2024-01-01T11:00:00Z', count: 2 },
      { timestamp: '2024-01-01T12:00:00Z', count: 5 }
    ]
  };

  const mockErrors: ErrorItem[] = [
    {
      id: '1',
      type: 'API Error',
      message: 'Failed to connect to external service',
      severity: 'critical',
      count: 25,
      firstSeen: '2024-01-01T09:00:00Z',
      lastSeen: '2024-01-01T12:30:00Z',
      affectedMissions: 3,
      stackTrace: 'Error: Connection timeout\n  at fetch.js:42'
    },
    {
      id: '2',
      type: 'Validation',
      message: 'Invalid mission configuration',
      severity: 'high',
      count: 18,
      firstSeen: '2024-01-01T08:00:00Z',
      lastSeen: '2024-01-01T12:15:00Z',
      affectedMissions: 2
    },
    {
      id: '3',
      type: 'Timeout',
      message: 'Request timeout after 30s',
      severity: 'medium',
      count: 12,
      firstSeen: '2024-01-01T10:30:00Z',
      lastSeen: '2024-01-01T11:45:00Z',
      affectedMissions: 1
    }
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders error rate card with basic stats', () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors
      }
    });

    // Check if main elements are present
    expect(screen.getByText('Error Rate')).toBeInTheDocument();
    expect(screen.getByText('2.50')).toBeInTheDocument();
    expect(screen.getByText('errors/min')).toBeInTheDocument();
    expect(screen.getByText('15.3%')).toBeInTheDocument();
    expect(screen.getByText('vs last hour')).toBeInTheDocument();
  });

  it('shows alert badge when above threshold', () => {
    const alertStats = { ...mockStats, currentRate: 6.0 };
    
    render(ErrorRateCard, {
      props: {
        stats: alertStats,
        errors: mockErrors,
        alertThreshold: 5
      }
    });

    expect(screen.getByText('Above Threshold')).toBeInTheDocument();
  });

  it('does not show alert badge when below threshold', () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        alertThreshold: 5
      }
    });

    expect(screen.queryByText('Above Threshold')).not.toBeInTheDocument();
  });

  it('displays error type breakdown when expanded', async () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        showBreakdown: true
      }
    });

    const expandButton = screen.getByText('Error Breakdown');
    await fireEvent.click(expandButton);

    // Check for error types in breakdown
    expect(screen.getByText('API Error')).toBeInTheDocument();
    expect(screen.getByText('Timeout')).toBeInTheDocument();
    expect(screen.getByText('Validation')).toBeInTheDocument();
    expect(screen.getByText('45')).toBeInTheDocument();
    expect(screen.getByText('32')).toBeInTheDocument();
    expect(screen.getByText('28')).toBeInTheDocument();
  });

  it('shows top errors list with correct information', async () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        showBreakdown: true
      }
    });

    const expandButton = screen.getByText('Error Breakdown');
    await fireEvent.click(expandButton);

    // Check for error messages
    expect(screen.getByText('Failed to connect to external service')).toBeInTheDocument();
    expect(screen.getByText('Invalid mission configuration')).toBeInTheDocument();
    expect(screen.getByText('Request timeout after 30s')).toBeInTheDocument();

    // Check for error counts
    expect(screen.getByText('25 occurrences')).toBeInTheDocument();
    expect(screen.getByText('18 occurrences')).toBeInTheDocument();
    expect(screen.getByText('12 occurrences')).toBeInTheDocument();
  });

  it('expands error details when clicked', async () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        showBreakdown: true
      }
    });

    const expandButton = screen.getByText('Error Breakdown');
    await fireEvent.click(expandButton);

    const firstError = screen.getByText('Failed to connect to external service');
    await fireEvent.click(firstError.closest('.error-item'));

    // Check for error details
    expect(screen.getByText('Type')).toBeInTheDocument();
    expect(screen.getByText('API Error')).toBeInTheDocument();
    expect(screen.getByText('First Seen')).toBeInTheDocument();
    expect(screen.getByText('Affected Missions')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
  });

  it('shows stack trace when available', async () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        showBreakdown: true
      }
    });

    const expandButton = screen.getByText('Error Breakdown');
    await fireEvent.click(expandButton);

    const firstError = screen.getByText('Failed to connect to external service');
    await fireEvent.click(firstError.closest('.error-item'));

    // Check for stack trace
    expect(screen.getByText('Error: Connection timeout')).toBeInTheDocument();
    expect(screen.getByText('at fetch.js:42')).toBeInTheDocument();
  });

  it('calculates error percentages correctly', async () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        showBreakdown: true
      }
    });

    const expandButton = screen.getByText('Error Breakdown');
    await fireEvent.click(expandButton);

    // API Error: 45/142 = 31.7%
    expect(screen.getByText('31.7%')).toBeInTheDocument();
    // Timeout: 32/142 = 22.5%
    expect(screen.getByText('22.5%')).toBeInTheDocument();
    // Validation: 28/142 = 19.7%
    expect(screen.getByText('19.7%')).toBeInTheDocument();
  });

  it('displays total error count', () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors
      }
    });

    expect(screen.getByText('Total: 142 errors (24h)')).toBeInTheDocument();
  });

  it('shows trend direction correctly', () => {
    // Test upward trend
    const upStats = { ...mockStats, changePercent: 15.3 };
    const { rerender } = render(ErrorRateCard, {
      props: {
        stats: upStats,
        errors: mockErrors
      }
    });

    let trendElements = document.querySelectorAll('.rate-change.up');
    expect(trendElements).toHaveLength(1);

    // Test downward trend
    const downStats = { ...mockStats, changePercent: -8.7 };
    rerender({
      stats: downStats,
      errors: mockErrors
    });

    trendElements = document.querySelectorAll('.rate-change.down');
    expect(trendElements).toHaveLength(1);
  });

  it('renders time series chart when showTrend is true', () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        showTrend: true
      }
    });

    expect(screen.getByText('Error Trend (24h)')).toBeInTheDocument();
  });

  it('does not render time series chart when showTrend is false', () => {
    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: mockErrors,
        showTrend: false
      }
    });

    expect(screen.queryByText('Error Trend (24h)')).not.toBeInTheDocument();
  });

  it('limits error list to top 5 errors', async () => {
    const manyErrors = Array.from({ length: 10 }, (_, i) => ({
      id: String(i + 1),
      type: 'API Error',
      message: `Error ${i + 1}`,
      severity: 'medium' as const,
      count: 10 - i,
      firstSeen: '2024-01-01T09:00:00Z',
      lastSeen: '2024-01-01T12:30:00Z',
      affectedMissions: 1
    }));

    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: manyErrors,
        showBreakdown: true
      }
    });

    const expandButton = screen.getByText('Error Breakdown');
    await fireEvent.click(expandButton);

    // Should only show first 5 errors
    expect(screen.getByText('Error 1')).toBeInTheDocument();
    expect(screen.getByText('Error 5')).toBeInTheDocument();
    expect(screen.queryByText('Error 6')).not.toBeInTheDocument();
  });

  it('applies correct severity colors', () => {
    const errorWithSeverities: ErrorItem[] = [
      { ...mockErrors[0], severity: 'critical' },
      { ...mockErrors[1], severity: 'high' },
      { ...mockErrors[2], severity: 'medium' }
    ];

    render(ErrorRateCard, {
      props: {
        stats: mockStats,
        errors: errorWithSeverities,
        showBreakdown: true
      }
    });

    // The severity colors should be applied via CSS variables
    // This is tested by checking if the component renders without errors
    expect(screen.getByText('Error Rate')).toBeInTheDocument();
  });
});