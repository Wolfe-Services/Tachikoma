import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ipcMain } from 'electron';
import { registerIpcHandlers } from '../ipc-handlers';

// Mock the native module
const mockNative = {
  startMission: vi.fn(),
  stopMission: vi.fn(),
  getMissionStatus: vi.fn(),
  listSpecs: vi.fn(),
  readSpec: vi.fn(),
  getConfig: vi.fn(),
  setConfig: vi.fn(),
};

vi.mock('../native', () => ({
  native: mockNative,
}));

describe('IPC Handlers', () => {
  let handleSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();
    handleSpy = vi.spyOn(ipcMain, 'handle');
  });

  it('should register all IPC handlers', () => {
    registerIpcHandlers();

    expect(handleSpy).toHaveBeenCalledWith('mission:start', expect.any(Function));
    expect(handleSpy).toHaveBeenCalledWith('mission:stop', expect.any(Function));
    expect(handleSpy).toHaveBeenCalledWith('mission:status', expect.any(Function));
    expect(handleSpy).toHaveBeenCalledWith('spec:list', expect.any(Function));
    expect(handleSpy).toHaveBeenCalledWith('spec:read', expect.any(Function));
    expect(handleSpy).toHaveBeenCalledWith('config:get', expect.any(Function));
    expect(handleSpy).toHaveBeenCalledWith('config:set', expect.any(Function));
  });

  describe('Mission Handlers', () => {
    beforeEach(() => {
      registerIpcHandlers();
    });

    it('should handle mission:start', async () => {
      const mockEvent = createMockEvent();
      const request = {
        specPath: '/test/spec.md',
        backend: 'claude' as const,
        mode: 'development' as const,
      };
      
      mockNative.startMission.mockResolvedValue('test-mission-id');

      // Get the registered handler
      const handler = handleSpy.mock.calls.find(call => call[0] === 'mission:start')?.[1];
      expect(handler).toBeDefined();

      const result = await handler!(mockEvent, request);

      expect(mockNative.startMission).toHaveBeenCalledWith(
        request.specPath,
        request.backend,
        request.mode
      );
      expect(result).toEqual({ missionId: 'test-mission-id' });
    });

    it('should handle mission:stop', async () => {
      const mockEvent = createMockEvent();
      const request = { missionId: 'test-mission-id' };
      
      mockNative.stopMission.mockResolvedValue(true);

      const handler = handleSpy.mock.calls.find(call => call[0] === 'mission:stop')?.[1];
      const result = await handler!(mockEvent, request);

      expect(mockNative.stopMission).toHaveBeenCalledWith(request.missionId);
      expect(result).toEqual({ success: true });
    });

    it('should handle mission:status', async () => {
      const mockEvent = createMockEvent();
      const request = { missionId: 'test-mission-id' };
      const mockStatus = {
        id: 'test-mission',
        status: 'running',
        progress: 0.5,
        message: 'Test mission running',
      };
      
      mockNative.getMissionStatus.mockResolvedValue(mockStatus);

      const handler = handleSpy.mock.calls.find(call => call[0] === 'mission:status')?.[1];
      const result = await handler!(mockEvent, request);

      expect(mockNative.getMissionStatus).toHaveBeenCalledWith(request.missionId);
      expect(result).toEqual(mockStatus);
    });
  });

  describe('Spec Handlers', () => {
    beforeEach(() => {
      registerIpcHandlers();
    });

    it('should handle spec:list', async () => {
      const mockEvent = createMockEvent();
      const request = { path: '/test/specs' };
      const mockSpecs = {
        specs: [
          { path: '/test/spec1.md', title: 'Test Spec 1' },
          { path: '/test/spec2.md', title: 'Test Spec 2' },
        ],
      };
      
      mockNative.listSpecs.mockResolvedValue(mockSpecs);

      const handler = handleSpy.mock.calls.find(call => call[0] === 'spec:list')?.[1];
      const result = await handler!(mockEvent, request);

      expect(mockNative.listSpecs).toHaveBeenCalledWith(request.path);
      expect(result).toEqual(mockSpecs);
    });

    it('should handle spec:read', async () => {
      const mockEvent = createMockEvent();
      const request = { path: '/test/spec.md' };
      const mockSpec = {
        content: '# Test Spec\n\nTest content',
        metadata: { title: 'Test Spec', id: '001' },
      };
      
      mockNative.readSpec.mockResolvedValue(mockSpec);

      const handler = handleSpy.mock.calls.find(call => call[0] === 'spec:read')?.[1];
      const result = await handler!(mockEvent, request);

      expect(mockNative.readSpec).toHaveBeenCalledWith(request.path);
      expect(result).toEqual(mockSpec);
    });
  });

  describe('Config Handlers', () => {
    beforeEach(() => {
      registerIpcHandlers();
    });

    it('should handle config:get', async () => {
      const mockEvent = createMockEvent();
      const request = { key: 'test.key' };
      const mockConfig = { value: 'test-value' };
      
      mockNative.getConfig.mockResolvedValue(mockConfig);

      const handler = handleSpy.mock.calls.find(call => call[0] === 'config:get')?.[1];
      const result = await handler!(mockEvent, request);

      expect(mockNative.getConfig).toHaveBeenCalledWith(request.key);
      expect(result).toEqual(mockConfig);
    });

    it('should handle config:set', async () => {
      const mockEvent = createMockEvent();
      const request = { key: 'test.key', value: 'new-value' };
      
      mockNative.setConfig.mockResolvedValue(true);

      const handler = handleSpy.mock.calls.find(call => call[0] === 'config:set')?.[1];
      const result = await handler!(mockEvent, request);

      expect(mockNative.setConfig).toHaveBeenCalledWith(request.key, request.value);
      expect(result).toEqual({ success: true });
    });
  });

  describe('Error Handling', () => {
    beforeEach(() => {
      registerIpcHandlers();
    });

    it('should handle native module errors', async () => {
      const mockEvent = createMockEvent();
      const request = { missionId: 'test-mission-id' };
      const error = new Error('Native module error');
      
      mockNative.stopMission.mockRejectedValue(error);

      const handler = handleSpy.mock.calls.find(call => call[0] === 'mission:stop')?.[1];
      
      await expect(handler!(mockEvent, request)).rejects.toThrow(error);
    });
  });
});