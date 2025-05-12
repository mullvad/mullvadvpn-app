import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  filterButton: () => page.getByRole('button', { name: 'Filter' }),
  accordion: (label: string) =>
    page.locator('button', {
      has: page.locator('div', { hasText: label }),
    }),
  allProvidersCheckbox: () => page.getByLabel('All providers'),
  applyButton: () => page.getByRole('button', { name: 'Apply' }),
  backButton: () => page.getByRole('button', { name: 'Back' }),
  providerFilterChip: (providers: number) =>
    page.locator('button', { hasText: `Providers: ${providers}` }),
  ownerFilterChip: (owned: boolean) =>
    page.locator('button', { hasText: owned ? 'Owned' : 'Rented' }),
  option: (label: string) =>
    page.locator('button', {
      has: page.locator('div', { hasText: label }),
    }),
});
