import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  settingsButton: () => page.locator('button[aria-label="Settings"]'),
  selectLocationButton: () => page.getByLabel('Select location'),
  connectionPanelChevronButton: () => page.getByTestId('connection-panel-chevron'),
  inIpLabel: () => page.getByTestId('in-ip'),
});
