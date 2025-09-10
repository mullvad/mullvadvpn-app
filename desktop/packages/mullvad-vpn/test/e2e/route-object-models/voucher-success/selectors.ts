import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  nextButton: () => page.getByRole('button', { name: 'Next' }),
});
