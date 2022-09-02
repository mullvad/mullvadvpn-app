import { Page } from 'playwright';
import { _electron as electron, ElectronApplication } from 'playwright-core';

interface StartAppResponse {
  electronApp: ElectronApplication;
  appWindow: Page;
}

const startApp = async (): Promise<StartAppResponse> => {
  process.env.CI = 'e2e';

  const electronApp = await electron.launch({
    args: ['.'],
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

export { startApp };
