import { contextBridge } from 'electron';
import { IpcRendererEventChannel } from './lib/ipc-event-channel';

contextBridge.exposeInMainWorld('ipc', IpcRendererEventChannel);
