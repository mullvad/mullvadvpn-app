import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  automaticOption: () => page.getByRole('option', { name: 'Automatic' }),
  fiveOneEightTwoZeroOption: () => page.getByRole('option', { name: '51820' }),
  fiveThreeOption: () => page.getByRole('option', { name: '53' }),
  customOption: () => page.getByRole('option', { name: 'Custom' }),
  customInput: () => page.getByPlaceholder('Port'),
});
