import { type Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  applicationWarningDialogText: (applicationName: string) =>
    page.getByText(
      `If it’s already running, close ${applicationName} before launching it from here. Otherwise it might not be excluded from the VPN tunnel.`,
    ),
  findAnotherAppButton: () =>
    page.getByRole('button', {
      name: 'Find another app',
    }),
  searchInput: () => page.getByPlaceholder('Search for...'),
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
  linuxApplication: (applicationName: string) =>
    page.getByRole('button', {
      name: applicationName,
      exact: true,
    }),
  linuxApplications: () => page.getByTestId('linux-applications').getByRole('button'),
  linuxApplicationWarningDialogCancelButton: () =>
    page.getByRole('dialog').getByRole('button', {
      name: 'Cancel',
    }),
  linuxApplicationWarningDialogLaunchButton: () =>
    page.getByRole('dialog').getByRole('button', {
      name: 'Launch',
    }),
});
