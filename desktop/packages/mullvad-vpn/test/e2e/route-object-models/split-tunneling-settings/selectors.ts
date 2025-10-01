import { type Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  applicationWarningLaunchesElsewhereDialogText: (applicationName: string) =>
    page.getByText(`${applicationName} is problematic and can’t be excluded from the VPN tunnel.`),
  applicationWarningLaunchesInExistingProcessDialogText: (applicationName: string) =>
    page.getByText(
      `If it’s already running, close ${applicationName} before launching it from here. Otherwise it might not be excluded from the VPN tunnel.`,
    ),
  findAnotherAppButton: () =>
    page.getByRole('button', {
      name: 'Find another app',
    }),
  linuxApplication: (applicationName: string) =>
    page.getByRole('button', {
      name: applicationName,
      exact: true,
    }),
  linuxApplications: () => page.getByTestId('linux-applications').getByRole('button'),
  linuxApplicationWarningDialogBackButton: () =>
    page.getByRole('dialog').getByRole('button', {
      name: 'Back',
    }),
  linuxApplicationWarningDialogCancelButton: () =>
    page.getByRole('dialog').getByRole('button', {
      name: 'Cancel',
    }),
  linuxApplicationWarningDialogLaunchButton: () =>
    page.getByRole('dialog').getByRole('button', {
      name: 'Launch',
    }),
  searchInput: () => page.getByPlaceholder('Search for...'),
  splitTunnelingUnsupportedDialogCloseButton: () =>
    page.getByRole('button', {
      name: 'Got it!',
    }),
  splitTunnelingUnsupportedDialogOpenLink: () =>
    page.getByRole('button', {
      name: 'Click here to learn more',
    }),
  splitTunnelingUnsupportedDialogText: () =>
    page.getByText(
      'To use Split tunneling, please change to a Linux kernel version that supports cgroup v1.',
    ),
});
