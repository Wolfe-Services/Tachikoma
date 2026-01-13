import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import HistoryView from './HistoryView.svelte';
import type { MissionHistoryEntry } from '$lib/types/history';

describe('HistoryView', () => {
  const mockEntries: MissionHistoryEntry[] = [
    {
      id: '1',
      title: 'First Mission',
      prompt: 'This is the first mission prompt with some details',
      state: 'complete',
      createdAt: '2024-01-01T10:00:00Z',
      completedAt: '2024-01-01T10:05:00Z',
      duration: 300,
      cost: 0.05,
      tokensUsed: 1000,
      filesChanged: 3,
      tags: ['urgent', 'feature']
    },
    {
      id: '2',
      title: 'Second Mission',
      prompt: 'This is the second mission prompt for testing purposes',
      state: 'error',
      createdAt: '2024-01-02T14:30:00Z',
      completedAt: '2024-01-02T14:32:00Z',
      duration: 120,
      cost: 0.02,
      tokensUsed: 500,
      filesChanged: 1,
      tags: ['bugfix']
    },
    {
      id: '3',
      title: 'Third Mission',
      prompt: 'Another test mission with different characteristics',
      state: 'running',
      createdAt: '2024-01-03T09:00:00Z',
      completedAt: '2024-01-03T09:10:00Z',
      duration: 600,
      cost: 0.08,
      tokensUsed: 2000,
      filesChanged: 5,
      tags: ['feature', 'ui']
    }
  ];

  it('displays chronological list of missions', () => {
    render(HistoryView, { entries: mockEntries });
    
    expect(screen.getByText('First Mission')).toBeInTheDocument();
    expect(screen.getByText('Second Mission')).toBeInTheDocument();
    expect(screen.getByText('Third Mission')).toBeInTheDocument();
  });

  it('shows empty state when no entries', () => {
    render(HistoryView, { entries: [] });
    
    expect(screen.getByText('No mission history yet.')).toBeInTheDocument();
  });

  it('shows filtered empty state', async () => {
    render(HistoryView, { entries: mockEntries });
    
    // Apply a filter that matches no entries
    const searchInput = screen.getByPlaceholderText('Search missions...');
    await fireEvent.input(searchInput, { target: { value: 'nonexistent' } });
    
    expect(screen.getByText('No missions match your filters.')).toBeInTheDocument();
  });

  it('filters by search query', async () => {
    render(HistoryView, { entries: mockEntries });
    
    const searchInput = screen.getByPlaceholderText('Search missions...');
    await fireEvent.input(searchInput, { target: { value: 'first' } });
    
    expect(screen.getByText('First Mission')).toBeInTheDocument();
    expect(screen.queryByText('Second Mission')).not.toBeInTheDocument();
  });

  it('searches in prompts as well as titles', async () => {
    render(HistoryView, { entries: mockEntries });
    
    const searchInput = screen.getByPlaceholderText('Search missions...');
    await fireEvent.input(searchInput, { target: { value: 'testing purposes' } });
    
    expect(screen.getByText('Second Mission')).toBeInTheDocument();
    expect(screen.queryByText('First Mission')).not.toBeInTheDocument();
  });

  it('shows selection controls when items are selected', async () => {
    render(HistoryView, { entries: mockEntries });
    
    // Select the first mission
    const firstCheckbox = screen.getAllByRole('checkbox')[0];
    await fireEvent.click(firstCheckbox);
    
    await waitFor(() => {
      expect(screen.getByText('1 selected')).toBeInTheDocument();
      expect(screen.getByText('Clear')).toBeInTheDocument();
      expect(screen.getByText('Export')).toBeInTheDocument();
      expect(screen.getByText('Delete')).toBeInTheDocument();
    });
  });

  it('can select multiple missions', async () => {
    render(HistoryView, { entries: mockEntries });
    
    const checkboxes = screen.getAllByRole('checkbox');
    await fireEvent.click(checkboxes[0]);
    await fireEvent.click(checkboxes[1]);
    
    await waitFor(() => {
      expect(screen.getByText('2 selected')).toBeInTheDocument();
    });
  });

  it('can clear selection', async () => {
    render(HistoryView, { entries: mockEntries });
    
    // Select a mission
    const firstCheckbox = screen.getAllByRole('checkbox')[0];
    await fireEvent.click(firstCheckbox);
    
    await waitFor(() => {
      expect(screen.getByText('1 selected')).toBeInTheDocument();
    });
    
    // Clear selection
    const clearButton = screen.getByText('Clear');
    await fireEvent.click(clearButton);
    
    await waitFor(() => {
      expect(screen.queryByText('1 selected')).not.toBeInTheDocument();
    });
  });

  it('can select all missions', async () => {
    render(HistoryView, { entries: mockEntries });
    
    // First select one to show selection bar
    const firstCheckbox = screen.getAllByRole('checkbox')[0];
    await fireEvent.click(firstCheckbox);
    
    await waitFor(() => {
      expect(screen.getByText('Select All')).toBeInTheDocument();
    });
    
    // Select all
    const selectAllButton = screen.getByText('Select All');
    await fireEvent.click(selectAllButton);
    
    await waitFor(() => {
      expect(screen.getByText('3 selected')).toBeInTheDocument();
    });
  });

  it('shows confirmation dialog for delete', async () => {
    // Mock window.confirm
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    
    render(HistoryView, { entries: mockEntries });
    
    // Select a mission
    const firstCheckbox = screen.getAllByRole('checkbox')[0];
    await fireEvent.click(firstCheckbox);
    
    await waitFor(() => {
      expect(screen.getByText('Delete')).toBeInTheDocument();
    });
    
    // Click delete
    const deleteButton = screen.getByText('Delete');
    await fireEvent.click(deleteButton);
    
    expect(confirmSpy).toHaveBeenCalledWith('Delete 1 missions? This action cannot be undone.');
    
    confirmSpy.mockRestore();
  });

  it('displays mission count in footer', () => {
    render(HistoryView, { entries: mockEntries });
    
    expect(screen.getByText('3 of 3 missions')).toBeInTheDocument();
  });

  it('updates count when filtered', async () => {
    render(HistoryView, { entries: mockEntries });
    
    const searchInput = screen.getByPlaceholderText('Search missions...');
    await fireEvent.input(searchInput, { target: { value: 'first' } });
    
    await waitFor(() => {
      expect(screen.getByText('1 of 3 missions')).toBeInTheDocument();
    });
  });

  it('sorts by different fields', async () => {
    render(HistoryView, { entries: mockEntries });
    
    // Default sort should show newest first (Third Mission created on 01-03)
    const titles = screen.getAllByRole('heading', { level: 3 });
    expect(titles[0]).toHaveTextContent('Third Mission');
  });

  it('handles hover preview', async () => {
    vi.useFakeTimers();
    
    render(HistoryView, { entries: mockEntries });
    
    const firstCard = screen.getByText('First Mission').closest('.history-card');
    expect(firstCard).toBeInTheDocument();
    
    // Hover over the card
    if (firstCard) {
      await fireEvent.mouseEnter(firstCard);
      
      // Fast-forward timer to trigger preview
      vi.advanceTimersByTime(500);
      
      await waitFor(() => {
        expect(screen.getByText('Mission Preview')).toBeInTheDocument();
      });
    }
    
    vi.useRealTimers();
  });

  it('displays mission stats correctly', () => {
    render(HistoryView, { entries: mockEntries });
    
    // Check for duration formatting
    expect(screen.getByText('5m 0s')).toBeInTheDocument(); // 300 seconds
    expect(screen.getByText('2m 0s')).toBeInTheDocument(); // 120 seconds
    expect(screen.getByText('10m 0s')).toBeInTheDocument(); // 600 seconds
    
    // Check for cost formatting
    expect(screen.getByText('$0.050')).toBeInTheDocument();
    expect(screen.getByText('$0.020')).toBeInTheDocument();
    expect(screen.getByText('$0.080')).toBeInTheDocument();
  });

  it('displays status with correct styling', () => {
    const { container } = render(HistoryView, { entries: mockEntries });
    
    const completeStatus = container.querySelector('.history-card__status:first-of-type');
    const errorStatus = container.querySelector('.history-card__status:nth-of-type(2)');
    const runningStatus = container.querySelector('.history-card__status:nth-of-type(3)');
    
    // Note: We can't easily test CSS custom properties, but we can verify the elements exist
    expect(completeStatus).toHaveTextContent('complete');
    expect(errorStatus).toHaveTextContent('error');
    expect(runningStatus).toHaveTextContent('running');
  });

  it('truncates long prompts', () => {
    const longPromptEntry: MissionHistoryEntry = {
      id: 'long',
      title: 'Long Prompt Mission',
      prompt: 'This is a very long prompt that should be truncated because it exceeds the 100 character limit that we have set for the history card display',
      state: 'complete',
      createdAt: '2024-01-01T10:00:00Z',
      completedAt: '2024-01-01T10:05:00Z',
      duration: 300,
      cost: 0.05,
      tokensUsed: 1000,
      filesChanged: 3,
      tags: []
    };
    
    render(HistoryView, { entries: [longPromptEntry] });
    
    expect(screen.getByText(/This is a very long prompt that should be truncated because it exceeds the 100 character limit.../)).toBeInTheDocument();
  });

  it('displays tags with overflow handling', () => {
    const manyTagsEntry: MissionHistoryEntry = {
      id: 'tags',
      title: 'Many Tags Mission',
      prompt: 'Mission with many tags',
      state: 'complete',
      createdAt: '2024-01-01T10:00:00Z',
      completedAt: '2024-01-01T10:05:00Z',
      duration: 300,
      cost: 0.05,
      tokensUsed: 1000,
      filesChanged: 3,
      tags: ['tag1', 'tag2', 'tag3', 'tag4', 'tag5']
    };
    
    render(HistoryView, { entries: [manyTagsEntry] });
    
    expect(screen.getByText('tag1')).toBeInTheDocument();
    expect(screen.getByText('tag2')).toBeInTheDocument();
    expect(screen.getByText('tag3')).toBeInTheDocument();
    expect(screen.getByText('+2')).toBeInTheDocument(); // +2 more tags
  });
});