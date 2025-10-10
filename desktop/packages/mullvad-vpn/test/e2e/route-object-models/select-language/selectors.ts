import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  languageOption: (language: string) => page.getByRole('option', { name: language }),
});
