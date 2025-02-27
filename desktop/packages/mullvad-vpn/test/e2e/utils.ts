import fs from 'fs';
import { _electron as electron, ElectronApplication, Locator, Page } from 'playwright';

export interface StartAppResponse {
  app: ElectronApplication;
  page: Page;
  util: TestUtils;
}

export interface TestUtils {
  currentRoute: () => Promise<string>;
  waitForNavigation: (initiateNavigation?: () => Promise<void> | void) => Promise<string>;
  waitForNoTransition: () => Promise<void>;
  waitForRoute: (route: string) => Promise<void>;
}

interface History {
  entries: Array<{ pathname: string }>;
  index: number;
}

type LaunchOptions = NonNullable<Parameters<typeof electron.launch>[0]>;

export const startApp = async (options: LaunchOptions): Promise<StartAppResponse> => {
  const app = await launch(options);
  const page = await app.firstWindow();

  page.on('pageerror', (error) => console.log(error));
  page.on('console', (msg) => console.log(msg.text()));

  const util: TestUtils = {
    currentRoute: currentRouteFactory(app),
    waitForNavigation: waitForNavigationFactory(app, page),
    waitForNoTransition: () => waitForNoTransition(page),
    waitForRoute: waitForRouteFactory(page),
  };

  return { app, page, util };
};

export const launch = (options: LaunchOptions): Promise<ElectronApplication> => {
  process.env.CI = 'e2e';
  return electron.launch(options);
};

const currentRouteFactory = (app: ElectronApplication) => {
  return () =>
    app.evaluate<string>(({ webContents }) =>
      webContents
        .getAllWebContents()
        // Select window that isn't devtools
        .find((webContents) => webContents.getURL().startsWith('file://'))!
        .executeJavaScript('window.e2e.location'),
    );
};

const waitForNavigationFactory = (app: ElectronApplication, page: Page) => {
  const waitForNextRoute = waitForNextRouteFactory(app);
  // Wait for navigation animation to finish. A function can be provided that initiates the
  // navigation, e.g. clicks a button.
  return async (initiateNavigation?: () => Promise<void> | void) => {
    // Wait for route to change after optionally initiating the navigation.
    const [route] = await Promise.all([waitForNextRoute(), initiateNavigation?.()]);

    // Wait for view corresponding to new route to appear
    await waitForTransitionEnd(page);

    return route;
  };
};

// This function returns a promise that resolves when a view transition ends. This requires a view transition to also be ongoing.
const waitForTransitionEnd = async (page: Page) => {
  await page.waitForFunction(() => window.isInViewTransition!());
  await waitForNoTransition(page);
};

// This function returns a promise that resolves when there is no view transition ongoing, which would be immediately.
const waitForNoTransition = async (page: Page) => {
  await page.waitForFunction(() => !window.isInViewTransition!());
};

// This factory returns a function which returns a boolean when the route passed to it matches that of the application.
const waitForRouteFactory = (page: Page) => {
  const waitForRoute = async (route: string) => {
    await page.waitForFunction((route) => route === window.e2e?.location, route);
  };

  return waitForRoute;
};

// Returns the route when it changes
const waitForNextRouteFactory = (app: ElectronApplication) => {
  return async () =>
    app.evaluate<string>(
      ({ ipcMain }) =>
        new Promise((resolve) => {
          ipcMain.once('navigation-setHistory', (_event, history: History) => {
            resolve(history.entries[history.index].pathname);
          });
        }),
    );
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

export function anyOf(...values: string[]): RegExp {
  return new RegExp(values.map(escapeRegExp).join('|'));
}

export function escapeRegExp(regexp: string): string {
  return regexp.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); // $& means the whole matched string
}

export function fileExists(filePath: string): boolean {
  try {
    fs.accessSync(filePath);
    return true;
  } catch {
    return false;
  }
}
