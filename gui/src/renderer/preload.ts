import { contextBridge } from 'electron';
import { IpcRendererEventChannel } from './lib/ipc-event-channel';

contextBridge.exposeInMainWorld('ipc', IpcRendererEventChannel);

const env = IpcRendererEventChannel.env.get();
contextBridge.exposeInMainWorld('env', {
  development: env.nodeEnv === 'development',
  platform: env.platform,
});
