import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';
import FilterBar from './FilterBar.svelte';
import type { FilterConfig, ActiveFilter } from '$lib/types/filters';

// Mock the Icon component
vi.mock('$lib/components/common/Icon.svelte', () => ({
  default: vi.fn(() => ({ $$: { fragment: null } }))
}));

describe('FilterBar', () => {
  let mockFilters: FilterConfig[];
  let mockActiveFilters: ActiveFilter[];
  let mockPresets: Array<{ id: string; name: string; filters: ActiveFilter[] }>;

  beforeEach(() => {
    mockFilters = [
      {
        id: 'status',
        label: 'Status',
        searchable: true,
        options: [
          { value: 'active', label: 'Active', count: 5 },
          { value: 'pending', label: 'Pending', count: 2 },
          { value: 'completed', label: 'Completed', count: 10 }
        ]
      },
      {
        id: 'category',
        label: 'Category',
        searchable: false,
        options: [
          { value: 'bug', label: 'Bug', icon: 'bug' },
          { value: 'feature', label: 'Feature', icon: 'star' },
          { value: 'docs', label: 'Documentation', icon: 'book' }
        ]
      }
    ];

    mockActiveFilters = [
      { id: 'status', label: 'Status', values: ['active', 'pending'] }
    ];

    mockPresets = [
      {
        id: 'active-bugs',
        name: 'Active Bugs',
        filters: [
          { id: 'status', label: 'Status', values: ['active'] },
          { id: 'category', label: 'Category', values: ['bug'] }
        ]
      },
      {
        id: 'completed-features',
        name: 'Completed Features',
        filters: [
          { id: 'status', label: 'Status', values: ['completed'] },
          { id: 'category', label: 'Category', values: ['feature'] }
        ]
      }
    ];
  });

  it('renders with filters and active filters', () => {
    const { getByRole } = render(FilterBar, {
      filters: mockFilters,
      activeFilters: mockActiveFilters
    });

    // Should show clear all button when active filters exist
    expect(getByRole('button', { name: /clear all/i })).toBeInTheDocument();
  });

  it('shows search input when showSearch is true', () => {
    const { getByPlaceholderText } = render(FilterBar, {
      filters: mockFilters,
      showSearch: true,
      searchPlaceholder: 'Search items...'
    });

    expect(getByPlaceholderText('Search items...')).toBeInTheDocument();
  });

  it('hides search input when showSearch is false', () => {
    const { queryByPlaceholderText } = render(FilterBar, {
      filters: mockFilters,
      showSearch: false,
      searchPlaceholder: 'Search items...'
    });

    expect(queryByPlaceholderText('Search items...')).not.toBeInTheDocument();
  });

  it('displays filter presets when provided', () => {
    const { getByRole, getByText } = render(FilterBar, {
      filters: mockFilters,
      presets: mockPresets
    });

    // Should show presets button
    expect(getByRole('button', { name: /presets/i })).toBeInTheDocument();
  });

  it('emits change event when filters are modified', async () => {
    let emittedFilters: ActiveFilter[] = [];
    
    const { component } = render(FilterBar, {
      filters: mockFilters,
      activeFilters: []
    });

    component.$on('change', (event) => {
      emittedFilters = event.detail;
    });

    // Simulate filter change
    await component.$set({
      activeFilters: [{ id: 'status', label: 'Status', values: ['active'] }]
    });

    expect(emittedFilters).toEqual([
      { id: 'status', label: 'Status', values: ['active'] }
    ]);
  });

  it('emits clear event when clear all is clicked', async () => {
    let clearEmitted = false;
    
    const { component, getByRole } = render(FilterBar, {
      filters: mockFilters,
      activeFilters: mockActiveFilters
    });

    component.$on('clear', () => {
      clearEmitted = true;
    });

    const clearButton = getByRole('button', { name: /clear all/i });
    await fireEvent.click(clearButton);

    expect(clearEmitted).toBe(true);
  });

  it('shows mobile filter toggle on mobile breakpoint', () => {
    // Mock mobile viewport
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 500,
    });

    const { getByRole } = render(FilterBar, {
      filters: mockFilters,
      activeFilters: mockActiveFilters
    });

    // Mobile filters button should exist
    expect(getByRole('button', { name: /filters/i })).toBeInTheDocument();
  });
});