# 003 - Electron Shell Setup

**Phase:** 0 - Setup
**Spec ID:** 003
**Status:** Completed
**Dependencies:** 001-project-structure
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Set up the Electron main process with proper security configuration, window management, and preparation for Rust native module integration.

---

## Acceptance Criteria

- [x] Electron project initialized in `electron/` directory
- [x] Main process entry point configured
- [x] BrowserWindow with security defaults
- [x] Preload script structure in place
- [x] Development and production configurations
- [x] TypeScript configured for Electron

---

## Implementation Details

### 1. electron/package.json

```json
{
  "name": "tachikoma-electron",
  "version": "0.1.0",
  "private": true,
  "main": "dist/main/index.js",
  "scripts": {
    "dev": "electron-vite dev",
    "build": "electron-vite build",
    "preview": "electron-vite preview",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "electron-updater": "^6.1.0"
  },
  "devDependencies": {
    "@electron-toolkit/tsconfig": "^1.0.0",
    "@types/node": "^20.10.0",
    "electron": "^28.0.0",
    "electron-builder": "^24.9.0",
    "electron-vite": "^2.0.0",
    "typescript": "^5.3.0"
  }
}
```

### 2. electron/electron.vite.config.ts

```typescript
import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import { resolve } from 'path'

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: 'dist/main',
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'main/index.ts')
        }
      }
    }
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: 'dist/preload',
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'preload/index.ts')
        }
      }
    }
  },
  renderer: {
    root: '../web',
    build: {
      outDir: '../web/dist'
    }
  }
})
```

### 3. electron/main/index.ts

```typescript
import { app, BrowserWindow, shell } from 'electron'
import { join } from 'path'
import { electronApp, optimizer, is } from '@electron-toolkit/utils'

let mainWindow: BrowserWindow | null = null

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 800,
    minHeight: 600,
    show: false,
    autoHideMenuBar: false,
    titleBarStyle: 'hiddenInset',
    trafficLightPosition: { x: 15, y: 10 },
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: true,
      contextIsolation: true,
      nodeIntegration: false,
      webSecurity: true
    }
  })

  mainWindow.on('ready-to-show', () => {
    mainWindow?.show()
  })

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  })

  // Load the renderer
  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../../web/dist/index.html'))
  }
}

app.whenReady().then(() => {
  // Set app user model id for windows
  electronApp.setAppUserModelId('com.tachikoma.app')

  // Watch for shortcuts in dev
  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window)
  })

  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
```

### 4. electron/preload/index.ts

```typescript
import { contextBridge, ipcRenderer } from 'electron'

// Expose protected methods to renderer
contextBridge.exposeInMainWorld('tachikoma', {
  // Platform info
  platform: process.platform,

  // IPC methods
  invoke: (channel: string, ...args: unknown[]) => {
    const validChannels = [
      'mission:start',
      'mission:stop',
      'mission:status',
      'spec:list',
      'spec:read',
      'config:get',
      'config:set'
    ]
    if (validChannels.includes(channel)) {
      return ipcRenderer.invoke(channel, ...args)
    }
    throw new Error(`Invalid channel: ${channel}`)
  },

  // Event subscriptions
  on: (channel: string, callback: (...args: unknown[]) => void) => {
    const validChannels = [
      'mission:progress',
      'mission:log',
      'mission:complete',
      'mission:error'
    ]
    if (validChannels.includes(channel)) {
      ipcRenderer.on(channel, (_event, ...args) => callback(...args))
    }
  },

  off: (channel: string, callback: (...args: unknown[]) => void) => {
    ipcRenderer.removeListener(channel, callback)
  }
})
```

### 5. electron/tsconfig.json

```json
{
  "extends": "@electron-toolkit/tsconfig/tsconfig.node.json",
  "compilerOptions": {
    "outDir": "dist",
    "rootDir": ".",
    "strict": true,
    "moduleResolution": "node",
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["main/**/*", "preload/**/*"]
}
```

---

## Security Configuration

1. **contextIsolation: true** - Isolates preload from renderer
2. **nodeIntegration: false** - No Node.js in renderer
3. **sandbox: true** - Chromium sandbox enabled
4. **webSecurity: true** - Same-origin policy enforced

---

## Testing Requirements

1. `npm install` succeeds in electron/
2. `npm run typecheck` passes
3. `npm run dev` launches Electron window
4. DevTools accessible in development

---

## Related Specs

- Depends on: [001-project-structure.md](001-project-structure.md)
- Next: [004-svelte-integration.md](004-svelte-integration.md)
