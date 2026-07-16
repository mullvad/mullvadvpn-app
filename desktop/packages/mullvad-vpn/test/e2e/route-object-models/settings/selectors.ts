import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  multihopSettingsButton: () => page.getByRole('button', { name: 'Multihop' }),
  daitaSettingsButton: () => page.getByRole('button', { name: 'Daita' }),
  userInterfaceButton: () => page.getByRole('button', { name: 'User interface settings' }),
  vpnSettingsButton: () => page.getByRole('button', { name: 'VPN settings' }),
  splitTunnelingSettingsButton: () => page.getByRole('button', { name: 'Split tunneling' }),
});
