import './lib/path-helpers';

import { expect } from '@playwright/test';
import fs from 'fs';
import { _electron as electron, ElectronApplication, Locator, Page } from 'playwright';

const forceMotion = process.env.TEST_FORCE_MOTION === '1';

export interface StartAppResponse {
  app: ElectronApplication;
  page: Page;
  util: TestUtils;
}

type TriggerFn = () => Promise<void> | void;

export interface TestUtils {
  closePage: () => Promise<void>;
  getCurrentRoute: () => Promise<string | null>;
  expectRoute: (route: string) => Promise<void>;
  expectRouteChange: (trigger: TriggerFn) => Promise<void>;
  setReducedMotion: (value: ReducedMotionValue) => Promise<void>;
}

type LaunchOptions = NonNullable<Parameters<typeof electron.launch>[0]>;

type ReducedMotionValue = 'no-preference' | 'reduce';

export const startApp = async (options: LaunchOptions): Promise<StartAppResponse> => {
  const app = await launch(options);
  const page = await app.firstWindow();

  if (!forceMotion) {
    await setReducedMotion(page, 'reduce');
  }

  await promiseTimeout(page.waitForEvent('load'));

  page.on('pageerror', (error) => console.log(error));
  page.on('console', (msg) => console.log(msg.text()));

  const util: TestUtils = {
    closePage: () => closePage(page),
    getCurrentRoute: () => getCurrentRoute(page),
    expectRoute: (route: string) => expectRoute(page, route),
    expectRouteChange: (trigger: TriggerFn) => expectRouteChange(page, trigger),
    setReducedMotion: (value: ReducedMotionValue) => setReducedMotion(page, value),
  };

  return { app, page, util };
};

export const launch = (options: LaunchOptions): Promise<ElectronApplication> => {
  process.env.CI = 'e2e';
  return electron.launch(options);
};

function promiseTimeout<T>(promise: Promise<T>): Promise<T | void> {
  const timeoutPromise = new Promise<void>((resolve) => setTimeout(resolve, 1000));
  return Promise.any([timeoutPromise, promise]);
}

async function closePage(page: Page) {
  try {
    await promiseTimeout(page?.close());
  } catch (e) {
    // no-op, if a window failes to close it will be cleaned up automatically by playwright at the
    // end of the run.
    const error = e as Error;
    console.error(`page.close() threw an error: ${error.message}`);
  }
}

function getCurrentRoute(page: Page): Promise<string | null> {
  return page.evaluate('window?.e2e?.location ?? null');
}

// Returns a promise which resolves when the provided route is reached.
async function expectRoute(page: Page, expectedRoute: string): Promise<void> {
  await expect.poll(() => getCurrentRoute(page)).toMatchPath(expectedRoute);
}

// Returns a promise which resolves when the route changes.
async function expectRouteChange(page: Page, trigger: TriggerFn) {
  const initialRoute = await getCurrentRoute(page);
  await trigger();
  await expect.poll(() => getCurrentRoute(page)).not.toMatchPath(initialRoute);
}

async function setReducedMotion(page: Page, value: ReducedMotionValue) {
  await page.emulateMedia({ reducedMotion: value });

  const query = `(prefers-reduced-motion: ${value})`;
  await page.evaluate((q) => window.matchMedia(q).matches, query);
}

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
