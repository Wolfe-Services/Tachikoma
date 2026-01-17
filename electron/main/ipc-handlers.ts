import { ipcMain, IpcMainInvokeEvent } from 'electron';
import type { IpcChannels } from '../shared/ipc';
import { native } from './native';

// Type-safe handler registration
function handle<K extends keyof IpcChannels>(
  channel: K,
  handler: (
    event: IpcMainInvokeEvent,
    request: IpcChannels[K]['request']
  ) => Promise<IpcChannels[K]['response']>
): void {
  ipcMain.handle(channel, handler);
}

export function registerIpcHandlers(): void {
  // Mission handlers
  handle('mission:start', async (_event, request) => {
    const missionId = await native.startMission(request.specPath, request.backend, request.mode);
    return { missionId };
  });

  handle('mission:stop', async (_event, request) => {
    const success = await native.stopMission(request.missionId);
    return { success };
  });

  handle('mission:status', async (_event, request) => {
    return await native.getMissionStatus(request.missionId);
  });

  // Spec handlers
  handle('spec:list', async (_event, request) => {
    return await native.listSpecs(request.path);
  });

  handle('spec:read', async (_event, request) => {
    return await native.readSpec(request.path);
  });

  // Config handlers
  handle('config:get', async (_event, request) => {
    return await native.getConfig(request.key);
  });

  handle('config:set', async (_event, request) => {
    const success = await native.setConfig(request.key, request.value);
    return { success };
  });

  // Forge handlers
  handle('forge:createSession', async (_event, request) => {
    return await native.createForgeSession(request);
  });

  handle('forge:getSession', async (_event, request) => {
    return await native.getForgeSession(request.sessionId);
  });

  handle('forge:listSessions', async (_event, _request) => {
    return await native.listForgeSessions();
  });

  handle('forge:deleteSession', async (_event, request) => {
    const success = await native.deleteForgeSession(request.sessionId);
    return { success };
  });

  handle('forge:startDeliberation', async (_event, request) => {
    const success = await native.startDeliberation(request.sessionId, request.phase);
    return { success };
  });

  handle('forge:stopDeliberation', async (_event, request) => {
    const success = await native.stopDeliberation(request.sessionId);
    return { success };
  });

  handle('forge:submitMessage', async (_event, request) => {
    const messageId = await native.submitForgeMessage(
      request.sessionId, 
      request.content, 
      request.participantId
    );
    return { messageId };
  });

  handle('forge:generateOutput', async (_event, request) => {
    return await native.generateForgeOutput(request);
  });
}