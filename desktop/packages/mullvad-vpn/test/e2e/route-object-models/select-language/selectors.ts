import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  languageOption: (language: string) => page.getByRole('option', { name: language }),
  // Select first button since aria-label changes based on selected language
  backButton: () => page.getByRole('button').first(),
});
