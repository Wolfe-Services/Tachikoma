export class IpcError extends Error {
  constructor(
    message: string,
    public channel: string,
    public originalError?: unknown
  ) {
    super(message);
    this.name = 'IpcError';
  }
}

export function handleIpcError(channel: string, error: unknown): IpcError {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`IPC Error on channel '${channel}':`, error);
  return new IpcError(`IPC call failed: ${message}`, channel, error);
}

export function isIpcAvailable(): boolean {
  return typeof window !== 'undefined' && !!window.tachikoma;
}