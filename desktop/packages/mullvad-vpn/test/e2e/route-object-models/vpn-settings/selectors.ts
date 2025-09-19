import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  heading: () => page.getByRole('heading', { name: 'VPN settings' }),
  launchAppOnStartupSwitch: () => page.getByLabel('Launch app on start-up'),
  autoConnectSwitch: () => page.getByLabel('Auto-connect'),
  lanSwitch: () => page.getByLabel('Local network sharing'),
  wireguardSettingsButton: () => page.getByRole('button', { name: 'WireGuard settings' }),
});
