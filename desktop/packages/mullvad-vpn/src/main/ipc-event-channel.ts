import { ipcMain, WebContents } from 'electron';

import { createIpcMain } from '../shared/ipc-helpers';
import { ipcSchema } from '../shared/ipc-schema';

// eslint-disable-next-line @typescript-eslint/naming-convention
export let IpcMainEventChannel = createIpcMain(ipcSchema, ipcMain, undefined);

// Change the `IpcMainEventChannel` for a new one with a new `WebContents`.
export function changeIpcWebContents(webContents: WebContents | undefined) {
  IpcMainEventChannel = createIpcMain(ipcSchema, ipcMain, webContents);
}

export function unsetIpcWebContents() {
  changeIpcWebContents(undefined);
}
