import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  enableMultihopSwitch: () => page.getByRole('switch', { name: 'Enable' }),
});
