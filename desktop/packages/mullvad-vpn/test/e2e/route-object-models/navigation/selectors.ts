import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  backButton: () => page.getByRole('button', { name: /(Back|Close)/ }),
});
