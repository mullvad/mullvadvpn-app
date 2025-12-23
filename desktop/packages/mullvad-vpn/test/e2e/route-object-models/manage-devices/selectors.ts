import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  currentDeviceText: () => page.getByText('Current device'),
  deviceListItems: () => page.getByRole('list'),
  deviceListItem: (name: string) => page.getByRole('listitem').filter({ hasText: name }),
  removeDeviceButton: (name: string) => page.getByLabel(`Remove device named ${name}`),
  confirmRemoveDeviceButton: () => page.getByRole('button', { name: 'Remove', exact: true }),
});
