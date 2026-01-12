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