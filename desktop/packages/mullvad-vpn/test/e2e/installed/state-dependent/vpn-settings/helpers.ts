import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  launchAppOnStartupSwitch: () => page.getByLabel('Launch app on start-up'),
  autoConnectSwitch: () => page.getByLabel('Auto-connect'),
});
