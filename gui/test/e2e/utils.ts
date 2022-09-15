import { Locator, Page } from 'playwright';
import { _electron as electron, ElectronApplication } from 'playwright-core';

interface StartAppResponse {
  electronApp: ElectronApplication;
  appWindow: Page;
}

let electronApp: ElectronApplication;

export const startApp = async (): Promise<StartAppResponse> => {
  process.env.CI = 'e2e';

  electronApp = await electron.launch({
    args: ['build/test/e2e/setup/main.js'],
  });

  const appWindow = await electronApp.firstWindow();

  appWindow.on('pageerror', (error) => {
    console.log(error);
  });

  appWindow.on('console', (msg) => {
    console.log(msg.text());
  });

  return { electronApp, appWindow };
};

type MockIpcHandleProps<T> = {
  channel: string;
  response: T;
};

export const mockIpcHandle = async <T>({ channel, response }: MockIpcHandleProps<T>) => {
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

type SendMockIpcResponseProps<T> = {
  channel: string;
  response: T;
};

export const sendMockIpcResponse = async <T>({ channel, response }: SendMockIpcResponseProps<T>) => {
  await electronApp.evaluate(
    ({ webContents }, { channel, response }) => {
      webContents.getAllWebContents()[0].send(channel, response);
    },
    { channel, response },
  );
};

const getStyleProperty = async (locator: Locator, property: string) => {
  return locator.evaluate(
    (el, { property }) => {
      return window.getComputedStyle(el).getPropertyValue(property);
    },
    { property },
  );
};

export const getColor = async (locator: Locator) => {
  return getStyleProperty(locator, 'color');
};

export const getBackgroundColor = async (locator: Locator) => {
  return getStyleProperty(locator, 'background-color');
};
