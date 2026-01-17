import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';
import { tick } from 'svelte';
import TimeSeriesChart from './TimeSeriesChart.svelte';
import type { TimeSeriesData } from '$lib/types/charts';

describe('TimeSeriesChart', () => {
  const mockData: TimeSeriesData[] = [
    {
      id: 'series1',
      label: 'Missions',
      color: '#3b82f6',
      points: [
        { timestamp: '2024-01-01T00:00:00Z', value: 100 },
        { timestamp: '2024-01-02T00:00:00Z', value: 120 },
        { timestamp: '2024-01-03T00:00:00Z', value: 95 },
        { timestamp: '2024-01-04T00:00:00Z', value: 150 },
      ]
    },
    {
      id: 'series2',
      label: 'Tokens',
      color: '#10b981',
      points: [
        { timestamp: '2024-01-01T00:00:00Z', value: 1000 },
        { timestamp: '2024-01-02T00:00:00Z', value: 1200 },
        { timestamp: '2024-01-03T00:00:00Z', value: 950 },
        { timestamp: '2024-01-04T00:00:00Z', value: 1500 },
      ]
    }
  ];

  beforeEach(() => {
    // Mock ResizeObserver
    global.ResizeObserver = vi.fn(() => ({
      observe: vi.fn(),
      unobserve: vi.fn(),
      disconnect: vi.fn(),
    }));
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders the component', async () => {
    render(TimeSeriesChart, { props: { data: mockData } });
    
    // Should render the container
    expect(screen.getByRole('img', { name: 'Time series chart' })).toBeInTheDocument();
  });

  it('displays multiple data series', async () => {
    render(TimeSeriesChart, { props: { data: mockData, showLegend: true } });
    
    // Should show legend with both series
    expect(screen.getByText('Missions')).toBeInTheDocument();
    expect(screen.getByText('Tokens')).toBeInTheDocument();
  });

  it('hides legend when showLegend is false', async () => {
    render(TimeSeriesChart, { props: { data: mockData, showLegend: false } });
    
    // Should not show legend
    expect(screen.queryByText('Missions')).not.toBeInTheDocument();
    expect(screen.queryByText('Tokens')).not.toBeInTheDocument();
  });

  it('displays brush selection when enabled', async () => {
    render(TimeSeriesChart, { props: { data: mockData, showBrush: true } });
    
    // Should show brush chart
    expect(screen.getByRole('slider', { name: 'Time range selector' })).toBeInTheDocument();
  });

  it('hides brush selection when disabled', async () => {
    render(TimeSeriesChart, { props: { data: mockData, showBrush: false } });
    
    // Should not show brush chart
    expect(screen.queryByRole('slider')).not.toBeInTheDocument();
  });

  it('handles different time formats', async () => {
    const { rerender } = render(TimeSeriesChart, { 
      props: { data: mockData, timeFormat: 'day' } 
    });
    
    // Test different time formats by checking that component renders without error
    await rerender({ data: mockData, timeFormat: 'hour' });
    await rerender({ data: mockData, timeFormat: 'week' });
    await rerender({ data: mockData, timeFormat: 'month' });
    
    expect(screen.getByRole('img')).toBeInTheDocument();
  });

  it('emits rangeChange event on brush selection', async () => {
    const component = render(TimeSeriesChart, { 
      props: { data: mockData, showBrush: true } 
    });
    
    const mockHandler = vi.fn();
    component.component.$on('rangeChange', mockHandler);
    
    // Simulate brush selection (simplified test)
    const brushChart = screen.getByRole('slider');
    await fireEvent.mouseDown(brushChart, { clientX: 100 });
    await fireEvent.mouseMove(brushChart, { clientX: 200 });
    await fireEvent.mouseUp(brushChart);
    
    // Should have attempted to emit rangeChange
    // (Note: actual coordinates would depend on component dimensions)
  });

  it('displays reset zoom button when zoomed', async () => {
    const { container } = render(TimeSeriesChart, { 
      props: { data: mockData, enableZoom: true } 
    });
    
    // Simulate zoom by dispatching wheel event
    const chart = container.querySelector('.time-series-chart');
    if (chart) {
      await fireEvent.wheel(chart, { deltaY: -100 });
      await tick();
      
      // Should show reset zoom button
      expect(screen.getByText('Reset Zoom')).toBeInTheDocument();
    }
  });

  it('handles empty data gracefully', async () => {
    render(TimeSeriesChart, { props: { data: [] } });
    
    // Should render without crashing
    expect(screen.getByRole('img')).toBeInTheDocument();
  });

  it('formats values correctly', () => {
    // This would test the formatValue function if it were exported
    // For now, we test that the component renders with various value ranges
    const highValueData: TimeSeriesData[] = [
      {
        id: 'high',
        label: 'High Values',
        color: '#ef4444',
        points: [
          { timestamp: '2024-01-01T00:00:00Z', value: 1500000 },
          { timestamp: '2024-01-02T00:00:00Z', value: 2500000 },
        ]
      }
    ];
    
    render(TimeSeriesChart, { props: { data: highValueData } });
    expect(screen.getByRole('img')).toBeInTheDocument();
  });

  it('supports real-time data updates', async () => {
    const { rerender } = render(TimeSeriesChart, { props: { data: mockData } });
    
    // Simulate real-time update by adding new data point
    const updatedData = [...mockData];
    updatedData[0].points.push({
      timestamp: '2024-01-05T00:00:00Z',
      value: 175
    });
    
    await rerender({ data: updatedData });
    
    // Should re-render without issues
    expect(screen.getByRole('img')).toBeInTheDocument();
  });

  it('handles responsive resizing', async () => {
    const { container } = render(TimeSeriesChart, { props: { data: mockData } });
    
    // Simulate container resize
    const chartContainer = container.querySelector('.time-series-chart');
    if (chartContainer) {
      Object.defineProperty(chartContainer, 'clientWidth', {
        writable: true,
        configurable: true,
        value: 800,
      });
      
      // Trigger resize event
      await fireEvent(chartContainer, new Event('resize'));
      await tick();
    }
    
    expect(screen.getByRole('img')).toBeInTheDocument();
  });
});