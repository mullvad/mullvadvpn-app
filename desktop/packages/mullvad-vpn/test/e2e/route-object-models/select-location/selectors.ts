import { type Locator, Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  entryInput: () =>
    page.getByPlaceholder('Search entry location or server').filter({ visible: true }),
  entryAutomaticButton: (country: string) =>
    page.getByRole('button', {
      name: `Automatic (${country})`,
    }),
  entryAutomaticMultihopModeToAlwaysButton: () =>
    page.getByRole('button', {
      name: 'Set multihop to "Always"',
    }),
  exitInput: () =>
    page.getByPlaceholder('Search exit location or server').filter({ visible: true }),
  selectLocationMenuButton: () =>
    page.getByRole('button', { name: 'Open select location menu' }).filter({ visible: true }),
  entryFiltersOverrideDialogButton: (label: string) =>
    page.getByRole('dialog').getByRole('button', {
      name: label,
    }),
  filterMenuOption: () =>
    page.getByRole('link', { name: 'Filters', exact: true }).filter({ visible: true }),
  filterEntryButton: () =>
    page
      .getByRole('button', { name: 'Configure filters for entry location', exact: true })
      .filter({ visible: true }),
  filterEntryAutomaticButton: () =>
    page
      .getByRole('button', {
        name: 'Entry location filters are overridden due to use of Automatic entry location',
        exact: true,
      })
      .filter({ visible: true }),
  filterExitButton: () =>
    page
      .getByRole('button', { name: 'Configure filters for exit location', exact: true })
      .filter({ visible: true }),
  filterChipLink: (label: string) => {
    return page.locator('a', { hasText: label }).filter({ visible: true });
  },
  filterChipButton: (label: string) => {
    return page.locator('button', { hasText: label }).filter({ visible: true });
  },
  headerMenu: () =>
    page.getByRole('button', {
      name: 'Open select location menu',
    }),
  headerMenuItem: (label: string) =>
    page.getByRole('dialog').getByRole('button', {
      name: label,
    }),
  accordionButton: (label: string) => {
    const possiblePrefix = ['Expand', 'Collapse'];
    return page.getByLabel(new RegExp(`${possiblePrefix.join('|')} ${label}`));
  },
  automaticLocation: () => page.getByLabel('Use automatic entry location'),
  automaticLocationWarningDialogContinueButton: () =>
    page.getByRole('dialog').getByRole('button', {
      name: 'Continue',
    }),
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
  nonMatchingFilterErrorMessage: () =>
    page.getByText('Selection does not match filter settings').filter({ visible: true }),
});
