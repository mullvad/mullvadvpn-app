import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  automaticPortOption: () =>
    page
      .getByRole('listbox', { name: 'Port' })
      .getByRole('option', { name: 'Automatic', exact: true }),
  customPortOption: () =>
    page.getByRole('listbox', { name: 'Port' }).getByRole('option', { name: 'Custom' }),
  portInput: () => page.getByPlaceholder('Port'),
});
