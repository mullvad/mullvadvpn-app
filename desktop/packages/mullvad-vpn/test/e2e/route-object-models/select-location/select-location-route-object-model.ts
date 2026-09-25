import { type Locator, Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SelectLocationRouteObjectModel {
  private readonly utils: TestUtils;
  private readonly selectors: ReturnType<typeof createSelectors>;
  private readonly page: Page;

  constructor(page: Page, util: TestUtils) {
    this.utils = util;
    this.page = page;
    this.selectors = createSelectors(page);
  }

  async expandAccordion(accordionName: string) {
    const accordionLocator = this.selectors.accordionButton(accordionName);
    const label = await accordionLocator.getAttribute('aria-label');
    if (label?.startsWith('Expand')) {
      await accordionLocator.click();
    }
  }

  async gotoEntryLocations() {
    await this.getEntryInput().click();
    await this.page.waitForFunction(() =>
      document.getAnimations().every((animation) => animation.playState === 'finished'),
    );
  }

  async gotoExitLocations() {
    await this.getExitInput().click();
    await this.page.waitForFunction(() =>
      document.getAnimations().every((animation) => animation.playState === 'finished'),
    );
  }

  getEntryInput() {
    return this.selectors.entryInput();
  }

  getExitInput() {
    return this.selectors.exitInput();
  }

  getLocationsMatching(relayNames: string[]) {
    return this.selectors.locationsMatching(relayNames);
  }

  getAutomaticLocation() {
    return this.selectors.automaticLocation();
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
