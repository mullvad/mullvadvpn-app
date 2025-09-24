import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  enableDaitaSwitch: () => page.getByRole('switch', { name: 'Enable' }),
});
