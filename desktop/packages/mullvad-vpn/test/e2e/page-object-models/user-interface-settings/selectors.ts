import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  languageButton: () =>
    page.locator('button', {
      has: page.locator('img'),
    }),
  languageButtonLabel: (label: string) =>
    page.locator('button', {
      hasText: label,
    }),
});
