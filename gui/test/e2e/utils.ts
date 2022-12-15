import { Locator, Page, _electron as electron, ElectronApplication } from 'playwright';

export interface StartAppResponse {
  app: ElectronApplication;
  page: Page;
  util: TestUtils;
}

export interface TestUtils {
  getByTestId: (id: string) => Locator;
  currentRoute: () => Promise<void>;
  nextRoute: () => Promise<string>;
}

interface History {
  entries: Array<{ pathname: string }>;
  index: number;
}

export const startApp = async (
  options: Parameters<typeof electron.launch>[0],
): Promise<StartAppResponse> => {
  process.env.CI = 'e2e';
  const app = await electron.launch(options);

  await app.evaluate(({ webContents }) => {
    return new Promise((resolve) => {
      webContents.getAllWebContents()[0].on('did-finish-load', resolve);
    });
  });

  const page = await app.firstWindow();

  page.on('pageerror', (error) => {
    console.log(error);
  });

  page.on('console', (msg) => {
    console.log(msg.text());
  });

  const util: TestUtils = {
    getByTestId: (id: string) => page.locator(`data-test-id=${id}`),
    currentRoute: currentRouteFactory(app),
    nextRoute: nextRouteFactory(app),
  };

  return { app, page, util };
};

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
