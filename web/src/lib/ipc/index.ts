// IPC Bridge exports for web app
export { ipc } from './client';
export type { IpcChannels, IpcEvents, MissionStatus, SpecFile, SpecMetadata, TachikomaConfig } from './types';
export { IpcError, handleIpcError, isIpcAvailable } from './errors';