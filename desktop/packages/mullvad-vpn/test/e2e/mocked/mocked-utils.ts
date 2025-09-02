import { ElectronApplication } from 'playwright';

import { AnyIpcCall, createIpc, Schema } from '../../../src/shared/ipc-helpers';
import { IpcSchema, ipcSchema } from '../../../src/shared/ipc-schema';
import { startApp, TestUtils } from '../utils';

// This option can be removed in the future when/if we're able to tun the tests with the sandbox
// enabled in GitHub actions (frontend.yml).
const noSandbox = process.env.NO_SANDBOX === '1';

interface StartMockedAppResponse extends Awaited<ReturnType<typeof startApp>> {
  util: MockedTestUtils;
}

export interface MockedTestUtils extends TestUtils {
  mockIpcHandle: MockIpcHandle;
  sendMockIpcResponse: SendMockIpcResponse;
  expectIpcCall: ExpectIpcCall;
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
  const mockIpcHandle = generateMockIpcHandle(startAppResult.app);
  const sendMockIpcResponse = generateSendMockIpcResponse(startAppResult.app);
  const expectIpcCall = generateExpectIpcCall(startAppResult.app);

  return {
    ...startAppResult,
    util: {
      ...startAppResult.util,
      mockIpcHandle,
      sendMockIpcResponse,
      expectIpcCall,

      ipc: createTestIpc(startAppResult.app),
    },
  };
};

type MockIpcHandleProps<T> = {
  channel: string;
  response: T;
};

export type MockIpcHandle = ReturnType<typeof generateMockIpcHandle>;

export const generateMockIpcHandle = (electronApp: ElectronApplication) => {
  return async <T>({ channel, response }: MockIpcHandleProps<T>): Promise<void> => {
    await electronApp.evaluate(
      ({ ipcMain }, { channel, response }) => {
        ipcMain.removeHandler(channel);
        ipcMain.handle(channel, () => {
          return Promise.resolve({
            type: 'success',
            value: response,
          });
        });
      },
      { channel, response },
    );
  };
};

type SendMockIpcResponseProps<T> = {
  channel: string;
  response: T;
};

export type SendMockIpcResponse = ReturnType<typeof generateSendMockIpcResponse>;

export const generateSendMockIpcResponse = (electronApp: ElectronApplication) => {
  return async <T>({ channel, response }: SendMockIpcResponseProps<T>) => {
    await electronApp.evaluate(
      ({ webContents }, { channel, response }) => {
        webContents
          .getAllWebContents()
          // Select window that isn't devtools
          .find((webContents) => webContents.getURL().startsWith('file://'))!
          .send(channel, response);
      },
      { channel, response },
    );
  };
};

export type ExpectIpcCall = ReturnType<typeof generateExpectIpcCall>;

export const generateExpectIpcCall = (electronApp: ElectronApplication) => {
  return <T>(channel: string): Promise<T> => {
    return electronApp.evaluate(
      ({ ipcMain }, { channel }) => {
        return new Promise<T>((resolve) => {
          ipcMain.handleOnce(channel, (_event, arg) => {
            resolve(arg);
            return {
              type: 'success',
              value: null,
            };
          });
        });
      },
      { channel },
    );
  };
};

type IpcMockedTestKey<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? 'notify'
  : 'handle';

type IpcMockedTestExpectKey<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? never
  : 'expect';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Async<F extends (...args: any) => any> = (arg: Parameters<F>[0]) => Promise<ReturnType<F>>;

type IpcMockedTestFn<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? Async<NonNullable<ReturnType<I['send']>>>
  : Async<Parameters<ReturnType<I['receive']>>[0]>;

export type IpcMockedTest<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G]]: {
      [C in IpcMockedTestKey<S[G][K]>]: IpcMockedTestFn<S[G][K]>;
    } & {
      [C in IpcMockedTestExpectKey<S[G][K]>]: () => Promise<void>;
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
        notify: <T>(message: T) =>
          generateSendMockIpcResponse(electronApp)({ channel: event, response: message }),
        handle: <T>(response: T) =>
          generateMockIpcHandle(electronApp)({ channel: event, response }),
        expect: () => generateExpectIpcCall(electronApp)(event),
      },
    ];
  });
}
