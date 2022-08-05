import { ipcMain, WebContents } from 'electron';

import { createIpcMain, IpcMain, Schema } from '../shared/ipc-helpers';
import { ipcSchema } from '../shared/ipc-schema';

// Type where the notify functions have been replaced with either `undefined` if no `WebContents` is
// available, or with the function curried with the `WebContents`.
type IpcMainBootstrappedWithWebContents<S extends Schema> = {
  [GK in keyof IpcMain<S>]: {
    [CK in keyof IpcMain<S>[GK]]: CK extends `notify${string}`
      ? undefined | ((arg: Parameters<IpcMain<S>[GK][CK]>[1]) => ReturnType<IpcMain<S>[GK][CK]>)
      : IpcMain<S>[GK][CK];
  };
};

const ipcMainFromSchema = createIpcMain(ipcSchema, ipcMain);
// eslint-disable-next-line @typescript-eslint/naming-convention
export let IpcMainEventChannel = bootstrapIpcMainWithWebContents();

// Curries all notify functions with `WebContents` if it's not `undefined`. If it is `undefined`
// then the whole function will be replaced with `undefined`.
function bootstrapIpcMainWithWebContents(
  webContents?: WebContents,
): IpcMainBootstrappedWithWebContents<typeof ipcSchema> {
  return Object.fromEntries(
    Object.entries(ipcMainFromSchema).map(([groupKey, group]) => {
      const newGroup = Object.fromEntries(
        Object.entries(group).map(([callKey, call]) => {
          if (callKey.startsWith('notify')) {
            const newCall = webContents
              ? (arg: Parameters<typeof call>[1]) => call(webContents, arg)
              : undefined;
            return [callKey, newCall];
          } else {
            return [callKey, call];
          }
        }),
      );

      return [groupKey, newGroup];
    }),
  ) as IpcMainBootstrappedWithWebContents<typeof ipcSchema>;
}

// Change the `IpcMainEventChannel` for a new one with a new `WebContents`.
export function changeIpcWebContents(webContents?: WebContents) {
  IpcMainEventChannel = bootstrapIpcMainWithWebContents(webContents);
}
