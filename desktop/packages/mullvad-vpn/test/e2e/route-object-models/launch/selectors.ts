import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  defaultFooterText: () => page.getByText('Unable to contact the Mullvad system service'),
  learnMoreButton: () => page.getByRole('button', { name: 'Learn more' }),
  tryAgainButton: () => page.getByRole('button', { name: 'Try again' }),
  detailsButton: () => page.getByRole('button', { name: 'Details' }),
  gotoSystemSettingsButton: () => page.getByRole('button', { name: 'Go to system settings' }),
});
