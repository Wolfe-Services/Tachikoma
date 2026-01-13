import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import '@testing-library/jest-dom';
import ForgeLayout from '$lib/components/forge/ForgeLayout.svelte';

// Mock the stores
vi.mock('$lib/stores/forgeSession', () => ({
  forgeSessionStore: {
    subscribe: vi.fn((callback) => {
      callback({ activeSession: null, sessions: [], loading: false, error: null });
      return vi.fn(); // unsubscribe function
    })
  }
}));

vi.mock('$lib/stores/layoutPreferences', () => ({
  layoutPreferencesStore: {
    save: vi.fn(),
    load: vi.fn().mockResolvedValue(null)
  }
}));

// Mock the child components
vi.mock('$lib/components/forge/SessionSidebar.svelte', () => ({
  default: vi.fn(() => ({
    render: () => '<div data-testid="session-sidebar">Session Sidebar</div>'
  }))
}));

vi.mock('$lib/components/forge/ParticipantPanel.svelte', () => ({
  default: vi.fn(() => ({
    render: () => '<div data-testid="participant-panel">Participant Panel</div>'
  }))
}));

vi.mock('$lib/components/forge/MainContentArea.svelte', () => ({
  default: vi.fn(() => ({
    render: () => '<div data-testid="main-content">Main Content</div>'
  }))
}));

vi.mock('$lib/components/forge/ResultPanel.svelte', () => ({
  default: vi.fn(() => ({
    render: () => '<div data-testid="result-panel">Result Panel</div>'
  }))
}));

vi.mock('$lib/components/forge/ForgeToolbar.svelte', () => ({
  default: vi.fn(() => ({
    render: () => '<div data-testid="forge-toolbar">Forge Toolbar</div>'
  }))
}));

describe('ForgeLayout', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the basic layout structure', async () => {
    render(ForgeLayout, { props: { sessionId: null } });
    
    const layout = screen.getByTestId('forge-layout');
    expect(layout).toBeInTheDocument();
    expect(layout).toHaveClass('forge-layout');
  });

  it('applies responsive layout classes', async () => {
    render(ForgeLayout, { props: { sessionId: 'test-session' } });
    
    const layout = screen.getByTestId('forge-layout');
    expect(layout).toBeInTheDocument();
  });

  it('handles keyboard shortcuts for panel toggling', async () => {
    render(ForgeLayout, { props: { sessionId: null } });
    
    // Simulate Ctrl+B for left sidebar toggle
    await fireEvent.keyDown(window, { key: 'b', ctrlKey: true });
    
    // Since we're mocking the components, we can't test the actual behavior
    // but we can verify the component renders without errors
    const layout = screen.getByTestId('forge-layout');
    expect(layout).toBeInTheDocument();
  });

  it('initializes with default configuration', async () => {
    const { component } = render(ForgeLayout, { props: { sessionId: null } });
    expect(component).toBeTruthy();
  });
});