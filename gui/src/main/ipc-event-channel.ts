import { ipcMain } from 'electron';

import { createIpcMain } from '../shared/ipc-helpers';
import { ipcSchema } from '../shared/ipc-schema';

export const IpcMainEventChannel = createIpcMain(ipcSchema, ipcMain);
