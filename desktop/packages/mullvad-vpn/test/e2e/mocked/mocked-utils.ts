import { ElectronApplication } from 'playwright';

import { AnyIpcCall, createIpc, Schema } from '../../../src/shared/ipc-helpers';
import { IpcSchema, ipcSchema } from '../../../src/shared/ipc-schema';
import { Async } from '../../../src/shared/utility-types';
import { startApp, TestUtils } from '../utils';

// This option can be removed in the future when/if we're able to tun the tests with the sandbox
// enabled in GitHub actions (frontend.yml).
const noSandbox = process.env.NO_SANDBOX === '1';

interface StartMockedAppResponse extends Awaited<ReturnType<typeof startApp>> {
  util: MockedTestUtils;
}

export interface MockedTestUtils extends TestUtils {
  ipc: IpcMockedTest<IpcSchema>;
}

export const startMockedApp = async (): Promise<StartMockedAppResponse> => {
  const args = ['.'];
  if (noSandbox) {
    console.log('Running tests without chromium sandbox');
    args.unshift('--no-sandbox');
  }
  // NOTE: Keep in sync with index.ts
  args.push('--gtk-version=3');

  const startAppResult = await startApp({ args });

  return {
    ...startAppResult,
    util: {
      ...startAppResult.util,
      ipc: createTestIpc(startAppResult.app),
    },
  };
};

export const createMockIpcHandle = (electronApp: ElectronApplication, event: string) => {
  // This function resolves when the handle is registered. To await the event, use `expect()`.
  return async <T>(response: T): Promise<void> => {
    await electronApp.evaluate(
      ({ ipcMain }, { event, response }) => {
        ipcMain.removeHandler(event);
        ipcMain.handle(event, () => {
          return Promise.resolve({
            type: 'success',
            value: response,
          });
        });
      },
      { event, response },
    );
  };
};

export const createMockIpcNotify = (electronApp: ElectronApplication, event: string) => {
  return async <T>(arg: T) => {
    await electronApp.evaluate(
      ({ webContents }, { event, arg }) => {
        webContents
          .getAllWebContents()
          // Select window that isn't devtools
          .find((webContents) => webContents.getURL().startsWith('file://'))!
          .send(event, arg);
      },
      { event, arg },
    );
  };
};

export const createMockIpcExpect = (electronApp: ElectronApplication, event: string) => {
  return <T>(): Promise<T> => {
    return electronApp.evaluate(
      ({ ipcMain }, { event }) => {
        return new Promise<T>((resolve) => {
          ipcMain.handleOnce(event, (_event, arg) => {
            resolve(arg);
            return {
              type: 'success',
              value: null,
            };
          });
        });
      },
      { event },
    );
  };
};

export const createMockIpcIgnore = (electronApp: ElectronApplication, event: string) => {
  return async (): Promise<void> => {
    await electronApp.evaluate(
      ({ ipcMain }, { event }) => {
        ipcMain.removeHandler(event);
        ipcMain.handle(event, () => ({
          type: 'success',
          value: null,
        }));
      },
      { event },
    );
  };
};

type IpcMockedTestKey<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? 'notify'
  : 'handle';

type IpcMockedTestExtraHandlerKey<
  I extends AnyIpcCall,
  K,
> = I['direction'] extends 'main-to-renderer' ? never : K;

type IpcMockedTestFn<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? Async<NonNullable<ReturnType<I['send']>>>
  : Async<Parameters<ReturnType<I['receive']>>[0]>;

export type IpcMockedTest<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G]]: {
      [C in IpcMockedTestKey<S[G][K]>]: IpcMockedTestFn<S[G][K]>;
    } & {
      [C in IpcMockedTestExtraHandlerKey<S[G][K], 'expect'>]: () => Promise<void>;
    } & {
      [C in IpcMockedTestExtraHandlerKey<S[G][K], 'ignore'>]: () => Promise<void>;
    } & {
      eventKey: string;
    };
  };
};

export function createTestIpc(electronApp: ElectronApplication): IpcMockedTest<IpcSchema> {
  return createIpc(ipcSchema, (event, key, _spec) => {
    return [
      key,
      {
        eventKey: event,
        notify: createMockIpcNotify(electronApp, event),
        handle: createMockIpcHandle(electronApp, event),
        expect: createMockIpcExpect(electronApp, event),
        ignore: createMockIpcIgnore(electronApp, event),
      },
    ];
  });
}
