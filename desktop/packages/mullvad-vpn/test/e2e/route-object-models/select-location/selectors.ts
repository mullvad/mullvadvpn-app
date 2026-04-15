import { type Locator, Page } from 'playwright';

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
    page.getByRole('button', {
      name: new RegExp(relayNames.map((name) => `Connect to ${name}`).join('|')),
    }),
  searchInput: () => page.getByPlaceholder('Search locations or servers'),
  allLocationsSection: () => page.getByRole('region', { name: 'All locations' }),
  customListsSection: () => page.getByRole('region', { name: 'Custom lists' }),
  recentSection: () => page.getByRole('region', { name: 'Recents' }),
  locations: (locator?: Locator) => (locator ?? page).getByRole('button', { name: 'Connect to' }),
  locationMenuButton: (locationName: string, locator?: Locator) => {
    return (locator ?? page).getByRole('button', { name: `Open menu for ${locationName}` });
  },
  addToCustomListButton: (locationName: string, customListName: string) =>
    page.getByRole('button', { name: `Add ${locationName} to ${customListName}` }),
  addToNewCustomListButton: (locationName: string) =>
    page.getByRole('button', { name: `Add ${locationName} to new list` }),
  editCustomListButton() {
    return page.getByRole('button', { name: 'Edit name' });
  },
  deleteCustomListButton() {
    return page.getByRole('button', { name: 'Delete' });
  },
});
