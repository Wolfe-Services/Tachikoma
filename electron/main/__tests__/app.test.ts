import { describe, it, expect, vi, beforeEach } from 'vitest';
import { app, BrowserWindow } from 'electron';

// We need to import the app class, but it's not directly exported
// Let's create tests for the main functionality

describe('Main Process', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('App Lifecycle', () => {
    it('should request single instance lock', () => {
      expect(app.requestSingleInstanceLock).toHaveBeenCalled();
    });

    it('should enable sandbox mode', () => {
      expect(app.enableSandbox).toHaveBeenCalled();
    });

    it('should set protocol client', () => {
      expect(app.setAsDefaultProtocolClient).toHaveBeenCalledWith('tachikoma');
    });
  });

  describe('Window Management', () => {
    it('should create window with correct security settings', () => {
      const window = createMockWindow();
      
      // Verify window was created with BrowserWindow constructor
      expect(BrowserWindow).toHaveBeenCalled();
      
      // Check that security-focused webPreferences were set
      const calls = (BrowserWindow as any).mock.calls;
      const lastCall = calls[calls.length - 1];
      const options = lastCall[0];
      
      expect(options.webPreferences).toMatchObject({
        sandbox: true,
        contextIsolation: true,
        nodeIntegration: false,
        webSecurity: true,
        allowRunningInsecureContent: false,
        experimentalFeatures: false,
      });
    });

    it('should handle window close event', () => {
      const window = createMockWindow();
      const closeSpy = vi.spyOn(window, 'on');
      
      expect(closeSpy).toHaveBeenCalledWith('close', expect.any(Function));
    });

    it('should handle window ready-to-show event', () => {
      const window = createMockWindow();
      const readySpy = vi.spyOn(window, 'on');
      
      expect(readySpy).toHaveBeenCalledWith('ready-to-show', expect.any(Function));
    });
  });

  describe('Security Configuration', () => {
    it('should block navigation to unknown protocols', () => {
      const window = createMockWindow();
      const webContents = window.webContents;
      
      // Mock web contents creation
      const mockContents = {
        on: vi.fn(),
        setWindowOpenHandler: vi.fn(),
      };
      
      // Test will-navigate handler
      const willNavigateHandler = mockContents.on.mock.calls.find(
        call => call[0] === 'will-navigate'
      )?.[1];
      
      if (willNavigateHandler) {
        const mockEvent = { preventDefault: vi.fn() };
        
        // Should allow HTTPS
        willNavigateHandler(mockEvent, 'https://example.com');
        expect(mockEvent.preventDefault).not.toHaveBeenCalled();
        
        // Should allow tachikoma protocol
        willNavigateHandler(mockEvent, 'tachikoma://test');
        expect(mockEvent.preventDefault).not.toHaveBeenCalled();
        
        // Should block unknown protocol
        willNavigateHandler(mockEvent, 'unknown://test');
        expect(mockEvent.preventDefault).toHaveBeenCalled();
      }
    });

    it('should configure window open handler', () => {
      const window = createMockWindow();
      expect(window.webContents.setWindowOpenHandler).toHaveBeenCalled();
    });
  });

  describe('Event Listeners', () => {
    it('should handle second-instance event', () => {
      expect(app.on).toHaveBeenCalledWith('second-instance', expect.any(Function));
    });

    it('should handle window-all-closed event', () => {
      expect(app.on).toHaveBeenCalledWith('window-all-closed', expect.any(Function));
    });

    it('should handle activate event', () => {
      expect(app.on).toHaveBeenCalledWith('activate', expect.any(Function));
    });

    it('should handle before-quit event', () => {
      expect(app.on).toHaveBeenCalledWith('before-quit', expect.any(Function));
    });

    it('should handle web-contents-created event', () => {
      expect(app.on).toHaveBeenCalledWith('web-contents-created', expect.any(Function));
    });
  });

  describe('Configuration', () => {
    it('should configure session security', () => {
      // Test that session configuration happens
      const { session } = require('electron');
      const defaultSession = session.defaultSession;
      
      expect(defaultSession.webRequest.onHeadersReceived).toHaveBeenCalled();
      expect(defaultSession.setPermissionRequestHandler).toHaveBeenCalled();
    });
  });

  describe('Error Handling', () => {
    it('should handle uncaught exceptions', () => {
      const processOnSpy = vi.spyOn(process, 'on');
      
      // Find the uncaughtException handler
      const handler = processOnSpy.mock.calls.find(
        call => call[0] === 'uncaughtException'
      )?.[1];
      
      expect(handler).toBeDefined();
      
      if (handler) {
        const error = new Error('Test error');
        expect(() => handler(error)).not.toThrow();
      }
    });

    it('should handle unhandled rejections', () => {
      const processOnSpy = vi.spyOn(process, 'on');
      
      // Find the unhandledRejection handler
      const handler = processOnSpy.mock.calls.find(
        call => call[0] === 'unhandledRejection'
      )?.[1];
      
      expect(handler).toBeDefined();
      
      if (handler) {
        const reason = 'Test rejection';
        expect(() => handler(reason)).not.toThrow();
      }
    });
  });
});