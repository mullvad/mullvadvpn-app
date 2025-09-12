import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  manageDevicesButton: () => page.getByRole('button', { name: 'Manage devices' }),
});
