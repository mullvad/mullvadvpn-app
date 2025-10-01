import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  startUsingTheAppButton: () => page.getByRole('button', { name: 'Start using the app' }),
});
