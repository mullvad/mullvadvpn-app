import { ElectronAppInfo, findLatestBuild, parseElectronApp } from 'electron-playwright-helpers';
import { Page } from 'playwright';
import { _electron as electron, ElectronApplication } from 'playwright-core';

interface StartAppResponse {
  electronApp: ElectronApplication;
  appWindow: Page;
  appInfo: ElectronAppInfo;
}

const startApp = async (): Promise<StartAppResponse> => {
  // find the latest build in the out directory
  const latestBuild = findLatestBuild('../dist');
  // parse the directory and find paths and other info
  const appInfo = parseElectronApp(latestBuild);
  process.env.CI = 'e2e';

  const electronApp = await electron.launch({
    args: [appInfo.main],
    executablePath: appInfo.executable,
  });

  const appWindow = await electronApp.firstWindow();

  process.env.CI = 'e2e';

  electronApp.on('window', (page) => {
    const filename = page.url()?.split('/').pop();
    console.log(`Window opened: ${filename}`);

    // capture errors
    page.on('pageerror', (error) => {
      console.error(error);
    });

    // capture console messages
    page.on('console', (msg) => {
      console.log(msg.text());
    });
  });

  return { electronApp, appInfo, appWindow };
};

export { startApp };
