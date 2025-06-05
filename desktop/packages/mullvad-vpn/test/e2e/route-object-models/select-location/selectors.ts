import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  filterButton: () => page.getByRole('button', { name: 'Filter' }),
  filterChip: (label: string) => {
    return page.locator('button', { hasText: label });
  },
  expandAccordionButton: (label: string) => page.getByLabel(`Expand ${label}`),
  relaysMatching: (relayNames: string[]) =>
    page.getByRole('button', { name: new RegExp(relayNames.join('|')) }),
});
