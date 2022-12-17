import { Locator, Page, _electron as electron, ElectronApplication } from 'playwright';

export interface StartAppResponse {
  app: ElectronApplication;
  page: Page;
  util: TestUtils;
}

export interface TestUtils {
  currentRoute: () => Promise<void>;
  nextRoute: () => Promise<string>;
}

interface History {
  entries: Array<{ pathname: string }>;
  index: number;
}

type LaunchOptions = Parameters<typeof electron.launch>[0];

export const startApp = async (options: LaunchOptions): Promise<StartAppResponse> => {
  const app = await launch(options);
  const page = await app.firstWindow();

  page.on('pageerror', (error) => console.log(error));
  page.on('console', (msg) => console.log(msg.text()));

  const util: TestUtils = {
    currentRoute: currentRouteFactory(app),
    nextRoute: nextRouteFactory(app),
  };

  return { app, page, util };
};

export const launch = async (options: LaunchOptions): Promise<ElectronApplication> => {
  process.env.CI = 'e2e';
  const app = await electron.launch(options);

  await app.evaluate(({ webContents }) => {
    return new Promise((resolve) => {
      webContents.getAllWebContents()[0].once('did-finish-load', resolve);
    });
  });

  return app;
}

export const currentRouteFactory = (app: ElectronApplication) => {
  return async () => {
    return await app.evaluate(({ webContents }) => {
      return webContents.getAllWebContents()[0].executeJavaScript('window.e2e.location');
    });
  };
}

export const nextRouteFactory = (app: ElectronApplication) => {
  return async () => {
    const nextRoute: string = await app.evaluate(({ ipcMain }) => {
      return new Promise((resolve) => {
        ipcMain.once('navigation-setHistory', (_event, history: History) => {
            resolve(history.entries[history.index].pathname);
        });
      });
    });

    // TODO: Disable view transitions and shorten timeout or remove completely.
    await new Promise((resolve) => setTimeout(resolve, 1000));
    return nextRoute;
  };
};

const getStyleProperty = (locator: Locator, property: string) => {
  return locator.evaluate(
    (el, { property }) => {
      return window.getComputedStyle(el).getPropertyValue(property);
    },
    { property },
  );
};

export const getColor = (locator: Locator) => {
  return getStyleProperty(locator, 'color');
};

export const getBackgroundColor = (locator: Locator) => {
  return getStyleProperty(locator, 'background-color');
};
