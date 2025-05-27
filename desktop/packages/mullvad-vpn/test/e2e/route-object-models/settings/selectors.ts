import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  userInterfaceButton: () => page.getByRole('button', { name: 'User interface settings' }),
  vpnSettingsButton: () => page.getByRole('button', { name: 'VPN settings' }),
});
