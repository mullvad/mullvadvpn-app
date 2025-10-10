import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  settingsButton: () => page.locator('button[aria-label="Settings"]'),
  accountButton: () => page.locator('button[aria-label="Account settings"]'),
  selectLocationButton: () => page.getByLabel('Select location'),
  connectionPanelChevronButton: () => page.getByTestId('connection-panel-chevron'),
  inIpLabel: () => page.getByTestId('in-ip'),
  outIpLabels: () => page.getByTestId('out-ip'),
  featureIndicators: () => page.getByTestId('feature-indicator'),
  featureIndicator: (name: string) =>
    page.getByTestId('feature-indicator').filter({ hasText: name }),
  moreFeatureIndicator: () => page.getByText(/^\d more.../),
  relayHostname: () => page.getByTestId('hostname-line'),
});
