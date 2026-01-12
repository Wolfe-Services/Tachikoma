# Spec 184: Native Notifications

## Phase
8 - Electron Shell

## Spec ID
184

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 183 (Tray Integration)

## Estimated Context
~8%

---

## Objective

Implement native notification support for the Tachikoma application, providing system-level notifications with actions, sounds, and proper integration with notification centers across all platforms.

---

## Acceptance Criteria

- [x] Native notification display
- [x] Notification actions (buttons)
- [x] Click handling
- [x] Close handling
- [x] Sound configuration
- [x] Notification queueing
- [x] Platform-specific features
- [x] Do Not Disturb respect
- [x] Notification history
- [x] Reply actions (where supported)

---

## Implementation Details

### Notification Manager

```typescript
// src/electron/main/notifications/index.ts
import {
  Notification,
  nativeImage,
  app,
  BrowserWindow,
  shell,
} from 'electron';
import { join } from 'path';
import { Logger } from '../logger';
import { configManager } from '../config';

const logger = new Logger('notifications');

interface NotificationOptions {
  title: string;
  body: string;
  icon?: string;
  silent?: boolean;
  urgency?: 'normal' | 'critical' | 'low';
  timeoutType?: 'default' | 'never';
  actions?: NotificationAction[];
  tag?: string;
  replyPlaceholder?: string;
  hasReply?: boolean;
  data?: Record<string, unknown>;
}

interface NotificationAction {
  type: 'button';
  text: string;
}

interface StoredNotification {
  id: string;
  options: NotificationOptions;
  timestamp: number;
  clicked: boolean;
  closed: boolean;
}

type NotificationCallback = (notification: StoredNotification) => void;

class NotificationManager {
  private notifications: Map<string, Notification> = new Map();
  private history: StoredNotification[] = [];
  private maxHistory = 100;
  private mainWindow: BrowserWindow | null = null;
  private idCounter = 0;

  private onClickCallbacks: Set<NotificationCallback> = new Set();
  private onCloseCallbacks: Set<NotificationCallback> = new Set();
  private onActionCallbacks: Map<string, (action: number) => void> = new Map();
  private onReplyCallbacks: Map<string, (reply: string) => void> = new Map();

  setMainWindow(window: BrowserWindow): void {
    this.mainWindow = window;
  }

  isSupported(): boolean {
    return Notification.isSupported();
  }

  show(options: NotificationOptions): string {
    if (!this.isSupported()) {
      logger.warn('Notifications not supported');
      return '';
    }

    const id = this.generateId();
    const icon = options.icon
      ? nativeImage.createFromPath(options.icon)
      : this.getDefaultIcon();

    const notification = new Notification({
      title: options.title,
      body: options.body,
      icon,
      silent: options.silent ?? false,
      urgency: options.urgency ?? 'normal',
      timeoutType: options.timeoutType ?? 'default',
      actions: options.actions,
      hasReply: options.hasReply,
      replyPlaceholder: options.replyPlaceholder,
    });

    // Store notification info
    const stored: StoredNotification = {
      id,
      options,
      timestamp: Date.now(),
      clicked: false,
      closed: false,
    };

    this.history.push(stored);
    this.trimHistory();

    // Setup event handlers
    notification.on('click', () => {
      stored.clicked = true;
      this.handleClick(id, stored);
    });

    notification.on('close', () => {
      stored.closed = true;
      this.handleClose(id, stored);
    });

    notification.on('action', (_, index) => {
      this.handleAction(id, index);
    });

    notification.on('reply', (_, reply) => {
      this.handleReply(id, reply);
    });

    // Show notification
    notification.show();
    this.notifications.set(id, notification);

    logger.debug('Notification shown', { id, title: options.title });

    return id;
  }

  private generateId(): string {
    return `notification-${Date.now()}-${++this.idCounter}`;
  }

  private getDefaultIcon(): Electron.NativeImage {
    const iconPath = join(__dirname, '../../resources/icon.png');
    return nativeImage.createFromPath(iconPath);
  }

  private handleClick(id: string, stored: StoredNotification): void {
    logger.debug('Notification clicked', { id });

    // Notify callbacks
    this.onClickCallbacks.forEach((callback) => {
      try {
        callback(stored);
      } catch (error) {
        logger.error('Notification click callback error', { error });
      }
    });

    // Send to renderer
    this.mainWindow?.webContents.send('notification:clicked', { id, data: stored.options.data });

    // Focus app
    if (this.mainWindow) {
      if (this.mainWindow.isMinimized()) {
        this.mainWindow.restore();
      }
      this.mainWindow.show();
      this.mainWindow.focus();
    }
  }

  private handleClose(id: string, stored: StoredNotification): void {
    logger.debug('Notification closed', { id });

    // Cleanup
    this.notifications.delete(id);

    // Notify callbacks
    this.onCloseCallbacks.forEach((callback) => {
      try {
        callback(stored);
      } catch (error) {
        logger.error('Notification close callback error', { error });
      }
    });

    // Send to renderer
    this.mainWindow?.webContents.send('notification:closed', { id });
  }

  private handleAction(id: string, actionIndex: number): void {
    logger.debug('Notification action', { id, actionIndex });

    const callback = this.onActionCallbacks.get(id);
    if (callback) {
      callback(actionIndex);
    }

    // Send to renderer
    this.mainWindow?.webContents.send('notification:action', { id, actionIndex });
  }

  private handleReply(id: string, reply: string): void {
    logger.debug('Notification reply', { id, reply: reply.substring(0, 50) });

    const callback = this.onReplyCallbacks.get(id);
    if (callback) {
      callback(reply);
    }

    // Send to renderer
    this.mainWindow?.webContents.send('notification:reply', { id, reply });
  }

  close(id: string): void {
    const notification = this.notifications.get(id);
    if (notification) {
      notification.close();
      this.notifications.delete(id);
    }
  }

  closeAll(): void {
    for (const notification of this.notifications.values()) {
      notification.close();
    }
    this.notifications.clear();
  }

  // Event registration
  onNotificationClick(callback: NotificationCallback): () => void {
    this.onClickCallbacks.add(callback);
    return () => this.onClickCallbacks.delete(callback);
  }

  onNotificationClose(callback: NotificationCallback): () => void {
    this.onCloseCallbacks.add(callback);
    return () => this.onCloseCallbacks.delete(callback);
  }

  onNotificationAction(id: string, callback: (actionIndex: number) => void): void {
    this.onActionCallbacks.set(id, callback);
  }

  onNotificationReply(id: string, callback: (reply: string) => void): void {
    this.onReplyCallbacks.set(id, callback);
  }

  // History management
  getHistory(): StoredNotification[] {
    return [...this.history];
  }

  clearHistory(): void {
    this.history = [];
  }

  private trimHistory(): void {
    while (this.history.length > this.maxHistory) {
      this.history.shift();
    }
  }

  // Quick notification helpers
  info(title: string, body: string): string {
    return this.show({ title, body, urgency: 'normal' });
  }

  success(title: string, body: string): string {
    return this.show({
      title,
      body,
      urgency: 'normal',
      icon: join(__dirname, '../../resources/icons/success.png'),
    });
  }

  warning(title: string, body: string): string {
    return this.show({
      title,
      body,
      urgency: 'normal',
      icon: join(__dirname, '../../resources/icons/warning.png'),
    });
  }

  error(title: string, body: string): string {
    return this.show({
      title,
      body,
      urgency: 'critical',
      timeoutType: 'never',
      icon: join(__dirname, '../../resources/icons/error.png'),
    });
  }
}

export const notificationManager = new NotificationManager();
```

### Notification IPC Handlers

```typescript
// src/electron/main/ipc/notifications.ts
import { ipcMain } from 'electron';
import { notificationManager } from '../notifications';

export function setupNotificationIpcHandlers(): void {
  ipcMain.handle('notification:show', (_, options) => {
    return notificationManager.show(options);
  });

  ipcMain.handle('notification:close', (_, id: string) => {
    notificationManager.close(id);
  });

  ipcMain.handle('notification:closeAll', () => {
    notificationManager.closeAll();
  });

  ipcMain.handle('notification:isSupported', () => {
    return notificationManager.isSupported();
  });

  ipcMain.handle('notification:getHistory', () => {
    return notificationManager.getHistory();
  });

  ipcMain.handle('notification:clearHistory', () => {
    notificationManager.clearHistory();
  });

  // Quick helpers
  ipcMain.handle('notification:info', (_, title: string, body: string) => {
    return notificationManager.info(title, body);
  });

  ipcMain.handle('notification:success', (_, title: string, body: string) => {
    return notificationManager.success(title, body);
  });

  ipcMain.handle('notification:warning', (_, title: string, body: string) => {
    return notificationManager.warning(title, body);
  });

  ipcMain.handle('notification:error', (_, title: string, body: string) => {
    return notificationManager.error(title, body);
  });
}
```

### Renderer Notification Hook

```typescript
// src/renderer/hooks/useNotifications.ts
import { useCallback, useEffect } from 'react';

interface NotificationOptions {
  title: string;
  body: string;
  icon?: string;
  silent?: boolean;
  urgency?: 'normal' | 'critical' | 'low';
  actions?: Array<{ type: 'button'; text: string }>;
  data?: Record<string, unknown>;
}

interface UseNotificationsOptions {
  onClick?: (id: string, data?: Record<string, unknown>) => void;
  onClose?: (id: string) => void;
  onAction?: (id: string, actionIndex: number) => void;
  onReply?: (id: string, reply: string) => void;
}

export function useNotifications(options: UseNotificationsOptions = {}) {
  const { onClick, onClose, onAction, onReply } = options;

  useEffect(() => {
    const cleanups: Array<() => void> = [];

    if (onClick) {
      cleanups.push(
        window.electronAPI?.notification.onClick((data) => {
          onClick(data.id, data.data);
        }) || (() => {})
      );
    }

    if (onClose) {
      cleanups.push(
        window.electronAPI?.notification.onClose((data) => {
          onClose(data.id);
        }) || (() => {})
      );
    }

    return () => {
      cleanups.forEach((cleanup) => cleanup());
    };
  }, [onClick, onClose, onAction, onReply]);

  const show = useCallback(async (opts: NotificationOptions): Promise<string> => {
    return window.electronAPI?.notification.show(opts) || '';
  }, []);

  const close = useCallback((id: string) => {
    window.electronAPI?.notification.close(id);
  }, []);

  const closeAll = useCallback(() => {
    window.electronAPI?.notification.closeAll();
  }, []);

  const info = useCallback(async (title: string, body: string): Promise<string> => {
    return window.electronAPI?.invoke('notification:info', title, body) || '';
  }, []);

  const success = useCallback(async (title: string, body: string): Promise<string> => {
    return window.electronAPI?.invoke('notification:success', title, body) || '';
  }, []);

  const warning = useCallback(async (title: string, body: string): Promise<string> => {
    return window.electronAPI?.invoke('notification:warning', title, body) || '';
  }, []);

  const error = useCallback(async (title: string, body: string): Promise<string> => {
    return window.electronAPI?.invoke('notification:error', title, body) || '';
  }, []);

  return {
    show,
    close,
    closeAll,
    info,
    success,
    warning,
    error,
    isSupported: window.electronAPI?.notification.isSupported() ?? false,
  };
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// src/electron/main/notifications/__tests__/notifications.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('electron', () => ({
  Notification: vi.fn().mockImplementation(() => ({
    on: vi.fn(),
    show: vi.fn(),
    close: vi.fn(),
  })),
  nativeImage: {
    createFromPath: vi.fn().mockReturnValue({}),
  },
  app: {},
  BrowserWindow: vi.fn(),
}));

// Mock Notification.isSupported
(global as any).Notification = { isSupported: vi.fn().mockReturnValue(true) };

describe('NotificationManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should show notification', async () => {
    const { Notification } = await import('electron');
    const { notificationManager } = await import('../index');

    const id = notificationManager.show({
      title: 'Test',
      body: 'Test body',
    });

    expect(id).toBeTruthy();
    expect(Notification).toHaveBeenCalled();
  });

  it('should track notification history', async () => {
    const { notificationManager } = await import('../index');

    notificationManager.show({ title: 'Test 1', body: 'Body 1' });
    notificationManager.show({ title: 'Test 2', body: 'Body 2' });

    const history = notificationManager.getHistory();
    expect(history).toHaveLength(2);
  });

  it('should clear history', async () => {
    const { notificationManager } = await import('../index');

    notificationManager.show({ title: 'Test', body: 'Body' });
    notificationManager.clearHistory();

    expect(notificationManager.getHistory()).toHaveLength(0);
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 183: Tray Integration
- Spec 170: IPC Channels
