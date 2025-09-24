import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  portNumber: (port: number) => page.getByRole('option', { name: `${port}` }),
  automaticPortOption: () =>
    page
      .getByRole('listbox', { name: 'Port' })
      .getByRole('option', { name: 'Automatic', exact: true }),
});
