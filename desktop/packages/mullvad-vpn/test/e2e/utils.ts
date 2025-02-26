import fs from 'fs';
import { _electron as electron, ElectronApplication, Locator, Page } from 'playwright';

export interface StartAppResponse {
  app: ElectronApplication;
  page: Page;
  util: TestUtils;
}

export interface TestUtils {
  currentRoute: () => Promise<string | null>;
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

  // Wait for initial navigation to finish
  await waitForNoTransition(page);

  page.on('pageerror', (error) => console.log(error));
  page.on('console', (msg) => console.log(msg.text()));

  const util: TestUtils = {
    currentRoute: currentRouteFactory(app),
    waitForNavigation: waitForNavigationFactory(app, page),
    waitForNoTransition: () => waitForNoTransition(page),
    waitForRoute: waitForRouteFactory(app),
  };

  return { app, page, util };
};

export const launch = (options: LaunchOptions): Promise<ElectronApplication> => {
  process.env.CI = 'e2e';
  return electron.launch(options);
};

const currentRouteFactory = (app: ElectronApplication) => {
  return () => {
    return app.evaluate<string | null>(({ webContents }) => {
      const electronWebContent = webContents
        .getAllWebContents()
        // Select window that isn't devtools
        .find((webContents) => webContents.getURL().startsWith('file://'));

      if (electronWebContent) {
        return electronWebContent.executeJavaScript('window.e2e.location');
      }

      return null;
    });
  };
};

const waitForNavigationFactory = (app: ElectronApplication, page: Page) => {
  // Wait for navigation animation to finish. A function can be provided that initiates the
  // navigation, e.g. clicks a button.
  return async (initiateNavigation?: () => Promise<void> | void) => {
    // Wait for route to change after optionally initiating the navigation.
    const [route] = await Promise.all([waitForNextRoute(app), initiateNavigation?.()]);

    // Wait for view corresponding to new route to appear
    await page.getByTestId(route).isVisible();
    await waitForNoTransition(page);

    return route;
  };
};

const waitForNoTransition = async (page: Page) => {
  // Wait until there's only one transitionContents
  let transitionContentsCount;
  do {
    if (transitionContentsCount !== undefined) {
      await new Promise((resolve) => setTimeout(resolve, 5));
    }

    try {
      transitionContentsCount = await page.getByTestId('transition-content').count();
    } catch {
      console.log('Transition content count failed');
      break;
    }
  } while (transitionContentsCount !== 1);
};

// This factory returns a function which returns a boolean when the route passed to it matches that of the application.
const waitForRouteFactory = (app: ElectronApplication) => {
  const getCurrentRoute = currentRouteFactory(app);

  const waitForRoute = async (route: string) => {
    const currentRoute = await getCurrentRoute();

    if (currentRoute !== route) {
      return waitForRoute(route);
    }
  };

  return waitForRoute;
};

// Returns the route when it changes
const waitForNextRoute = (app: ElectronApplication): Promise<string> => {
  return app.evaluate(
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
