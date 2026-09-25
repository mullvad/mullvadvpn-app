import { type Locator, Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  entryInput: () =>
    page.getByPlaceholder('Search entry location or server').filter({ visible: true }),
  exitInput: () =>
    page.getByPlaceholder('Search exit location or server').filter({ visible: true }),
  selectLocationMenuButton: () => page.getByRole('button', { name: 'Open select location menu' }),
  filterMenuOption: () => page.getByRole('button', { name: 'Filter' }),
  filterChip: (label: string) => {
    return page.locator('button', { hasText: label }).filter({ visible: true });
  },
  accordionButton: (label: string) => {
    const possiblePrefix = ['Expand', 'Collapse'];
    return page.getByLabel(new RegExp(`${possiblePrefix.join('|')} ${label}`));
  },
  automaticLocation: () => page.getByLabel('Use automatic entry location'),
  locationsMatching: (relayNames: string[]) => {
    const possiblePrefix = ['Connect to', 'Use', 'Connect and use'];
    const possibleNames = possiblePrefix.flatMap((prefix) =>
      relayNames.map((name) => `${prefix} ${name}`),
    );

    return page.getByRole('button', {
      name: new RegExp(possibleNames.join('|')),
    });
  },
  searchInput: () => page.getByPlaceholder('Search locations or servers'),
  allLocationsSection: () => page.getByRole('region', { name: 'All locations' }),
  customListsSection: () => page.getByRole('region', { name: 'Custom lists' }),
  recentSection: () => page.getByRole('region', { name: 'Recents' }),
  locations: (locator?: Locator) => {
    const possiblePrefix = ['Connect to', 'Use', 'Connect and use'];
    return (locator ?? page).getByRole('button', {
      name: new RegExp(possiblePrefix.join('|')),
    });
  },
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
