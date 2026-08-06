import { Locator, Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  multihopModeItems: () =>
    page.getByRole('listbox', {
      name: 'Mode',
    }),
  multihopModeItem: (itemsLocator: Locator, name: string) =>
    itemsLocator.getByRole('option', {
      name,
    }),
});
