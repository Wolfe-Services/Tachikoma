import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { tick } from 'svelte';
import RefreshControl from './RefreshControl.svelte';

// Mock the Icon component
vi.mock('$lib/components/common/Icon.svelte', () => ({
  default: vi.fn(() => ({ 
    $$: { 
      component: { 
        render: () => '<span data-testid="icon"></span>' 
      } 
    } 
  }))
}));

describe('RefreshControl', () => {
  let mockDispatch: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.useFakeTimers();
    mockDispatch = vi.fn();
    
    // Mock document.hidden property
    Object.defineProperty(document, 'hidden', {
      writable: true,
      value: false
    });

    // Mock document.addEventListener/removeEventListener
    vi.spyOn(document, 'addEventListener');
    vi.spyOn(document, 'removeEventListener');
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  describe('Manual refresh button', () => {
    it('displays refresh button', () => {
      render(RefreshControl);
      
      const refreshButton = screen.getByLabelText('Refresh data');
      expect(refreshButton).toBeInTheDocument();
    });

    it('dispatches refresh event when clicked', async () => {
      const { component } = render(RefreshControl);
      
      component.$on('refresh', mockDispatch);
      
      const refreshButton = screen.getByLabelText('Refresh data');
      await fireEvent.click(refreshButton);
      
      expect(mockDispatch).toHaveBeenCalledTimes(1);
    });

    it('is disabled when loading', () => {
      render(RefreshControl, { loading: true });
      
      const refreshButton = screen.getByLabelText('Refresh data');
      expect(refreshButton).toBeDisabled();
    });

    it('is disabled when explicitly disabled', () => {
      render(RefreshControl, { disabled: true });
      
      const refreshButton = screen.getByLabelText('Refresh data');
      expect(refreshButton).toBeDisabled();
    });

    it('shows spinning icon when loading', () => {
      const { container } = render(RefreshControl, { loading: true });
      
      // Check for spinning class on icon
      const icon = container.querySelector('.spinning');
      expect(icon).toBeInTheDocument();
    });
  });

  describe('Auto-refresh toggle', () => {
    it('displays auto-refresh toggle button', () => {
      render(RefreshControl);
      
      const toggleButton = screen.getByTitle('Enable auto-refresh');
      expect(toggleButton).toBeInTheDocument();
    });

    it('toggles auto-refresh state when clicked', async () => {
      const { component } = render(RefreshControl);
      
      component.$on('autoRefreshChange', mockDispatch);
      
      const toggleButton = screen.getByTitle('Enable auto-refresh');
      await fireEvent.click(toggleButton);
      
      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ detail: true })
      );
    });

    it('shows active state when auto-refresh is enabled', () => {
      const { container } = render(RefreshControl, { autoRefresh: true });
      
      const toggleButton = container.querySelector('.auto-refresh-toggle.active');
      expect(toggleButton).toBeInTheDocument();
    });

    it('shows countdown when auto-refresh is active', async () => {
      const { container } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 5000 
      });
      
      await tick();
      
      const countdown = container.querySelector('.countdown');
      expect(countdown).toBeInTheDocument();
      expect(countdown?.textContent).toBe('5s');
    });
  });

  describe('Configurable refresh intervals', () => {
    it('displays interval dropdown button', () => {
      render(RefreshControl);
      
      const intervalButton = screen.getByTitle('Set refresh interval');
      expect(intervalButton).toBeInTheDocument();
    });

    it('shows interval menu when dropdown clicked', async () => {
      const { container } = render(RefreshControl);
      
      const intervalButton = screen.getByTitle('Set refresh interval');
      await fireEvent.click(intervalButton);
      
      const intervalMenu = container.querySelector('.interval-menu');
      expect(intervalMenu).toBeInTheDocument();
    });

    it('displays all interval options', async () => {
      render(RefreshControl);
      
      const intervalButton = screen.getByTitle('Set refresh interval');
      await fireEvent.click(intervalButton);
      
      expect(screen.getByText('10s')).toBeInTheDocument();
      expect(screen.getByText('30s')).toBeInTheDocument();
      expect(screen.getByText('1m')).toBeInTheDocument();
      expect(screen.getByText('5m')).toBeInTheDocument();
      expect(screen.getByText('10m')).toBeInTheDocument();
    });

    it('dispatches intervalChange when option selected', async () => {
      const { component } = render(RefreshControl);
      
      component.$on('intervalChange', mockDispatch);
      
      const intervalButton = screen.getByTitle('Set refresh interval');
      await fireEvent.click(intervalButton);
      
      const option = screen.getByText('1m');
      await fireEvent.click(option);
      
      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ detail: 60000 })
      );
    });

    it('shows selected interval with check mark', async () => {
      const { container } = render(RefreshControl, { interval: 60000 });
      
      const intervalButton = screen.getByTitle('Set refresh interval');
      await fireEvent.click(intervalButton);
      
      const selectedOption = container.querySelector('.interval-option.selected');
      expect(selectedOption).toBeInTheDocument();
    });

    it('hides menu when clicking outside', async () => {
      const { container } = render(RefreshControl);
      
      const intervalButton = screen.getByTitle('Set refresh interval');
      await fireEvent.click(intervalButton);
      
      // Menu should be visible
      expect(container.querySelector('.interval-menu')).toBeInTheDocument();
      
      // Click outside
      await fireEvent.click(document.body);
      
      // Menu should be hidden
      expect(container.querySelector('.interval-menu')).not.toBeInTheDocument();
    });
  });

  describe('Last updated timestamp display', () => {
    it('shows last updated time when provided', () => {
      const lastUpdated = new Date('2024-01-01T12:00:00Z');
      render(RefreshControl, { lastUpdated });
      
      expect(screen.getByText(/Updated/)).toBeInTheDocument();
    });

    it('formats recent updates as "Just now"', () => {
      const lastUpdated = new Date(Date.now() - 30000); // 30 seconds ago
      render(RefreshControl, { lastUpdated });
      
      expect(screen.getByText('Updated Just now')).toBeInTheDocument();
    });

    it('formats minutes ago correctly', () => {
      const lastUpdated = new Date(Date.now() - 120000); // 2 minutes ago
      render(RefreshControl, { lastUpdated });
      
      expect(screen.getByText('Updated 2m ago')).toBeInTheDocument();
    });

    it('formats hours ago correctly', () => {
      const lastUpdated = new Date(Date.now() - 7200000); // 2 hours ago
      render(RefreshControl, { lastUpdated });
      
      expect(screen.getByText('Updated 2h ago')).toBeInTheDocument();
    });

    it('shows date for old updates', () => {
      const lastUpdated = new Date(Date.now() - 86400000 * 2); // 2 days ago
      render(RefreshControl, { lastUpdated });
      
      const element = screen.getByText(/Updated/);
      expect(element.textContent).toContain(lastUpdated.toLocaleDateString());
    });

    it('hides timestamp when showLastUpdated is false', () => {
      const lastUpdated = new Date();
      render(RefreshControl, { lastUpdated, showLastUpdated: false });
      
      expect(screen.queryByText(/Updated/)).not.toBeInTheDocument();
    });

    it('shows full timestamp in tooltip', () => {
      const lastUpdated = new Date('2024-01-01T12:00:00Z');
      const { container } = render(RefreshControl, { lastUpdated });
      
      const element = container.querySelector('.last-updated');
      expect(element).toHaveAttribute('title', lastUpdated.toLocaleString());
    });
  });

  describe('Loading state indicator', () => {
    it('shows loading state when loading prop is true', () => {
      const { container } = render(RefreshControl, { loading: true });
      
      const spinner = container.querySelector('.spinning');
      expect(spinner).toBeInTheDocument();
    });

    it('disables buttons when loading', () => {
      render(RefreshControl, { loading: true });
      
      const refreshButton = screen.getByLabelText('Refresh data');
      expect(refreshButton).toBeDisabled();
    });

    it('stops auto-refresh when loading', async () => {
      const { component } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 1000 
      });
      
      // Enable loading
      await component.$set({ loading: true });
      
      // Advance time
      vi.advanceTimersByTime(2000);
      
      // Should not dispatch refresh because loading is true
      component.$on('refresh', mockDispatch);
      expect(mockDispatch).not.toHaveBeenCalled();
    });
  });

  describe('Auto-refresh functionality', () => {
    it('starts countdown when auto-refresh enabled', async () => {
      const { container } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 3000 
      });
      
      await tick();
      
      const countdown = container.querySelector('.countdown');
      expect(countdown?.textContent).toBe('3s');
    });

    it('triggers refresh when countdown reaches zero', async () => {
      const { component } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 2000 
      });
      
      component.$on('refresh', mockDispatch);
      
      // Advance time to trigger refresh
      vi.advanceTimersByTime(2000);
      
      expect(mockDispatch).toHaveBeenCalled();
    });

    it('resets countdown after refresh', async () => {
      const { component, container } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 2000 
      });
      
      component.$on('refresh', mockDispatch);
      
      // Advance time to trigger first refresh
      vi.advanceTimersByTime(2000);
      
      // Advance timer to see countdown reset
      vi.advanceTimersByTime(1000);
      
      const countdown = container.querySelector('.countdown');
      expect(countdown?.textContent).toBe('1s');
    });
  });

  describe('Pause refresh on tab inactive', () => {
    it('pauses auto-refresh when document becomes hidden', async () => {
      const { component } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 1000,
        pauseOnHidden: true 
      });
      
      component.$on('refresh', mockDispatch);
      
      // Simulate document becoming hidden
      Object.defineProperty(document, 'hidden', { value: true });
      const visibilityEvent = new Event('visibilitychange');
      document.dispatchEvent(visibilityEvent);
      
      // Advance time - should not trigger refresh
      vi.advanceTimersByTime(2000);
      
      expect(mockDispatch).not.toHaveBeenCalled();
    });

    it('resumes auto-refresh when document becomes visible', async () => {
      const { component } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 1000,
        pauseOnHidden: true 
      });
      
      component.$on('refresh', mockDispatch);
      
      // Hide document
      Object.defineProperty(document, 'hidden', { value: true });
      document.dispatchEvent(new Event('visibilitychange'));
      
      // Show document again
      Object.defineProperty(document, 'hidden', { value: false });
      document.dispatchEvent(new Event('visibilitychange'));
      
      // Should resume and trigger refresh
      vi.advanceTimersByTime(1000);
      
      expect(mockDispatch).toHaveBeenCalled();
    });

    it('does not pause when pauseOnHidden is false', async () => {
      const { component } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 1000,
        pauseOnHidden: false 
      });
      
      component.$on('refresh', mockDispatch);
      
      // Hide document
      Object.defineProperty(document, 'hidden', { value: true });
      document.dispatchEvent(new Event('visibilitychange'));
      
      // Should still trigger refresh
      vi.advanceTimersByTime(1000);
      
      expect(mockDispatch).toHaveBeenCalled();
    });

    it('registers visibility change listener on mount', () => {
      render(RefreshControl);
      
      expect(document.addEventListener).toHaveBeenCalledWith(
        'visibilitychange', 
        expect.any(Function)
      );
    });

    it('removes visibility change listener on destroy', async () => {
      const { component } = render(RefreshControl);
      
      // Destroy component
      component.$destroy();
      
      expect(document.removeEventListener).toHaveBeenCalledWith(
        'visibilitychange', 
        expect.any(Function)
      );
    });
  });

  describe('Cleanup on destroy', () => {
    it('clears interval on component destroy', async () => {
      const { component } = render(RefreshControl, { 
        autoRefresh: true, 
        interval: 1000 
      });
      
      const clearIntervalSpy = vi.spyOn(global, 'clearInterval');
      
      // Destroy component
      component.$destroy();
      
      expect(clearIntervalSpy).toHaveBeenCalled();
    });

    it('removes event listeners on destroy', async () => {
      const { component } = render(RefreshControl);
      
      component.$destroy();
      
      expect(document.removeEventListener).toHaveBeenCalledWith(
        'visibilitychange', 
        expect.any(Function)
      );
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA labels on buttons', () => {
      render(RefreshControl);
      
      expect(screen.getByLabelText('Refresh data')).toBeInTheDocument();
      expect(screen.getByTitle('Enable auto-refresh')).toBeInTheDocument();
      expect(screen.getByTitle('Set refresh interval')).toBeInTheDocument();
    });

    it('updates auto-refresh button title based on state', () => {
      const { container } = render(RefreshControl, { autoRefresh: true });
      
      const toggleButton = screen.getByTitle('Disable auto-refresh');
      expect(toggleButton).toBeInTheDocument();
    });
  });
});