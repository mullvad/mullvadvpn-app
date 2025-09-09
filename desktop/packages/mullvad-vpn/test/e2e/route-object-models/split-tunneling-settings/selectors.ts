import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  splitTunnelingUnsupportedDialogOpenLink: () =>
    page.locator('a', {
      hasText: 'Click here to learn more',
    }),
  splitTunnelingUnsupportedDialogCloseButton: () =>
    page.locator('button', {
      hasText: 'Got it!',
    }),
  linuxApplications: () => page.getByTestId('linux-applications').locator('button'),
});
