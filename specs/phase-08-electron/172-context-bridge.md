# Spec 172: Context Bridge

## Phase
8 - Electron Shell

## Spec ID
172

## Status
Planned

## Dependencies
- Spec 169 (Security Configuration)
- Spec 170 (IPC Channels)
- Spec 171 (Preload Scripts)

## Estimated Context
~8%

---

## Objective

Implement a secure Context Bridge API that safely exposes functionality from the main process to the renderer. This provides React components with type-safe access to native capabilities while maintaining security boundaries.

---

## Acceptance Criteria

- [ ] Type-safe API exposed via contextBridge
- [ ] React hooks for consuming the API
- [ ] Proper cleanup of event listeners
- [ ] Error boundary integration
- [ ] Loading states for async operations
- [ ] Mock implementations for development/testing
- [ ] TypeScript declarations for window.electronAPI
- [ ] API versioning support

---

## Implementation Details

### Context Bridge Type Declarations

```typescript
// src/shared/electron-api.d.ts
import type { ElectronAPI } from '../electron/preload';

declare global {
  interface Window {
    electronAPI?: ElectronAPI;
  }
}

export {};
```

### Electron Context Provider

```typescript
// src/renderer/context/ElectronContext.tsx
import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  ReactNode,
} from 'react';

// Re-export types from preload
export interface AppInfo {
  name: string;
  version: string;
  electron: string;
  chrome: string;
  node: string;
  platform: NodeJS.Platform;
  arch: string;
  isPackaged: boolean;
}

export interface FileInfo {
  name: string;
  path: string;
  size: number;
  isDirectory: boolean;
  isFile: boolean;
  modified: Date;
}

interface ElectronContextValue {
  isElectron: boolean;
  platform: NodeJS.Platform | null;
  appInfo: AppInfo | null;
  isLoading: boolean;

  // Window operations
  minimizeWindow: () => void;
  maximizeWindow: () => void;
  closeWindow: () => void;

  // File system
  openFile: (options?: { filters?: Array<{ name: string; extensions: string[] }> }) => Promise<string[]>;
  saveFile: (options?: { defaultPath?: string }) => Promise<string | null>;
  readFile: (path: string) => Promise<string>;
  writeFile: (path: string, content: string) => Promise<void>;

  // Dialogs
  showConfirm: (message: string, detail?: string) => Promise<boolean>;
  showAlert: (message: string, detail?: string) => Promise<void>;
  showError: (message: string, detail?: string) => Promise<void>;

  // Shell
  openExternal: (url: string) => Promise<void>;
  showItemInFolder: (path: string) => void;

  // Theme
  isDarkMode: boolean;
}

const ElectronContext = createContext<ElectronContextValue | null>(null);

interface ElectronProviderProps {
  children: ReactNode;
  fallback?: ReactNode;
}

export function ElectronProvider({ children, fallback }: ElectronProviderProps): JSX.Element {
  const [isLoading, setIsLoading] = useState(true);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [isDarkMode, setIsDarkMode] = useState(false);

  const isElectron = typeof window !== 'undefined' && !!window.electronAPI;
  const platform = isElectron ? window.electronAPI!.platform : null;

  useEffect(() => {
    const initialize = async () => {
      if (!isElectron) {
        setIsLoading(false);
        return;
      }

      try {
        const info = await window.electronAPI!.getAppInfo();
        setAppInfo(info);

        // Check initial theme
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        setIsDarkMode(prefersDark);
      } catch (error) {
        console.error('Failed to initialize Electron context:', error);
      } finally {
        setIsLoading(false);
      }
    };

    initialize();
  }, [isElectron]);

  // Listen for theme changes
  useEffect(() => {
    if (!isElectron) return;

    const cleanup = window.electronAPI!.onThemeChange((isDark) => {
      setIsDarkMode(isDark);
    });

    return cleanup;
  }, [isElectron]);

  // Window operations
  const minimizeWindow = useCallback(() => {
    if (isElectron) {
      window.electronAPI!.minimizeWindow();
    }
  }, [isElectron]);

  const maximizeWindow = useCallback(() => {
    if (isElectron) {
      window.electronAPI!.maximizeWindow();
    }
  }, [isElectron]);

  const closeWindow = useCallback(() => {
    if (isElectron) {
      window.electronAPI!.closeWindow();
    }
  }, [isElectron]);

  // File operations
  const openFile = useCallback(
    async (options?: { filters?: Array<{ name: string; extensions: string[] }> }) => {
      if (!isElectron) return [];
      return window.electronAPI!.dialog.openFile(options);
    },
    [isElectron]
  );

  const saveFile = useCallback(
    async (options?: { defaultPath?: string }) => {
      if (!isElectron) return null;
      return window.electronAPI!.dialog.saveFile(options);
    },
    [isElectron]
  );

  const readFile = useCallback(
    async (path: string) => {
      if (!isElectron) throw new Error('Not in Electron environment');
      const result = await window.electronAPI!.fs.readFile(path, { encoding: 'utf-8' });
      return result as string;
    },
    [isElectron]
  );

  const writeFile = useCallback(
    async (path: string, content: string) => {
      if (!isElectron) throw new Error('Not in Electron environment');
      await window.electronAPI!.fs.writeFile(path, content);
    },
    [isElectron]
  );

  // Dialogs
  const showConfirm = useCallback(
    async (message: string, detail?: string) => {
      if (!isElectron) return window.confirm(message);
      return window.electronAPI!.dialog.confirm(message, detail);
    },
    [isElectron]
  );

  const showAlert = useCallback(
    async (message: string, detail?: string) => {
      if (!isElectron) {
        window.alert(message);
        return;
      }
      await window.electronAPI!.dialog.alert(message, detail);
    },
    [isElectron]
  );

  const showError = useCallback(
    async (message: string, detail?: string) => {
      if (!isElectron) {
        window.alert(`Error: ${message}`);
        return;
      }
      await window.electronAPI!.dialog.error(message, detail);
    },
    [isElectron]
  );

  // Shell
  const openExternal = useCallback(
    async (url: string) => {
      if (!isElectron) {
        window.open(url, '_blank');
        return;
      }
      await window.electronAPI!.shell.openExternal(url);
    },
    [isElectron]
  );

  const showItemInFolder = useCallback(
    (path: string) => {
      if (isElectron) {
        window.electronAPI!.shell.showItemInFolder(path);
      }
    },
    [isElectron]
  );

  const value: ElectronContextValue = {
    isElectron,
    platform,
    appInfo,
    isLoading,
    minimizeWindow,
    maximizeWindow,
    closeWindow,
    openFile,
    saveFile,
    readFile,
    writeFile,
    showConfirm,
    showAlert,
    showError,
    openExternal,
    showItemInFolder,
    isDarkMode,
  };

  if (isLoading && fallback) {
    return <>{fallback}</>;
  }

  return (
    <ElectronContext.Provider value={value}>
      {children}
    </ElectronContext.Provider>
  );
}

export function useElectron(): ElectronContextValue {
  const context = useContext(ElectronContext);

  if (!context) {
    throw new Error('useElectron must be used within an ElectronProvider');
  }

  return context;
}

export function useIsElectron(): boolean {
  const context = useContext(ElectronContext);
  return context?.isElectron ?? false;
}
```

### Specialized Hooks

```typescript
// src/renderer/hooks/useFileSystem.ts
import { useState, useCallback } from 'react';
import { useElectron } from '../context/ElectronContext';

interface UseFileSystemOptions {
  onError?: (error: Error) => void;
}

interface FileSystemState {
  isLoading: boolean;
  error: Error | null;
}

export function useFileSystem(options: UseFileSystemOptions = {}) {
  const { isElectron } = useElectron();
  const [state, setState] = useState<FileSystemState>({
    isLoading: false,
    error: null,
  });

  const readFile = useCallback(
    async (path: string): Promise<string | null> => {
      if (!isElectron) {
        console.warn('File system not available outside Electron');
        return null;
      }

      setState({ isLoading: true, error: null });

      try {
        const content = await window.electronAPI!.fs.readFile(path, {
          encoding: 'utf-8',
        });
        setState({ isLoading: false, error: null });
        return content as string;
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        setState({ isLoading: false, error: err });
        options.onError?.(err);
        return null;
      }
    },
    [isElectron, options]
  );

  const writeFile = useCallback(
    async (path: string, content: string): Promise<boolean> => {
      if (!isElectron) {
        console.warn('File system not available outside Electron');
        return false;
      }

      setState({ isLoading: true, error: null });

      try {
        await window.electronAPI!.fs.writeFile(path, content);
        setState({ isLoading: false, error: null });
        return true;
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        setState({ isLoading: false, error: err });
        options.onError?.(err);
        return false;
      }
    },
    [isElectron, options]
  );

  const deleteFile = useCallback(
    async (path: string): Promise<boolean> => {
      if (!isElectron) return false;

      setState({ isLoading: true, error: null });

      try {
        await window.electronAPI!.fs.deleteFile(path);
        setState({ isLoading: false, error: null });
        return true;
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        setState({ isLoading: false, error: err });
        options.onError?.(err);
        return false;
      }
    },
    [isElectron, options]
  );

  const readDirectory = useCallback(
    async (path: string) => {
      if (!isElectron) return [];

      setState({ isLoading: true, error: null });

      try {
        const files = await window.electronAPI!.fs.readDirectory(path);
        setState({ isLoading: false, error: null });
        return files;
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        setState({ isLoading: false, error: err });
        options.onError?.(err);
        return [];
      }
    },
    [isElectron, options]
  );

  const exists = useCallback(
    async (path: string): Promise<boolean> => {
      if (!isElectron) return false;

      try {
        return await window.electronAPI!.fs.exists(path);
      } catch {
        return false;
      }
    },
    [isElectron]
  );

  return {
    ...state,
    readFile,
    writeFile,
    deleteFile,
    readDirectory,
    exists,
  };
}
```

### File Watcher Hook

```typescript
// src/renderer/hooks/useFileWatcher.ts
import { useEffect, useRef, useCallback } from 'react';
import { useElectron } from '../context/ElectronContext';

interface FileWatchEvent {
  eventType: 'change' | 'rename';
  filename: string;
}

interface UseFileWatcherOptions {
  onEvent?: (event: FileWatchEvent) => void;
  onChange?: (filename: string) => void;
  onRename?: (filename: string) => void;
  enabled?: boolean;
}

export function useFileWatcher(
  path: string | null,
  options: UseFileWatcherOptions = {}
) {
  const { isElectron } = useElectron();
  const cleanupRef = useRef<(() => void) | null>(null);
  const { onEvent, onChange, onRename, enabled = true } = options;

  const handleEvent = useCallback(
    (event: FileWatchEvent) => {
      onEvent?.(event);

      if (event.eventType === 'change') {
        onChange?.(event.filename);
      } else if (event.eventType === 'rename') {
        onRename?.(event.filename);
      }
    },
    [onEvent, onChange, onRename]
  );

  useEffect(() => {
    if (!isElectron || !path || !enabled) {
      return;
    }

    const setupWatcher = async () => {
      try {
        const cleanup = await window.electronAPI!.fs.watch(path, handleEvent);
        cleanupRef.current = cleanup;
      } catch (error) {
        console.error('Failed to setup file watcher:', error);
      }
    };

    setupWatcher();

    return () => {
      if (cleanupRef.current) {
        cleanupRef.current();
        cleanupRef.current = null;
      }
    };
  }, [isElectron, path, enabled, handleEvent]);

  const stop = useCallback(() => {
    if (cleanupRef.current) {
      cleanupRef.current();
      cleanupRef.current = null;
    }
  }, []);

  return { stop };
}
```

### Update Hook

```typescript
// src/renderer/hooks/useUpdater.ts
import { useState, useEffect, useCallback } from 'react';
import { useElectron } from '../context/ElectronContext';

interface UpdateInfo {
  version: string;
  releaseDate: string;
  releaseNotes?: string;
}

interface UpdateProgress {
  percent: number;
  bytesPerSecond: number;
  total: number;
  transferred: number;
}

type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';

export function useUpdater() {
  const { isElectron } = useElectron();
  const [status, setStatus] = useState<UpdateStatus>('idle');
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isElectron) return;

    const cleanups: Array<() => void> = [];

    cleanups.push(
      window.electronAPI!.updater.onChecking(() => {
        setStatus('checking');
        setError(null);
      })
    );

    cleanups.push(
      window.electronAPI!.updater.onAvailable((info) => {
        setStatus('available');
        setUpdateInfo(info);
      })
    );

    cleanups.push(
      window.electronAPI!.updater.onNotAvailable(() => {
        setStatus('idle');
      })
    );

    cleanups.push(
      window.electronAPI!.updater.onProgress((progressInfo) => {
        setStatus('downloading');
        setProgress(progressInfo);
      })
    );

    cleanups.push(
      window.electronAPI!.updater.onDownloaded((info) => {
        setStatus('ready');
        setUpdateInfo(info);
        setProgress(null);
      })
    );

    cleanups.push(
      window.electronAPI!.updater.onError((err) => {
        setStatus('error');
        setError(err.message);
      })
    );

    return () => {
      cleanups.forEach((cleanup) => cleanup());
    };
  }, [isElectron]);

  const checkForUpdates = useCallback(
    async (silent = false) => {
      if (!isElectron) return;
      await window.electronAPI!.updater.check(silent);
    },
    [isElectron]
  );

  const downloadUpdate = useCallback(() => {
    if (!isElectron) return;
    window.electronAPI!.updater.download();
  }, [isElectron]);

  const installUpdate = useCallback(() => {
    if (!isElectron) return;
    window.electronAPI!.updater.install();
  }, [isElectron]);

  return {
    status,
    updateInfo,
    progress,
    error,
    checkForUpdates,
    downloadUpdate,
    installUpdate,
  };
}
```

### Mock Implementation for Testing

```typescript
// src/renderer/mocks/electronAPI.ts
import type { ElectronAPI } from '../../electron/preload';

export const createMockElectronAPI = (): ElectronAPI => ({
  platform: 'darwin',
  isPackaged: false,

  getAppInfo: async () => ({
    name: 'Tachikoma',
    version: '1.0.0-mock',
    electron: '25.0.0',
    chrome: '114.0.0',
    node: '18.0.0',
    platform: 'darwin' as const,
    arch: 'x64',
    isPackaged: false,
  }),

  getAppPath: async () => '/mock/path',
  quit: async () => {},
  restart: async () => {},

  minimizeWindow: async () => {},
  maximizeWindow: async () => {},
  closeWindow: async () => {},
  isWindowMaximized: async () => false,
  setWindowTitle: async () => {},
  onWindowMaximize: () => () => {},

  fs: {
    exists: async () => true,
    stat: async (path) => ({
      name: 'mock-file.txt',
      path,
      size: 1024,
      isDirectory: false,
      isFile: true,
      isSymlink: false,
      created: new Date(),
      modified: new Date(),
      accessed: new Date(),
      extension: '.txt',
    }),
    readFile: async () => 'mock file content',
    writeFile: async () => {},
    deleteFile: async () => {},
    readDirectory: async () => [],
    createDirectory: async () => {},
    watch: async () => () => {},
    getAppPath: async () => '/mock/app',
  },

  dialog: {
    openFile: async () => ['/mock/selected/file.txt'],
    saveFile: async () => '/mock/saved/file.txt',
    showMessage: async () => ({ response: 0, checkboxChecked: false }),
    confirm: async () => true,
    alert: async () => {},
    error: async () => {},
  },

  menu: {
    updateState: async () => {},
    showContextMenu: async () => {},
  },

  updater: {
    check: async () => {},
    download: async () => {},
    install: async () => {},
    getState: async () => ({
      checking: false,
      available: false,
      downloading: false,
      downloaded: false,
      error: null,
      updateInfo: null,
      progress: null,
    }),
    onChecking: () => () => {},
    onAvailable: () => () => {},
    onNotAvailable: () => () => {},
    onProgress: () => () => {},
    onDownloaded: () => () => {},
    onError: () => () => {},
  },

  notification: {
    show: async () => 'notification-id',
    onClick: () => () => {},
    onClose: () => () => {},
  },

  reportException: () => {},
  reportRejection: () => {},

  onSuspend: () => () => {},
  onResume: () => () => {},
  onThemeChange: () => () => {},
  onDeepLink: () => () => {},
  onConnectivity: () => () => {},

  shell: {
    openExternal: async () => {},
    openPath: async () => {},
    showItemInFolder: () => {},
  },

  clipboard: {
    readText: () => '',
    writeText: () => {},
    readImage: () => null,
    writeImage: () => {},
  },
});

// Install mock for testing
export function installMockElectronAPI(): void {
  if (typeof window !== 'undefined') {
    (window as any).electronAPI = createMockElectronAPI();
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/renderer/context/__tests__/ElectronContext.test.tsx
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import { ElectronProvider, useElectron } from '../ElectronContext';
import { createMockElectronAPI } from '../../mocks/electronAPI';

const TestComponent = () => {
  const { isElectron, appInfo, platform } = useElectron();
  return (
    <div>
      <span data-testid="is-electron">{String(isElectron)}</span>
      <span data-testid="platform">{platform}</span>
      <span data-testid="version">{appInfo?.version}</span>
    </div>
  );
};

describe('ElectronContext', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (window as any).electronAPI = undefined;
  });

  it('should detect when not in Electron', async () => {
    render(
      <ElectronProvider>
        <TestComponent />
      </ElectronProvider>
    );

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
    });

    expect(screen.getByTestId('is-electron')).toHaveTextContent('false');
  });

  it('should detect when in Electron', async () => {
    (window as any).electronAPI = createMockElectronAPI();

    render(
      <ElectronProvider>
        <TestComponent />
      </ElectronProvider>
    );

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    expect(screen.getByTestId('is-electron')).toHaveTextContent('true');
    expect(screen.getByTestId('platform')).toHaveTextContent('darwin');
    expect(screen.getByTestId('version')).toHaveTextContent('1.0.0-mock');
  });

  it('should throw when useElectron is used outside provider', () => {
    expect(() => {
      render(<TestComponent />);
    }).toThrow('useElectron must be used within an ElectronProvider');
  });
});
```

---

## Related Specs

- Spec 169: Security Configuration
- Spec 170: IPC Channels
- Spec 171: Preload Scripts
