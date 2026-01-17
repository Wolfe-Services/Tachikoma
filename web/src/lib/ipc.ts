// IPC bridge for communication with Electron main process
// Falls back to mock implementation when not running in Electron

type TachikomaAPI = NonNullable<Window['tachikoma']>;

// Check if running in Electron
const isElectron = typeof window !== 'undefined' && window.tachikoma !== undefined;

// Mock IPC for development in browser
const mockIpc = {
  async invoke(channel: string, ..._args: unknown[]): Promise<unknown> {
    console.log(`[IPC Mock] ${channel}`, _args);
    
    // Return mock data based on channel
    switch (channel) {
      case 'config:get':
        return {
          backend: { brain: 'claude', thinkTank: 'o3' },
          loop: { maxIterations: 100 }
        };
      case 'spec:list':
        return [];
      case 'mission:status':
        return { state: 'idle', progress: 0 };
      default:
        return null;
    }
  },
  
  on(channel: string, _callback: (...args: unknown[]) => void): void {
    console.log(`[IPC Mock] Registered listener for: ${channel}`);
  },
  
  off(channel: string, _callback: (...args: unknown[]) => void): void {
    console.log(`[IPC Mock] Removed listener for: ${channel}`);
  }
};

// Export the appropriate IPC implementation
export const ipc: TachikomaAPI = isElectron ? window.tachikoma! : (mockIpc as unknown as TachikomaAPI);

// Type-safe invoke helper
export async function invoke<T>(channel: string, ...args: unknown[]): Promise<T> {
  return ipc.invoke(channel, ...args) as Promise<T>;
}

// Event listener helpers
export function on(channel: string, callback: (...args: unknown[]) => void): void {
  ipc.on(channel, callback);
}

export function off(channel: string, callback: (...args: unknown[]) => void): void {
  ipc.off(channel, callback);
}
