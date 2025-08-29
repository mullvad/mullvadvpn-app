import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  portNumber: (port: number) => page.getByRole('option', { name: `${port}` }),
});
