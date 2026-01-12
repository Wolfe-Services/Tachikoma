# 005b - IPC Listener Memory Leak Fix

**Phase:** 0 - Project Setup & Foundation
**Spec ID:** 005b
**Status:** Planned
**Dependencies:** 005-ipc-bridge
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Fix the memory leak in the IPC listener implementation where callback wrappers are not properly tracked, causing `off()` to fail silently and listeners to accumulate.

---

## Problem

The current implementation wraps callbacks when registering but tries to remove the original callback:

```typescript
// BUG: Wrapper is created but not stored
on: (channel, callback) => {
  ipcRenderer.on(channel, (_event, ...args) => callback(...args)) // wrapper created
},
off: (channel, callback) => {
  ipcRenderer.removeListener(channel, callback) // tries to remove original, not wrapper!
}
```

This causes:
- Listeners accumulate over component lifecycle
- Memory grows unbounded in long-running sessions
- Duplicate event handling after re-subscriptions

---

## Acceptance Criteria

- [ ] Callback wrapper map tracks original → wrapper relationship
- [ ] `off()` correctly removes the wrapper, not original
- [ ] `removeAllListeners()` method for channel cleanup
- [ ] Listener count tracking for debugging
- [ ] Unit tests verify proper cleanup

---

## Implementation Details

### electron/preload/index.ts

```typescript
import { contextBridge, ipcRenderer, IpcRendererEvent } from 'electron'

// Track callback wrappers for proper removal
type Callback = (...args: unknown[]) => void
type Wrapper = (event: IpcRendererEvent, ...args: unknown[]) => void

const listenerMap = new Map<string, Map<Callback, Wrapper>>()

function getChannelMap(channel: string): Map<Callback, Wrapper> {
  let map = listenerMap.get(channel)
  if (!map) {
    map = new Map()
    listenerMap.set(channel, map)
  }
  return map
}

const validInvokeChannels = [
  'mission:start',
  'mission:stop',
  'mission:status',
  'spec:list',
  'spec:read',
  'config:get',
  'config:set',
  'vcs:status',
  'vcs:commit',
  'vcs:diff'
] as const

const validEventChannels = [
  'mission:progress',
  'mission:log',
  'mission:complete',
  'mission:error',
  'vcs:changed'
] as const

contextBridge.exposeInMainWorld('tachikoma', {
  platform: process.platform,

  invoke: (channel: string, ...args: unknown[]) => {
    if (validInvokeChannels.includes(channel as typeof validInvokeChannels[number])) {
      return ipcRenderer.invoke(channel, ...args)
    }
    throw new Error(`Invalid invoke channel: ${channel}`)
  },

  on: (channel: string, callback: Callback) => {
    if (!validEventChannels.includes(channel as typeof validEventChannels[number])) {
      console.warn(`Invalid event channel: ${channel}`)
      return
    }

    const channelMap = getChannelMap(channel)

    // Prevent duplicate registration
    if (channelMap.has(callback)) {
      return
    }

    // Create and store wrapper
    const wrapper: Wrapper = (_event, ...args) => callback(...args)
    channelMap.set(callback, wrapper)
    ipcRenderer.on(channel, wrapper)
  },

  off: (channel: string, callback: Callback) => {
    const channelMap = listenerMap.get(channel)
    if (!channelMap) return

    const wrapper = channelMap.get(callback)
    if (wrapper) {
      ipcRenderer.removeListener(channel, wrapper)
      channelMap.delete(callback)
    }
  },

  offAll: (channel: string) => {
    const channelMap = listenerMap.get(channel)
    if (!channelMap) return

    for (const wrapper of channelMap.values()) {
      ipcRenderer.removeListener(channel, wrapper)
    }
    channelMap.clear()
  },

  listenerCount: (channel: string): number => {
    return listenerMap.get(channel)?.size ?? 0
  }
})
```

### web/src/lib/ipc/types.ts (update)

```typescript
export interface TachikomaIPC {
  platform: string
  invoke: (channel: string, ...args: unknown[]) => Promise<unknown>
  on: (channel: string, callback: (...args: unknown[]) => void) => void
  off: (channel: string, callback: (...args: unknown[]) => void) => void
  offAll: (channel: string) => void
  listenerCount: (channel: string) => number
}
```

---

## Testing Requirements

1. Register listener, verify count is 1
2. Call off(), verify count is 0
3. Register same callback twice, verify only 1 listener
4. Register multiple callbacks, offAll() removes all
5. Component mount/unmount cycle doesn't leak listeners

---

## Related Specs

- Depends on: [005-ipc-bridge.md](005-ipc-bridge.md)
- Related: [170-ipc-channels.md](../phase-08-electron/170-ipc-channels.md)
