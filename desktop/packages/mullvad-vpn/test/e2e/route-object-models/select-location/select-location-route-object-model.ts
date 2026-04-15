import { type Locator, Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SelectLocationRouteObjectModel {
  private readonly utils: TestUtils;
  private readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: TestUtils) {
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async toggleAccordion(accordionName: string) {
    const expandAccordion = this.selectors.expandAccordionButton(accordionName);
    if ((await expandAccordion.count()) > 0) {
      await expandAccordion.click();
    }
  }

  getEntryButton() {
    return this.selectors.entryButton();
  }

  getExitButton() {
    return this.selectors.exitButton();
  }

  getSearchInput() {
    return this.selectors.searchInput();
  }

  getRelaysMatching(relayNames: string[]) {
    return this.selectors.relaysMatching(relayNames);
  }

  getFilterChip(label: string) {
    return this.selectors.filterChip(label);
  }

  getAllLocationsSection() {
    return this.selectors.allLocationsSection();
  }

  getRecentsSection() {
    return this.selectors.recentSection();
  }

  getLocationsInLocator(locator: Locator) {
    return this.selectors.locations(locator);
  }

  getLocationsInAllLocations() {
    const allLocationsSection = this.getAllLocationsSection();
    return this.getLocationsInLocator(allLocationsSection);
  }

  getLocationsInRecents() {
    const recentsSection = this.selectors.recentSection();
    return this.selectors.locations(recentsSection);
  }

  getRecentMenuButton(locationName: string) {
    const recentsSection = this.selectors.recentSection();
    return this.selectors.locationMenuButton(locationName, recentsSection);
  }

  getAddToCustomListButton(locationName: string, customListName: string) {
    return this.selectors.addToCustomListButton(locationName, customListName);
  }

  getAddToNewCustomListButton(locationName: string) {
    return this.selectors.addToNewCustomListButton(locationName);
  }

  getEditCustomListButton() {
    return this.selectors.editCustomListButton();
  }

  getDeleteCustomListButton() {
    return this.selectors.deleteCustomListButton();
  }

  async gotoFilter() {
    await this.selectors.selectLocationMenuButton().click();
    await this.selectors.filterMenuOption().click();
    await this.utils.expectRoute(RoutePath.filter);
  }
}
