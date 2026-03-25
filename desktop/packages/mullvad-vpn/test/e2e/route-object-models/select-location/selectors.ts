import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  entryButton: () => page.getByRole('button', { name: 'Entry' }),
  exitButton: () => page.getByRole('button', { name: 'Exit' }),
  selectLocationMenuButton: () => page.getByRole('button', { name: 'Open select location menu' }),
  filterMenuOption: () => page.getByRole('button', { name: 'Filter' }),
  filterChip: (label: string) => {
    return page.locator('button', { hasText: label });
  },
  expandAccordionButton: (label: string) => page.getByLabel(`Expand ${label}`),
  relaysMatching: (relayNames: string[]) =>
    page.getByRole('button', { name: new RegExp(relayNames.join('|')) }),
  searchInput: () => page.getByPlaceholder('Search locations or servers'),
});
