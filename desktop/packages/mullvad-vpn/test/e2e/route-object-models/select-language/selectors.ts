import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  languageOption: (language: string) =>
    page.locator('button', {
      has: page.locator('div', { hasText: language }),
    }),
});
