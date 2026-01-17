import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';
import FilterDropdown from './FilterDropdown.svelte';
import type { FilterConfig } from '$lib/types/filters';

// Mock the Icon component
vi.mock('$lib/components/common/Icon.svelte', () => ({
  default: vi.fn(() => ({ $$: { fragment: null } }))
}));

describe('FilterDropdown', () => {
  let mockFilter: FilterConfig;

  beforeEach(() => {
    mockFilter = {
      id: 'status',
      label: 'Status',
      searchable: true,
      options: [
        { value: 'active', label: 'Active', count: 5 },
        { value: 'pending', label: 'Pending', count: 2 },
        { value: 'completed', label: 'Completed', count: 10 },
        { value: 'archived', label: 'Archived', count: 1 }
      ]
    };
  });

  it('renders with filter label', () => {
    const { getByRole } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: []
    });

    expect(getByRole('button', { name: /status/i })).toBeInTheDocument();
  });

  it('shows selected count in label when items are selected', () => {
    const { getByRole } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: ['active', 'pending']
    });

    expect(getByRole('button', { name: /status \(2\)/i })).toBeInTheDocument();
  });

  it('opens dropdown when trigger is clicked', async () => {
    const { getByRole, getByText } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: []
    });

    const trigger = getByRole('button', { name: /status/i });
    await fireEvent.click(trigger);

    // Should show options
    expect(getByText('Active')).toBeInTheDocument();
    expect(getByText('Pending')).toBeInTheDocument();
  });

  it('shows search input when filter is searchable', async () => {
    const { getByRole, getByPlaceholderText } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: []
    });

    const trigger = getByRole('button');
    await fireEvent.click(trigger);

    expect(getByPlaceholderText('Search...')).toBeInTheDocument();
  });

  it('filters options based on search query', async () => {
    const { getByRole, getByPlaceholderText, queryByText } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: []
    });

    const trigger = getByRole('button');
    await fireEvent.click(trigger);

    const searchInput = getByPlaceholderText('Search...');
    await fireEvent.input(searchInput, { target: { value: 'act' } });

    // Should only show "Active"
    expect(queryByText('Active')).toBeInTheDocument();
    expect(queryByText('Pending')).not.toBeInTheDocument();
    expect(queryByText('Completed')).not.toBeInTheDocument();
  });

  it('shows select all and clear actions', async () => {
    const { getByRole, getByText } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: []
    });

    const trigger = getByRole('button');
    await fireEvent.click(trigger);

    expect(getByText('Select All')).toBeInTheDocument();
    expect(getByText('Clear')).toBeInTheDocument();
  });

  it('emits change event when option is selected', async () => {
    let emittedValues: string[] = [];
    
    const { component, getByRole } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: []
    });

    component.$on('change', (event) => {
      emittedValues = event.detail;
    });

    const trigger = getByRole('button');
    await fireEvent.click(trigger);

    // Select an option
    const activeCheckbox = screen.getByLabelText(/active/i);
    await fireEvent.click(activeCheckbox);

    expect(emittedValues).toEqual(['active']);
  });

  it('handles multi-select correctly', async () => {
    let emittedValues: string[] = [];
    
    const { component, getByRole } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: ['active']
    });

    component.$on('change', (event) => {
      emittedValues = event.detail;
    });

    const trigger = getByRole('button');
    await fireEvent.click(trigger);

    // Select additional option
    const pendingCheckbox = screen.getByLabelText(/pending/i);
    await fireEvent.click(pendingCheckbox);

    expect(emittedValues).toEqual(['active', 'pending']);
  });

  it('shows option counts when provided', async () => {
    const { getByRole, getByText } = render(FilterDropdown, {
      filter: mockFilter,
      selectedValues: []
    });

    const trigger = getByRole('button');
    await fireEvent.click(trigger);

    expect(getByText('5')).toBeInTheDocument(); // Active count
    expect(getByText('2')).toBeInTheDocument(); // Pending count
  });
});