import { ipcRenderer } from 'electron';
import { createIpcRenderer } from '../../shared/ipc-helpers';
import { ipcSchema } from '../../shared/ipc-schema';

export const IpcRendererEventChannel = createIpcRenderer(ipcSchema, ipcRenderer);
