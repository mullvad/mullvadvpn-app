import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  accordion: (label: string) =>
    page.locator('button', {
      has: page.locator('div', { hasText: label }),
    }),
  applyButton: () => page.getByRole('button', { name: 'Apply' }),
  backButton: () => page.getByRole('button', { name: 'Back' }),
  ownershipOption: (label: string) =>
    page.locator('li', {
      has: page.locator('div', { hasText: label }),
    }),
  providersOption: (label: string) => page.getByRole('checkbox', { name: label }),
});

export type FilterSelectors = ReturnType<typeof createSelectors>;
