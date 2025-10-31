import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  heading: () => page.getByRole('heading', { name: 'VPN settings' }),
  launchAppOnStartupSwitch: () => page.getByRole('switch', { name: 'Launch app on start-up' }),
  autoConnectSwitch: () => page.getByRole('switch', { name: 'Auto-connect' }),
  lanSwitch: () => page.getByRole('switch', { name: 'Local network sharing' }),
  antiCensorship: () => page.getByRole('button', { name: 'Anti-censorship' }),
});
