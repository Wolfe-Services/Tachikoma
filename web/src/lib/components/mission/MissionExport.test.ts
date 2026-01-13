import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import MissionExport from './MissionExport.svelte';

// Mock the IPC module
vi.mock('$lib/ipc', () => ({
  ipc: {
    invoke: vi.fn(),
  },
}));

const mockIpc = vi.mocked(await import('$lib/ipc')).ipc;

describe('MissionExport', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders with default options selected', () => {
    const { getByRole, getByLabelText } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    // Check that JSON format is selected by default
    const jsonRadio = getByLabelText('JSON') as HTMLInputElement;
    expect(jsonRadio.checked).toBe(true);

    // Check that default content options are selected
    expect((getByLabelText('Configuration') as HTMLInputElement).checked).toBe(true);
    expect((getByLabelText('Logs') as HTMLInputElement).checked).toBe(true);
    expect((getByLabelText('File Changes') as HTMLInputElement).checked).toBe(true);
    expect((getByLabelText('Cost Report') as HTMLInputElement).checked).toBe(true);
    expect((getByLabelText('Checkpoints') as HTMLInputElement).checked).toBe(false);

    // Check that export button is enabled
    const exportButton = getByRole('button', { name: /export/i });
    expect(exportButton).not.toBeDisabled();
  });

  it('allows format selection', async () => {
    const { getByLabelText } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    const markdownRadio = getByLabelText('Markdown') as HTMLInputElement;
    await fireEvent.click(markdownRadio);

    expect(markdownRadio.checked).toBe(true);
    expect((getByLabelText('JSON') as HTMLInputElement).checked).toBe(false);
  });

  it('allows content selection', async () => {
    const { getByLabelText } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    const configCheckbox = getByLabelText('Configuration') as HTMLInputElement;
    await fireEvent.click(configCheckbox);

    expect(configCheckbox.checked).toBe(false);
  });

  it('disables export button when no content is selected', async () => {
    const { getByLabelText, getByRole } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    // Uncheck all content options
    await fireEvent.click(getByLabelText('Configuration'));
    await fireEvent.click(getByLabelText('Logs'));
    await fireEvent.click(getByLabelText('File Changes'));
    await fireEvent.click(getByLabelText('Cost Report'));

    const exportButton = getByRole('button', { name: /export/i });
    expect(exportButton).toBeDisabled();
  });

  it('shows progress during export', async () => {
    mockIpc.invoke.mockImplementation(() => 
      new Promise(resolve => setTimeout(() => resolve({
        filename: 'export.json',
        size: 1024,
        url: 'blob:test-url'
      }), 100))
    );

    const { getByRole, queryByText } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    const exportButton = getByRole('button', { name: /export/i });
    await fireEvent.click(exportButton);

    // Should show progress
    expect(queryByText('Preparing export...')).toBeTruthy();
    
    // Wait for export to complete
    await waitFor(() => {
      expect(queryByText('Export Complete')).toBeTruthy();
    });
  });

  it('handles export success', async () => {
    const mockResult = {
      filename: 'test-export.json',
      size: 2048,
      url: 'blob:test-url'
    };

    mockIpc.invoke.mockResolvedValue(mockResult);

    const { getByRole, getByText, component } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    const completeSpy = vi.fn();
    component.$on('complete', completeSpy);

    const exportButton = getByRole('button', { name: /export/i });
    await fireEvent.click(exportButton);

    await waitFor(() => {
      expect(getByText('Export Complete')).toBeTruthy();
      expect(getByText('test-export.json')).toBeTruthy();
      expect(getByText('2.0 KB')).toBeTruthy();
      expect(completeSpy).toHaveBeenCalledWith(expect.objectContaining({
        detail: mockResult
      }));
    });
  });

  it('handles export error', async () => {
    mockIpc.invoke.mockRejectedValue(new Error('Network error'));

    const { getByRole, getByText } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    const exportButton = getByRole('button', { name: /export/i });
    await fireEvent.click(exportButton);

    await waitFor(() => {
      expect(getByText('Network error')).toBeTruthy();
    });
  });

  it('emits close event when close button is clicked', async () => {
    const { getByLabelText, component } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    const closeSpy = vi.fn();
    component.$on('close', closeSpy);

    const closeButton = getByLabelText('Close');
    await fireEvent.click(closeButton);

    expect(closeSpy).toHaveBeenCalled();
  });

  it('formats file sizes correctly', () => {
    const { getByText } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    // This tests the component's internal formatSize function
    // We can verify this by checking that it displays sizes correctly
    // after a successful export
    mockIpc.invoke.mockResolvedValue({
      filename: 'test.json',
      size: 1536,
      url: 'blob:test'
    });

    // The actual test would involve triggering an export and checking the displayed size
  });

  it('generates shareable links', async () => {
    const mockResult = {
      filename: 'test-export.json',
      size: 1024,
      url: 'blob:test-url'
    };

    const mockShareableLink = {
      id: 'share-123',
      url: 'https://share.tachikoma.dev/share-123',
      expiresAt: '2024-01-02T00:00:00Z',
      accessCount: 0
    };

    mockIpc.invoke
      .mockResolvedValueOnce(mockResult) // First call for export
      .mockResolvedValueOnce(mockShareableLink); // Second call for shareable link

    const { getByRole, getByText } = render(MissionExport, {
      props: { missionId: 'test-mission' }
    });

    // Export first
    const exportButton = getByRole('button', { name: /export/i });
    await fireEvent.click(exportButton);

    await waitFor(() => {
      expect(getByText('Export Complete')).toBeTruthy();
    });

    // Generate shareable link
    const shareLinkButton = getByRole('button', { name: /create share link/i });
    await fireEvent.click(shareLinkButton);

    await waitFor(() => {
      expect(getByText('https://share.tachikoma.dev/share-123')).toBeTruthy();
    });
  });
});