import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  continueButton: () => page.getByRole('button', { name: 'Continue' }),
});
