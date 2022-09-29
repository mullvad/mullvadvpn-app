import { Locator, Page, _electron as electron, ElectronApplication } from 'playwright';

export interface StartAppResponse {
  app: ElectronApplication;
  page: Page;
}

export const startApp = async (mainPath: string): Promise<StartAppResponse> => {
  process.env.CI = 'e2e';

  const app = await electron.launch({
    args: [mainPath],
  });

  const page = await app.firstWindow();

  page.on('pageerror', (error) => {
    console.log(error);
  });

  page.on('console', (msg) => {
    console.log(msg.text());
  });

  return { app, page };
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
