import { contextBridge } from 'electron';

import { IpcRendererEventChannel } from './lib/ipc-event-channel';

contextBridge.exposeInMainWorld('ipc', IpcRendererEventChannel);

contextBridge.exposeInMainWorld('env', {
  e2e: process.env.CI,
  development: process.env.NODE_ENV === 'development',
  platform: process.platform,
});
