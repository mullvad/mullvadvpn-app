import { type Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  splitTunnelingUnsupportedDialogOpenLink: () =>
    page.getByRole('button', {
      name: 'Click here to learn more',
    }),
  splitTunnelingUnsupportedDialogCloseButton: () =>
    page.getByRole('button', {
      name: 'Got it!',
    }),
  splitTunnelingUnsupportedDialogText: () =>
    page.getByText(
      'To use Split tunneling, please change to a Linux kernel version that supports cgroup v1.',
    ),
  linuxApplications: () => page.getByTestId('linux-applications').locator('button'),
});
