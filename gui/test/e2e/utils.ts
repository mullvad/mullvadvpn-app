import { Locator, Page, _electron as electron, ElectronApplication } from 'playwright';

export interface StartAppResponse {
  app: ElectronApplication;
  page: Page;
  util: TestUtils;
}

export interface TestUtils {
  currentRoute: () => Promise<void>;
  waitForNavigation: (initiateNavigation?: () => Promise<void> | void) =>  Promise<string>;
}

interface History {
  entries: Array<{ pathname: string }>;
  index: number;
}

type LaunchOptions = NonNullable<Parameters<typeof electron.launch>[0]>;

export const startApp = async (options: LaunchOptions): Promise<StartAppResponse> => {
  const app = await launch(options);
  const page = await app.firstWindow();

  // Wait for initial navigation to finish
  await waitForNoTransition(page);

  page.on('pageerror', (error) => console.log(error));
  page.on('console', (msg) => console.log(msg.text()));

  const util: TestUtils = {
    currentRoute: currentRouteFactory(app),
    waitForNavigation: waitForNavigationFactory(app, page),
  };

  return { app, page, util };
};

export const launch = async (options: LaunchOptions): Promise<ElectronApplication> => {
  process.env.CI = 'e2e';
  const app = await electron.launch(options);

  await app.evaluate(({ webContents }) => {
    return new Promise((resolve) => {
      webContents.getAllWebContents()
          // Select window that isn't devtools
          .find((webContents) => webContents.getURL().startsWith('file://'))!
          .once('did-finish-load', resolve);
    });
  });

  return app;
}

const currentRouteFactory = (app: ElectronApplication) => {
  return async () => {
    return await app.evaluate(({ webContents }) => {
      return webContents.getAllWebContents()
          // Select window that isn't devtools
          .find((webContents) => webContents.getURL().startsWith('file://'))!
          .executeJavaScript('window.e2e.location');
    });
  };
}

const waitForNavigationFactory = (
  app: ElectronApplication,
  page: Page,
) => {
  // Wait for navigation animation to finish. A function can be provided that initiates the
  // navigation, e.g. clicks a button.
  return async (initiateNavigation?: () => Promise<void> | void) => {
    // Wait for route to change after optionally initiating the navigation.
    const [route] = await Promise.all([
      waitForNextRoute(app),
      initiateNavigation?.(),
    ]);

    // Wait for view corresponding to new route to appear
    await page.getByTestId(route).isVisible();
    await waitForNoTransition(page);

    return route;
  };
};

const waitForNoTransition = async (page: Page) => {
  // Wait until there's only one transitionContents
  let  transitionContentsCount;
  do {
    if (transitionContentsCount !== undefined) {
      await new Promise((resolve) => setTimeout(resolve, 5));
    }

    transitionContentsCount = await page.getByTestId('transition-content').count();
  } while (transitionContentsCount !== 1);
};

// Returns the route when it changes
const waitForNextRoute = async (app: ElectronApplication): Promise<string> => {
  return await app.evaluate(({ ipcMain }) => {
    return new Promise((resolve) => {
      ipcMain.once('navigation-setHistory', (_event, history: History) => {
          resolve(history.entries[history.index].pathname);
      });
    });
  });
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
