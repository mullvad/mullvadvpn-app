import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  settingsButton: () => page.locator('button[aria-label="Settings"]'),
});
