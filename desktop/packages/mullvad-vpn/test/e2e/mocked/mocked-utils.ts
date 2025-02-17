import { ElectronApplication } from 'playwright';

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
}

export const startMockedApp = async (): Promise<StartMockedAppResponse> => {
  const args = ['.'];
  if (noSandbox) {
    console.log('Running tests without chromium sandbox');
    args.unshift('--no-sandbox');
  }

  const startAppResult = await startApp({ args });
  const mockIpcHandle = generateMockIpcHandle(startAppResult.app);
  const sendMockIpcResponse = generateSendMockIpcResponse(startAppResult.app);

  return {
    ...startAppResult,
    util: {
      ...startAppResult.util,
      mockIpcHandle,
      sendMockIpcResponse,
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
